use anyhow::Error;

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
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
