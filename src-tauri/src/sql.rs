use std::{collections::HashMap, sync::LazyLock};

use crate::types::{Channel, Filters, MediaType, Source};
use anyhow::{anyhow, bail, Context, Result};
use directories::ProjectDirs;
use num_enum::{FromPrimitive, TryFromPrimitive};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, MappedRows, OptionalExtension, Row, Transaction};

const PAGE_SIZE: u8 = 36;
static CONN: LazyLock<Pool<SqliteConnectionManager>> = LazyLock::new(|| create_connection_pool());

pub fn get_conn() -> Result<PooledConnection<SqliteConnectionManager>> {
    CONN.try_get().context("No sqlite conns available")
}

fn create_connection_pool() -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file(get_and_create_sqlite_db_path());
    r2d2::Pool::builder().max_size(20).build(manager).unwrap()
}

fn get_and_create_sqlite_db_path() -> String {
    let mut path = ProjectDirs::from("dev", "fredol", "open-tv")
        .unwrap()
        .data_dir()
        .to_owned();
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    path.push("db.sqlite");
    return path.to_string_lossy().to_string();
}

//@TODO: Nullable types
fn create_structure() -> Result<()> {
    let sql = get_conn()?;
    sql.execute_batch(
        r#"
CREATE TABLE "sources" (
  "id"          INTEGER PRIMARY KEY AUTOINCREMENT,
  "name"        varchar(100),
  "source_type" integer,
  "url"         varchar(500),
  "username"    varchar(100),
  "password"    varchar(100)
);

CREATE TABLE "channels" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" varchar(100),
  "group_name" varchar(100),
  "image" varchar(500),
  "url" varchar(500),
  "media_type" integer,
  "source_id" integer,
  "favorite" integer,
  FOREIGN KEY (source_id) REFERENCES sources(id)
);

CREATE TABLE "settings" (
  "key" VARCHAR(50) PRIMARY KEY,
  "value" VARCHAR(100)
);

CREATE INDEX index_channel_name
ON channels(name);

CREATE INDEX index_channel_group ON channels(group_name);

CREATE UNIQUE INDEX index_source_name ON sources(name);
"#,
    )?;
    Ok(())
}

fn structure_exists() -> Result<bool> {
    let sql = get_conn()?;
    let table_exists: bool = sql
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'channels' LIMIT 1",
            [],
            |row| row.get::<_, i32>(0),
        )
        .optional()?
        .is_some();
    Ok(table_exists)
}

pub fn create_or_initialize_db() -> Result<()> {
    if !structure_exists()? {
        create_structure()?;
    }
    Ok(())
}

pub fn drop_db() -> Result<()> {
    let sql = get_conn()?;
    sql.execute_batch("DROP TABLE channels; DROP TABLE sources; DROP TABLE settings;")?;
    Ok(())
}

pub fn create_or_find_source_by_name(source: &mut Source) -> Result<()> {
    let sql = get_conn()?;
    let id: Option<i64> = sql
        .query_row(
            "SELECT id FROM sources WHERE name = ?1",
            params![source.name],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(id) = id {
        source.id = Some(id);
        return Ok(());
    }
    sql.execute(
        "INSERT INTO sources (name, source_type, url, username, password) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![source.name, source.source_type.clone() as u8, source.url, source.username, source.password],
    )?;
    source.id = Some(sql.last_insert_rowid());
    Ok(())
}

pub fn insert_channel(tx: &Transaction, channel: Channel) -> Result<()> {
    tx.execute(
        r#"
INSERT INTO channels (name, group_name, image, url, source_id, media_type) 
VALUES (?1, ?2, ?3, ?4, ?5, ?6); 
"#,
        params![
            channel.name,
            channel.group,
            channel.image,
            channel.url,
            channel.source_id,
            channel.media_type as u8
        ],
    )?;
    Ok(())
}

pub fn get_settings() -> Result<HashMap<String, String>> {
    let sql = get_conn()?;
    let map = sql
        .prepare("SELECT key, value FROM Settings")?
        .query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })?
        .filter_map(Result::ok)
        .collect();
    Ok(map)
}

