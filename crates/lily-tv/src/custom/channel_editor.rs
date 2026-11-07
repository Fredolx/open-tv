use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, IndexPath,
    button::{Button, ButtonVariants as _},
    form::{Field, Form},
    h_flex,
    input::{Input, InputState},
    radio::RadioGroup,
    select::{Select, SelectState},
    switch::Switch,
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{
    anyhow::bail,
    media_type, sql,
    types::{Channel, ChannelHttpHeaders, CustomChannel, IdName},
};

use super::EditorEvent;
use crate::backend::Backend;

const KINDS: [(u8, &str); 2] = [(media_type::LIVESTREAM, "Live"), (media_type::MOVIE, "Movie")];
const NO_GROUP: &str = "No group";

/// Adds a channel to a custom source, or edits one already there.
pub struct ChannelEditor {
    /// The channel being edited; None when adding
    existing: Option<Channel>,
    source_id: i64,
    name: Entity<InputState>,
    url: Entity<InputState>,
    image: Entity<InputState>,
    kind: usize,
    groups: Vec<IdName>,
    group: Entity<SelectState<Vec<SharedString>>>,
    referrer: Entity<InputState>,
    user_agent: Entity<InputState>,
    origin: Entity<InputState>,
    ignore_ssl: bool,
    saving: bool,
    error: Option<SharedString>,
}

impl EventEmitter<EditorEvent> for ChannelEditor {}

impl ChannelEditor {
    pub fn add(source_id: i64, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::build(None, source_id, window, cx)
    }

    pub fn edit(channel: Channel, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let source_id = channel.source_id.unwrap_or_default();
        Self::build(Some(channel), source_id, window, cx)
    }

