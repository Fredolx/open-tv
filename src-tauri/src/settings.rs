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
pub const PERFORMANCE_MODE: &str = "performanceMode";

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
    let mut map: HashMap<String, String> = HashMap::with_capacity(20);
    
    macro_rules! insert_if_some {
        ($key:expr, $value:expr) => {
            if let Some(v) = $value {
                map.insert($key.to_string(), v.to_string());
            }
        };
    }
    
    insert_if_some!(MPV_PARAMS, settings.mpv_params);
    insert_if_some!(RECORDING_PATH, settings.recording_path);
    insert_if_some!(USE_STREAM_CACHING, settings.use_stream_caching);
    insert_if_some!(DEFAULT_VIEW, settings.default_view);
    insert_if_some!(VOLUME, settings.volume);
    insert_if_some!(REFRESH_ON_START, settings.refresh_on_start);
    insert_if_some!(RESTREAM_PORT, settings.restream_port);
    insert_if_some!(ENABLE_TRAY_ICON, settings.enable_tray_icon);
    insert_if_some!(ZOOM, settings.zoom);
    insert_if_some!(DEFAULT_SORT, settings.default_sort);
    insert_if_some!(ENABLE_HWDEC, settings.enable_hwdec);
    insert_if_some!(ALWAYS_ASK_SAVE, settings.always_ask_save);
    insert_if_some!(ENABLE_GPU, settings.enable_gpu);
    insert_if_some!(USE_SINGLE_COLUMN, settings.use_single_column);
    insert_if_some!(MAX_TEXT_LINES, settings.max_text_lines);
    insert_if_some!(COMPACT_MODE, settings.compact_mode);
    insert_if_some!(REFRESH_INTERVAL, settings.refresh_interval);
    insert_if_some!(LAST_REFRESH, settings.last_refresh);
    insert_if_some!(ENHANCED_VIDEO, settings.enhanced_video);
    insert_if_some!(THEME, settings.theme);
    
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
