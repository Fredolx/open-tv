use std::{
    borrow::BorrowMut,
    sync::{LazyLock, Mutex},
};

use crate::types::{Channel, SourceType};
use anyhow::Result;
use directories::ProjectDirs;
use rusqlite::{params, types::Null, Connection, OptionalExtension, Transaction};

pub static CONN: LazyLock<Mutex<Connection>> =
    LazyLock::new(|| Mutex::new(Connection::open(get_and_create_sqlite_db_path()).unwrap()));

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

fn create_structure() -> Result<()> {
    let sql = CONN.lock().unwrap();
    sql.execute_batch(
        r#"
CREATE TABLE "sources" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" varchar(100),
  "source_type" integer
);

CREATE TABLE "channels" (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" varchar(250),
  "group_name" varchar(250),
  "image" varchar(600),
  "url" varchar(600),
  "source_id" integer,
  FOREIGN KEY (source_id) REFERENCES sources(id)
);
"#,
    )?;
    Ok(())
}

fn structure_exists() -> Result<bool> {
    let sql = CONN.lock().unwrap();
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

pub fn create_or_find_source_by_name(source_name: String, source_type: SourceType) -> Result<i64> {
    let sql = CONN.lock().unwrap();
    let id: Option<i64> = sql
        .query_row(
            "SELECT id FROM sources WHERE name = ?1",
            params![source_name],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(id) = id {
        return Ok(id);
    }
    sql.execute(
        "INSERT INTO sources (name, source_type) VALUES (?1, ?2)",
        params![source_name, source_type as u8],
    )?;
    Ok(sql.last_insert_rowid())
}

pub fn insert_channel(tx: &Transaction, channel: Channel) -> Result<()> {
    tx.execute(
        r#"
INSERT INTO channels (name, group_name, image, url, source_id) 
VALUES (?1, ?2, ?3, ?4, ?5); 
"#,
        rusqlite::params![
            channel.name,
            channel.group,
            channel.image,
            channel.url,
            channel.source_id
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod test_sql {
    use crate::sql::{create_structure, get_and_create_sqlite_db_path, structure_exists};

    #[test]
    fn test_structure_exists() {
        std::fs::remove_file(get_and_create_sqlite_db_path()).unwrap_or_default();
        assert_eq!(structure_exists().unwrap(), false);
        create_structure().unwrap();
        assert_eq!(structure_exists().unwrap(), true);
    }
}
