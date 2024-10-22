use std::collections::HashMap;
use std::str::FromStr;

use crate::media_type;
use crate::sql;
use crate::sql::delete_source;
use crate::types::Channel;
use crate::types::Source;
use anyhow::anyhow;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde::Serialize;
use tokio::join;
use url::Url;
use crate::log;

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
    container_extension: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamSeries {
    episodes: HashMap<String, Vec<XtreamEpisode>>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamEpisode {
    id: String,
    title: String,
    container_extension: String,
    info: Option<XtreamEpisodeInfo>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamEpisodeInfo {
    movie_image: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamCategory {
    category_id: String,
    category_name: String,
}

fn build_xtream_url(source: &mut Source) -> Result<Url> {
    let mut url = Url::parse(&source.url.clone().context("Missing URL")?)?;
    source.url_origin = Some(
        Url::from_str(&source.url.clone().unwrap())?
            .origin()
            .ascii_serialization(),
    );
    url.query_pairs_mut()
        .append_pair(
            "username",
            &source.username.clone().context("Missing username")?,
        )
        .append_pair(
            "password",
            &source.password.clone().context("Missing password")?,
        );
    Ok(url)
}

pub async fn get_xtream(mut source: Source) -> Result<()> {
    let url = build_xtream_url(&mut source)?;
    let new_source = sql::create_or_find_source_by_name(&mut source)?;

    let (live, live_cats, vods, vods_cats, series, series_cats) = join!(
        get_xtream_http_data::<Vec<XtreamStream>>(url.clone(), GET_LIVE_STREAMS),
        get_xtream_http_data::<Vec<XtreamCategory>>(url.clone(), GET_LIVE_STREAM_CATEGORIES),
        get_xtream_http_data::<Vec<XtreamStream>>(url.clone(), GET_VODS),
        get_xtream_http_data::<Vec<XtreamCategory>>(url.clone(), GET_VOD_CATEGORIES),
        get_xtream_http_data::<Vec<XtreamStream>>(url.clone(), GET_SERIES),
        get_xtream_http_data::<Vec<XtreamCategory>>(url.clone(), GET_SERIES_CATEGORIES),
    );
    let mut fail_count = 0;
    live.and_then(|live| process_xtream(live, live_cats?, &source, media_type::LIVESTREAM))
        .unwrap_or_else(|e| {
            log::log(format!("{:?}", e));
            fail_count += 1;
        });
    vods.and_then(|vods: Vec<XtreamStream>| {
        process_xtream(vods, vods_cats?, &source, media_type::MOVIE)
    })
    .unwrap_or_else(|e| {
        log::log(format!("{:?}", e));
        fail_count += 1;
    });
    series
        .and_then(|series: Vec<XtreamStream>| {
            process_xtream(series, series_cats?, &source, media_type::SERIE)
        })
        .unwrap_or_else(|e| {
            log::log(format!("{:?}", e));
            fail_count += 1;
        });
    if fail_count > 2 {
        if new_source {
            delete_source(source.id.context("no source id")?).unwrap_or_else(|e| log::log(format!("{:?}", e)));
        }
        return Err(anyhow::anyhow!("Too many Xtream requests failed"));
    }
    Ok(())
}

async fn get_xtream_http_data<T>(mut url: Url, action: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let client = reqwest::Client::new();
    url.query_pairs_mut().append_pair("action", action);
    let data = client.get(url).send().await?.json::<T>().await?;
    Ok(data)
}

fn process_xtream(
    streams: Vec<XtreamStream>,
    cats: Vec<XtreamCategory>,
    source: &Source,
    stream_type: u8,
) -> Result<()> {
    let cats: HashMap<String, String> = cats
        .into_iter()
        .map(|f| (f.category_id, f.category_name))
        .collect();
    let mut sql = sql::get_conn()?;
    let tx = sql.transaction()?;
    let mut groups: HashMap<String, i64> = HashMap::new();
    for live in streams {
        let category_name = get_cat_name(&cats, live.category_id.clone());
        convert_xtream_live_to_channel(live, &source, stream_type.clone(), category_name)
            .and_then(|mut channel| {
                sql::set_channel_group_id(
                    &mut groups,
                    &mut channel,
                    &tx,
                    source.id.as_ref().unwrap(),
                )
                .unwrap_or_else(|e| log::log(format!("{:?}", e)));
                sql::insert_channel(&tx, channel)?;
                Ok(())
            })
            .unwrap_or_else(|e| log::log(format!("{:?}", e)));
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
    stream_type: u8,
    category_name: Option<String>,
) -> Result<Channel> {
    Ok(Channel {
        id: None,
        group: category_name.map(|x| x.trim().to_string()),
        image: stream
            .stream_icon
            .or(stream.cover)
            .map(|x| x.trim().to_string()),
        media_type: stream_type.clone(),
        name: stream.name.context("No name")?.trim().to_string(),
        source_id: source.id.unwrap(),
        url: if stream_type == media_type::SERIE {
            Some(stream.series_id.context("no series id")?.to_string())
        } else {
            Some(get_url(
                stream.stream_id.context("no stream id")?.to_string(),
                source,
                stream_type,
                stream.container_extension,
            )?)
        },
        favorite: false,
        group_id: None,
        series_id: None,
    })
}

fn get_url(
    stream_id: String,
    source: &Source,
    stream_type: u8,
    extension: Option<String>,
) -> Result<String> {
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

fn get_media_type_string(stream_type: u8) -> Result<String> {
    match stream_type {
        media_type::LIVESTREAM => Ok("live".to_string()),
        media_type::MOVIE => Ok("movie".to_string()),
        media_type::SERIE => Ok("series".to_string()),
        _ => Err(anyhow!("Invalid stream_type")),
    }
}

pub async fn get_episodes(series_id: i64) -> Result<()> {
    if sql::series_has_episodes(series_id).unwrap_or_else(|e| {
        log::log(format!("{:?}", e));
        return false;
    }) {
        return Ok(());
    }
    let mut source = sql::get_source_from_series_id(series_id)?;
    let mut url = build_xtream_url(&mut source)?;
    url.query_pairs_mut()
        .append_pair("series_id", &series_id.to_string());
    let episodes = (get_xtream_http_data::<XtreamSeries>(url, GET_SERIES_INFO).await?).episodes;
    let episodes: Vec<XtreamEpisode> = episodes.into_values().flat_map(|episode| episode).collect();
    let mut sql = sql::get_conn()?;
    let tx = sql.transaction()?;
    for episode in episodes {
        let episode = episode_to_channel(episode, &source, series_id)?;
        log::log(format!("{:?}", episode));
        sql::insert_channel(&tx, episode)?;
    }
    tx.commit()?;
    Ok(())
}

fn episode_to_channel(episode: XtreamEpisode, source: &Source, series_id: i64) -> Result<Channel> {
    Ok(Channel {
        id: None,
        group: None,
        image: episode.info.map(|info| info.movie_image).unwrap_or(None),
        media_type: media_type::MOVIE,
        name: episode.title.trim().to_string(),
        source_id: source.id.context("Invalid ID")?,
        url: Some(get_url(
            episode.id,
            &source,
            media_type::SERIE,
            Some(episode.container_extension),
        )?),
        series_id: Some(series_id),
        group_id: None,
        favorite: false,
    })
}

#[cfg(test)]
mod test_xtream {

    use std::env;

    use crate::source_type;
    use crate::sql::{self, drop_db};
    use crate::types::Source;
    use crate::xtream::get_xtream;

    #[tokio::test]
    async fn test_get_xtream() {
        drop_db().unwrap_or_default();
        sql::create_or_initialize_db().unwrap();
        get_xtream(Source {
            name: "my-xtream".to_string(),
            id: None,
            username: Some(env::var("OPEN_TV_TEST_XTREAM_USERNAME").unwrap()),
            password: Some(env::var("OPEN_TV_TEST_XTREAM_PASSWORD").unwrap()),
            url: Some(env::var("OPEN_TV_TEST_XTREAM_LINK").unwrap()),
            url_origin: None,
            source_type: source_type::XTREAM,
            enabled: true,
        })
        .await
        .unwrap();
    }
}
