use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Local, NaiveDateTime};
use flate2::bufread::MultiGzDecoder;
use quick_xml::{
    Reader,
    events::{BytesStart, BytesText, Event},
};

use crate::{
    sql,
    types::{Channel, EPG, Source},
    utils::{get_local_time, get_user_agent_from_source},
};

const CACHE_MAX_AGE: Duration = Duration::from_secs(12 * 60 * 60);

enum Field {
    None,
    Title,
    Desc,
}

pub async fn get_epg(channel: Channel) -> Result<Vec<EPG>> {
    let tvg_id = channel.tvg_id.context("No tvg-id on channel")?;
    let source_id = channel.source_id.context("No source id")?;
    let source = sql::get_source_from_id(source_id)?;
    let path = get_cache_path(source_id);
    if !is_cache_fresh(&path) {
        download(&source, &path).await?;
    }
    tokio::task::spawn_blocking(move || read_programmes(&path, &tvg_id, source_id)).await?
}

pub fn delete_cache(source_id: i64) {
    let _ = std::fs::remove_file(get_cache_path(source_id));
}

fn get_cache_path(source_id: i64) -> PathBuf {
    let mut path = directories::ProjectDirs::from("dev", "fredol", "open-tv")
        .unwrap()
        .cache_dir()
        .to_owned();
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path.push(format!("epg-{source_id}.xmltv"));
    path
}

fn is_cache_fresh(path: &PathBuf) -> bool {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age < CACHE_MAX_AGE)
}

async fn download(source: &Source, path: &PathBuf) -> Result<()> {
    let url = source
        .epg_url
        .as_deref()
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .context("No EPG URL set for this source")?;
    let user_agent = get_user_agent_from_source(source)?;
    let client = reqwest::Client::builder().user_agent(user_agent).build()?;
    let mut response = client.get(url).send().await?;
    if !response.status().is_success() {
        bail!("Failed to get EPG, status: {}", response.status());
    }
    let tmp = path.with_extension("tmp");
    let mut file = File::create(&tmp)?;
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn read_programmes(path: &PathBuf, tvg_id: &str, source_id: i64) -> Result<Vec<EPG>> {
    let mut file = BufReader::new(File::open(path).context("Failed to open EPG cache")?);
    let is_gzip = file.fill_buf()?.starts_with(&[0x1f, 0x8b]);
    let input: Box<dyn BufRead> = if is_gzip {
        Box::new(BufReader::new(MultiGzDecoder::new(file)))
    } else {
        Box::new(file)
    };
    parse_programmes(input, tvg_id, source_id, Local::now().timestamp())
}

fn parse_programmes<R: BufRead>(
    input: R,
    tvg_id: &str,
    source_id: i64,
    now: i64,
) -> Result<Vec<EPG>> {
    let mut reader = Reader::from_reader(input);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut skip_buf = Vec::new();
    let mut epgs = Vec::new();
    let mut current: Option<(i64, i64)> = None;
    let mut field = Field::None;
    let mut title = String::new();
    let mut desc = String::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => match e.name().as_ref() {
                b"programme" => {
                    let times = (get_attr_time(&e, "start"), get_attr_time(&e, "stop"));
                    if let (true, Some(start), Some(stop)) = (is_channel(&e, tvg_id), times.0, times.1)
                    {
                        current = Some((start, stop));
                        title.clear();
                        desc.clear();
                    } else {
                        reader.read_to_end_into(e.name(), &mut skip_buf)?;
                        skip_buf.clear();
                    }
                }
                b"title" if current.is_some() && title.is_empty() => field = Field::Title,
                b"desc" if current.is_some() && desc.is_empty() => field = Field::Desc,
                _ => {}
            },
            Event::Text(t) => match field {
                Field::Title => title.push_str(&get_text(&t)),
                Field::Desc => desc.push_str(&get_text(&t)),
                Field::None => {}
            },
            Event::CData(c) => match field {
                Field::Title => title.push_str(&String::from_utf8_lossy(&c)),
                Field::Desc => desc.push_str(&String::from_utf8_lossy(&c)),
                Field::None => {}
            },
            Event::End(e) => match e.name().as_ref() {
                b"programme" => {
                    if let Some((start, stop)) = current.take() {
                        if stop > now {
                            epgs.push(to_epg(source_id, tvg_id, start, stop, &title, &desc, now)?);
                        }
                    }
                }
                b"title" | b"desc" => field = Field::None,
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    epgs.sort_by_key(|x| x.start_timestamp);
    Ok(epgs)
}

fn is_channel(e: &BytesStart, tvg_id: &str) -> bool {
    e.try_get_attribute("channel")
        .ok()
        .flatten()
        .and_then(|a| a.unescape_value().ok().map(|v| v.trim().eq_ignore_ascii_case(tvg_id)))
        .unwrap_or(false)
}

fn get_attr_time(e: &BytesStart, name: &str) -> Option<i64> {
    let attr = e.try_get_attribute(name).ok()??;
    parse_xmltv_time(&attr.unescape_value().ok()?)
}

fn get_text(t: &BytesText) -> String {
    t.unescape()
        .map(|x| x.into_owned())
        .unwrap_or_else(|_| String::from_utf8_lossy(t).into_owned())
}

fn parse_xmltv_time(value: &str) -> Option<i64> {
    let value = value.trim();
    DateTime::parse_from_str(value, "%Y%m%d%H%M%S %z")
        .map(|x| x.timestamp())
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value.get(..14)?, "%Y%m%d%H%M%S")
                .ok()
                .map(|x| x.and_utc().timestamp())
        })
}

