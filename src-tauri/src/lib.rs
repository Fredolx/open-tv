
use anyhow::Error;

pub mod sql;
pub mod types;
pub mod xtream;
pub mod settings;
pub mod m3u;
pub mod mpv;

fn print_error_stack(e: Error) {
    eprintln!("{:?}", e);
}