use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::JoinHandle,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Channel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub name: String,
    pub url: Option<String>,
    pub group: Option<String>,
    pub image: Option<String>,
    pub media_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<i64>,
    pub favorite: bool,
    pub stream_id: Option<u64>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Source {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub source_type: u8,
    pub use_tvg_id: Option<bool>,
    pub enabled: bool,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub recording_path: Option<String>,
    pub mpv_params: Option<String>,
    pub use_stream_caching: Option<bool>,
    pub default_view: Option<u8>,
    pub volume: Option<u8>,
    pub refresh_on_start: Option<bool>,
    pub restream_port: Option<u16>,
    pub zoom: Option<f32>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Filters {
    pub query: Option<String>,
    pub source_ids: Vec<i64>,
    pub media_types: Option<Vec<u8>>,
    pub view_type: u8,
    pub page: u8,
    pub series_id: Option<i64>,
    pub group_id: Option<i64>,
    pub use_keywords: bool,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct ChannelHttpHeaders {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<i64>,
    pub referrer: Option<String>,
    pub user_agent: Option<String>,
    pub http_origin: Option<String>,
    pub ignore_ssl: Option<bool>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct CustomChannel {
    pub data: Channel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<ChannelHttpHeaders>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Group {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub name: String,
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<i64>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct IdName {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct CustomChannelExtraData {
    pub headers: Option<ChannelHttpHeaders>,
    pub group: Option<Group>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ExportedGroup {
    pub group: Group,
    pub channels: Vec<CustomChannel>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ExportedSource {
    pub source: Source,
    pub groups: Vec<ExportedGroup>,
    pub channels: Vec<CustomChannel>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct EPG {
    pub epg_id: String,
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub start_timestamp: i64,
    pub end_time: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct EPGNotify {
    pub epg_id: String,
    pub title: String,
    pub start_timestamp: i64,
    pub channel_name: String,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub notify_stop: Arc<AtomicBool>,
    pub thread_handle: Option<JoinHandle<Result<(), anyhow::Error>>>,
    pub restream_stop_signal: Arc<AtomicBool>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct NetworkInfo {
    pub port: u16,
    pub local_ips: Vec<String>,
    pub wan_ip: String,
}
