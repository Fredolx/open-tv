# Beats TV - Premium IPTV Player for Windows, macOS & Linux

<p align="center">
  <img src="readme_imgs/beats_tv_logo.png" alt="Beats TV - Best Free IPTV Player" width="200"/>
</p>

<p align="center">
  <strong>The Ultimate M3U & Xtream IPTV Player with Premium Themes</strong><br>
  <em>Fast • Beautiful • Feature-Rich • Free & Open Source</em>
</p>

<p align="center">
  <a href="https://github.com/officebeats/open-tv/releases">
    <img src="https://img.shields.io/github/v/release/officebeats/open-tv?style=for-the-badge&color=ff0033" alt="Latest Release"/>
  </a>
  <a href="https://github.com/officebeats/open-tv/stargazers">
    <img src="https://img.shields.io/github/stars/officebeats/open-tv?style=for-the-badge&color=ff0033" alt="GitHub Stars"/>
  </a>
  <a href="https://github.com/officebeats/open-tv/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/officebeats/open-tv?style=for-the-badge" alt="License"/>
  </a>
</p>

---

## 🎬 What is Beats TV?

**Beats TV** is a high-performance, cross-platform IPTV player designed for users who want a premium streaming experience. Built with Rust and Angular for speed on the backend and beauty on the frontend, Beats TV supports M3U playlists, M3U8 links, and Xtream Codes APIs.

Whether you're streaming live TV, movies, or TV series from your IPTV provider, Beats TV delivers a stunning visual experience with multiple themes, smart content filtering, and blazing-fast performance.

---

## ✨ Key Features

### 🎨 Premium Visual Themes

Choose from three stunning visual themes:

- **Clay-Mation** – Soft, puffy 3D claymorphism with red accents and glowing tiles
- **Smooth Glass** – Elegant glassmorphism with frosted transparency and blue highlights
- **Matrix Terminal** – Hacker-style green terminal aesthetic inspired by The Matrix

### 🎬 Enhanced Video Mode

IPTV-optimized playback with:

- Display resampling and interpolation for smoother video
- Aggressive caching (512MB) to prevent buffering
- High-quality catmull_rom scaling
- Auto-reconnect for dropped streams
- Platform-specific GPU optimizations (D3D11 for Windows, OpenGL for macOS)

### 🏷️ Smart Content Filtering

- **Tag-Based Filtering** – Automatically detects country/language tags (USA, UK, EN, Spanish, etc.)
- **Priority Sorting** – US/English tags bubble to the top for quick access
- **Content Type Filters** – Filter by Live TV, Movies/VOD, or Series
- **Bulk Hide/Show** – Select/deselect all visible tags at once

### ⚡ Performance & Efficiency

- **Ultra-Low Resource Usage** – Minimal RAM footprint
- **Instant Search** – Lightning-fast channel search
- **Hardware Acceleration** – GPU-accelerated playback with HDR support
- **Modern MPV Integration** – Powered by the mpv media player

### 📺 Multi-Source Management

- Import M3U files and URLs
- Xtream Codes API support
- Manage multiple IPTV providers
- Custom channel organization

### 🎥 Recording & Streaming

- **Record While Watching** – Save your favorite content
- **Re-streaming** – Share streams to other devices (phones, tablets, TVs)

### 🛋️ Living Room Ready

- Full keyboard and TV remote navigation
- Large, readable interface optimized for 10-foot viewing
- Smart tooltips and accessibility features

---

## 📥 Download & Installation

Download the latest release for your platform:

