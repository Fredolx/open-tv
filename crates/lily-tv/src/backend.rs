//! Bridge between GPUI's executor and lily-core, which is written against tokio.
//!
//! Every backend call runs on a tokio runtime owned by this global; GPUI tasks await
//! the returned `JoinHandle`, which is an ordinary future and needs no tokio context.
//! Events the backend pushes (download progress, EPG reminders, restream state) are
//! re-emitted on the UI thread by the [`EventBus`] entity.

use std::{future::Future, sync::Arc};

use anyhow::{Result, anyhow};
use futures::{StreamExt as _, channel::mpsc};
use gpui_kit::{App, AppContext as _, Entity, EventEmitter, Global};
use lily_core::{
    events::{CoreEvent, EventSink},
    types::AppState,
};
use tokio::{runtime::Runtime, sync::Mutex};

pub type SharedState = Arc<Mutex<AppState>>;

/// Subscribe to this entity to receive backend events on the UI thread.
pub struct EventBus;

impl EventEmitter<CoreEvent> for EventBus {}

pub struct Backend {
    runtime: Runtime,
    state: SharedState,
    sink: EventSink,
    bus: Entity<EventBus>,
}

impl Global for Backend {}

impl Backend {
    pub fn init(cx: &mut App) {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("lily-core")
            .build()
            .expect("failed to start the backend runtime");

        let (tx, mut rx) = mpsc::unbounded::<CoreEvent>();
        let sink: EventSink = Arc::new(move |event| {
            // only fails once the app is shutting down
            _ = tx.unbounded_send(event);
        });
        let bus = cx.new(|_| EventBus);
        let forward = bus.clone();
        cx.spawn(async move |cx| {
            while let Some(event) = rx.next().await {
                forward.update(cx, |_, cx| cx.emit(event));
            }
        })
        .detach();

        cx.set_global(Self {
            runtime,
            state: Arc::new(Mutex::new(AppState::default())),
            sink,
            bus,
        });
    }

    pub fn bus(cx: &App) -> Entity<EventBus> {
        cx.global::<Self>().bus.clone()
    }

    /// Runs an async backend operation (network, mpv) on the tokio runtime.
    pub fn run<F, Fut, T>(cx: &App, op: F) -> impl Future<Output = Result<T>> + use<F, Fut, T>
    where
        F: FnOnce(SharedState) -> Fut,
        Fut: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let backend = cx.global::<Self>();
        let handle = backend.runtime.spawn(op(backend.state.clone()));
        async move { handle.await.map_err(|e| anyhow!("backend task failed: {e}"))? }
    }

    /// Like [`Self::run`], for operations that also report progress through events.
    pub fn run_with_events<F, Fut, T>(cx: &App, op: F) -> impl Future<Output = Result<T>> + use<F, Fut, T>
    where
        F: FnOnce(SharedState, EventSink) -> Fut,
        Fut: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let sink = cx.global::<Self>().sink.clone();
        Self::run(cx, move |state| op(state, sink))
    }

    /// Runs a blocking backend operation (SQLite, file parsing) off the UI thread.
    pub fn blocking<F, T>(cx: &App, op: F) -> impl Future<Output = Result<T>> + use<F, T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let handle = cx.global::<Self>().runtime.spawn_blocking(op);
        async move { handle.await.map_err(|e| anyhow!("backend task failed: {e}"))? }
    }
}
