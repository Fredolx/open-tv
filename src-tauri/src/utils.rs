use crate::{
    log::log,
    m3u,
    settings::{get_default_record_path, get_settings},
    source_type, sql,
    types::Source,
    xtream,
};
use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Local, Utc};
use directories::ProjectDirs;
use regex::Regex;
use reqwest::Client;
use serde::Serialize;
use std::{
    env::{consts::OS, current_exe},
    fs::File,
    path::{Path, PathBuf},
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering::Relaxed},
    },
};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;
use which::which;

const MACOS_POTENTIAL_PATHS: [&str; 3] = [
    "/opt/local/bin",    // MacPorts
    "/opt/homebrew/bin", // Homebrew on AARCH64 Mac
    "/usr/local/bin",    // Homebrew on AMD64 Mac
];

const DEFAULT_USER_AGENT: &str = "Fred TV";

static ILLEGAL_CHARS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[<>:"/\\|?*\x00-\x1F]"#).unwrap());

pub async fn refresh_source(source: Source) -> Result<()> {
    match source.source_type {
        source_type::M3U => m3u::read_m3u8(source, true)?,
        source_type::M3U_LINK => m3u::get_m3u8_from_link(source, true).await?,
        source_type::XTREAM => xtream::get_xtream(source, true).await?,
        source_type::CUSTOM => {}
        _ => return Err(anyhow!("invalid source_type")),
    }
    Ok(())
}

pub async fn refresh_all() -> Result<()> {
    let sources = sql::get_sources()?;
    for source in sources {
        refresh_source(source).await?;
    }
    Ok(())
}

pub fn get_local_time(timestamp: i64) -> Result<DateTime<Local>> {
    let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0).context("no time")?;
    Ok(DateTime::<Local>::from(datetime))
}

pub async fn download(
    stop: Arc<AtomicBool>,
    app: AppHandle,
    name: Option<String>,
    url: String,
    download_id: &str,
    path: Option<String>,
) -> Result<()> {
    let client = Client::new();
    let mut response = client.get(&url).send().await?;
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0;
    let path = match path {
        Some(p) => p,
        None => get_download_path(get_filename(
            name.context("no name provided to download")?,
            url,
        )?)?,
    };
    let mut file = tokio::fs::File::create(&path).await?;
    let mut send_threshold: f64 = 0.1;
    if !response.status().is_success() {
        let error = response.status();
        bail!("Failed to download movie: HTTP {error}")
    }
    while let Some(chunk) = response.chunk().await? {
        if stop.load(Relaxed) {
            drop(file);
            tokio::fs::remove_file(path).await?;
            bail!("download aborted");
        }
        file.write(&chunk).await?;
        downloaded += chunk.len() as u64;
        if total_size > 0 {
            let progress: f64 = (downloaded as f64 / total_size as f64) * 100.0;
            let progress = (progress * 10.0).trunc() / 10.0;
            if progress > send_threshold {
                app.emit(&format!("progress-{}", download_id), progress)?;
                send_threshold = progress + 0.1 as f64;
            }
        }
    }
    Ok(())
}

fn get_filename(channel_name: String, url: String) -> Result<String> {
    let extension = get_extension(url);
    let channel_name = sanitize(channel_name);
    let filename = format!("{channel_name}.{extension}").to_string();
    Ok(filename)
}

fn get_extension(url: String) -> String {
    url.rsplit(".")
        .next()
        .filter(|ext| !ext.starts_with("php?"))
        .unwrap_or("mp4")
        .to_string()
}

pub fn sanitize(str: String) -> String {
    ILLEGAL_CHARS_REGEX.replace_all(&str, "").to_string()
}

fn get_download_path(file_name: String) -> Result<String> {
    let settings = get_settings()?;
    let path = match settings.recording_path {
        Some(path) => path,
        None => get_default_record_path()?,
    };
    let mut path = Path::new(&path).to_path_buf();
    path.push(file_name);
    Ok(path.to_string_lossy().to_string())
}
pub fn get_bin(bin: &str) -> String {
    if OS == "linux" || which(bin).is_ok() {
        return bin.to_string();
    } else if OS == "macos" {
        return find_macos_bin(bin);
    }
    return get_bin_from_deps(bin);
}

fn get_bin_from_deps(bin: &str) -> String {
    let mut path = current_exe().unwrap();
    path.pop();
    path.push("deps");
    path.push(bin);
    return path.to_string_lossy().to_string();
}

pub fn find_macos_bin(bin: &str) -> String {
    return MACOS_POTENTIAL_PATHS
        .iter()
        .map(|path| {
            let mut path = Path::new(path).to_path_buf();
            path.push(bin);
            return path;
        })
        .find(|path| path.exists())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            log(format!("Could not find {} on MacOS host", bin));
            return bin.to_string();
        });
}

pub fn serialize_to_file<T: Serialize>(obj: T, path: String) -> Result<()> {
    let data = serde_json::to_string(&obj)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn backup_favs(source_id: i64, path: String) -> Result<()> {
    sql::do_tx(|tx| {
        let preserve = sql::get_channel_preserve(tx, source_id)?;
        serialize_to_file(preserve, path)?;
        Ok(())
    })?;
    Ok(())
}

pub fn restore_favs(source_id: i64, path: String) -> Result<()> {
    let data = std::fs::read_to_string(path)?;
    let preserve = serde_json::from_str(&data)?;
    sql::do_tx(|tx| {
        sql::restore_preserve(tx, source_id, preserve)?;
        Ok(())
    })?;
    Ok(())
}

pub fn is_container() -> bool {
    std::env::var("container").is_ok()
}

pub fn create_nuke_request() -> Result<()> {
    let path = get_nuke_path()?;
    File::create(path)?;
    std::process::exit(0);
}

fn get_nuke_path() -> Result<PathBuf> {
    let path = ProjectDirs::from("dev", "fredol", "open-tv").context("project dir not found")?;
    let path = path.cache_dir();
    let path = path.join("nuke.txt");
    Ok(path)
}

pub fn check_nuke() -> Result<()> {
    let path = get_nuke_path()?;
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_file(path)?;
    let path = ProjectDirs::from("dev", "fredol", "open-tv").context("project dir not found")?;
    let path = path.data_dir();
    let path = path.join(sql::DB_NAME);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn get_user_agent_from_source(source: &Source) -> Result<String> {
    let user_agent: &str = source
        .user_agent
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(DEFAULT_USER_AGENT);
    Ok(user_agent.to_string())
}

#[cfg(test)]
mod test_utils {
    use super::sanitize;

    #[test]
    fn test_sanitize() {
        assert_eq!(
            "SuperShow Who will win the million".to_string(),
            sanitize("SuperShow: Who will win the million?".to_string())
        );
    }
}
