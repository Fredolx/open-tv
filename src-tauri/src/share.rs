use anyhow::Context;
use anyhow::Result;
use directories::UserDirs;
use serde::Serialize;
use crate::types::CustomChannel;
use crate::types::ExportedGroup;
use crate::types::Group;
use crate::{sql, types::Channel};

const CHANNEL_SHARE_EXTENSION: &str = ".otv";
const PLAYLIST_SHARE_EXTENSION: &str = ".otvp";
const GROUP_SHARE_EXTENSION: &str = ".ovtg";

pub fn share_custom_channel(channel: Channel) -> Result<()> {
    let channel = get_custom_channel(channel)?;
    let path = get_download_path(channel.data.id.context("No id on channel?")?, 
        CHANNEL_SHARE_EXTENSION)?;
    serialize_to_file(channel, path)
}

fn get_download_path(id: i64, extension: &str) -> Result<String> {
    let path = UserDirs::new().context("No user dirs?")?;
    let path = path.download_dir().context("No downloads folder")?;
    let mut path = path.to_path_buf();
    path.push(format!("{id}{extension}"));
    Ok(path.to_string_lossy().to_string())
}

fn get_custom_channel(channel: Channel) -> Result<CustomChannel> {
    Ok(CustomChannel {
        headers: sql::get_channel_headers_by_id(channel.id.context("No id on channel?")?)?,
        data: channel,
    })
}

fn serialize_to_file<T: Serialize>(obj: T, path: String) -> Result<()> {
    let data = serde_json::to_string(&obj)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn share_custom_group(group: Channel) -> Result<()> {
    let to_export = ExportedGroup {
        group: Group {
            id: group.id,
            image: group.image,
            name: group.name,
            source_id: None
        },
        channels: sql::get_custom_channels_by_group(group.id.context("No group id")?)?
    };
    let path = get_download_path(
        group.id.context("No group id?")?, 
        GROUP_SHARE_EXTENSION
    )?;
    serialize_to_file(to_export, path)?;
    Ok(())
}