// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::{Context, Result};
use open_tv_lib::log;
#[cfg(target_os = "linux")]
use std::env;
#[cfg(target_os = "linux")]
use std::path::Path;

fn main() -> Result<()> {
    apply_gpu_fixes();
    _ = open_tv_lib::utils::check_nuke()
        .with_context(|| "Failed to delete db after nuke request")
        .inspect_err(|e| log::log(format!("{:?}", e)));
    open_tv_lib::sql::create_or_initialize_db()?;
    open_tv_lib::run();
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn apply_gpu_fixes() {
    if Path::new("/proc/driver/nvidia").exists() {
        eprintln!("NVIDIA GPU detected. Setting WEBKIT_DISABLE_DMABUF_RENDERER=1");
        unsafe {
            env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn apply_gpu_fixes() {}
