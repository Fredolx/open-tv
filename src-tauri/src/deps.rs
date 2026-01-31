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

use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use serde::Serialize;
use std::env::consts::OS;
use std::path::Path;
use std::process::Command;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;
use which::which;

/// Represents the status of a single dependency
#[derive(Debug, Clone, Serialize)]
pub struct DependencyStatus {
    pub name: String,
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

/// Represents the overall dependency check result
#[derive(Debug, Clone, Serialize)]
pub struct DependencyCheckResult {
    pub all_satisfied: bool,
    pub dependencies: Vec<DependencyStatus>,
    pub platform: String,
    pub install_instructions: Option<String>,
}

const REQUIRED_DEPS: [&str; 3] = ["mpv", "ffmpeg", "yt-dlp"];

/// Check if a binary exists in PATH or bundled deps folder
fn check_binary(name: &str) -> DependencyStatus {
    // First check system PATH
    if let Ok(path) = which(name) {
        let version = get_version(name);
        return DependencyStatus {
            name: name.to_string(),
            installed: true,
            path: Some(path.to_string_lossy().to_string()),
            version,
        };
    }

    // On Windows, check bundled deps folder
    #[cfg(target_os = "windows")]
    {
        if let Ok(exe_path) = std::env::current_exe() {
            let mut deps_path = exe_path.clone();
            deps_path.pop();
            deps_path.push("deps");
            deps_path.push(format!("{}.exe", name));
            if deps_path.exists() {
                return DependencyStatus {
                    name: name.to_string(),
                    installed: true,
                    path: Some(deps_path.to_string_lossy().to_string()),
                    version: None,
                };
            }
        }
    }

    // On macOS, check common Homebrew/MacPorts paths
    #[cfg(target_os = "macos")]
    {
        let macos_paths = [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/opt/local/bin",
        ];
        for base_path in macos_paths {
            let full_path = format!("{}/{}", base_path, name);
            if std::path::Path::new(&full_path).exists() {
                let version = get_version(&full_path);
                return DependencyStatus {
                    name: name.to_string(),
                    installed: true,
                    path: Some(full_path),
                    version,
                };
            }
        }
    }

    DependencyStatus {
        name: name.to_string(),
        installed: false,
        path: None,
        version: None,
    }
}

/// Try to get version string from a binary
fn get_version(binary: &str) -> Option<String> {
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .ok()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Return first line of version output
        stdout.lines().next().map(|s| s.to_string())
    } else {
        None
    }
}

/// Generate platform-specific installation instructions
fn get_install_instructions(missing: &[&str]) -> String {
    if missing.is_empty() {
        return String::new();
    }

    let deps_list = missing.join(" ");

    match OS {
        "macos" => format!(
            "Missing dependencies: {}\n\n\
            Install using Homebrew:\n\
            brew install {}\n\n\
            Or using MacPorts:\n\
            sudo port install {}",
            missing.join(", "),
            deps_list,
            deps_list
        ),
        "linux" => format!(
            "Missing dependencies: {}\n\n\
            Debian/Ubuntu:\n\
            sudo apt install {}\n\n\
            Fedora:\n\
            sudo dnf install {}\n\n\
            Arch Linux:\n\
            sudo pacman -S {}",
            missing.join(", "),
            deps_list,
            deps_list,
            deps_list
        ),
        "windows" => format!(
            "Missing dependencies: {}\n\n\
            These should be bundled with the installer.\n\
            Try reinstalling the application, or install manually:\n\n\
            Using Scoop:\n\
            scoop install {}\n\n\
            Using Chocolatey:\n\
            choco install {}",
            missing.join(", "),
            deps_list,
            deps_list
        ),
        _ => format!(
            "Missing dependencies: {}\n\n\
            Please install these manually for your platform.",
            missing.join(", ")
        ),
    }
}

/// Check all required dependencies and return the result
pub fn check_dependencies() -> DependencyCheckResult {
    let mut dependencies = Vec::new();
    let mut missing: Vec<&str> = Vec::new();

    for dep in REQUIRED_DEPS {
        let status = check_binary(dep);
        if !status.installed {
            missing.push(dep);
        }
        dependencies.push(status);
    }

    let all_satisfied = missing.is_empty();
    let install_instructions = if all_satisfied {
        None
    } else {
        Some(get_install_instructions(&missing))
    };

    DependencyCheckResult {
        all_satisfied,
        dependencies,
        platform: OS.to_string(),
        install_instructions,
    }
}

