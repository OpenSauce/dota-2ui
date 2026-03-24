mod api;
mod app;
mod cache;
mod config;
mod input;
mod models;
mod ui;

use app::App;
use cache::DiskCache;
use config::Config;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::{self, Write};
use std::time::Duration;

struct FetchResult {
    matches: Vec<models::Match>,
    tournaments: Vec<models::Tournament>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::load()?;
    let mut app = App::new(config);

    // Load cached data so we show something immediately
    let disk_cache = DiskCache::new(DiskCache::default_path());
    let cache_ttl = Duration::from_secs(app.config.refresh_interval * 2);
    if let Ok(Some(data)) = disk_cache.read("matches", cache_ttl) {
        if let Ok(matches) = serde_json::from_str::<Vec<models::Match>>(&data) {
            app.matches = matches;
        }
    }
    if let Ok(Some(data)) = disk_cache.read("tournaments", cache_ttl) {
        if let Ok(tournaments) = serde_json::from_str::<Vec<models::Tournament>>(&data) {
            app.tournaments = tournaments;
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app).await;

    // Restore terminal title on exit
    let _ = write!(terminal.backend_mut(), "\x1b]0;\x07");
    let _ = io::Write::flush(terminal.backend_mut());

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    let _ = app.config.save();
    result
}

fn sanitize_for_title(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).collect()
}

