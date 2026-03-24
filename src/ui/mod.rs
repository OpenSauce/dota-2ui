pub mod broadcast;
pub mod dashboard;
pub mod tournament_browser;
pub mod tournament_detail;
pub mod settings;
pub mod widgets;

use crate::app::App;
use ratatui::prelude::*;

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        crate::input::Screen::Dashboard => dashboard::render(frame, app),
        crate::input::Screen::TournamentBrowser => tournament_browser::render(frame, app),
        crate::input::Screen::TournamentDetail => tournament_detail::render(frame, app),
        crate::input::Screen::Settings => settings::render(frame, app),
        crate::input::Screen::Broadcast => broadcast::render(frame, app),
    }
}
