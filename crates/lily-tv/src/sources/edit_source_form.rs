use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, IconName,
    button::{Button, ButtonVariants as _},
    form::{Field, Form},
    h_flex,
    input::{Input, InputState},
    switch::Switch,
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{source_type, sql, types::Source, utils, xtream};

use crate::backend::Backend;

/// Placeholder shown for an empty fetch user agent; the backend falls back to it.
const DEFAULT_USER_AGENT: &str = "Lily TV";

pub enum EditSourceEvent {
    /// Saved; `refreshed` says whether channels were re-fetched to apply new details.
    Saved { name: String, refreshed: bool },
    Cancelled,
}

/// Inline editor for an existing source, covering every field that can change after adding it.
pub struct EditSourceForm {
    original: Source,
    url: Entity<InputState>,
    file: Option<String>,
    username: Entity<InputState>,
    password: Entity<InputState>,
    user_agent: Entity<InputState>,
    stream_user_agent: Entity<InputState>,
    max_streams: Entity<InputState>,
    use_tvg_id: bool,
    saving: bool,
    error: Option<SharedString>,
}

impl EventEmitter<EditSourceEvent> for EditSourceForm {}

impl EditSourceForm {
    pub fn new(source: Source, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text = |value: &Option<String>| value.clone().unwrap_or_default();
        let url = text(&source.url);
        let username = text(&source.username);
        let password = text(&source.password);
        let user_agent = text(&source.user_agent);
        let stream_user_agent = text(&source.stream_user_agent);
        let max_streams = source.max_streams.map(|n| n.to_string()).unwrap_or_default();
        Self {
            file: (source.source_type == source_type::M3U).then(|| url.clone()),
            url: cx.new(|cx| InputState::new(window, cx).default_value(url)),
            username: cx.new(|cx| InputState::new(window, cx).default_value(username)),
            password: cx.new(|cx| InputState::new(window, cx).masked(true).default_value(password)),
            user_agent: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(DEFAULT_USER_AGENT)
                    .default_value(user_agent)
            }),
            stream_user_agent: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Player default")
                    .default_value(stream_user_agent)
            }),
            max_streams: cx.new(|cx| InputState::new(window, cx).placeholder("1").default_value(max_streams)),
            use_tvg_id: source.use_tvg_id.unwrap_or(false),
            original: source,
            saving: false,
            error: None,
        }
    }

    fn kind(&self) -> u8 {
        self.original.source_type
    }

    fn pick_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = crate::file_prompts::open_file(cx, "Choose an M3U playlist");
        cx.spawn_in(window, async move |this, cx| {
            let Some(path) = picked.await else {
                return;
            };
            _ = this.update(cx, |this, cx| {
                this.file = Some(path);
                this.error = None;
                cx.notify();
            });
        })
        .detach();
    }

    /// The edited source, or what needs fixing.
    fn read_source(&self, cx: &App) -> Result<Source, SharedString> {
        let text = |input: &Entity<InputState>| input.read(cx).value().trim().to_string();
        let optional = |input: &Entity<InputState>| Some(text(input)).filter(|s| !s.is_empty());
        let kind = self.kind();
        let mut source = self.original.clone();

        source.url = match kind {
            source_type::M3U => self.file.clone(),
            source_type::M3U_LINK => match optional(&self.url) {
                Some(url) => Some(url),
                None => return Err("Enter the playlist URL.".into()),
            },
            source_type::XTREAM => Some(
                xtream::normalize_xtream_url(&text(&self.url))
                    .map_err(|_| SharedString::from("Enter the server address, e.g. provider.example:8080"))?,
            ),
            _ => source.url,
        };
        if kind == source_type::XTREAM {
            source.username = optional(&self.username);
            source.password = optional(&self.password);
            if source.username.is_none() || source.password.is_none() {
                return Err("Enter your username and password.".into());
            }
        }
        if kind != source_type::M3U && kind != source_type::CUSTOM {
            source.user_agent = optional(&self.user_agent);
        }
        source.stream_user_agent = optional(&self.stream_user_agent);
        if kind != source_type::CUSTOM {
            source.max_streams = match optional(&self.max_streams) {
                None => None,
                Some(n) => match n.parse::<u8>() {
                    Ok(n) if n >= 1 => Some(n),
                    _ => return Err("Max streams must be a number from 1 to 255.".into()),
                },
            };
        }
        if kind == source_type::M3U || kind == source_type::M3U_LINK {
            source.use_tvg_id = Some(self.use_tvg_id);
        }
        Ok(source)
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        let source = match self.read_source(cx) {
            Ok(source) => source,
            Err(message) => {
                self.error = Some(message);
                cx.notify();
                return;
            }
        };
        // Stream URLs embed the server and credentials, so those changes need a refresh
        let original = &self.original;
        let refresh = source.url != original.url
            || source.username != original.username
            || source.password != original.password
            || source.use_tvg_id != original.use_tvg_id;
        self.saving = true;
        self.error = None;
        cx.notify();

        let name = source.name.clone();
        let saved = source.clone();
        let request = Backend::run(cx, move |_| async move {
            sql::update_source(source.clone())?;
            if refresh {
                utils::refresh_source(source).await?;
            }
            Ok(())
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update(cx, |this, cx| {
                this.saving = false;
                match result {
                    Ok(()) => {
                        this.original = saved;
                        cx.emit(EditSourceEvent::Saved { name, refreshed: refresh });
                    }
                    Err(e) => {
                        this.error = Some(if refresh {
                            format!("Saved, but refreshing with the new details failed: {e:#}").into()
                        } else {
                            format!("Couldn't save: {e:#}").into()
                        });
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for EditSourceForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let kind = self.kind();
        let saving = self.saving;
        let id = self.original.id.unwrap_or_default();
        let file_label: SharedString = self.file.clone().map(Into::into).unwrap_or_else(|| "No file chosen".into());
        let use_tvg_id = self.use_tvg_id;
        v_flex()
            .gap_3()
            .child(
                Form::new()
                    .when(kind == source_type::M3U, |form| {
                        form.child(
                            Field::new().label("Playlist").child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Button::new(ElementId::Name(format!("edit-{id}-file").into()))
                                            .outline()
                                            .icon(IconName::FolderOpen)
                                            .label("Choose file…")
                                            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                                                this.pick_file(window, cx)
                                            })),
                                    )
                                    .child(
                                        div()
                                            .min_w_0()
                                            .truncate()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(file_label),
                                    ),
                            ),
                        )
                    })
                    .when(kind == source_type::M3U_LINK, |form| {
                        form.child(Field::new().label("Playlist URL").child(Input::new(&self.url)))
                    })
                    .when(kind == source_type::XTREAM, |form| {
                        form.child(Field::new().label("Server").child(Input::new(&self.url)))
                            .child(Field::new().label("Username").child(Input::new(&self.username)))
                            .child(
                                Field::new()
                                    .label("Password")
                                    .child(Input::new(&self.password).mask_toggle()),
                            )
                    })
                    .when(kind != source_type::M3U && kind != source_type::CUSTOM, |form| {
                        form.child(
                            Field::new()
                                .label("User agent")
                                .description("Sent when downloading the channel list.")
                                .child(Input::new(&self.user_agent)),
                        )
                    })
                    .child(
                        Field::new()
                            .label("Stream user agent")
                            .description("Sent by mpv when playing. Leave empty for the player default.")
                            .child(Input::new(&self.stream_user_agent)),
                    )
                    .when(kind != source_type::CUSTOM, |form| {
                        form.child(
                            Field::new()
                                .label("Max streams")
                                .description("Opening one more than this closes the oldest stream.")
                                .child(div().w_24().child(Input::new(&self.max_streams))),
                        )
                    })
                    .when(kind == source_type::M3U || kind == source_type::M3U_LINK, |form| {
                        form.child(
                            Field::new().label_indent(false).child(
                                Switch::new(ElementId::Name(format!("edit-{id}-tvg").into()))
                                    .label("Use TVG-ID for names when TVG-NAME is missing")
                                    .checked(use_tvg_id)
                                    .on_change(cx.listener(|this, value: &bool, _, cx| {
                                        this.use_tvg_id = *value;
                                        cx.notify();
                                    })),
                            ),
                        )
                    }),
            )
            .when_some(self.error.clone(), |el, error| {
                el.child(div().text_color(cx.theme().danger).child(error))
            })
            .child(
                h_flex()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new(ElementId::Name(format!("edit-{id}-cancel").into()))
                            .ghost()
                            .label("Cancel")
                            .disabled(saving)
                            .on_click(cx.listener(|_, _: &ClickEvent, _, cx| cx.emit(EditSourceEvent::Cancelled))),
                    )
                    .child(
                        Button::new(ElementId::Name(format!("edit-{id}-save").into()))
                            .primary()
                            .label(if saving { "Saving…" } else { "Save" })
                            .loading(saving)
                            .disabled(saving)
                            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.save(window, cx))),
                    ),
            )
    }
}
