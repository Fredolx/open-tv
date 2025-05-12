use std::collections::HashMap;

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
        enable_tray_icon: map.get(ENABLE_TRAY_ICON).and_then(|s| s.parse().ok()),
        zoom: map.get(ZOOM).and_then(|s| s.parse().ok()),
        default_sort: map.get(DEFAULT_SORT).and_then(|s| s.parse().ok()),
        enable_hwdec: map.get(ENABLE_HWDEC).and_then(|s| s.parse().ok()),
        always_ask_save: map.get(ALWAYS_ASK_SAVE).and_then(|s| s.parse().ok()),
        enable_gpu: map.get(ENABLE_GPU).and_then(|s| s.parse().ok()),
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