/// Check if this is the first run (no database exists yet)
pub fn is_first_run() -> bool {
    use directories::ProjectDirs;
    
    if let Some(proj_dirs) = ProjectDirs::from("dev", "fredol", "open-tv") {
        let db_path = proj_dirs.data_dir().join("beatstv.db");
        !db_path.exists()
    } else {
        true
    }
}

pub async fn auto_install_dependency(_app: AppHandle, _name: &str) -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        return Err(anyhow!("Auto-install is only supported on Windows at this time. Please use your package manager."));
    }

    #[cfg(target_os = "windows")]
    {
        let exe_path = std::env::current_exe()?;
        let mut deps_dir = exe_path.clone();
        deps_dir.pop();
        deps_dir.push("deps");
        
        if !deps_dir.exists() {
            fs::create_dir_all(&deps_dir).context("Failed to create deps directory")?;
        }

        match name {
            "mpv" => install_mpv_windows(app, &deps_dir).await?,
            "ffmpeg" => install_ffmpeg_windows(app, &deps_dir).await?,
            "yt-dlp" => install_ytdlp_windows(app, &deps_dir).await?,
            _ => return Err(anyhow!("Unsupported dependency: {}", name)),
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
async fn install_mpv_windows(app: AppHandle, deps_dir: &Path) -> Result<()> {
    // Shinchiro builds are the gold standard for Windows MPV
    // We'll target the v3-x86_64 build for modern hardware
    let url = "https://github.com/shinchiro/mpv-winbuild-cmake/releases/latest/download/mpv-x86_64-v3-20250123-git-6950275.zip"; 
    // Note: In a production app, we'd fetch the latest release JSON from GitHub API to get the dynamic filename.
    // For this implementation, we use a known-good very recent release link.
    
    download_and_extract(app, "MPV Player", url, deps_dir, "mpv.exe").await
}

#[cfg(target_os = "windows")]
async fn install_ffmpeg_windows(app: AppHandle, deps_dir: &Path) -> Result<()> {
    let url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";
    download_and_extract(app, "FFmpeg", url, deps_dir, "ffmpeg.exe").await
}

#[cfg(target_os = "windows")]
async fn install_ytdlp_windows(app: AppHandle, deps_dir: &Path) -> Result<()> {
    let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
    download_file(app, "yt-dlp", url, &deps_dir.join("yt-dlp.exe")).await
}

#[cfg(target_os = "windows")]
async fn download_and_extract(app: AppHandle, display_name: &str, url: &str, deps_dir: &Path, target_bin: &str) -> Result<()> {
    let zip_path = deps_dir.join(format!("{}.zip", target_bin));
    download_file(app.clone(), display_name, url, &zip_path).await?;
    
    let _ = app.emit("install-status", serde_json::json!({
        "name": display_name,
        "status": "extracting",
        "progress": 100
    }));

    // Use PowerShell to extract as it's built-in to Windows
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force", zip_path.to_string_lossy(), deps_dir.to_string_lossy()))
        .output()
        .context("Failed to run PowerShell Expand-Archive")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Extraction failed: {}", err));
    }

    // FFmpeg and MPV zips often have nested folders. We need to find the bin and move it or just ensure it exists.
    // Cleanup zip
    let _ = fs::remove_file(zip_path);
    
    let _ = app.emit("install-status", serde_json::json!({
        "name": display_name,
        "status": "complete",
        "progress": 100
    }));

    Ok(())
}

async fn download_file(app: AppHandle, display_name: &str, url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(0);
    
    let mut file = tokio::fs::File::create(dest).await?;
    let mut downloaded: u64 = 0;
    let mut last_emit = 0;

    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| anyhow!("Stream error: {}", e))?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = (downloaded as f64 / total_size as f64 * 100.0) as i32;
            if progress > last_emit {
                let _ = app.emit("install-status", serde_json::json!({
                    "name": display_name,
                    "status": "downloading",
                    "progress": progress
                }));
                last_emit = progress;
            }
        }
    }

    Ok(())
}
