# This project NEEDS your help. Please consider donating on [Github](https://github.com/sponsors/Fredolx), [Patreon](https://www.patreon.com/fredol) or directly by [crypto](#crypto)
I've been developing and maintaining this project alone and for entirely for free over the past 2 years. I am in dire need of support to continue developing this project. I've never added annoying donation pop-ups or anything of the sort to make sure you have the fastest and cleanest IPTV experience and I'm committed to keep this project FREE & OPEN-SOURCE. To keep that commitment, I need your support!

I'm currently developing a big rework of the app in Rust to allow for new big features that have been requested by the community for some time:

- Multi IPTV Sources with per-source filtering
- Even faster search & start-up
- Flatpak publishing
- 3x less memory usage and 95% leaner executable (100+ MB to 5 MB)

I've been making some huge progress on another branch (rust-rewrite). If you can please donate, I will be able to finish it and deliver you those improvements I've carefully crafted for **you**

# Open-TV

Simple & fast IPTV app made with Electron and Angular

![alt text](https://github.com/Fredolx/open-tv/blob/main/demo.png)

## Features

- Super fast
- M3U file, M3U link support and partial Xtream support
- Easy to use
- Recording & favorites
- Fully customizable player through mpv conf
- Bad/slow/unstable stream mitigations

## Prerequisites
The app both depends on mpv and ffmpeg. ffmpeg is a depedency of mpv on all package managers. On Fedora you will need to add rpmfusion and on OpenSUSE you will need to 
install codecs with opi beforehand.

The Windows build **comes with mpv included**, but you should still install mpv from a package manager of your choice to always have the latest version installed

```
sudo dnf install mpv #Fedora
sudo zypper install mpv #OpenSUSE
sudo pacman -Syu mpv #Arch
sudo apt install mpv #Debian/Ubuntu
scoop install mpv # Windows
choco install mpv # Windows alternative
```

The .deb package should include mpv as a dependency but [due to a bug in electron forge it's not working](https://github.com/electron/forge/issues/3127). So install it manually alongside ffmpeg if you want full functionality on Ubuntu/Debian.

## Feedback
Feel free to submit any kind of feedback by creating a new issue.

## Install
You can install the latest version from [Releases](https://github.com/Fredolx/open-tv/releases/)

If you use Arch Linux, you can also [install it from the AUR](https://aur.archlinux.org/packages/open-tv-bin) 

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

## Donate Crypto (Thank you!)
BTC:
```
bc1q7v27u4mrxhtqzl97pcp4vl52npss760epsheu3
```

ETH:
```
0x171D5B628eff75c98c141aD5584FffA209274365
```

LTC:
```
ltc1qzxgp2grt9ayvpv0dur7lgzgf88yp09h2ytmga0
```

BCH:
```
bitcoincash:qz4mauqyytkvhp9yze0qhgn2nnlv4z5glckyysxg2n
```

