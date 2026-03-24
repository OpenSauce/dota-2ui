use crate::app::App;
use crate::input::Screen;
use ratatui::prelude::*;

pub fn render_keybind_bar(app: &App, area: Rect, buf: &mut Buffer) {
    let screen = &app.screen;

    let binds: Vec<(&str, &str)> = match screen {
        Screen::Dashboard => vec![
            ("b", "Broadcast"), ("t", "Tourn."), ("r", "Refresh"),
            (",", "Settings"), ("q", "Quit"),
        ],
        Screen::TournamentBrowser => vec![
            ("j/k", "Navigate"), ("Enter", "Select"), ("Esc", "Back"),
        ],
        Screen::TournamentDetail => vec![
            ("Esc", "Back"),
        ],
        Screen::Settings => vec![
            ("Esc", "Back"),
        ],
        Screen::Broadcast => vec![
            ("b", "Dashboard"), ("q", "Quit"),
        ],
    };

    let mut spans: Vec<Span> = Vec::new();

    if *screen == Screen::Dashboard {
        spans.push(Span::styled(
            format!(" {}L ", app.live_matches().len()),
            Style::default().fg(Color::Green),
        ));
        spans.push(Span::styled(
            format!("{}U ", app.upcoming_matches().len()),
            Style::default().fg(Color::Yellow),
        ));
        spans.push(Span::styled(
            format!("{} {} ", app.data_source(), app.last_refresh_display()),
            Style::default().fg(Color::DarkGray),
        ));
        spans.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));
    }

    for (i, (key, desc)) in binds.iter().enumerate() {
        spans.push(Span::styled(
            format!("[{}]", key),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {} ", desc),
            Style::default().fg(Color::DarkGray),
        ));
        if i < binds.len() - 1 {
            spans.push(Span::raw(" "));
        }
    }

    ratatui::widgets::Paragraph::new(vec![Line::from(spans)])
        .alignment(Alignment::Center)
        .render(area, buf);
}
