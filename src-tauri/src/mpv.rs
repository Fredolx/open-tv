use crate::settings::get_default_record_path;
use crate::types::{AppState, ChannelHttpHeaders, Source};
use crate::utils::{find_macos_bin, get_bin};
use crate::{log, sql};
use crate::{media_type, settings::get_settings, types::Channel};
use anyhow::{Context, Result};
use chrono::Local;

use std::sync::LazyLock;
use std::{env::consts::OS, path::Path, process::Stdio};
use tauri::State;
use tokio::sync::Mutex;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tokio_util::sync::CancellationToken;

const ARG_SAVE_POSITION_ON_QUIT: &str = "--save-position-on-quit";
const ARG_CACHE: &str = "--cache=";
const ARG_NO: &str = "no";
const ARG_RECORD: &str = "--stream-record=";
const ARG_TITLE: &str = "--title=";
const ARG_MSG_LEVEL: &str = "--msg-level=all=error";
const ARG_YTDLP_PATH: &str = "--script-opts=ytdl_hook-ytdl_path=";
const ARG_VOLUME: &str = "--volume=";
const ARG_HTTP_HEADERS: &str = "--http-header-fields=";
const ARG_USER_AGENT: &str = "--user-agent=";
const ARG_IGNORE_SSL: &str = "--ytdl-raw-options=no-check-certificates=True";
const ARG_PREFETCH_PLAYLIST: &str = "--prefetch-playlist=yes";
const ARG_LOOP_PLAYLIST: &str = "--loop-playlist=inf";
const ARG_HWDEC: &str = "--hwdec=auto";
const ARG_GPU_NEXT: &str = "--vo=gpu-next";
const ARG_GPU_PROFILE_HIGH_QUALITY: &str = "--profile=high-quality";
const ARG_NO_RESUME_PLAYBACK: &str = "--no-resume-playback";
const MPV_BIN_NAME: &str = "mpv";
const YTDLP_BIN_NAME: &str = "yt-dlp";
const HTTP_ORIGIN: &str = "origin:";
const HTTP_REFERRER: &str = "referer:";
static MPV_PATH: LazyLock<String> = LazyLock::new(|| get_bin(MPV_BIN_NAME));
static YTDLP_PATH: LazyLock<String> = LazyLock::new(|| find_macos_bin(YTDLP_BIN_NAME));

