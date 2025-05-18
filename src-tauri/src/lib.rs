use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicBool, Ordering},
};

use anyhow::{Context, Error};
use tauri::{
    AppHandle, Manager, State,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tokio::sync::Mutex;
use types::{
    AppState, Channel, CustomChannel, CustomChannelExtraData, EPG, EPGNotify, Filters, Group,
    IdName, NetworkInfo, Settings, Source,
};

pub mod epg;
pub mod log;
pub mod m3u;
pub mod media_type;
pub mod mpv;
pub mod restream;
pub mod settings;
pub mod share;
pub mod sort_type;
pub mod source_type;
pub mod sql;
pub mod stalker;
pub mod types;
pub mod utils;
pub mod view_type;
pub mod xtream;

static ENABLE_TRAY_ICON: LazyLock<bool> = LazyLock::new(|| {
    settings::get_settings()
        .and_then(|s| s.enable_tray_icon.context("no value"))
        .unwrap_or(true)
});

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let window = app.get_webview_window("main").expect("no main window");
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_m3u8,
            get_m3u8_from_link,
            play,
            get_settings,
            update_settings,
            search,
            get_xtream,
            refresh_source,
            get_episodes,
            favorite_channel,
            unfavorite_channel,
            source_name_exists,
            get_sources,
            delete_source,
            refresh_all,
            get_enabled_sources,
            toggle_source,
            delete_database,
            add_custom_channel,
            get_custom_channel_extra_data,
            edit_custom_channel,
            delete_custom_channel,
            add_custom_source,
            share_custom_channel,
            group_auto_complete,
            edit_custom_channel,
            edit_custom_group,
            add_custom_group,
            delete_custom_group,
            group_not_empty,
            group_exists,
            share_custom_group,
            share_custom_source,
            import,
            channel_exists,
            update_source,
            get_epg,
            download,
            add_epg,
            remove_epg,
            get_epg_ids,
            on_start_check_epg,
            start_restream,
            stop_restream,
            watch_self,
            get_network_info,
            share_restream,
            add_last_watched,
            backup_favs,
            restore_favs,
            abort_download,
            clear_history,
            is_container
        ])
        .setup(|app| {
            app.manage(Mutex::new(AppState {
                ..Default::default()
            }));
            if *ENABLE_TRAY_ICON {
                let _ = build_tray_icon(app);
            }
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if !*ENABLE_TRAY_ICON {
                    return;
                }
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| match event {
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                if !*ENABLE_TRAY_ICON {
                    return;
                }
                let window = _app.get_webview_window("main").expect("no main window");
                let _ = window.show();
                let _ = window.set_focus();
            }
            _ => {}
        });
}

fn build_tray_icon(app: &mut tauri::App) -> anyhow::Result<()> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;
    TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;
    Ok(())
}

fn map_err_frontend(e: Error) -> String {
    return format!("{:?}", e);
}

#[tauri::command(async)]
fn get_m3u8(source: Source) -> Result<(), String> {
    m3u::read_m3u8(source, false).map_err(map_err_frontend)
}

#[tauri::command]
async fn get_m3u8_from_link(source: Source) -> Result<(), String> {
    m3u::get_m3u8_from_link(source, false)
        .await
        .map_err(map_err_frontend)
}

#[tauri::command]
async fn play(channel: Channel, record: bool, record_path: Option<String>) -> Result<(), String> {
    mpv::play(channel, record, record_path)
        .await
        .map_err(map_err_frontend)
}

#[tauri::command(async)]
fn get_settings() -> Result<Settings, String> {
    settings::get_settings().map_err(map_err_frontend)
}

