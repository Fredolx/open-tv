use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::LazyLock,
};

use anyhow::{bail, Context, Result};
use regex::{Captures, Regex};
use rusqlite::Connection;

struct Channel {
    name: String,
    url: String,
    group: Option<String>,
    image: Option<String>,
    media_type: MediaType,
}

enum MediaType {
    Livestream,
    Movie,
    Serie,
    Group,
}

pub mod sql;

static NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"tvg-name="(?P<name>[^"]*)""#).unwrap());
static ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"tvg-id="(?P<id>[^"]*)""#).unwrap());
static LOGO_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"tvg-logo="(?P<logo>[^"]*)""#).unwrap());
static GROUP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"group-title="(?P<group>[^"]*)""#).unwrap());

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn read_m3u8(path: String, source_name: String) -> Result<()> {
    let file = File::open("foo.txt").context("Failed to open m3u8 file")?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().enumerate().skip(1);
    let mut problematic_lines: Vec<usize> = Vec::new();
    while let (Some((c1, l1)), Some((c2, l2))) = (lines.next(), lines.next()) {
        let l1 = match l1.with_context(|| format!("(l1) Error on line: {}, skipping", c1)) {
            Ok(line) => line,
            Err(e) => {
                problematic_lines.push(c1);
                eprintln!("{}", e);
                continue;
            }
        };
        let l2 = match l2.with_context(|| format!("(l2) Error on line: {}, skipping", c2)) {
            Ok(line) => line,
            Err(e) => {
                problematic_lines.push(c2);
                eprintln!("{}", e);
                continue;
            }
        };
        let channel = match get_channel_from_lines(l1, l2)
            .with_context(|| format!("Failed to process lines #{} #{}, skipping", c1, c2))
        {
            Ok(val) => val,
            Err(e) => {
                problematic_lines.push(c1);
                eprintln!("{}", e);
                continue;
            }
        };
        //commit_channel();
    }
    Ok(())
}

fn commit_channel(channel: Channel) -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute(
    "
        
    ", ())?;
    Ok(())
}

fn extract_non_empty_capture(caps: Captures) -> Option<String> {
    caps.get(1)
        .map(|m| m.as_str().to_string())
        .filter(|s| !s.trim().is_empty())
}

fn get_channel_from_lines(first: String, second: String) -> Result<Channel> {
    if second.trim().is_empty() {
        bail!("second line is empty");
    }
    let name = NAME_REGEX
        .captures(&first)
        .and_then(extract_non_empty_capture)
        .or_else(|| {
            ID_REGEX
                .captures(&first)
                .and_then(extract_non_empty_capture)
        })
        .context("Couldn't find name from Name or ID")?;
    let group = GROUP_REGEX.captures(&first)
    .and_then(extract_non_empty_capture);
    let image = LOGO_REGEX.captures(&first)
        .and_then(extract_non_empty_capture);
    let channel = Channel {
        name: name,
        group: group,
        image: image,
        url: second.clone(),
        media_type: get_media_type(second)
    };
    //let group
    Ok(channel)
}

fn get_media_type(url: String) -> MediaType {
    let media_type = if url.ends_with(".mp4") || url.ends_with("mkv")
        { MediaType::Movie } 
        else { MediaType::Livestream };
    return media_type;
}

#[cfg(test)]
mod test_m3u {
    use crate::get_channel_from_lines;

    #[test]
    fn test_get_channel_from_lines() {
        let res1 = get_channel_from_lines(r#"#EXTINF:-1 tvg-id="Amazing Channel" tvg-name="Amazing Channel" tvg-logo="http://myurl.local/logos/amazing/amazing-1.png" group-title="The Best Channels"#.to_string()
       , r#"http://myurl.local/1234/1234/1234"#.to_string());
       assert!(!res1.is_err());
       let res2 = get_channel_from_lines(r#"#EXTINF:-1 tvg-id="Amazing Channel" tvg-name="" tvg-logo="http://myurl.local/logos/amazing/amazing-1.png" group-title="The Best Channels"#.to_string()
       , r#"http://myurl.local/1234/1234/1234"#.to_string());
       assert!(!res2.is_err());
       let res3 = get_channel_from_lines(r#"#EXTINF:-1 tvg-id="" tvg-name="" tvg-logo="http://myurl.local/logos/amazing/amazing-1.png" group-title="The Best Channels"#.to_string()
       , r#"http://myurl.local/1234/1234/1234"#.to_string());
       assert!(res3.is_err());
       let res4 = get_channel_from_lines(r#"#EXTINF:-1 tvg-id=" " tvg-name="" tvg-logo="http://myurl.local/logos/amazing/amazing-1.png" group-title="The Best Channels"#.to_string()
       , r#"http://myurl.local/1234/1234/1234"#.to_string());
       assert!(res4.is_err());
    }
}
