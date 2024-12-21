use std::sync::{
    atomic::{AtomicBool, Ordering::Relaxed},
    Arc,
};

use anyhow::{Context, Result};
use chrono::Local;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::{log, types::EPGNotify, utils};

pub fn poll(mut to_watch: Vec<EPGNotify>, stop: Arc<AtomicBool>, app: AppHandle) -> Result<()> {
    while stop.load(Relaxed) && !to_watch.is_empty() {
        to_watch.retain(|epg| {
            let is_timestamp_over = match is_timestamp_over(&epg.start_timestamp) {
                Ok(v) => v,
                Err(e) => {
                    log::log(format!("{:?}", e));
                    return false;
                }
            };
            if is_timestamp_over {
                match notify(epg, &app).context("Failed to notify EPG") {
                    Ok(_) => {}
                    Err(e) => log::log(format!("{:?}", e)),
                }
                return false;
            }
            return true;
        });
    }
    Ok(())
}

fn notify(epg: &EPGNotify, app: &AppHandle) -> Result<()> {
    app.notification()
        .builder()
        .title(format!("LIVE: {}", epg.title))
        .body(format!("Watch on {}", epg.channel_name))
        .show()?;
    Ok(())
}

fn is_timestamp_over(timestamp: &str) -> Result<bool> {
    let time = utils::get_local_time(timestamp)?;
    let current_time = Local::now();
    Ok(current_time >= time)
}
