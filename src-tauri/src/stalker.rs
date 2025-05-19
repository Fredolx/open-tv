use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{stalker_type, types::Source};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerJsData<T> {
    js: StalkerData<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerData<T> {
    data: Vec<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerLiveItem {
    name: String,
    cmd: String,
    tv_genre_id: String,
    logo: String,
    enable_tv_archive: u8,
    tv_archive_duration: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerVod {
    name: String,
    category_id: String,
    screenshot_uri: String,
    cmd: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StalkerSeries {
    id: String,
    name: String,
    screenshot_uri: String,
    category_id: String,
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

async fn get_stalker(source: Source, wipe: bool) -> Result<()> {
    let url = build_base_url(source)?;
    let token = handshake(url.clone()).await?;
    let live_cats = get_genres(url.clone()).await?;
    let vod_cats = get_categories(url.clone(), stalker_type::VOD).await?;
    Ok(())
}

fn build_stalker_client() -> Result<Client> {
    let client = Client::builder().user_agent("Mozilla/5.0 (QtEmbedded; U; Linux; C) AppleWebKit/533.3 (KHTML, like Gecko) MAG200 stbapp ver: 2 rev: 250 Safari/533.3
  X-User-Agent: Model: MAG250; Link: WiFi").build()?;
    Ok(client)
}

fn build_base_url(source: Source) -> Result<Url> {
    let mut url = Url::from_str(source.url.as_ref().context("no url in source.url")?)?;
    url.query_pairs_mut().append_pair(
        "mac",
        source
            .username
            .as_ref()
            .context("no mac address in source.username")?,
    );
    Ok(url)
}

fn add_token_to_url(url: &mut Url, token: &str) {
    url.query_pairs_mut().append_pair("token", token);
}

async fn handshake(mut url: Url) -> Result<StalkerJs<HandshakeData>> {
    let client = build_stalker_client()?;
    url.query_pairs_mut()
        .append_pair("type", "stb")
        .append_pair("action", "handshake");
    Ok(client.get(url).send().await?.json().await?)
}

async fn get_all_channels<T>(mut url: Url, content_type: &str) -> Result<()>
where
    T: serde::de::DeserializeOwned,
{
    let mut page: u32 = 0;
    let mut result: StalkerJsData<T> = get_ordered_list(url.clone(), content_type, page).await?;
    while result.js.data.len() != 0 {
        page += 1;
        result = get_ordered_list(url.clone(), content_type, page).await?;
    }
    Ok(())
}

async fn get_ordered_list<T>(mut url: Url, content_type: &str, page: u32) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let client = build_stalker_client()?;
    url.query_pairs_mut()
        .append_pair("type", content_type)
        .append_pair("p", &page.to_string())
        .append_pair("action", "get_ordered_list");
    Ok(client.get(url).send().await?.json::<T>().await?)
}

async fn get_categories(mut url: Url, content_type: &str) -> Result<HashMap<String, String>> {
    let client = build_stalker_client()?;
    if content_type == stalker_type::LIVE {
        bail!("get_categories does not support LIVE (itv)")
    }
    url.query_pairs_mut()
        .append_pair("type", content_type)
        .append_pair("action", "get_categories");
    let result = client
        .get(url)
        .send()
        .await?
        .json::<StalkerJs<Vec<StalkerGenre>>>()
        .await?;
    Ok(result.js.into_iter().map(|f| (f.id, f.title)).collect())
}

async fn get_genres(mut url: Url) -> Result<HashMap<String, String>> {
    let client = build_stalker_client()?;
    url.query_pairs_mut()
        .append_pair("type", stalker_type::LIVE)
        .append_pair("action", "get_genres");
    let result = client
        .get(url)
        .send()
        .await?
        .json::<StalkerJs<Vec<StalkerGenre>>>()
        .await?;
    Ok(result.js.into_iter().map(|f| (f.id, f.title)).collect())
}

async fn get_episodes(
    client: Client,
    mut url: Url,
    movie_id: &str,
) -> Result<StalkerSeriesEpisode> {
    url.query_pairs_mut()
        .append_pair("type", "series")
        .append_pair("action", "get_ordered_list")
        .append_pair("movie_id", movie_id);
    Ok(client.get(url).send().await?.json().await?)
}
