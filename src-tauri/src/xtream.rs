use std::collections::HashMap;
use std::str::FromStr;

use crate::print_error_stack;
use crate::sql;
use crate::types::Channel;
use crate::types::MediaType;
use crate::types::Source;
use anyhow::anyhow;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde::Serialize;
use tokio::join;
use url::Url;

const GET_LIVE_STREAMS: &str = "get_live_streams";
const GET_VODS: &str = "get_vod_streams";
const GET_SERIES: &str = "get_series";
const GET_SERIES_INFO: &str = "get_series_info";
const GET_SERIES_CATEGORIES: &str = "get_series_categories";
const GET_LIVE_STREAM_CATEGORIES: &str = "get_live_categories";
const GET_VOD_CATEGORIES: &str = "get_vod_categories";
const LIVE_STREAM_EXTENSION: &str = "ts";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamStream {
    stream_id: Option<u64>,
    name: Option<String>,
    category_id: Option<String>,
    stream_icon: Option<String>,
    series_id: Option<u64>,
    cover: Option<String>,
    container_extension: Option<String>
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamCategory {
    category_id: String,
    category_name: String,
}

pub async fn get_xtream(mut source: Source) -> Result<()> {
    let mut url = Url::parse(&source.url.clone().context("Missing URL")?)?;
    source.url_origin = Some(
        Url::from_str(&source.url.clone().unwrap())?
            .origin()
            .ascii_serialization(),
    );
    sql::create_or_find_source_by_name(&mut source)?;
    url.query_pairs_mut()
        .append_pair(
            "username",
            &source.username.clone().context("Missing username")?,
        )
        .append_pair(
            "password",
            &source.password.clone().context("Missing password")?,
        );
    let (live, live_cats, vods, vods_cats, series, series_cats) = join!(
        get_xtream_http_data::<XtreamStream>(url.clone(), GET_LIVE_STREAMS),
        get_xtream_http_data::<XtreamCategory>(url.clone(), GET_LIVE_STREAM_CATEGORIES),
        get_xtream_http_data::<XtreamStream>(url.clone(), GET_VODS),
        get_xtream_http_data::<XtreamCategory>(url.clone(), GET_VOD_CATEGORIES),
        get_xtream_http_data::<XtreamStream>(url.clone(), GET_SERIES),
        get_xtream_http_data::<XtreamCategory>(url.clone(), GET_SERIES_CATEGORIES),
    );
    live.and_then(|live| process_xtream(live, live_cats?, &source, MediaType::Livestream))
        .unwrap_or_else(print_error_stack);
    vods.and_then(|vods: Vec<XtreamStream>| {
        process_xtream(vods, vods_cats?, &source, MediaType::Movie)
    })
    .unwrap_or_else(print_error_stack);
    series
        .and_then(|series: Vec<XtreamStream>| {
            process_xtream(series, series_cats?, &source, MediaType::Serie)
        })
        .unwrap_or_else(print_error_stack);
    Ok(())
}

async fn get_xtream_http_data<T>(mut url: Url, action: &str) -> Result<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    let client = reqwest::Client::new();
    url.query_pairs_mut().append_pair("action", action);
    let data = client.get(url).send().await?.json::<Vec<T>>().await?;
    Ok(data)
}

fn process_xtream(
    streams: Vec<XtreamStream>,
    cats: Vec<XtreamCategory>,
    source: &Source,
    stream_type: MediaType,
) -> Result<()> {
    let cats: HashMap<String, String> = cats.into_iter().map(|f| (f.category_id, f.category_name)).collect();
    let mut sql = sql::CONN.lock().unwrap();
    let tx = sql.transaction()?;
    for live in streams {
        let category_name = get_cat_name(&cats, live.category_id.clone());
        convert_xtream_live_to_channel(live, &source, stream_type.clone(), category_name)
            .and_then(|channel| sql::insert_channel(&tx, channel))
            .unwrap_or_else(print_error_stack);
    }
    tx.commit()?;
    Ok(())
}

fn get_cat_name(cats: &HashMap<String, String>, category_id: Option<String>) -> Option<String> {
    if category_id.is_none() {
        return None;
    }
    return cats.get(&category_id.unwrap()).map(|t| t.to_string());
}

fn convert_xtream_live_to_channel(
    stream: XtreamStream,
    source: &Source,
    stream_type: MediaType,
    category_name: Option<String>,
) -> Result<Channel> {
    Ok(Channel {
        group: category_name,
        image: stream.stream_icon.or(stream.cover),
        media_type: stream_type.clone(),
        name: stream.name.context("No name")?,
        source_id: source.id.unwrap(),
        url: if stream_type == MediaType::Serie {
            stream.series_id.context("no series id")?.to_string()
        } else {
            get_url(
                stream.stream_id.context("no stream id")?,
                source,
                stream_type,
                stream.container_extension
            )?
        },
    })
}

fn get_url(stream_id: u64, source: &Source, stream_type: MediaType, extension: Option<String>) -> Result<String> {
    Ok(format!(
        "{}/{}/{}/{}/{}.{}",
        source.url_origin.clone().unwrap(),
        get_media_type_string(stream_type)?,
        source.username.clone().unwrap(),
        source.password.clone().unwrap(),
        stream_id,
        extension.unwrap_or(LIVE_STREAM_EXTENSION.to_string())
    ))
}

fn get_media_type_string(stream_type: MediaType) -> Result<String> {
    match stream_type {
        MediaType::Livestream => Ok("live".to_string()),
        MediaType::Movie => Ok("movie".to_string()),
        MediaType::Serie => Ok("series".to_string()),
        _ => Err(anyhow!("my error")),
    }
}

#[cfg(test)]
mod test_xtream {

    use std::env;

    use crate::sql::{self, drop_db};
    use crate::types::Source;
    use crate::xtream::get_xtream;

    #[tokio::test]
    async fn test_get_xtream() {
        drop_db().unwrap();
        sql::create_or_initialize_db().unwrap();
        get_xtream(Source {
            name: "my-xtream".to_string(),
            id: None,
            username: Some(env::var("OPEN_TV_TEST_XTREAM_USERNAME").unwrap()),
            password: Some(env::var("OPEN_TV_TEST_XTREAM_PASSWORD").unwrap()),
            url: Some(env::var("OPEN_TV_TEST_XTREAM_LINK").unwrap()),
            url_origin: None,
            source_type: crate::types::SourceType::Xtream
        }).await.unwrap();
    }
}
