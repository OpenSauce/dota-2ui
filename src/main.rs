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
use std::io;
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

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    let _ = app.config.save();
    result
}

async fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<FetchResult, String>>(2);

    loop {
        terminal.draw(|frame| ui::render(frame, app))?;
        app.tick_count = app.tick_count.wrapping_add(1);

        if app.needs_refresh() && !app.is_loading {
            app.is_loading = true;
            app.mark_refreshed();

            let api_key = app.config.pandascore_api_key.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let provider = api::provider_from_config(api_key.as_deref());
                let result = provider.fetch_all().await
                    .map(|r| FetchResult { matches: r.matches, tournaments: r.tournaments })
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
                    if let Some(action) = input::map_key(key, &app.screen) {
                        app.handle_action(action);
                    }
                }
            }
        }

        if app.should_quit { break; }
    }
    Ok(())
}
