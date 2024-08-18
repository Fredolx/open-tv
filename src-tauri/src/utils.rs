use crate::{m3u, sql, types::{Source, SourceType}, xtream};
use anyhow::{Context, Result};

pub async fn refresh_source(source: Source) -> Result<()> {
    sql::delete_channels_by_source(source.id.context("no ID")?)?;
    match source.source_type {
        SourceType::M3U => m3u::read_m3u8(source)?,
        SourceType::M3ULink => m3u::get_m3u8_from_link(source).await?,
        SourceType::Xtream => xtream::get_xtream(source).await?
    }
    Ok(())
}