use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::Duration,
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use anyhow::{Context, Result};
use tauri::State;
use warp::Filter;
use tokio::{
    fs,
    sync::{
        Mutex,
        oneshot::{self, Sender},
    },
};

use crate::{
    sql,
    types::{AppState, Channel, StreamInfo},
    utils::get_bin,
};

const FFMPEG_BIN_NAME: &str = "ffmpeg";
const DEFAULT_LOCAL_PLAYER_PORT: u16 = 9321;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn get_stream_info(channel: &Channel) -> Result<StreamInfo> {
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
    let is_hls = url_path.ends_with(".m3u8") || url_path.ends_with(".m3u");
    Ok(StreamInfo {
        url,
        has_custom_headers,
        is_hls,
    })
}

fn start_ffmpeg(channel: Channel, cache_dir: PathBuf) -> Result<Child> {
    let headers = sql::get_channel_headers_by_id(channel.id.context("no channel id")?)?;
    let mut playlist_path = cache_dir;
    playlist_path.push("stream.m3u8");
    let playlist_str = playlist_path.to_string_lossy().to_string();
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
    let child = command
        .arg("-fflags")
        .arg("+genpts+discardcorrupt")
        .arg("-analyzeduration")
        .arg("2000000")
        .arg("-probesize")
        .arg("5000000")
        .arg("-i")
        .arg(channel.url.context("no channel url")?)
        .arg("-c")
        .arg("copy")
        .arg("-f")
        .arg("hls")
        .arg("-hls_time")
        .arg("2")
        .arg("-hls_list_size")
        .arg("10")
        .arg("-hls_flags")
        .arg("delete_segments+append_list")
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
        .arg(playlist_str)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(child)
}

async fn start_web_server(
    cache_dir: PathBuf,
    port: u16,
) -> Result<(Sender<bool>, tokio::task::JoinHandle<()>)> {
    let file_server = warp::fs::dir(cache_dir);
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "HEAD", "OPTIONS"])
        .allow_headers(vec!["Range", "Content-Type"]);
    let routes = file_server.with(cors);
    let (tx, rx) = oneshot::channel::<bool>();
    let (_, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
            rx.await.ok();
        });
    let handle = tokio::spawn(server);
    Ok((tx, handle))
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

async fn clean_cache(dir: &std::path::Path) -> Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir).await?;
    }
    fs::create_dir_all(dir).await?;
    Ok(())
}

pub async fn start_local_stream(
    channel: Channel,
    state: State<'_, Mutex<AppState>>,
) -> Result<String> {
    let stop = {
        let s = state.lock().await;
        s.local_player_stop.clone()
    };
    stop.store(false, std::sync::atomic::Ordering::Relaxed);

    let cache_dir = get_local_player_folder()?;
    clean_cache(&cache_dir).await?;

    let port = DEFAULT_LOCAL_PLAYER_PORT;
    {
        let mut s = state.lock().await;
        s.local_player_port = Some(port);
    }

    let mut ffmpeg_child = start_ffmpeg(channel, cache_dir.clone())?;
    let (web_tx, web_handle) = start_web_server(cache_dir, port).await?;

    let local_url = format!("http://127.0.0.1:{port}/stream.m3u8");

    tokio::spawn(async move {
        while !stop.load(std::sync::atomic::Ordering::Relaxed)
            && ffmpeg_child
                .try_wait()
                .map(|opt| opt.is_none())
                .unwrap_or(true)
            && !web_handle.is_finished()
        {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        let _ = ffmpeg_child.kill();
        let _ = web_tx.send(true);
        let _ = ffmpeg_child.wait();
        let _ = web_handle.await;
    });

    Ok(local_url)
}

pub async fn stop_local_stream(state: State<'_, Mutex<AppState>>) -> Result<()> {
    let s = state.lock().await;
    s.local_player_stop
        .store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
