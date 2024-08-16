// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::Result;
fn main() -> Result<()> {
    open_tv_lib::sql::create_or_initialize_db()?;
    open_tv_lib::run();
    Ok(())
}
