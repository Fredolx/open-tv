use anyhow::Error;
use types::Source;

pub mod m3u;
pub mod mpv;
pub mod settings;
pub mod sql;
pub mod types;
pub mod xtream;

fn print_error_stack(e: Error) {
    eprintln!("{:?}", e);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_m3u8,
            get_m3u8_from_link
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn map_err_frontend(e: Error) -> String {
    return e.to_string();
}

#[tauri::command(async)]
fn get_m3u8(path: String, source: Source) -> Result<(), String> {
    m3u::read_m3u8(path, source).map_err(map_err_frontend)?;
    Ok(())
}

#[tauri::command]
async fn get_m3u8_from_link(source: Source) -> Result<(), String> {
    m3u::get_m3u8_from_link(source).await.map_err(map_err_frontend)?;
    Ok(())
}
