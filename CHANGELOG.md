# Changelog

All notable changes to this project will be documented in this file.

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
