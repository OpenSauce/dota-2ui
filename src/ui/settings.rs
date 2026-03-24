use crate::app::App;
use crate::ui::widgets::keybind_bar;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(area);

    frame.render_widget(
        Paragraph::new(Span::styled(
            " Settings",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        ),
        layout[0],
    );

    let api_display = match &app.config.pandascore_api_key {
        Some(key) => format!("{}...", &key[..key.len().min(8)]),
        None => "Not set (using Liquipedia)".to_string(),
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Refresh Interval: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}s", app.config.refresh_interval),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                "  PandaScore API Key: ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(api_display, Style::default().fg(Color::White)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Favorite Teams: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if app.config.favorite_teams.is_empty() {
                    "None".to_string()
                } else {
                    app.config.favorite_teams.join(", ")
                },
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                "  Favorite Tournaments: ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                if app.config.favorite_tournaments.is_empty() {
                    "None".to_string()
                } else {
                    app.config.favorite_tournaments.join(", ")
                },
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "  Config: ~/.config/dota-tui/config.toml",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .padding(Padding::horizontal(1)),
        ),
        layout[1],
    );

    keybind_bar::render_keybind_bar(&app.screen, layout[2], frame.buffer_mut());
}
