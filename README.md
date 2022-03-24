# Open-TV

Simple Rust IPTV Front-end and alternative to Hypnotix. Made with Tauri.
This is my first app in Rust as a C# Developer, if you see anything that could
be improved, feel free to contribute.

![alt text](https://github.com/Fredolx/open-tv/blob/main/demo.png)

## Why

- Lack of good open source IPTV apps
- Faster and much more stable than the alternatives

## Planned features

- Recording channels
- Clearing cache via GUI and/or multiple sources support
- Various optimizations
- Xtream support

## Prerequisites
```
sudo dnf install mpv #Fedora
sudo pacman -Syu mpv #Arch
sudo apt install mpv #Debian/Ubuntu
choco install mpv # Windows
```
## Install
You can install the latest version from [Releases](https://github.com/Fredolx/open-tv/releases/)

## Build/Dev

```
npm install
npm run tauri dev 
#or
npm run tauri build
```






