<p align="center"><img src="crates/lily-tv/assets/logo.png" alt="Lily TV logo: a golden retriever" width="160"></p>

# Lily TV

A native IPTV player built in Rust with [GPUI](https://www.gpui.rs/) (via
[gpui-kit](https://gpui-kit.com)).

The channel database lives in `%APPDATA%\lilytv\lily-tv\data\db.sqlite` on Windows. On its
first start Lily TV copies over the database from the app's previous data folder, if there
is one, so existing sources, favourites and history carry over. Channels, groups and sources
can be shared as `.otv`, `.otvg` and `.otvp` files, and favourites backed up as `.otvf`.

## Features

- **Sources:** Xtream, M3U links, M3U files, custom sources and shared-file imports; refresh,
  enable/disable, edit (URL, login, user agents, max streams, TVG-ID) or delete them; Xtream
  account expiry; favourites backup and restore
- **Browsing:** All, Favourites, Categories, History and Hidden views; drill into categories,
  series and seasons; instant search (exact phrase or words in any order); Live/Movies/Series
  filters; provider or A-Z/Z-A ordering; group names and catch-up tags on every row
- **Every row** has a menu (button or right-click): favourite, hide/unhide, remove from history,
  record, download, TV guide, re-stream, copy URL, and edit/share/delete for custom items
- **Bulk actions:** favourite, unfavourite, hide or unhide every result of the current search
- **TV guide** for Xtream live channels: day by day, opens on what's on now, reminders with
  system notifications, catch-up play and download from the provider's archive
- **Playback** in mpv with a stop control; **recording** live channels; **downloads** with
  progress and cancel; the recordings folder and "always ask where to save" settings apply
- **Re-streaming** a channel to other devices on the network, with "watch here" and export as
  a channel file
- **Custom sources:** add/edit/delete channels (with Referer, User-Agent and Origin headers)
  and groups; share channels, groups or whole sources as files
- **Settings:** start view, sort, volume, extra mpv arguments, stream caching, hardware
  decoding, gpu-next renderer, refresh on start, recordings folder, re-stream port, zoom, tray,
  and "Delete everything"
- **Windows:** tray icon (Show / Quit), closing to the tray, and a single running instance

## Keys

| Keys | Action |
| --- | --- |
| Arrow keys, Enter | Move through and open results |
| Esc, Backspace, Alt+Left | Go back a level |
| Ctrl+A / Ctrl+D / Ctrl+S / Ctrl+H | All / Favourites / Categories / History |
| Ctrl+F | Search |
| Ctrl+Q / Ctrl+W / Ctrl+E | Show or hide live channels / movies / series |
| F1 | Shortcut list |

## Building

Requires a recent stable Rust toolchain (on Windows, the MSVC build tools).

```
cargo run -p lily-tv            # debug
cargo build --release -p lily-tv
cargo test --workspace
```

Playback needs `mpv` (and `ffmpeg`/`yt-dlp` for recording, re-streaming and some streams).
On Windows they are looked up on `PATH`, then in a `deps` folder next to the executable,
then in an installed Lily TV's `C:\Program Files\Lily TV\deps`.

Set `LILY_TV_DIR` to keep the database and caches in another folder, for example to test
against a copy.

### Windows installer

```
.\packaging\fetch-deps.ps1          # or -FromInstalled to copy them from an installed Lily TV
cargo install cargo-packager --locked
cd crates\lily-tv
cargo packager --release            # target\release\lily-tv_<version>_x64-setup.exe
```

The installer bundles mpv, ffmpeg and yt-dlp in a `deps` folder beside `lily-tv.exe`.
CI (`.github/workflows/build.yml`) builds it on every push, and builds and tests on Linux and
macOS.

## Layout

- `crates/lily-core`: the backend: sources (M3U/Xtream), the SQLite
  channel database, mpv playback, EPG, restream, sharing
- `crates/lily-tv`: the GPUI application, one module per feature (`browse`, `guide`,
  `sources`, `custom`, `downloads`, `restream`, `settings`) plus the window shell and
  Windows desktop integration
- `packaging`: the installer's dependency script, and `logo/make_logo.py`, which draws the
  logo and writes `crates/lily-tv/assets/icon.png`, `icon.ico` and `logo.png`
  (`python packaging/logo/make_logo.py <out-dir>`, needs Pillow)

## License

Lily TV is a modified version of [Fred TV](https://github.com/Fredolx/open-tv) (formerly
Open TV) by [Fredolx](https://github.com/Fredolx), and like it is licensed under the GNU
General Public License, version 2 (see `LICENSE`).

Changes from Fred TV, made in November 2026:

- The Tauri + Angular user interface was replaced by a native one written with GPUI
  (`crates/lily-tv`)
- Fred TV's Rust backend was moved into `crates/lily-core` and adapted to run without
  Tauri; dependencies were updated
- The app was renamed to Lily TV, with its own data folder, logo and installer

Note that GPUI and gpui-kit are Apache-2.0, which the FSF considers incompatible with
GPL-2.0-only; resolve this before distributing binaries.
