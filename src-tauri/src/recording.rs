use std::{
    io::{Read as _, Write},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;

use crate::{
    settings::{get_default_record_path, get_settings},
    sql,
    types::{ActiveRecording, AppState, Channel},
    utils::{get_bin, sanitize},
};

const FFMPEG_BIN_NAME: &str = "ffmpeg";
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordingInfo {
    pub recording_id: String,
    pub channel_id: i64,
    pub channel_name: String,
    pub channel_image: Option<String>,
    pub file_path: String,
    pub start_timestamp: u64,
}

fn make_recording_id(channel_id: i64) -> String {
    let epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("rec-{channel_id}-{epoch_ms}")
}

fn build_output_path(channel_name: &str) -> Result<String> {
    let settings = get_settings().ok();
    let dir = settings
        .and_then(|s| s.recording_path)
        .unwrap_or_else(|| get_default_record_path().unwrap_or_else(|_| "/tmp".to_string()));
    std::fs::create_dir_all(&dir)?;
    let sanitized = sanitize(channel_name.to_string());
    let timestamp = Local::now().format("%Y-%m-%d-%H-%M-%S");
    let filename = format!("{sanitized}_{timestamp}.mp4");
    let mut path = std::path::PathBuf::from(dir);
    path.push(filename);
    Ok(path.to_string_lossy().to_string())
}

fn spawn_recording_ffmpeg(channel: &Channel) -> Result<(Child, String)> {
    let url = channel.url.clone().context("no channel url")?;
    let channel_id = channel.id.context("no channel id")?;
    let output_path = build_output_path(&channel.name)?;

    let headers = sql::get_channel_headers_by_id(channel_id)?;
    let source_ua = channel
        .source_id
        .and_then(|sid| sql::get_source_from_id(sid).ok())
        .and_then(|s| s.stream_user_agent);

    let mut command = Command::new(get_bin(FFMPEG_BIN_NAME));

    if let Some(headers) = &headers {
        if let Some(referrer) = &headers.referrer {
            command.arg("-headers");
            command.arg(format!("Referer: {referrer}"));
        }
        let ua = headers.user_agent.as_ref().or(source_ua.as_ref());
        if let Some(user_agent) = ua {
            command.arg("-headers");
            command.arg(format!("User-Agent: {user_agent}"));
        }
        if let Some(origin) = &headers.http_origin {
            command.arg("-headers");
            command.arg(format!("Origin: {origin}"));
        }
        if let Some(true) = headers.ignore_ssl {
            command.arg("-tls_verify");
            command.arg("0");
        }
    } else if let Some(ua) = &source_ua {
        command.arg("-headers");
        command.arg(format!("User-Agent: {ua}"));
    }

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let child = command
        .arg("-fflags")
        .arg("+genpts+discardcorrupt")
        .arg("-analyzeduration")
        .arg("2000000")
        .arg("-probesize")
        .arg("5000000")
        .arg("-reconnect")
        .arg("1")
        .arg("-reconnect_at_eof")
        .arg("1")
        .arg("-reconnect_streamed")
        .arg("1")
        .arg("-reconnect_on_network_error")
        .arg("1")
        .arg("-reconnect_delay_max")
        .arg("5")
        .arg("-i")
        .arg(&url)
        .arg("-c")
        .arg("copy")
        .arg("-movflags")
        .arg("+faststart")
        .arg("-y")
        .arg(&output_path)
        .arg("-nostats")
        .arg("-loglevel")
        .arg("warning")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    Ok((child, output_path))
}

pub async fn start_recording(
    channel: Channel,
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
) -> Result<RecordingInfo> {
    let channel_id = channel.id.context("no channel id")?;
    let recording_id = make_recording_id(channel_id);

    let (child, file_path) = spawn_recording_ffmpeg(&channel)?;

    let start_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let info = RecordingInfo {
        recording_id: recording_id.clone(),
        channel_id,
        channel_name: channel.name.clone(),
        channel_image: channel.image.clone(),
        file_path,
        start_timestamp,
    };

    let stop_signal = Arc::new(AtomicBool::new(false));

    {
        let mut guard = state.lock().await;
        guard.active_recordings.insert(
            recording_id.clone(),
            ActiveRecording {
                info: info.clone(),
                stop_signal: stop_signal.clone(),
            },
        );
    }

    let _ = app.emit("recording-started", &info);

    // Spawn monitor task
    let monitor_id = recording_id.clone();
    let monitor_info = info.clone();
    let monitor_app = app.clone();
    tokio::spawn(async move {
        let state: tauri::State<'_, Mutex<AppState>> = monitor_app.state();
        monitor_recording(child, stop_signal, monitor_id, monitor_info, state, monitor_app.clone()).await;
    });

    Ok(info)
}

async fn monitor_recording(
    mut child: Child,
    stop_signal: Arc<AtomicBool>,
    recording_id: String,
    info: RecordingInfo,
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
) {
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        if stop_signal.load(Ordering::Relaxed) {
            // Graceful shutdown: write 'q' to ffmpeg's stdin so it finalizes the moov atom
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(b"q\n");
                let _ = stdin.flush();
                drop(stdin);
            }
            // Wait up to 5 seconds for ffmpeg to finalize and exit
            let mut exited = false;
            for _ in 0..10 {
                tokio::time::sleep(Duration::from_millis(500)).await;
                if let Ok(Some(_)) = child.try_wait() {
                    exited = true;
                    break;
                }
            }
            // Force kill only as a last resort
            if !exited {
                let _ = child.kill();
                let _ = child.wait();
            }
            break;
        }

        match child.try_wait() {
            Ok(Some(_)) => break, // process exited
            Ok(None) => continue, // still running
            Err(_) => break,
        }
    }

    // Log any ffmpeg warnings/errors after exit
    if let Some(mut stderr) = child.stderr.take() {
        let mut buf = String::new();
        if stderr.read_to_string(&mut buf).is_ok() && !buf.trim().is_empty() {
            let truncated = &buf[..buf.len().min(2000)];
            crate::log::log(format!("Recording ffmpeg output for {}: {}", info.channel_name, truncated));
        }
    }

    {
        let mut guard = state.lock().await;
        guard.active_recordings.remove(&recording_id);
    }

    let _ = app.emit("recording-stopped", &info);
}

pub async fn stop_recording(
    recording_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<String> {
    let guard = state.lock().await;
    let recording = guard
        .active_recordings
        .get(&recording_id)
        .context("recording not found")?;
    recording.stop_signal.store(true, Ordering::Relaxed);
    let path = recording.info.file_path.clone();
    Ok(path)
}

pub async fn get_active_recordings(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<RecordingInfo>> {
    let guard = state.lock().await;
    let recordings: Vec<RecordingInfo> = guard
        .active_recordings
        .values()
        .map(|r| r.info.clone())
        .collect();
    Ok(recordings)
}