pub fn update_settings(map: HashMap<String, String>) -> Result<()> {
    let mut sql: PooledConnection<SqliteConnectionManager> = get_conn()?;
    let tx = sql.transaction()?;
    for (key, value) in map {
        tx.execute(
            r#"
            INSERT INTO Settings (key, value)
            VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = ?2
            "#,
            params![key, value],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn search(filters: Filters) -> Result<Vec<Channel>> {
    let sql = get_conn()?;
    let offset = filters.page * PAGE_SIZE - PAGE_SIZE;
    let channels: Vec<Channel> = sql
        .prepare(
            r#"
        SELECT * FROM CHANNELS
        WHERE name like '%?1%'
        AND media_type = ?2
		AND source_id in (?3)
        AND url IS NOT NULL
		LIMIT ?4, ?5
    "#,
        )?
        .query_map(
            params![
                filters.query,
                (filters.media_type as u8),
                filters.source_ids.join(","),
                offset,
                PAGE_SIZE,
            ],
            row_to_channel,
        )?
        .filter_map(Result::ok)
        .collect();
    Ok(channels)
}

pub fn search_group(filters: Filters) -> Result<Vec<Channel>> {
    let sql = get_conn()?;
    let offset = filters.page * PAGE_SIZE - PAGE_SIZE;
    let channels: Vec<Channel> = sql
        .prepare(
            r#"
        SELECT id, group_name, image, source_id
        FROM (
            SELECT *,
                ROW_NUMBER() OVER (PARTITION BY group_name ORDER BY id) AS row_num
            FROM channels
        ) ranked_channels
        WHERE row_num = 1
        AND group_name like '%?1%'
        AND media_type = ?2
        AND source_id in (?3)
        LIMIT ?4, ?5
    "#,
        )?
        .query_map(
            params![
                filters.query,
                (filters.media_type as u8),
                filters.source_ids.join(","),
                offset,
                PAGE_SIZE,
            ],
            row_to_group,
        )?
        .filter_map(Result::ok)
        .collect();
    Ok(channels)
}

fn row_to_group(row: &Row) -> std::result::Result<Channel, rusqlite::Error> {
    let channel = Channel {
        id: row.get("id")?,
        name: row.get("group_name")?,
        group: None,
        image: row.get("image")?,
        media_type: MediaType::Group,
        source_id: row.get("source_id")?,
        url: None
    };
    Ok(channel)
}

fn get_media_type(row: &Row) -> std::result::Result<MediaType, rusqlite::Error> {
    MediaType::try_from(row.get::<&str, u8>("media_type")?).map_err(|_| {
        rusqlite::Error::InvalidColumnType(
            6,
            "Could not convert media_type to enum".to_string(),
            rusqlite::types::Type::Integer,
        )
    })
}

fn row_to_channel(row: &Row) -> std::result::Result<Channel, rusqlite::Error> {
    let channel = Channel {
        id: row.get("id")?,
        name: row.get("name")?,
        group: row.get("group")?,
        image: row.get("image")?,
        media_type: get_media_type(row)?,
        source_id: row.get("source_id")?,
        url: row.get("url")?,
    };
    Ok(channel)
}

pub fn delete_channels_by_source(source_id: i64) -> Result<()> {
    let sql = get_conn()?;
    sql.execute(
        r#"
        DELETE FROM channels
        WHERE source_id = ?1;
    "#,
        params![source_id.to_string()],
    )?;
    Ok(())
}

pub fn favorite_channel(channel_id: i64, favorite: bool) -> Result<()> {
    let sql = get_conn()?;
    sql.execute(r#"
        UPDATE channels
        SET favorite = ?1
        WHERE id = ?2
    "#, params![favorite, channel_id])?;
    Ok(())
}

#[cfg(test)]
mod test_sql {
    use std::collections::HashMap;

    use crate::{
        settings::{RECORDING_PATH, USE_STREAM_CACHING},
        sql::{create_structure, drop_db, structure_exists},
    };

    use super::update_settings;

    #[test]
    fn test_structure_exists() {
        drop_db().unwrap();
        assert_eq!(structure_exists().unwrap(), false);
        create_structure().unwrap();
        assert_eq!(structure_exists().unwrap(), true);
    }
    #[test]
    fn test_update_settings() {
        drop_db().unwrap_or_default();
        create_structure().unwrap();
        let mut map: HashMap<String, String> = HashMap::with_capacity(3);
        map.insert(USE_STREAM_CACHING.to_string(), true.to_string());
        map.insert(RECORDING_PATH.to_string(), "somePath".to_string());
        update_settings(map.clone()).unwrap();
        update_settings(map).unwrap();
    }
}
