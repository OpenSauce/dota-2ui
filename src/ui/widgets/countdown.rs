use crate::models::{Tournament, TournamentStatus};
use ratatui::prelude::*;

pub fn render_countdown(t: &Tournament, area: Rect, buf: &mut Buffer) {
    let status_span = match t.status {
        TournamentStatus::Live => Span::styled(
            "● LIVE ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        TournamentStatus::Upcoming => {
            if let Some((days, hours, mins, _)) = t.countdown() {
                let text = if days > 0 {
                    format!("{}d {}h ", days, hours)
                } else if hours > 0 {
                    format!("{}h {}m ", hours, mins)
                } else {
                    format!("{}m ", mins)
                };
                Span::styled(text, Style::default().fg(Color::Yellow))
            } else {
                Span::styled("Soon ", Style::default().fg(Color::Yellow))
            }
        }
        TournamentStatus::Completed => {
            Span::styled("Done ", Style::default().fg(Color::DarkGray))
        }
    };

    let date_range = format!(
        "{} - {}",
        t.start_date.format("%b %d"),
        t.end_date.format("%b %d")
    );

    let line = Line::from(vec![
        status_span,
        Span::styled(&t.name, Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled(date_range, Style::default().fg(Color::DarkGray)),
    ]);

    ratatui::widgets::Paragraph::new(vec![line]).render(area, buf);
}
