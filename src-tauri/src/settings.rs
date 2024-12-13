use std::collections::HashMap;

use anyhow::Result;

use crate::{sql, types::Settings};

pub const MPV_PARAMS: &str = "mpvParams";
pub const USE_STREAM_CACHING: &str = "useStreamingCaching";
pub const RECORDING_PATH: &str = "recordingPath";
pub const DEFAULT_VIEW: &str = "defaultView";
pub const VOLUME: &str = "volume";
pub const REFRESH_ON_START: &str = "refreshOnStart";

pub fn get_settings() -> Result<Settings> {
    let map = sql::get_settings()?;
    let settings = Settings {
        mpv_params: map.get(MPV_PARAMS).map(|s| s.to_string()),
        recording_path: map.get(RECORDING_PATH).map(|s| s.to_string()),
        use_stream_caching: map.get(USE_STREAM_CACHING).and_then(|s| s.parse().ok()),
        default_view: map.get(DEFAULT_VIEW).and_then(|s| s.parse().ok()),
        volume: map.get(VOLUME).and_then(|s| s.parse().ok()),
        refresh_on_start: map.get(REFRESH_ON_START).and_then(|s| s.parse().ok()),
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
    sql::update_settings(map)?;
    Ok(())
}
