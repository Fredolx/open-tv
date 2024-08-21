use crate::{
    m3u, source_type, sql, types::Source, xtream
};
use anyhow::{anyhow, Context, Result};

pub async fn refresh_source(source: Source) -> Result<()> {
    sql::delete_channels_by_source(source.id.context("no ID")?)?;
    match source.source_type {
        source_type::M3U => m3u::read_m3u8(source)?,
        source_type::M3U_LINK => m3u::get_m3u8_from_link(source).await?,
        source_type::XTREAM => xtream::get_xtream(source).await?,
        _ => return Err(anyhow!("invalid source_type"))
    }
    Ok(())
}
