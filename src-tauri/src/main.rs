// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::{Context, Result};
use open_tv_lib::log;
fn main() -> Result<()> {
    _ = open_tv_lib::utils::check_nuke()
        .with_context(|| "Failed to delete db after nuke request")
        .inspect_err(|e| log::log(format!("{:?}", e)));
    open_tv_lib::sql::create_or_initialize_db()?;
    open_tv_lib::run();
    Ok(())
}
