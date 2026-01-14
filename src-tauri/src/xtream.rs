use crate::log;
use crate::media_type;
use crate::sql;
use crate::sql::insert_season;
use crate::types::Channel;
use crate::types::ChannelPreserve;
use crate::types::EPG;
use crate::types::Season;
use crate::types::Source;
use crate::utils::get_local_time;
use crate::utils::get_user_agent_from_source;
use anyhow::anyhow;
use anyhow::{Context, Result};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::DateTime;
use chrono::Local;
use chrono::NaiveDateTime;
use reqwest::Client;
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
const GET_EPG: &str = "get_simple_data_table";
const LIVE_STREAM_EXTENSION: &str = "ts";
const NO_SEASON_NUMBER: i64 = -9999;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamStream {
    #[serde(default)]
    stream_id: serde_json::Value,
    name: Option<String>,
    #[serde(default)]
    category_id: serde_json::Value,
    stream_icon: Option<String>,
    #[serde(default)]
    series_id: serde_json::Value,
    cover: Option<String>,
    container_extension: Option<String>,
    #[serde(default)]
    tv_archive: serde_json::Value,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamSeries {
    seasons: Vec<XtreamSeason>,
    episodes: HashMap<String, Vec<XtreamEpisode>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamSeason {
    #[serde(default)]
    season_number: serde_json::Value,
    #[serde(default)]
    overview: Option<String>,
    #[serde(default)]
    cover: Option<String>,
    #[serde(default)]
    cover_tmdb: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamEpisode {
    id: serde_json::Value,
    title: String,
    container_extension: String,
    #[serde(default)]
    episode_num: serde_json::Value,
    #[serde(default)]
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
    #[serde(default)]
    category_id: serde_json::Value,
    category_name: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamEPG {
    epg_listings: Vec<XtreamEPGItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct XtreamEPGItem {
    id: serde_json::Value,
    title: String,
    description: String,
    start_timestamp: serde_json::Value,
    stop_timestamp: serde_json::Value,
    now_playing: u8,
    has_archive: u8,
    start: String,
    end: String,
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
    let user_agent = get_user_agent_from_source(&source)?;
    let (live, live_cats, vods, vods_cats, series, series_cats) = join!(
        get_xtream_http_data::<Vec<XtreamStream>>(url.clone(), GET_LIVE_STREAMS, &user_agent),
        get_xtream_http_data::<Vec<XtreamCategory>>(
            url.clone(),
            GET_LIVE_STREAM_CATEGORIES,
            &user_agent
        ),
        get_xtream_http_data::<Vec<XtreamStream>>(url.clone(), GET_VODS, &user_agent),
        get_xtream_http_data::<Vec<XtreamCategory>>(url.clone(), GET_VOD_CATEGORIES, &user_agent),
        get_xtream_http_data::<Vec<XtreamStream>>(url.clone(), GET_SERIES, &user_agent),
        get_xtream_http_data::<Vec<XtreamCategory>>(
            url.clone(),
            GET_SERIES_CATEGORIES,
            &user_agent
        ),
    );
    let mut sql = sql::get_conn()?;
    let tx = sql.transaction()?;
    let mut channel_preserve: Vec<ChannelPreserve> = Vec::new();
    if wipe {
        channel_preserve = sql::get_preserve(&tx, source.id.context("no source id")?)
            .unwrap_or_default();
        sql::wipe(&tx, source.id.context("Source should have id")?)?;
    } else {
        source.id = Some(sql::create_or_find_source_by_name(&tx, &source)?);
    }
    let mut fail_count = 0;
    live.and_then(|live| process_xtream(&tx, live, live_cats?, &source, media_type::LIVESTREAM))
        .unwrap_or_else(|e| {
            log::log(format!("{:?}", e.context("Failed to process live")));
            fail_count += 1;
        });
    vods.and_then(|vods: Vec<XtreamStream>| {
        process_xtream(&tx, vods, vods_cats?, &source, media_type::MOVIE)
    })
    .unwrap_or_else(|e| {
        log::log(format!("{:?}", e.context("Failed to process vods")));
        fail_count += 1;
    });
    series
        .and_then(|series: Vec<XtreamStream>| {
            process_xtream(&tx, series, series_cats?, &source, media_type::SERIE)
        })
        .unwrap_or_else(|e| {
            log::log(format!("{:?}", e.context("Failed to process series")));
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
    sql::analyze(&tx)?;
    tx.commit()?;
    Ok(())
}

async fn get_xtream_http_data<T>(mut url: Url, action: &str, user_agent: &String) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let client = Client::builder().user_agent(user_agent).build()?;
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
    let stream_id = get_serde_json_u64(&stream.stream_id);
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
                stream_id.context("missing stream id")?.to_string(),
                source,
                stream_type,
                stream.container_extension,
            )?)
        },
        stream_id,
        favorite: false,
        group_id: None,
        series_id: None,
        tv_archive: get_serde_json_u64(&stream.tv_archive).map(|x| x == 1),
        season_id: None,
        episode_num: None,
        hidden: false,
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
    let user_agent = get_user_agent_from_source(&source)?;
    url.query_pairs_mut()
        .append_pair("series_id", &series_id.to_string());
    let mut series =
        get_xtream_http_data::<XtreamSeries>(url, GET_SERIES_INFO, &user_agent).await?;
    let mut episodes: Vec<XtreamEpisode> = series
        .episodes
        .into_values()
        .flat_map(|episode| episode)
        .collect();
    series
        .seasons
        .sort_by_key(|f| get_serde_json_i64(&f.season_number));
    let seasons: HashMap<i64, XtreamSeason> = series
        .seasons
        .iter()
        .filter_map(|x| get_serde_json_i64(&x.season_number).map(|y| (y, x.clone())))
        .collect();
    episodes.sort_by(|a, b| {
        get_serde_json_u64(&a.season)
            .cmp(&get_serde_json_u64(&b.season))
            .then_with(|| {
                get_serde_json_u64(&a.episode_num).cmp(&get_serde_json_u64(&b.episode_num))
            })
    });
    insert_episodes(&source, seasons, episodes, series_id, channel.image)?;
    Ok(())
}

fn insert_episodes(
    source: &Source,
    seasons: HashMap<i64, XtreamSeason>,
    episodes: Vec<XtreamEpisode>,
    series_id: u64,
    default_season_image: Option<String>,
) -> Result<()> {
    let mut seasons_db: HashMap<i64, i64> = HashMap::new();
    sql::do_tx(|tx| {
        for episode in episodes {
            match insert_episode(
                episode.clone(),
                source,
                tx,
                &mut seasons_db,
                &seasons,
                series_id,
                default_season_image.clone(),
            )
            .with_context(|| format!("Failed to insert episode {:?}", episode))
            {
                Ok(_) => (),
                Err(e) => {
                    log::log(format!("{:?}", e));
                    continue;
                }
            }
        }
        Ok(())
    })
}

fn insert_episode(
    episode: XtreamEpisode,
    source: &Source,
    tx: &Transaction,
    seasons_db: &mut HashMap<i64, i64>,
    seasons: &HashMap<i64, XtreamSeason>,
    series_id: u64,
    default_season_image: Option<String>,
) -> Result<()> {
    let season_number = get_serde_json_i64(&episode.season).unwrap_or(NO_SEASON_NUMBER);
    let season_id = seasons_db.get(&season_number);
    let season_id: i64 = match season_id {
        Some(s) => s.clone(),
        None => {
            let season = seasons
                .get(&season_number)
                .and_then(|f| {
                    xtream_season_to_season(f.clone(), source.id.unwrap(), series_id)
                        .with_context(|| "Failed to convert XtreamSeason to Season")
                        .inspect_err(|e| log::log(format!("{}", e)))
                        .ok()
                })
                .unwrap_or_else(|| {
                    create_makeshift_season(
                        season_number,
                        series_id,
                        source.id.unwrap(),
                        default_season_image,
                    )
                });
            let id = insert_season(tx, season)?;
            seasons_db.insert(season_number, id);
            id
        }
    };
    let episode = episode_to_channel(episode, &source, series_id, season_id)?;
    sql::insert_channel(&tx, episode)?;
    Ok(())
}

fn create_makeshift_season(
    number: i64,
    series_id: u64,
    source_id: i64,
    image: Option<String>,
) -> Season {
    Season {
        name: match number == NO_SEASON_NUMBER {
            true => "Uncategorized".to_string(),
            false => format!("Season {number}"),
        },
        series_id,
        season_number: number,
        source_id,
        image,
        ..Default::default()
    }
}

fn get_serde_json_string(value: &serde_json::Value) -> Option<String> {
    value
        .as_str()
        .map(|cid| cid.to_string())
        .or_else(|| value.as_u64().map(|cid| cid.to_string()))
        .map(|cid| cid.trim().to_string())
}

fn get_serde_json_u64(value: &serde_json::Value) -> Option<u64> {
    value
        .as_str()
        .and_then(|val| val.trim().parse::<u64>().ok())
        .or_else(|| value.as_u64())
}

fn get_serde_json_i64(value: &serde_json::Value) -> Option<i64> {
    value
        .as_str()
        .and_then(|val| val.trim().parse::<i64>().ok())
        .or_else(|| value.as_i64())
}

fn xtream_season_to_season(season: XtreamSeason, source_id: i64, series_id: u64) -> Result<Season> {
    let season_number = get_serde_json_i64(&season.season_number).context("no season number")?;
    Ok(Season {
        season_number,
        series_id,
        source_id,
        image: season.cover_tmdb.or(season.cover).or(season.overview),
        name: format!("Season {season_number}"),
        ..Default::default()
    })
}

fn episode_to_channel(
    episode: XtreamEpisode,
    source: &Source,
    series_id: u64,
    season_id: i64,
) -> Result<Channel> {
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
            get_serde_json_string(&episode.id).context("no id")?,
            &source,
            media_type::SERIE,
            Some(episode.container_extension),
        )?),
        series_id: Some(series_id),
        episode_num: get_serde_json_i64(&episode.episode_num),
        season_id: Some(season_id),
        stream_id: None,
        group_id: None,
        favorite: false,
        tv_archive: None,
        hidden: false,
    })
}

