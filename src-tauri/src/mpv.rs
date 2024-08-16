use anyhow::{Context, Result};
use directories::UserDirs;
use std::{process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::timeout,
};
use types::Channel;

use crate::{
    settings::get_settings,
    types::{self, MediaType},
};

const MPV_END_STR: [&str; 3] = ["AO", "VO", "AV"];
const ARG_SAVE_POSITION_ON_QUIT: &str = "--save-position-on-quit";
const ARG_CACHE: &str = "--cache";
const ARG_RECORD: &str = "--stream-record=";

async fn play(channel: Channel, record: bool) -> Result<()> {
    let args = get_play_args(channel, record)?;

    let mut cmd = Command::new("mpv")
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

//@TODO: Ask the user to set a custom recording path if default can't be found
fn get_play_args(channel: Channel, record: bool) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let settings = get_settings()?;
    if channel.media_type == MediaType::Livestream {
        args.push(ARG_SAVE_POSITION_ON_QUIT.to_string());
    }
    if settings.use_stream_caching == Some(false) {
        let stream_caching_arg = format!("{ARG_CACHE} no",);
        args.push(stream_caching_arg);
    }
    if record {
        let record_path = match settings.recording_path {
            Some(path) => path,
            None => get_default_record_path()?,
        };
        args.push(format!("{ARG_RECORD}{record_path}"));
    }
    Ok(args)
}

fn get_default_record_path() -> Result<String> {
    let user_dirs = UserDirs::new().context("Failed to get user dirs")?;
    let mut path = user_dirs.video_dir().context("No videos dir")?.to_owned();
    path.push("open-tv");
    std::fs::create_dir_all(&path)?;
    Ok(path.to_string_lossy().to_string())
}
