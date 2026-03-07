# FredTV-Next

A personal fork of [Fred TV (open-tv)](https://github.com/Fredolx/open-tv) with a major UI overhaul, inline player, server-side recording, and modern design system.

All changes are being submitted as PRs back to upstream. This repo exists as the full working source for those PRs.

## What's New

### Inline Player
- **Web Player** (hls.js) and **Embedded MPV** playback engines, alongside the existing External MPV
- Configurable in Settings, three-state layout (closed / mini / expanded)
- `local_player.rs` backend proxies streams via ffmpeg to localhost

### Server-Side Recording
- Record any livestream to disk via ffmpeg, independent of playback
- Graceful shutdown (writes "q" to ffmpeg stdin to preserve MP4 moov atom)
- Start/stop from the channel context menu, busy state tracking

### UI Overhaul
- **Sidebar navigation** with collapsible sections (Home, All, Categories, Favorites, History, Hidden, per-source filtering)
- **Dashboard home view** with content rows (recently watched, favorites, by source)
- **Search overlay** (Ctrl+K / Cmd+K) with instant results
- **Channel detail panel** (slide-out info from right-click > Info)
- **Now-playing bar** at the bottom when something is playing
- **Multiple channel views**: grid large, grid compact, and list/table mode
- **Skeleton loading** and contextual **empty states**
- **Settings redesign** with card-based sections

### Design System
- CSS custom properties (`--ftv-*`) for surfaces, borders, text, accent, semantic colors, radius, transitions, shadows
- Material menu overrides, modal animations, toastr dark theme
- All hardcoded colors migrated to design tokens

### Bug Fixes
- Loading component interval leak (clearInterval on destroy)
- SQL page offset underflow clamp (`.max(1)`)
- Home nav dead code (`tmpFocus / 3` was missing `=`)

### Performance
- `trackBy` on channel ngFor prevents full DOM re-renders
- Cached `anyXtream()` check (computed once, not on every change detection)

## Upstream PRs

These changes are split into 8 PRs against [Fredolx/open-tv](https://github.com/Fredolx/open-tv):

| PR | Branch | Description |
|----|--------|-------------|
| [#387](https://github.com/Fredolx/open-tv/pull/387) | `fix/loading-interval-leak` | Fix interval memory leak |
| [#388](https://github.com/Fredolx/open-tv/pull/388) | `fix/sql-page-offset-clamp` | Fix SQL page offset underflow |
| [#389](https://github.com/Fredolx/open-tv/pull/389) | `fix/home-nav-dead-code` | Fix dead code in nav() |
| [#390](https://github.com/Fredolx/open-tv/pull/390) | `feat/design-system` | CSS custom properties design system |
| [#391](https://github.com/Fredolx/open-tv/pull/391) | `perf/home-improvements` | trackBy + cached anyXtream |
| [#392](https://github.com/Fredolx/open-tv/pull/392) | `feat/recording-system` | Server-side recording |
| [#393](https://github.com/Fredolx/open-tv/pull/393) | `feat/inline-player` | Inline player (stacked on #392) |
| [#394](https://github.com/Fredolx/open-tv/pull/394) | `feat/ui-overhaul` | UI overhaul (stacked on #393) |

PRs 1-5 are independent. PRs 6-8 are stacked (merge in order).

## Building

Same as upstream. Requires Rust, Node.js, and pnpm.

```bash
pnpm install
cargo tauri dev    # development
cargo tauri build  # production
```

## Credits

All credit to [Fredolx](https://github.com/Fredolx) for the original Fred TV. Please [support the project](https://github.com/sponsors/Fredolx).
