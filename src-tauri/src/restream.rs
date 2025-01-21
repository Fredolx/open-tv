use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use anyhow::{Context, Result};
use if_addrs::IfAddr;
use tauri::State;
use tokio::{
    fs,
    sync::{
        oneshot::{self, Sender},
        Mutex,
    },
};

use crate::{
    log::log,
    mpv,
    settings::get_settings,
    sql,
    types::{AppState, Channel, NetworkInfo},
};

const WAN_IP_API: &str = "https://api.ipify.org";

fn start_ffmpeg_listening(channel: Channel, restream_dir: PathBuf) -> Result<Child> {
    let headers = sql::get_channel_headers_by_id(channel.id.context("no channel id")?)?;
    let playlist_dir = get_playlist_dir(restream_dir);
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(child)
}

async fn start_web_server(
    restream_dir: PathBuf,
) -> Result<(Sender<bool>, tokio::task::JoinHandle<()>)> {
    let file_server = warp::fs::dir(restream_dir);
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
    let restream_dir = get_restream_folder()?;
    delete_old_segments(&restream_dir).await?;
    state.ffmpeg_child = Some(start_ffmpeg_listening(channel, restream_dir.clone())?);
    (state.web_server_tx, state.web_server_handle) = start_web_server(restream_dir)
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

pub async fn watch_self(port: u16) -> Result<()> {
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
    };
    mpv::play(channel, false).await
}

fn share_restream(address: String, channel: Channel) -> Result<()> {
    crate::share::share_custom_channel(Channel {
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
    })
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
