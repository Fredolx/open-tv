/*
 * Beats TV - Premium IPTV Player
 * Copyright (C) 2026 Beats TV Team
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * This project is a fork of Open TV by Fredolx.
 */

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
        // Safe alternative to unsafe block: std::env::set_var is safe to use
        // It sets the environment variable for the current process and all future child processes
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}

#[cfg(not(target_os = "linux"))]
pub fn apply_gpu_fixes() {}
