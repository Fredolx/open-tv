//! Top-level window content: the first-run setup screen or the main tabs.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, WindowExt as _, h_flex, notification::Notification,
    spinner::Spinner, tab::TabBar, v_flex,
};
use gpui_kit::*;
use lily_core::{
    epg,
    events::CoreEvent,
    settings, source_type, sql,
    types::{Settings, Source},
    utils,
};

use crate::{
    backend::{Backend, EventBus},
    browse::{BrowseModel, BrowseView},
    downloads::DownloadsBar,
    settings::SettingsView,
    sources::{AddSourceForm, SourceAdded, SourcesChanged, SourcesView},
};

const PAGES: [&str; 3] = ["Browse", "Sources", "Settings"];

enum Screen {
    Loading,
    Failed(SharedString),
    Setup(Entity<AddSourceForm>),
    Main(Main),
}

struct Main {
    page: usize,
    browse: Entity<BrowseView>,
    sources: Entity<SourcesView>,
    settings: Entity<SettingsView>,
}

pub struct Shell {
    screen: Screen,
    downloads: Entity<DownloadsBar>,
    refreshed_on_start: bool,
    reminders_armed: bool,
    _events: Subscription,
    _subscriptions: Vec<Subscription>,
}

impl Shell {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let bus = Backend::bus(cx);
        let events = cx.subscribe_in(&bus, window, |_, _: &Entity<EventBus>, event: &CoreEvent, window, cx| {
            if let CoreEvent::EpgLive { title, channel_name } = event {
                window.push_notification(
                    Notification::info(format!("Watch on {channel_name}"))
                        .title(format!("Starting now: {title}"))
                        .autohide(false)
                        .in_app_and_system(),
                    cx,
                );
            }
        });
        let mut this = Self {
            screen: Screen::Loading,
            downloads: cx.new(DownloadsBar::new),
            refreshed_on_start: false,
            reminders_armed: false,
            _events: events,
            _subscriptions: Vec::new(),
        };
        this.load(window, cx);
        this
    }

    /// (Re)reads sources and settings, then shows the matching screen.
    fn load(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let startup = Backend::blocking(cx, || {
            let any_source = !sql::get_sources()?.is_empty();
            let enabled = sql::get_enabled_sources()?;
            let settings = settings::get_settings()?;
            Ok((any_source, enabled, settings))
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = startup.await;
            _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok((false, _, _)) => this.show_setup(window, cx),
                    Ok((true, enabled, settings)) => {
                        let has_xtream = enabled.iter().any(|s| s.source_type == source_type::XTREAM);
                        this.show_main(enabled, &settings, window, cx);
                        if has_xtream && !this.reminders_armed {
                            this.reminders_armed = true;
                            this.arm_reminders(cx);
                        }
                        if settings.refresh_on_start == Some(true) && !this.refreshed_on_start {
                            this.refreshed_on_start = true;
                            this.refresh_all_on_start(window, cx);
                        }
                    }
                    Err(e) => {
                        this.screen =
                            Screen::Failed(format!("Couldn't open the channel database: {e:#}").into())
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn show_setup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let form = cx.new(|cx| AddSourceForm::new(window, cx));
        self._subscriptions = vec![cx.subscribe_in(&form, window, |this, _, _: &SourceAdded, window, cx| {
            this.load(window, cx)
        })];
        self.screen = Screen::Setup(form);
    }

    /// Re-arms guide reminders saved in an earlier session.
    fn arm_reminders(&mut self, cx: &mut Context<Self>) {
        let request = Backend::run_with_events(cx, |state, events| async move {
            epg::on_start_check_epg(&state, events).await
        });
        cx.spawn(async move |_, _| {
            if let Err(e) = request.await {
                lily_core::log::log(format!("restore reminders: {e:?}"));
            }
        })
        .detach();
    }

    fn show_main(&mut self, sources: Vec<Source>, settings: &Settings, window: &mut Window, cx: &mut Context<Self>) {
        let page = match &self.screen {
            Screen::Main(main) => main.page,
            _ => 0,
        };
        let browse = Self::new_browse(sources, settings, window, cx);
        let sources = cx.new(|cx| SourcesView::new(window, cx));
        let settings_view = cx.new(|cx| SettingsView::new(window, cx));
        self._subscriptions = vec![cx.subscribe_in(&sources, window, |this, _, _: &SourcesChanged, window, cx| {
            this.rebuild_browse(window, cx)
        })];
        self.screen = Screen::Main(Main {
            page,
            browse,
            sources,
            settings: settings_view,
        });
    }

    fn new_browse(
        sources: Vec<Source>,
        settings: &Settings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<BrowseView> {
        let ids = sources.iter().filter_map(|s| s.id).collect();
        let model = BrowseModel::new(ids, settings.default_view, settings.default_sort);
        cx.new(|cx| BrowseView::new(model, &sources, window, cx))
    }

    /// Sources changed: browse needs the new source list. Deleting the last source
    /// sends the user back to setup.
    fn rebuild_browse(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let request = Backend::blocking(cx, || {
            let any_source = !sql::get_sources()?.is_empty();
            Ok((any_source, sql::get_enabled_sources()?, settings::get_settings()?))
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok((false, _, _)) => this.show_setup(window, cx),
                    Ok((true, enabled, settings)) => {
                        if let Screen::Main(main) = &mut this.screen {
                            main.browse = Self::new_browse(enabled, &settings, window, cx);
                        }
                    }
                    Err(e) => window.push_notification(
                        Notification::error(format!("{e:#}")).title("Couldn't reload sources"),
                        cx,
                    ),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn refresh_all_on_start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.push_notification(Notification::info("Refreshing all sources…"), cx);
        let request = Backend::run(cx, |_| utils::refresh_all());
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                let note = match result {
                    Ok(()) => Notification::success("All sources refreshed."),
                    Err(e) => Notification::error(format!("{e:#}")).title("Refresh on start failed"),
                };
                window.push_notification(note, cx);
                if let Screen::Main(main) = &this.screen {
                    main.sources.update(cx, |sources, cx| sources.reload(window, cx));
                }
                this.rebuild_browse(window, cx);
            });
        })
        .detach();
    }

    fn render_setup(form: &Entity<AddSourceForm>, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("setup")
            .size_full()
            .overflow_y_scroll()
            .items_center()
            .p_8()
            .child(
                v_flex()
                    .w_full()
                    .max_w_128()
                    .gap_4()
                    .child(div().text_2xl().child("Welcome to Lily TV"))
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child("Add your IPTV provider to get started. You can add more sources later."),
                    )
                    .child(form.clone()),
            )
    }

    fn render_main(main: &Main, downloads: &Entity<DownloadsBar>, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match main.page {
            1 => main.sources.clone().into_any_element(),
            2 => main.settings.clone().into_any_element(),
            _ => main.browse.clone().into_any_element(),
        };
        v_flex()
            .size_full()
            .child(
                h_flex()
                    .px_3()
                    .pt_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        TabBar::new("pages")
                            .underline()
                            .children(PAGES)
                            .selected_index(main.page)
                            .on_click(cx.listener(|this, ix: &usize, _, cx| {
                                if let Screen::Main(main) = &mut this.screen {
                                    main.page = *ix;
                                    cx.notify();
                                }
                            })),
                    ),
            )
            .child(div().flex_1().min_h_0().child(content))
            .child(downloads.clone())
    }
}

impl Render for Shell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match &self.screen {
            Screen::Loading => h_flex()
                .size_full()
                .justify_center()
                .child(Spinner::new().large())
                .into_any_element(),
            Screen::Failed(message) => v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .p_8()
                .text_color(cx.theme().danger)
                .child(message.clone())
                .into_any_element(),
            Screen::Setup(form) => Self::render_setup(form, cx).into_any_element(),
            Screen::Main(main) => Self::render_main(main, &self.downloads, cx).into_any_element(),
        };
        div()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(body)
    }
}
