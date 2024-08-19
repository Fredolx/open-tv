use anyhow::Error;
use types::{Channel, Filters, Settings, Source};

pub mod m3u;
pub mod mpv;
pub mod settings;
pub mod sql;
pub mod types;
pub mod xtream;
pub mod utils;

fn print_error_stack(e: Error) {
    eprintln!("{:?}", e);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_m3u8,
            get_m3u8_from_link,
            play,
            get_settings,
            update_settings,
            search,
            get_xtream,
            refresh_source,
            get_episodes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn map_err_frontend(e: Error) -> String {
    return e.to_string();
}

#[tauri::command(async)]
fn get_m3u8(source: Source) -> Result<(), String> {
    m3u::read_m3u8(source).map_err(map_err_frontend)
}

#[tauri::command]
async fn get_m3u8_from_link(source: Source) -> Result<(), String> {
    m3u::get_m3u8_from_link(source).await.map_err(map_err_frontend)
}

#[tauri::command]
async fn play(channel: Channel, record: bool) -> Result<(), String> {
    mpv::play(channel, record).await.map_err(map_err_frontend)
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
    xtream::get_xtream(source).await.map_err(map_err_frontend)
}

#[tauri::command]
async fn refresh_source(source: Source) -> Result<(), String> {
    utils::refresh_source(source).await.map_err(map_err_frontend)
}

#[tauri::command]
async fn get_episodes(source: Source, series_id: u64) -> Result<Vec<Channel>, String> {
    xtream::get_episodes(source, series_id).await.map_err(map_err_frontend)
}
