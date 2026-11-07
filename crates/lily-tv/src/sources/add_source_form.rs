use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, IconName,
    button::{Button, ButtonVariants as _},
    form::{Field, Form},
    h_flex,
    input::{Input, InputState},
    radio::RadioGroup,
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{
    anyhow::{Result, bail},
    m3u, source_type, sql,
    types::Source,
    xtream,
};

use crate::backend::Backend;

const KINDS: [(u8, &str); 3] = [
    (source_type::XTREAM, "Xtream"),
    (source_type::M3U_LINK, "M3U link"),
    (source_type::M3U, "M3U file"),
];

/// Opens the system file dialog for an M3U playlist; resolves to its path, or None if cancelled.
pub(super) fn prompt_for_playlist(cx: &mut App) -> impl Future<Output = Option<String>> + use<> {
    let paths = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some("Choose an M3U playlist".into()),
    });
    async move {
        let path = paths.await.ok()?.ok()??.into_iter().next()?;
        Some(path.to_string_lossy().to_string())
    }
}

/// Emitted once a source has been imported.
pub struct SourceAdded;

pub struct AddSourceForm {
    kind: usize,
    name: Entity<InputState>,
    url: Entity<InputState>,
    username: Entity<InputState>,
    password: Entity<InputState>,
    file: Option<String>,
    importing: bool,
    error: Option<SharedString>,
}

impl EventEmitter<SourceAdded> for AddSourceForm {}

impl AddSourceForm {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            kind: 0,
            name: cx.new(|cx| InputState::new(window, cx).placeholder("e.g. My provider")),
            url: cx.new(|cx| InputState::new(window, cx).placeholder("http://provider.example:8080")),
            username: cx.new(|cx| InputState::new(window, cx)),
            password: cx.new(|cx| InputState::new(window, cx).masked(true)),
            file: None,
            importing: false,
            error: None,
        }
    }

    fn source_type(&self) -> u8 {
        KINDS[self.kind].0
    }

    fn pick_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = prompt_for_playlist(cx);
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

    /// Builds the source from the form, or explains what is missing.
    fn read_source(&self, cx: &App) -> Result<Source, SharedString> {
        let text = |input: &Entity<InputState>| input.read(cx).value().trim().to_string();
        let name = text(&self.name);
        if name.is_empty() {
            return Err("Give the source a name.".into());
        }
        let kind = self.source_type();
        let (url, username, password) = match kind {
            source_type::M3U => match &self.file {
                Some(file) => (Some(file.clone()), None, None),
                None => return Err("Choose a playlist file.".into()),
            },
            source_type::M3U_LINK => {
                let url = text(&self.url);
                if url.is_empty() {
                    return Err("Enter the playlist URL.".into());
                }
                (Some(url), None, None)
            }
            _ => {
                let url = xtream::normalize_xtream_url(&text(&self.url))
                    .map_err(|_| SharedString::from("Enter the server address, e.g. provider.example:8080"))?;
                let (username, password) = (text(&self.username), text(&self.password));
                if username.is_empty() || password.is_empty() {
                    return Err("Enter your username and password.".into());
                }
                (Some(url), Some(username), Some(password))
            }
        };
        Ok(Source {
            id: None,
            name,
            url,
            url_origin: None,
            username,
            password,
            source_type: kind,
            use_tvg_id: (kind != source_type::XTREAM).then_some(false),
            enabled: true,
            user_agent: None,
            max_streams: None,
            stream_user_agent: None,
            last_updated: None,
        })
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.importing {
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
        self.importing = true;
        self.error = None;
        cx.notify();

        let import = Backend::run(cx, move |_| async move {
            if sql::source_name_exists(&source.name)? {
                bail!("A source called “{}” already exists.", source.name);
            }
            match source.source_type {
                source_type::M3U => {
                    tokio::task::spawn_blocking(move || m3u::read_m3u8(source, false)).await?
                }
                source_type::M3U_LINK => m3u::get_m3u8_from_link(source, false).await,
                _ => xtream::get_xtream(source, false).await,
            }
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = import.await;
            _ = this.update_in(cx, |this, window, cx| {
                this.importing = false;
                match result {
                    Ok(()) => {
                        this.reset(window, cx);
                        cx.emit(SourceAdded);
                    }
                    Err(e) => this.error = Some(format!("{e:#}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for input in [&self.name, &self.url, &self.username, &self.password] {
            input.update(cx, |input, cx| input.set_value("", window, cx));
        }
        self.file = None;
    }
}

impl Render for AddSourceForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let kind = self.source_type();
        let importing = self.importing;
        let file_label: SharedString = self
            .file
            .clone()
            .map(Into::into)
            .unwrap_or_else(|| "No file chosen".into());
        v_flex()
            .gap_3()
            .child(
                Form::new()
                    .child(
                        Field::new().label("Type").child(
                            RadioGroup::horizontal("source-kind")
                                .children(KINDS.iter().map(|(_, label)| *label))
                                .selected_index(Some(self.kind))
                                .on_change(cx.listener(|this, ix: &usize, _, cx| {
                                    this.kind = *ix;
                                    this.error = None;
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(Field::new().label("Name").child(Input::new(&self.name)))
                    .when(kind == source_type::M3U, |form| {
                        form.child(
                            Field::new().label("Playlist").child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Button::new("pick-file")
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
                    }),
            )
            .when_some(self.error.clone(), |el, error| {
                el.child(div().text_color(cx.theme().danger).child(error))
            })
            .child(
                h_flex().justify_end().child(
                    Button::new("import")
                        .primary()
                        .label(if importing { "Importing…" } else { "Add source" })
                        .loading(importing)
                        .disabled(importing)
                        .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.submit(window, cx))),
                ),
            )
    }
}
