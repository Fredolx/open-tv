use std::sync::Arc;

/// Events the backend pushes to the UI (these were Tauri `emit`s / notifications).
#[derive(Clone, Debug)]
pub enum CoreEvent {
    DownloadProgress { download_id: String, progress: f64 },
    RestreamStarted,
    EpgLive { title: String, channel_name: String },
}

pub type EventSink = Arc<dyn Fn(CoreEvent) + Send + Sync>;
