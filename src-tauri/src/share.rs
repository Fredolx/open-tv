use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use directories::UserDirs;
use serde::Serialize;
use crate::types::CustomChannel;
use crate::types::ExportedGroup;
use crate::types::ExportedSource;
use crate::types::Group;
use crate::types::Source;
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
        channels: sql::get_custom_channels(group.id, group.source_id.context("no source id?")?)?
    };
    let path = get_download_path(
        group.id.context("No group id?")?, 
        GROUP_SHARE_EXTENSION
    )?;
    serialize_to_file(to_export, path)?;
    Ok(())
}

pub fn share_custom_source(mut source: Source) -> Result<()> {
    let id = source.id.context("No source id?")?.clone();
    source.id = None;
    let to_export = ExportedSource {
        source: source,
        groups: sql::get_custom_groups(id)?,
        channels: sql::get_custom_channels(None, id)?
    };
    let path = get_download_path(
        id, 
        PLAYLIST_SHARE_EXTENSION
    )?;
    serialize_to_file(to_export, path)?;
    Ok(())
}

pub fn import(path: String, source_id: Option<i64>, name_override: Option<String>) -> Result<()> {
    let data = std::fs::read_to_string(&path)?;
    match path.split(".").last().context("Invalid path, no extension")? {
        "otv" => import_channel(data, source_id.context("No source id")?, name_override),
        "otvg" => import_group(data, source_id.context("No source id")?, name_override),
        "otvp" => import_playlist(data, name_override),
        _ => Err(anyhow::anyhow!("Invalid path"))
    }
}

fn import_channel(data: String, source_id: i64, name_override: Option<String>) -> Result<()> {
    let mut data: CustomChannel = serde_json::from_str(&data)?;
    if let Some(name) = name_override {
        data.data.name = name;
    }
    if sql::channel_exists(&data.data.name, data.data.url.as_ref().context("No channel url")?, source_id)? {
        bail!("Duplicate exists");
    }
    data.data.source_id = Some(source_id);
    sql::do_tx(|tx| sql::add_custom_channel(tx, data))?;
    Ok(())
}

fn import_group(data: String, source_id: i64, name_override: Option<String>) -> Result<()> {
    let mut data: ExportedGroup = serde_json::from_str(&data)?;
    if let Some(name) = name_override {
        data.group.name = name;
    }
    if sql::group_exists(&data.group.name, source_id)? {
        bail!("Duplicate exists");
    }
    sql::do_tx(|tx| {
        data.group.source_id = Some(source_id);
        let group_id = sql::add_custom_group(&tx, data.group)?;
        for mut channel in data.channels {
            channel.data.group_id = Some(group_id);
            channel.data.source_id = Some(source_id);
            sql::add_custom_channel(&tx, channel)?;
        }
        Ok(())
    })?;
    Ok(())
}

fn import_playlist(data: String, name_override: Option<String>) -> Result<()> {
    let mut data: ExportedSource = serde_json::from_str(&data)?;
    if let Some(name) = name_override {
        data.source.name = name;
    }
    if sql::source_name_exists(&data.source.name)? {
        bail!("Duplicate exists");
    }
    sql::do_tx(|tx| {
        let source_id = sql::create_or_find_source_by_name(tx, &data.source)?;
        for mut group in data.groups {
            group.group.source_id = Some(source_id);
            let group_id = sql::add_custom_group(&tx, group.group)?;
            for mut channel in group.channels {
                channel.data.group_id = Some(group_id);
                channel.data.source_id = Some(source_id);
                sql::add_custom_channel(tx, channel)?
            }
        }
        Ok(())
    })?;
    Ok(())
}

