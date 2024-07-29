pub enum MediaType {
    Livestream,
    Movie,
    Serie,
    Group,
}
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