pub async fn get_epg(channel: Channel) -> Result<Vec<EPG>> {
    let mut source = sql::get_source_from_id(channel.source_id.context("no source id")?)?;
    let mut url = build_xtream_url(&mut source)?;
    let user_agent = get_user_agent_from_source(&source)?;
    let stream_id = channel.stream_id.context("No stream id")?.to_string();
    url.query_pairs_mut().append_pair("stream_id", &stream_id);
    let epg: XtreamEPG = get_xtream_http_data(url, GET_EPG, &user_agent).await?;
    let url = get_timeshift_url_base(&source)?;
    let current_time = Local::now();
    let mut otv_epgs = Vec::new();
    for item in epg.epg_listings {
        let item = xtream_epg_to_epg(item, &url, &stream_id)?;
        if is_valid_epg(&item, &current_time)? {
            otv_epgs.push(item);
        }
    }
    Ok(otv_epgs)
}

fn is_valid_epg(epg: &EPG, now: &DateTime<Local>) -> Result<bool> {
    let epg_start_local = crate::utils::get_local_time(epg.start_timestamp)?;
    if epg_start_local < *now && !epg.has_archive && !epg.now_playing {
        return Ok(false);
    }
    Ok(true)
}

fn xtream_epg_to_epg(epg: XtreamEPGItem, url: &Url, stream_id: &str) -> Result<EPG> {
    let start_timestamp =
        get_serde_json_i64(&epg.start_timestamp).context("no valid start timestamp")?;
    Ok(EPG {
        epg_id: get_serde_json_string(&epg.id).context("no epg id")?,
        title: String::from_utf8(BASE64_STANDARD.decode(&epg.title)?)?,
        description: String::from_utf8(BASE64_STANDARD.decode(&epg.description)?)?,
        start_time: get_local_time(start_timestamp)?
            .format("%B %d, %H:%M")
            .to_string(),
        end_time: get_local_time(
            get_serde_json_i64(&epg.stop_timestamp).context("no valid end timestamp")?,
        )?
        .format("%B %d, %H:%M")
        .to_string(),
        start_timestamp,
        timeshift_url: if epg.has_archive == 1 {
            Some(get_timeshift_url(
                url.clone(),
                epg.start,
                epg.end,
                stream_id,
            )?)
        } else {
            None
        },
        has_archive: epg.has_archive == 1,
        now_playing: epg.now_playing == 1,
    })
}

fn get_timeshift_url_base(source: &Source) -> Result<Url> {
    let mut url = Url::parse(source.url_origin.as_ref().context("no origin")?)?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Can't mutate url"))?
        .extend(&["streaming", "timeshift.php"]);
    url.query_pairs_mut()
        .append_pair("username", source.username.as_ref().context("no username")?)
        .append_pair("password", source.password.as_ref().context("no password")?);
    Ok(url)
}

fn get_timeshift_url(mut url: Url, start: String, end: String, stream_id: &str) -> Result<String> {
    let start = NaiveDateTime::parse_from_str(&start, "%Y-%m-%d %H:%M:%S")?;
    let duration = NaiveDateTime::parse_from_str(&end, "%Y-%m-%d %H:%M:%S")?
        .signed_duration_since(start)
        .num_minutes()
        .to_string();
    let start = start.format("%Y-%m-%d:%H-%M").to_string();
    url.query_pairs_mut()
        .append_pair("stream", stream_id)
        .append_pair("start", &start)
        .append_pair("duration", &duration);
    Ok(url.to_string())
}
