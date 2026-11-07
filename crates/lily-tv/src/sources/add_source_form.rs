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
    m3u, share, source_type, sql,
    types::Source,
    xtream,
};

use crate::{backend::Backend, custom, file_prompts};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Xtream,
    M3uLink,
    M3uFile,
    /// An empty source to fill with your own channels
    Custom,
    /// A source shared from Lily TV or Fred TV (.otvp)
    ImportFile,
}

const KINDS: [(Kind, &str); 5] = [
    (Kind::Xtream, "Xtream"),
    (Kind::M3uLink, "M3U link"),
    (Kind::M3uFile, "M3U file"),
    (Kind::Custom, "Custom"),
    (Kind::ImportFile, "Import shared file"),
];

/// Emitted once a source has been imported.
pub struct SourceAdded;

/// What the form will create.
enum Request {
    Fetch(Source),
    Custom(String),
    Import { path: String, name: Option<String> },
}

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

    fn kind(&self) -> Kind {
        KINDS[self.kind].0
    }

    fn pick_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = match self.kind() {
            Kind::ImportFile => "Choose a shared source file (.otvp)",
            _ => "Choose an M3U playlist",
        };
        let picked = file_prompts::open_file(cx, prompt);
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

    /// Works out what to create from the form, or explains what is missing.
    fn read_request(&self, cx: &App) -> Result<Request, SharedString> {
        let text = |input: &Entity<InputState>| input.read(cx).value().trim().to_string();
        let name = text(&self.name);
        let kind = self.kind();
        if name.is_empty() && kind != Kind::ImportFile {
            return Err("Give the source a name.".into());
        }
        let (source_type, url, username, password) = match kind {
            Kind::Custom => return Ok(Request::Custom(name)),
            Kind::ImportFile => {
                return match &self.file {
                    Some(path) => Ok(Request::Import {
                        path: path.clone(),
                        name: Some(name).filter(|n| !n.is_empty()),
                    }),
                    None => Err("Choose the file to import.".into()),
                };
            }
            Kind::M3uFile => match &self.file {
                Some(file) => (source_type::M3U, Some(file.clone()), None, None),
                None => return Err("Choose a playlist file.".into()),
            },
            Kind::M3uLink => {
                let url = text(&self.url);
                if url.is_empty() {
                    return Err("Enter the playlist URL.".into());
                }
                (source_type::M3U_LINK, Some(url), None, None)
            }
            Kind::Xtream => {
                let url = xtream::normalize_xtream_url(&text(&self.url))
                    .map_err(|_| SharedString::from("Enter the server address, e.g. provider.example:8080"))?;
                let (username, password) = (text(&self.username), text(&self.password));
                if username.is_empty() || password.is_empty() {
                    return Err("Enter your username and password.".into());
                }
                (source_type::XTREAM, Some(url), Some(username), Some(password))
            }
        };
        Ok(Request::Fetch(Source {
            id: None,
            name,
            url,
            url_origin: None,
            username,
            password,
            source_type,
            use_tvg_id: (source_type != source_type::XTREAM).then_some(false),
            enabled: true,
            user_agent: None,
            max_streams: None,
            stream_user_agent: None,
            last_updated: None,
        }))
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.importing {
            return;
        }
        let request = match self.read_request(cx) {
            Ok(request) => request,
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
            let name = match &request {
                Request::Fetch(source) => Some(source.name.clone()),
                Request::Custom(name) => Some(name.clone()),
                Request::Import { name, .. } => name.clone(),
            };
            if let Some(name) = name {
                if sql::source_name_exists(&name)? {
                    bail!("A source called “{name}” already exists.");
                }
            }
            match request {
                Request::Custom(name) => custom::create_source(name),
                Request::Import { path, name } => {
                    tokio::task::spawn_blocking(move || share::import(path, None, name)).await?
                }
                Request::Fetch(source) => match source.source_type {
                    source_type::M3U => tokio::task::spawn_blocking(move || m3u::read_m3u8(source, false)).await?,
                    source_type::M3U_LINK => m3u::get_m3u8_from_link(source, false).await,
                    _ => xtream::get_xtream(source, false).await,
                },
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
        let kind = self.kind();
        let importing = self.importing;
        let file_label: SharedString = self
            .file
            .clone()
            .map(Into::into)
            .unwrap_or_else(|| "No file chosen".into());
        let needs_file = kind == Kind::M3uFile || kind == Kind::ImportFile;
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
                                    this.file = None;
                                    this.error = None;
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(
                        Field::new()
                            .label("Name")
                            .when(kind == Kind::ImportFile, |f| {
                                f.description("Leave empty to keep the name stored in the file.")
                            })
                            .when(kind == Kind::Custom, |f| {
                                f.description("Add channels and groups to it afterwards from the Sources tab.")
                            })
                            .child(Input::new(&self.name)),
                    )
                    .when(needs_file, |form| {
                        form.child(
                            Field::new().label(if kind == Kind::ImportFile { "File" } else { "Playlist" }).child(
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
                    .when(kind == Kind::M3uLink, |form| {
                        form.child(Field::new().label("Playlist URL").child(Input::new(&self.url)))
                    })
                    .when(kind == Kind::Xtream, |form| {
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
