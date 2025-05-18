use crate::types::Source;

struct StalkerLive {
    js: StalkerLiveData,
}

struct StalkerLiveData {
    data: Vec<StalkerLiveItem>,
}

struct StalkerLiveItem {
    name: String,
    cmd: String,
    tv_genre_id: String,
    logo: String,
    enable_tv_archive: u8,
    tv_archive_duration: u32,
}

struct StalkerVod {
    js: StalkerVodData,
}

struct StalkerVodData {
    data: Vec<StalkerVodChannel>,
}

struct StalkerVodChannel {
    name: String,
    category_id: String,
    screenshot_uri: String,
    cmd: String,
}

struct StalkerSeries {
    js: StalkerSeriesData,
}

struct StalkerSeriesData {
    data: Vec<StalkerSeriesItem>,
}

struct StalkerSeriesItem {
    id: String,
    name: String,
    screenshot_uri: String,
    category_id: String,
}

struct StalkerSeriesEpisodes {
    js: StalkerSeriesEpisodesData,
}

struct StalkerSeriesEpisodesData {
    data: Vec<StalkerSeriesEpisodesItem>,
}

struct StalkerSeriesEpisodesItem {
    name: String,
    series: Vec<u32>,
    screenshot_uri: String,
    cmd: String,
}

fn get_stalker(source: Source, wipe: bool) {}
