use crate::models::{Match, MatchStatus};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render_match_card(m: &Match, area: Rect, buf: &mut Buffer) {
    let status_style = match m.status {
        MatchStatus::Live => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        MatchStatus::Upcoming => Style::default().fg(Color::DarkGray),
        MatchStatus::Completed => Style::default().fg(Color::Gray),
    };

    let score_text = match m.status {
        MatchStatus::Upcoming => "vs".to_string(),
        _ => format!("{}:{}", m.score_a, m.score_b),
    };

    let time_text = match m.status {
        MatchStatus::Live => {
            if let Some(secs) = m.game_time_secs {
                format!("{}:{:02}", secs / 60, secs % 60)
            } else {
                "LIVE".to_string()
            }
        }
        MatchStatus::Upcoming => m.start_time.format("%H:%M UTC").to_string(),
        MatchStatus::Completed => "Final".to_string(),
    };

    let status_indicator = match m.status {
        MatchStatus::Live => Span::styled(
            "LIVE ",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        _ => Span::raw(""),
    };

    let line = Line::from(vec![
        status_indicator,
        Span::styled(&m.team_a.tag, Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(&score_text, status_style),
        Span::raw("  "),
        Span::styled(&m.team_b.tag, Style::default().fg(Color::Yellow)),
        Span::raw("   "),
        Span::styled(&time_text, Style::default().fg(Color::DarkGray)),
    ]);

    let tournament_line = Line::from(vec![Span::styled(
        format!("{} · {}", m.tournament_name, m.series_format),
        Style::default().fg(Color::DarkGray),
    )]);

    Paragraph::new(vec![line, tournament_line]).render(area, buf);
}