| Platform    | Download                                                                            |
| ----------- | ----------------------------------------------------------------------------------- |
| **Windows** | [Download .exe Installer](https://github.com/officebeats/open-tv/releases)          |
| **macOS**   | [Download .dmg](https://github.com/officebeats/open-tv/releases)                    |
| **Linux**   | [Download .deb / .rpm / .AppImage](https://github.com/officebeats/open-tv/releases) |

### Dependencies

| Platform                | Dependency Handling                                    |
| ----------------------- | ------------------------------------------------------ |
| **Windows (.msi/.exe)** | ✅**Automatic** – All dependencies bundled             |
| **Linux (.deb/.rpm)**   | ✅**Automatic** – Package managers handle dependencies |
| **Flatpak**             | ✅**Automatic** – Sandboxed with all dependencies      |
| **macOS**               | ⚠️**Manual** – See below                               |

#### macOS Users Only

Install dependencies via [Homebrew](https://brew.sh/):

```bash
brew install mpv ffmpeg yt-dlp
```

---

## ⌨️ Keyboard Shortcuts

| Shortcut           | Action             |
| ------------------ | ------------------ |
| `F1`               | Help               |
| `Ctrl + A`         | Show all channels  |
| `Ctrl + S`         | Show categories    |
| `Ctrl + D`         | Show favorites     |
| `Ctrl + F`         | Search             |
| `Ctrl + Q`         | Toggle livestreams |
| `Ctrl + W`         | Toggle movies      |
| `Ctrl + E`         | Toggle series      |
| `Backspace / Esc`  | Go back            |
| `Arrow Keys / Tab` | Navigation         |

---

## 🏗️ Building from Source

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install)
- [pnpm](https://pnpm.io/)

### Build Steps

```bash
# Clone the repository
git clone https://github.com/officebeats/open-tv.git
cd open-tv

# Install dependencies
pnpm install

# Development mode
pnpm tauri dev

# Production build
pnpm tauri build
```

---

## 🙏 Credits & Inspiration

This project is a fork of the excellent [Open TV](https://github.com/Fredolx/open-tv) by Fredolx. Full credit to the original developers for their incredible foundation.

**Why this fork exists:**
While Open TV is fantastic, we embrace AI-assisted development to push the boundaries of what's possible. Beats TV is built in the spirit of "Vibe Coding" – using AI as a powerful tool for rapid development, optimization, and innovation.

---

## 🔗 Connect

- [![LinkedIn](https://img.shields.io/badge/LinkedIn-0077B5?style=flat&logo=linkedin&logoColor=white)](https://www.linkedin.com/in/productmg/) – Connect professionally
- [![Twitter/X](https://img.shields.io/badge/X-000000?style=flat&logo=x&logoColor=white)](https://x.com/officebeats) – Follow for updates
- [GitHub Issues](https://github.com/officebeats/open-tv/issues) – Report bugs or request features
- [Releases](https://github.com/officebeats/open-tv/releases) – Download the latest version

---

## 📄 License

Beats TV is open source software, licensed under the same terms as the original Open TV project.

---

## 🔍 SEO Keywords

_IPTV Player, M3U Player, Xtream Player, Xtream Codes Player, Free IPTV App, Free IPTV Player, Free IPTV Software, Open Source IPTV, Open Source IPTV Player, Open Source M3U Player, IPTV for Windows, IPTV for Windows 10, IPTV for Windows 11, IPTV for Mac, IPTV for macOS, IPTV for Linux, IPTV for Ubuntu, IPTV for Debian, M3U8 Player, M3U Playlist Player, HLS Player, Live TV Streaming, Live TV Player, TV Series Player, VOD Player, Video on Demand Player, mpv IPTV, mpv Media Player, Desktop IPTV, Desktop Streaming App, Cross Platform IPTV, Multi Platform IPTV, Tauri App, Rust IPTV, Angular IPTV, Best Free IPTV Player, Best IPTV App Windows, Best M3U Player Windows, Best IPTV Player 2024, Best IPTV Player 2025, Free TV Streaming Software, Free Live TV App, Free Streaming Player, Stream Recorder, IPTV Recorder, Record Live TV, Record IPTV Streams, Hardware Accelerated Video, GPU Video Player, HDR Video Player, 4K IPTV Player, HD Streaming, IPTV Favorites, Channel Manager, Playlist Manager, IPTV Categories, EPG Player, Electronic Program Guide, TV Guide App, Claymorphism UI, Glassmorphism App, Matrix Theme, Dark Mode IPTV, Modern UI Player, Premium IPTV App, Beautiful IPTV Player, Lightweight IPTV, Fast IPTV Player, Low Resource IPTV, Minimal RAM Player, Instant Search IPTV, Quick Channel Search, TV Remote Compatible, Living Room IPTV, 10 Foot Interface, Smart TV Streaming, IPTV Restreaming, Share IPTV Streams, IPTV to Phone, IPTV to Tablet, Multi Source IPTV, Multiple Playlist IPTV, IPTV Provider Manager, Cord Cutting App, Cord Cutter Software, Free Cable TV Alternative, Watch Live TV Free, Stream TV Shows, Stream Movies Free, IPTV GitHub, IPTV Open Source GitHub_
