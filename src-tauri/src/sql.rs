use std::{collections::HashMap, sync::LazyLock};

use crate::{
    media_type,
    types::{Channel, Filters, Source},
    view_type,
};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, params_from_iter, OptionalExtension, Row, Transaction};

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
  "series_id" integer,
  FOREIGN KEY (source_id) REFERENCES sources(id)
  FOREIGN KEY (series_id) REFERENCES channels(id)
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

/// # Returns
/// If a new source was created
pub fn create_or_find_source_by_name(source: &mut Source) -> Result<bool> {
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
        return Ok(false);
    }
    sql.execute(
        "INSERT INTO sources (name, source_type, url, username, password) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![source.name, source.source_type.clone() as u8, source.url, source.username, source.password],
    )?;
    source.id = Some(sql.last_insert_rowid());
    Ok(true)
}

pub fn insert_channel(tx: &Transaction, channel: Channel) -> Result<()> {
    tx.execute(
        r#"
INSERT INTO channels (name, group_name, image, url, source_id, media_type, favorite) 
VALUES (?1, ?2, ?3, ?4, ?5, ?6, false); 
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
    if filters.view_type == view_type::CATEGORIES
        && filters.group_name.is_none()
        && filters.series_id.is_none()
    {
        return search_group(filters);
    }
    let sql = get_conn()?;
    let offset = filters.page * PAGE_SIZE - PAGE_SIZE;
    let mut sql_query = format!(
        r#"
        SELECT * FROM CHANNELS
        WHERE name LIKE ?
        AND media_type IN ({})
		AND source_id IN ({})
        AND url IS NOT NULL"#,
        generate_placeholders(
            filters
                .media_types
                .as_ref()
                .context("no media types")?
                .len()
        ),
        generate_placeholders(filters.source_ids.len()),
    );
    if filters.view_type == view_type::FAVORITES {
        sql_query += "\nAND favorite = 1";
    }
    if let Some(series_id) = filters.series_id {
        sql_query += &format!("\nAND series_id = {series_id}")
    } else if let Some(group_name) = filters.group_name {
        sql_query += &format!("\nAND group_name = {group_name}");
    }
    sql_query += "\nLIMIT ?, ?";

    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(
        3 + filters.media_types.as_ref().unwrap().len() + filters.source_ids.len(),
    );
    let query = to_sql_like(filters.query);
    let media_types = filters.media_types.unwrap();
    params.push(&query);
    params.extend(to_to_sql(&media_types));
    params.extend(to_to_sql(&filters.source_ids));
    params.push(&offset);
    params.push(&PAGE_SIZE);
    let channels: Vec<Channel> = sql
        .prepare(&sql_query)?
        .query_map(params_from_iter(params), row_to_channel)?
        .filter_map(Result::ok)
        .collect();
    Ok(channels)
}

fn to_to_sql<T: rusqlite::ToSql>(values: &[T]) -> Vec<&dyn rusqlite::ToSql> {
    values.iter().map(|x| x as &dyn rusqlite::ToSql).collect()
}

fn generate_placeholders(size: usize) -> String {
    std::iter::repeat("?")
        .take(size)
        .collect::<Vec<_>>()
        .join(",")
}

fn to_sql_like(query: Option<String>) -> String {
    query.map(|x| format!("%{x}%")).unwrap_or("%".to_string())
}

pub fn search_group(filters: Filters) -> Result<Vec<Channel>> {
    let sql = get_conn()?;
    let offset = filters.page * PAGE_SIZE - PAGE_SIZE;
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(3 + filters.source_ids.len());
    let sql_query = format!(
        r#"
        SELECT id, group_name, image, source_id
        FROM (
            SELECT *,
                ROW_NUMBER() OVER (PARTITION BY group_name ORDER BY id) AS row_num
            FROM channels
        ) ranked_channels
        WHERE row_num = 1
        AND group_name like ?
        AND source_id in ({})
        LIMIT ?, ?
    "#,
        generate_placeholders(filters.source_ids.len())
    );
    let query = to_sql_like(filters.query);
    params.push(&query);
    params.extend(to_to_sql(&filters.source_ids));
    params.push(&offset);
    params.push(&PAGE_SIZE);
    let channels: Vec<Channel> = sql
        .prepare(&sql_query)?
        .query_map(params_from_iter(params), row_to_group)?
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
        media_type: media_type::GROUP,
        source_id: row.get("source_id")?,
        url: None,
        series_id: None,
    };
    Ok(channel)
}

