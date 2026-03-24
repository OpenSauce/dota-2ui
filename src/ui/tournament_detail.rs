use crate::app::App;
use crate::input::TournamentTab;
use crate::models::TournamentStatus;
use crate::ui::widgets::{keybind_bar, match_card};
use chrono::Local;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let tournament = app
        .selected_tournament_id
        .as_ref()
        .and_then(|id| app.tournaments.iter().find(|t| &t.id == id));

    let Some(tournament) = tournament else {
        frame.render_widget(
            Paragraph::new("No tournament selected. Press Esc to go back."),
            area,
        );
        return;
    };

    let layout = Layout::vertical([
        Constraint::Length(1), // Tab bar
        Constraint::Length(5), // Header
        Constraint::Min(5),    // Content
        Constraint::Length(1), // Keybind bar
    ])
    .split(area);

    // Tab bar
    render_tab_bar(frame, app.tournament_detail_tab, layout[0]);

    // Header with tournament info
    let status_text = match tournament.status {
        TournamentStatus::Live => Span::styled(
            "● LIVE",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        TournamentStatus::Upcoming => {
            if let Some((d, h, _, _)) = tournament.countdown() {
                Span::styled(
                    format!("Starts in {}d {}h", d, h),
                    Style::default().fg(Color::Yellow),
                )
            } else {
                Span::styled("Starting soon", Style::default().fg(Color::Yellow))
            }
        }
        TournamentStatus::Completed => {
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
        layout[1],
    );

    // Content area based on active tab
    match app.tournament_detail_tab {
        TournamentTab::Overview => {
            // Overview: live + upcoming matches only (the default glanceable view)
            render_matches_tab(frame, app, tournament, layout[2], false);
        }
        TournamentTab::Matches => {
            // Matches: ALL matches including completed
            render_matches_tab(frame, app, tournament, layout[2], true);
        }
        TournamentTab::Bracket => {
            render_bracket_tab(frame, app, tournament, layout[2]);
        }
        TournamentTab::Info => {
            render_info_tab(frame, tournament, layout[2]);
        }
    }

    keybind_bar::render_keybind_bar(app, layout[3], frame.buffer_mut());
}

fn render_tab_bar(frame: &mut Frame, active: TournamentTab, area: Rect) {
    let tabs = [
        ("Overview", TournamentTab::Overview),
        ("Matches", TournamentTab::Matches),
        ("Bracket", TournamentTab::Bracket),
        ("Info", TournamentTab::Info),
    ];
    let spans: Vec<Span> = tabs
        .iter()
        .map(|(label, tab)| {
            if *tab == active {
                Span::styled(
                    format!(" [{}] ", label),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!("  {}  ", label),
                    Style::default().fg(Color::DarkGray),
                )
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_matches_tab(
    frame: &mut Frame,
    app: &App,
    tournament: &crate::models::Tournament,
    area: Rect,
    include_completed: bool,
) {
    let tournament_matches: Vec<_> = app
        .matches
        .iter()
        .filter(|m| m.tournament_name == tournament.name || m.tournament_id == tournament.id)
        .filter(|m| include_completed || m.status != crate::models::MatchStatus::Completed)
        .collect();

    let title = format!(" Matches ({}) ", tournament_matches.len());
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if tournament_matches.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No matches found",
                Style::default().fg(Color::DarkGray),
            )),
            inner,
        );
    } else {
        for (i, m) in tournament_matches.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }
            match_card::render_match_card(
                m,
                Rect {
                    x: inner.x,
                    y,
                    width: inner.width,
                    height: 1,
                },
                frame.buffer_mut(),
                true,
                app.tick_count,
                &app.config.favorite_teams,
            );
        }
    }
}

fn render_info_tab(frame: &mut Frame, tournament: &crate::models::Tournament, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Info ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
            match tournament.status {
                TournamentStatus::Live => Span::styled("LIVE", Style::default().fg(Color::Green)),
                TournamentStatus::Upcoming => {
                    Span::styled("Upcoming", Style::default().fg(Color::Yellow))
                }
                TournamentStatus::Completed => {
                    Span::styled("Completed", Style::default().fg(Color::DarkGray))
                }
            },
        ]),
        Line::from(vec![
            Span::styled("Start: ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                tournament
                    .start_date
                    .with_timezone(&Local)
                    .format("%b %d, %Y %H:%M")
                    .to_string(),
            ),
        ]),
        Line::from(vec![
            Span::styled("End: ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                tournament
                    .end_date
                    .with_timezone(&Local)
                    .format("%b %d, %Y %H:%M")
                    .to_string(),
            ),
        ]),
    ];
    if !tournament.tier.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Tier: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &tournament.tier,
                Style::default().fg(tournament.tier_color()),
            ),
        ]));
    }
    if let Some(ref prize) = tournament.prize_pool {
        lines.push(Line::from(vec![
            Span::styled("Prize Pool: ", Style::default().fg(Color::DarkGray)),
            Span::styled(prize, Style::default().fg(Color::Green)),
        ]));
    }
    if let Some(ref loc) = tournament.location {
        lines.push(Line::from(vec![
            Span::styled("Location: ", Style::default().fg(Color::DarkGray)),
            Span::raw(loc.as_str()),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_bracket_tab(
    frame: &mut Frame,
    app: &App,
    tournament: &crate::models::Tournament,
    area: Rect,
) {
    use crate::ui::widgets::bracket_column;

    if let Some(bracket) = app.bracket_cache.get(&tournament.id) {
        match app.bracket_view_mode {
            crate::models::BracketViewMode::Column => {
                bracket_column::render_column_bracket(
                    bracket,
                    area,
                    frame.buffer_mut(),
                    app.bracket_round_offset,
                    app.bracket_match_offset,
                    &app.config.favorite_teams,
                );
            }
            crate::models::BracketViewMode::AsciiTree => {
                use crate::ui::widgets::bracket_tree;
                bracket_tree::render_tree_bracket(
                    bracket,
                    area,
                    frame.buffer_mut(),
                    app.bracket_round_offset,
                    app.bracket_match_offset,
                    &app.config.favorite_teams,
                );
            }
        }
    } else if app.bracket_loading {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "Loading bracket...",
                Style::default().fg(Color::Yellow),
            )),
            area,
        );
    } else {
        let has_pandascore = app.config.pandascore_api_key.is_some();
        let msg = if has_pandascore {
            "No bracket data available for this tournament."
        } else {
            "Full brackets require a PandaScore API key. Configure in Settings (,)."
        };
        let split = Layout::vertical([Constraint::Length(2), Constraint::Min(3)]).split(area);
        frame.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(Color::DarkGray))),
            split[0],
        );
        render_matches_tab(frame, app, tournament, split[1], true);
    }
}
