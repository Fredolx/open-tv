/*
 * Beats TV - Premium IPTV Player
 * Copyright (C) 2026 Beats TV Team
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * This project is a fork of Open TV by Fredolx.
 */

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

    let mut cmd = Command::new(MPV_PATH.clone());
    cmd.args(args)
        .stdout(Stdio::piped())
        .kill_on_drop(true);

    #[cfg(target_os = "windows")]
    // 0x08000000 is the CREATE_NO_WINDOW flag.
    // This is required to prevent MPV from opening a blank console window on Windows.
    // See: https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
    cmd.creation_flags(0x08000000);

    let mut cmd = cmd.spawn()?;
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
        // args.push(ARG_GPU_PROFILE_HIGH_QUALITY.to_string());
    }
    /* Enhanced Video Mode removed from auto-injection. User must populate mpv_params via preset. */
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
        eprintln!("Raw User MPV Params: {}", mpv_params);
        
        #[cfg(not(target_os = "windows"))]
        let mut params = shell_words::split(&mpv_params)?;
        
        #[cfg(target_os = "windows")]
        let mut params = winsplit::split(&mpv_params);

        eprintln!("Params before filter: {:?}", params);
        
        // Filter out unsupported options that might be saved in legacy settings
        params.retain(|arg| {
            let keep = !arg.contains("color-levels") && !arg.contains("protocol_whitelist");
            if !keep {
                eprintln!("Filtering out incompatible arg: {}", arg);
            }
            keep
        });
        
        eprintln!("Params after filter: {:?}", params);
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

pub fn get_stable_params() -> String {
    let mut args = Vec::new();
    
    // Aggressive Caching for Slow/VPN connections
    args.push("--cache=yes");
    args.push("--demuxer-max-bytes=1GiB");  
    args.push("--demuxer-max-back-bytes=0"); // DISABLING BACK-BUFFER (Stops the 3s repeat loop)
    args.push("--demuxer-readahead-secs=300");  
    args.push("--stream-buffer-size=16MiB");  
    args.push("--cache-secs=300");
    args.push("--cache-pause=yes");
    args.push("--cache-pause-wait=10"); // Massive 10s buffer lead after a drop
    
    // Performance optimizations
    args.push("--framedrop=vo");
    args.push("--vd-lavc-fast");
    args.push("--vd-lavc-skiploopfilter=all");
    args.push("--vd-lavc-threads=4"); 
    args.push("--demuxer-thread=yes");
    args.push("--hr-seek=no"); // Disable high-res seeking to prevent frame jitter
    
    // VPN-Optimized Reconnection & Network Tuning
    args.push("--stream-lavf-o-add=reconnect_streamed=1");
    args.push("--stream-lavf-o-add=reconnect_delay_max=5");
    args.push("--stream-lavf-o-add=reconnect_on_network_error=1");
    args.push("--stream-lavf-o-add=timeout=120000000");
    args.push("--stream-lavf-o-add=tcp_fastopen=1");
    args.push("--tls-verify=no"); 
    args.push("--cache-on-disk=no");
    
    // Efficiency
    args.push("--stream-lavf-o-add=protocol_whitelist=file,http,https,tcp,tls,crypto,rtp,udp");
    args.push("--demuxer-lavf-o-add=fflags=+nobuffer+discardcorrupt");
    
    args.push("--user-agent=\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36\"");
    
    args.push("--prefetch-playlist=yes");
    
    if OS == "windows" {
        args.push("--d3d11-flip=yes");
        args.push("--gpu-api=d3d11");
    } else if OS == "macos" {
        args.push("--gpu-api=opengl");
    }

    args.push("--msg-level=all=v");
    args.push("--log-file=\"C:/Users/admin-beats/OneDrive/xo Vibe Coding xo/iptvnator/open-tv/mpv_debug.log\"");

    args.join(" ")
}

pub fn get_enhanced_params() -> String {
    let mut args = Vec::new();
    
    // Pro-Level Rendering Engine
    args.push("--vo=gpu-next");
    args.push("--gpu-api=d3d11");
    args.push("--hwdec=d3d11va"); // Zero-copy hardware decoding for lower GPU load
    
    // High-Fidelity Scaling Shaders
    args.push("--scale=ewa_lanczossharp");
    args.push("--cscale=ewa_lanczossharp");
    args.push("--dscale=mitchell");
    args.push("--scale-antiring=0.7");
    args.push("--cscale-antiring=0.7");

    // Real-time De-banding (Anti-Banding)
    args.push("--deband=yes");
    args.push("--deband-iterations=4");
    args.push("--deband-threshold=48");
    args.push("--deband-range=16");
    args.push("--deband-grain=48");

    // Motion & Sync
    args.push("--video-sync=display-resample");
    args.push("--interpolation=yes");
    args.push("--tscale=linear");
    
    // Color Management
    args.push("--target-colorspace-hint=yes");
    // args.push("--color-levels=auto"); // Removed causing fatal error
    
    // Premium Caching (Higher readahead for 4K)
    args.push("--cache=yes");
    args.push("--demuxer-max-bytes=1GiB");
    args.push("--demuxer-readahead-secs=120"); 
    
    // Pro-Level Connection Tuning
    args.push("--stream-lavf-o-add=reconnect_streamed=1");
    args.push("--stream-lavf-o-add=reconnect_on_network_error=1");
    args.push("--stream-lavf-o-add=timeout=60000000");
    args.push("--stream-lavf-o-add=tcp_fastopen=1");
    // args.push("--stream-lavf-o-add=protocol_whitelist=file,http,https,tcp,tls,crypto,rtp,udp");
    
    args.push("--user-agent=\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36\"");
    
    if OS == "windows" {
        args.push("--d3d11-flip=yes");
        args.push("--gpu-api=d3d11");
    }

    args.push("--msg-level=all=v");
    args.push("--log-file=\"C:/Users/admin-beats/OneDrive/xo Vibe Coding xo/iptvnator/open-tv/mpv_debug.log\"");

    args.join(" ")
}

pub fn get_performance_params() -> String {
    let mut args = Vec::new();

    // Standard high-speed engine (GPU mode for compatibility)
    args.push("--vo=gpu");
    args.push("--gpu-api=d3d11");
    args.push("--hwdec=d3d11va"); 

    // Low-Cost Upscaling (Perceptually Crisp)
    args.push("--scale=spline36"); // Good balance, significantly lighter than Lanczos
    args.push("--cscale=bilinear");
    args.push("--dscale=mitchell");

    // Low-Cost Frame Smoothing (The "Soap Opera" effect on a budget)
    args.push("--video-sync=display-resample");
    args.push("--interpolation=yes");
    args.push("--tscale=oversample"); // Temporal oversampling is very cheap on GPU
    
    // Performance Hacks
    args.push("--opengl-pbo=yes"); // Faster texture uploads
    args.push("--sws-scaler=fast-bilinear");
    
    // Network Optimization
    args.push("--stream-lavf-o-add=reconnect_streamed=1");
    args.push("--stream-lavf-o-add=reconnect_delay_max=2");
    args.push("--stream-lavf-o-add=reconnect_on_network_error=1");
    args.push("--stream-lavf-o-add=timeout=30000000");
    args.push("--stream-lavf-o-add=tcp_fastopen=1");
    args.push("--cache=yes");
    args.push("--demuxer-max-bytes=256MiB");
    
    args.push("--user-agent=\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36\"");

    args.push("--msg-level=all=v");
    args.push("--log-file=\"C:/Users/admin-beats/OneDrive/xo Vibe Coding xo/iptvnator/open-tv/mpv_debug.log\"");

    args.join(" ")
}
