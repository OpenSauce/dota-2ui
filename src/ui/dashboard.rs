use crate::app::App;
use crate::input::MatchFilter;
use crate::ui::widgets::{countdown, keybind_bar, match_card};
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if area.width < 80 {
        render_narrow(frame, app, area);
        return;
    }

    let live = app.live_matches();
    // 1 line per match in the live panel
    let live_rows = live.len().div_ceil(2) as u16; // 2 columns
    let live_height = live_rows.clamp(1, 8) + 2; // +2 for border

    let main_layout = Layout::vertical([
        Constraint::Length(live_height),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .split(area);

    render_live_panel(frame, app, main_layout[0]);

    let grid = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[1]);

    render_upcoming_panel(frame, app, grid[0]);
    render_right_panel(frame, app, grid[1]);

    // Status message, search bar, or keybind bar
    if let Some((ref msg, _)) = app.status_message {
        let status = Paragraph::new(Span::styled(
            format!(" {} ", msg),
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ));
        frame.render_widget(status, main_layout[2]);
    } else if app.search_active {
        let search = Paragraph::new(Span::styled(
            format!("/ {}_", app.search_query),
            Style::default().fg(Color::Cyan),
        ));
        frame.render_widget(search, main_layout[2]);
    } else {
        keybind_bar::render_keybind_bar(app, main_layout[2], frame.buffer_mut());
    }

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

    // Favorite picker dialog overlay
    if let Some(ref picker) = app.favorite_picker {
        render_favorite_picker(frame, picker, &app.config.favorite_teams, area);
    }
}

fn render_favorite_picker(
    frame: &mut Frame,
    picker: &crate::app::FavoritePicker,
    favorite_teams: &[String],
    area: Rect,
) {
    let width = 40u16.min(area.width.saturating_sub(4));
    let height = 6u16;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let popup_area = Rect::new(x, y, width, height);

    // Clear background
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(
            " Favorite Team ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let a_fav = favorite_teams.iter().any(|f| f == &picker.team_a);
    let b_fav = favorite_teams.iter().any(|f| f == &picker.team_b);

    let teams = [(&picker.team_a, a_fav, 0), (&picker.team_b, b_fav, 1)];

    for (i, (name, is_fav, idx)) in teams.iter().enumerate() {
        let row = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
        let star = if *is_fav { "★ " } else { "  " };
        let cursor = if picker.selected == *idx { ">" } else { " " };
        let style = if picker.selected == *idx {
            Style::default().fg(Color::White).bg(Color::Indexed(236))
        } else {
            Style::default().fg(Color::DarkGray)
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", cursor), style),
                Span::styled(star.to_string(), Style::default().fg(Color::Yellow)),
                Span::styled(name.to_string(), style),
            ])),
            row,
        );
    }

    // Help text
    if inner.height > 2 {
        let help_row = Rect::new(inner.x, inner.y + 3, inner.width, 1);
        frame.render_widget(
            Paragraph::new(Span::styled(
                " j/k move  s/Enter toggle  Esc close",
                Style::default().fg(Color::DarkGray),
            )),
            help_row,
        );
    }
}

fn panel_border(is_active: bool) -> (BorderType, Style) {
    if is_active {
        (BorderType::Double, Style::default().fg(Color::White))
    } else {
        (BorderType::Plain, Style::default().fg(Color::DarkGray))
    }
}

fn render_live_panel(frame: &mut Frame, app: &App, area: Rect) {
    let (border_type, border_style) = panel_border(app.active_panel == 0);
    let block = Block::default()
        .title(Span::styled(
            " LIVE ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style)
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let live: Vec<_> = match app.active_filter {
        MatchFilter::UpcomingOnly => vec![],
        MatchFilter::FavoritesOnly => app
            .favorite_teams_matches()
            .into_iter()
            .filter(|m| m.status.is_live())
            .collect(),
        _ => app.live_matches(),
    };

    if live.is_empty() {
        let msg = match app.active_filter {
            MatchFilter::UpcomingOnly => "No matches (filter: Upcoming Only)",
            _ if app.is_loading => "Loading...",
            _ => "No live matches",
        };
        frame.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(Color::DarkGray))),
            inner,
        );
        return;
    }

    let is_active = app.active_panel == 0;
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
        let cell_area = Rect {
            x,
            y,
            width: col_width,
            height: 1,
        };

        match_card::render_match_card(
            m,
            cell_area,
            frame.buffer_mut(),
            true,
            app.tick_count,
            &app.config.favorite_teams,
        );

        // Highlight selected item when panel is active
        if is_active && i == app.scroll_offset {
            frame.buffer_mut().set_style(
                cell_area,
                Style::default().bg(Color::Indexed(236)).fg(Color::White),
            );
        }
    }
}

