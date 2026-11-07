//! Virtualized channel list: owns the loaded rows and paging; the browse view owns
//! the filters and decides what a confirmed row means.

use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    ActiveTheme as _, Icon, IconName, IndexPath, Sizable as _, h_flex,
    list::{ListDelegate, ListItem, ListState},
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{media_type, sql, types::{Channel, Filters}};

use crate::backend::Backend;

/// The backend pages by 36 rows and stores the page number in a u8.
const PAGE_SIZE: usize = 36;
const MAX_PAGE: u8 = u8::MAX;

pub struct ChannelListDelegate {
    rows: Vec<Channel>,
    filters: Option<Filters>,
    /// Bumped on every new search so late pages from an old search are dropped
    revision: u64,
    loading_first: bool,
    loading_more: bool,
    end_reached: bool,
    error: Option<SharedString>,
    selected: Option<IndexPath>,
    _load_task: Option<Task<()>>,
}

impl ChannelListDelegate {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            filters: None,
            revision: 0,
            loading_first: false,
            loading_more: false,
            end_reached: true,
            error: None,
            selected: None,
            _load_task: None,
        }
    }

    pub fn row(&self, ix: IndexPath) -> Option<&Channel> {
        self.rows.get(ix.row)
    }

    /// Flips a row's favourite flag right away and reverts it if saving fails.
    fn toggle_favorite(&mut self, ix: IndexPath, cx: &mut Context<ListState<Self>>) {
        let Some(row) = self.rows.get_mut(ix.row) else {
            return;
        };
        let Some(id) = row.id else {
            return;
        };
        let favorite = !row.favorite;
        row.favorite = favorite;
        cx.notify();
        let save = Backend::blocking(cx, move || sql::favorite_channel(id, favorite));
        cx.spawn(async move |list, cx| {
            if let Err(e) = save.await {
                lily_core::log::log(format!("favourite {id}: {e:?}"));
                _ = list.update(cx, |list, cx| {
                    let this = list.delegate_mut();
                    if let Some(row) = this.rows.iter_mut().find(|r| r.id == Some(id) && is_favoritable(r.media_type)) {
                        row.favorite = !favorite;
                    }
                    this.error = Some("Couldn't save the favourite.".into());
                    cx.notify();
                });
            }
        })
        .detach();
    }

    /// Replaces the results with the first page for `filters`.
    pub fn load(&mut self, filters: Filters, cx: &mut Context<ListState<Self>>) {
        self.revision += 1;
        self.filters = Some(filters.clone());
        self.loading_first = true;
        self.loading_more = false;
        self.end_reached = false;
        self.error = None;
        self.fetch(filters, false, cx);
        cx.notify();
    }

    fn fetch(&mut self, filters: Filters, append: bool, cx: &mut Context<ListState<Self>>) {
        let revision = self.revision;
        let request = Backend::blocking(cx, move || sql::search(filters));
        self._load_task = Some(cx.spawn(async move |list, cx| {
            let result = request.await;
            _ = list.update(cx, |list, cx| {
                let this = list.delegate_mut();
                if this.revision != revision {
                    return;
                }
                this.loading_first = false;
                this.loading_more = false;
                match result {
                    Ok(page) => {
                        this.end_reached = page.len() < PAGE_SIZE;
                        if append {
                            this.rows.extend(page);
                        } else {
                            this.rows = page;
                            this.selected = None;
                        }
                    }
                    Err(e) => {
                        this.end_reached = true;
                        this.error = Some(format!("Couldn't load channels: {e:#}").into());
                        if !append {
                            this.rows.clear();
                        }
                    }
                }
                cx.notify();
            });
        }));
    }

    fn render_logo(channel: &Channel, cx: &App) -> AnyElement {
        let placeholder_icon = match channel.media_type {
            media_type::GROUP => IconName::Folder,
            media_type::SERIE | media_type::SEASON => IconName::GalleryVerticalEnd,
            media_type::MOVIE => IconName::Frame,
            _ => IconName::Play,
        };
        let frame = div()
            .flex_none()
            .size_10()
            .rounded(cx.theme().radius)
            .bg(cx.theme().muted)
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden();
        let muted = cx.theme().muted_foreground;
        match channel.image.as_deref().filter(|u| u.starts_with("http")) {
            Some(url) => frame
                .child(
                    img(SharedUri::from(url.to_string()))
                        .size_full()
                        .object_fit(ObjectFit::Contain)
                        .with_fallback(move || {
                            Icon::new(placeholder_icon.clone()).text_color(muted).into_any_element()
                        }),
                )
                .into_any_element(),
            None => frame
                .child(Icon::new(placeholder_icon).text_color(muted))
                .into_any_element(),
        }
    }
}

