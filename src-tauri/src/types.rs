use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive, Deserialize, Serialize)]
#[repr(u8)]
pub enum MediaType {
    Livestream,
    Movie,
    Serie,
    Group,
}
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum SourceType {
    M3U,
    M3ULink,
    Xtream,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Channel {
    pub name: String,
    pub url: String,
    pub group: Option<String>,
    pub image: Option<String>,
    pub media_type: MediaType,
    pub source_id: i64,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Source {
    pub id: Option<i64>,
    pub name: String,
    pub url: Option<String>,
    pub url_origin: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub source_type: SourceType,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub recording_path: Option<String>,
    pub mpv_params: Option<String>,
    pub use_stream_caching: Option<bool>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Filters {
    pub query: Option<String>,
    pub source_ids: Vec<String>,
    pub media_type: MediaType,
    pub page: u8,
}
