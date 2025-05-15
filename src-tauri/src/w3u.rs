use anyhow::anyhow;
use anyhow::{Context, Result};
use rusqlite::Transaction;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::sql;
use crate::types::{Channel, ChannelHttpHeaders, ChannelPreserve};
use crate::utils::get_media_type;
use crate::{
    source_type,
    types::Source,
    utils::{self, get_tmp_path},
};

const W3U_TEMP_FILENAME: &str = "get.w3u";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct W3uData {
    groups: Vec<W3UGroup>,
    stations: Vec<W3UStream>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct W3UGroup {
    name: String,
    image: Option<String>,
    stations: Vec<W3UStream>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct W3UStream {
    name: String,
    image: Option<String>,
    url: String,
    referer: Option<String>,
    user_agent: Option<String>,
}

pub async fn get_w3u_from_link(source: Source, wipe: bool) -> Result<()> {
    utils::get_file_http(
        source.url.as_ref().context("no url")?,
        get_tmp_path(W3U_TEMP_FILENAME).await?,
    )
    .await?;
    read_w3u(source, wipe).await
}

pub async fn read_w3u(source: Source, wipe: bool) -> Result<()> {
    let path = match source.source_type {
        source_type::W3U => source.url.clone().context("no url")?,
        source_type::W3U_LINK => get_tmp_path(W3U_TEMP_FILENAME).await?,
        _ => return Err(anyhow!("invalid source_type")),
    };
    let data = fs::read(path).await?;
    let data: W3uData = serde_json::from_slice(&data)?;
    let mut channel_preserve: Vec<ChannelPreserve> = Vec::new();
    sql::do_tx(|tx| {
        let mut source_id = source.id.context("no source id")?;
        if wipe {
            channel_preserve = sql::get_channel_preserve(&tx, source_id).unwrap_or(Vec::new());
            sql::wipe(&tx, source_id)?;
        } else {
            source_id = sql::create_or_find_source_by_name(&tx, &source)?;
        }
        for group in data.groups {
            process_group(group, source_id, tx)?;
        }
        for station in data.stations {
            process_station(station, source_id, None, tx)?;
        }
        if wipe {
            sql::restore_preserve(&tx, source_id, channel_preserve)?;
        }
        Ok(())
    })?;
    Ok(())
}

fn process_group(group: W3UGroup, source_id: i64, tx: &Transaction) -> Result<()> {
    let group_id = sql::get_or_insert_group(tx, &group.name, &group.image, &source_id)?;
    for station in group.stations {
        process_station(station, source_id, Some(group_id), tx)?;
    }
    Ok(())
}

fn process_station(
    station: W3UStream,
    source_id: i64,
    group_id: Option<i64>,
    tx: &Transaction,
) -> Result<()> {
    let channel = Channel {
        name: station.name,
        group_id,
        image: station.image,
        media_type: get_media_type(&station.url),
        favorite: false,
        group: None,
        series_id: None,
        stream_id: None,
        source_id: Some(source_id),
        tv_archive: None,
        id: None,
        url: Some(station.url),
    };
    sql::insert_channel(tx, channel)?;
    if station.referer.is_some() || station.user_agent.is_some() {
        let headers = ChannelHttpHeaders {
            channel_id: Some(tx.last_insert_rowid()),
            referrer: station.referer,
            user_agent: station.user_agent,
            ..Default::default()
        };
        sql::insert_channel_headers(tx, headers)?
    }
    Ok(())
}
