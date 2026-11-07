use std::{collections::HashSet, time::Duration};

use gpui_kit::component::{
    ActiveTheme as _, IconName, IndexPath, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _, Toggle},
    h_flex,
    input::{Input, InputEvent, InputState},
    list::{List, ListEvent, ListState},
    notification::Notification,
    select::{Select, SelectEvent, SelectState},
    spinner::Spinner,
    tab::TabBar,
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{media_type, mpv, sort_type, sql, types::Channel, view_type, xtream};

use super::{
    channel_list::ChannelListDelegate,
    model::{Activation, BrowseModel, NodeKind},
};
use crate::backend::Backend;

const SEARCH_DEBOUNCE: Duration = Duration::from_millis(150);
const KEY_CONTEXT: &str = "Browse";

actions!(browse, [GoBack]);

pub fn init(cx: &mut App) {
    // Text inputs bind escape/backspace themselves, so these only fire outside them
    cx.bind_keys([
        KeyBinding::new("escape", GoBack, Some(KEY_CONTEXT)),
        KeyBinding::new("backspace", GoBack, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-left", GoBack, Some(KEY_CONTEXT)),
    ]);
}

const VIEWS: [(u8, &str); 4] = [
    (view_type::ALL, "All"),
    (view_type::FAVORITES, "Favourites"),
    (view_type::CATEGORIES, "Categories"),
    (view_type::HISTORY, "History"),
];
const MEDIA: [(u8, &str); 3] = [
    (media_type::LIVESTREAM, "Live"),
    (media_type::MOVIE, "Movies"),
    (media_type::SERIE, "Series"),
];
const SORTS: [(u8, &str); 3] = [
    (sort_type::PROVIDER, "Provider order"),
    (sort_type::ALPHABETICAL_ASC, "A to Z"),
    (sort_type::ALPHABETICAL_DESC, "Z to A"),
];

/// What mpv is doing, shown in the status bar.
enum Playback {
    Idle,
    Playing { channel: Channel },
}

pub struct BrowseView {
    focus_handle: FocusHandle,
    model: BrowseModel,
    list: Entity<ListState<ChannelListDelegate>>,
    search: Entity<InputState>,
    sort: Entity<SelectState<Vec<&'static str>>>,
    playback: Playback,
    /// Series whose episodes were already fetched from the provider this session
    refreshed_series: HashSet<i64>,
    opening_series: bool,
    _search_debounce: Task<()>,
    _subscriptions: Vec<Subscription>,
}

impl BrowseView {
    pub fn new(model: BrowseModel, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let list = cx.new(|cx| ListState::new(ChannelListDelegate::new(), window, cx));
        let search = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Search channels, movies and series")
        });
        let sort_ix = SORTS.iter().position(|(s, _)| *s == model.sort()).unwrap_or(0);
        let sort = cx.new(|cx| {
            SelectState::new(
                SORTS.iter().map(|(_, label)| *label).collect::<Vec<_>>(),
                Some(IndexPath::new(sort_ix)),
                window,
                cx,
            )
        });

        let subscriptions = vec![
            cx.subscribe_in(&list, window, Self::on_list_event),
            cx.subscribe_in(&search, window, |this, state, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    let query = state.read(cx).value().to_string();
                    this.schedule_search(query, window, cx);
                }
            }),
            cx.subscribe_in(&sort, window, |this, _, event: &SelectEvent<Vec<&'static str>>, window, cx| {
                let SelectEvent::Confirm(Some(label)) = event else {
                    return;
                };
                if let Some((sort, _)) = SORTS.iter().find(|(_, l)| l == label) {
                    this.model.set_sort(*sort);
                    this.reload(window, cx);
                }
            }),
        ];

        // arrow keys and Enter work on the results straight away
        list.update(cx, |list, cx| list.focus(window, cx));

        let this = Self {
            focus_handle: cx.focus_handle(),
            model,
            list,
            search,
            sort,
            playback: Playback::Idle,
            refreshed_series: HashSet::new(),
            opening_series: false,
            _search_debounce: Task::ready(()),
            _subscriptions: subscriptions,
        };
        this.reload(window, cx);
        this
    }

    fn focus_list(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| list.focus(window, cx));
    }

    fn reload(&self, window: &mut Window, cx: &mut Context<Self>) {
        let filters = self.model.first_page();
        self.list.update(cx, |list, cx| {
            // a new result set starts unselected
            list.set_selected_index(None, window, cx);
            list.delegate_mut().load(filters, cx);
        });
        cx.notify();
    }

    fn schedule_search(&mut self, query: String, window: &mut Window, cx: &mut Context<Self>) {
        self._search_debounce = cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(SEARCH_DEBOUNCE).await;
            _ = this.update_in(cx, |this, window, cx| {
                if this.model.query() != query {
                    this.model.set_query(&query);
                    this.reload(window, cx);
                }
            });
        });
    }

    fn set_search_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.search
            .update(cx, |search, cx| search.set_value(text.to_string(), window, cx));
    }

    fn on_list_event(
        &mut self,
        list: &Entity<ListState<ChannelListDelegate>>,
        event: &ListEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            ListEvent::Confirm(ix) => {
                let Some(row) = list.read(cx).delegate().row(*ix).cloned() else {
                    return;
                };
                self.activate(row, window, cx);
            }
            // Esc in the list goes up one level, like Backspace/Esc in Fred TV
            ListEvent::Cancel => self.go_back(window, cx),
            ListEvent::Select(_) => {}
        }
    }

    fn activate(&mut self, row: Channel, window: &mut Window, cx: &mut Context<Self>) {
        if self.opening_series {
            return;
        }
        match self.model.activate(&row) {
            Activation::Play(channel) => self.play(channel, window, cx),
            Activation::Entered(NodeKind::Series) => {
                self.set_search_text("", window, cx);
                self.open_series(row, window, cx);
            }
            Activation::Entered(_) => {
                self.set_search_text("", window, cx);
                self.reload(window, cx);
            }
        }
        self.focus_list(window, cx);
    }

    /// Xtream series only list their episodes after asking the provider once.
    fn open_series(&mut self, series: Channel, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = series.id else {
            self.reload(window, cx);
            return;
        };
        if self.refreshed_series.contains(&id) {
            self.reload(window, cx);
            return;
        }
        self.opening_series = true;
        cx.notify();
        let fetch = Backend::run(cx, move |_| xtream::get_episodes(series));
        cx.spawn_in(window, async move |this, cx| {
            let result = fetch.await;
            _ = this.update_in(cx, |this, window, cx| {
                this.opening_series = false;
                match result {
                    Ok(()) => {
                        this.refreshed_series.insert(id);
                    }
                    Err(e) => window.push_notification(
                        Notification::error(format!("{e:#}")).title("Couldn't fetch the series"),
                        cx,
                    ),
                }
                this.reload(window, cx);
            });
        })
        .detach();
    }

    fn go_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.model.back() {
            let query = self.model.query().to_string();
            self.set_search_text(&query, window, cx);
            self.reload(window, cx);
            self.focus_list(window, cx);
        }
    }

    fn play(&mut self, channel: Channel, window: &mut Window, cx: &mut Context<Self>) {
        if let Playback::Playing { channel: current } = &self.playback {
            if current.id == channel.id {
                return;
            }
        }
        self.playback = Playback::Playing { channel: channel.clone() };
        cx.notify();

        let id = channel.id;
        let name = channel.name.clone();
        let session = Backend::run(cx, move |state| async move {
            // remembered for History even if mpv later fails to open the stream
            if let Some(id) = channel.id {
                if let Err(e) = sql::add_last_watched(id) {
                    lily_core::log::log(format!("{e:?}"));
                }
            }
            mpv::play(channel, false, None, &state).await
        });
        cx.spawn_in(window, async move |this, cx| {
            // resolves when mpv exits
            let result = session.await;
            _ = this.update_in(cx, |this, window, cx| {
                if matches!(&this.playback, Playback::Playing { channel } if channel.id == id) {
                    this.playback = Playback::Idle;
                }
                if let Err(e) = result {
                    window.push_notification(
                        Notification::error(format!("{e:#}")).title(format!("Couldn't play {name}")),
                        cx,
                    );
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn stop(&mut self, cx: &mut Context<Self>) {
        let Playback::Playing { channel } = &self.playback else {
            return;
        };
        let (Some(source_id), Some(id)) = (channel.source_id, channel.id) else {
            return;
        };
        // mpv's exit then resolves the play task, which resets the status
        let cancel = Backend::run(cx, move |state| async move {
            mpv::cancel_play(source_id, id.to_string(), &state).await
        });
        cx.spawn(async move |_, _| {
            if let Err(e) = cancel.await {
                lily_core::log::log(format!("stop playback: {e:?}"));
            }
        })
        .detach();
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let stack = self.model.stack();
        let selected_view = VIEWS
            .iter()
            .position(|(v, _)| *v == self.model.view())
            .unwrap_or(0);
        v_flex()
            .gap_2()
            .p_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .gap_2()
                    .when(!stack.is_empty(), |el| {
                        el.child(
                            Button::new("back")
                                .ghost()
                                .icon(IconName::ArrowLeft)
                                .tooltip("Back (Esc)")
                                .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                                    this.go_back(window, cx)
                                })),
                        )
                        .child(
                            div()
                                .flex_none()
                                .max_w_80()
                                .truncate()
                                .text_color(cx.theme().muted_foreground)
                                .child(
                                    stack
                                        .iter()
                                        .map(|n| n.name.as_str())
                                        .collect::<Vec<_>>()
                                        .join(" › "),
                                ),
                        )
                    })
                    .child(
                        div().flex_1().child(
                            Input::new(&self.search)
                                .cleanable(true)
                                .prefix(gpui_kit::component::Icon::new(IconName::Search).small()),
                        ),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .justify_between()
                    .child(
                        TabBar::new("views")
                            .segmented()
                            .small()
                            .children(VIEWS.iter().map(|(_, label)| *label))
                            .selected_index(selected_view)
                            .on_click(cx.listener(|this, ix: &usize, window, cx| {
                                if let Some((view, _)) = VIEWS.get(*ix) {
                                    this.model.set_view(*view);
                                    let query = this.model.query().to_string();
                                    this.set_search_text(&query, window, cx);
                                    this.reload(window, cx);
                                }
                            })),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .children(MEDIA.iter().map(|(media, label)| {
                                let media = *media;
                                Toggle::new(ElementId::Name(format!("media-{media}").into()))
                                    .small()
                                    .label(*label)
                                    .checked(self.model.shows_media_type(media))
                                    .on_click(cx.listener(move |this, _: &bool, window, cx| {
                                        this.model.toggle_media_type(media);
                                        this.reload(window, cx);
                                    }))
                            }))
                            .child(div().w_40().child(Select::new(&self.sort).small())),
                    ),
            )
    }

    fn render_status(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let label: SharedString = match &self.playback {
            Playback::Playing { channel } => format!("Playing {}", channel.name).into(),
            Playback::Idle if self.opening_series => "Fetching episodes…".into(),
            Playback::Idle => return None,
        };
        let playing = matches!(self.playback, Playback::Playing { .. });
        Some(
            h_flex()
                .gap_2()
                .px_3()
                .py_2()
                .border_t_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().secondary)
                .child(Spinner::new().small())
                .child(div().flex_1().truncate().child(label))
                .when(playing, |el| {
                    el.child(
                        Button::new("stop")
                            .small()
                            .outline()
                            .label("Stop")
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.stop(cx))),
                    )
                }),
        )
    }
}

impl Render for BrowseView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &GoBack, window, cx| this.go_back(window, cx)))
            .size_full()
            .child(self.render_header(cx))
            .child(div().flex_1().min_h_0().child(List::new(&self.list)))
            .children(self.render_status(cx))
    }
}
