use std::sync::{LazyLock, Mutex};

use anyhow::Result;
use directories::ProjectDirs;
use rusqlite::{Connection, OptionalExtension};

static CONN: LazyLock<Mutex<Connection>> =
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
  "group" varchar(250),
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
