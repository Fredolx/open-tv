use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::Duration,
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use anyhow::{Context, Result};
use tauri::State;
use tokio::sync::{
    broadcast,
    Mutex,
    oneshot::{self, Sender},
};
use warp::Filter;

use crate::{
    sql,
    types::{AppState, Channel, StreamInfo},
    utils::get_bin,
};

const FFMPEG_BIN_NAME: &str = "ffmpeg";
const DEFAULT_LOCAL_PLAYER_PORT: u16 = 9321;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn get_stream_info(channel: &Channel) -> Result<StreamInfo> {
    let url = channel.url.clone().context("no channel url")?;
    let channel_id = channel.id.context("no channel id")?;
    let headers = sql::get_channel_headers_by_id(channel_id)?;

    // Also check if the source has a stream_user_agent set
    let source_has_ua = channel
        .source_id
        .and_then(|sid| sql::get_source_from_id(sid).ok())
        .and_then(|s| s.stream_user_agent)
        .is_some();

    let has_custom_headers = source_has_ua
        || headers
            .as_ref()
            .map(|h| {
                h.referrer.is_some()
                    || h.user_agent.is_some()
                    || h.http_origin.is_some()
                    || h.ignore_ssl == Some(true)
            })
            .unwrap_or(false);
    let url_path = url.split('?').next().unwrap_or(&url);
    let mut is_hls = url_path.ends_with(".m3u8") || url_path.ends_with(".m3u");
    let mut resolved_url: Option<String> = None;

    // For non-HLS URLs without custom headers, follow redirects to check
    // if the final URL is actually an HLS stream (e.g. tvpass.org -> .m3u8)
    if !is_hls && !has_custom_headers {
        if let Ok(client) = crate::http::build_client() {
            if let Ok(resp) = client
                .head(&url)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                let final_url = resp.url().to_string();
                let fp = final_url.split('?').next().unwrap_or(&final_url);
                if fp.ends_with(".m3u8") || fp.ends_with(".m3u") {
                    is_hls = true;
                    resolved_url = Some(final_url);
                }
            }
        }
    }

    Ok(StreamInfo {
        url,
        has_custom_headers,
        is_hls,
        resolved_url,
    })
}

/// Start ffmpeg outputting raw MPEG-TS to stdout, returning the child process
/// and a broadcast sender that streams the TS data.
fn start_ffmpeg_streaming(
    channel: Channel,
    log_path: PathBuf,
) -> Result<(Child, broadcast::Sender<bytes::Bytes>)> {
    let headers = sql::get_channel_headers_by_id(channel.id.context("no channel id")?)?;
    let log_file = std::fs::File::create(&log_path).ok();
    eprintln!("[local_player] ffmpeg log: {:?}", log_path);

    let mut command = Command::new(get_bin(FFMPEG_BIN_NAME));
    if let Some(headers) = headers {
        if let Some(referrer) = headers.referrer {
            command.arg("-headers");
            command.arg(format!("Referer: {referrer}"));
        }
        if let Some(user_agent) = headers.user_agent {
            command.arg("-headers");
            command.arg(format!("User-Agent: {user_agent}"));
        }
        if let Some(origin) = headers.http_origin {
            command.arg("-headers");
            command.arg(format!("Origin: {origin}"));
        }
        if let Some(ignore_ssl) = headers.ignore_ssl {
            if ignore_ssl {
                command.arg("-tls_verify");
                command.arg("0");
            }
        }
    }
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let mut child = command
        .arg("-fflags")
        .arg("+genpts+discardcorrupt")
        .arg("-analyzeduration")
        .arg("1000000")
        .arg("-probesize")
        .arg("2000000")
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
        .arg(channel.url.context("no channel url")?)
        .arg("-c")
        .arg("copy")
        .arg("-f")
        .arg("mpegts")
        .arg("-mpegts_flags")
        .arg("resend_headers")
        .arg("-pat_period")
        .arg("0.1")
        .arg("-sdt_period")
        .arg("0.1")
        .arg("pipe:1")
        .stdout(Stdio::piped())
        .stderr(log_file.map(Stdio::from).unwrap_or_else(Stdio::null))
        .spawn()?;

    let stdout = child.stdout.take().context("no stdout from ffmpeg")?;
    let (tx, _) = broadcast::channel::<bytes::Bytes>(512);
    let tx_writer = tx.clone();

    // Blocking reader thread: reads ffmpeg stdout and broadcasts chunks.
    // Exits naturally when ffmpeg is killed (stdout closes).
    std::thread::spawn(move || {
        use std::io::Read;
        let mut stdout = stdout;
        let mut buf = vec![0u8; 65536];
        loop {
            match stdout.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let _ = tx_writer.send(bytes::Bytes::copy_from_slice(&buf[..n]));
                }
                Err(_) => break,
            }
        }
        eprintln!("[local_player] stdout reader exited");
    });

    Ok((child, tx))
}

