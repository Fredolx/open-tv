#[derive(Clone, PartialEq, Debug)]
pub enum MediaType {
    Livestream,
    Movie,
    Serie,
    Group,
}
#[derive(Clone, PartialEq, Debug)]
pub enum SourceType {
    M3U,
    M3ULink,
    Xtream
}
pub struct Channel {
    pub name: String,
    pub url: String,
    pub group: Option<String>,
    pub image: Option<String>,
    pub media_type: MediaType,
    pub source_id: i64
}

pub struct Source {
    pub id: Option<i64>,
    pub name: String,
    pub url: Option<String>,
    pub url_origin: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub source_type: SourceType
}