#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use directories::ProjectDirs;
use std::{
    error::{self, Error},
    fs::{self, File},
    io::{self, BufRead, BufReader, Read},
    path::Path,
    process::{self, Child, Command, Stdio},
    sync::Mutex,
    time::SystemTime, string,
};
extern crate custom_error;
use custom_error::custom_error;
use regex::{Regex, RegexSet};
use serde::{Deserialize, Serialize};

struct State(Mutex<StateContent>);
struct StateContent {
    processes: Vec<Child>,
}
#[derive(Serialize, Deserialize)]
struct Channel {
    name: String,
    group: Option<String>,
    logo: Option<String>,
    url: String,
}

custom_error! {ProcessM3uError
    StringEmptyErr {line: usize} = "Line #{line} was empty",
    MissingName {line: usize} = "Missing tvg-name for line #{line}",
    IOErr {source: std::io::Error} = "{source}"
}

fn main() {
    let vec: Vec<Child> = Vec::new();
    let state = State(Mutex::new(StateContent { processes: vec }));
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            play_channel,
            get_playlist,
            get_cache,
            delete_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command(async)]
fn play_channel(link: String, state: tauri::State<State>) {
    println!("Playing {}", link);
    let processes = &mut state.0.lock().unwrap().processes;
    if processes.len() > 0 {
        terminate_all(processes);
    }
    let mut child = Command::new("mpv")
        .arg("--fs")
        .arg(&link)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdout = child.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        let start = SystemTime::now();
        for line in stdout_lines {
            let line = line.unwrap();
            if line.starts_with("AO:")
                || line.starts_with("VO:")
                || line.starts_with("AV:")
                || start.elapsed().unwrap().as_millis() > 10000
            {
                break;
            }
        }
    }
    processes.push(child);
}

#[tauri::command(async)]
fn get_playlist(url: String) -> Option<Vec<Channel>> {
    let regex_name = Regex::new(r#"tvg-name="{1}(?P<name>[^"]*)"{1}"#).unwrap();
    let regex_id = Regex::new(r#"tvg-id="{1}(?P<name>[^"]*)"{1}"#).unwrap();
    let regex_logo = Regex::new(r#"tvg-logo="{1}(?P<logo>[^"]*)"{1}"#).unwrap();
    let regex_group = Regex::new(r#"group-title="{1}(?P<group>[^"]*)"{1}"#).unwrap();
    let mut channels: Vec<Channel> = Vec::new();
    let file = File::open(url).unwrap();
    let mut reader = BufReader::new(file);
    let mut buf = vec![];
    let mut twoLines: Vec<String> = vec![];
    let mut firstLinePassed = false;
    while let Ok(_) = reader.read_until(b'\n', &mut buf) {
        if !firstLinePassed{
            firstLinePassed = true;
            continue;
        }
        if buf.is_empty() {
            break;
        }
        let mut line = String::from_utf8_lossy(&buf).to_string();
        trim_newline(&mut line);
        twoLines.push(line);
        buf.clear();
        if twoLines.len() == 2 {
            let mut name = regex_name.captures(&twoLines[0]);
            if name.is_none() {
                name = regex_id.captures(&twoLines[0]);
                if name.is_none() {
                    twoLines.clear();
                    continue;
                }
            }
            let name = name.unwrap()["name"].to_string();
            let group = regex_group
                .captures(&twoLines[0])
                .map(|group| group["group"].to_string());
            let res_logo = regex_logo
                .captures(&twoLines[0])
                .map(|logo| logo["logo"].to_string());
            let channel = Channel {
                logo: res_logo,
                name: name,
                group: group,
                url: twoLines[1].to_string(),
            };
            twoLines.clear();
            channels.push(channel);
        }
    }
    println!("{} channels found!", channels.len());
    save_to_cache(&channels);
    return Some(channels);
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

#[tauri::command(async)]
fn get_cache() -> Option<Vec<Channel>> {
    let cache_path = get_cache_path();
    if Path::exists(&cache_path) {
        let file = fs::read_to_string(cache_path).unwrap();
        return serde_json::from_str(&file).unwrap();
    }
    return None;
}

#[tauri::command(async)]
fn delete_cache() {
    let cache_path = get_cache_path();
    fs::remove_file(cache_path).expect("Could not delete cache");
}

fn save_to_cache(channels: &Vec<Channel>) {
    let serialized = serde_json::to_string(channels).unwrap();
    let path = get_cache_path();
    let folder = &path.parent().unwrap();
    std::fs::create_dir_all(&folder).unwrap();
    fs::write(path, serialized).expect("Could not save to cache");
}

fn get_cache_path() -> std::path::PathBuf {
    let path = ProjectDirs::from("", "fredolx", "open-tv")
        .unwrap()
        .cache_dir()
        .join("cache.json");
    return path;
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn terminate_all(processes: &mut Vec<Child>) {
    for child in processes.iter_mut() {
        let _ = child.kill();
    }
    processes.clear();
}
