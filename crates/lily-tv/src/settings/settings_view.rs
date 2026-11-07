use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, IconName, IndexPath, WindowExt as _,
    button::{Button, ButtonVariant, ButtonVariants as _},
    form::{Field, Form},
    h_flex,
    input::{Input, InputState},
    notification::Notification,
    select::{Select, SelectState},
    switch::Switch,
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{settings, sort_type, types::Settings, utils, view_type};

use crate::{backend::Backend, file_prompts};

const VIEWS: [(u8, &str); 4] = [
    (view_type::ALL, "All"),
    (view_type::FAVORITES, "Favourites"),
    (view_type::CATEGORIES, "Categories"),
    (view_type::HISTORY, "History"),
];
const SORTS: [(u8, &str); 3] = [
    (sort_type::PROVIDER, "Provider order"),
    (sort_type::ALPHABETICAL_ASC, "A to Z"),
    (sort_type::ALPHABETICAL_DESC, "Z to A"),
];
const ZOOMS: [u16; 8] = [75, 90, 100, 110, 125, 150, 175, 200];
const DEFAULT_RESTREAM_PORT: u16 = 3000;

/// Editable copy of the stored settings; fields without a control round-trip untouched.
pub struct SettingsView {
    loaded: Option<Settings>,
    mpv_params: Entity<InputState>,
    volume: Entity<InputState>,
    default_view: Entity<SelectState<Vec<&'static str>>>,
    default_sort: Entity<SelectState<Vec<&'static str>>>,
    stream_caching: bool,
    hwdec: bool,
    gpu: bool,
    refresh_on_start: bool,
    /// None means the default Videos/open-tv folder
    recording_path: Option<String>,
    always_ask_save: bool,
    restream_port: Entity<InputState>,
    zoom: Entity<SelectState<Vec<SharedString>>>,
    tray: bool,
    saving: bool,
    error: Option<SharedString>,
}