pub async fn play(
    channel: Channel,
    record: bool,
    record_path: Option<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<()> {
    eprintln!(
        "{} playing",
        channel.url.as_ref().context("no channel url")?
    );
    let source = channel
        .source_id
        .and_then(|id| {
            sql::get_source_from_id(id)
                .with_context(|| format!("failed to fetch source with id {}", id))
                .ok()
        })
        .or(None);
    let args = get_play_args(&channel, record, record_path, &source)?;
    eprintln!("with args: {:?}", args);

    if let Some(source) = source.as_ref() {
        _ = crate::utils::handle_max_streams(source, &state)
            .await
            .map_err(|e| log::log(format!("{:?}", e)));
    }

    let mut cmd = Command::new(MPV_PATH.clone())
        .args(args)
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let token = CancellationToken::new();
    let channel_id = channel.id.context("no channel id")?;
    if let Some(source_id) = source.as_ref().and_then(|s| s.id) {
        _ = crate::utils::insert_play_token(
            source_id,
            channel_id.to_string(),
            token.clone(),
            &state,
        )
        .await
        .map_err(|e| log::log(format!("{:?}", e)));
    }
    let result: Result<()> = tokio::select! {
        status = cmd.wait() => {
            let status = status?;
            if status.success() {
                Ok(())
            } else {
                let stdout = cmd.stdout.take();
                if stdout.is_none() {
                     Ok(())
                } else {
                    let stdout = stdout.context("no stdout")?;
                    let mut error: String = String::new();
                    let mut lines = BufReader::new(stdout).lines();
                    let mut first = true;
                    while let Some(line) = lines.next_line().await? {
                        error += &line;
                        if !first {
                            error += "\n";
                        } else {
                            first = false;
                        }
                    }
                    if error != "" {
                        Err(anyhow::anyhow!(error))
                    } else {
                        Err(anyhow::anyhow!("Mpv encountered an unknown error"))
                    }
                }
            }
        },
        _ = token.cancelled() => {
            cmd.kill().await?;
            Ok(())
        }
    };

    if let Some(source_id) = source.as_ref().and_then(|s| s.id) {
        _ = crate::utils::remove_from_play_stop(state, &source_id, &channel_id.to_string())
            .await
            .map_err(|e| log::log(format!("{:?}", e)));
    }
    result
}

pub async fn cancel_play(
    source_id: i64,
    key: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<()> {
    log::log(format!("Cancelling play for channel: {}", key));
    let token = crate::utils::remove_from_play_stop(state, &source_id, &key).await?;
    let token = token.context("no channel found")?;
    token.cancel();
    Ok(())
}

fn get_play_args(
    channel: &Channel,
    record: bool,
    record_path: Option<String>,
    source: &Option<Source>,
) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let settings = get_settings()?;
    let headers = sql::get_channel_headers_by_id(channel.id.context("no channel id?")?)?;
    args.push(channel.url.clone().context("no url")?);
    if channel.episode_num.is_some() {
        for url in sql::find_all_episodes_after(channel)? {
            args.push(url);
        }
        args.push(ARG_NO_RESUME_PLAYBACK.to_string());
    }
    if channel.media_type != media_type::LIVESTREAM {
        args.push(ARG_SAVE_POSITION_ON_QUIT.to_string());
    }
    if settings.use_stream_caching == Some(false) {
        let stream_caching_arg = format!("{ARG_CACHE}{ARG_NO}",);
        args.push(stream_caching_arg);
    }
    if settings.enable_hwdec.unwrap_or(true) {
        args.push(ARG_HWDEC.to_string());
    }
    if settings.enable_gpu.unwrap_or(false) {
        args.push(ARG_GPU_NEXT.to_string());
        args.push(ARG_GPU_PROFILE_HIGH_QUALITY.to_string());
    }
    if record {
        let path = if let Some(p) = record_path {
            p
        } else if let Some(p) = settings.recording_path.map(get_path) {
            p
        } else {
            get_path(get_default_record_path()?)
        };
        args.push(format!("{ARG_RECORD}{path}"));
    }
    if OS == "macos" && *MPV_PATH != MPV_BIN_NAME {
        args.push(format!("{}{}", ARG_YTDLP_PATH, *YTDLP_PATH));
    }
    args.push(format!("{}{}", ARG_TITLE, channel.name));
    args.push(ARG_MSG_LEVEL.to_string());
    if channel.media_type == media_type::LIVESTREAM {
        args.push(ARG_PREFETCH_PLAYLIST.to_string());
        args.push(ARG_LOOP_PLAYLIST.to_string());
    }
    if let Some(volume) = settings.volume {
        args.push(format!("{ARG_VOLUME}{volume}"));
    }
    if headers.is_some() || source.is_some() {
        set_headers(headers, &mut args, source);
    }
    if let Some(mpv_params) = settings.mpv_params {
        #[cfg(not(target_os = "windows"))]
        let mut params = shell_words::split(&mpv_params)?;
        #[cfg(target_os = "windows")]
        let mut params = winsplit::split(&mpv_params);
        args.append(&mut params);
    }
    Ok(args)
}

fn set_headers(
    headers: Option<ChannelHttpHeaders>,
    args: &mut Vec<String>,
    source: &Option<Source>,
) {
    let headers = headers.unwrap_or_default();
    let mut headers_vec: Vec<String> = Vec::with_capacity(2);
    if let Some(origin) = headers.http_origin {
        headers_vec.push(format!("{HTTP_ORIGIN}{origin}"));
    }
    if let Some(referrer) = headers.referrer {
        headers_vec.push(format!("{HTTP_REFERRER}{referrer}"));
    }
    if let Some(user_agent) = headers
        .user_agent
        .or_else(|| source.as_ref().and_then(|f| f.stream_user_agent.clone()))
    {
        args.push(format!("{ARG_USER_AGENT}{user_agent}"));
    }
    if let Some(ignore_ssl) = headers.ignore_ssl {
        if ignore_ssl == true {
            args.push(ARG_IGNORE_SSL.to_string());
        }
    }
    if headers_vec.len() > 0 {
        let headers = headers_vec.join(",");
        args.push(format!("{ARG_HTTP_HEADERS}{headers}"));
    }
}

fn get_path(path_str: String) -> String {
    let path = Path::new(&path_str);
    let path = path.join(get_file_name());
    return path.to_string_lossy().to_string();
}

fn get_file_name() -> String {
    let current_time = Local::now();
    let formatted_time = current_time.format("%Y-%m-%d-%H-%M-%S").to_string();
    format!("{formatted_time}.mp4")
}
