use std::collections::HashMap;

use anyhow::Result;

use crate::{sql, types::Settings};

pub const MPV_PARAMS: &str = "mpvParams";
pub const USE_STREAM_CACHING: &str = "useStreamingCaching";
pub const RECORDING_PATH: &str = "recordingPath";

#[tauri::command(async)]
pub fn get_settings() -> Result<Settings> {
    let map = sql::get_settings()?;
    let settings = Settings {
        mpv_params: map.get(MPV_PARAMS).map(|s| s.to_string()),
        recording_path: map.get(RECORDING_PATH).map(|s| s.to_string()),
        use_stream_caching: map.get(USE_STREAM_CACHING).and_then(|s| s.parse().ok()),
    };
    Ok(settings)
}

#[tauri::command(async)]
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
    Ok(())
}