fn row_to_channel(row: &Row) -> std::result::Result<Channel, rusqlite::Error> {
    let channel = Channel {
        id: row.get("id")?,
        name: row.get("name")?,
        group: row.get("group_name")?,
        image: row.get("image")?,
        media_type: row.get("media_type")?,
        source_id: row.get("source_id")?,
        url: row.get("url")?,
        series_id: None,
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

pub fn delete_source(id: i64) -> Result<()> {
    let sql = get_conn()?;
    sql.execute(
        r#"
        DELETE FROM channels
        WHERE source_id = ?1;

        DELETE FROM sources
        WHERE id = ?1;
    "#,
        [id],
    )?;
    Ok(())
}

pub fn source_name_exists(name: String) -> Result<bool> {
    let sql = get_conn()?;
    Ok(sql
        .query_row(
            r#"
    SELECT 1 
    FROM sources 
    WHERE name = ?1
    "#,
            [name],
            |row| row.get::<_, i32>(0),
        )
        .optional()?
        .is_some())
}

pub fn favorite_channel(channel_id: i64, favorite: bool) -> Result<()> {
    let sql = get_conn()?;
    sql.execute(
        r#"
        UPDATE channels
        SET favorite = ?1
        WHERE id = ?2
    "#,
        params![favorite, channel_id],
    )?;
    Ok(())
}

pub fn get_sources() -> Result<Vec<Source>> {
    let sql = get_conn()?;
    let sources: Vec<Source> = sql
        .prepare("SELECT * FROM sources")?
        .query_map([], row_to_source)?
        .filter_map(Result::ok)
        .collect();
    Ok(sources)
}

fn row_to_source(row: &Row) -> std::result::Result<Source, rusqlite::Error> {
    Ok(Source {
        id: row.get("id")?,
        name: row.get("name")?,
        username: row.get("username")?,
        password: row.get("password")?,
        url: row.get("url")?,
        source_type: row.get("source_type")?,
        url_origin: None,
    })
}

pub fn get_source_from_series_id(series_id: i64) -> Result<Source> {
    let sql = get_conn()?;
    Ok(sql.query_row(r#"
    SELECT * FROM sources where id = (
        SELECT source_id FROM channels WHERE url = ?
    )"#, [series_id], row_to_source)?)
}

#[cfg(test)]
mod test_sql {
    use std::collections::HashMap;

    use crate::{
        media_type,
        settings::{RECORDING_PATH, USE_STREAM_CACHING},
        sql::{create_structure, drop_db, structure_exists},
        types::Filters,
        view_type,
    };

    use super::{get_sources, search, update_settings};

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

    #[test]
    fn test_search() {
        let results = search(Filters {
            media_types: Some(vec![media_type::LIVESTREAM, media_type::MOVIE]),
            page: 1,
            query: Some("Fra".to_string()),
            source_ids: get_sources()
                .unwrap()
                .iter()
                .map(|x| x.id.unwrap())
                .collect(),
            view_type: view_type::ALL,
            group_name: None,
            series_id: None,
        })
        .unwrap();
        println!("{:?}\n\n", results);
        println!("{}", results.len());
    }

    #[test]
    fn test_search_group() {
        let results = search(Filters {
            media_types: None,
            page: 1,
            query: Some("Fra".to_string()),
            source_ids: get_sources()
                .unwrap()
                .iter()
                .map(|x| x.id.unwrap())
                .collect(),
            view_type: view_type::CATEGORIES,
            group_name: None,
            series_id: None,
        })
        .unwrap();
        println!("{:?}\n\n", results);
        println!("{}", results.len());
    }

    #[test]
    fn test_get_sources() {
        let results = get_sources().unwrap();
        println!("{:?}", results);
    }
}
