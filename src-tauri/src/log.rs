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

use chrono::Local;
use directories::ProjectDirs;
use std::{fs, sync::LazyLock};

static USE_LOGGER: LazyLock<bool> = LazyLock::new(|| init_logger());

pub fn log(message: String) {
    if *USE_LOGGER {
        log::error!("{message}");
    } else {
        eprintln!("{message}");
    }
}

fn init_logger() -> bool {
    let file = match fs::File::create(get_and_create_log_path()) {
        Ok(val) => val,
        Err(e) => {
            eprint!("Failed to create file for logger, {:?}", e);
            return false;
        }
    };
    match simplelog::WriteLogger::init(
        simplelog::LevelFilter::Error,
        simplelog::Config::default(),
        file,
    ) {
        Ok(_) => true,
        Err(e) => {
            eprint!("Failed to init logger, {:?}", e);
            return false;
        }
    }
}

fn get_and_create_log_path() -> String {
    let mut path = ProjectDirs::from("dev", "fredol", "open-tv")
        .unwrap()
        .cache_dir()
        .to_owned();
    path.push("logs");
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path.push(get_log_name());
    return path.to_string_lossy().to_string();
}

fn get_log_name() -> String {
    let current_time = Local::now();
    let formatted_time = current_time.format("%Y-%m-%d-%H-%M-%S").to_string();
    format!("{formatted_time}.log")
}
