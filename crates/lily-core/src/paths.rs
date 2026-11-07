//! Where the database, logs and caches live.
//!
//! By default these are Fred TV's folders, so both apps share one channel database.
//! Setting `LILY_TV_DIR` moves everything under that folder instead, which keeps a
//! test or second profile away from the real data.

use std::path::PathBuf;

use directories::ProjectDirs;

pub const DIR_OVERRIDE_ENV: &str = "LILY_TV_DIR";

fn override_dir() -> Option<PathBuf> {
    std::env::var_os(DIR_OVERRIDE_ENV)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("dev", "fredol", "open-tv").expect("no home directory to store data in")
}

/// The database folder, created if missing.
pub fn data_dir() -> PathBuf {
    ensure(match override_dir() {
        Some(dir) => dir.join("data"),
        None => project_dirs().data_dir().to_owned(),
    })
}

/// The cache folder (logs, temporary playlists, restream segments), created if missing.
pub fn cache_dir() -> PathBuf {
    ensure(match override_dir() {
        Some(dir) => dir.join("cache"),
        None => project_dirs().cache_dir().to_owned(),
    })
}

fn ensure(dir: PathBuf) -> PathBuf {
    // callers report a clearer error when they try to use a folder that couldn't be made
    let _ = std::fs::create_dir_all(&dir);
    dir
}