fn render_upcoming_panel(frame: &mut Frame, app: &App, area: Rect) {
    let (border_type, border_style) = panel_border(app.active_panel == 1);
    let block = Block::default()
        .title(Span::styled(
            " UPCOMING ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style)
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let upcoming: Vec<_> = match app.active_filter {
        MatchFilter::LiveOnly => vec![],
        MatchFilter::FavoritesOnly => app
            .favorite_teams_matches()
            .into_iter()
            .filter(|m| m.status == crate::models::MatchStatus::Upcoming)
            .collect(),
        _ => app.upcoming_matches(),
    };

    if upcoming.is_empty() {
        let msg = match app.active_filter {
            MatchFilter::LiveOnly => "No matches (filter: Live Only)",
            _ => "No upcoming matches",
        };
        frame.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(Color::DarkGray))),
            inner,
        );
        return;
    }

    let is_active = app.active_panel == 1;
    let compact = area.width < 100;
    let row_height: u16 = if compact { 1 } else { 2 };
    for (i, m) in upcoming.iter().enumerate() {
        let y = inner.y + (i as u16) * row_height;
        if y + row_height > inner.y + inner.height {
            break;
        }
        let card_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: row_height,
        };
        match_card::render_match_card(
            m,
            card_area,
            frame.buffer_mut(),
            compact,
            app.tick_count,
            &app.config.favorite_teams,
        );

        // Highlight selected item when panel is active
        if is_active && i == app.scroll_offset {
            frame
                .buffer_mut()
                .set_style(card_area, Style::default().bg(Color::DarkGray));
        }
    }
}

fn render_right_panel(frame: &mut Frame, app: &App, area: Rect) {
    let has_favorites = !app.config.favorite_teams.is_empty();

    let split = if has_favorites {
        Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).split(area)
    } else {
        Layout::vertical([Constraint::Percentage(100)]).split(area)
    };

    let (t_border_type, t_border_style) = panel_border(app.active_panel == 2);
    let t_block = Block::default()
        .title(Span::styled(
            " TOURNAMENTS ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(t_border_type)
        .border_style(t_border_style)
        .padding(Padding::horizontal(1));
    let t_inner = t_block.inner(split[0]);
    frame.render_widget(t_block, split[0]);

    let t_is_active = app.active_panel == 2;
    for (i, t) in app.upcoming_tournaments().iter().enumerate() {
        let y = t_inner.y + (i as u16) * 2;
        if y + 1 >= t_inner.y + t_inner.height {
            break;
        }
        let t_area = Rect {
            x: t_inner.x,
            y,
            width: t_inner.width,
            height: 2,
        };
        countdown::render_countdown_with_gauge(t, t_area, frame.buffer_mut());

        if t_is_active && i == app.scroll_offset {
            frame
                .buffer_mut()
                .set_style(t_area, Style::default().bg(Color::DarkGray));
        }
    }

    if has_favorites && split.len() > 1 {
        let (f_border_type, f_border_style) = panel_border(app.active_panel == 3);
        let f_block = Block::default()
            .title(Span::styled(
                " FAVORITES ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(f_border_type)
            .border_style(f_border_style)
            .padding(Padding::horizontal(1));
        let f_inner = f_block.inner(split[1]);
        frame.render_widget(f_block, split[1]);

        let fav_matches = app.favorite_teams_matches();
        if fav_matches.is_empty() {
            frame.render_widget(
                Paragraph::new(Span::styled(
                    "No matches for favorites",
                    Style::default().fg(Color::DarkGray),
                )),
                f_inner,
            );
        } else {
            let f_is_active = app.active_panel == 3;
            for (i, m) in fav_matches.iter().enumerate() {
                let y = f_inner.y + i as u16;
                if y >= f_inner.y + f_inner.height {
                    break;
                }
                let fav_area = Rect {
                    x: f_inner.x,
                    y,
                    width: f_inner.width,
                    height: 1,
                };
                match_card::render_match_card(
                    m,
                    fav_area,
                    frame.buffer_mut(),
                    true,
                    app.tick_count,
                    &app.config.favorite_teams,
                );

                if f_is_active && i == app.scroll_offset {
                    frame
                        .buffer_mut()
                        .set_style(fav_area, Style::default().bg(Color::DarkGray));
                }
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

    if let Some((ref msg, _)) = app.status_message {
        let status = Paragraph::new(Span::styled(
            format!(" {} ", msg),
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ));
        frame.render_widget(status, layout[3]);
    } else if app.search_active {
        let search = Paragraph::new(Span::styled(
            format!("/ {}_", app.search_query),
            Style::default().fg(Color::Cyan),
        ));
        frame.render_widget(search, layout[3]);
    } else {
        keybind_bar::render_keybind_bar(app, layout[3], frame.buffer_mut());
    }
}