fn labels<const N: usize>(options: &[(u8, &'static str); N]) -> Vec<&'static str> {
    options.iter().map(|(_, label)| *label).collect()
}

fn index_of(options: &[(u8, &str)], value: Option<u8>) -> usize {
    value
        .and_then(|v| options.iter().position(|(o, _)| *o == v))
        .unwrap_or(0)
}

impl SettingsView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            loaded: None,
            mpv_params: cx.new(|cx| {
                InputState::new(window, cx).placeholder("Extra mpv arguments, e.g. --fs --ontop")
            }),
            volume: cx.new(|cx| InputState::new(window, cx).placeholder("100")),
            default_view: cx.new(|cx| {
                SelectState::new(labels(&VIEWS), Some(IndexPath::new(0)), window, cx)
            }),
            default_sort: cx.new(|cx| {
                SelectState::new(labels(&SORTS), Some(IndexPath::new(0)), window, cx)
            }),
            stream_caching: true,
            hwdec: true,
            gpu: false,
            refresh_on_start: false,
            recording_path: None,
            always_ask_save: false,
            restream_port: cx.new(|cx| InputState::new(window, cx).placeholder(DEFAULT_RESTREAM_PORT.to_string())),
            zoom: cx.new(|cx| {
                let labels = ZOOMS.iter().map(|z| SharedString::from(format!("{z}%"))).collect();
                SelectState::new(labels, Some(IndexPath::new(2)), window, cx)
            }),
            tray: true,
            saving: false,
            error: None,
        };
        this.load(window, cx);
        this
    }

    fn load(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let request = Backend::blocking(cx, settings::get_settings);
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(settings) => this.show(settings, window, cx),
                    Err(e) => this.error = Some(format!("Couldn't load settings: {e:#}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn show(&mut self, settings: Settings, window: &mut Window, cx: &mut Context<Self>) {
        let mpv = settings.mpv_params.clone().unwrap_or_default();
        self.mpv_params.update(cx, |input, cx| input.set_value(mpv, window, cx));
        let volume = settings.volume.unwrap_or(100).to_string();
        self.volume.update(cx, |input, cx| input.set_value(volume, window, cx));
        let view = index_of(&VIEWS, settings.default_view);
        self.default_view
            .update(cx, |select, cx| select.set_selected_index(Some(IndexPath::new(view)), window, cx));
        let sort = index_of(&SORTS, settings.default_sort);
        self.default_sort
            .update(cx, |select, cx| select.set_selected_index(Some(IndexPath::new(sort)), window, cx));
        self.stream_caching = settings.use_stream_caching.unwrap_or(true);
        self.hwdec = settings.enable_hwdec.unwrap_or(true);
        self.gpu = settings.enable_gpu.unwrap_or(false);
        self.refresh_on_start = settings.refresh_on_start.unwrap_or(false);
        self.recording_path = settings.recording_path.clone();
        self.always_ask_save = settings.always_ask_save.unwrap_or(false);
        self.tray = settings.enable_tray_icon.unwrap_or(true);
        let port = settings.restream_port.map(|p| p.to_string()).unwrap_or_default();
        self.restream_port.update(cx, |input, cx| input.set_value(port, window, cx));
        let zoom = ZOOMS.iter().position(|z| Some(*z) == settings.zoom).unwrap_or(2);
        self.zoom
            .update(cx, |select, cx| select.set_selected_index(Some(IndexPath::new(zoom)), window, cx));
        self.loaded = Some(settings);
    }

    fn collect(&self, cx: &App) -> Result<Settings, SharedString> {
        let Some(mut settings) = self.loaded.clone() else {
            return Err("Settings haven't loaded yet.".into());
        };
        let volume = self.volume.read(cx).value().trim().to_string();
        settings.volume = match volume.parse::<u8>() {
            Ok(v) if v <= 100 => Some(v),
            _ if volume.is_empty() => None,
            _ => return Err("Volume must be a number from 0 to 100.".into()),
        };
        let mpv = self.mpv_params.read(cx).value().trim().to_string();
        settings.mpv_params = (!mpv.is_empty()).then_some(mpv);
        let pick = |select: &Entity<SelectState<Vec<&'static str>>>, options: &[(u8, &str)]| {
            select
                .read(cx)
                .selected_index(cx)
                .and_then(|ix| options.get(ix.row))
                .map(|(v, _)| *v)
        };
        settings.default_view = pick(&self.default_view, &VIEWS);
        settings.default_sort = pick(&self.default_sort, &SORTS);
        settings.use_stream_caching = Some(self.stream_caching);
        settings.enable_hwdec = Some(self.hwdec);
        settings.enable_gpu = Some(self.gpu);
        settings.refresh_on_start = Some(self.refresh_on_start);
        settings.recording_path = self.recording_path.clone();
        settings.always_ask_save = Some(self.always_ask_save);
        settings.enable_tray_icon = Some(self.tray);
        let port = self.restream_port.read(cx).value().trim().to_string();
        settings.restream_port = match port.parse::<u16>() {
            Ok(p) if p >= 1024 => Some(p),
            _ if port.is_empty() => None,
            _ => return Err("Re-stream port must be a number from 1024 to 65535.".into()),
        };
        settings.zoom = self
            .zoom
            .read(cx)
            .selected_index(cx)
            .and_then(|ix| ZOOMS.get(ix.row))
            .copied();
        Ok(settings)
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        let settings = match self.collect(cx) {
            Ok(settings) => settings,
            Err(message) => {
                self.error = Some(message);
                cx.notify();
                return;
            }
        };
        self.saving = true;
        self.error = None;
        cx.notify();
        let stored = settings.clone();
        let request = Backend::blocking(cx, move || settings::update_settings(settings));
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update_in(cx, |this, window, cx| {
                this.saving = false;
                match result {
                    Ok(()) => {
                        super::apply_zoom(stored.zoom, cx);
                        this.loaded = Some(stored);
                        window.push_notification(Notification::success("Settings saved."), cx);
                    }
                    Err(e) => this.error = Some(format!("Couldn't save settings: {e:#}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn pick_recording_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = file_prompts::open_folder(cx, "Choose where recordings and downloads go");
        cx.spawn_in(window, async move |this, cx| {
            if let Some(folder) = picked.await {
                _ = this.update(cx, |this, cx| {
                    this.recording_path = Some(folder);
                    cx.notify();
                });
            }
        })
        .detach();
    }

    /// Fred TV's "Delete everything": the database is removed on the next start.
    fn confirm_reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.open_alert_dialog(cx, |alert, _, _| {
            alert
                .title("Delete everything?")
                .description(
                    "All sources, channels, favourites, history and settings are deleted and Lily TV restarts. \
                     Fred TV shares this data, so it is cleared there too. This can't be undone.",
                )
                .confirm()
                .ok_text("Delete everything")
                .ok_variant(ButtonVariant::Danger)
                .on_ok(|_, window, cx| {
                    match utils::create_nuke_request() {
                        Ok(()) => cx.restart(),
                        Err(e) => window.push_notification(
                            Notification::error(format!("{e:#}")).title("Couldn't reset"),
                            cx,
                        ),
                    }
                    true
                })
        });
    }

    fn switch(
        &self,
        id: &'static str,
        label: &'static str,
        checked: bool,
        set: fn(&mut Self, bool),
        cx: &mut Context<Self>,
    ) -> Field {
        Field::new().label_indent(false).child(
            Switch::new(id)
                .label(label)
                .checked(checked)
                .on_change(cx.listener(move |this, value: &bool, _, cx| {
                    set(this, *value);
                    cx.notify();
                })),
        )
    }
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let saving = self.saving;
        let recording_path = self.recording_path.clone();
        v_flex()
            .id("settings")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .child(div().text_lg().child("Settings"))
            .child(
                Form::new()
                    .child(
                        Field::new()
                            .label("Start in")
                            .child(div().w_56().child(Select::new(&self.default_view))),
                    )
                    .child(
                        Field::new()
                            .label("Sort by")
                            .child(div().w_56().child(Select::new(&self.default_sort))),
                    )
                    .child(
                        Field::new()
                            .label("Volume")
                            .description("Starting volume for mpv, 0 to 100.")
                            .child(div().w_24().child(Input::new(&self.volume))),
                    )
                    .child(
                        Field::new()
                            .label("mpv arguments")
                            .description("Added to every playback, separated by spaces.")
                            .child(Input::new(&self.mpv_params)),
                    )
                    .child(self.switch(
                        "caching",
                        "Stream caching (smoother on slow connections, more delay)",
                        self.stream_caching,
                        |this, v| this.stream_caching = v,
                        cx,
                    ))
                    .child(self.switch(
                        "hwdec",
                        "Hardware decoding",
                        self.hwdec,
                        |this, v| this.hwdec = v,
                        cx,
                    ))
                    .child(self.switch(
                        "gpu",
                        "High-quality GPU renderer (gpu-next)",
                        self.gpu,
                        |this, v| this.gpu = v,
                        cx,
                    ))
                    .child(self.switch(
                        "refresh",
                        "Refresh all sources when Lily TV starts",
                        self.refresh_on_start,
                        |this, v| this.refresh_on_start = v,
                        cx,
                    ))
                    .child(
                        Field::new()
                            .label("Recordings and downloads")
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .truncate()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(
                                                recording_path
                                                    .clone()
                                                    .unwrap_or_else(|| "Default (Videos\\open-tv)".to_string()),
                                            ),
                                    )
                                    .child(
                                        Button::new("recording-folder")
                                            .outline()
                                            .icon(IconName::FolderOpen)
                                            .label("Choose…")
                                            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                                                this.pick_recording_folder(window, cx)
                                            })),
                                    )
                                    .when(recording_path.is_some(), |el| {
                                        el.child(
                                            Button::new("recording-default")
                                                .ghost()
                                                .label("Use default")
                                                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                                    this.recording_path = None;
                                                    cx.notify();
                                                })),
                                        )
                                    }),
                            ),
                    )
                    .child(self.switch(
                        "ask-save",
                        "Always ask where to save recordings and downloads",
                        self.always_ask_save,
                        |this, v| this.always_ask_save = v,
                        cx,
                    ))
                    .child(
                        Field::new()
                            .label("Re-stream port")
                            .description("Other devices connect to this port when you re-stream.")
                            .child(div().w_24().child(Input::new(&self.restream_port))),
                    )
                    .child(
                        Field::new()
                            .label("Zoom")
                            .child(div().w_32().child(Select::new(&self.zoom))),
                    )
                    .child(self.switch(
                        "tray",
                        "Keep running in the system tray when the window is closed",
                        self.tray,
                        |this, v| this.tray = v,
                        cx,
                    )),
            )
            .when_some(self.error.clone(), |el, error| {
                el.child(div().text_color(cx.theme().danger).child(error))
            })
            .child(
                h_flex().child(
                    Button::new("save-settings")
                        .primary()
                        .label("Save")
                        .loading(saving)
                        .disabled(saving || self.loaded.is_none())
                        .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.save(window, cx))),
                ),
            )
            .child(
                v_flex()
                    .mt_6()
                    .pt_4()
                    .gap_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(div().text_color(cx.theme().danger).child("Danger zone"))
                    .child(
                        h_flex().child(
                            Button::new("delete-everything")
                                .danger()
                                .label("Delete everything…")
                                .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                                    this.confirm_reset(window, cx)
                                })),
                        ),
                    ),
            )
    }
}
