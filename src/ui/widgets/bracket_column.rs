use crate::models::{Bracket, BracketMatch, BracketRound, MatchStatus};
use ratatui::prelude::*;

const COLUMN_WIDTH: u16 = 28;

pub fn render_column_bracket(
    bracket: &Bracket,
    area: Rect,
    buf: &mut Buffer,
    round_offset: usize,
    match_offset: usize,
    favorite_teams: &[String],
) {
    if area.width < 4 || area.height < 2 {
        return;
    }

    if bracket.upper_rounds.is_empty() {
        let msg = Span::styled("No bracket rounds available.", Style::default().fg(Color::DarkGray));
        buf.set_line(area.x, area.y, &Line::from(msg), area.width);
        return;
    }

    if let Some(ref lower_rounds) = bracket.lower_rounds {
        if !lower_rounds.is_empty() {
            // Double elimination: split area in half
            let upper_height = (area.height.saturating_sub(1)) / 2;
            let divider_y = area.y + upper_height;
            let lower_height = area.height.saturating_sub(upper_height + 1);

            let upper_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: upper_height,
            };
            let lower_area = Rect {
                x: area.x,
                y: divider_y + 1,
                width: area.width,
                height: lower_height,
            };

            render_rounds(
                &bracket.upper_rounds,
                upper_area,
                buf,
                round_offset,
                match_offset,
                favorite_teams,
            );

            // Divider
            render_divider("LOWER BRACKET", divider_y, area.x, area.width, buf);

            render_rounds(
                lower_rounds,
                lower_area,
                buf,
                round_offset,
                0,
                favorite_teams,
            );

            // Grand final after upper bracket if present
            if let Some(ref gf) = bracket.grand_final {
                // Render in upper area's last visible column if space
                let visible_cols = area.width / COLUMN_WIDTH;
                let upper_visible = bracket.upper_rounds.len().saturating_sub(round_offset);
                if (upper_visible as u16) < visible_cols {
                    let col_x = area.x + (upper_visible as u16) * COLUMN_WIDTH;
                    if col_x + COLUMN_WIDTH <= area.x + area.width {
                        render_round_header("Grand Final", col_x, area.y, COLUMN_WIDTH, buf, true);
                        render_separator(col_x, area.y + 1, COLUMN_WIDTH, buf);
                        render_match_cell(gf, col_x, area.y + 2, COLUMN_WIDTH, buf, favorite_teams, false);
                    }
                }
            }

            return;
        }
    }

    // Single elimination or no lower bracket
    render_rounds(
        &bracket.upper_rounds,
        area,
        buf,
        round_offset,
        match_offset,
        favorite_teams,
    );

    // Grand final as extra column
    if let Some(ref gf) = bracket.grand_final {
        let visible_cols = area.width / COLUMN_WIDTH;
        let upper_visible = bracket.upper_rounds.len().saturating_sub(round_offset);
        if (upper_visible as u16) < visible_cols {
            let col_x = area.x + (upper_visible as u16) * COLUMN_WIDTH;
            if col_x + COLUMN_WIDTH <= area.x + area.width {
                render_round_header("Grand Final", col_x, area.y, COLUMN_WIDTH, buf, true);
                render_separator(col_x, area.y + 1, COLUMN_WIDTH, buf);
                render_match_cell(gf, col_x, area.y + 2, COLUMN_WIDTH, buf, favorite_teams, false);
            }
        }
    }
}

fn render_rounds(
    rounds: &[BracketRound],
    area: Rect,
    buf: &mut Buffer,
    round_offset: usize,
    match_offset: usize,
    favorite_teams: &[String],
) {
    if area.width < 4 || area.height < 2 {
        return;
    }

    let visible_cols = (area.width / COLUMN_WIDTH).max(1) as usize;

    for (col_idx, round_idx) in (round_offset..).enumerate() {
        if col_idx >= visible_cols {
            break;
        }
        if round_idx >= rounds.len() {
            break;
        }

        let round = &rounds[round_idx];
        let col_x = area.x + (col_idx as u16) * COLUMN_WIDTH;
        let col_width = COLUMN_WIDTH.min(area.x + area.width - col_x);

        let is_active = col_idx == 0; // First visible column is "active"

        // Header
        render_round_header(&round.name, col_x, area.y, col_width, buf, is_active);

        // Separator
        if area.height > 1 {
            render_separator(col_x, area.y + 1, col_width, buf);
        }

        // Matches
        let match_area_y = area.y + 2;
        if match_area_y >= area.y + area.height {
            continue;
        }
        let match_area_height = area.y + area.height - match_area_y;

        let skip = if col_idx == 0 { match_offset } else { 0 };
        let matches_to_render = round.matches.iter().skip(skip);
        let lines_per_match: u16 = 3; // team_a, team_b, gap

        for (m_idx, bracket_match) in matches_to_render.enumerate() {
            let y_offset = (m_idx as u16) * lines_per_match;
            if y_offset + 2 > match_area_height {
                break;
            }
            let y = match_area_y + y_offset;
            let selected = col_idx == 0 && m_idx == 0 && match_offset > 0;
            render_match_cell(bracket_match, col_x, y, col_width, buf, favorite_teams, selected);
        }
    }
}

