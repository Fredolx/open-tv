use anyhow::Error;

pub mod sql;
pub mod types;
pub mod xtream;
pub mod settings;
pub mod m3u;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn print_error_stack(e: Error) {
    eprintln!("{:?}", e);
}