fn to_epg(
    source_id: i64,
    tvg_id: &str,
    start: i64,
    stop: i64,
    title: &str,
    desc: &str,
    now: i64,
) -> Result<EPG> {
    Ok(EPG {
        epg_id: format!("{source_id}-{tvg_id}-{start}"),
        title: title.to_string(),
        description: desc.to_string(),
        start_time: get_local_time(start)?.format("%B %d, %H:%M").to_string(),
        end_time: get_local_time(stop)?.format("%B %d, %H:%M").to_string(),
        start_timestamp: start,
        timeshift_url: None,
        has_archive: false,
        now_playing: start <= now && now < stop,
    })
}

#[cfg(test)]
mod test_xmltv {
    use super::{parse_programmes, parse_xmltv_time};

    #[test]
    fn test_parse_xmltv_time() {
        assert_eq!(parse_xmltv_time("20261002120000 +0000"), Some(1790942400));
        assert_eq!(parse_xmltv_time("20261002130000 +0100"), Some(1790942400));
        assert_eq!(parse_xmltv_time("20261002120000"), Some(1790942400));
        assert_eq!(parse_xmltv_time("garbage"), None);
    }

    #[test]
    fn test_parse_programmes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tv>
  <channel id="bbc1.uk"><display-name>BBC One</display-name></channel>
  <programme start="20261002100000 +0000" stop="20261002110000 +0000" channel="bbc1.uk">
    <title lang="en">Finished</title>
  </programme>
  <programme start="20261002110000 +0000" stop="20261002130000 +0000" channel="other.uk">
    <title lang="en">Other channel</title>
  </programme>
  <programme start="20261002130000 +0000" stop="20261002140000 +0000" channel="BBC1.uk">
    <title lang="en">Later &amp; Later</title>
    <title lang="cy">Ail deitl</title>
    <desc lang="en"><![CDATA[A description]]></desc>
  </programme>
  <programme start="20261002110000 +0000" stop="20261002130000 +0000" channel="bbc1.uk">
    <title lang="en">On now</title>
  </programme>
</tv>"#;
        let now = 1790942400;
        let epgs = parse_programmes(xml.as_bytes(), "bbc1.uk", 7, now).unwrap();
        assert_eq!(epgs.len(), 2);
        assert_eq!(epgs[0].title, "On now");
        assert!(epgs[0].now_playing);
        assert_eq!(epgs[1].title, "Later & Later");
        assert_eq!(epgs[1].description, "A description");
        assert!(!epgs[1].now_playing);
        assert_eq!(epgs[1].epg_id, "7-bbc1.uk-1790946000");
    }
}
