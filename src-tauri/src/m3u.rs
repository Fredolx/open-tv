use std::{
    collections::HashMap, fs::File, io::{BufRead, BufReader, Lines, Write}, iter::{Enumerate, Skip}, sync::LazyLock
};

use anyhow::{bail, Context, Result};
use regex::{Captures, Regex};
use types::{Channel, Source};

use crate::{
    log, media_type, source_type,
    sql::{self, delete_source},
    types::{self, ChannelHttpHeaders},
};

static NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"tvg-name="(?P<name>[^"]*)""#).unwrap());
static ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"tvg-id="(?P<id>[^"]*)""#).unwrap());
static LOGO_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"tvg-logo="(?P<logo>[^"]*)""#).unwrap());
static GROUP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"group-title="(?P<group>[^"]*)""#).unwrap());

static HTTP_ORIGIN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"http-origin=(?P<origin>.+)"#).unwrap());
static HTTP_REFERRER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"http-referrer=(?P<referrer>.+)"#).unwrap());
static HTTP_USER_AGENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"http-user-agent=(?P<user_agent>.+)"#).unwrap());

pub fn read_m3u8(mut source: Source) -> Result<()> {
    let path = match source.source_type {
        source_type::M3U_LINK => get_tmp_path(),
        _ => source.url.clone().context("no file path found")?,
    };
    let file = File::open(path).context("Failed to open m3u8 file")?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().enumerate().skip(1);
    let mut problematic_lines: usize = 0;
    let mut lines_count: usize = 0;
    let mut groups: HashMap<String, i64> = HashMap::new();
    let mut sql = sql::get_conn()?;
    let tx = sql.transaction()?;
    let source_id = sql::create_or_find_source_by_name(&tx, &source)?;
    while let (Some((c1, l1)), Some((c2, l2))) = (lines.next(), lines.next()) {
        lines_count = c2;
        let l1 = match l1.with_context(|| format!("(l1) Error on line: {c1}, skipping")) {
            Ok(line) => line,
            Err(e) => {
                log::log(format!("{:?}", e));
                problematic_lines += 1;
                continue;
            }
        };
        let mut l2 = match l2.with_context(|| format!("(l2) Error on line: {c2}, skipping")) {
            Ok(line) => line,
            Err(e) => {
                log::log(format!("{:?}", e));
                problematic_lines += 1;
                continue;
            }
        };
        let mut headers: Option<ChannelHttpHeaders> = None;
        if l2.starts_with("#EXTVLCOPT") {
            let (fail, _headers) = extract_headers(&mut l2, &mut lines)?; 
            if fail {
                continue;
            }
            headers = _headers;
        }
        let mut channel = match get_channel_from_lines(l1, l2, source_id)
            .with_context(|| format!("Failed to process lines #{c1} #{c2}, skipping"))
        {
            Ok(val) => val,
            Err(e) => {
                log::log(format!("{:?}", e));
                problematic_lines += 2;
                continue;
            }
        };
        sql::set_channel_group_id(&mut groups, &mut channel, &tx, source.id.as_ref().unwrap())
            .unwrap_or_else(|e| log::log(format!("{:?}", e)));
        sql::insert_channel(&tx, channel)?;
        if let Some(mut headers) = headers {
            headers.channel_id = Some(tx.last_insert_rowid());
            sql::insert_channel_headers(&tx, headers)?;
        }
    }
    if problematic_lines > lines_count / 2 {
        tx.rollback()
            .unwrap_or_else(|e| log::log(format!("{:?}", e)));
        return Err(anyhow::anyhow!(
            "Too many problematic lines, read considered failed"
        ));
    }
    tx.commit()?;
    Ok(())
}

pub async fn get_m3u8_from_link(source: Source) -> Result<()> {
    let client = reqwest::Client::new();
    let url = source.url.clone().context("Invalid source")?;
    let mut response = client.get(&url).send().await?;

    let mut file = std::fs::File::create(get_tmp_path())?;
    while let Some(chunk) = response.chunk().await? {
        file.write(&chunk)?;
    }
    read_m3u8(source)
}

fn get_tmp_path() -> String {
    let mut path = directories::ProjectDirs::from("dev", "fredol", "open-tv")
        .unwrap()
        .cache_dir()
        .to_owned();
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path.push("get.m3u");
    return path.to_string_lossy().to_string();
}

fn extract_non_empty_capture(caps: Captures) -> Option<String> {
    caps.get(1)
        .map(|m| m.as_str().to_string())
        .filter(|s| !s.trim().is_empty())
}

fn extract_headers(l2: &mut String, lines: &mut Skip<Enumerate<Lines<BufReader<File>>>>) -> Result<(bool, Option<ChannelHttpHeaders>)> {
    let mut headers = ChannelHttpHeaders {
        id: None,
        channel_id: None,
        http_origin: None,
        referrer: None,
        user_agent: None,
        ignore_ssl: None
    };
    let mut at_least_one: bool = false;
    while l2.starts_with("#EXTVLCOPT") {
        let result = set_http_headers(&l2, &mut headers);
        if result && !at_least_one {
            at_least_one = true;
        }
        let result = lines.next().context("EOF?")?;
        if let Ok(line) = result.1 {
            l2.clear();
            l2.push_str(&line);
        }
        else {
            log::log(format!("{:?}", 
                result.1.context(format!("Failed to get line at {}", result.0)).unwrap_err())
            );
            return Ok((true, None));
        }
    }
    if at_least_one {
        return Ok((false, Some(headers)));
    }
    else {
        return Ok((true, None));
    }
    
}

fn set_http_headers(line: &str, headers: &mut ChannelHttpHeaders) -> bool {
    if let Some(origin) = HTTP_ORIGIN_REGEX
        .captures(&line)
        .and_then(extract_non_empty_capture)
    {
        headers.http_origin = Some(origin);
        return true;
    } else if let Some(referrer) = HTTP_REFERRER_REGEX
        .captures(&line)
        .and_then(extract_non_empty_capture)
    {
        headers.referrer = Some(referrer);
        return true;
    } else if let Some(user_agent) = HTTP_USER_AGENT_REGEX
        .captures(&line)
        .and_then(extract_non_empty_capture)
    {
        headers.user_agent = Some(user_agent);
        return true;
    }
    return false;
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
        source_id: Some(source_id),
        series_id: None,
        group_id: None,
        favorite: false,
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
            enabled: true,
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
            enabled: true,
        };
        get_m3u8_from_link(source).await.unwrap();
        let time = now.elapsed().as_millis().to_string();
        println!("{time}");
        std::fs::write("bench2.txt", time).unwrap();
    }
}
