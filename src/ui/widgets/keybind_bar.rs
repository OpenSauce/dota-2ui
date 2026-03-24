use crate::input::Screen;
use ratatui::prelude::*;

pub fn render_keybind_bar(screen: &Screen, area: Rect, buf: &mut Buffer) {
    let binds = match screen {
        Screen::Dashboard => vec![
            ("t", "Tournaments"),
            ("f", "Filter"),
            ("r", "Refresh"),
            ("/", "Search"),
            ("s", "Favorite"),
            ("o", "Stream"),
            ("q", "Quit"),
        ],
        Screen::TournamentBrowser => vec![
            ("↑↓", "Navigate"),
            ("Enter", "Select"),
            ("/", "Search"),
            ("Esc", "Back"),
            ("q", "Quit"),
        ],
        Screen::TournamentDetail => vec![
            ("g", "Groups"),
            ("m", "Matches"),
            ("d", "Standings"),
            ("s", "Favorite"),
            ("Esc", "Back"),
            ("q", "Quit"),
        ],
        Screen::Settings => vec![
            ("↑↓", "Navigate"),
            ("Enter", "Edit"),
            ("Esc", "Back"),
            ("q", "Quit"),
        ],
    };

    let spans: Vec<Span> = binds
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut v = vec![
                Span::styled(
                    format!("[{}]", key),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} ", desc),
                    Style::default().fg(Color::DarkGray),
                ),
            ];
            if i < binds.len() - 1 {
                v.push(Span::raw(" "));
            }
            v
        })
        .collect();

    ratatui::widgets::Paragraph::new(vec![Line::from(spans)])
        .alignment(Alignment::Center)
        .render(area, buf);
}
