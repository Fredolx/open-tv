# Open TV

Completely rewritten to accommodate new features and to be even speedier, Open TV has been carefully crafted to deliver the best IPTV experience.

![Image of the app](https://raw.githubusercontent.com/Fredolx/open-tv/refs/heads/rust-rewrite/demo1.png)

## Features:
- Import your IPTV channels from any source (M3U File, M3U link, Xtream) 🗃️
- Record while watching 🎥
- Multi IPTV sources 🎊
- Control the UI from a TV remote 📺
- Super low RAM usage, crazy speeds, and instant search 🚅
- Refresh your sources when you need it 🔄
- Add channels to favorites 🌟

## Install
You can install the latest version from [Releases](https://github.com/Fredolx/open-tv/releases/)
Open TV is also now available on [Flathub](TBD) 🎊
If you use Arch Linux, you can also [install it from the AUR](https://aur.archlinux.org/packages/open-tv-bin) 

## Prerequisites
If you are on Windows or use the flatpak on Linux; SKIP THIS PART. 

The app both depends on mpv, ffmpeg and yt-dlp. ffmpeg is a depedency of mpv on all package managers. On Fedora you will need to add rpmfusion and on OpenSUSE you will need to 
install codecs with opi beforehand.

The Windows build **comes with mpv included** (.msi), but you should still install mpv from a package manager of your choice to always have the latest version installed

```
sudo dnf install mpv ffmpeg yt-dlp #Fedora
sudo zypper install mpv ffmpeg yt-dlp #OpenSUSE
sudo pacman -Syu mpv ffmpeg yt-dlp #Arch
sudo apt install mpv ffmpeg yt-dlp #Debian/Ubuntu
scoop install mpv ffmpeg yt-dlp # Windows
choco install mpv ffmpeg yt-dlp # Windows alternative
```

## Feedback
Feel free to submit any kind of feedback by creating a new issue.

## Hotkeys
* F1: Help
* Ctrl + a: Show all channels
* Ctrl + s: Show categories
* Ctrl + d: Show favorites
* Ctrl + f: Search
* Ctrl + q: Enable/Disable livestreams
* Ctrl + w: Enable/Disable movies
* Ctrl + e: Enable/Disable series
* Backspace/Esc: Go back
* Arrow keys/Tab/Shift+Tab: Navigation

If you have a tv remote or air mouse that has slightly different bindings for general nav (back, up, down, left, right),
please open an issue and I will add them if it's feasible. Otherwise, you can still use hwdb to make them match OpenTV's bindings.

## Settings explained

**Stream Caching**

Why enabling:
  - If you have a slow internet connection/IPTV provider causing the stream to pause often

Why disabling: 
  - If the stream often drops completely. It will prevent the stream from jumping too far ahead/behind
  - If you have a good internet/provider and want lower latency
  - Can prevent some weird bugs/slowdowns
