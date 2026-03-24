use crate::app::App;
use crate::models::{MatchStatus, Match};
use crate::ui::widgets::match_card;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

/// Build the scrolling ticker string from live and upcoming matches.
fn build_ticker(app: &App) -> String {
    let mut parts: Vec<String> = Vec::new();

    for m in &app.matches {
        match m.status {
            MatchStatus::Live => {
                parts.push(format!(
                    "{} {} {}:{} {} (LIVE)",
                    m.team_a.tag, m.series_format, m.score_a, m.score_b, m.team_b.tag
                ));
            }
            MatchStatus::Upcoming => {
                let time_str = m.start_time.format("%H:%M").to_string();
                parts.push(format!(
                    "{} vs {} ({} UTC)",
                    m.team_a.tag, m.team_b.tag, time_str
                ));
            }
            _ => {}
        }
    }

    let mut ticker = if parts.is_empty() {
        // Fallback: show next tournament countdown or generic message
        if let Some(t) = app.upcoming_tournaments().first() {
            if let Some((d, h, m, _s)) = t.countdown() {
                format!("  No live matches -- next: {} in {}d {}h {}m  ", t.name, d, h, m)
            } else {
                "  No live matches  ".to_string()
            }
        } else {
            "  No live matches  ".to_string()
        }
    } else {
        format!("  {}  ", parts.join("  |  "))
    };

    // Append staleness indicator
    let elapsed_secs = app.last_refresh.elapsed().as_secs();
    let staleness = if elapsed_secs >= 60 {
        format!("(data {}m ago)", elapsed_secs / 60)
    } else {
        "(data <1m ago)".to_string()
    };
    ticker.push_str(&format!("  {}  ", staleness));

    ticker
}

/// Pick the featured match to display in center stage.
fn featured_match(app: &App) -> Option<usize> {
    let favorites = &app.config.favorite_teams;

    // 1. First live match involving a favorite team
    if !favorites.is_empty() {
        for (i, m) in app.matches.iter().enumerate() {
            if m.status == MatchStatus::Live {
                let involves_fav = favorites.iter().any(|fav| {
                    m.team_a.name.eq_ignore_ascii_case(fav)
                        || m.team_b.name.eq_ignore_ascii_case(fav)
                        || m.team_a.tag.eq_ignore_ascii_case(fav)
                        || m.team_b.tag.eq_ignore_ascii_case(fav)
                });
                if involves_fav {
                    return Some(i);
                }
            }
        }
    }

    // 2. First live match by start_time
    let mut live_indices: Vec<usize> = app
        .matches
        .iter()
        .enumerate()
        .filter(|(_, m)| m.status == MatchStatus::Live)
        .map(|(i, _)| i)
        .collect();
    live_indices.sort_by_key(|&i| app.matches[i].start_time);
    if let Some(&i) = live_indices.first() {
        return Some(i);
    }

    // 3. Next upcoming match by start_time
    let mut upcoming_indices: Vec<usize> = app
        .matches
        .iter()
        .enumerate()
        .filter(|(_, m)| m.status == MatchStatus::Upcoming)
        .map(|(i, _)| i)
        .collect();
    upcoming_indices.sort_by_key(|&i| app.matches[i].start_time);
    upcoming_indices.first().copied()
}

pub fn render(frame: &mut Frame, app: &App) {
    let outer = frame.area();

    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Length(3),
    ])
    .split(outer);

    let ticker_area = vertical[0];
    let main_area = vertical[1];
    let bottom_area = vertical[2];

    let horizontal = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ])
    .split(main_area);

    let center_area = horizontal[0];
    let side_area = horizontal[1];

    let featured_idx = featured_match(app);

    render_ticker(frame, app, ticker_area);
    render_center_stage(frame, app, center_area, featured_idx);
    render_side_rail(frame, app, side_area, featured_idx);
    render_bottom_countdown(frame, app, bottom_area);
}

fn render_ticker(frame: &mut Frame, app: &App, area: Rect) {
    let ticker = build_ticker(app);
    let len = ticker.chars().count().max(1);
    let offset = app.ticker_offset % len;
    let visible: String = ticker
        .chars()
        .cycle()
        .skip(offset)
        .take(area.width as usize)
        .collect();

    let paragraph = Paragraph::new(visible).style(
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray),
    );
    frame.render_widget(paragraph, area);
}

