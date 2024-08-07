use std::sync::{LazyLock, Mutex};

use crate::types::{Channel, Source};
use anyhow::{Ok, Result};
use directories::ProjectDirs;
use rusqlite::{params, Connection, OptionalExtension, Transaction};

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

//@TODO: Nullable types
fn create_structure() -> Result<()> {
    let sql = CONN.lock().unwrap();
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
  "source_id" integer,
  FOREIGN KEY (source_id) REFERENCES sources(id)
);

CREATE INDEX index_channel_name
ON channels(name);
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

pub fn drop_db() -> Result<()> {
    let sql = CONN.lock().unwrap();
    sql.execute_batch("DROP TABLE channels; DROP TABLE sources;")?;
    Ok(())
}

pub fn create_or_find_source_by_name(source: &mut Source) -> Result<()> {
    let sql = CONN.lock().unwrap();
    let id: Option<i64> = sql
        .query_row(
            "SELECT id FROM sources WHERE name = ?1",
            params![source.name],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(id) = id {
        source.id = Some(id);
        return Ok(())
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
INSERT INTO channels (name, group_name, image, url, source_id) 
VALUES (?1, ?2, ?3, ?4, ?5); 
"#,
        params![
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
    use crate::sql::{create_structure, drop_db, structure_exists};

    #[test]
    fn test_structure_exists() {
        drop_db().unwrap();
        assert_eq!(structure_exists().unwrap(), false);
        create_structure().unwrap();
        assert_eq!(structure_exists().unwrap(), true);
    }
}
