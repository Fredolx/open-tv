# Open TV

Completely rewritten to accommodate new features and to be even speedier, Open TV has been carefully crafted to deliver the best IPTV experience.

# This project NEEDS your help. Please consider donating on [Github](https://github.com/sponsors/Fredolx), [Patreon](https://www.patreon.com/fredol) or directly by [crypto](#crypto)
I've been developing and maintaining this project alone and for entirely for free over the past 2 years. I am in dire need of support to continue developing this project. I've never added annoying donation pop-ups or anything of the sort to make sure you have the fastest and cleanest IPTV experience and I'm committed to keep this project FREE & OPEN-SOURCE. To keep that commitment, I need your support!

~~I'm currently developing a big rework of the app in Rust to allow for new big features that have been requested by the community for some time...~~

The app rewrite is done 🎊 Thank you to all who have donated during the last 3 months. I still need your support for the next updates!

![Image of the app](https://raw.githubusercontent.com/Fredolx/open-tv/refs/heads/main/demo1.png)

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

Open TV is also now available on Flatub (LINK AVAILAIBLE SOON) 🎊

If you use Arch Linux, you can also [install it from the AUR](https://aur.archlinux.org/packages/open-tv-bin) 

## Prerequisites
If you are on Windows or use the flatpak on Linux; SKIP THIS PART. 

The app both depends on mpv, ffmpeg and yt-dlp. ffmpeg is a depedency of mpv on all package managers. On Fedora you will need to add rpmfusion and on OpenSUSE you will need to install codecs with opi beforehand.

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

SOL:
```
AM7roSrxBKrS5mG7q6aXnQHZKh3ArtBxvG3x1B1VjKhj
```

BNB:
```
0x0C8C5217a8044b3736aD82CCFB9f099597b65253
```
