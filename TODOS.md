# TODOS

## P1 — High Priority

### AUR Package
**What:** Create an AUR package for `dota-2ui` so Arch users can install via `yay -S dota-2ui`
**Why:** Target audience (Dota players who are terminal nerds) heavily overlaps with Arch Linux users
**Context:** Depends on GitHub Releases CI being stable. PKGBUILD pulls prebuilt binary from GitHub Release. If notifications feature is desired, need `libdbus` as a dependency.
**Effort:** S (human: ~1 hour / CC: ~10 min)
**Depends on:** GitHub Releases CI (stable)

## P2 — Polish & Enhancement

### DESIGN.md
**What:** Create a DESIGN.md documenting the visual language — color vocabulary, border conventions, spacing patterns, and the reasoning behind each
**Why:** `theme.rs` has the constants but no documentation of why Yellow = Team B, LightCyan = Upcoming, etc. Future contributors (or future-you) will wonder why colors were chosen
**Context:** Should be generated from the actual `theme.rs` after the broadcast mode ships. Include: color table with meanings, border style rules (double-line active, single-line inactive), the scoreboard layout spec, and the ultra-compact side rail format.
**Effort:** S (human: ~1 hour / CC: ~10 min)
**Depends on:** Nothing

### Document Match Iteration Patterns
**What:** Add code comments documenting why broadcast/notifications iterate `app.matches` directly while dashboard uses accessor methods (which apply search/filter). The split is intentional — broadcast ignores filters by design.
**Why:** Maintenance hazard — future contributors may accidentally apply search/filter to broadcast.
**Effort:** S (human: ~15 min / CC: ~5 min)
**Depends on:** Nothing

### Recently Ended Matches on Dashboard
**What:** Show recently completed matches (last 1-2 hours) in a dedicated section or below the live panel on the dashboard. Currently completed matches only appear in the tournament detail Matches tab.
**Why:** During a tournament, you want to see what just happened — "OG beat Nigma 2-0" — without navigating to the tournament detail screen.
**Context:** Data already exists in `app.matches` with `MatchStatus::Completed`. Could add a "RECENT" panel or show completed matches in the live panel with "END" tag (which already renders). Filter to matches where `start_time` was within the last 2 hours.
**Effort:** S (human: ~1 hour / CC: ~10 min)
**Depends on:** Nothing

### Structured Logging
**What:** Add optional file-based logging via `tracing` or `log` + `simplelog`
**Why:** Zero logging currently. When Liquipedia HTML changes or API errors occur, only signal is the UI error bar. File logs help debug user-reported issues.
**Context:** Log to `~/.cache/dota-tui/dota-tui.log` or similar. Use `RUST_LOG` env var for level control. Keep it optional — don't add overhead for users who don't need it.
**Effort:** S (human: ~2 hours / CC: ~15 min)
**Depends on:** Nothing
