# TODOS

## P1 — Stub Handlers

### Wire Favorite Toggle
**What:** Make `s` key actually toggle favorite teams/tournaments. `Config` already has `toggle_favorite_team()` and `toggle_favorite_tournament()` — just need to call them from `AppAction::ToggleFavorite` in `app.rs` based on the current context (selected match → toggle team, tournament view → toggle tournament). Persist on toggle via `config.save()`.
**Why:** Keybind is advertised in the status bar but does nothing. Users press `s` and nothing happens — erodes trust.
**Effort:** S (human: ~1 hour / CC: ~10 min)
**Depends on:** Nothing

### Wire Tournament Detail Views (Groups/Matches/Standings)
**What:** Make `g`/`m`/`d` keys in TournamentDetail screen actually switch between sub-views. Currently `ShowGroups`, `ShowMatches`, `ShowStandings` actions are no-ops. Need: active tab state on App, render different content in `tournament_detail.rs` per tab. Data may need enrichment from API (group tables, match lists per tournament).
**Why:** Keys are advertised and mapped but do nothing.
**Effort:** M (human: ~4 hours / CC: ~30 min)
**Depends on:** API data availability for group/standings info

### Wire Filter Functionality
**What:** Connect the `f` (filter) keybind to actual match/tournament filtering (by tier, region, status, etc.)
**Why:** Keybind advertised but does nothing
**Effort:** S (human: ~2 hours / CC: ~15 min)
**Depends on:** Nothing

## P1 — CI/CD

### GitHub Actions CI
**What:** Add `.github/workflows/test.yml` that runs `cargo test` on push + PR. Add `.github/workflows/release.yml` that builds cross-platform binaries and publishes to GitHub Releases on tag push.
**Why:** No CI at all right now. Tests only run locally. No automated release pipeline.
**Effort:** S (human: ~1 hour / CC: ~10 min)
**Depends on:** Nothing

## P2 — Next Round

### Tournament Bracket View
**What:** Render tournament brackets (single/double elimination) in TUI
**Why:** Brackets are the most visually impressive thing in esports coverage — the natural next feature after broadcast mode
**Context:** Requires research spike first. Liquipedia bracket data is embedded in Lua templates and may not be extractable via the `action=parse` API. PandaScore has cleaner bracket data but requires API key. Need to investigate data availability before committing to implementation.
**Effort:** M (human: ~1 week / CC: ~1-2 hours) after research spike
**Depends on:** Broadcast mode shipping, Liquipedia bracket data investigation

### Wire Search Functionality
**What:** Connect `search_active`/`search_query` fields (already on `App`) to UI rendering and match/tournament filtering
**Why:** The `/` keybind is advertised and mapped but does nothing — same trust-eroding issue as the stub handlers
**Context:** Fields exist in `app.rs`, `OpenSearch` action toggles `search_active`, but no search input UI is rendered and `search_query` never filters data. Need: text input widget, filter logic for matches/tournaments by name.
**Effort:** S (human: ~2 hours / CC: ~15 min)
**Depends on:** Nothing

## P3 — Polish

### AUR Package
**What:** Create an AUR package for `dota-2ui` so Arch users can install via `yay -S dota-2ui`
**Why:** Target audience (Dota players who are terminal nerds) heavily overlaps with Arch Linux users
**Context:** Depends on GitHub Releases CI being stable. PKGBUILD pulls prebuilt binary from GitHub Release. If notifications feature is desired, need `libdbus` as a dependency.
**Effort:** S (human: ~1 hour / CC: ~10 min)
**Depends on:** GitHub Releases CI

### DESIGN.md
**What:** Create a DESIGN.md documenting the visual language — color vocabulary, border conventions, spacing patterns, and the reasoning behind each
**Why:** `theme.rs` has the constants but no documentation of why Yellow = Team B, LightCyan = Upcoming, etc. Future contributors (or future-you) will wonder why colors were chosen
**Context:** Should be generated from the actual `theme.rs` after the broadcast mode ships. Include: color table with meanings, border style rules (double-line active, single-line inactive), the scoreboard layout spec, and the ultra-compact side rail format.
**Effort:** S (human: ~1 hour / CC: ~10 min)
**Depends on:** Broadcast mode shipping

### Structured Logging
**What:** Add optional file-based logging via `tracing` or `log` + `simplelog`
**Why:** Zero logging currently. When Liquipedia HTML changes or API errors occur, only signal is the UI error bar. File logs help debug user-reported issues.
**Context:** Log to `~/.cache/dota-tui/dota-tui.log` or similar. Use `RUST_LOG` env var for level control. Keep it optional — don't add overhead for users who don't need it.
**Effort:** S (human: ~2 hours / CC: ~15 min)
**Depends on:** Nothing
