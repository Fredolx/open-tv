# Lily TV

A native IPTV player built with [GPUI](https://www.gpui.rs/) (via [gpui-kit](https://gpui-kit.com)),
forked from [Fred TV / Open TV](https://github.com/Fredolx/open-tv). The Tauri + Angular
frontend has been replaced by a native Rust UI; the IPTV backend is Fred TV's.

Lily TV reads and writes the same channel database as Fred TV
(`%APPDATA%\fredol\open-tv\data\db.sqlite` on Windows), so sources, favourites and
history carry over in both directions.

## Features

- Add Xtream, M3U link and M3U file sources; refresh, enable/disable or delete them
- Browse All, Favourites, Categories and History; drill into categories, series and seasons
- Instant search, Live/Movies/Series filters, provider or A-Z/Z-A ordering
- Favourite channels inline; plays everything in mpv, with a stop control
- Settings: start view, sort, volume, extra mpv arguments, stream caching,
  hardware decoding, gpu-next renderer, refresh on start

Not yet ported from Fred TV: EPG and reminders, recording and downloads, restream,
custom channels/groups and sharing, hiding channels/groups, favourites backup, tray icon,
installers.

## Building

Requires a recent stable Rust toolchain (on Windows, the MSVC build tools).

```
cargo run -p lily-tv            # debug
cargo build --release -p lily-tv
cargo test --workspace
```

Playback needs `mpv` (and `ffmpeg`/`yt-dlp` for some streams). On Windows they are looked
up on `PATH`, then in a `deps` folder next to the executable, then in an installed
Fred TV's `C:\Program Files\Fred TV\deps`.

## Layout

- `crates/lily-core`: Fred TV's backend without Tauri: sources (M3U/Xtream), the SQLite
  channel database, mpv playback, EPG, restream
- `crates/lily-tv`: the GPUI application (`browse`, `sources`, `settings` features and the
  window shell)

## Keys

- Arrow keys and Enter: move through and open results
- Esc, Backspace or Alt+Left: go back a level

## License

GPL-2.0, inherited from Fred TV (see `LICENSE`). Note that GPUI and gpui-kit are
Apache-2.0, which the FSF considers incompatible with GPL-2.0-only; resolve this before
distributing binaries.
