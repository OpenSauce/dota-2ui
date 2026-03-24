use crate::app::App;
use crate::ui::widgets::{countdown, keybind_bar, match_card};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if area.width < 80 {
        render_narrow(frame, app, area);
        return;
    }

    let live = app.live_matches();
    // 1 line per match in the live panel
    let live_rows = ((live.len() + 1) / 2) as u16; // 2 columns
    let live_height = live_rows.max(1).min(8) + 2; // +2 for border

    let main_layout = Layout::vertical([
        Constraint::Length(live_height),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .split(area);

    render_live_panel(frame, app, main_layout[0]);

    let grid = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(main_layout[1]);

    render_upcoming_panel(frame, app, grid[0]);
    render_right_panel(frame, app, grid[1]);
    keybind_bar::render_keybind_bar(&app.screen, main_layout[2], frame.buffer_mut());

    if let Some(ref err) = app.error_message {
        let err_area = Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" Error: {} ", err),
                Style::default().fg(Color::Red).bg(Color::DarkGray),
            )),
            err_area,
        );
    }
}

fn render_live_panel(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " LIVE ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let live = app.live_matches();
    if live.is_empty() {
        let msg = if app.is_loading { "Loading..." } else { "No live matches" };
        frame.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(Color::DarkGray))),
            inner,
        );
        return;
    }

    // 2-column grid: left column gets even indices, right gets odd
    let col_width = inner.width / 2;
    for (i, m) in live.iter().enumerate() {
        let row = (i / 2) as u16;
        let col = (i % 2) as u16;
        let y = inner.y + row;
        if y >= inner.y + inner.height {
            break;
        }
        let x = inner.x + col * col_width;
        match_card::render_match_card(
            m,
            Rect { x, y, width: col_width, height: 1 },
            frame.buffer_mut(),
            true,
            app.tick_count,
        );
    }
}

fn render_upcoming_panel(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " UPCOMING ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let upcoming = app.upcoming_matches();
    if upcoming.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("No upcoming matches", Style::default().fg(Color::DarkGray))),
            inner,
        );
        return;
    }

    for (i, m) in upcoming.iter().enumerate() {
        let y = inner.y + (i as u16) * 2;
        if y + 1 > inner.y + inner.height {
            break;
        }
        match_card::render_match_card(
            m,
            Rect { x: inner.x, y, width: inner.width, height: 2 },
            frame.buffer_mut(),
            false,
            app.tick_count,
        );
    }
}

fn render_right_panel(frame: &mut Frame, app: &App, area: Rect) {
    let has_favorites = !app.config.favorite_teams.is_empty();

    let split = if has_favorites {
        Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).split(area)
    } else {
        Layout::vertical([Constraint::Percentage(100)]).split(area)
    };

    let t_block = Block::default()
        .title(Span::styled(
            " TOURNAMENTS ",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));
    let t_inner = t_block.inner(split[0]);
    frame.render_widget(t_block, split[0]);

    for (i, t) in app.upcoming_tournaments().iter().enumerate() {
        let y = t_inner.y + (i as u16) * 2;
        if y + 1 >= t_inner.y + t_inner.height { break; }
        countdown::render_countdown_with_gauge(
            t,
            Rect { x: t_inner.x, y, width: t_inner.width, height: 2 },
            frame.buffer_mut(),
        );
    }

    if has_favorites && split.len() > 1 {
        let f_block = Block::default()
            .title(Span::styled(
                " FAVORITES ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1));
        let f_inner = f_block.inner(split[1]);
        frame.render_widget(f_block, split[1]);

        let fav_matches = app.favorite_teams_matches();
        if fav_matches.is_empty() {
            frame.render_widget(
                Paragraph::new(Span::styled("No matches for favorites", Style::default().fg(Color::DarkGray))),
                f_inner,
            );
        } else {
            for (i, m) in fav_matches.iter().enumerate() {
                let y = f_inner.y + i as u16;
                if y >= f_inner.y + f_inner.height {
                    break;
                }
                match_card::render_match_card(
                    m,
                    Rect { x: f_inner.x, y, width: f_inner.width, height: 1 },
                    frame.buffer_mut(),
                    true,
                    app.tick_count,
                );
            }
        }
    }
}

fn render_narrow(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::vertical([
        Constraint::Length(6),
        Constraint::Min(4),
        Constraint::Length(4),
        Constraint::Length(1),
    ])
    .split(area);

    render_live_panel(frame, app, layout[0]);
    render_upcoming_panel(frame, app, layout[1]);
    render_right_panel(frame, app, layout[2]);
    keybind_bar::render_keybind_bar(&app.screen, layout[3], frame.buffer_mut());
}
