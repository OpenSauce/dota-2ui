# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-03-24

### Added
- Relative timestamps on all matches ("in 20m", "in 1d 3h", "starting soon")
- Absolute local time alongside relative time for upcoming matches ("in 20m (14:00)")
- Color-coded urgency for countdowns (red <15m, yellow <2h, default otherwise)
- Favorite team toggle via `s` key with team picker dialog (j/k to select, Enter to confirm)
- Favorite tournament toggle via `s` key in tournament browser/detail
- Favorites panel now shows matches from both favorite teams AND favorite tournaments
- Search functionality (`/` key) — filter matches/tournaments by name in real-time
- Filter cycling (`f` key) — All / Live Only / Upcoming Only / Favorites Only
- Tournament detail tabs (Overview / Matches / Info) with `m` and `d` keys
- Tournament Info tab showing dates, tier, prize pool, location metadata
- Completed matches shown in tournament Matches tab
- Match stage info extracted from Liquipedia (Group Stage, Playoffs, etc.)
- "Up next" preview in broadcast center stage showing next upcoming match
- Terminal title updates with featured match info (visible in tmux tab)
- Terminal bell on favorite team going live (zero-dependency alert)
- Visual selection indicator in all dashboard panels (grid-aware for live panel)
- Toast messages for favorite toggle feedback (3-second auto-dismiss)
- Auto-detect terminal width for compact mode (<100 cols)
- GitHub Actions CI workflow (check, clippy, test, format)
- Makefile with `make lint`, `make test`, `make fmt` targets
- `Match::involves_team()` helper for consistent case-insensitive team matching
- `Match::relative_time()` and `Match::urgency_color()` model methods
- `MatchFilter` and `TournamentTab` enums for UI state
- 15 new unit tests (scroll clamping, filter cycling, search, favorites, picker, timestamps)

### Changed
- All times now displayed in local timezone instead of UTC
- Keybind bar shows all functional keybinds (s/Fav, //Search, f/Filter, m/Matches, d/Info)
- Keybind bar shows active filter and search query status
- Star indicator (★) uses fixed-width column for consistent alignment
- Favorite matching uses `involves_team()` (checks name AND tag, case-insensitive)
- `scroll_offset` now clamped to panel list length (prevents silent no-ops)
- Liquipedia status detection simplified (removed redundant branch)

### Fixed
- Pre-existing clippy warnings resolved (manual_clamp, manual_div_ceil, if_same_then_else, etc.)

## [0.2.0] - 2026-03-24

### Added
- Broadcast mode (`b` key) — fullscreen ESPN-style view with scrolling ticker, center stage featured match, side rail, and tournament countdown gauge
- Blinking LIVE indicator on match cards (toggles red/dim every ~0.5s)
- Color-coded tournament tiers (gold for Tier 1/Major, silver for Tier 2/Minor, white for Tier 3/Qualifier)
- Countdown gauge bars showing 24-hour proximity to tournament start
- Double-line borders on focused panels for visual clarity
- Enhanced status bar with live/upcoming match counts, data source indicator, and last refresh time
- Desktop notifications for favorite team matches (feature-gated behind `notifications` cargo feature)
  - Match starting in 15 minutes
  - Match going live
  - Tournament starting today
- Notification deduplication (max 1 per match per event type per session)
- `enable_notifications` config option in `config.toml`

### Changed
- Event loop poll interval reduced from 1s to 100ms for smooth animations
- Keybind bar now only shows keybinds that actually work (removed 7 stub keybind labels)

### Fixed
- Ticker scroll now uses character count instead of byte length (multibyte-safe)
- Featured match computed once per frame instead of twice
- Stale notification keys pruned on data refresh (prevents unbounded memory growth)
- Notification checks throttled to 1/sec instead of 10/sec

## [0.1.0] - 2026-03-22

### Added
- Initial release: Dota 2 esports TUI dashboard
- Live match display with scores and game time
- Upcoming match schedule
- Tournament browser with detail view
- Liquipedia data provider (HTML parsing)
- PandaScore data provider (optional, API key)
- Disk cache for offline display
- Configurable refresh interval
- Favorite teams tracking
- Stream URL opening
