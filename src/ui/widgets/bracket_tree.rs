use crate::models::{Bracket, BracketMatch, MatchStatus};
use ratatui::prelude::*;

const MATCH_WIDTH: usize = 20;
const CONNECTOR_WIDTH: usize = 4;
const CELL_WIDTH: usize = MATCH_WIDTH + CONNECTOR_WIDTH;

/// A single styled character on the virtual canvas.
#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
        }
    }
}

/// Virtual 2D canvas that we draw the entire bracket onto, then blit the
/// visible window into the ratatui Buffer.
struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); width * height],
        }
    }

    fn set(&mut self, x: usize, y: usize, ch: char, style: Style) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = Cell { ch, style };
        }
    }

    fn put_str(&mut self, x: usize, y: usize, s: &str, style: Style) {
        for (i, ch) in s.chars().enumerate() {
            self.set(x + i, y, ch, style);
        }
    }

    fn get(&self, x: usize, y: usize) -> Cell {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x]
        } else {
            Cell::default()
        }
    }
}

pub fn render_tree_bracket(
    bracket: &Bracket,
    area: Rect,
    buf: &mut Buffer,
    scroll_x: usize,
    scroll_y: usize,
    favorite_teams: &[String],
) {
    if area.width < 4 || area.height < 2 {
        return;
    }

    let rounds = &bracket.upper_rounds;
    if rounds.is_empty() {
        let msg = Span::styled(
            "No bracket rounds available.",
            Style::default().fg(Color::DarkGray),
        );
        buf.set_line(area.x, area.y, &Line::from(msg), area.width);
        return;
    }

    // Determine the number of R1 matches to calculate canvas height.
    let r1_match_count = rounds[0].matches.len().max(1);
    let canvas_height = r1_match_count * 2;

    // Include grand final as an extra round if present.
    let has_grand_final = bracket.grand_final.is_some();
    let num_rounds = rounds.len() + if has_grand_final { 1 } else { 0 };
    let canvas_width = num_rounds * CELL_WIDTH;

    let mut canvas = Canvas::new(canvas_width, canvas_height);

    // Recursively draw from the final round backward.
    // The last round's match 0 spans the full vertical range.
    let last_round_idx = rounds.len() - 1;
    draw_match_recursive(
        &mut canvas,
        rounds,
        last_round_idx,
        0,      // match index within round
        0,      // top of vertical span
        canvas_height,
        favorite_teams,
    );

    // Draw grand final if present.
    if let Some(ref gf) = bracket.grand_final {
        let gf_col = rounds.len();
        let mid = canvas_height / 2;
        // The grand final connects from the final match's midpoint.
        draw_single_match(&mut canvas, gf, gf_col, mid, favorite_teams);
        // Connector from previous round's midpoint to grand final.
        let prev_mid = mid;
        let gf_x = gf_col * CELL_WIDTH;
        let _prev_end_x = gf_col * CELL_WIDTH; // start of gf column = end of connector region
        // Draw horizontal connector from end of last round to the grand final.
        let conn_start = last_round_idx * CELL_WIDTH + MATCH_WIDTH;
        draw_horizontal_connector(&mut canvas, conn_start, prev_mid, gf_x);
    }

    // Blit the visible portion onto the ratatui buffer.
    for row in 0..area.height as usize {
        let cy = scroll_y + row;
        for col in 0..area.width as usize {
            let cx = scroll_x + col;
            let cell = canvas.get(cx, cy);
            if cell.ch != ' ' || cell.style != Style::default() {
                let bx = area.x + col as u16;
                let by = area.y + row as u16;
                if bx < area.x + area.width && by < area.y + area.height {
                    if let Some(buf_cell) = buf.cell_mut((bx, by)) {
                        buf_cell.set_char(cell.ch).set_style(cell.style);
                    }
                }
            }
        }
    }
}

/// Recursively draw a match and its feeder matches.
/// `round_idx` and `match_idx` identify the match.
/// `span_top` and `span_height` define the vertical region this match owns.
fn draw_match_recursive(
    canvas: &mut Canvas,
    rounds: &[crate::models::BracketRound],
    round_idx: usize,
    match_idx: usize,
    span_top: usize,
    span_height: usize,
    favorite_teams: &[String],
) {
    let round = &rounds[round_idx];
    if match_idx >= round.matches.len() {
        return;
    }
    let m = &round.matches[match_idx];

    // The midpoint of this match's span is where we draw the team lines.
    let mid = span_top + span_height / 2;

    // Draw the match itself.
    draw_single_match(canvas, m, round_idx, mid, favorite_teams);

    if round_idx == 0 {
        // Base case: R1 matches have no feeders. Draw two team lines.
        // They are already drawn by draw_single_match at mid-1 and mid (or mid and mid+1).
        return;
    }

    // Recurse into the two feeder matches from the previous round.
    let prev_round_idx = round_idx - 1;
    let feeder_a_idx = match_idx * 2;
    let feeder_b_idx = match_idx * 2 + 1;

    let half = span_height / 2;
    let top_span_top = span_top;
    let top_span_height = half;
    let bot_span_top = span_top + half;
    let bot_span_height = span_height - half;

    // Draw feeder A (top half).
    if feeder_a_idx < rounds[prev_round_idx].matches.len() {
        draw_match_recursive(
            canvas,
            rounds,
            prev_round_idx,
            feeder_a_idx,
            top_span_top,
            top_span_height,
            favorite_teams,
        );

        // Draw connector from feeder A's midpoint to this match's midpoint.
        let feeder_a_mid = top_span_top + top_span_height / 2;
        draw_connector(canvas, prev_round_idx, feeder_a_mid, mid);
    }

    // Draw feeder B (bottom half).
    if feeder_b_idx < rounds[prev_round_idx].matches.len() {
        draw_match_recursive(
            canvas,
            rounds,
            prev_round_idx,
            feeder_b_idx,
            bot_span_top,
            bot_span_height,
            favorite_teams,
        );

        let feeder_b_mid = bot_span_top + bot_span_height / 2;
        draw_connector(canvas, prev_round_idx, feeder_b_mid, mid);
    }
}

