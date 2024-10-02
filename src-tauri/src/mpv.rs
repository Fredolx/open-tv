use anyhow::{Context, Result};
use chrono::Local;
use directories::UserDirs;
use which::which;
use std::{env::{consts::OS, current_exe}, path::Path, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::timeout,
};
use std::sync::LazyLock;
use crate::{media_type, settings::get_settings, types::Channel};

const MPV_END_STR: [&str; 3] = ["AO", "VO", "AV"];
const ARG_SAVE_POSITION_ON_QUIT: &str = "--save-position-on-quit";
const ARG_CACHE: &str = "--cache";
const ARG_RECORD: &str = "--stream-record=";
static MPV_PATH: LazyLock<String> = LazyLock::new(|| get_mpv_path());

pub async fn play(channel: Channel, record: bool) -> Result<()> {
    println!("{} playing", channel.url.as_ref().unwrap());
    let args = get_play_args(channel, record)?;
    let mut cmd = Command::new(MPV_PATH.clone())
        .args(args) // Add any arguments your command needs
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = cmd.stdout.take().context("No stdout")?;
    let mut reader = BufReader::new(stdout).lines();
    let read_timeout = Duration::from_secs(25);
    // Read stdout line by line with a timeout
    while let Ok(Some(line)) = timeout(read_timeout, reader.next_line()).await? {
        if MPV_END_STR.iter().any(|x| x.contains(&line)) {
            break;
        }
    }
    Ok(())
}

fn get_mpv_path() -> String {
    if OS != "windows" || which("mpv").is_ok() {
        return "mpv".to_string();
    }
    let mut path = current_exe().unwrap();
    path.pop();
    path.push("deps");
    path.push("mpv.exe");
    return path.to_string_lossy().to_string();
}

//@TODO: Ask the user to set a custom recording path if default can't be found
fn get_play_args(channel: Channel, record: bool) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let settings = get_settings()?;
    args.push(channel.url.context("no url")?);
    if channel.media_type != media_type::LIVESTREAM {
        args.push(ARG_SAVE_POSITION_ON_QUIT.to_string());
    }
    if settings.use_stream_caching == Some(false) {
        let stream_caching_arg = format!("{ARG_CACHE} no",);
        args.push(stream_caching_arg);
    }
    if record {
        let record_path = match settings.recording_path {
            Some(path) => get_path(path),
            None => get_default_record_path()?,
        };
        args.push(format!("{ARG_RECORD}{record_path}"));
    }
    Ok(args)
}

fn get_path(path_str: String) -> String {
    let path = Path::new(&path_str);
    let path = path.join(get_file_name());
    return path.to_string_lossy().to_string(); // Check if it causes problems for some OS languages?
}

fn get_file_name() -> String {
    let current_time = Local::now();
    let formatted_time = current_time.format("%Y-%m-%d-%H-%M-%S").to_string();
    format!("{formatted_time}.mp4")
}

fn get_default_record_path() -> Result<String> {
    let user_dirs = UserDirs::new().context("Failed to get user dirs")?;
    let mut path = user_dirs.video_dir().context("No videos dir")?.to_owned();
    path.push("open-tv");
    std::fs::create_dir_all(&path)?;
    Ok(path.to_string_lossy().to_string())
}
