# dota-2ui

TUI dashboard for Dota 2 esports. Live matches, fixtures, tournament countdowns.

## Install

```bash
cargo install --git https://github.com/OpenSauce/dota-2ui
```

## Keybinds

| Key | Action |
|-----|--------|
| `t` | Tournaments |
| `s` | Favorite team/tournament |
| `o` | Open stream |
| `r` | Refresh |
| `j/k` | Scroll |
| `Tab` | Cycle panels |
| `Enter` | Select |
| `Esc` | Back |
| `q` | Quit |

## Config

`~/.config/dota-tui/config.toml` (created on first run)

```toml
refresh_interval = 120
# pandascore_api_key = "your-key-here"
favorite_teams = ["Team Liquid"]
favorite_tournaments = []
```

## Data sources

**Liquipedia** (default) — no API key, parses the wiki. Refreshes every 120s to respect rate limits.

**PandaScore** (optional) — set `pandascore_api_key` for structured data. Free tier at [pandascore.co](https://pandascore.co).

## License

MIT
