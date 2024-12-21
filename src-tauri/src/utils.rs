use crate::{m3u, source_type, sql, types::Source, xtream};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local, Utc};

pub async fn refresh_source(source: Source) -> Result<()> {
    match source.source_type {
        source_type::M3U => m3u::read_m3u8(source, true)?,
        source_type::M3U_LINK => m3u::get_m3u8_from_link(source, true).await?,
        source_type::XTREAM => xtream::get_xtream(source, true).await?,
        source_type::CUSTOM => {}
        _ => return Err(anyhow!("invalid source_type")),
    }
    Ok(())
}

pub async fn refresh_all() -> Result<()> {
    let sources = sql::get_sources()?;
    for source in sources {
        refresh_source(source).await?;
    }
    Ok(())
}

pub fn get_local_time(timestamp: &str) -> Result<DateTime<Local>> {
    let timestamp: i64 = timestamp.parse()?;
    let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0).context("no time")?;
    Ok(DateTime::<Local>::from(datetime))
}
