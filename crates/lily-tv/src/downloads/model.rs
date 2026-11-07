use std::collections::HashSet;

use gpui_kit::*;
use lily_core::{events::CoreEvent, mpv, types::Channel, utils};

use crate::backend::{Backend, EventBus};

#[derive(Clone, Debug, PartialEq)]
pub enum DownloadStatus {
    Running,
    Done,
    Cancelled,
    Failed(SharedString),
}

#[derive(Clone, Debug)]
pub struct Download {
    pub id: String,
    pub name: String,
    pub channel_id: Option<i64>,
    source_id: i64,
    /// 0 to 100; stays at 0 when the server doesn't report a size
    pub progress: f64,
    pub status: DownloadStatus,
}

/// Every download started this session. Emits [`DownloadsChanged`] on any change.
pub struct Downloads {
    items: Vec<Download>,
    _progress: Subscription,
}

pub struct DownloadsChanged;

impl EventEmitter<DownloadsChanged> for Downloads {}

struct GlobalDownloads(Entity<Downloads>);

impl Global for GlobalDownloads {}

impl Downloads {
    pub fn init(cx: &mut App) {
        let bus = Backend::bus(cx);
        let downloads = cx.new(|cx| Self {
            items: Vec::new(),
            _progress: cx.subscribe(&bus, |this: &mut Self, _: Entity<EventBus>, event: &CoreEvent, cx: &mut Context<Self>| {
                if let CoreEvent::DownloadProgress { download_id, progress } = event {
                    if let Some(item) = this.items.iter_mut().find(|d| &d.id == download_id) {
                        item.progress = *progress;
                        cx.emit(DownloadsChanged);
                        cx.notify();
                    }
                }
            }),
        });
        cx.set_global(GlobalDownloads(downloads));
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalDownloads>().0.clone()
    }

    pub fn items(&self) -> &[Download] {
        &self.items
    }

    /// Channels with a download still running, for the row menus.
    pub fn running_channel_ids(&self) -> HashSet<i64> {
        self.items
            .iter()
            .filter(|d| d.status == DownloadStatus::Running)
            .filter_map(|d| d.channel_id)
            .collect()
    }

    /// Starts downloading `channel`. `path` None saves to the recordings folder.
    pub fn start(&mut self, channel: Channel, download_id: String, path: Option<String>, cx: &mut Context<Self>) {
        if self.items.iter().any(|d| d.id == download_id && d.status == DownloadStatus::Running) {
            return;
        }
        let Some(source_id) = channel.source_id else {
            return;
        };
        self.items.retain(|d| d.id != download_id);
        self.items.push(Download {
            id: download_id.clone(),
            name: channel.name.clone(),
            channel_id: channel.id.filter(|id| *id >= 0),
            source_id,
            progress: 0.,
            status: DownloadStatus::Running,
        });
        cx.emit(DownloadsChanged);
        cx.notify();

        let id = download_id.clone();
        let request = Backend::run_with_events(cx, move |state, events| async move {
            utils::download(&state, events, channel, &id, path).await
        });
        cx.spawn(async move |this, cx| {
            let result = request.await;
            _ = this.update(cx, |this, cx| {
                if let Some(item) = this.items.iter_mut().find(|d| d.id == download_id) {
                    item.status = match result {
                        Ok(()) => {
                            item.progress = 100.;
                            DownloadStatus::Done
                        }
                        Err(e) if e.to_string() == "download aborted" => DownloadStatus::Cancelled,
                        Err(e) => DownloadStatus::Failed(format!("{e:#}").into()),
                    };
                }
                cx.emit(DownloadsChanged);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn cancel(&mut self, download_id: &str, cx: &mut Context<Self>) {
        let Some(item) = self.items.iter().find(|d| d.id == download_id) else {
            return;
        };
        let (source_id, id) = (item.source_id, item.id.clone());
        let request = Backend::run(cx, move |state| async move { mpv::cancel_play(source_id, id, &state).await });
        cx.spawn(async move |_, _| {
            if let Err(e) = request.await {
                lily_core::log::log(format!("cancel download: {e:?}"));
            }
        })
        .detach();
    }

    /// Cancels the running download for a channel, from its row menu.
    pub fn cancel_channel(&mut self, channel_id: i64, cx: &mut Context<Self>) {
        let ids: Vec<String> = self
            .items
            .iter()
            .filter(|d| d.channel_id == Some(channel_id) && d.status == DownloadStatus::Running)
            .map(|d| d.id.clone())
            .collect();
        for id in ids {
            self.cancel(&id, cx);
        }
    }

    /// Removes finished downloads from the list.
    pub fn clear_finished(&mut self, cx: &mut Context<Self>) {
        self.items.retain(|d| d.status == DownloadStatus::Running);
        cx.emit(DownloadsChanged);
        cx.notify();
    }
}
