use crate::app::App;
use crate::input::{MatchFilter, Screen};
use ratatui::prelude::*;

pub fn render_keybind_bar(app: &App, area: Rect, buf: &mut Buffer) {
    // Toast message takes priority
    if let Some((ref msg, _)) = app.status_message {
        ratatui::widgets::Paragraph::new(Line::from(Span::styled(
            format!(" {} ", msg),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )))
        .alignment(Alignment::Center)
        .render(area, buf);
        return;
    }

    // Search bar takes priority over keybinds
    if app.search_active {
        ratatui::widgets::Paragraph::new(Line::from(Span::styled(
            format!(" / {}_ ", app.search_query),
            Style::default().fg(Color::Cyan),
        )))
        .render(area, buf);
        return;
    }

    let screen = &app.screen;

    let binds: Vec<(&str, &str)> = match screen {
        Screen::Dashboard => vec![
            ("s", "Fav"),
            ("/", "Search"),
            ("f", "Filter"),
            ("b", "Broadcast"),
            ("t", "Tourn."),
            ("r", "Refresh"),
            ("q", "Quit"),
        ],
        Screen::TournamentBrowser => vec![
            ("j/k", "Navigate"),
            ("s", "Fav"),
            ("Enter", "Select"),
            ("Esc", "Back"),
        ],
        Screen::TournamentDetail => vec![
            ("g", "Overview"),
            ("m", "Matches"),
            ("d", "Info"),
            ("s", "Fav"),
            ("Esc", "Back"),
        ],
        Screen::Settings => vec![("Esc", "Back")],
        Screen::Broadcast => vec![("b", "Dashboard"), ("q", "Quit")],
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

        // Show active filter if not All
        if app.active_filter != MatchFilter::All {
            spans.push(Span::styled(
                format!("│ {} ", app.active_filter.label()),
                Style::default().fg(Color::Magenta),
            ));
        }

        // Show active search query if present
        if !app.search_query.is_empty() && !app.search_active {
            spans.push(Span::styled(
                format!("│ /{}  ", app.search_query),
                Style::default().fg(Color::Cyan),
            ));
        }

        spans.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));
    }

    for (i, (key, desc)) in binds.iter().enumerate() {
        spans.push(Span::styled(
            format!("[{}]", key),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
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
