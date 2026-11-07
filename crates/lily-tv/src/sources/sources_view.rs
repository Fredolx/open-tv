use std::collections::HashSet;

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, IconName, WindowExt as _,
    button::{Button, ButtonVariant, ButtonVariants as _},
    h_flex,
    notification::Notification,
    switch::Switch,
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{source_type, sql, types::Source, utils};

use super::add_source_form::{AddSourceForm, SourceAdded};
use crate::backend::Backend;

/// Emitted whenever the set of usable sources changes.
pub struct SourcesChanged;

pub struct SourcesView {
    sources: Vec<Source>,
    add_form: Entity<AddSourceForm>,
    adding: bool,
    /// Source ids with a refresh in flight
    refreshing: HashSet<i64>,
    refreshing_all: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<SourcesChanged> for SourcesView {}

impl SourcesView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let add_form = cx.new(|cx| AddSourceForm::new(window, cx));
        let subscription = cx.subscribe_in(&add_form, window, |this, _, _: &SourceAdded, window, cx| {
            this.adding = false;
            window.push_notification(Notification::success("Source added."), cx);
            this.changed(window, cx);
        });
        let mut this = Self {
            sources: Vec::new(),
            add_form,
            adding: false,
            refreshing: HashSet::new(),
            refreshing_all: false,
            _subscriptions: vec![subscription],
        };
        this.reload(window, cx);
        this
    }

    /// Re-reads the source list, e.g. after a refresh started elsewhere.
    pub fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let request = Backend::blocking(cx, sql::get_sources);
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(sources) => this.sources = sources,
                    Err(e) => window.push_notification(
                        Notification::error(format!("{e:#}")).title("Couldn't load sources"),
                        cx,
                    ),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reload(window, cx);
        cx.emit(SourcesChanged);
    }

    /// Runs a backend change, then reloads; errors become a notification titled `failure`.
    fn apply<F>(&mut self, failure: &'static str, op: F, window: &mut Window, cx: &mut Context<Self>)
    where
        F: FnOnce() -> lily_core::anyhow::Result<()> + Send + 'static,
    {
        let request = Backend::blocking(cx, op);
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                if let Err(e) = result {
                    window.push_notification(Notification::error(format!("{e:#}")).title(failure), cx);
                }
                this.changed(window, cx);
            });
        })
        .detach();
    }

    fn set_enabled(&mut self, id: i64, enabled: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.apply(
            "Couldn't update the source",
            move || sql::set_source_enabled(enabled, id),
            window,
            cx,
        );
    }

    fn confirm_delete(&mut self, source: &Source, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = source.id else {
            return;
        };
        let name = source.name.clone();
        let view = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, _| {
            let view = view.clone();
            alert
                .title(format!("Delete “{name}”?"))
                .description("Its channels, favourites and watch history are removed. You can add it again later.")
                .confirm()
                .ok_text("Delete")
                .ok_variant(ButtonVariant::Danger)
                .on_ok(move |_, window, cx| {
                    _ = view.update(cx, |this, cx| {
                        this.apply("Couldn't delete the source", move || sql::delete_source(id), window, cx);
                    });
                    true
                })
        });
    }

    fn refresh(&mut self, source: Source, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = source.id else {
            return;
        };
        if !self.refreshing.insert(id) {
            return;
        }
        cx.notify();
        let name = source.name.clone();
        let request = Backend::run(cx, move |_| utils::refresh_source(source));
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                this.refreshing.remove(&id);
                let note = match result {
                    Ok(()) => Notification::success(format!("Refreshed “{name}”.")),
                    Err(e) => Notification::error(format!("{e:#}")).title(format!("Couldn't refresh “{name}”")),
                };
                window.push_notification(note, cx);
                this.changed(window, cx);
            });
        })
        .detach();
    }

    fn refresh_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.refreshing_all {
            return;
        }
        self.refreshing_all = true;
        cx.notify();
        let request = Backend::run(cx, |_| utils::refresh_all());
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                this.refreshing_all = false;
                let note = match result {
                    Ok(()) => Notification::success("All sources refreshed."),
                    Err(e) => Notification::error(format!("{e:#}")).title("Refresh failed"),
                };
                window.push_notification(note, cx);
                this.changed(window, cx);
            });
        })
        .detach();
    }

    fn render_source(&self, source: &Source, cx: &mut Context<Self>) -> impl IntoElement {
        let id = source.id.unwrap_or_default();
        let refreshing = self.refreshing.contains(&id) || self.refreshing_all;
        let kind = match source.source_type {
            source_type::XTREAM => "Xtream",
            source_type::M3U_LINK => "M3U link",
            source_type::M3U => "M3U file",
            _ => "Custom",
        };
        let updated = source
            .last_updated
            .and_then(|ts| utils::get_local_time(ts).ok())
            .map(|t| format!("Updated {}", t.format("%-d %b %Y, %H:%M")))
            .unwrap_or_else(|| "Never refreshed".to_string());
        let refreshable = source.source_type != source_type::CUSTOM;
        let enabled = source.enabled;
        let for_refresh = source.clone();
        let for_delete = source.clone();
        h_flex()
            .id(ElementId::Name(format!("source-{id}").into()))
            .gap_3()
            .p_3()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .child(
                Switch::new(ElementId::Name(format!("source-{id}-enabled").into()))
                    .checked(enabled)
                    .tooltip(if enabled { "Hide this source's channels" } else { "Show this source's channels" })
                    .on_change(cx.listener(move |this, checked: &bool, window, cx| {
                        this.set_enabled(id, *checked, window, cx)
                    })),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(div().truncate().child(source.name.clone()))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{kind} · {updated}")),
                    ),
            )
            .when(refreshable, |el| {
                el.child(
                    Button::new(ElementId::Name(format!("source-{id}-refresh").into()))
                        .ghost()
                        .icon(IconName::RefreshCw)
                        .loading(refreshing)
                        .disabled(refreshing)
                        .tooltip("Refresh channels")
                        .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                            this.refresh(for_refresh.clone(), window, cx)
                        })),
                )
            })
            .child(
                Button::new(ElementId::Name(format!("source-{id}-delete").into()))
                    .ghost()
                    .icon(IconName::Delete)
                    .tooltip("Delete source")
                    .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                        this.confirm_delete(&for_delete, window, cx)
                    })),
            )
    }
}

impl Render for SourcesView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let adding = self.adding;
        let sources: Vec<_> = self
            .sources
            .clone()
            .iter()
            .map(|source| self.render_source(source, cx).into_any_element())
            .collect();
        v_flex()
            .id("sources")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .child(div().flex_1().text_lg().child("Sources"))
                    .child(
                        Button::new("refresh-all")
                            .outline()
                            .icon(IconName::RefreshCw)
                            .label("Refresh all")
                            .loading(self.refreshing_all)
                            .disabled(self.refreshing_all || self.sources.is_empty())
                            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.refresh_all(window, cx))),
                    )
                    .child(
                        Button::new("add-source")
                            .primary()
                            .icon(if adding { IconName::Close } else { IconName::Plus })
                            .label(if adding { "Cancel" } else { "Add source" })
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.adding = !this.adding;
                                cx.notify();
                            })),
                    ),
            )
            .when(adding, |el| {
                el.child(
                    div()
                        .p_4()
                        .rounded(cx.theme().radius)
                        .border_1()
                        .border_color(cx.theme().border)
                        .child(self.add_form.clone()),
                )
            })
            .when(sources.is_empty() && !adding, |el| {
                el.child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child("No sources yet. Add one to start watching."),
                )
            })
            .children(sources)
    }
}