fn render_round_header(name: &str, x: u16, y: u16, width: u16, buf: &mut Buffer, active: bool) {
    let style = if active {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let truncated = truncate_str(name, width as usize);
    let span = Span::styled(truncated, style);
    buf.set_line(x, y, &Line::from(span), width);
}

fn render_separator(x: u16, y: u16, width: u16, buf: &mut Buffer) {
    let line_str: String = "─".repeat(width as usize);
    let span = Span::styled(line_str, Style::default().fg(Color::DarkGray));
    buf.set_line(x, y, &Line::from(span), width);
}

fn render_divider(label: &str, y: u16, x: u16, width: u16, buf: &mut Buffer) {
    let label_str = format!("=== {} ===", label);
    let label_display_width = label_str.chars().count() as u16;
    let pad = width.saturating_sub(label_display_width) / 2;
    let right_pad = width.saturating_sub(pad + label_display_width);
    let full: String = format!(
        "{}{}{}",
        "=".repeat(pad as usize),
        label_str,
        "=".repeat(right_pad as usize)
    );
    let span = Span::styled(
        truncate_str(&full, width as usize),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    );
    buf.set_line(x, y, &Line::from(span), width);
}

fn render_match_cell(
    m: &BracketMatch,
    x: u16,
    y: u16,
    width: u16,
    buf: &mut Buffer,
    favorite_teams: &[String],
    selected: bool,
) {
    let bg = if selected { Color::DarkGray } else { Color::Reset };

    // Team A line
    let team_a_line = format_team_line(
        m.team_a.as_deref(),
        m.score_a,
        width as usize,
        m.status,
        is_winner_a(m),
        is_favorite(m.team_a.as_deref(), favorite_teams),
        bg,
    );
    buf.set_line(x, y, &team_a_line, width);

    // Team B line
    let team_b_line = format_team_line(
        m.team_b.as_deref(),
        m.score_b,
        width as usize,
        m.status,
        is_winner_b(m),
        is_favorite(m.team_b.as_deref(), favorite_teams),
        bg,
    );
    if y + 1 < y.saturating_add(3) {
        buf.set_line(x, y + 1, &team_b_line, width);
    }
}

fn format_team_line(
    team: Option<&str>,
    score: u8,
    width: usize,
    status: MatchStatus,
    is_winner: bool,
    is_fav: bool,
    bg: Color,
) -> Line<'static> {
    let (name, is_tbd) = match team {
        Some(n) if !n.is_empty() => (n.to_string(), false),
        _ => ("TBD".to_string(), true),
    };

    let score_str = format!(" {}", score);
    let name_width = width.saturating_sub(score_str.len() + 1);
    let truncated_name = truncate_str(&name, name_width);

    let mut style = Style::default().bg(bg);

    if is_tbd {
        style = style.fg(Color::DarkGray).add_modifier(Modifier::ITALIC);
    } else {
        match status {
            MatchStatus::Live => {
                style = style.fg(Color::Red);
            }
            MatchStatus::Completed => {
                if is_winner {
                    style = style.fg(Color::White);
                } else {
                    style = style.fg(Color::DarkGray);
                }
            }
            MatchStatus::Upcoming => {
                style = style.fg(Color::Gray);
            }
        }
    }

    if is_fav {
        style = style.add_modifier(Modifier::BOLD);
    }

    let padding = name_width.saturating_sub(truncated_name.len());
    let line_str = format!("{}{}{}", truncated_name, " ".repeat(padding), score_str);

    Line::from(Span::styled(line_str, style))
}

fn is_winner_a(m: &BracketMatch) -> bool {
    m.status == MatchStatus::Completed && m.score_a > m.score_b
}

fn is_winner_b(m: &BracketMatch) -> bool {
    m.status == MatchStatus::Completed && m.score_b > m.score_a
}

fn is_favorite(team: Option<&str>, favorites: &[String]) -> bool {
    match team {
        Some(name) => favorites.iter().any(|f| f.eq_ignore_ascii_case(name)),
        None => false,
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else if max_len > 1 {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    } else {
        s.chars().take(max_len).collect()
    }
}
