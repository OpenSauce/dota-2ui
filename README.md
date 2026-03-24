# dota-2ui

TUI dashboard for Dota 2 esports. Live matches, fixtures, tournament countdowns.

```
┌─ LIVE ─────────────────────────────────┬─ TOURNAMENTS ────────────────────────┐
│  ESL One Birmingham · Bo2              │  ● LIVE  ESL One Birmingham   Mar 22 │
│  ► Liquid    1:0  Gaimin Glad.         │  5d 12h  DreamLeague S25     Apr 01  │
│                    Game 2 · 24:31      │  28d     TI 2026 Quals       Apr 22  │
├─ UPCOMING ─────────────────────────────┤──────────────────────────────────────│
│  15:00  VP vs TSpirit · Bo2            │  FAVORITES                           │
│  17:30  NGX vs XG · Bo2               │  Team Liquid — LIVE vs Gaimin Glad.  │
│  20:00  Xtreme Gaming vs BB · Bo2     │  OG — LIVE vs Aurora                 │
└────────────────────────────────────────┴──────────────────────────────────────┘
```

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
