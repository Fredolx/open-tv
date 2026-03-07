use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result, bail};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::sql;
use crate::types::{
    Channel, ChannelHttpHeaders, Group, Season, Source, Settings,
};
use crate::settings;

const BACKUP_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct Backup {
    pub version: u32,
    pub created_at: String,
    pub settings: Settings,
    pub sources: Vec<Source>,
    pub groups: Vec<Group>,
    pub channels: Vec<Channel>,
    pub channel_http_headers: Vec<ChannelHttpHeaders>,
    pub seasons: Vec<Season>,
    pub epg_watchlist: Vec<EpgEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EpgEntry {
    pub epg_id: String,
    pub channel_name: String,
    pub title: String,
    pub start_timestamp: i64,
}

pub fn export_backup(path: String) -> Result<()> {
    let conn = sql::get_conn()?;

    let app_settings = settings::get_settings()?;

    let sources: Vec<Source> = conn
        .prepare("SELECT * FROM sources")?
        .query_map([], |row| {
            Ok(Source {
                id: row.get("id")?,
                name: row.get("name")?,
                username: row.get("username")?,
                password: row.get("password")?,
                url: row.get("url")?,
                source_type: row.get("source_type")?,
                url_origin: None,
                enabled: row.get("enabled")?,
                use_tvg_id: row.get("use_tvg_id")?,
                user_agent: row.get("user_agent")?,
                max_streams: row.get("max_streams")?,
                stream_user_agent: row.get("stream_user_agent")?,
                last_updated: row.get("last_updated")?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let groups: Vec<Group> = conn
        .prepare("SELECT * FROM groups")?
        .query_map([], |row| {
            Ok(Group {
                id: row.get("id")?,
                name: row.get("name")?,
                image: row.get("image")?,
                source_id: row.get("source_id")?,
                hidden: row.get("hidden")?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let channels: Vec<Channel> = conn
        .prepare("SELECT * FROM channels")?
        .query_map([], |row| {
            Ok(Channel {
                id: row.get("id")?,
                name: row.get("name")?,
                url: row.get("url")?,
                group: None,
                image: row.get("image")?,
                media_type: row.get("media_type")?,
                source_id: row.get("source_id")?,
                series_id: row.get("series_id")?,
                group_id: row.get("group_id")?,
                favorite: row.get("favorite")?,
                stream_id: row.get("stream_id")?,
                tv_archive: row.get("tv_archive")?,
                season_id: row.get("season_id")?,
                episode_num: row.get("episode_num")?,
                hidden: row.get("hidden")?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let headers: Vec<ChannelHttpHeaders> = conn
        .prepare("SELECT * FROM channel_http_headers")?
        .query_map([], |row| {
            Ok(ChannelHttpHeaders {
                id: row.get("id")?,
                channel_id: row.get("channel_id")?,
                referrer: row.get("referrer")?,
                user_agent: row.get("user_agent")?,
                http_origin: row.get("http_origin")?,
                ignore_ssl: row.get("ignore_ssl")?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let seasons: Vec<Season> = conn
        .prepare("SELECT * FROM seasons")?
        .query_map([], |row| {
            Ok(Season {
                id: row.get("id")?,
                name: row.get("name")?,
                season_number: row.get("season_number")?,
                series_id: row.get("series_id")?,
                source_id: row.get("source_id")?,
                image: row.get("image")?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let epg_watchlist: Vec<EpgEntry> = conn
        .prepare("SELECT * FROM epg")?
        .query_map([], |row| {
            Ok(EpgEntry {
                epg_id: row.get("epg_id")?,
                channel_name: row.get("channel_name")?,
                title: row.get("title")?,
                start_timestamp: row.get("start_timestamp")?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let backup = Backup {
        version: BACKUP_VERSION,
        created_at: chrono::Local::now().to_rfc3339(),
        settings: app_settings,
        sources,
        groups,
        channels,
        channel_http_headers: headers,
        seasons,
        epg_watchlist,
    };

    let json = serde_json::to_string_pretty(&backup)?;
    fs::write(&path, json)?;
    Ok(())
}

pub fn import_backup(path: String, mode: String) -> Result<()> {
    let data = fs::read_to_string(&path)?;
    let backup: Backup = serde_json::from_str(&data)
        .context("Invalid backup file format")?;

    if backup.version > BACKUP_VERSION {
        bail!("Backup version {} is newer than supported version {}", backup.version, BACKUP_VERSION);
    }

    match mode.as_str() {
        "replace" => import_replace(backup),
        "merge" => import_merge(backup),
        _ => bail!("Unknown import mode: {}", mode),
    }
}

fn import_replace(backup: Backup) -> Result<()> {
    let mut conn = sql::get_conn()?;
    let tx = conn.transaction()?;

    // Wipe all data tables (order matters for FK constraints)
    tx.execute_batch(
        r#"
        DELETE FROM channel_http_headers;
        DELETE FROM epg;
        DELETE FROM seasons;
        DELETE FROM channels;
        DELETE FROM groups;
        DELETE FROM sources;
        DELETE FROM settings;
        "#
    )?;

    // Restore sources (with original IDs so FK refs work)
    for source in &backup.sources {
        tx.execute(
            r#"INSERT INTO sources (id, name, source_type, url, username, password, enabled, use_tvg_id, user_agent, max_streams, stream_user_agent, last_updated)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
            params![
                source.id, source.name, source.source_type, source.url,
                source.username, source.password, source.enabled,
                source.use_tvg_id, source.user_agent, source.max_streams,
                source.stream_user_agent, source.last_updated
            ],
        )?;
    }

    // Restore groups
    for group in &backup.groups {
        tx.execute(
            r#"INSERT INTO groups (id, name, image, source_id, hidden)
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
            params![group.id, group.name, group.image, group.source_id, group.hidden],
        )?;
    }

    // Restore seasons
    for season in &backup.seasons {
        tx.execute(
            r#"INSERT INTO seasons (id, name, season_number, series_id, source_id, image)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            params![season.id, season.name, season.season_number, season.series_id, season.source_id, season.image],
        )?;
    }

    // Restore channels
    for ch in &backup.channels {
        tx.execute(
            r#"INSERT INTO channels (id, name, image, url, media_type, source_id, favorite, series_id, group_id, stream_id, tv_archive, last_watched, season_id, episode_num, hidden)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, NULL, ?12, ?13, ?14)"#,
            params![
                ch.id, ch.name, ch.image, ch.url, ch.media_type, ch.source_id,
                ch.favorite, ch.series_id, ch.group_id, ch.stream_id,
                ch.tv_archive, ch.season_id, ch.episode_num, ch.hidden
            ],
        )?;
    }

    // Restore channel headers
    for h in &backup.channel_http_headers {
        tx.execute(
            r#"INSERT INTO channel_http_headers (id, channel_id, referrer, user_agent, http_origin, ignore_ssl)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            params![h.id, h.channel_id, h.referrer, h.user_agent, h.http_origin, h.ignore_ssl],
        )?;
    }

    // Restore EPG watchlist
    for e in &backup.epg_watchlist {
        tx.execute(
            r#"INSERT INTO epg (epg_id, channel_name, title, start_timestamp)
               VALUES (?1, ?2, ?3, ?4)"#,
            params![e.epg_id, e.channel_name, e.title, e.start_timestamp],
        )?;
    }

    tx.commit()?;

    // Restore settings separately (uses the existing settings update path)
    settings::update_settings(backup.settings)?;

    Ok(())
}

fn import_merge(backup: Backup) -> Result<()> {
    let mut conn = sql::get_conn()?;
    let tx = conn.transaction()?;

    // Build a mapping from backup source IDs to existing/new source IDs
    let mut source_id_map: HashMap<i64, i64> = HashMap::new();

    for source in &backup.sources {
        let backup_id = source.id.unwrap_or(0);
        // Check if source with same name + type exists
        let existing: Option<i64> = tx.query_row(
            "SELECT id FROM sources WHERE name = ?1 AND source_type = ?2",
            params![source.name, source.source_type],
            |row| row.get(0),
        ).ok();

        if let Some(existing_id) = existing {
            source_id_map.insert(backup_id, existing_id);
        } else {
            tx.execute(
                r#"INSERT INTO sources (name, source_type, url, username, password, enabled, use_tvg_id, user_agent, max_streams, stream_user_agent, last_updated)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
                params![
                    source.name, source.source_type, source.url,
                    source.username, source.password, source.enabled,
                    source.use_tvg_id, source.user_agent, source.max_streams,
                    source.stream_user_agent, source.last_updated
                ],
            )?;
            let new_id = tx.last_insert_rowid();
            source_id_map.insert(backup_id, new_id);
        }
    }

    // Build group ID mapping
    let mut group_id_map: HashMap<i64, i64> = HashMap::new();

    for group in &backup.groups {
        let backup_id = group.id.unwrap_or(0);
        let mapped_source_id = group.source_id.and_then(|sid| source_id_map.get(&sid).copied());

        let existing: Option<i64> = if let Some(sid) = mapped_source_id {
            tx.query_row(
                "SELECT id FROM groups WHERE name = ?1 AND source_id = ?2",
                params![group.name, sid],
                |row| row.get(0),
            ).ok()
        } else {
            None
        };

        if let Some(existing_id) = existing {
            group_id_map.insert(backup_id, existing_id);
        } else {
            tx.execute(
                r#"INSERT INTO groups (name, image, source_id, hidden)
                   VALUES (?1, ?2, ?3, ?4)"#,
                params![group.name, group.image, mapped_source_id, group.hidden],
            )?;
            let new_id = tx.last_insert_rowid();
            group_id_map.insert(backup_id, new_id);
        }
    }

    // Import seasons (skip duplicates)
    let mut season_id_map: HashMap<i64, i64> = HashMap::new();
    for season in &backup.seasons {
        let backup_id = season.id.unwrap_or(0);
        let mapped_source_id = source_id_map.get(&season.source_id).copied().unwrap_or(season.source_id);

        let existing: Option<i64> = tx.query_row(
            "SELECT id FROM seasons WHERE season_number = ?1 AND series_id = ?2 AND source_id = ?3",
            params![season.season_number, season.series_id, mapped_source_id],
            |row| row.get(0),
        ).ok();

        if let Some(existing_id) = existing {
            season_id_map.insert(backup_id, existing_id);
        } else {
            tx.execute(
                r#"INSERT INTO seasons (name, season_number, series_id, source_id, image)
                   VALUES (?1, ?2, ?3, ?4, ?5)"#,
                params![season.name, season.season_number, season.series_id, mapped_source_id, season.image],
            )?;
            season_id_map.insert(backup_id, tx.last_insert_rowid());
        }
    }

    // Import channels (skip duplicates by name+source_id)
    for ch in &backup.channels {
        let mapped_source_id = ch.source_id.and_then(|sid| source_id_map.get(&sid).copied());
        let mapped_group_id = ch.group_id.and_then(|gid| group_id_map.get(&gid).copied());
        let mapped_season_id = ch.season_id.and_then(|sid| season_id_map.get(&sid).copied());

        if let Some(sid) = mapped_source_id {
            let exists: bool = tx.query_row(
                "SELECT 1 FROM channels WHERE name = ?1 AND source_id = ?2",
                params![ch.name, sid],
                |_| Ok(true),
            ).unwrap_or(false);

            if exists {
                continue;
            }
        }

        tx.execute(
            r#"INSERT INTO channels (name, image, url, media_type, source_id, favorite, series_id, group_id, stream_id, tv_archive, season_id, episode_num, hidden)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"#,
            params![
                ch.name, ch.image, ch.url, ch.media_type, mapped_source_id,
                ch.favorite, ch.series_id, mapped_group_id, ch.stream_id,
                ch.tv_archive, mapped_season_id, ch.episode_num, ch.hidden
            ],
        )?;
    }

    tx.commit()?;

    // Merge settings (update existing with backup values, keeping any not in backup)
    settings::update_settings(backup.settings)?;

    Ok(())
}
