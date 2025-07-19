use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use anyhow::{Context, Result};
use tauri::{AppHandle, Emitter, State};
use tokio::{
    fs,
    sync::{
        Mutex,
        oneshot::{self, Sender},
    },
};

use crate::{
    mpv,
    settings::get_settings,
    sql,
    types::{AppState, Channel, CustomChannel, NetworkInfo},
    utils::{get_bin, serialize_to_file},
};

const WAN_IP_API: &str = "https://api.ipify.org";
const FFMPEG_BIN_NAME: &str = "ffmpeg";
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn start_ffmpeg_listening(channel: Channel, restream_dir: PathBuf) -> Result<Child> {
    let headers = sql::get_channel_headers_by_id(channel.id.context("no channel id")?)?;
    let playlist_dir = get_playlist_dir(restream_dir);
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
        .arg("-i")
        .arg(channel.url.context("no channel url")?)
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
        .arg("-reconnect")
        .arg("1")
        .arg("-reconnect_at_eof")
        .arg("1")
        .arg("-reconnect_streamed")
        .arg("1")
        .arg("-reconnect_on_network_error")
        .arg("1")
        .arg(playlist_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(child)
}

async fn start_web_server(
    restream_dir: PathBuf,
    port: u16,
) -> Result<(Sender<bool>, tokio::task::JoinHandle<()>)> {
    let file_server = warp::fs::dir(restream_dir);
    let (tx, rx) = oneshot::channel::<bool>();
    let (_, server) =
        warp::serve(file_server).bind_with_graceful_shutdown(([0, 0, 0, 0], port), async {
            rx.await.ok();
        });
    let handle = tokio::spawn(server);
    return Ok((tx, handle));
}

pub async fn start_restream(
    port: u16,
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    channel: Channel,
) -> Result<()> {
    let stop = state.lock().await.restream_stop_signal.clone();
    stop.store(false, std::sync::atomic::Ordering::Relaxed);
    let restream_dir = get_restream_folder()?;
    delete_old_segments(&restream_dir).await?;
    let mut ffmpeg_child = start_ffmpeg_listening(channel, restream_dir.clone())?;
    let (web_server_tx, web_server_handle) = start_web_server(restream_dir, port).await?;
    let _ = app.emit("restream_started", true);
    while !stop.load(std::sync::atomic::Ordering::Relaxed)
        && ffmpeg_child
            .try_wait()
            .map(|option| option.is_none())
            .unwrap_or(true)
        && !web_server_handle.is_finished()
    {
        tokio::time::sleep(Duration::from_millis(500)).await
    }
    let _ = ffmpeg_child.kill();
    let _ = web_server_tx.send(true);
    let _ = ffmpeg_child.wait();
    let _ = web_server_handle.await;
    Ok(())
}

pub async fn stop_restream(state: State<'_, Mutex<AppState>>) -> Result<()> {
    let state = state.lock().await;
    state
        .restream_stop_signal
        .store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

fn get_playlist_dir(mut folder: PathBuf) -> String {
    folder.push("stream.m3u8");
    folder.to_string_lossy().to_string()
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

async fn delete_old_segments(dir: &Path) -> Result<()> {
    fs::remove_dir_all(dir).await?;
    fs::create_dir_all(dir).await?;
    Ok(())
}

pub async fn watch_self(port: u16, state: State<'_, Mutex<AppState>>) -> Result<()> {
    let channel = Channel {
        url: Some(format!("http://127.0.0.1:{port}/stream.m3u8").to_string()),
        name: "Local livestream".to_string(),
        favorite: false,
        group: None,
        group_id: None,
        id: Some(-1),
        image: None,
        media_type: crate::media_type::LIVESTREAM,
        series_id: None,
        source_id: None,
        stream_id: None,
        tv_archive: None,
        season_id: None,
        episode_num: None,
    };
    mpv::play(channel, false, None, state).await
}

pub fn share_restream(address: String, channel: Channel, path: String) -> Result<()> {
    let channel = CustomChannel {
        headers: sql::get_channel_headers_by_id(channel.id.context("No id on channel?")?)?,
        data: Channel {
            id: Some(-1),
            name: format!("RST | {}", channel.name).to_string(),
            url: Some(address),
            group: None,
            image: channel.image,
            media_type: crate::media_type::LIVESTREAM,
            source_id: None,
            series_id: None,
            group_id: None,
            favorite: false,
            stream_id: None,
            tv_archive: None,
            season_id: None,
            episode_num: None,
        },
    };
    serialize_to_file(channel, path)
}

pub async fn get_network_info() -> Result<NetworkInfo> {
    let port = get_settings()?.restream_port.unwrap_or(3000);
    Ok(NetworkInfo {
        port,
        local_ips: get_ips(port)?,
        wan_ip: get_wan_ip(port).await?,
    })
}

fn get_ips(port: u16) -> Result<Vec<String>> {
    Ok(if_addrs::get_if_addrs()?
        .iter()
        .filter(|i| i.ip().is_ipv4() && !i.ip().is_loopback())
        .map(|i| format!("http://{}:{port}/stream.m3u8", i.ip().to_string()))
        .collect())
}

async fn get_wan_ip(port: u16) -> Result<String> {
    Ok(format!(
        "http://{}:{port}/stream.m3u8",
        reqwest::get(WAN_IP_API).await?.text().await?
    ))
}