fn render_center_stage(frame: &mut Frame, app: &App, area: Rect, featured: Option<usize>) {
    let block = Block::default().borders(Borders::RIGHT);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let featured_idx = match featured {
        Some(i) => i,
        None => {
            let msg = Paragraph::new("No matches")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            // Center vertically
            let y = inner.y + inner.height / 2;
            let centered = Rect::new(inner.x, y, inner.width, 1);
            frame.render_widget(msg, centered);
            return;
        }
    };

    let m = &app.matches[featured_idx];

    // Build lines to render, then center them vertically
    let mut lines: Vec<Line> = Vec::new();

    // Tournament name
    lines.push(
        Line::from(Span::styled(
            &m.tournament_name,
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center),
    );

    // Status line
    let status_line = match m.status {
        MatchStatus::Live => {
            let blink_on = (app.tick_count / 5) % 2 == 0;
            let style = if blink_on {
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled("LIVE", style)).alignment(Alignment::Center)
        }
        MatchStatus::Upcoming => Line::from(Span::styled(
            "UPCOMING",
            Style::default().fg(Color::Yellow),
        ))
        .alignment(Alignment::Center),
        MatchStatus::Completed => Line::from(Span::styled(
            "COMPLETED",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center),
    };
    lines.push(status_line);

    // Blank line
    lines.push(Line::from(""));

    // Team A
    lines.push(
        Line::from(Span::styled(
            &m.team_a.name,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
    );

    // Score line
    let score_line = if m.status == MatchStatus::Upcoming {
        format!("vs  ({})", m.series_format)
    } else {
        format!("{}  :  {}  ({})", m.score_a, m.score_b, m.series_format)
    };
    lines.push(
        Line::from(Span::styled(
            score_line,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
    );

    // Team B
    lines.push(
        Line::from(Span::styled(
            &m.team_b.name,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
    );

    // Blank line
    lines.push(Line::from(""));

    // Time info
    let time_info = match m.status {
        MatchStatus::Live => {
            if let Some(secs) = m.game_time_secs {
                format!("Game time: {}:{:02}", secs / 60, secs % 60)
            } else {
                "In progress".to_string()
            }
        }
        MatchStatus::Upcoming => m.start_time.format("%b %d, %H:%M UTC").to_string(),
        MatchStatus::Completed => "Match completed".to_string(),
    };
    lines.push(
        Line::from(Span::styled(
            time_info,
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center),
    );

    let total_lines = lines.len() as u16;
    let start_y = if inner.height > total_lines {
        inner.y + (inner.height - total_lines) / 2
    } else {
        inner.y
    };

    let text_area = Rect::new(inner.x, start_y, inner.width, total_lines.min(inner.height));
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, text_area);
}

fn render_side_rail(frame: &mut Frame, app: &App, area: Rect, featured_idx: Option<usize>) {

    // Collect live + upcoming, excluding the featured match
    let mut rail_matches: Vec<&Match> = Vec::new();
    for (i, m) in app.matches.iter().enumerate() {
        if Some(i) == featured_idx {
            continue;
        }
        if m.status == MatchStatus::Live || m.status == MatchStatus::Upcoming {
            rail_matches.push(m);
        }
    }

    // Sort: live first, then by start_time
    rail_matches.sort_by(|a, b| {
        let a_live = a.status == MatchStatus::Live;
        let b_live = b.status == MatchStatus::Live;
        b_live.cmp(&a_live).then(a.start_time.cmp(&b.start_time))
    });

    // Small left padding
    let padded = if area.width > 1 {
        Rect::new(area.x + 1, area.y, area.width - 1, area.height)
    } else {
        area
    };

    let mut y = padded.y;
    let buf = frame.buffer_mut();
    for m in &rail_matches {
        if y >= padded.y + padded.height {
            break;
        }
        let row = Rect::new(padded.x, y, padded.width, 1);
        match_card::render_match_card(m, row, buf, true, app.tick_count);
        y += 1;
    }
}

fn render_bottom_countdown(frame: &mut Frame, app: &App, area: Rect) {
    let upcoming = app.upcoming_tournaments();
    let tournament = match upcoming.first() {
        Some(t) => *t,
        None => {
            let msg = Paragraph::new("No upcoming tournaments")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, area);
            return;
        }
    };

    if let Some((d, h, m, s)) = tournament.countdown() {
        let label = format!("{} starts in {}d {}h {}m {}s", tournament.name, d, h, m, s);
        let ratio = tournament.countdown_ratio();
        let color = if ratio > 0.8 {
            Color::Red
        } else if ratio > 0.5 {
            Color::Yellow
        } else {
            Color::Cyan
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::TOP))
            .gauge_style(Style::default().fg(color))
            .ratio(ratio.clamp(0.0, 1.0))
            .label(Span::styled(
                label,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        frame.render_widget(gauge, area);
    } else {
        let msg = Paragraph::new(format!("{} -- starting soon", tournament.name))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(msg, area);
    }
}
