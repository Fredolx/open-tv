use crate::log;
use crate::media_type;
use crate::sql;
use crate::types::Channel;
use crate::types::ChannelPreserve;
use crate::types::Source;
use crate::types::EPG;
use crate::utils::get_local_time;
use anyhow::anyhow;
use anyhow::{Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use rusqlite::Transaction;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::join;
use url::Url;

const GET_LIVE_STREAMS: &str = "get_live_streams";
const GET_VODS: &str = "get_vod_streams";
const GET_SERIES: &str = "get_series";
const GET_SERIES_INFO: &str = "get_series_info";
const GET_SERIES_CATEGORIES: &str = "get_series_categories";
const GET_LIVE_STREAM_CATEGORIES: &str = "get_live_categories";
const GET_VOD_CATEGORIES: &str = "get_vod_categories";
const GET_SHORT_EPG: &str = "get_short_epg";
const LIVE_STREAM_EXTENSION: &str = "ts";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamStream {
    stream_id: serde_json::Value,
    name: Option<String>,
    category_id: serde_json::Value,
    stream_icon: Option<String>,
    series_id: serde_json::Value,
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
    episode_num: serde_json::Value,
    season: serde_json::Value,
    #[serde(default)]
    info: serde_json::Value,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamEpisodeInfo {
    movie_image: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamCategory {
    category_id: serde_json::Value,
    category_name: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamEPG {
    epg_listings: Vec<XtreamEPGItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamEPGItem {
    id: String,
    title: String,
    description: String,
    start_timestamp: String,
    stop_timestamp: String,
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

pub async fn get_xtream(mut source: Source, wipe: bool) -> Result<()> {
    let url = build_xtream_url(&mut source)?;
    let (live, live_cats, vods, vods_cats, series, series_cats) = join!(
        get_xtream_http_data::<Vec<XtreamStream>>(url.clone(), GET_LIVE_STREAMS),
        get_xtream_http_data::<Vec<XtreamCategory>>(url.clone(), GET_LIVE_STREAM_CATEGORIES),
        get_xtream_http_data::<Vec<XtreamStream>>(url.clone(), GET_VODS),
        get_xtream_http_data::<Vec<XtreamCategory>>(url.clone(), GET_VOD_CATEGORIES),
        get_xtream_http_data::<Vec<XtreamStream>>(url.clone(), GET_SERIES),
        get_xtream_http_data::<Vec<XtreamCategory>>(url.clone(), GET_SERIES_CATEGORIES),
    );
    let mut sql = sql::get_conn()?;
    let tx = sql.transaction()?;
    let mut channel_preserve: Vec<ChannelPreserve> = Vec::new();
    if wipe {
        channel_preserve = sql::get_channel_preserve(&tx, source.id.context("no source id")?)
            .unwrap_or(Vec::new());
        sql::wipe(&tx, source.id.context("Source should have id")?)?;
    } else {
        source.id = Some(sql::create_or_find_source_by_name(&tx, &source)?);
    }
    let mut fail_count = 0;
    live.and_then(|live| process_xtream(&tx, live, live_cats?, &source, media_type::LIVESTREAM))
        .unwrap_or_else(|e| {
            log::log(format!("{:?}", e));
            fail_count += 1;
        });
    vods.and_then(|vods: Vec<XtreamStream>| {
        process_xtream(&tx, vods, vods_cats?, &source, media_type::MOVIE)
    })
    .unwrap_or_else(|e| {
        log::log(format!("{:?}", e));
        fail_count += 1;
    });
    series
        .and_then(|series: Vec<XtreamStream>| {
            process_xtream(&tx, series, series_cats?, &source, media_type::SERIE)
        })
        .unwrap_or_else(|e| {
            log::log(format!("{:?}", e));
            fail_count += 1;
        });
    if fail_count > 2 {
        match tx.rollback() {
            Ok(_) => {}
            Err(e) => log::log(format!("Failed to rollback tx: {:?}", e)),
        }
        return Err(anyhow::anyhow!("Too many Xtream requests failed"));
    }
    if wipe {
        sql::restore_preserve(&tx, source.id.context("no source id")?, channel_preserve)?;
    }
    tx.commit()?;
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
    tx: &Transaction,
    streams: Vec<XtreamStream>,
    cats: Vec<XtreamCategory>,
    source: &Source,
    stream_type: u8,
) -> Result<()> {
    let cats: HashMap<String, String> = cats
        .into_iter()
        .filter_map(|f| {
            let category_id = get_serde_json_string(&f.category_id);
            category_id.map(|cid| (cid, f.category_name))
        })
        .collect();
    let mut groups: HashMap<String, i64> = HashMap::new();
    for live in streams {
        let category_name = get_cat_name(&cats, get_serde_json_string(&live.category_id));
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
    let stream_id = get_serde_json_number(&stream.stream_id).context("missing stream id")?;
    Ok(Channel {
        id: None,
        group: category_name.map(|x| x.trim().to_string()),
        image: stream
            .stream_icon
            .or(stream.cover)
            .map(|x| x.trim().to_string()),
        media_type: stream_type.clone(),
        name: stream.name.context("No name")?.trim().to_string(),
        source_id: source.id,
        url: if stream_type == media_type::SERIE {
            get_serde_json_string(&stream.series_id)
        } else {
            Some(get_url(
                stream_id.to_string(),
                source,
                stream_type,
                stream.container_extension,
            )?)
        },
        stream_id: Some(stream_id),
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

pub async fn get_episodes(channel: Channel) -> Result<()> {
    let series_id = channel.url.context("no url")?.parse()?;
    if sql::series_has_episodes(series_id, channel.source_id.context("no source id")?)
        .unwrap_or_else(|e| {
            log::log(format!("{:?}", e));
            return false;
        })
    {
        return Ok(());
    }
    let mut source = sql::get_source_from_id(channel.source_id.context("no source id")?)?;
    let mut url = build_xtream_url(&mut source)?;
    url.query_pairs_mut()
        .append_pair("series_id", &series_id.to_string());
    let episodes = (get_xtream_http_data::<XtreamSeries>(url, GET_SERIES_INFO).await?).episodes;
    let mut episodes: Vec<XtreamEpisode> =
        episodes.into_values().flat_map(|episode| episode).collect();
    episodes.sort_by(|a, b| {
        get_serde_json_number(&a.season)
            .cmp(&get_serde_json_number(&b.season))
            .then_with(|| {
                get_serde_json_number(&a.episode_num).cmp(&get_serde_json_number(&b.episode_num))
            })
    });
    sql::do_tx(|tx| {
        for episode in episodes {
            let episode = episode_to_channel(episode, &source, series_id)?;
            sql::insert_channel(&tx, episode)?;
        }
        Ok(())
    })?;
    Ok(())
}

fn get_serde_json_string(value: &serde_json::Value) -> Option<String> {
    value
        .as_str()
        .map(|cid| cid.to_string())
        .or_else(|| value.as_u64().map(|cid| cid.to_string()))
        .map(|cid| cid.trim().to_string())
}

fn get_serde_json_number(value: &serde_json::Value) -> Option<u64> {
    value
        .as_str()
        .and_then(|val| val.trim().parse::<u64>().ok())
        .or_else(|| value.as_u64())
}

fn episode_to_channel(episode: XtreamEpisode, source: &Source, series_id: u64) -> Result<Channel> {
    Ok(Channel {
        id: None,
        group: None,
        image: serde_json::from_value::<XtreamEpisodeInfo>(episode.info)
            .map(|e| e.movie_image)
            .unwrap_or_default(),
        media_type: media_type::MOVIE,
        name: episode.title.trim().to_string(),
        source_id: source.id,
        url: Some(get_url(
            episode.id,
            &source,
            media_type::SERIE,
            Some(episode.container_extension),
        )?),
        series_id: Some(series_id),
        stream_id: None,
        group_id: None,
        favorite: false,
    })
}

pub async fn get_short_epg(channel: Channel) -> Result<Vec<EPG>> {
    let mut source = sql::get_source_from_id(channel.source_id.context("no source id")?)?;
    let mut url = build_xtream_url(&mut source)?;
    url.query_pairs_mut().append_pair(
        "stream_id",
        &channel
            .stream_id
            .context("No stream id, please refresh your sources (Settings -> Refresh All) if you recently updated Open TV")?
            .to_string(),
    );
    let epg: XtreamEPG = get_xtream_http_data(url, GET_SHORT_EPG).await?;
    epg.epg_listings
        .iter()
        .map(xtream_epg_to_epg)
        .collect::<Result<Vec<EPG>>>()
}

fn xtream_epg_to_epg(epg: &XtreamEPGItem) -> Result<EPG> {
    Ok(EPG {
        epg_id: epg.id.clone(),
        title: String::from_utf8(BASE64_STANDARD.decode(&epg.title)?)?,
        description: String::from_utf8(BASE64_STANDARD.decode(&epg.description)?)?,
        start_time: get_local_time(epg.start_timestamp.parse()?)?
            .format("%Hh%M")
            .to_string(),
        end_time: get_local_time(epg.stop_timestamp.parse()?)?
            .format("%Hh%M")
            .to_string(),
        start_timestamp: epg.start_timestamp.parse()?,
    })
}

#[cfg(test)]
mod test_xtream {

    use std::env;

    use crate::source_type;
    use crate::sql::{self, drop_db};
    use crate::types::Source;
    use crate::xtream::{episode_to_channel, get_xtream};

    use super::{get_local_time, XtreamEpisode, XtreamSeries};

    #[tokio::test]
    async fn test_get_xtream() {
        drop_db().unwrap_or_default();
        sql::create_or_initialize_db().unwrap();
        get_xtream(
            Source {
                name: "my-xtream".to_string(),
                id: None,
                username: Some(env::var("OPEN_TV_TEST_XTREAM_USERNAME").unwrap()),
                password: Some(env::var("OPEN_TV_TEST_XTREAM_PASSWORD").unwrap()),
                url: Some(env::var("OPEN_TV_TEST_XTREAM_LINK").unwrap()),
                url_origin: None,
                source_type: source_type::XTREAM,
                enabled: true,
                use_tvg_id: None,
            },
            false,
        )
        .await
        .unwrap();
    }

    #[test]
    fn deserialize_bad_json() {
        let source = Source {
            name: "my-xtream".to_string(),
            id: None,
            username: Some("test".to_string()),
            password: Some("test".to_string()),
            url: Some("http://test.com".to_string()),
            url_origin: Some("test.com".to_string()),
            source_type: source_type::XTREAM,
            enabled: true,
            use_tvg_id: None,
        };
        let data = std::fs::read_to_string("/Users/fred/Desktop/bad.json").unwrap();
        let obj = serde_json::from_str::<XtreamSeries>(&data).unwrap();
        let episodes: Vec<XtreamEpisode> = obj
            .episodes
            .into_values()
            .flat_map(|episode| episode)
            .collect();
        let e = episodes.iter().find(|e| e.id == "29247").unwrap();
        let e = episode_to_channel(e.clone(), &source, 1).unwrap();
        let e2 = episodes.iter().find(|e| e.id == "29041").unwrap();
        let e2 = episode_to_channel(e2.clone(), &source, 1).unwrap();
        println!("{:?}", e);
        println!("{:?}", e2);
    }

    #[test]
    fn test_get_local_time() {
        println!("{}", get_local_time(1734217200).unwrap());
    }
}
