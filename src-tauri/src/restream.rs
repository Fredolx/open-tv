use std::{
    path::PathBuf,
    process::{Child, Command},
};

use anyhow::{Context, Result};
use tauri::State;
use tokio::sync::{
    oneshot::{self, Sender},
    Mutex,
};

use crate::{
    log::log,
    sql,
    types::{AppState, Channel},
};

fn start_ffmpeg_listening(channel: Channel) -> Result<Child> {
    let headers = sql::get_channel_headers_by_id(channel.id.context("no channel id")?)?;
    let mut playlist_dir = get_playlist_dir()?;
    let mut command = Command::new("ffmpeg");
    command
        .arg("-i")
        .arg(channel.url.context("no channel url")?);
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
    let child = command
        .arg("-c")
        .arg("copy")
        .arg("-f")
        .arg("hls")
        .arg("-hls_time")
        .arg("5")
        .arg("-hls_list_size")
        .arg("6")
        .arg("-hls_flags")
        .arg("delete_segments")
        .arg(playlist_dir)
        .spawn()?;
    Ok(child)
}

async fn start_web_server() -> Result<(Sender<bool>, tokio::task::JoinHandle<()>)> {
    let files_dir = get_restream_folder()?;
    let file_server = warp::fs::dir(files_dir);
    let (tx, rx) = oneshot::channel::<bool>();
    let (_, server) =
        warp::serve(file_server).bind_with_graceful_shutdown(([0, 0, 0, 0], 3000), async {
            rx.await.ok();
        });
    let handle = tokio::spawn(server);
    return Ok((tx, handle));
}

pub async fn start_restream(state: State<'_, Mutex<AppState>>, channel: Channel) -> Result<()> {
    let mut state = state.lock().await;
    state.ffmpeg_child = Some(start_ffmpeg_listening(channel)?);
    (state.web_server_tx, state.web_server_handle) = start_web_server()
        .await
        .map(|(tx, handle)| (Some(tx), Some(handle)))?;
    Ok(())
}

pub async fn stop_restream(state: State<'_, Mutex<AppState>>) -> Result<()> {
    let mut state = state.lock().await;
    let mut ffmpeg_child = state.ffmpeg_child.take().context("no ffmpeg child")?;
    let web_server_tx = state.web_server_tx.take().context("no web server tx")?;
    let web_server_handle = state
        .web_server_handle
        .take()
        .context("no web server handle")?;
    let _ = ffmpeg_child.kill();
    let _ = web_server_tx.send(true);
    let _ = ffmpeg_child.wait();
    let _ = web_server_handle
        .await
        .unwrap_or_else(|e| log(format!("{:?}", e)));
    Ok(())
}

fn get_playlist_dir() -> Result<String> {
    let mut restream_folder = get_restream_folder()?;
    restream_folder.push("stream.m3u8");
    Ok(restream_folder.to_string_lossy().to_string())
}

fn get_restream_folder() -> Result<PathBuf> {
    let mut path = directories::ProjectDirs::from("dev", "fredol", "open-tv")
        .context("can't find project folder")?
        .cache_dir()
        .to_owned();
    path.push("restream");
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    Ok(path)
}
