#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use directories::ProjectDirs;
use std::{
    fs::{self, File},
    io::{self, BufRead, Read},
    path::Path,
    process::{Child, Command, self},
    sync::Mutex, error::{Error, self},
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
    url: String
}

custom_error!{ProcessM3uError
    StringEmptyErr {line: usize} = "Line #{line} was empty",
    MissingName {line: usize} = "Missing tvg-name for line #{line}",
    IOErr {source: std::io::Error} = "{source}"
}

fn main() {
    let vec: Vec<Child> = Vec::new();
    let state = State(Mutex::new(StateContent {
        processes: vec
    }));
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![play_channel, get_playlist, get_cache, delete_cache, any_threads_active])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn play_channel(link: String, state: tauri::State<State>) {
    println!("Playing {}", link);
    let processes = &mut state.0.lock().unwrap().processes;
    if processes.len() > 0 {
        terminate_all(processes);
    }
    let child = Command::new("mpv")
        .arg("--fs")
        .arg(&link)
        .spawn()
        .expect("Failed to get child process");
    processes.push(child);
}

#[tauri::command(async)]
fn get_playlist(url: String) -> Option<Vec<Channel>> {
    let mut file = read_lines(url).unwrap();   
    let channels = match process_m3u(&mut file) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            return Option::None;
        },
    };
    save_to_cache(&channels);
    return Some(channels);
}

fn process_m3u(file: &mut std::io::Lines<std::io::BufReader<std::fs::File>>) -> Result<Vec<Channel>, ProcessM3uError> {
    let regex_name = Regex::new(r#"tvg-name="{1}(?P<name>[^=]*)"{1}"#).unwrap();
    let regex_id = Regex::new(r#"tvg-id="{1}(?P<name>[^=]*)"{1}"#).unwrap();
    let regex_logo = Regex::new(r#"tvg-logo="{1}(?P<logo>[^=]*)"{1}"#).unwrap();
    let regex_group = Regex::new(r#"group-title="{1}(?P<group>[^=]*)"{1}"#).unwrap();
    let mut channels: Vec<Channel> = Vec::new();

    let mut file = file.enumerate();
    file.next();
    while let Some(line_res) = file.next() {
        let line2_res = match file.next() {
            None => return Err(ProcessM3uError::StringEmptyErr {line: line_res.0}),
            Some(l) => l 
        };
        let line = line_res.1?;
        let line2 = line2_res.1?;
        let mut name = regex_name.captures(&line);
        if name.is_none() {
            name = regex_id.captures(&line);
            if name.is_none() {
                return Err(ProcessM3uError::MissingName { line: line_res.0 });
            }
        }
        let name = name.unwrap()["name"].to_string();
        let group = regex_group.captures(&line).map(|group| group["group"].to_string());
        let res_logo = regex_logo
            .captures(&line)
            .map(|logo| logo["logo"].to_string());
        let channel = Channel {
            logo: res_logo,
            name: name,
            group: group,
            url: line2,
        };
        channels.push(channel);
    }

    Ok(channels)
}

#[tauri::command(async)]
fn get_cache() -> Option<Vec<Channel>>{
    let cache_path = get_cache_path();
    if Path::exists(&cache_path) {
        let file = fs::read_to_string(cache_path).unwrap();
        return serde_json::from_str(&file).unwrap();
    }
    return None;
}

#[tauri::command(async)]
fn any_threads_active(state: tauri::State<State>) -> Option<i32> {
    let processes = &mut state.0.lock().unwrap().processes;
    for child in processes.iter_mut(){
        match child.try_wait() {
            Ok(v) => match v {
                Some(z) => return z.code(),
                None => return None
            },
            Err(e) => e
        };
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

fn get_cache_path() -> std::path::PathBuf{
    let path = ProjectDirs::from("", "fredolx", "open-tv").unwrap().cache_dir().join("cache.json");
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