/// Start a warp server that streams MPEG-TS data from the broadcast channel.
async fn start_streaming_server(
    tx: broadcast::Sender<bytes::Bytes>,
    port: u16,
) -> Result<(Sender<bool>, tokio::task::JoinHandle<()>)> {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "HEAD", "OPTIONS"])
        .allow_headers(vec!["Range", "Content-Type"]);

    let stream_route = warp::path("stream")
        .and(warp::get())
        .map(move || {
            let (mut body_tx, body) = hyper::Body::channel();
            let mut rx = tx.subscribe();
            tokio::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(chunk) => {
                            if body_tx.send_data(chunk).await.is_err() {
                                break; // Client disconnected
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            eprintln!("[local_player] stream consumer lagged, skipped {} chunks", n);
                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                eprintln!("[local_player] stream consumer exited");
            });
            warp::http::Response::builder()
                .header("Content-Type", "video/mp2t")
                .header("Cache-Control", "no-cache, no-store")
                .body(body)
                .unwrap()
        });

    let routes = stream_route.with(cors);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<bool>();
    let (_, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
            shutdown_rx.await.ok();
        });
    let handle = tokio::spawn(server);
    Ok((shutdown_tx, handle))
}

fn get_local_player_folder() -> Result<PathBuf> {
    let mut path = directories::ProjectDirs::from("dev", "fredol", "open-tv")
        .context("can't find project folder")?
        .cache_dir()
        .to_owned();
    path.push("local_player");
    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }
    Ok(path)
}

/// Kill any orphaned ffmpeg processes from previous app sessions.
fn kill_orphaned_ffmpeg() {
    #[cfg(unix)]
    {
        // Match ffmpeg processes outputting mpegts to pipe:1 (our signature)
        let _ = Command::new("pkill")
            .arg("-9")
            .arg("-f")
            .arg("ffmpeg.*-f mpegts.*pipe:1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        // Also kill any old-style ffmpegs writing to our cache directory
        if let Ok(cache_dir) = get_local_player_folder() {
            let pattern = cache_dir.join("stream.m3u8").to_string_lossy().to_string();
            let _ = Command::new("pkill")
                .arg("-9")
                .arg("-f")
                .arg(&pattern)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub async fn start_local_stream(
    channel: Channel,
    state: State<'_, Mutex<AppState>>,
) -> Result<String> {
    // Signal old monitor to stop, then await its full cleanup
    let (stop, old_monitor) = {
        let mut s = state.lock().await;
        let stop = s.local_player_stop.clone();
        let monitor = s.local_player_monitor.take();
        (stop, monitor)
    };
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    if let Some(handle) = old_monitor {
        let _ = tokio::time::timeout(Duration::from_secs(3), handle).await;
    }
    stop.store(false, std::sync::atomic::Ordering::Relaxed);

    kill_orphaned_ffmpeg();

    let cache_dir = get_local_player_folder()?;
    let log_path = cache_dir.join("ffmpeg.log");

    let port = DEFAULT_LOCAL_PLAYER_PORT;
    {
        let mut s = state.lock().await;
        s.local_player_port = Some(port);
    }

    let (mut ffmpeg_child, broadcast_tx) = start_ffmpeg_streaming(channel, log_path)?;
    eprintln!("[local_player] ffmpeg started, pid={}", ffmpeg_child.id());

    let (web_tx, web_handle) = start_streaming_server(broadcast_tx, port).await?;
    eprintln!("[local_player] streaming server started on port {}", port);

    let local_url = format!("http://127.0.0.1:{port}/stream");
    eprintln!("[local_player] returning url: {}", local_url);

    let monitor_handle = tokio::spawn(async move {
        while !stop.load(std::sync::atomic::Ordering::Relaxed)
            && ffmpeg_child
                .try_wait()
                .map(|opt| opt.is_none())
                .unwrap_or(true)
            && !web_handle.is_finished()
        {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        let stop_val = stop.load(std::sync::atomic::Ordering::Relaxed);
        let ffmpeg_exited = ffmpeg_child
            .try_wait()
            .map(|opt| opt.is_some())
            .unwrap_or(false);
        let web_finished = web_handle.is_finished();
        eprintln!(
            "[local_player] monitor exiting: stop={}, ffmpeg_exited={}, web_finished={}",
            stop_val, ffmpeg_exited, web_finished
        );
        let _ = ffmpeg_child.kill();
        let _ = web_tx.send(true);
        let _ = ffmpeg_child.wait();
        let _ = web_handle.await;
        eprintln!("[local_player] cleanup done");
    });
    {
        let mut s = state.lock().await;
        s.local_player_monitor = Some(monitor_handle);
    }

    Ok(local_url)
}

pub async fn stop_local_stream(state: State<'_, Mutex<AppState>>) -> Result<()> {
    let s = state.lock().await;
    s.local_player_stop
        .store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
