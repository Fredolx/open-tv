use gpui_kit::*;
use lily_core::{events::CoreEvent, restream, types::Channel};

use crate::backend::{Backend, EventBus};

#[derive(Clone, Debug, PartialEq)]
pub enum RestreamStatus {
    Stopped,
    Starting,
    Running,
}

/// The single restream session; it outlives the dialog that started it.
pub struct Restream {
    channel: Option<Channel>,
    status: RestreamStatus,
    last_error: Option<SharedString>,
    _events: Subscription,
}

struct GlobalRestream(Entity<Restream>);

impl Global for GlobalRestream {}

impl Restream {
    pub fn init(cx: &mut App) {
        let bus = Backend::bus(cx);
        let restream = cx.new(|cx| Self {
            channel: None,
            status: RestreamStatus::Stopped,
            last_error: None,
            _events: cx.subscribe(&bus, |this: &mut Self, _: Entity<EventBus>, event: &CoreEvent, cx: &mut Context<Self>| {
                if matches!(event, CoreEvent::RestreamStarted) && this.status == RestreamStatus::Starting {
                    this.status = RestreamStatus::Running;
                    cx.notify();
                }
            }),
        });
        cx.set_global(GlobalRestream(restream));
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalRestream>().0.clone()
    }

    pub fn status(&self) -> &RestreamStatus {
        &self.status
    }

    pub fn channel(&self) -> Option<&Channel> {
        self.channel.as_ref()
    }

    pub fn last_error(&self) -> Option<&SharedString> {
        self.last_error.as_ref()
    }

    /// Starts re-streaming `channel`; the session ends on [`Self::stop`] or if the stream dies.
    pub fn start(&mut self, channel: Channel, port: u16, cx: &mut Context<Self>) {
        if self.status != RestreamStatus::Stopped {
            return;
        }
        self.channel = Some(channel.clone());
        self.status = RestreamStatus::Starting;
        self.last_error = None;
        cx.notify();
        let session = Backend::run_with_events(cx, move |state, events| async move {
            restream::start_restream(port, &state, events, channel).await
        });
        cx.spawn(async move |this, cx| {
            let result = session.await;
            _ = this.update(cx, |this, cx| {
                this.status = RestreamStatus::Stopped;
                if let Err(e) = result {
                    this.last_error = Some(format!("{e:#}").into());
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn stop(&mut self, cx: &mut Context<Self>) {
        let request = Backend::run(cx, |state| async move { restream::stop_restream(&state).await });
        cx.spawn(async move |_, _| {
            if let Err(e) = request.await {
                lily_core::log::log(format!("stop restream: {e:?}"));
            }
        })
        .detach();
    }
}
