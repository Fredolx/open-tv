use std::{collections::HashMap, sync::LazyLock};

use crate::types::{Channel, Source};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension, Transaction};

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
            params![key, value]
        )?;
    }
    tx.commit()?;
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
