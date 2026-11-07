use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, IconName, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    notification::Notification,
    radio::RadioGroup,
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{
    restream::{self, get_network_info},
    types::{Channel, NetworkInfo},
};

use super::model::{Restream, RestreamStatus};
use crate::{
    backend::Backend,
    file_prompts,
    naming::{CHANNEL_EXTENSION, sanitize_file_name},
};

/// Dialog content for re-streaming a live channel to other devices on the network.
pub struct RestreamPanel {
    channel: Channel,
    restream: Entity<Restream>,
    network: Option<NetworkInfo>,
    network_error: Option<SharedString>,
    /// Index into the local addresses, or one past them for the internet address
    address: usize,
    watching: bool,
    _observe: Subscription,
}

impl RestreamPanel {
    pub fn new(channel: Channel, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let restream = Restream::global(cx);
        let request = Backend::run(cx, |_| get_network_info());
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update(cx, |this, cx| {
                match result {
                    Ok(info) => this.network = Some(info),
                    Err(e) => this.network_error = Some(format!("Couldn't read network addresses: {e:#}").into()),
                }
                cx.notify();
            });
        })
        .detach();
        Self {
            channel,
            _observe: cx.observe(&restream, |_, _, cx| cx.notify()),
            restream,
            network: None,
            network_error: None,
            address: 0,
            watching: false,
        }
    }

    fn addresses(&self) -> Vec<String> {
        let Some(info) = &self.network else {
            return Vec::new();
        };
        let mut all = info.local_ips.clone();
        all.push(info.wan_ip.clone());
        all
    }

    fn selected_address(&self) -> Option<String> {
        self.addresses().get(self.address).cloned()
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        let port = self.network.as_ref().map(|n| n.port).unwrap_or(3000);
        let channel = self.channel.clone();
        self.restream.update(cx, |r, cx| r.start(channel, port, cx));
    }

    fn watch_here(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(port) = self.network.as_ref().map(|n| n.port) else {
            return;
        };
        self.watching = true;
        cx.notify();
        let request = Backend::run(cx, move |state| async move { restream::watch_self(port, &state).await });
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                this.watching = false;
                if let Err(e) = result {
                    window.push_notification(Notification::error(format!("{e:#}")).title("Couldn't play the re-stream"), cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn share(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(address) = self.selected_address() else {
            return;
        };
        let channel = self.channel.clone();
        let name = format!("{}_rst.{CHANNEL_EXTENSION}", sanitize_file_name(&channel.name));
        let path = file_prompts::save_file(cx, None, &name);
        cx.spawn_in(window, async move |this, cx| {
            let Some(path) = path.await else {
                return;
            };
            let export_path = path.clone();
            let request = this.update(cx, |_, cx| {
                Backend::blocking(cx, move || restream::share_restream(address, channel, export_path))
            });
            let Ok(request) = request else {
                return;
            };
            let result = request.await;
            _ = this.update_in(cx, |_, window, cx| {
                let note = match result {
                    Ok(()) => Notification::success(format!("Exported to {path}")),
                    Err(e) => Notification::error(format!("{e:#}")).title("Couldn't export the re-stream"),
                };
                window.push_notification(note, cx);
            });
        })
        .detach();
    }
}

impl Render for RestreamPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let restream = self.restream.read(cx);
        let status = restream.status().clone();
        let other_channel = restream
            .channel()
            .filter(|c| status != RestreamStatus::Stopped && c.id != self.channel.id)
            .map(|c| c.name.clone());
        let last_error = restream.last_error().cloned();
        let ours = status != RestreamStatus::Stopped && other_channel.is_none();
        let addresses = self.addresses();
        let wan_index = addresses.len().saturating_sub(1);
        let port = self.network.as_ref().map(|n| n.port);

        v_flex()
            .gap_3()
            .child(
                div().text_color(cx.theme().muted_foreground).child(
                    "Re-streaming relays this channel through Lily TV so other devices can play it from the address below.",
                ),
            )
            .when_some(other_channel, |el, name| {
                el.child(div().text_color(cx.theme().warning).child(format!(
                    "Already re-streaming “{name}”. Stop it first to re-stream this channel."
                )))
            })
            .when_some(self.network_error.clone(), |el, e| el.child(div().text_color(cx.theme().danger).child(e)))
            .when_some(last_error.filter(|_| status == RestreamStatus::Stopped), |el, e| {
                el.child(div().text_color(cx.theme().danger).child(format!("The last re-stream stopped: {e}")))
            })
            .when(!addresses.is_empty(), |el| {
                el.child(
                    v_flex()
                        .gap_1()
                        .child(div().text_sm().child("Address to share"))
                        .child(
                            RadioGroup::vertical("restream-address")
                                .children(addresses.iter().enumerate().map(|(i, a)| {
                                    if i == wan_index {
                                        format!("{a}  (internet; needs port {} forwarded)", port.unwrap_or(3000))
                                    } else {
                                        a.clone()
                                    }
                                }))
                                .selected_index(Some(self.address))
                                .on_change(cx.listener(|this, ix: &usize, _, cx| {
                                    this.address = *ix;
                                    cx.notify();
                                })),
                        ),
                )
            })
            .child(
                h_flex()
                    .gap_2()
                    .child(match status {
                        RestreamStatus::Stopped => Button::new("restream-start")
                            .primary()
                            .icon(IconName::Play)
                            .label("Start re-stream")
                            .disabled(self.network.is_none())
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.start(cx))),
                        RestreamStatus::Starting => Button::new("restream-start")
                            .primary()
                            .label("Starting…")
                            .loading(true)
                            .disabled(true),
                        RestreamStatus::Running => Button::new("restream-stop")
                            .danger()
                            .icon(IconName::Square)
                            .label("Stop re-stream")
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.restream.update(cx, |r, cx| r.stop(cx))
                            })),
                    })
                    .when(ours && status == RestreamStatus::Running, |el| {
                        el.child(
                            Button::new("restream-watch")
                                .outline()
                                .label("Watch here")
                                .loading(self.watching)
                                .disabled(self.watching)
                                .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.watch_here(window, cx))),
                        )
                    })
                    .child(
                        Button::new("restream-share")
                            .outline()
                            .small()
                            .icon(IconName::ExternalLink)
                            .label("Export as channel file…")
                            .disabled(addresses.is_empty())
                            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.share(window, cx))),
                    ),
            )
    }
}
