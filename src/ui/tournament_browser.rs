use crate::app::App;
use crate::ui::widgets::keybind_bar;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Padding, Paragraph};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(area);

    let mode_label = if app.show_all_tournaments {
        "All"
    } else {
        "Upcoming"
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Tournament Browser",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  ({} {} tournaments)",
                app.browsable_tournaments().len(),
                mode_label
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(header, layout[0]);

    let tournaments = app.browsable_tournaments();
    let items: Vec<ListItem> = tournaments
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let status = match t.status {
                crate::models::TournamentStatus::Live => {
                    Span::styled("● LIVE ", Style::default().fg(Color::Green))
                }
                crate::models::TournamentStatus::Upcoming => {
                    if let Some((days, hours, _, _)) = t.countdown() {
                        let text = if days > 0 {
                            format!("{}d ", days)
                        } else {
                            format!("{}h ", hours)
                        };
                        Span::styled(text, Style::default().fg(Color::Yellow))
                    } else {
                        Span::styled("Soon ", Style::default().fg(Color::Yellow))
                    }
                }
                crate::models::TournamentStatus::Completed => {
                    Span::styled("Done ", Style::default().fg(Color::DarkGray))
                }
            };

            let is_fav = app.config.favorite_tournaments.iter().any(|f| f == &t.name);
            let fav = if is_fav {
                Span::styled("★ ", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("  ")
            };

            let info = format!(
                "{}  {} - {}  {}",
                t.tier,
                t.start_date.format("%b %d"),
                t.end_date.format("%b %d"),
                t.location.as_deref().unwrap_or("")
            );

            let style = if i == app.scroll_offset {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(vec![
                Line::from(vec![
                    fav,
                    status,
                    Span::styled(
                        &t.name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(info, Style::default().fg(Color::DarkGray)),
                ]),
            ])
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1)),
    );
    frame.render_widget(list, layout[1]);
    keybind_bar::render_keybind_bar(app, layout[2], frame.buffer_mut());
}