fn build_terminal_title(app: &App) -> String {
    // Find featured match (same logic as broadcast)
    let live_fav = app.matches.iter().find(|m| {
        m.status == models::MatchStatus::Live
            && app
                .config
                .favorite_teams
                .iter()
                .any(|fav| m.involves_team(fav))
    });
    let live_any = app
        .matches
        .iter()
        .find(|m| m.status == models::MatchStatus::Live);
    let upcoming = app
        .matches
        .iter()
        .filter(|m| m.status == models::MatchStatus::Upcoming)
        .min_by_key(|m| m.start_time);

    let featured = live_fav.or(live_any).or(upcoming);

    match featured {
        Some(m) => {
            let a = sanitize_for_title(&m.team_a.tag);
            let b = sanitize_for_title(&m.team_b.tag);
            let time = m.relative_time();
            if m.status == models::MatchStatus::Live {
                format!("{} {}:{} {} (LIVE) | dota-2ui", a, m.score_a, m.score_b, b)
            } else {
                format!("{} vs {} ({}) | dota-2ui", a, b, time)
            }
        }
        None => "dota-2ui".to_string(),
    }
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<FetchResult, String>>(2);
    let bracket_disk_cache = DiskCache::new(DiskCache::default_path().join("brackets"));
    let (bracket_tx, mut bracket_rx) =
        tokio::sync::mpsc::channel::<Result<(String, Option<models::Bracket>), String>>(2);

    loop {
        terminal.draw(|frame| ui::render(frame, app))?;
        app.tick_count = app.tick_count.wrapping_add(1);
        if app.broadcast_mode && app.tick_count.is_multiple_of(10) {
            app.ticker_offset = app.ticker_offset.wrapping_add(1);
        }

        // Update terminal title with featured match info
        {
            let title = build_terminal_title(app);
            let _ = write!(terminal.backend_mut(), "\x1b]0;{}\x07", title);
            let _ = io::Write::flush(terminal.backend_mut());
        }

        // Check notifications once per second (every 10 ticks), not every tick
        if app.tick_count.is_multiple_of(10) {
            let notifications = app.pending_notifications();
            for (_message, event) in &notifications {
                // Terminal bell for favorite team going live
                // MatchLive is already deduped by match ID in pending_notifications,
                // so the bell fires exactly once per match going live.
                if event == &app::NotificationEvent::MatchLive {
                    let _ = write!(terminal.backend_mut(), "\x07");
                    let _ = io::Write::flush(terminal.backend_mut());
                }

                #[cfg(feature = "notifications")]
                {
                    let _ = notify_rust::Notification::new()
                        .summary("Dota 2 TUI")
                        .body(_message)
                        .show();
                }
            }
        }

        if app.needs_refresh() && !app.is_loading {
            app.is_loading = true;
            app.mark_refreshed();

            let api_key = app.config.pandascore_api_key.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let provider = api::provider_from_config(api_key.as_deref());
                let result = provider
                    .fetch_all()
                    .await
                    .map(|r| FetchResult {
                        matches: r.matches,
                        tournaments: r.tournaments,
                    })
                    .map_err(|e| e.to_string());
                let _ = tx.send(result).await;
            });
        }

        while let Ok(result) = rx.try_recv() {
            match result {
                Ok(data) => {
                    // Save to disk cache
                    let disk_cache = DiskCache::new(DiskCache::default_path());
                    if let Ok(json) = serde_json::to_string(&data.matches) {
                        let _ = disk_cache.write("matches", &json);
                    }
                    if let Ok(json) = serde_json::to_string(&data.tournaments) {
                        let _ = disk_cache.write("tournaments", &json);
                    }
                    app.matches = data.matches;
                    app.tournaments = data.tournaments;
                    app.error_message = None;
                    // Prune stale notification keys for matches/tournaments no longer present
                    app.notified_events.retain(|key| {
                        app.matches
                            .iter()
                            .any(|m| m.id == key.match_or_tournament_id)
                            || app
                                .tournaments
                                .iter()
                                .any(|t| t.id == key.match_or_tournament_id)
                    });
                }
                Err(e) => {
                    app.error_message = Some(e);
                }
            }
            app.is_loading = false;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(action) = input::map_key(key, &app.screen, app.search_active) {
                        app.handle_action(action);
                    }
                }
            }
        }

        // Check if we need to fetch bracket data
        if app.screen == input::Screen::TournamentDetail
            && app.tournament_detail_tab == input::TournamentTab::Bracket
        {
            if let Some(ref tid) = app.selected_tournament_id {
                if !app.bracket_cache.contains_key(tid) && !app.bracket_attempted.contains(tid) {
                    let bracket_ttl = Duration::from_secs(600);
                    if let Ok(Some(data)) = bracket_disk_cache.read(&format!("bracket-{}", tid), bracket_ttl) {
                        if let Ok(bracket) = serde_json::from_str::<models::Bracket>(&data) {
                            app.bracket_cache.insert(tid.clone(), bracket);
                        }
                    }
                    if !app.bracket_cache.contains_key(tid) && !app.bracket_loading {
                        app.bracket_loading = true;
                        let tid_clone = tid.clone();
                        let api_key = app.config.pandascore_api_key.clone();
                        let bracket_tx = bracket_tx.clone();
                        let tournament_matches: Vec<models::Match> = app.matches.iter()
                            .filter(|m| m.tournament_id == tid_clone)
                            .cloned()
                            .collect();
                        tokio::spawn(async move {
                            let provider = api::provider_from_config(api_key.as_deref());
                            let result = provider
                                .fetch_bracket(&tid_clone)
                                .await
                                .map(|bracket| {
                                    let bracket = bracket.or_else(|| {
                                        api::liquipedia::LiquipediaProvider::build_stage_bracket(&tournament_matches)
                                    });
                                    (tid_clone, bracket)
                                })
                                .map_err(|e| e.to_string());
                            let _ = bracket_tx.send(result).await;
                        });
                    }
                }
            }
        }

        while let Ok(result) = bracket_rx.try_recv() {
            match result {
                Ok((tid, bracket)) => {
                    if let Some(bracket) = bracket {
                        if let Ok(json) = serde_json::to_string(&bracket) {
                            let _ = bracket_disk_cache.write(&format!("bracket-{}", tid), &json);
                        }
                        app.bracket_cache.insert(tid, bracket);
                    } else {
                        app.bracket_attempted.insert(tid);
                    }
                }
                Err(e) => {
                    app.error_message = Some(format!("Bracket fetch error: {}", e));
                }
            }
            app.bracket_loading = false;
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
