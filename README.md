# dota-2ui

A terminal dashboard for Dota 2 esports. Track live matches, upcoming fixtures, and tournament countdowns — right from your terminal.

Built with Rust and [ratatui](https://github.com/ratatui/ratatui). Designed to be a lightweight desktop widget for your rice.

## Install

```bash
cargo install --git https://github.com/OpenSauce/dota-2ui
```

## Usage

```bash
dota-2ui
```

### Keybinds

| Key | Action |
|-----|--------|
| `t` | Tournament browser |
| `f` | Filter |
| `r` | Refresh |
| `/` | Search |
| `s` | Favorite (team or tournament) |
| `o` | Open stream in browser |
| `,` | Settings |
| `Tab` | Cycle panels |
| `Enter` | Select / drill in |
| `Esc` | Back |
| `q` | Quit |

### Dashboard

The home screen shows a tiled layout:

- **Live matches** span the full width at the top
- **Upcoming fixtures** on the bottom left
- **Tournament countdowns** and **favorites** on the bottom right

Countdowns tick in real-time. The layout collapses to single-column on narrow terminals.

### Tournament Browser

Press `t` to browse tournaments. Scroll with `j`/`k`, press `Enter` to see full details including matches and group stages.

### Favorites

Press `s` to favorite a team or tournament:

- **Favorite teams** show their next match in the dashboard favorites panel
- **Favorite tournaments** get pinned with prominent countdowns

## Configuration

Config lives at `~/.config/dota-tui/config.toml`. Created automatically on first run.

```toml
refresh_interval = 60
# pandascore_api_key = "your-key-here"
favorite_teams = ["Team Liquid"]
favorite_tournaments = ["ESL One Birmingham 2026"]
```

### Data Sources

**Liquipedia** (default) — no API key needed. Rate limited to 1 req/2s.

**PandaScore** (optional) — set `pandascore_api_key` in config for cleaner data. Free tier: 1000 req/hour. Get a key at [pandascore.co](https://pandascore.co).

## License

MIT