#[tauri::command(async)]
fn update_settings(settings: Settings) -> Result<(), String> {
    settings::update_settings(settings).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn search(filters: Filters) -> Result<Vec<Channel>, String> {
    sql::search(filters).map_err(map_err_frontend)
}

#[tauri::command]
async fn get_xtream(source: Source) -> Result<(), String> {
    xtream::get_xtream(source, false)
        .await
        .map_err(map_err_frontend)
}

#[tauri::command]
async fn refresh_source(source: Source) -> Result<(), String> {
    utils::refresh_source(source)
        .await
        .map_err(map_err_frontend)
}

#[tauri::command]
async fn refresh_all() -> Result<(), String> {
    utils::refresh_all().await.map_err(map_err_frontend)
}

#[tauri::command]
async fn get_episodes(channel: Channel) -> Result<(), String> {
    xtream::get_episodes(channel)
        .await
        .map_err(map_err_frontend)
}

#[tauri::command(async)]
fn favorite_channel(channel_id: i64) -> Result<(), String> {
    sql::favorite_channel(channel_id, true).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn unfavorite_channel(channel_id: i64) -> Result<(), String> {
    sql::favorite_channel(channel_id, false).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn source_name_exists(name: String) -> Result<bool, String> {
    sql::source_name_exists(&name).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn get_sources() -> Result<Vec<Source>, String> {
    sql::get_sources().map_err(map_err_frontend)
}

#[tauri::command(async)]
fn get_enabled_sources() -> Result<Vec<Source>, String> {
    sql::get_enabled_sources().map_err(map_err_frontend)
}

#[tauri::command(async)]
fn delete_source(id: i64) -> Result<(), String> {
    sql::delete_source(id).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn toggle_source(value: bool, source_id: i64) -> Result<(), String> {
    sql::set_source_enabled(value, source_id).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn delete_database() -> Result<(), String> {
    sql::delete_database().map_err(map_err_frontend)
}

#[tauri::command(async)]
fn add_custom_channel(channel: CustomChannel) -> Result<(), String> {
    sql::do_tx(|tx| sql::add_custom_channel(tx, channel)).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn edit_custom_channel(channel: CustomChannel) -> Result<(), String> {
    sql::edit_custom_channel(channel).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn delete_custom_channel(id: i64) -> Result<(), String> {
    sql::delete_custom_channel(id).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn get_custom_channel_extra_data(
    id: i64,
    group_id: Option<i64>,
) -> Result<CustomChannelExtraData, String> {
    sql::get_custom_channel_extra_data(id, group_id).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn add_custom_source(name: String) -> Result<(), String> {
    sql::do_tx(|tx| sql::create_or_find_source_by_name(tx, &mut sql::get_custom_source(name)))
        .map_err(map_err_frontend)?;
    Ok(())
}

#[tauri::command(async)]
fn share_custom_channel(channel: Channel, path: String) -> Result<(), String> {
    share::share_custom_channel(channel, path).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn group_auto_complete(query: Option<String>, source_id: i64) -> Result<Vec<IdName>, String> {
    sql::group_auto_complete(query, source_id).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn edit_custom_group(group: Group) -> Result<(), String> {
    sql::edit_custom_group(group).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn add_custom_group(group: Group) -> Result<(), String> {
    sql::do_tx(|tx| {
        sql::add_custom_group(tx, group)?;
        Ok(())
    })
    .map_err(map_err_frontend)
}

#[tauri::command(async)]
fn delete_custom_group(
    id: i64,
    new_id: Option<i64>,
    do_channels_update: bool,
) -> Result<(), String> {
    sql::delete_custom_group(id, new_id, do_channels_update).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn group_not_empty(id: i64) -> Result<bool, String> {
    sql::group_not_empty(id).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn group_exists(name: String, source_id: i64) -> Result<bool, String> {
    sql::group_exists(&name, source_id).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn share_custom_group(group: Channel, path: String) -> Result<(), String> {
    share::share_custom_group(group, path).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn share_custom_source(source: Source, path: String) -> Result<(), String> {
    share::share_custom_source(source, path).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn import(
    path: String,
    source_id: Option<i64>,
    name_override: Option<String>,
) -> Result<(), String> {
    share::import(path, source_id, name_override).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn channel_exists(name: String, url: String, source_id: i64) -> Result<bool, String> {
    sql::channel_exists(&name, &url, source_id).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn update_source(source: Source) -> Result<(), String> {
    sql::update_source(source).map_err(map_err_frontend)
}

#[tauri::command]
async fn get_epg(channel: Channel) -> Result<Vec<EPG>, String> {
    xtream::get_epg(channel).await.map_err(map_err_frontend)
}

#[tauri::command]
async fn download(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    name: Option<String>,
    url: String,
    download_id: String,
    path: Option<String>,
) -> Result<(), String> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    {
        let mut state = state.lock().await;
        state.download_stop.insert(download_id.clone(), stop);
    }
    let result = utils::download(stop_clone, app, name, url, &download_id, path)
        .await
        .map_err(map_err_frontend);
    {
        let mut state = state.lock().await;
        state.download_stop.remove(&download_id);
    }
    result
}

#[tauri::command]
async fn abort_download(
    state: State<'_, Mutex<AppState>>,
    download_id: String,
) -> Result<(), String> {
    let mutex = state.lock().await;
    let download = mutex
        .download_stop
        .get(&download_id)
        .context("download not found")
        .map_err(map_err_frontend)?;
    download.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
async fn add_epg(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    epg: EPGNotify,
) -> Result<(), String> {
    epg::add_epg(state, app, epg)
        .await
        .map_err(map_err_frontend)
}

#[tauri::command(async)]
async fn remove_epg(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    epg_id: String,
) -> Result<(), String> {
    epg::remove_epg(state, app, epg_id)
        .await
        .map_err(map_err_frontend)
}

#[tauri::command(async)]
fn get_epg_ids() -> Result<Vec<String>, String> {
    sql::get_epg_ids().map_err(map_err_frontend)
}

#[tauri::command]
async fn on_start_check_epg(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
) -> Result<(), String> {
    epg::on_start_check_epg(state, app)
        .await
        .map_err(map_err_frontend)
}

#[tauri::command]
async fn start_restream(
    port: u16,
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    channel: Channel,
) -> Result<(), String> {
    crate::restream::start_restream(port, state, app, channel)
        .await
        .map_err(map_err_frontend)
}

#[tauri::command]
async fn stop_restream(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    crate::restream::stop_restream(state)
        .await
        .map_err(map_err_frontend)
}

#[tauri::command]
async fn watch_self(port: u16) -> Result<(), String> {
    restream::watch_self(port).await.map_err(map_err_frontend)
}

#[tauri::command]
async fn get_network_info() -> Result<NetworkInfo, String> {
    restream::get_network_info().await.map_err(map_err_frontend)
}

#[tauri::command(async)]
fn share_restream(address: String, channel: Channel, path: String) -> Result<(), String> {
    restream::share_restream(address, channel, path).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn add_last_watched(id: i64) -> Result<(), String> {
    sql::add_last_watched(id).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn backup_favs(id: i64, path: String) -> Result<(), String> {
    utils::backup_favs(id, path).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn restore_favs(id: i64, path: String) -> Result<(), String> {
    utils::restore_favs(id, path).map_err(map_err_frontend)
}

#[tauri::command(async)]
fn clear_history() -> Result<(), String> {
    sql::clear_history().map_err(map_err_frontend)
}

#[tauri::command(async)]
fn is_container() -> bool {
    utils::is_container()
}
