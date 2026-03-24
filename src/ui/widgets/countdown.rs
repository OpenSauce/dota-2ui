use crate::models::{Tournament, TournamentStatus};
use ratatui::prelude::*;
use ratatui::widgets::Gauge;

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
        Span::styled(&t.name, Style::default().fg(t.tier_color())),
        Span::raw("  "),
        Span::styled(date_range, Style::default().fg(Color::DarkGray)),
    ]);

    ratatui::widgets::Paragraph::new(vec![line]).render(area, buf);
}

pub fn render_countdown_with_gauge(t: &Tournament, area: Rect, buf: &mut Buffer) {
    if area.height < 2 {
        render_countdown(t, area, buf);
        return;
    }
    let top = Rect { x: area.x, y: area.y, width: area.width, height: 1 };
    render_countdown(t, top, buf);

    if t.status == TournamentStatus::Upcoming {
        let gauge_area = Rect { x: area.x, y: area.y + 1, width: area.width, height: 1 };
        let ratio = t.countdown_ratio();
        let color = if ratio > 0.8 { Color::Red }
            else if ratio > 0.5 { Color::Yellow }
            else { Color::DarkGray };
        Gauge::default()
            .ratio(ratio)
            .gauge_style(Style::default().fg(color))
            .render(gauge_area, buf);
    }
}
