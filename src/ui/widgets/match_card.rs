use crate::models::{Match, MatchStatus};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Render a match card. If `compact` is true, renders on a single line (no tournament line).
pub fn render_match_card(m: &Match, area: Rect, buf: &mut Buffer, compact: bool) {
    let score_text = match m.status {
        MatchStatus::Upcoming => "vs".to_string(),
        _ => format!("{}:{}", m.score_a, m.score_b),
    };

    let score_style = match m.status {
        MatchStatus::Live => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        MatchStatus::Upcoming => Style::default().fg(Color::DarkGray),
        MatchStatus::Completed => Style::default().fg(Color::Gray),
    };

    let time_text = match m.status {
        MatchStatus::Live => {
            if let Some(secs) = m.game_time_secs {
                format!("{}:{:02}", secs / 60, secs % 60)
            } else {
                String::new()
            }
        }
        MatchStatus::Upcoming => m.start_time.format("%H:%M UTC").to_string(),
        MatchStatus::Completed => String::new(),
    };

    let mut spans = Vec::new();

    // Status indicator
    match m.status {
        MatchStatus::Live => spans.push(Span::styled(
            "LIVE ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        MatchStatus::Completed => spans.push(Span::styled(
            "END  ",
            Style::default().fg(Color::DarkGray),
        )),
        MatchStatus::Upcoming => spans.push(Span::raw("     ")),
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
        format!(" {:>3} ", m.series_format),
        Style::default().fg(Color::DarkGray),
    ));

    // Time
    if !time_text.is_empty() {
        spans.push(Span::styled(time_text, Style::default().fg(Color::DarkGray)));
    }

    let match_line = Line::from(spans);

    if compact || area.height < 2 {
        Paragraph::new(vec![match_line]).render(area, buf);
    } else {
        let tournament_line = Line::from(Span::styled(
            format!("      {}", m.tournament_name),
            Style::default().fg(Color::DarkGray),
        ));
        Paragraph::new(vec![match_line, tournament_line]).render(area, buf);
    }
}
