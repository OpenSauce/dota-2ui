use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, _app: &App) {
    frame.render_widget(Paragraph::new("Dashboard — loading..."), frame.area());
}
