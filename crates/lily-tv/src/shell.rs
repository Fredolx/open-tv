//! Top-level window content: picks the first-run setup screen or the main screens.

use gpui_kit::component::{ActiveTheme as _, Sizable as _, h_flex, spinner::Spinner, v_flex};
use gpui_kit::*;
use lily_core::{settings, sql, types::Settings};

use crate::{backend::Backend, browse::{BrowseModel, BrowseView}};

enum Screen {
    Loading,
    Failed(SharedString),
    Browse(Entity<BrowseView>),
}

pub struct Shell {
    screen: Screen,
}

impl Shell {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self { screen: Screen::Loading };
        this.load(window, cx);
        this
    }

    /// (Re)reads sources and settings, then shows the matching screen.
    fn load(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = Screen::Loading;
        let startup = Backend::blocking(cx, || {
            let sources = sql::get_enabled_sources()?;
            let settings = settings::get_settings()?;
            Ok((sources, settings))
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = startup.await;
            _ = this.update_in(cx, |this, window, cx| {
                this.screen = match result {
                    Ok((sources, settings)) => {
                        let ids = sources.iter().filter_map(|s| s.id).collect();
                        Screen::Browse(Self::browse(ids, &settings, window, cx))
                    }
                    Err(e) => Screen::Failed(format!("Couldn't open the channel database: {e:#}").into()),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn browse(
        source_ids: Vec<i64>,
        settings: &Settings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<BrowseView> {
        let model = BrowseModel::new(source_ids, settings.default_view, settings.default_sort);
        cx.new(|cx| BrowseView::new(model, window, cx))
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
                .text_color(cx.theme().danger)
                .child(message.clone())
                .into_any_element(),
            Screen::Browse(browse) => browse.clone().into_any_element(),
        };
        div()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(body)
    }
}
