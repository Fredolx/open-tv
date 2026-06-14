use crate::settings::{get_default_record_path, get_settings};
use crate::types::{AppState, ChannelHttpHeaders, Settings, Source};
use crate::utils::{find_macos_bin, get_bin};
use crate::{log, sql};
use crate::{media_type, types::Channel};
use anyhow::{Context, Result};
use chrono::Local;

use std::sync::LazyLock;
use std::{env::consts::OS, path::Path, process::Stdio};
use tauri::State;
use tokio::io::AsyncRead;
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

const VLC_ARG_META_TITLE: &str = "--meta-title=";
const VLC_ARG_NETWORK_CACHING: &str = "--network-caching=";
const VLC_ARG_AVCODEC_HW: &str = "--avcodec-hw=";
const VLC_ARG_HTTP_USER_AGENT: &str = "--http-user-agent=";
const VLC_ARG_HTTP_REFERRER: &str = "--http-referrer=";
const VLC_ARG_GAIN: &str = "--gain=";
const VLC_ARG_SOUT: &str = "--sout=";
const VLC_ARG_PLAY_AND_EXIT: &str = "--play-and-exit";
const VLC_ARG_LOOP: &str = "--loop";
const VLC_ARG_QUIET: &str = "--quiet";
const VLC_ARG_NO_VIDEO_TITLE_SHOW: &str = "--no-video-title-show";
const VLC_HW_ANY: &str = "any";
const VLC_HW_NONE: &str = "none";
const VLC_NETWORK_CACHE_DISABLED: &str = "0";

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
    let source = channel.source_id.and_then(|id| {
        sql::get_source_from_id(id)
            .with_context(|| format!("failed to fetch source with id {}", id))
            .ok()
    });

    let settings = get_settings()?;
    let mut player = settings.player.clone().unwrap_or(MPV_PATH.to_string());
    let player_path_exists = Path::new(&player).exists();
    if player == "mpv" || !player_path_exists {
        player = MPV_PATH.to_string();
    }
    eprintln!("using player: {}", player);
    let is_vlc =
        player.to_lowercase().ends_with("vlc") || player.to_lowercase().ends_with("vlc.exe");
    let args = match is_vlc {
        true => get_vlc_args(&channel, record, record_path, &source, &settings)?,
        _ => get_mpv_args(&channel, record, record_path, &source, &settings)?,
    };
    eprintln!("with args: {:?}", args);

    if let Some(source) = source.as_ref() {
        _ = crate::utils::handle_max_streams(source, &state)
            .await
            .map_err(|e| log::log(format!("{:?}", e)));
    }

    let mut cmd = Command::new(&player)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
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
                let mut error = String::new();
                if let Some(stderr) = cmd.stderr.take() {
                    append_lines(stderr, &mut error).await?;
                }
                if error.is_empty() {
                    if let Some(stdout) = cmd.stdout.take() {
                        append_lines(stdout, &mut error).await?;
                    }
                }
                if !error.is_empty() {
                    Err(anyhow::anyhow!(error))
                } else {
                    Err(anyhow::anyhow!(
                        "{} encountered an unknown error",
                        player
                    ))
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

async fn append_lines<R: AsyncRead + Unpin>(reader: R, buf: &mut String) -> Result<()> {
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await? {
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(&line);
    }
    Ok(())
}

fn get_mpv_args(
    channel: &Channel,
    record: bool,
    record_path: Option<String>,
    source: &Option<Source>,
    settings: &Settings,
) -> Result<Vec<String>> {
    let mut args = Vec::new();
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
        args.push(format!("{ARG_CACHE}{ARG_NO}"));
    }
    if settings.enable_hwdec.unwrap_or(true) {
        args.push(ARG_HWDEC.to_string());
    }
    if settings.enable_gpu.unwrap_or(false) {
        args.push(ARG_GPU_NEXT.to_string());
        args.push(ARG_GPU_PROFILE_HIGH_QUALITY.to_string());
    }
    if record {
        args.push(format!(
            "{ARG_RECORD}{}",
            resolve_record_path(record_path, settings)?
        ));
    }
    if OS == "macos" {
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
        set_mpv_headers(headers, &mut args, source);
    }
    if let Some(mpv_params) = settings.mpv_params.as_ref() {
        let mut params = split_params(mpv_params)?;
        args.append(&mut params);
    }
    Ok(args)
}

