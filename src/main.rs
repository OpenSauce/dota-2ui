mod api;
mod app;
mod cache;
mod config;
mod input;
mod models;
mod ui;

use app::App;
use config::Config;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;

enum FetchResult {
    Matches(Result<Vec<models::Match>, String>),
    Tournaments(Result<Vec<models::Tournament>, String>),
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::load()?;
    let mut app = App::new(config);

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
    let (tx, mut rx) = tokio::sync::mpsc::channel::<FetchResult>(4);

    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        // Kick off background fetch if needed
        if app.needs_refresh() && !app.is_loading {
            app.is_loading = true;
            app.mark_refreshed();

            let api_key = app.config.pandascore_api_key.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let provider = api::provider_from_config(api_key.as_deref());
                let matches = provider.fetch_matches().await.map_err(|e| e.to_string());
                let tournaments = provider.fetch_tournaments().await.map_err(|e| e.to_string());
                let _ = tx.send(FetchResult::Matches(matches)).await;
                let _ = tx.send(FetchResult::Tournaments(tournaments)).await;
            });
        }

        // Drain background fetch results
        while let Ok(result) = rx.try_recv() {
            match result {
                FetchResult::Matches(Ok(matches)) => {
                    app.matches = matches;
                    app.error_message = None;
                }
                FetchResult::Matches(Err(e)) => app.error_message = Some(e),
                FetchResult::Tournaments(Ok(tournaments)) => {
                    app.tournaments = tournaments;
                    app.is_loading = false;
                }
                FetchResult::Tournaments(Err(e)) => {
                    if app.error_message.is_none() {
                        app.error_message = Some(e);
                    }
                    app.is_loading = false;
                }
            }
        }

        // Handle input with 1-second tick
        if event::poll(Duration::from_secs(1))? {
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
