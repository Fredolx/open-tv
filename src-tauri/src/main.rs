#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use directories::ProjectDirs;
use directories::UserDirs;
use std::{
    fs::{self, File},
    io::{self, BufRead, Read},
    path::Path,
    process::{Child, Command},
    sync::Mutex,
};

use regex::{Regex, RegexSet};
use serde::{Deserialize, Serialize};

struct State(Mutex<StateContent>);
struct StateContent {
    processes: Vec<Child>,
    media_url: String
}
#[derive(Serialize, Deserialize)]
struct Channel {
    name: String,
    group: String,
    logo: Option<String>,
    url: String
}

fn main() {
    let vec: Vec<Child> = Vec::new();
    let url = "".to_string();
    let state = State(Mutex::new(StateContent {
        processes: vec,
        media_url: url,
    }));
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![play_channel, get_playlist, get_cache])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn play_channel(link: String, state: tauri::State<State>) {
    let processes = &mut state.0.lock().unwrap().processes;
    if processes.len() > 0 {
        terminate_all(processes);
    }
    let child = Command::new("sh")
        .arg("-c")
        .arg(format!("mpv {}", link))
        .spawn()
        .expect("Failed to get child process");
    processes.push(child);
}

#[tauri::command(async)]
fn get_playlist(url: String) -> Vec<Channel> {
    let regex_name = Regex::new(r#"tvg-name="{1}(?P<name>[^=]*)"{1}"#).unwrap();
    let regex_logo = Regex::new(r#"tvg-logo="{1}(?P<logo>[^=]*)"{1}"#).unwrap();
    let regex_group = Regex::new(r#"group-title="{1}(?P<group>[^=]*)"{1}"#).unwrap();
    let set = RegexSet::new(&[
        r#"tvg-name="{1}(?P<name>[^=]*)"{1}"#,
        r#"tvg-logo="{1}(?P<logo>[^=]*)"{1}"#,
        r#"group-title="{1}(?P<group>[^=]*)"{1}"#,
    ])
    .unwrap();
    let mut file = read_lines(url).unwrap();
    let mut channels: Vec<Channel> = Vec::new();
    file.next();
    while let Some(line_res) = file.next() {
        let line2 = file.next().unwrap().unwrap();
        let line = line_res.unwrap();
        let name = regex_name.captures(&line).unwrap()["name"].to_string();
        let group = regex_group.captures(&line).unwrap()["group"].to_string();
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
    save_to_cache(&channels);
    return channels;
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

fn save_to_cache(channels: &Vec<Channel>) {
    let serialized = serde_json::to_string(channels).unwrap();
    let path = get_cache_path();
    let folder = &path.parent().unwrap();
    std::fs::create_dir_all(&folder).unwrap();
    fs::write(path, serialized).expect("Could not save to cache");
}

fn get_cache_path() -> std::path::PathBuf{
    let path = ProjectDirs::from("", "fredolx", "open-tv").unwrap().cache_dir().join("cache.json");
    println!("{}", path.to_str().unwrap());
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