fn get_vlc_args(
    channel: &Channel,
    record: bool,
    record_path: Option<String>,
    source: &Option<Source>,
    settings: &Settings,
) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let headers = sql::get_channel_headers_by_id(channel.id.context("no channel id?")?)?;
    let is_livestream = channel.media_type == media_type::LIVESTREAM;

    args.push(channel.url.clone().context("no url")?);
    if channel.episode_num.is_some() {
        for url in sql::find_all_episodes_after(channel)? {
            args.push(url);
        }
    }

    if is_livestream {
        args.push(VLC_ARG_LOOP.to_string());
    } else {
        args.push(VLC_ARG_PLAY_AND_EXIT.to_string());
    }

    if settings.use_stream_caching == Some(false) {
        args.push(format!(
            "{VLC_ARG_NETWORK_CACHING}{VLC_NETWORK_CACHE_DISABLED}"
        ));
    }

    args.push(format!(
        "{VLC_ARG_AVCODEC_HW}{}",
        if settings.enable_hwdec.unwrap_or(true) {
            VLC_HW_ANY
        } else {
            VLC_HW_NONE
        }
    ));

    if record {
        let path = resolve_record_path(record_path, settings)?
            .replace('\\', "/")
            .replace('\'', "\\'");
        args.push(format!(
            "{VLC_ARG_SOUT}#duplicate{{dst=display,dst=std{{access=file,mux=mp4,dst='{path}'}}}}"
        ));
    }

    args.push(format!("{VLC_ARG_META_TITLE}{}", channel.name));
    args.push(VLC_ARG_NO_VIDEO_TITLE_SHOW.to_string());
    args.push(VLC_ARG_QUIET.to_string());

    if let Some(volume) = settings.volume {
        args.push(format!("{VLC_ARG_GAIN}{:.2}", volume as f32 / 100.0));
    }

    set_vlc_headers(headers, &mut args, source);

    Ok(args)
}

fn set_mpv_headers(
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
    if let Some(true) = headers.ignore_ssl {
        args.push(ARG_IGNORE_SSL.to_string());
    }
    if !headers_vec.is_empty() {
        args.push(format!("{ARG_HTTP_HEADERS}{}", headers_vec.join(",")));
    }
}

fn set_vlc_headers(
    headers: Option<ChannelHttpHeaders>,
    args: &mut Vec<String>,
    source: &Option<Source>,
) {
    let headers = headers.unwrap_or_default();
    if let Some(referrer) = headers.referrer {
        args.push(format!("{VLC_ARG_HTTP_REFERRER}{referrer}"));
    }
    if let Some(user_agent) = headers
        .user_agent
        .or_else(|| source.as_ref().and_then(|f| f.stream_user_agent.clone()))
    {
        args.push(format!("{VLC_ARG_HTTP_USER_AGENT}{user_agent}"));
    }
}

fn split_params(params: &str) -> Result<Vec<String>> {
    #[cfg(not(target_os = "windows"))]
    {
        Ok(shell_words::split(params)?)
    }
    #[cfg(target_os = "windows")]
    {
        Ok(winsplit::split(params))
    }
}

fn resolve_record_path(record_path: Option<String>, settings: &Settings) -> Result<String> {
    if let Some(p) = record_path {
        return Ok(p);
    }
    let dir = match settings.recording_path.clone() {
        Some(p) => p,
        None => get_default_record_path()?,
    };
    Ok(get_path(dir))
}

fn get_path(path_str: String) -> String {
    let path = Path::new(&path_str);
    let path = path.join(get_file_name());
    path.to_string_lossy().to_string()
}

fn get_file_name() -> String {
    let current_time = Local::now();
    let formatted_time = current_time.format("%Y-%m-%d-%H-%M-%S").to_string();
    format!("{formatted_time}.mp4")
}