fn is_favoritable(media: u8) -> bool {
    matches!(media, media_type::LIVESTREAM | media_type::MOVIE | media_type::SERIE)
}

fn kind_label(media: u8) -> Option<&'static str> {
    match media {
        media_type::MOVIE => Some("Movie"),
        media_type::SERIE => Some("Series"),
        media_type::GROUP => Some("Category"),
        media_type::SEASON => Some("Season"),
        _ => None,
    }
}

impl ListDelegate for ChannelListDelegate {
    type Item = ListItem;

    fn items_count(&self, _: usize, _: &App) -> usize {
        self.rows.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let channel = self.rows.get(ix.row)?;
        // ids are only unique per table, so qualify them by kind
        let id = SharedString::from(format!(
            "row-{}-{}",
            channel.media_type,
            channel.id.unwrap_or_default()
        ));
        let subtitle = match (kind_label(channel.media_type), channel.group.as_deref()) {
            (Some(kind), Some(group)) => format!("{kind} · {group}"),
            (Some(kind), None) => kind.to_string(),
            (None, Some(group)) => group.to_string(),
            (None, None) => String::new(),
        };
        let favorite = channel.favorite;
        let can_favorite = is_favoritable(channel.media_type);
        let star_id = SharedString::from(format!("{id}-fav"));
        Some(
            ListItem::new(ElementId::Name(id))
                .py_1()
                .selected(self.selected == Some(ix))
                .child(
                    h_flex()
                        .gap_3()
                        .w_full()
                        .child(Self::render_logo(channel, cx))
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .child(div().truncate().child(channel.name.clone()))
                                .when(!subtitle.is_empty(), |el| {
                                    el.child(
                                        div()
                                            .truncate()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(subtitle),
                                    )
                                }),
                        )
                        .when(can_favorite, |el| {
                            el.child(
                                Button::new(ElementId::Name(star_id))
                                    .ghost()
                                    .small()
                                    .icon(if favorite { IconName::StarFill } else { IconName::Star })
                                    .when(favorite, |b| b.text_color(cx.theme().warning))
                                    .tooltip(if favorite { "Remove from favourites" } else { "Add to favourites" })
                                    .on_click(cx.listener(move |list, _: &ClickEvent, _, cx| {
                                        // the row itself would otherwise start playing
                                        cx.stop_propagation();
                                        list.delegate_mut().toggle_favorite(ix, cx);
                                    })),
                            )
                        }),
                ),
        )
    }

    fn render_empty(&mut self, _: &mut Window, cx: &mut Context<ListState<Self>>) -> impl IntoElement {
        let message: SharedString = self
            .error
            .clone()
            .unwrap_or_else(|| "Nothing here. Try another search or view.".into());
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_2()
            .text_color(cx.theme().muted_foreground)
            .child(Icon::new(IconName::Inbox).size_10())
            .child(message)
    }

    fn loading(&self, _: &App) -> bool {
        self.loading_first
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected = ix;
        cx.notify();
    }

    fn has_more(&self, _: &App) -> bool {
        !self.end_reached
            && !self.loading_first
            && !self.loading_more
            && self.filters.as_ref().is_some_and(|f| f.page < MAX_PAGE)
    }

    fn load_more_threshold(&self) -> usize {
        PAGE_SIZE / 2
    }

    fn load_more(&mut self, _: &mut Window, cx: &mut Context<ListState<Self>>) {
        if !self.has_more(cx) {
            return;
        }
        let Some(filters) = self.filters.as_mut() else {
            return;
        };
        filters.page += 1;
        let next = filters.clone();
        self.loading_more = true;
        self.fetch(next, true, cx);
    }
}
