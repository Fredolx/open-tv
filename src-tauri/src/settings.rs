use std::collections::HashMap;

use anyhow::Result;

use crate::{sql, types::Settings};

pub const MPV_PARAMS: &str = "mpvParams";
pub const USE_STREAM_CACHING: &str = "useStreamingCaching";
pub const RECORDING_PATH: &str = "recordingPath";


fn get_settings() -> Result<Settings> {
    let map = sql::get_settings()?;
    let settings = Settings {
        MpvParams: map.get(MPV_PARAMS).map(|s| s.to_string()),
        RecordingPath: map.get(RECORDING_PATH).map(|s| s.to_string()),
        UseStreamCaching: map.get(USE_STREAM_CACHING).and_then(|s| s.parse().ok()),
    };
    Ok(settings)
}

fn update_settings(settings: Settings) -> Result<()> {
    let mut map: HashMap<String, String> = HashMap::with_capacity(3);
    if let Some(mpv_params) = settings.MpvParams {
        map.insert(MPV_PARAMS.to_string(), mpv_params);
    }
    if let Some(recording_path) = settings.RecordingPath {
        map.insert(RECORDING_PATH.to_string(), recording_path);
    }
    if let Some(use_stream_caching) = settings.UseStreamCaching{
        map.insert(USE_STREAM_CACHING.to_string(), use_stream_caching.to_string());
    }
    Ok(())
}