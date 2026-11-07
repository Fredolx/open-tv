//! Bridge between GPUI's executor and lily-core, which is written against tokio.
//!
//! Every backend call runs on a tokio runtime owned by this global; GPUI tasks await
//! the returned `JoinHandle`, which is an ordinary future and needs no tokio context.

use std::{future::Future, sync::Arc};

use anyhow::{Result, anyhow};
use gpui_kit::{App, Global};
use lily_core::types::AppState;
use tokio::{runtime::Runtime, sync::Mutex};

pub type SharedState = Arc<Mutex<AppState>>;

pub struct Backend {
    runtime: Runtime,
    state: SharedState,
}

impl Global for Backend {}

impl Backend {
    pub fn init(cx: &mut App) {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("lily-core")
            .build()
            .expect("failed to start the backend runtime");
        cx.set_global(Self {
            runtime,
            state: Arc::new(Mutex::new(AppState::default())),
        });
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
