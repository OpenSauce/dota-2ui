# dota-2ui

TUI dashboard for Dota 2 esports. Live matches, fixtures, tournament countdowns, and a fullscreen broadcast mode.

## Install

```bash
cargo install --git https://github.com/OpenSauce/dota-2ui
```

With desktop notifications (requires `libdbus-1-dev` on Linux):

```bash
cargo install --git https://github.com/OpenSauce/dota-2ui --features notifications
```

## Keybinds

| Key | Action |
|-----|--------|
| `Enter` | Select match / tournament (opens detail view) |
| `Esc` | Back to previous screen |
| `Tab` | Cycle panels (match detail: cycle games) |
| `j/k` | Scroll up/down |
| `s` | Toggle favorite |
| `/` | Search |
| `f` | Cycle filter (All / Live / Upcoming / Favorites) |
| `r` | Force refresh |
| `b` | Toggle broadcast mode |
| `t` | Tournament browser |
| `o` | Open stream URL |
| `,` | Settings |
| `q` | Quit |

## Broadcast mode

Press `b` for a fullscreen ESPN-style view:
- Scrolling ticker with all live/upcoming matches
- Center stage featured match (prefers your favorite teams)
- Side rail with other matches
- Animated tournament countdown gauge

## Config

`~/.config/dota-tui/config.toml` (created on first run)

```toml
refresh_interval = 120
# pandascore_api_key = "your-key-here"
favorite_teams = ["Team Liquid"]
favorite_tournaments = []
enable_notifications = false
```

## Notifications

Compile with `--features notifications` to get desktop alerts for:
- Favorite team match starting in 15 minutes
- Favorite team match going live
- Tournament starting today

Set `enable_notifications = true` in config.

## Data sources

**Liquipedia** (default) — no API key, parses the wiki. Refreshes every 120s to respect rate limits.

**PandaScore** (optional) — set `pandascore_api_key` for structured data. Free tier at [pandascore.co](https://pandascore.co).

## License

MIT
