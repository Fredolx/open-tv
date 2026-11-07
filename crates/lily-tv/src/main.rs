// Hide the console window in release builds on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backend;
mod browse;
mod custom;
mod desktop;
mod downloads;
mod file_prompts;
mod guide;
mod naming;
mod restream;
mod settings;
mod shell;
mod sources;

use std::sync::Arc;

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;

const APP_NAME: &str = "Lily TV";

fn main() -> anyhow::Result<()> {
    if let Err(e) = lily_core::utils::check_nuke() {
        lily_core::log::log(format!("Failed to delete db after nuke request: {e:?}"));
    }
    // a second launch just brings the running window forward
    let Some(instance) = desktop::SingleInstance::claim() else {
        return Ok(());
    };
    lily_core::sql::create_or_initialize_db()?;
    let startup_settings = lily_core::settings::get_settings().ok();

    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Dark, None, cx);
            settings::apply_zoom(startup_settings.as_ref().and_then(|s| s.zoom), cx);
            backend::Backend::init(cx);
            downloads::Downloads::init(cx);
            restream::Restream::init(cx);
            browse::init(cx);
            // channel logos are remote images
            match reqwest_client::ReqwestClient::user_agent(APP_NAME) {
                Ok(client) => cx.set_http_client(Arc::new(client)),
                Err(e) => lily_core::log::log(format!("no HTTP client, logos disabled: {e:?}")),
            }

            let bounds = Bounds::centered(None, size(px(1100.), px(760.)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(APP_NAME.into()),
                    ..Default::default()
                }),
                window_min_size: Some(size(px(640.), px(420.))),
                ..Default::default()
            };
            let tray = startup_settings.as_ref().and_then(|s| s.enable_tray_icon).unwrap_or(true);
            let (window, _) = gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| shell::Shell::new(window, cx))
            })
            .expect("failed to open window");
            let raw = window
                .update(cx, |_, window, cx| {
                    desktop::install_tray(tray, window, cx);
                    desktop::window_raw(window)
                })
                .ok()
                .flatten();
            instance.on_activate(
                move |_| {
                    if let Some(raw) = raw {
                        desktop::show_window(raw);
                    }
                },
                cx,
            );
            cx.activate(true);
        });
    Ok(())
}
