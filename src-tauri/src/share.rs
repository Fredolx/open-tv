use anyhow::Context;
use anyhow::Result;
use directories::UserDirs;
use crate::types::CustomChannel;
use crate::{sql, types::Channel};

pub fn share_custom_channel(channel: Channel) -> Result<()> {
    let channel = get_custom_channel(channel)?;
    let path = get_download_path(channel.data.id.context("No id on channel?")?)?;
    serialize_to_file(channel, path)
}

fn get_download_path(id: i64) -> Result<String> {
    let path = UserDirs::new().context("No user dirs?")?;
    let path = path.download_dir().context("No downloads folder")?;
    let mut path = path.to_path_buf();
    path.push(format!("{id}.json"));
    Ok(path.to_string_lossy().to_string())
}

fn get_custom_channel(channel: Channel) -> Result<CustomChannel> {
    Ok(CustomChannel {
        headers: sql::get_channel_headers_by_id(channel.id.context("No id on channel?")?)?,
        data: channel,
    })
}

fn serialize_to_file(channel: CustomChannel, path: String) -> Result<()> {
    let data = serde_json::to_string(&channel)?;
    std::fs::write(path, data)?;
    Ok(())
}