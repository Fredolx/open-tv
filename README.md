# Beats TV

A modern, feature-rich IPTV player and manager. Free and open source.

**Fast • Beautiful • Feature-Rich • Free & Open Source**

---

## Features

| Category | Features |
|----------|----------|
| **Playback** | MPV & VLC support, Live TV, Movies, Series, TV Archive, External Players |
| **Organization** | Groups, Favorites, Custom Channels, Smart Filters, EPG Guide |
| **Content** | Xtream Codes, M3U, Custom Sources, Genre Filtering, Release Date Sorting |
| **Privacy** | VPN Mode, Custom Headers, Ignore SSL, Password Protection |
| **UX** | Dark/Light/Matrix Themes, Keyboard Navigation, Search, Bulk Actions |

---

## Downloads

| Platform | Download |
|----------|----------|
| Windows | [Beats TV Setup.exe](https://github.com/admin-beats/beats-tv/releases/latest) |
| Linux | .deb, .AppImage, Arch AUR, Flatpak |
| macOS | .dmg (Apple Silicon & Intel) |
| Android | APK |

---

## Quick Start

1. **Add Your Source**
   - Xtream Codes API (most providers)
   - M3U Playlist URL
   - Manual channel definition

2. **Navigate**
   - `Arrow keys` - Browse channels
   - `Enter` - Play
   - `F` - Favorites
   - `S` - Search
   - `Esc` - Back/Close

3. **Customize**
   - Theme: Settings → Theme
   - Player: Settings → Player
   - VPN: Settings → VPN Mode

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Play/Pause |
| `Arrows` | Navigate |
| `F` | Toggle Favorite |
| `S` | Search |
| `G` | Go to Channel |
| `R` | Refresh |
| `M` | Mute |
| `+/-` | Volume |
| `Esc` | Back/Close |

---

## Build from Source

```bash
git clone https://github.com/admin-beats/beats-tv.git
cd beats-tv

# Frontend
npm install
npm run dev

# Desktop App
cargo install tauri-cli
cargo tauri dev
```

---

## Credits

- **Original Project**: [Open TV](https://github.com/fredolx/open-tv) by [Fredolx](https://github.com/fredolx) - Licensed under GPL-2.0
- **Beats TV**: A feature-enhanced fork with modern UI and additional functionality

---

## License

GPL-2.0 - See [LICENSE](LICENSE) for details.
