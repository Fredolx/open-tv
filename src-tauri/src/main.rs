#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    fs::{self, File},
    io::{self, BufRead, Read},
    path::Path,
    process::{Child, Command},
    sync::{Mutex}
};

use regex::{Regex, RegexSet};
use serde::{Serialize, Deserialize};

struct State(Mutex<StateContent>);
struct StateContent {
    processes: Vec<Child>,
    media_url: String,
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
        .invoke_handler(tauri::generate_handler![play_channel, get_playlist])
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

fn get_file(media_url: String) -> Vec<u8> {
    let mut file = std::fs::File::open(media_url).unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    return bytes;
}

#[tauri::command]
fn get_playlist() -> Vec<Channel> {
    //Todo: add param for path
    let regex_name = Regex::new( r#"tvg-name="{1}(?P<name>[^=]*)"{1}"#).unwrap();
    let regex_logo = Regex::new(r#"tvg-logo="{1}(?P<logo>[^=]*)"{1}"#).unwrap();
    let regex_group = Regex::new(r#"group-title="{1}(?P<group>[^=]*)"{1}"#).unwrap();
    let set = RegexSet::new(&[
        r#"tvg-name="{1}(?P<name>[^=]*)"{1}"#,
        r#"tvg-logo="{1}(?P<logo>[^=]*)"{1}"#,
        r#"group-title="{1}(?P<group>[^=]*)"{1}"#,
    ]).unwrap();
    let mut file = read_lines("/Users/fredericlachapelle/Downloads/test.m3u").unwrap();
    let mut channels: Vec<Channel> = Vec::new();
    file.next();
    while let Some(line_res) = file.next() {
        let line2 = file.next().unwrap().unwrap();
        let line  = line_res.unwrap();
        let name = regex_name.captures(&line).unwrap()["name"].to_string();
        let group = regex_group.captures(&line).unwrap()["group"].to_string();
        let res_logo = regex_logo.captures(&line).map(|logo| logo["logo"].to_string());
        let channel = Channel 
        {
            logo: res_logo, 
            name: name, 
            group: group, 
            url: line2
        };
        channels.push(channel);
    }
    return channels;
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
