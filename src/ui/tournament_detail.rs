use crate::app::App;
use crate::ui::widgets::{keybind_bar, match_card};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let tournament = app
        .selected_tournament
        .and_then(|idx| app.upcoming_tournaments().get(idx).copied());

    let Some(tournament) = tournament else {
        frame.render_widget(
            Paragraph::new("No tournament selected. Press Esc to go back."),
            area,
        );
        return;
    };

    let layout = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(area);

    let status_text = match tournament.status {
        crate::models::TournamentStatus::Live => Span::styled(
            "● LIVE",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        crate::models::TournamentStatus::Upcoming => {
            if let Some((d, h, _, _)) = tournament.countdown() {
                Span::styled(
                    format!("Starts in {}d {}h", d, h),
                    Style::default().fg(Color::Yellow),
                )
            } else {
                Span::styled("Starting soon", Style::default().fg(Color::Yellow))
            }
        }
        crate::models::TournamentStatus::Completed => {
            Span::styled("Completed", Style::default().fg(Color::DarkGray))
        }
    };

    let date_range = format!(
        "{} - {}",
        tournament.start_date.format("%b %d, %Y"),
        tournament.end_date.format("%b %d, %Y")
    );
    let mut info_lines = vec![
        Line::from(vec![
            Span::styled(
                &tournament.name,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            status_text,
        ]),
        Line::from(Span::styled(
            format!("{} · {}", tournament.tier, date_range),
            Style::default().fg(Color::DarkGray),
        )),
    ];
    if let Some(ref loc) = tournament.location {
        info_lines.push(Line::from(Span::styled(
            format!("Location: {}", loc),
            Style::default().fg(Color::DarkGray),
        )));
    }
    if let Some(ref prize) = tournament.prize_pool {
        info_lines.push(Line::from(Span::styled(
            format!("Prize Pool: {}", prize),
            Style::default().fg(Color::DarkGray),
        )));
    }

    frame.render_widget(
        Paragraph::new(info_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .padding(Padding::horizontal(1)),
        ),
        layout[0],
    );

    let tournament_matches: Vec<_> = app
        .matches
        .iter()
        .filter(|m| {
            m.tournament_name == tournament.name || m.tournament_id == tournament.id
        })
        .collect();

    let matches_block = Block::default()
        .title(Span::styled(
            format!(" Matches ({}) ", tournament_matches.len()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));
    let matches_inner = matches_block.inner(layout[1]);
    frame.render_widget(matches_block, layout[1]);

    if tournament_matches.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No matches found",
                Style::default().fg(Color::DarkGray),
            )),
            matches_inner,
        );
    } else {
        for (i, m) in tournament_matches.iter().enumerate() {
            let y = matches_inner.y + i as u16;
            if y >= matches_inner.y + matches_inner.height {
                break;
            }
            match_card::render_match_card(
                m,
                Rect {
                    x: matches_inner.x,
                    y,
                    width: matches_inner.width,
                    height: 1,
                },
                frame.buffer_mut(),
                true,
                app.tick_count,
            );
        }
    }

    keybind_bar::render_keybind_bar(app, layout[2], frame.buffer_mut());
}