    fn build(existing: Option<Channel>, source_id: i64, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text = |value: Option<&String>| value.cloned().unwrap_or_default();
        let ch = existing.as_ref();
        let name = text(ch.map(|c| &c.name));
        let url = text(ch.and_then(|c| c.url.as_ref()));
        let image = text(ch.and_then(|c| c.image.as_ref()));
        let kind = ch
            .and_then(|c| KINDS.iter().position(|(k, _)| *k == c.media_type))
            .unwrap_or(0);
        let mut this = Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value(name)),
            url: cx.new(|cx| InputState::new(window, cx).placeholder("http://…").default_value(url)),
            image: cx.new(|cx| InputState::new(window, cx).placeholder("Logo URL (optional)").default_value(image)),
            kind,
            groups: Vec::new(),
            group: cx.new(|cx| {
                SelectState::new(vec![SharedString::from(NO_GROUP)], Some(IndexPath::new(0)), window, cx)
            }),
            referrer: cx.new(|cx| InputState::new(window, cx)),
            user_agent: cx.new(|cx| InputState::new(window, cx)),
            origin: cx.new(|cx| InputState::new(window, cx)),
            ignore_ssl: false,
            existing,
            source_id,
            saving: false,
            error: None,
        };
        this.load_extras(window, cx);
        this
    }

    /// Loads the source's groups and, when editing, the channel's headers.
    fn load_extras(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let source_id = self.source_id;
        let existing = self.existing.clone();
        let request = Backend::blocking(cx, move || {
            let groups = sql::group_auto_complete(None, source_id)?;
            let extra = match &existing {
                Some(c) => Some(sql::get_custom_channel_extra_data(c.id.unwrap_or_default(), c.group_id)?),
                None => None,
            };
            Ok((groups, extra))
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok((groups, extra)) => {
                        let current_group = this.existing.as_ref().and_then(|c| c.group_id);
                        let mut names = vec![SharedString::from(NO_GROUP)];
                        names.extend(groups.iter().map(|g| SharedString::from(g.name.clone())));
                        let selected = current_group
                            .and_then(|id| groups.iter().position(|g| g.id == id))
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        this.groups = groups;
                        this.group.update(cx, |select, cx| {
                            select.set_items(names, window, cx);
                            select.set_selected_index(Some(IndexPath::new(selected)), window, cx);
                        });
                        if let Some(headers) = extra.and_then(|e| e.headers) {
                            let set = |input: &Entity<InputState>, value: Option<String>, window: &mut Window, cx: &mut App| {
                                input.update(cx, |i, cx| i.set_value(value.unwrap_or_default(), window, cx))
                            };
                            set(&this.referrer, headers.referrer, window, cx);
                            set(&this.user_agent, headers.user_agent, window, cx);
                            set(&this.origin, headers.http_origin, window, cx);
                            this.ignore_ssl = headers.ignore_ssl.unwrap_or(false);
                        }
                    }
                    Err(e) => this.error = Some(format!("Couldn't load groups: {e:#}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn read(&self, cx: &App) -> Result<CustomChannel, SharedString> {
        let text = |input: &Entity<InputState>| input.read(cx).value().trim().to_string();
        let optional = |input: &Entity<InputState>| Some(text(input)).filter(|s| !s.is_empty());
        let name = text(&self.name);
        if name.is_empty() {
            return Err("Give the channel a name.".into());
        }
        let Some(url) = optional(&self.url) else {
            return Err("Enter the stream URL.".into());
        };
        let group_id = self
            .group
            .read(cx)
            .selected_index(cx)
            .filter(|ix| ix.row > 0)
            .and_then(|ix| self.groups.get(ix.row - 1))
            .map(|g| g.id);
        let headers = ChannelHttpHeaders {
            id: None,
            channel_id: self.existing.as_ref().and_then(|c| c.id),
            referrer: optional(&self.referrer),
            user_agent: optional(&self.user_agent),
            http_origin: optional(&self.origin),
            ignore_ssl: self.ignore_ssl.then_some(true),
        };
        let has_headers = headers.referrer.is_some()
            || headers.user_agent.is_some()
            || headers.http_origin.is_some()
            || headers.ignore_ssl.is_some();
        let base = self.existing.clone();
        Ok(CustomChannel {
            data: Channel {
                id: base.as_ref().and_then(|c| c.id),
                name,
                url: Some(url),
                group: None,
                image: optional(&self.image),
                media_type: KINDS[self.kind].0,
                source_id: Some(self.source_id),
                series_id: None,
                group_id,
                favorite: base.as_ref().is_some_and(|c| c.favorite),
                stream_id: None,
                tv_archive: None,
                season_id: None,
                episode_num: None,
                hidden: Some(false),
            },
            headers: has_headers.then_some(headers),
        })
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        let channel = match self.read(cx) {
            Ok(channel) => channel,
            Err(message) => {
                self.error = Some(message);
                cx.notify();
                return;
            }
        };
        self.saving = true;
        self.error = None;
        cx.notify();
        let adding = self.existing.is_none();
        let request = Backend::blocking(cx, move || {
            if adding {
                let data = &channel.data;
                if sql::channel_exists(&data.name, data.url.as_deref().unwrap_or_default(), channel.data.source_id.unwrap_or_default())? {
                    bail!("This source already has a channel with that name and URL.");
                }
                sql::do_tx(|tx| sql::add_custom_channel(tx, channel))
            } else {
                sql::edit_custom_channel(channel)
            }
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update(cx, |this, cx| {
                this.saving = false;
                match result {
                    Ok(()) => cx.emit(EditorEvent::Saved),
                    Err(e) => this.error = Some(format!("{e:#}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for ChannelEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let saving = self.saving;
        let ignore_ssl = self.ignore_ssl;
        v_flex()
            .gap_3()
            .child(
                Form::new()
                    .child(Field::new().label("Name").child(Input::new(&self.name)))
                    .child(Field::new().label("Stream URL").child(Input::new(&self.url)))
                    .child(Field::new().label("Logo").child(Input::new(&self.image)))
                    .child(
                        Field::new().label("Type").child(
                            RadioGroup::horizontal("channel-kind")
                                .children(KINDS.iter().map(|(_, l)| *l))
                                .selected_index(Some(self.kind))
                                .on_change(cx.listener(|this, ix: &usize, _, cx| {
                                    this.kind = *ix;
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(Field::new().label("Group").child(div().w_64().child(Select::new(&self.group))))
                    .child(Field::new().label("Referer header").child(Input::new(&self.referrer)))
                    .child(Field::new().label("User agent").child(Input::new(&self.user_agent)))
                    .child(Field::new().label("Origin header").child(Input::new(&self.origin)))
                    .child(
                        Field::new().label_indent(false).child(
                            Switch::new("channel-ignore-ssl")
                                .label("Ignore certificate errors")
                                .checked(ignore_ssl)
                                .on_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.ignore_ssl = *v;
                                    cx.notify();
                                })),
                        ),
                    ),
            )
            .when_some(self.error.clone(), |el, e| el.child(div().text_color(cx.theme().danger).child(e)))
            .child(
                h_flex()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("channel-cancel")
                            .ghost()
                            .label("Cancel")
                            .disabled(saving)
                            .on_click(cx.listener(|_, _: &ClickEvent, _, cx| cx.emit(EditorEvent::Cancelled))),
                    )
                    .child(
                        Button::new("channel-save")
                            .primary()
                            .label(if self.existing.is_some() { "Save" } else { "Add channel" })
                            .loading(saving)
                            .disabled(saving)
                            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.save(window, cx))),
                    ),
            )
    }
}
