use crate::app::App;
use crate::models::{FetchStatus, MatchStatus};
use crate::ui::widgets::keybind_bar::render_keybind_bar;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let main_layout = Layout::vertical([
        Constraint::Min(8),
        Constraint::Length(1), // keybind bar
    ])
    .split(area);

    let content_area = main_layout[0];

    // Look up the match from app.matches
    let m = app
        .selected_match_id
        .as_ref()
        .and_then(|mid| app.matches.iter().find(|m| m.id == *mid));

    let Some(m) = m else {
        let msg = Paragraph::new("Match not found")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(msg, content_area);
        render_keybind_bar(app, main_layout[1], frame.buffer_mut());
        return;
    };

    // Check detail cache
    let detail = app
        .selected_match_id
        .as_ref()
        .and_then(|mid| app.match_detail_cache.get(mid));

    // Determine fetch status — if no cache entry yet, show loading
    let fetch_status = detail
        .map(|(d, _)| &d.fetch_status)
        .unwrap_or(&FetchStatus::Loading);

    match fetch_status {
        FetchStatus::Loading => {
            render_loading(frame, app, m, content_area);
        }
        FetchStatus::Error(ref e) => {
            render_error(frame, app, m, content_area, e);
        }
        FetchStatus::Ready => {
            let games = detail.map(|(d, _)| &d.games);
            render_ready(frame, app, m, content_area, games);
        }
    }

    render_keybind_bar(app, main_layout[1], frame.buffer_mut());
}

fn render_header(m: &crate::models::Match, area: Rect, buf: &mut Buffer, tick_count: u64) {
    let status_str = match m.status {
        MatchStatus::Live => {
            let blink = (tick_count / 5).is_multiple_of(2);
            if blink {
                Span::styled(
                    " LIVE ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(" LIVE ", Style::default().fg(Color::DarkGray))
            }
        }
        MatchStatus::Completed => Span::styled(" END  ", Style::default().fg(Color::DarkGray)),
        MatchStatus::Upcoming => Span::styled(
            format!(" {}  ", m.relative_time()),
            Style::default().fg(Color::Yellow),
        ),
    };

    let header = Line::from(vec![
        Span::styled(
            format!("{:>16}", m.team_a.name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{}", m.score_a),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" : ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", m.score_b),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{:<16}", m.team_b.name),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        status_str,
        Span::raw("  "),
        Span::styled(
            m.series_format.to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let para = Paragraph::new(header).alignment(Alignment::Center);
    para.render(area, buf);
}

fn render_series_progress(
    games: Option<&Vec<crate::models::GameDetail>>,
    selected_game: usize,
    area: Rect,
    buf: &mut Buffer,
) {
    let Some(games) = games else {
        let para = Paragraph::new(Span::styled(
            "No per-game data available",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center);
        para.render(area, buf);
        return;
    };

    if games.is_empty() {
        let para = Paragraph::new(Span::styled(
            "No games played yet",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center);
        para.render(area, buf);
        return;
    }

    let mut spans = Vec::new();
    for (i, g) in games.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        }

        let is_selected = i == selected_game;
        let label = format!("Game {}", g.game_number);

        let status_text = match g.status {
            MatchStatus::Completed => {
                let winner = g.winner.as_deref().unwrap_or("?");
                format!("{}: {} win", label, winner)
            }
            MatchStatus::Live => format!("{}: LIVE", label),
            MatchStatus::Upcoming => format!("{}: --", label),
        };

        let style = if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            match g.status {
                MatchStatus::Completed => Style::default().fg(Color::Gray),
                MatchStatus::Live => Style::default().fg(Color::Green),
                MatchStatus::Upcoming => Style::default().fg(Color::DarkGray),
            }
        };

        spans.push(Span::styled(status_text, style));
    }

    let line = Line::from(spans);
    let para = Paragraph::new(line).alignment(Alignment::Center);
    para.render(area, buf);
}

fn render_match_info(m: &crate::models::Match, area: Rect, buf: &mut Buffer) {
    let mut lines = vec![
        Line::from(vec![
            Span::styled(" Tournament: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&m.tournament_name, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" Format:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                m.series_format.to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Start:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(m.relative_time(), Style::default().fg(Color::White)),
        ]),
    ];

    if let Some(ref stage) = m.stage {
        lines.insert(
            1,
            Line::from(vec![
                Span::styled(" Stage:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(stage, Style::default().fg(Color::White)),
            ]),
        );
    }

    if let Some(ref url) = m.stream_url {
        lines.push(Line::from(vec![
            Span::styled(" Stream:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(url, Style::default().fg(Color::LightCyan)),
        ]));
    }

    let para = Paragraph::new(lines);
    para.render(area, buf);
}

fn render_loading(frame: &mut Frame, app: &App, m: &crate::models::Match, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Match Detail ")
        .title_style(Style::default().fg(Color::Cyan))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let inner_layout = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(1), // spacer
        Constraint::Min(1),    // spinner
    ])
    .split(inner);

    render_header(m, inner_layout[0], frame.buffer_mut(), app.tick_count);

    let dots = match (app.tick_count / 5) % 4 {
        0 => ".",
        1 => "..",
        2 => "...",
        _ => "",
    };
    let spinner = Paragraph::new(format!("Loading match detail{}", dots))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(spinner, inner_layout[2]);
}

fn render_error(frame: &mut Frame, app: &App, m: &crate::models::Match, area: Rect, error: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Match Detail ")
        .title_style(Style::default().fg(Color::Cyan))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let inner_layout = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(1), // spacer
        Constraint::Length(1), // error
        Constraint::Length(1), // hint
        Constraint::Min(0),
    ])
    .split(inner);

    render_header(m, inner_layout[0], frame.buffer_mut(), app.tick_count);

    let err_msg = Paragraph::new(format!("Error: {}", error))
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center);
    frame.render_widget(err_msg, inner_layout[2]);

    let hint = Paragraph::new("Press [r] to retry")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(hint, inner_layout[3]);
}

fn render_ready(
    frame: &mut Frame,
    app: &App,
    m: &crate::models::Match,
    area: Rect,
    games: Option<&Vec<crate::models::GameDetail>>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Match Detail ")
        .title_style(Style::default().fg(Color::Cyan))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let inner_layout = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(1), // spacer
        Constraint::Length(1), // series progress
        Constraint::Length(1), // spacer
        Constraint::Length(1), // future HUD placeholder
        Constraint::Length(1), // spacer
        Constraint::Min(5),    // match info
    ])
    .split(inner);

    render_header(m, inner_layout[0], frame.buffer_mut(), app.tick_count);
    render_series_progress(
        games,
        app.selected_game,
        inner_layout[2],
        frame.buffer_mut(),
    );

    let placeholder = Paragraph::new(Span::styled(
        "── coming soon: add Stratz/OpenDota for live game data ──",
        Style::default().fg(Color::DarkGray),
    ))
    .alignment(Alignment::Center);
    frame.render_widget(placeholder, inner_layout[4]);

    render_match_info(m, inner_layout[6], frame.buffer_mut());
}
