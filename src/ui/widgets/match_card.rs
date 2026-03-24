use crate::models::{Match, MatchStatus};
use chrono::Local;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Render a match card. If `compact` is true, renders on a single line (no tournament line).
pub fn render_match_card(
    m: &Match,
    area: Rect,
    buf: &mut Buffer,
    compact: bool,
    tick_count: u64,
    favorite_teams: &[String],
) {
    let score_text = match m.status {
        MatchStatus::Upcoming => "vs".to_string(),
        _ => format!("{}:{}", m.score_a, m.score_b),
    };

    let score_style = match m.status {
        MatchStatus::Live => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        MatchStatus::Upcoming => Style::default().fg(Color::DarkGray),
        MatchStatus::Completed => Style::default().fg(Color::Gray),
    };

    let is_favorite = favorite_teams.iter().any(|fav| m.involves_team(fav));

    let mut spans = Vec::new();

    // Status indicator (fixed 5 chars)
    match m.status {
        MatchStatus::Live => {
            let blink_on = (tick_count / 5).is_multiple_of(2);
            if blink_on {
                spans.push(Span::styled(
                    "LIVE ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled("LIVE ", Style::default().fg(Color::DarkGray)));
            }
        }
        MatchStatus::Completed => {
            spans.push(Span::styled("END  ", Style::default().fg(Color::DarkGray)))
        }
        MatchStatus::Upcoming => spans.push(Span::raw("     ")),
    }

    // Favorite star — fixed 2-char column so alignment is consistent
    if is_favorite {
        spans.push(Span::styled("★ ", Style::default().fg(Color::Yellow)));
    } else {
        spans.push(Span::raw("  "));
    }

    // Team A (right-aligned in a fixed width)
    let tag_a = format!("{:>12}", m.team_a.tag);
    spans.push(Span::styled(tag_a, Style::default().fg(Color::Cyan)));

    spans.push(Span::raw(" "));
    spans.push(Span::styled(format!("{:^5}", score_text), score_style));
    spans.push(Span::raw(" "));

    // Team B
    let tag_b = format!("{:<12}", m.team_b.tag);
    spans.push(Span::styled(tag_b, Style::default().fg(Color::Yellow)));

    // Format
    spans.push(Span::styled(
        format!(" {:>3}", m.series_format),
        Style::default().fg(Color::DarkGray),
    ));

    // Time info after format
    match m.status {
        MatchStatus::Upcoming => {
            // Show both relative and local time: "in 13h 56m (14:00)"
            let rel = m.relative_time();
            let local_time = m.start_time.with_timezone(&Local);
            let abs = local_time.format("%H:%M").to_string();
            spans.push(Span::styled(
                format!(" {}", rel),
                Style::default().fg(m.urgency_color()),
            ));
            spans.push(Span::styled(
                format!(" ({})", abs),
                Style::default().fg(Color::DarkGray),
            ));
        }
        MatchStatus::Live => {
            spans.push(Span::styled(
                " LIVE".to_string(),
                Style::default().fg(Color::Red),
            ));
            if let Some(secs) = m.game_time_secs {
                spans.push(Span::styled(
                    format!(" {}:{:02}", secs / 60, secs % 60),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }
        MatchStatus::Completed => {}
    }

    let match_line = Line::from(spans);

    if compact || area.height < 2 {
        Paragraph::new(vec![match_line]).render(area, buf);
    } else {
        let stage_text = match &m.stage {
            Some(s) => format!("      {} · {}", m.tournament_name, s),
            None => format!("      {}", m.tournament_name),
        };
        let tournament_line = Line::from(Span::styled(
            stage_text,
            Style::default().fg(Color::DarkGray),
        ));
        Paragraph::new(vec![match_line, tournament_line]).render(area, buf);
    }
}
