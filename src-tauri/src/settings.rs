use std::{collections::HashMap, env::consts::OS};

use anyhow::{Context, Result};
use directories::UserDirs;

use crate::{sql, types::Settings};

pub const MPV_PARAMS: &str = "mpvParams";
pub const USE_STREAM_CACHING: &str = "useStreamingCaching";
pub const RECORDING_PATH: &str = "recordingPath";
pub const DEFAULT_VIEW: &str = "defaultView";
pub const VOLUME: &str = "volume";
pub const REFRESH_ON_START: &str = "refreshOnStart";
pub const RESTREAM_PORT: &str = "restreamPort";
pub const ENABLE_TRAY_ICON: &str = "enableTrayIcon";
pub const ZOOM: &str = "zoom";
pub const DEFAULT_SORT: &str = "defaultSort";
pub const ENABLE_HWDEC: &str = "enableHWDEC";
pub const ALWAYS_ASK_SAVE: &str = "alwaysAskSave";
pub const ENABLE_GPU: &str = "enableGPU";
pub const USE_SINGLE_COLUMN: &str = "useSingleColumn";
pub const MAX_TEXT_LINES: &str = "maxTextLines";
pub const COMPACT_MODE: &str = "compactMode";
pub const REFRESH_INTERVAL: &str = "refreshInterval";
pub const LAST_REFRESH: &str = "lastRefresh";
pub const ENHANCED_VIDEO: &str = "enhancedVideo";
pub const THEME: &str = "theme";

pub fn get_settings() -> Result<Settings> {
    let map = sql::get_settings()?;
    let settings = Settings {
        mpv_params: map.get(MPV_PARAMS).map(|s| s.to_string()),
        recording_path: map.get(RECORDING_PATH).map(|s| s.to_string()),
        use_stream_caching: map.get(USE_STREAM_CACHING).and_then(|s| s.parse().ok()),
        default_view: map.get(DEFAULT_VIEW).and_then(|s| s.parse().ok()),
        volume: map.get(VOLUME).and_then(|s| s.parse().ok()),
        refresh_on_start: map.get(REFRESH_ON_START).and_then(|s| s.parse().ok()),
        restream_port: map.get(RESTREAM_PORT).and_then(|s| s.parse().ok()),
        enable_tray_icon: if OS == "linux" {
            Some(false)
        } else {
            map.get(ENABLE_TRAY_ICON).and_then(|s| s.parse().ok())
        },
        zoom: map.get(ZOOM).and_then(|s| s.parse().ok()),
        default_sort: map.get(DEFAULT_SORT).and_then(|s| s.parse().ok()),
        enable_hwdec: map.get(ENABLE_HWDEC).and_then(|s| s.parse().ok()),
        always_ask_save: map.get(ALWAYS_ASK_SAVE).and_then(|s| s.parse().ok()),
        enable_gpu: map.get(ENABLE_GPU).and_then(|s| s.parse().ok()),
        use_single_column: map.get(USE_SINGLE_COLUMN).and_then(|s| s.parse().ok()),
        max_text_lines: map.get(MAX_TEXT_LINES).and_then(|s| s.parse().ok()),
        compact_mode: map.get(COMPACT_MODE).and_then(|s| s.parse().ok()),
        refresh_interval: map.get(REFRESH_INTERVAL).and_then(|s| s.parse().ok()),
        last_refresh: map.get(LAST_REFRESH).and_then(|s| s.parse().ok()),
        enhanced_video: map.get(ENHANCED_VIDEO).and_then(|s| s.parse().ok()),
        theme: map.get(THEME).and_then(|s| s.parse().ok()),
    };
    Ok(settings)
}

pub fn update_settings(settings: Settings) -> Result<()> {
    let mut map: HashMap<String, String> = HashMap::with_capacity(3);
    if let Some(mpv_params) = settings.mpv_params {
        map.insert(MPV_PARAMS.to_string(), mpv_params);
    }
    if let Some(recording_path) = settings.recording_path {
        map.insert(RECORDING_PATH.to_string(), recording_path);
    }
    if let Some(use_stream_caching) = settings.use_stream_caching {
        map.insert(
            USE_STREAM_CACHING.to_string(),
            use_stream_caching.to_string(),
        );
    }
    if let Some(default_view) = settings.default_view {
        map.insert(DEFAULT_VIEW.to_string(), default_view.to_string());
    }
    if let Some(volume) = settings.volume {
        map.insert(VOLUME.to_string(), volume.to_string());
    }
    if let Some(refresh_on_start) = settings.refresh_on_start {
        map.insert(REFRESH_ON_START.to_string(), refresh_on_start.to_string());
    }
    if let Some(port) = settings.restream_port {
        map.insert(RESTREAM_PORT.to_string(), port.to_string());
    }
    if let Some(enable_tray) = settings.enable_tray_icon {
        map.insert(ENABLE_TRAY_ICON.to_string(), enable_tray.to_string());
    }
    if let Some(zoom) = settings.zoom {
        map.insert(ZOOM.to_string(), zoom.to_string());
    }
    if let Some(sort) = settings.default_sort {
        map.insert(DEFAULT_SORT.to_string(), sort.to_string());
    }
    if let Some(hwdec) = settings.enable_hwdec {
        map.insert(ENABLE_HWDEC.to_string(), hwdec.to_string());
    }
    if let Some(save) = settings.always_ask_save {
        map.insert(ALWAYS_ASK_SAVE.to_string(), save.to_string());
    }
    if let Some(gpu) = settings.enable_gpu {
        map.insert(ENABLE_GPU.to_string(), gpu.to_string());
    }
    if let Some(single_col) = settings.use_single_column {
        map.insert(USE_SINGLE_COLUMN.to_string(), single_col.to_string());
    }
    if let Some(lines) = settings.max_text_lines {
        map.insert(MAX_TEXT_LINES.to_string(), lines.to_string());
    }
    if let Some(compact) = settings.compact_mode {
        map.insert(COMPACT_MODE.to_string(), compact.to_string());
    }
    // Update refresh interval setting
    if let Some(interval) = settings.refresh_interval {
        map.insert(REFRESH_INTERVAL.to_string(), interval.to_string());
    }
    // Update last refresh timestamp
    if let Some(last) = settings.last_refresh {
        map.insert(LAST_REFRESH.to_string(), last.to_string());
    }
    // Enhanced video mode
    if let Some(enhanced) = settings.enhanced_video {
        map.insert(ENHANCED_VIDEO.to_string(), enhanced.to_string());
    }
    // Theme
    if let Some(theme) = settings.theme {
        map.insert(THEME.to_string(), theme.to_string());
    }
    sql::update_settings(map)?;
    Ok(())
}

pub fn get_default_record_path() -> Result<String> {
    let user_dirs = UserDirs::new().context("Failed to get user dirs")?;
    let mut path = user_dirs
        .video_dir()
        .context("No videos dir in ~, please set a recording path in Settings")?
        .to_owned();
    path.push("open-tv");
    std::fs::create_dir_all(&path)?;
    Ok(path.to_string_lossy().to_string())
}
