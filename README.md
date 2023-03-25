# Open-TV

Simple & fast IPTV app made with Electron and Angular

![alt text](https://github.com/Fredolx/open-tv/blob/main/demo.png)

## Features

- Super fast
- M3U file and link support
- Easy to use
- Recording & favorites
- Fully customizable player through mpv conf

## Planned features

- Xtream support
- Providing better support for bad streams (proxying the source and relaying to mpv)
- Publishing on Flathub

## Prerequisites
The app both depends on mpv and ffmpeg. ffmpeg is a depedency of mpv on all package managers. On Fedora you will need to add rpmfusion.

The Windows build **comes with mpv included**, but you may still install mpv from a package manager of your choice to always have the latest version installe

```
sudo dnf install mpv #Fedora
sudo pacman -Syu mpv #Arch
sudo apt install mpv #Debian/Ubuntu
scoop install mpv # Windows
choco install mpv # Windows alternative
```
The .deb package should include mpv as a dependency but [due to a bug in electron forge it's not working](https://github.com/electron/forge/issues/3127). So install it manually alongside ffmpeg if you want full functionality on Ubuntu/Debian.

## Contribute
Submit a PR anytime if you find something to improve. There may also be some suggestions in the issues. I'm not the most expert Javascript/NodeJS guy so you will certainly find some little things to fix.

## Install
You can install the latest version from [Releases](https://github.com/Fredolx/open-tv/releases/)

If you use Arch Linux, you can also [install it from the AUR](https://aur.archlinux.org/packages/open-tv-bin) 

## Hotkeys
* F1: Help
* Ctrl + a: Show all channels
* Ctrl + s: Show favorites
* Ctrl + d: Select first channel
  * Tab: Select next channel
  * Shift + Tab: Select previous channel
* Ctrl + f: Search

## Settings explained

**Stream Caching**

Why enabling:
  - If you have a slow internet connection/IPTV provider causing the stream to pause often

Why disabling: 
  - If the stream often drops completely. It will prevent the stream from jumping too far ahead/behind
  - If you have a good internet/provider and want lower latency
  - Can prevent some weird bugs/slowdowns

## Build
For building from source, you will need those packages
```
rpm dpkg fakeroot
```
For node and npm, I would strongly suggest to use NVM (node version manager) to get the latest LTS.

And then to build
```
cd ng-open-tv
npm run prod
cd ../electron-open-tv
npm run publish
```
You can add/edit targets in forge.config.js. 
On Windows, to prepackage mpv, place it in /libs (you will need to also create the folder).

## Repackaging
I am fine if you repackage the app on open source platforms like the AUR or Snap. As long as proper credit is given and it's free. I do not authorize any repackaging on proprietary platforms like the Microsoft Store.





