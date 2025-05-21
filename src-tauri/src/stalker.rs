use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Result, bail};
use reqwest::Client;
use rusqlite::Transaction;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use url::Url;

use crate::{
    media_type, sql, stalker_type,
    types::{Channel, ChannelPreserve, Source},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerJsData {
    js: StalkerData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerData {
    total_items: Option<u32>,
    max_page_items: Option<u32>,
    data: Vec<StalkerItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerItem {
    name: String,
    cmd: String,
    #[serde(default)]
    tv_genre_id: Option<String>,
    #[serde(default)]
    category_id: Option<String>,
    #[serde(default)]
    logo: Option<String>,
    #[serde(default)]
    screenshot_uri: Option<String>,
    #[serde(default)]
    enable_tv_archive: Option<u8>,
    #[serde(default)]
    tv_archive_duration: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerSeriesEpisode {
    name: String,
    series: Vec<u32>,
    screenshot_uri: String,
    cmd: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerJs<T> {
    js: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct HandshakeData {
    token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerGenre {
    id: String,
    title: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CreateStreamDTO {
    cmd: String,
}

pub async fn get_stalker(source: Source, wipe: bool) -> Result<()> {
    let url = source.url.as_ref().context("no url")?;
    let mac = source
        .username
        .as_ref()
        .context("no mac address in source.username")?;
    let token = get_token(url, mac).await?;
    let live_cats = get_genres(url, &token, mac).await?;
    let vod_cats = get_categories(url, stalker_type::VOD, &token, mac).await?;
    let series_cats = get_categories(url, stalker_type::SERIES, &token, mac).await?;
    let lives = get_all_channels(url, stalker_type::LIVE, &token, mac).await?;
    let vods = get_all_channels(url, stalker_type::VOD, &token, mac).await?;
    let series = get_all_channels(url, stalker_type::SERIES, &token, mac).await?;
    let mut channel_preserve: Vec<ChannelPreserve> = Vec::new();
    sql::do_tx(|tx| {
        let mut source_id = source.id.unwrap_or(-1);
        if wipe {
            if source_id == -1 {
                bail!("invalid source id");
            }
            channel_preserve = sql::get_channel_preserve(tx, source_id).unwrap_or(Vec::new());
            sql::wipe(&tx, source_id)?;
        } else {
            source_id = sql::create_or_find_source_by_name(tx, &source)?;
        }
        process_channels(live_cats, lives, stalker_type::LIVE, source_id, tx)?;
        process_channels(vod_cats, vods, stalker_type::VOD, source_id, tx)?;
        process_channels(series_cats, series, stalker_type::SERIES, source_id, tx)?;
        if wipe {
            sql::restore_preserve(&tx, source_id, channel_preserve)?;
        }
        Ok(())
    })
}

fn process_channels(
    cats: HashMap<String, String>,
    items: Vec<StalkerItem>,
    content_type: &str,
    source_id: i64,
    tx: &Transaction,
) -> Result<()> {
    let mut groups: HashMap<String, i64> = HashMap::new();
    for item in items {
        let category_name = item
            .category_id
            .as_ref()
            .or(item.tv_genre_id.as_ref())
            .and_then(|f| cats.get(f).map(|x| x.to_string()));
        let mut channel = stalker_to_channel(item, category_name, content_type, source_id);
        sql::set_channel_group_id(&mut groups, &mut channel, tx, &source_id)?;
        sql::insert_channel(tx, channel)?;
    }
    Ok(())
}

fn stalker_to_channel(
    item: StalkerItem,
    category_name: Option<String>,
    content_type: &str,
    source_id: i64,
) -> Channel {
    Channel {
        name: item.name,
        group: category_name,
        image: item.logo.or(item.screenshot_uri),
        url: Some(item.cmd),
        media_type: stalker_type_to_otv_type(content_type),
        tv_archive: item.enable_tv_archive.map(|f| f == 1),
        source_id: Some(source_id),
        ..Default::default()
    }
}

fn stalker_type_to_otv_type(stalker_type: &str) -> u8 {
    match stalker_type {
        stalker_type::LIVE => media_type::LIVESTREAM,
        stalker_type::VOD => media_type::MOVIE,
        stalker_type::SERIES => media_type::SERIE,
        _ => media_type::LIVESTREAM,
    }
}

fn build_stalker_client() -> Result<Client> {
    let client = Client::builder().user_agent("Mozilla/5.0 (QtEmbedded; U; Linux; C) AppleWebKit/533.3 (KHTML, like Gecko) MAG200 stbapp ver: 2 rev: 250 Safari/533.3 X-User-Agent: Model: MAG250; Link: WiFi").build()?;
    Ok(client)
}

pub async fn get_token(url: &str, mac: &str) -> Result<String> {
    let client = build_stalker_client()?;
    let mut url = Url::from_str(&url)?;
    url.query_pairs_mut()
        .append_pair("mac", &mac)
        .append_pair("type", "stb")
        .append_pair("action", "handshake");
    Ok(client
        .get(url)
        .send()
        .await?
        .json::<StalkerJs<HandshakeData>>()
        .await?
        .js
        .token)
}

async fn get_all_channels(
    url: &str,
    content_type: &str,
    token: &str,
    mac: &str,
) -> Result<Vec<StalkerItem>> {
    let mut channels: Vec<StalkerItem> = Vec::new();
    let result: StalkerJsData = get_ordered_list(url, content_type, 1, token, mac).await?;
    let total_items = result.js.total_items.context("no total items")?;
    let max_page_items = result.js.max_page_items.context("no max page items")?;
    let pages = (total_items as f32 / max_page_items as f32).ceil() as u32;
    let mut set = JoinSet::new();
    println!("pages: {}", pages);
    for page in 1..=pages {
        let url = url.to_string();
        let content_type = content_type.to_string();
        let token = token.to_string();
        let mac = mac.to_string();
        set.spawn(async move { get_ordered_list(&url, &content_type, page, &token, &mac).await });
    }
    while let Some(res) = set.join_next().await {
        println!("data collected");
        if let Ok(Ok(data)) = res {
            let data: StalkerJsData = data;
            channels.extend(data.js.data);
        }
    }
    Ok(channels)
}

async fn get_ordered_list<T>(
    url: &str,
    content_type: &str,
    page: u32,
    token: &str,
    mac: &str,
) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let client = build_stalker_client()?;
    let mut url = Url::from_str(url)?;
    url.query_pairs_mut()
        .append_pair("type", content_type)
        .append_pair("token", token)
        .append_pair("mac", mac)
        .append_pair("p", &page.to_string())
        .append_pair("action", "get_ordered_list");
    Ok(client.get(url).send().await?.json::<T>().await?)
}

async fn get_categories(
    url: &str,
    content_type: &str,
    token: &str,
    mac: &str,
) -> Result<HashMap<String, String>> {
    let client = build_stalker_client()?;
    if content_type == stalker_type::LIVE {
        bail!("get_categories does not support LIVE (itv)")
    }
    let mut url = Url::from_str(&url)?;
    url.query_pairs_mut()
        .append_pair("type", content_type)
        .append_pair("token", token)
        .append_pair("mac", mac)
        .append_pair("action", "get_categories");
    let result = client
        .get(url)
        .send()
        .await?
        .json::<StalkerJs<Vec<StalkerGenre>>>()
        .await?;
    Ok(result.js.into_iter().map(|f| (f.id, f.title)).collect())
}

async fn get_genres(url: &str, token: &str, mac: &str) -> Result<HashMap<String, String>> {
    let client = build_stalker_client()?;
    let mut url = Url::from_str(&url)?;
    url.query_pairs_mut()
        .append_pair("type", stalker_type::LIVE)
        .append_pair("token", token)
        .append_pair("mac", mac)
        .append_pair("action", "get_genres");
    let result = client
        .get(url)
        .send()
        .await?
        .json::<StalkerJs<Vec<StalkerGenre>>>()
        .await?;
    Ok(result.js.into_iter().map(|f| (f.id, f.title)).collect())
}

pub async fn create_stream(
    url: &str,
    cmd: String,
    mac: &str,
    episode: Option<u32>,
) -> Result<String> {
    let client = build_stalker_client()?;
    let token = get_token(url, mac).await?;
    let mut url = Url::from_str(&url)?;
    url.query_pairs_mut()
        .append_pair("type", stalker_type::LIVE)
        .append_pair("mac", mac)
        .append_pair("action", "create_link")
        .append_pair("token", &token)
        .append_pair("cmd", &cmd);
    if let Some(episode) = episode {
        url.query_pairs_mut()
            .append_pair("series", &episode.to_string());
    }
    Ok(client
        .get(url)
        .send()
        .await?
        .json::<StalkerJs<CreateStreamDTO>>()
        .await?
        .js
        .cmd)
}

async fn get_episodes(url: &str, movie_id: &str, mac: &str) -> Result<StalkerSeriesEpisode> {
    let client = build_stalker_client()?;
    let token = get_token(url, mac).await?;
    let mut url = Url::from_str(&url)?;
    url.query_pairs_mut()
        .append_pair("token", &token)
        .append_pair("mac", mac)
        .append_pair("type", "series")
        .append_pair("action", "get_ordered_list")
        .append_pair("movie_id", movie_id);
    Ok(client.get(url).send().await?.json().await?)
}
