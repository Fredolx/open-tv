use std::{collections::HashSet, rc::Rc, time::Duration};

use gpui_kit::component::{
    ActiveTheme as _, IconName, IndexPath, Selectable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariant, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    kbd::Kbd,
    list::{List, ListEvent, ListState},
    menu::{DropdownMenu as _, PopupMenuItem},
    notification::Notification,
    select::{Select, SelectEvent, SelectState},
    spinner::Spinner,
    tab::TabBar,
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{
    bulk_action_type, media_type, mpv, settings, sort_type, source_type, sql,
    types::{Channel, Source},
    view_type, xtream,
};

use super::{
    channel_list::ChannelListDelegate,
    model::{Activation, BrowseModel, NodeKind},
    row_actions::{RowAction, RowContext},
};
use crate::{
    backend::Backend,
    custom,
    downloads::{Downloads, DownloadsChanged},
    file_prompts,
    guide::{GuideEvent, GuidePanel},
    naming::{RECORD_EXTENSION, date_stamp, sanitize_file_name, url_extension},
    restream::RestreamPanel,
};

const SEARCH_DEBOUNCE: Duration = Duration::from_millis(150);
const KEY_CONTEXT: &str = "Browse";

const VIEWS: [(u8, &str); 5] = [
    (view_type::ALL, "All"),
    (view_type::FAVORITES, "Favourites"),
    (view_type::CATEGORIES, "Categories"),
    (view_type::HISTORY, "History"),
    (view_type::HIDDEN, "Hidden"),
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

actions!(
    browse,
    [
        GoBack,
        ShowAll,
        ShowFavorites,
        ShowCategories,
        ShowHistory,
        FocusSearch,
        ToggleLive,
        ToggleMovies,
        ToggleSeries,
        ShowShortcuts
    ]
);

/// (keys, description) for the shortcut help, in the order bound below
const SHORTCUTS: [(&str, &str); 10] = [
    ("ctrl-a", "Show all channels"),
    ("ctrl-d", "Show favourites"),
    ("ctrl-s", "Show categories"),
    ("ctrl-h", "Show history"),
    ("ctrl-f", "Search"),
    ("ctrl-q", "Show or hide live channels"),
    ("ctrl-w", "Show or hide movies"),
    ("ctrl-e", "Show or hide series"),
    ("escape", "Go back (also Backspace, Alt+Left)"),
    ("f1", "This list"),
];

pub fn init(cx: &mut App) {
    let context = Some(KEY_CONTEXT);
    // Text inputs bind escape/backspace/ctrl-a themselves, so these only fire outside them
    cx.bind_keys([
        KeyBinding::new("escape", GoBack, context),
        KeyBinding::new("backspace", GoBack, context),
        KeyBinding::new("alt-left", GoBack, context),
        KeyBinding::new("ctrl-a", ShowAll, context),
        KeyBinding::new("ctrl-d", ShowFavorites, context),
        KeyBinding::new("ctrl-s", ShowCategories, context),
        KeyBinding::new("ctrl-h", ShowHistory, context),
        KeyBinding::new("ctrl-f", FocusSearch, context),
        KeyBinding::new("ctrl-q", ToggleLive, context),
        KeyBinding::new("ctrl-w", ToggleMovies, context),
        KeyBinding::new("ctrl-e", ToggleSeries, context),
        KeyBinding::new("f1", ShowShortcuts, context),
    ]);
}

/// What mpv is doing, shown in the status bar.
enum Playback {
    Idle,
    Playing { channel: Box<Channel>, recording: bool },
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
    _guide: Option<Subscription>,
    _subscriptions: Vec<Subscription>,
}

impl BrowseView {
    pub fn new(model: BrowseModel, sources: &[Source], window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = cx.entity().downgrade();
        let on_action: super::channel_list::RowActionHandler = Rc::new(move |action, channel, window, cx| {
            _ = view.update(cx, |this, cx| this.on_row_action(action, channel, window, cx));
        });
        let list = cx.new(|cx| ListState::new(ChannelListDelegate::new().on_action(on_action), window, cx));
        let downloads = Downloads::global(cx);
        let running = downloads.read(cx).running_channel_ids();
        list.update(cx, |list, _| {
            let context = list.delegate_mut().context_mut();
            *context = RowContext {
                view: model.view(),
                xtream_sources: ids_of_type(sources, source_type::XTREAM),
                custom_sources: ids_of_type(sources, source_type::CUSTOM),
                downloading: running,
            };
        });

        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search channels, movies and series"));
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
            cx.subscribe(&downloads, |this, downloads, _: &DownloadsChanged, cx| {
                let running = downloads.read(cx).running_channel_ids();
                this.list.update(cx, |list, cx| {
                    list.delegate_mut().context_mut().downloading = running;
                    cx.notify();
                });
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
            _guide: None,
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
        let view = self.model.view();
        self.list.update(cx, |list, cx| {
            // a new result set starts unselected
            list.set_selected_index(None, window, cx);
            list.delegate_mut().context_mut().view = view;
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

    fn set_view(&mut self, view: u8, window: &mut Window, cx: &mut Context<Self>) {
        self.model.set_view(view);
        let query = self.model.query().to_string();
        self.set_search_text(&query, window, cx);
        self.reload(window, cx);
    }

    fn toggle_media(&mut self, media: u8, window: &mut Window, cx: &mut Context<Self>) {
        self.model.toggle_media_type(media);
        self.reload(window, cx);
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
            // Esc in the list goes up one level, like Backspace
            ListEvent::Cancel => self.go_back(window, cx),
            ListEvent::Select(_) => {}
        }
    }

    fn activate(&mut self, row: Channel, window: &mut Window, cx: &mut Context<Self>) {
        if self.opening_series {
            return;
        }
        match self.model.activate(&row) {
            Activation::Play(channel) => self.play(*channel, false, None, window, cx),
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

    /// Plays (or records) a channel; `record_path` None records into the default folder.
    fn play(
        &mut self,
        channel: Channel,
        record: bool,
        record_path: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Playback::Playing { channel: current, .. } = &self.playback
            && current.id == channel.id
            && current.url == channel.url
            && !record
        {
            return;
        }
        self.playback = Playback::Playing { channel: Box::new(channel.clone()), recording: record };
        cx.notify();

        let id = channel.id;
        let url = channel.url.clone();
        let name = channel.name.clone();
        let session = Backend::run(cx, move |state| async move {
            // remembered for History even if mpv later fails to open the stream
            if let Some(id) = channel.id.filter(|id| *id >= 0)
                && let Err(e) = sql::add_last_watched(id)
            {
                lily_core::log::log(format!("{e:?}"));
            }
            mpv::play(channel, record, record_path, &state).await
        });
        cx.spawn_in(window, async move |this, cx| {
            // resolves when mpv exits
            let result = session.await;
            _ = this.update_in(cx, |this, window, cx| {
                if matches!(&this.playback, Playback::Playing { channel, .. } if channel.id == id && channel.url == url) {
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
        let Playback::Playing { channel, .. } = &self.playback else {
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

    fn on_row_action(&mut self, action: RowAction, row: Channel, window: &mut Window, cx: &mut Context<Self>) {
        let refresh: custom::Done = {
            let view = cx.entity().downgrade();
            Rc::new(move |window, cx| {
                _ = view.update(cx, |this, cx| this.reload(window, cx));
            })
        };
        match action {
            RowAction::ToggleFavorite => {
                if let Some(id) = row.id {
                    self.list.update(cx, |list, cx| list.delegate_mut().toggle_favorite(id, cx));
                }
            }
            RowAction::ToggleHidden => {
                let (Some(id), hidden) = (row.id, row.hidden != Some(true)) else {
                    return;
                };
                let is_group = row.media_type == media_type::GROUP;
                self.apply(
                    move || if is_group { sql::hide_group(id, hidden) } else { sql::hide_channel(id, hidden) },
                    if hidden { "Hidden. Find it again in the Hidden view." } else { "Unhidden." },
                    "Couldn't change visibility",
                    window,
                    cx,
                );
            }
            RowAction::RemoveFromHistory => {
                if let Some(id) = row.id {
                    self.apply(move || sql::remove_last_watched(id), "Removed from history.", "Couldn't remove it", window, cx);
                }
            }
            RowAction::Record => self.record(row, window, cx),
            RowAction::Download => self.download(row, None, window, cx),
            RowAction::CancelDownload => {
                if let Some(id) = row.id {
                    Downloads::global(cx).update(cx, |d, cx| d.cancel_channel(id, cx));
                }
            }
            RowAction::Guide => self.open_guide(row, window, cx),
            RowAction::Restream => {
                let panel = cx.new(|cx| RestreamPanel::new(row.clone(), window, cx));
                let title = format!("Re-stream {}", row.name);
                window.open_dialog(cx, move |dialog, _, _| dialog.title(title.clone()).w(px(620.)).child(panel.clone()));
            }
            RowAction::CopyUrl => {
                if let Some(url) = row.url {
                    cx.write_to_clipboard(ClipboardItem::new_string(url));
                    window.push_notification(Notification::success("Copied the stream URL."), cx);
                }
            }
            RowAction::Edit => custom::edit(row, refresh, window, cx),
            RowAction::Share => custom::share_row(row, window, cx),
            RowAction::Delete => custom::confirm_delete(row, refresh, window, cx),
        }
    }

    /// Runs a blocking change, notifies, and reloads the results.
    fn apply<F>(&self, op: F, success: &'static str, failure: &'static str, window: &mut Window, cx: &mut Context<Self>)
    where
        F: FnOnce() -> lily_core::anyhow::Result<()> + Send + 'static,
    {
        let request = Backend::blocking(cx, op);
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                let note = match result {
                    Ok(()) => Notification::success(success),
                    Err(e) => Notification::error(format!("{e:#}")).title(failure),
                };
                window.push_notification(note, cx);
                this.reload(window, cx);
            });
        })
        .detach();
    }

    fn open_guide(&mut self, channel: Channel, window: &mut Window, cx: &mut Context<Self>) {
        let title = format!("TV guide · {}", channel.name);
        let panel = cx.new(|cx| GuidePanel::new(channel, window, cx));
        self._guide = Some(cx.subscribe_in(&panel, window, |this, _, event: &GuideEvent, window, cx| {
            window.close_dialog(cx);
            match event {
                GuideEvent::PlayCatchUp(channel) => this.play(channel.clone(), false, None, window, cx),
                GuideEvent::DownloadCatchUp(channel) => {
                    let id = format!("catchup-{}", channel.url.clone().unwrap_or_default());
                    this.download(channel.clone(), Some(id), window, cx)
                }
            }
        }));
        window.open_dialog(cx, move |dialog, _, _| dialog.title(title.clone()).w(px(760.)).child(panel.clone()));
    }

    /// Asks for a file when the "always ask where to save" setting is on; otherwise
    /// resolves to None so the backend uses the recordings folder.
    fn save_location(
        &self,
        suggested_name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, Option<String>, &mut Window, &mut Context<Self>) + 'static,
    ) {
        let settings = Backend::blocking(cx, settings::get_settings);
        cx.spawn_in(window, async move |this, cx| {
            let settings = settings.await.ok();
            let ask = settings.as_ref().and_then(|s| s.always_ask_save).unwrap_or(false);
            let path = if ask {
                let folder = settings.and_then(|s| s.recording_path);
                let picked = cx.update(|_, cx| file_prompts::save_file(cx, folder, &suggested_name));
                match picked {
                    Ok(picked) => match picked.await {
                        Some(path) => Some(path),
                        None => return, // cancelled
                    },
                    Err(_) => return,
                }
            } else {
                None
            };
            _ = this.update_in(cx, |this, window, cx| then(this, path, window, cx));
        })
        .detach();
    }

    fn record(&mut self, channel: Channel, window: &mut Window, cx: &mut Context<Self>) {
        let name = format!("{}_{}.{RECORD_EXTENSION}", sanitize_file_name(&channel.name), date_stamp());
        self.save_location(name, window, cx, move |this, path, window, cx| {
            this.play(channel, true, path, window, cx)
        });
    }

    fn download(&mut self, channel: Channel, download_id: Option<String>, window: &mut Window, cx: &mut Context<Self>) {
        let ext = url_extension(channel.url.as_deref().unwrap_or_default()).to_string();
        let name = format!("{}.{ext}", sanitize_file_name(&channel.name));
        let id = download_id.unwrap_or_else(|| channel.id.unwrap_or_default().to_string());
        self.save_location(name, window, cx, move |_, path, window, cx| {
            Downloads::global(cx).update(cx, |d, cx| d.start(channel, id, path, cx));
            window.push_notification(Notification::info("Download started."), cx);
        });
    }

    fn bulk(&mut self, action: u8, window: &mut Window, cx: &mut Context<Self>) {
        let filters = self.model.first_page();
        self.apply(
            move || sql::bulk_update(filters, action),
            "Updated every matching result.",
            "Couldn't update the results",
            window,
            cx,
        );
    }

    fn confirm_clear_history(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, _| {
            let view = view.clone();
            alert
                .title("Clear watch history?")
                .description("Everything is removed from the History view. Favourites are kept.")
                .confirm()
                .ok_text("Clear history")
                .ok_variant(ButtonVariant::Danger)
                .on_ok(move |_, window, cx| {
                    _ = view.update(cx, |this, cx| {
                        this.apply(sql::clear_history, "History cleared.", "Couldn't clear history", window, cx)
                    });
                    true
                })
        });
    }

    fn show_shortcuts(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.open_dialog(cx, |dialog, _, cx| {
            dialog.title("Keyboard shortcuts").w(px(460.)).child(
                v_flex().gap_2().children(SHORTCUTS.iter().map(|(keys, what)| {
                    h_flex()
                        .justify_between()
                        .child(div().text_color(cx.theme().muted_foreground).child(*what))
                        .children(Keystroke::parse(keys).ok().map(Kbd::new))
                })),
            )
        });
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let stack = self.model.stack();
        let view = self.model.view();
        let selected_view = VIEWS.iter().position(|(v, _)| *v == view).unwrap_or(0);
        let can_bulk = self.model.can_bulk_update();
        let entity = cx.entity().downgrade();
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
                                .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.go_back(window, cx))),
                        )
                        .child(
                            div()
                                .flex_none()
                                .max_w_80()
                                .truncate()
                                .text_color(cx.theme().muted_foreground)
                                .child(stack.iter().map(|n| n.name.as_str()).collect::<Vec<_>>().join(" › ")),
                        )
                    })
                    .child(
                        div().flex_1().child(
                            Input::new(&self.search)
                                .cleanable(true)
                                .prefix(gpui_kit::component::Icon::new(IconName::Search).small()),
                        ),
                    )
                    .child(
                        filter_button("keywords", "Words in any order", self.model.use_keywords())
                            .tooltip("Match every word anywhere in the name, instead of the exact phrase")
                            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                                this.model.toggle_keywords();
                                this.reload(window, cx);
                            })),
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
                                    this.set_view(*view, window, cx);
                                }
                            })),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .children(MEDIA.iter().map(|(media, label)| {
                                let media = *media;
                                filter_button(format!("media-{media}"), label, self.model.shows_media_type(media))
                                    .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                                        this.toggle_media(media, window, cx)
                                    }))
                            }))
                            .child(div().w_40().child(Select::new(&self.sort).small()))
                            .when(view == view_type::HISTORY, |el| {
                                el.child(
                                    Button::new("clear-history")
                                        .small()
                                        .ghost()
                                        .label("Clear history")
                                        .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                                            this.confirm_clear_history(window, cx)
                                        })),
                                )
                            })
                            .when(can_bulk && view != view_type::HISTORY, |el| {
                                el.child(
                                    Button::new("bulk-actions")
                                        .small()
                                        .ghost()
                                        .icon(IconName::Ellipsis)
                                        .tooltip("Apply to every result")
                                        .dropdown_menu(move |menu, _, _| {
                                            [
                                                ("Favourite all results", bulk_action_type::FAVORITE),
                                                ("Unfavourite all results", bulk_action_type::UNFAVORITE),
                                                ("Hide all results", bulk_action_type::HIDE),
                                                ("Unhide all results", bulk_action_type::UNHIDE),
                                            ]
                                            .into_iter()
                                            .fold(menu, |menu, (label, action)| {
                                                let entity = entity.clone();
                                                menu.item(PopupMenuItem::new(label).on_click(move |_, window, cx| {
                                                    _ = entity.update(cx, |this, cx| this.bulk(action, window, cx));
                                                }))
                                            })
                                        }),
                                )
                            })
                            .child(
                                Button::new("shortcuts")
                                    .small()
                                    .ghost()
                                    .icon(IconName::Info)
                                    .tooltip("Keyboard shortcuts (F1)")
                                    .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                                        this.show_shortcuts(window, cx)
                                    })),
                            ),
                    ),
            )
    }

    fn render_status(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let label: SharedString = match &self.playback {
            Playback::Playing { channel, recording: true } => format!("Recording {}", channel.name).into(),
            Playback::Playing { channel, .. } => format!("Playing {}", channel.name).into(),
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

/// An on/off filter: outlined when off, selected with a check mark when on.
fn filter_button(id: impl Into<SharedString>, label: &'static str, on: bool) -> Button {
    Button::new(ElementId::Name(id.into()))
        .small()
        .outline()
        .selected(on)
        .when(on, |b| b.icon(IconName::Check))
        .label(label)
}

fn ids_of_type(sources: &[Source], kind: u8) -> HashSet<i64> {
    sources.iter().filter(|s| s.source_type == kind).filter_map(|s| s.id).collect()
}

impl Render for BrowseView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &GoBack, window, cx| this.go_back(window, cx)))
            .on_action(cx.listener(|this, _: &ShowAll, window, cx| this.set_view(view_type::ALL, window, cx)))
            .on_action(cx.listener(|this, _: &ShowFavorites, window, cx| this.set_view(view_type::FAVORITES, window, cx)))
            .on_action(cx.listener(|this, _: &ShowCategories, window, cx| {
                this.set_view(view_type::CATEGORIES, window, cx)
            }))
            .on_action(cx.listener(|this, _: &ShowHistory, window, cx| this.set_view(view_type::HISTORY, window, cx)))
            .on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
                this.search.update(cx, |search, cx| search.focus(window, cx))
            }))
            .on_action(cx.listener(|this, _: &ToggleLive, window, cx| this.toggle_media(media_type::LIVESTREAM, window, cx)))
            .on_action(cx.listener(|this, _: &ToggleMovies, window, cx| this.toggle_media(media_type::MOVIE, window, cx)))
            .on_action(cx.listener(|this, _: &ToggleSeries, window, cx| this.toggle_media(media_type::SERIE, window, cx)))
            .on_action(cx.listener(|this, _: &ShowShortcuts, window, cx| this.show_shortcuts(window, cx)))
            .size_full()
            .child(self.render_header(cx))
            .child(div().flex_1().min_h_0().child(List::new(&self.list)))
            .children(self.render_status(cx))
    }
}
