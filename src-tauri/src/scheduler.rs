use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::{log, settings, utils};

/// Start background refresh timers based on current settings.
/// Spawns up to two tokio tasks (EPG + source refresh).
/// Returns stop signals so callers can cancel them.
pub fn start_scheduler(app: AppHandle) -> (Arc<AtomicBool>, Arc<AtomicBool>) {
    let epg_stop = Arc::new(AtomicBool::new(false));
    let source_stop = Arc::new(AtomicBool::new(false));

    let settings = match settings::get_settings() {
        Ok(s) => s,
        Err(e) => {
            log::log(format!("Scheduler: failed to read settings: {:?}", e));
            return (epg_stop, source_stop);
        }
    };

    // EPG auto-refresh
    if let Some(hours) = settings.epg_refresh_interval {
        if hours > 0 {
            let stop = epg_stop.clone();
            let app = app.clone();
            tokio::spawn(async move {
                let interval = Duration::from_secs(hours as u64 * 3600);
                loop {
                    tokio::time::sleep(interval).await;
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    log::log("Scheduler: auto-refreshing sources (EPG interval)".to_string());
                    match utils::refresh_all().await {
                        Ok(_) => {
                            let _ = app.emit("sources-refreshed", ());
                        }
                        Err(e) => {
                            log::log(format!("Scheduler: EPG refresh failed: {:?}", e));
                        }
                    }
                }
            });
        }
    }

    // Source auto-refresh
    if let Some(hours) = settings.source_refresh_interval {
        if hours > 0 {
            let stop = source_stop.clone();
            let app = app.clone();
            tokio::spawn(async move {
                let interval = Duration::from_secs(hours as u64 * 3600);
                loop {
                    tokio::time::sleep(interval).await;
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    log::log("Scheduler: auto-refreshing sources (source interval)".to_string());
                    match utils::refresh_all().await {
                        Ok(_) => {
                            let _ = app.emit("sources-refreshed", ());
                        }
                        Err(e) => {
                            log::log(format!("Scheduler: source refresh failed: {:?}", e));
                        }
                    }
                }
            });
        }
    }

    (epg_stop, source_stop)
}

/// Restart the scheduler. Signals old tasks to stop, starts new ones.
pub fn restart_scheduler(
    app: AppHandle,
    old_epg_stop: &Arc<AtomicBool>,
    old_source_stop: &Arc<AtomicBool>,
) -> (Arc<AtomicBool>, Arc<AtomicBool>) {
    old_epg_stop.store(true, Ordering::Relaxed);
    old_source_stop.store(true, Ordering::Relaxed);
    start_scheduler(app)
}
