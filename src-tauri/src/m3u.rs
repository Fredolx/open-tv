use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    sync::LazyLock,
};

use anyhow::{anyhow, bail, Context, Result};
use bytes::Bytes;
use regex::{Captures, Regex};
use reqwest::Response;
use types::{Channel, Source};

use crate::{
    media_type, print_error_stack,
    sql::{self, delete_source},
    types,
};

static NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"tvg-name="(?P<name>[^"]*)""#).unwrap());
static ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"tvg-id="(?P<id>[^"]*)""#).unwrap());
static LOGO_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"tvg-logo="(?P<logo>[^"]*)""#).unwrap());
static GROUP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"group-title="(?P<group>[^"]*)""#).unwrap());

pub fn read_m3u8(mut source: Source) -> Result<()> {
    let file =
        File::open(source.url.clone().context("No path")?).context("Failed to open m3u8 file")?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().enumerate().skip(1);
    let mut problematic_lines: usize = 0;
    let mut lines_count: usize = 0;
    let new_source = sql::create_or_find_source_by_name(&mut source)?;
    let mut groups: HashMap<String, i64> = HashMap::new();
    let mut sql = sql::get_conn()?;
    let tx = sql.transaction()?;
    while let (Some((c1, l1)), Some((c2, l2))) = (lines.next(), lines.next()) {
        lines_count = c2;
        let l1 = match l1.with_context(|| format!("(l1) Error on line: {c1}, skipping")) {
            Ok(line) => line,
            Err(e) => {
                problematic_lines += 1;
                print_error_stack(e);
                continue;
            }
        };
        let l2 = match l2.with_context(|| format!("(l2) Error on line: {c2}, skipping")) {
            Ok(line) => line,
            Err(e) => {
                problematic_lines += 1;
                print_error_stack(e);
                continue;
            }
        };
        let mut channel = match get_channel_from_lines(l1, l2, source.id.unwrap())
            .with_context(|| format!("Failed to process lines #{c1} #{c2}, skipping"))
        {
            Ok(val) => val,
            Err(e) => {
                problematic_lines += 2;
                print_error_stack(e);
                continue;
            }
        };
        sql::set_channel_group_id(&mut groups, &mut channel, &tx, source.id.as_ref().unwrap())
            .unwrap_or_else(print_error_stack);
        sql::insert_channel(&tx, channel)?;
    }
    if problematic_lines > lines_count / 2 {
        tx.rollback().unwrap_or_else(|e| eprintln!("{:?}", e));
        if new_source {delete_source(source.id.context("no source id")?).unwrap_or_else(print_error_stack);
        }
        return Err(anyhow::anyhow!(
            "Too many problematic lines, read considered failed"
        ));
    }
    tx.commit()?;
    Ok(())
}

pub async fn get_m3u8_from_link(mut source: Source) -> Result<()> {
    let client = reqwest::Client::new();
    let url = source.url.clone().context("Invalid source")?;
    let response = client.get(&url).send().await?;

    let new_source = sql::create_or_find_source_by_name(&mut source)?;
    if let Err(e) = process_chunks(&source, response).await {
        if new_source {
            delete_source(source.id.unwrap()).unwrap_or_else(print_error_stack);
        }
        return Err(e);
    }
    Ok(())
}

async fn process_chunks(source: &Source, mut response: Response) -> Result<()> {
    let mut str_buffer: String = String::new();
    let mut skipped_first = false;
    let mut groups: HashMap<String, i64> = HashMap::new();
    let mut count = 0;
    while let Some(chunk) = response.chunk().await? {
        let split = get_lines_from_chunk(chunk, &mut str_buffer, skipped_first)?;
        if !skipped_first {
            skipped_first = true;
        }
        count += process_chunk_split(split, &source, &mut groups)?;
    }
    if count == 0 {
        return Err(anyhow!("No valid lines found"));
    }
    Ok(())
}

fn process_chunk_split(
    split: Vec<String>,
    source: &Source,
    groups: &mut HashMap<String, i64>,
) -> Result<u64> {
    let mut count = 0;
    let mut two_lines = Vec::new();
    let mut sql = sql::get_conn()?;
    let tx = sql.transaction()?;
    for line in split {
        two_lines.push(line);
        if two_lines.len() == 2 {
            let first = two_lines.remove(0);
            let second = two_lines.remove(0);
            let mut channel = match get_channel_from_lines(
                first.to_string(),
                second.to_string(),
                source.id.unwrap(),
            )
            .with_context(|| format!("Failed to process lines:\n{first}\n{second}"))
            {
                Ok(val) => val,
                Err(e) => {
                    print_error_stack(e);
                    continue;
                }
            };
            sql::set_channel_group_id(groups, &mut channel, &tx, source.id.as_ref().unwrap())
                .unwrap_or_else(print_error_stack);
            sql::insert_channel(&tx, channel)?;
            count += 1;
        }
    }
    tx.commit()?;
    Ok(count)
}

fn get_lines_from_chunk(
    chunk: Bytes,
    str_buffer: &mut String,
    skipped_first: bool,
) -> Result<Vec<String>> {
    let lossy = String::from_utf8_lossy(&chunk);
    let lossy = lossy.into_owned();
    str_buffer.push_str(&lossy);
    let mut split: Vec<String> = str_buffer.split('\n').map(String::from).collect();
    str_buffer.clear();
    if !skipped_first {
        split.remove(0);
    }
    let last = split.last().context("failed to get last")?;
    if !last.ends_with('\n') {
        _ = std::mem::replace(str_buffer, last.to_string());
        split.pop();
    }
    let len = split.len();
    if len > 0 && len % 2 != 0 {
        let mut ele = split.pop().context("failed to pop")?;
        ele.push('\n');
        str_buffer.insert_str(0, &ele);
    }
    Ok(split)
}

fn extract_non_empty_capture(caps: Captures) -> Option<String> {
    caps.get(1)
        .map(|m| m.as_str().to_string())
        .filter(|s| !s.trim().is_empty())
}

fn get_channel_from_lines(first: String, mut second: String, source_id: i64) -> Result<Channel> {
    second = second.trim().to_string();
    if second.is_empty() {
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
        id: None,
        name: name.trim().to_string(),
        group: group.map(|x| x.trim().to_string()),
        image: image.map(|x| x.trim().to_string()),
        url: Some(second.clone()),
        media_type: get_media_type(second),
        source_id: source_id,
        series_id: None,
        group_id: None,
        favorite: false
    };
    Ok(channel)
}

fn get_media_type(url: String) -> u8 {
    let media_type = if url.ends_with(".mp4") || url.ends_with(".mkv") {
        media_type::MOVIE
    } else {
        media_type::LIVESTREAM
    };
    return media_type;
}

#[cfg(test)]
mod test_m3u {
    use std::{env, time::Instant};

    use crate::{
        m3u::{get_channel_from_lines, get_m3u8_from_link},
        types::Source,
    };

    use super::read_m3u8;

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
        let source = Source {
            url: Some("/home/fred/Downloads/get.php".to_string()),
            name: "main".to_string(),
            id: None,
            password: None,
            username: None,
            url_origin: None,
            source_type: crate::source_type::M3U,
            enabled: true
        };
        read_m3u8(source).unwrap();
        std::fs::write("bench.txt", now.elapsed().as_millis().to_string()).unwrap();
    }

    #[tokio::test]
    async fn test_get_m3u8_from_link() {
        crate::sql::drop_db().unwrap_or_default();
        crate::sql::create_or_initialize_db().unwrap();
        let now = Instant::now();
        let source = Source {
            url: Some(env::var("OPEN_TV_TEST_LINK").unwrap()),
            name: "m3ulink1".to_string(),
            id: None,
            password: None,
            username: None,
            url_origin: None,
            source_type: crate::source_type::M3U_LINK,
            enabled: true
        };
        get_m3u8_from_link(source).await.unwrap();
        let time = now.elapsed().as_millis().to_string();
        println!("{time}");
        std::fs::write("bench2.txt", time).unwrap();
    }
}