/// Draw a single match (two team lines) centered at the given `mid` row.
/// Team A is drawn at `mid - 1`, team B at `mid` (if mid > 0), or mid and mid+1.
fn draw_single_match(
    canvas: &mut Canvas,
    m: &BracketMatch,
    round_col: usize,
    mid: usize,
    favorite_teams: &[String],
) {
    let x = round_col * CELL_WIDTH;

    // Team A at row above midpoint, team B at midpoint.
    let row_a = if mid > 0 { mid - 1 } else { 0 };
    let row_b = if mid > 0 { mid } else { 1 };

    let team_a_str = format_team_str(m.team_a.as_deref(), m.score_a);
    let team_b_str = format_team_str(m.team_b.as_deref(), m.score_b);

    let style_a = team_style(
        m.team_a.as_deref(),
        m.status,
        m.status == MatchStatus::Completed && m.score_a > m.score_b,
        is_favorite(m.team_a.as_deref(), favorite_teams),
    );
    let style_b = team_style(
        m.team_b.as_deref(),
        m.status,
        m.status == MatchStatus::Completed && m.score_b > m.score_a,
        is_favorite(m.team_b.as_deref(), favorite_teams),
    );

    canvas.put_str(x, row_a, &team_a_str, style_a);
    canvas.put_str(x, row_b, &team_b_str, style_b);

    // Draw " --+" after each team line to start the connector.
    let conn_style = Style::default().fg(Color::DarkGray);
    let conn_x = x + MATCH_WIDTH;
    canvas.put_str(conn_x, row_a, "--+", conn_style);
    canvas.put_str(conn_x, row_b, "--+", conn_style);
}

/// Draw connector lines from a feeder match's midpoint to the current match's midpoint.
/// The connector goes from the end of the feeder's column rightward,
/// with a vertical bar and then horizontal into the next match.
fn draw_connector(
    canvas: &mut Canvas,
    feeder_round_idx: usize,
    feeder_mid: usize,
    target_mid: usize,
) {
    let conn_style = Style::default().fg(Color::DarkGray);

    // The vertical connector runs at the x position just after the "--+" of the feeder.
    // That's at feeder_round_idx * CELL_WIDTH + MATCH_WIDTH + 2 (the '+' position).
    let vert_x = feeder_round_idx * CELL_WIDTH + MATCH_WIDTH + 2;

    // Draw vertical line between feeder_mid and target_mid.
    let (top, bot) = if feeder_mid < target_mid {
        (feeder_mid, target_mid)
    } else {
        (target_mid, feeder_mid)
    };

    // The '+' characters are already at the feeder's team rows.
    // Draw '|' for intermediate rows.
    for y in (top + 1)..bot {
        canvas.set(vert_x, y, '|', conn_style);
    }

    // Draw horizontal connector from vert_x to the next round's match start.
    // The midpoint between the two feeders connects to the next round match.
    // We need "+--" at target_mid leading into the next round.
    let next_round_x = (feeder_round_idx + 1) * CELL_WIDTH;
    canvas.set(vert_x, target_mid, '+', conn_style);
    // Horizontal line from vert_x+1 to the start of the next match.
    for x in (vert_x + 1)..next_round_x {
        canvas.set(x, target_mid, '-', conn_style);
    }
}

/// Draw a simple horizontal connector (used for grand final).
fn draw_horizontal_connector(canvas: &mut Canvas, from_x: usize, y: usize, to_x: usize) {
    let style = Style::default().fg(Color::DarkGray);
    for x in from_x..to_x {
        canvas.set(x, y, '-', style);
    }
}

fn format_team_str(team: Option<&str>, score: u8) -> String {
    let name = match team {
        Some(n) if !n.is_empty() => n,
        _ => "TBD",
    };
    let score_str = format!(" {}", score);
    let name_max = MATCH_WIDTH.saturating_sub(score_str.len() + 1);
    let char_count = name.chars().count();
    let truncated = if char_count > name_max && name_max > 1 {
        let t: String = name.chars().take(name_max - 1).collect();
        format!("{}…", t)
    } else if char_count > name_max {
        name.chars().take(name_max).collect()
    } else {
        name.to_string()
    };
    let padding = name_max.saturating_sub(truncated.chars().count());
    format!(" {}{}{}", truncated, " ".repeat(padding), score_str)
}

fn team_style(team: Option<&str>, status: MatchStatus, is_winner: bool, is_fav: bool) -> Style {
    let is_tbd = team.is_none() || team.map_or(false, |n| n.is_empty());
    let mut style = Style::default();

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

    style
}

fn is_favorite(team: Option<&str>, favorites: &[String]) -> bool {
    match team {
        Some(name) => favorites.iter().any(|f| f.eq_ignore_ascii_case(name)),
        None => false,
    }
}
