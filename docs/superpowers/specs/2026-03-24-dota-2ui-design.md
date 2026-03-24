# dota-2ui — Dota 2 Esports TUI Dashboard

A terminal-based dashboard for tracking Dota 2 esports tournaments, live matches, and fixtures. Built with Rust and ratatui. Designed as an Arch rice-friendly desktop widget that runs in any terminal.

## Problem

Keeping up with Dota 2 esports requires checking Liquipedia or third-party sites. There's no lightweight, always-on desktop widget for tracking fixtures, live matches, and tournament countdowns. The goal is a glanceable TUI that answers: "When's the next tournament?", "What matches are live?", and "What's the schedule for the tournament I'm following?"

## Data Sources

**Liquipedia API (default):** No API key required — just a custom `User-Agent` header. Rate limited to 1 request per 2 seconds. Free, publicly accessible, most complete esports data source. Data is wiki-structured so parsing requires more work.

**PandaScore API (optional upgrade):** Requires a free API key. Rate limited to 1,000 requests/hour on free tier. Clean REST endpoints purpose-built for match data. Structured JSON responses.

**Strategy:** Liquipedia by default for zero-friction setup. If user provides a PandaScore API key in config, prefer PandaScore. Both backends implement a unified `MatchProvider` trait so the rest of the app is source-agnostic.

## Architecture

**Single-threaded async (tokio).** One async runtime handles the TUI event loop and API polling. Ratatui renders on a 1-second tick interval (for countdown timers) and on input events. API calls run in background tasks via `tokio::spawn`. API responses are cached to disk (`~/.cache/dota-tui/`) with TTL to avoid redundant requests on restart.

**Config persistence:** `~/.config/dota-tui/config.toml` stores settings, favorite teams, and favorite tournaments.

## Data Models

- **Match** — two teams, score, status (live/upcoming/completed), series format (Bo1/Bo2/Bo3/Bo5), tournament reference, start time, stream URLs (opened in default browser via `o` key)
- **Tournament** — name, dates, status (upcoming/live/completed), tier, location, prize pool, groups/standings
- **Team** — name, tag, region

## Screens

### Dashboard (Home)

The default view. Tiled layout with live matches as full-width hero at the top, and a 2-column grid below.

```
┌─ LIVE (full width) ───────────────────────────────────────────────────────┐
│  ESL One Birmingham · Bo2                Stampede Champions · Bo3         │
│  ► Team Liquid  1:0  Gaimin Glad.        Tundra  2:1  Entity             │
│                      Game 2 · 24:31                   Game 3 · 12:07     │
├─ UPCOMING ─────────────────────┬─ TOURNAMENTS ────────────────────────────┤
│  15:00  VP vs TSpirit          │  ● LIVE  ESL One Birmingham    Mar 22-30│
│         ESL Birmingham · Bo2   │  5d 12h  DreamLeague S25       Apr 01   │
│  17:30  NGX vs XG              │  28d     TI 2026 Quals         Apr 22   │
│         ESL Birmingham · Bo2   │──────────────────────────────────────────│
│  20:00  Xtreme Gaming vs BB   │  FAVORITES                               │
│         ESL Birmingham · Bo2   │  Team Liquid — LIVE vs Gaimin Glad.     │
│  Tmrw   Falcons vs Heroic     │  OG — LIVE vs Aurora                     │
├────────────────────────────────┴──────────────────────────────────────────┤
│  [t] Tournaments  [f] Filter  [r] Refresh  [/] Search  [?] Help  [q] Quit│
└───────────────────────────────────────────────────────────────────────────┘
```

- Live panel: full width, adapts height to number of live matches (min 3 rows)
- Bottom left: upcoming fixtures with times
- Bottom right: tournament countdowns (ticking) + favorites panel
- Footer: keybind bar
- Responsive: collapses to single-column below 80 cols

### Tournament Browser

Press `t` from dashboard. Searchable, filterable list of tournaments. Shows name, dates, status, tier, countdown. Enter to drill into detail.

### Tournament Detail

Full tournament view: metadata (dates, prize pool, location), group standings tables, today's matches, full schedule. Sub-navigation: `[g] Groups  [m] Matches  [d] Standings`. (Note: `s` is reserved for favorite toggle globally, so standings uses `d`.)

### Settings

In-app settings screen for refresh rate, PandaScore API key, and managing favorites. Changes persist to config file.

## Navigation & Keybinds

| Key | Action |
|-----|--------|
| `t` | Tournament browser |
| `f` | Filter popup (team, region, tier) |
| `/` | Search (fuzzy match across tournaments + teams) |
| `Enter` | Drill into selected item |
| `Esc` / `Backspace` | Back |
| `s` | Star/favorite toggle (context-dependent: team or tournament) |
| `r` | Force refresh |
| `j`/`k` or arrows | Scroll |
| `Tab` | Cycle focus between dashboard panels |
| `q` | Quit |

## Favorites

Two types of favorites, same keybind (`s`), context-dependent:

- **Favorite team:** "I always want to know when Liquid plays." Shows their next/current match in the favorites panel. Can filter fixtures to only favorited teams.
- **Favorite tournament:** "I'm following ESL Birmingham this week." Pins tournament to dashboard with prominent countdown/status.

## Refresh & Timers

- **API refresh:** configurable interval, default 60 seconds
- **Countdown tick:** 1-second local tick for tournament countdowns and live match game timers (no API call)
- **Manual refresh:** `r` key triggers immediate API fetch

## Project Structure

```
dota-2ui/
├── src/
│   ├── main.rs              — entry point, tokio runtime, arg parsing
│   ├── app.rs               — app state machine, event loop
│   ├── config.rs            — config loading/saving (toml)
│   ├── api/
│   │   ├── mod.rs           — MatchProvider trait
│   │   ├── liquipedia.rs    — Liquipedia implementation
│   │   └── pandascore.rs    — PandaScore implementation
│   ├── cache.rs             — disk cache with TTL
│   ├── models.rs            — Match, Tournament, Team types
│   ├── ui/
│   │   ├── mod.rs           — render dispatch
│   │   ├── dashboard.rs     — tiled home view
│   │   ├── tournament_browser.rs
│   │   ├── tournament_detail.rs
│   │   ├── settings.rs
│   │   └── widgets/         — reusable components (match card, countdown, etc.)
│   └── input.rs             — keybind handling
├── Cargo.toml
└── config.example.toml
```

## Dependencies

- `ratatui` + `crossterm` — TUI rendering
- `tokio` — async runtime
- `reqwest` — HTTP client (gzip support)
- `serde` + `serde_json` + `toml` — serialization
- `chrono` — time handling and countdowns
- `clap` — CLI args
- `dirs` — XDG paths

## Name

**dota-2ui** — Dota 2 + TUI.
