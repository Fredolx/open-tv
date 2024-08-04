use std::{
    borrow::{Borrow, BorrowMut},
    fs::File,
    io::{BufRead, BufReader, Write},
    sync::LazyLock,
};

use anyhow::{bail, Context, Result};
use regex::{Captures, Regex};
use types::{Channel, MediaType};

pub mod sql;
pub mod types;

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

pub fn read_m3u8(path: String, source_name: String) -> Result<Vec<usize>> {
    let file = File::open(path).context("Failed to open m3u8 file")?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().enumerate().skip(1);
    let mut problematic_lines: Vec<usize> = Vec::new();
    let source_id = sql::create_or_find_source_by_name(source_name, types::SourceType::M3U)?;
    let mut sql = sql::CONN.lock().unwrap();
    let tx = sql.transaction()?;
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
        let channel = match get_channel_from_lines(l1, l2, source_id)
            .with_context(|| format!("Failed to process lines #{} #{}, skipping", c1, c2))
        {
            Ok(val) => val,
            Err(e) => {
                problematic_lines.push(c1);
                eprintln!("{}", e);
                continue;
            }
        };
        sql::insert_channel(&tx, channel)?;
    }
    tx.commit()?;
    Ok(problematic_lines)
}

async fn get_m3u8_from_link(url: String, source_name: String) -> Result<()> {
    let client = reqwest::Client::new();
    let mut response = client.get(url).send().await?;
    let mut str_buffer: String = String::new();
    let mut skipped_first = false;

    let source_id = sql::create_or_find_source_by_name(source_name, types::SourceType::M3U)?;
    let mut sql = sql::CONN.lock().unwrap();
    let tx = sql.transaction()?;

    while let Some(chunk) = response.chunk().await? {
        let mut two_lines = Vec::new();
        let lossy = String::from_utf8_lossy(&chunk);
        let lossy = lossy.into_owned();
        str_buffer.push_str(&lossy);
        let mut split: Vec<String> = str_buffer.split('\n').map(String::from).collect();
        str_buffer.clear();
        if !skipped_first {
            skipped_first = true;
            split.remove(0);
        }
        let last = split.last().context("failed to get last")?;
        if !last.ends_with('\n') {
            str_buffer = last.to_string();
            split.pop();
        }
        let len = split.len();
        if len > 0 && len % 2 != 0 {
            let mut ele = split.pop().context("failed to pop")?;
            ele.push('\n');
            str_buffer.insert_str(0, &ele);
        }
        for line in split {
            two_lines.push(line);
            if two_lines.len() == 2 {
                let first = two_lines.remove(0);
                let second = two_lines.remove(0);
                let channel = match get_channel_from_lines(
                    first.to_string(),
                    second.to_string(),
                    source_id,
                )
                .with_context(|| format!("Failed to process lines:\n{}\n{}", first, second))
                {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        continue;
                    }
                };
                sql::insert_channel(&tx, channel)?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

fn extract_non_empty_capture(caps: Captures) -> Option<String> {
    caps.get(1)
        .map(|m| m.as_str().to_string())
        .filter(|s| !s.trim().is_empty())
}

fn get_channel_from_lines(first: String, second: String, source_id: i64) -> Result<Channel> {
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
    let group = GROUP_REGEX
        .captures(&first)
        .and_then(extract_non_empty_capture);
    let image = LOGO_REGEX
        .captures(&first)
        .and_then(extract_non_empty_capture);
    let channel = Channel {
        name: name,
        group: group,
        image: image,
        url: second.clone(),
        media_type: get_media_type(second),
        source_id: source_id,
    };
    //let group
    Ok(channel)
}

fn get_media_type(url: String) -> MediaType {
    let media_type = if url.ends_with(".mp4") || url.ends_with("mkv") {
        MediaType::Movie
    } else {
        MediaType::Livestream
    };
    return media_type;
}

#[cfg(test)]
mod test_m3u {
    use crate::{get_channel_from_lines, get_m3u8_from_link, read_m3u8};
    use std::{env, time::Instant};

    #[test]
    fn test_get_channel_from_lines() {
        get_channel_from_lines(r#"#EXTINF:-1 tvg-id="Amazing Channel" tvg-name="Amazing Channel" tvg-logo="http://myurl.local/logos/amazing/amazing-1.png" group-title="The Best Channels"#.to_string()
       , r#"http://myurl.local/1234/1234/1234"#.to_string(), 0).unwrap();
        get_channel_from_lines(r#"#EXTINF:-1 tvg-id="Amazing Channel" tvg-name="" tvg-logo="http://myurl.local/logos/amazing/amazing-1.png" group-title="The Best Channels"#.to_string()
       , r#"http://myurl.local/1234/1234/1234"#.to_string(), 0).unwrap();
        assert!(get_channel_from_lines(r#"#EXTINF:-1 tvg-id="" tvg-name="" tvg-logo="http://myurl.local/logos/amazing/amazing-1.png" group-title="The Best Channels"#.to_string()
       , r#"http://myurl.local/1234/1234/1234"#.to_string(), 0).is_err());
        assert!(get_channel_from_lines(r#"#EXTINF:-1 tvg-id=" " tvg-name="" tvg-logo="http://myurl.local/logos/amazing/amazing-1.png" group-title="The Best Channels"#.to_string()
       , r#"http://myurl.local/1234/1234/1234"#.to_string(), 0).is_err());
    }

    #[test]
    fn test_read_m3u8() {
        crate::sql::drop_db().unwrap_or_default();
        crate::sql::create_or_initialize_db().unwrap();
        let now = Instant::now();
        read_m3u8(
            "/home/fred/Downloads/get.php".to_string(),
            "main".to_string(),
        )
        .unwrap();
        std::fs::write("bench.txt", now.elapsed().as_millis().to_string()).unwrap();
    }

    #[tokio::test]
    async fn test_get_m3u8_from_link() {
        crate::sql::drop_db().unwrap_or_default();
        crate::sql::create_or_initialize_db().unwrap();
        let now = Instant::now();
        let url = env::var("OPEN_TV_TEST_LINK").unwrap();
        get_m3u8_from_link(url, 
        "source2".to_string()).await.unwrap();
        std::fs::write("bench2.txt", now.elapsed().as_millis().to_string()).unwrap();
    }
}
