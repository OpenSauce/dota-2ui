use crate::config::Config;
use crate::input::{AppAction, MatchFilter, Screen, TournamentTab};
use crate::models::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NotificationEvent {
    MatchSoon,
    MatchLive,
    TournamentToday,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotificationKey {
    pub match_or_tournament_id: String,
    pub event: NotificationEvent,
}

#[derive(Debug, Clone)]
pub struct FavoritePicker {
    pub team_a: String,
    pub team_b: String,
    pub selected: usize, // 0 = team_a, 1 = team_b
}

pub struct App {
    pub screen: Screen,
    pub config: Config,
    pub matches: Vec<Match>,
    pub tournaments: Vec<Tournament>,
    pub selected_tournament_id: Option<String>,
    pub scroll_offset: usize,
    pub active_panel: usize,
    pub should_quit: bool,
    pub last_refresh: Instant,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub search_query: String,
    pub search_active: bool,
    pub tick_count: u64,
    pub broadcast_mode: bool,
    pub ticker_offset: usize,
    pub notified_events: HashSet<NotificationKey>,
    pub active_filter: MatchFilter,
    pub tournament_detail_tab: TournamentTab,
    pub status_message: Option<(String, u64)>,
    pub favorite_picker: Option<FavoritePicker>,
    pub show_all_tournaments: bool,
    pub bracket_cache: HashMap<String, Bracket>,
    pub bracket_attempted: HashSet<String>,
    pub bracket_loading: bool,
    pub bracket_round_offset: usize,
    pub bracket_match_offset: usize,
    pub bracket_view_mode: BracketViewMode,
    // Match detail
    pub selected_match_id: Option<String>,
    pub match_detail_cache: HashMap<String, (MatchDetailData, Instant)>,
    pub match_detail_loading: bool,
    pub previous_screen: Option<Screen>,
    pub selected_game: usize,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            screen: Screen::Dashboard,
            config,
            matches: Vec::new(),
            tournaments: Vec::new(),
            selected_tournament_id: None,
            scroll_offset: 0,
            active_panel: 0,
            should_quit: false,
            last_refresh: Instant::now() - Duration::from_secs(9999),
            is_loading: false,
            error_message: None,
            search_query: String::new(),
            search_active: false,
            tick_count: 0,
            broadcast_mode: false,
            ticker_offset: 0,
            notified_events: HashSet::new(),
            active_filter: MatchFilter::All,
            tournament_detail_tab: TournamentTab::Overview,
            status_message: None,
            favorite_picker: None,
            show_all_tournaments: false,
            bracket_cache: HashMap::new(),
            bracket_attempted: HashSet::new(),
            bracket_loading: false,
            bracket_round_offset: 0,
            bracket_match_offset: 0,
            bracket_view_mode: BracketViewMode::Column,
            selected_match_id: None,
            match_detail_cache: HashMap::new(),
            match_detail_loading: false,
            previous_screen: None,
            selected_game: 0,
        }
    }

    pub fn handle_action(&mut self, action: AppAction) {
        // Clear status message after 30 ticks (~3s at 10fps)
        if let Some((_, tick)) = &self.status_message {
            if self.tick_count.wrapping_sub(*tick) > 30 {
                self.status_message = None;
            }
        }

        // If favorite picker is open, handle keys for the picker
        if self.favorite_picker.is_some() {
            match action {
                AppAction::ScrollUp | AppAction::ScrollDown => {
                    if let Some(ref mut picker) = self.favorite_picker {
                        picker.selected = if picker.selected == 0 { 1 } else { 0 };
                    }
                }
                AppAction::ToggleFavorite | AppAction::Select => {
                    if let Some(picker) = self.favorite_picker.take() {
                        let team = if picker.selected == 0 {
                            &picker.team_a
                        } else {
                            &picker.team_b
                        };
                        let was_fav = self.config.favorite_teams.iter().any(|f| f == team);
                        self.config.toggle_favorite_team(team);
                        let msg = if was_fav {
                            format!("Removed {} from favorites", team)
                        } else {
                            format!("★ Added {} to favorites", team)
                        };
                        self.status_message = Some((msg, self.tick_count));
                        if let Err(e) = self.config.save() {
                            self.error_message = Some(format!("Failed to save: {}", e));
                        }
                    }
                }
                AppAction::Back | AppAction::SearchCancel => {
                    self.favorite_picker = None;
                }
                AppAction::Quit => {
                    self.favorite_picker = None;
                    self.should_quit = true;
                }
                _ => {}
            }
            return;
        }

        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => {
                if self.search_active {
                    self.search_active = false;
                    self.search_query.clear();
                } else {
                    match self.screen {
                        Screen::Dashboard => self.should_quit = true,
                        Screen::MatchDetail => {
                            self.screen = self.previous_screen.take().unwrap_or(Screen::Dashboard);
                            self.selected_match_id = None;
                            self.match_detail_loading = false;
                            self.scroll_offset = 0;
                        }
                        _ => {
                            self.screen = Screen::Dashboard;
                            self.scroll_offset = 0;
                            self.tournament_detail_tab = TournamentTab::Overview;
                        }
                    }
                }
            }
            AppAction::ScrollUp => {
                if self.screen == Screen::TournamentDetail
                    && self.tournament_detail_tab == TournamentTab::Bracket
                {
                    self.bracket_match_offset = self.bracket_match_offset.saturating_sub(1);
                } else {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                }
            }
            AppAction::ScrollDown => {
                if self.screen == Screen::TournamentDetail
                    && self.tournament_detail_tab == TournamentTab::Bracket
                {
                    if let Some(max) = self.bracket_round_match_count() {
                        if self.bracket_match_offset < max.saturating_sub(1) {
                            self.bracket_match_offset += 1;
                        }
                    }
                } else {
                    let max = self.current_panel_len().saturating_sub(1);
                    if self.scroll_offset < max {
                        self.scroll_offset += 1;
                    }
                }
            }
            AppAction::NextPanel => {
                if self.screen == Screen::TournamentDetail
                    && self.tournament_detail_tab == TournamentTab::Bracket
                {
                    if let Some(max_rounds) = self.bracket_round_count() {
                        if max_rounds > 0 {
                            self.bracket_round_offset =
                                (self.bracket_round_offset + 1) % max_rounds;
                            self.bracket_match_offset = 0;
                        }
                    }
                } else {
                    self.active_panel = (self.active_panel + 1) % 4;
                    self.scroll_offset = 0;
                }
            }
            AppAction::Select => match self.screen {
                Screen::TournamentBrowser => {
                    if let Some(t) = self.browsable_tournaments().get(self.scroll_offset) {
                        self.selected_tournament_id = Some(t.id.clone());
                        self.screen = Screen::TournamentDetail;
                        self.scroll_offset = 0;
                        self.tournament_detail_tab = TournamentTab::Overview;
                        self.bracket_loading = false;
                    }
                }
                Screen::Dashboard | Screen::TournamentDetail => {
                    if let Some(m) = self.selected_match() {
                        let mid = m.id.clone();
                        self.previous_screen = Some(self.screen.clone());
                        self.selected_match_id = Some(mid);
                        self.screen = Screen::MatchDetail;
                        self.selected_game = 0;
                        self.scroll_offset = 0;
                    }
                }
                _ => {}
            },
            AppAction::ToggleFavorite => {
                self.handle_toggle_favorite();
            }
            AppAction::Refresh => {
                self.last_refresh = Instant::now() - Duration::from_secs(9999);
                // Also invalidate match detail cache when on that screen
                if self.screen == Screen::MatchDetail {
                    if let Some(mid) = &self.selected_match_id {
                        self.match_detail_cache.remove(mid);
                    }
                    self.selected_game = 0;
                }
            }
            AppAction::OpenTournaments => {
                self.screen = Screen::TournamentBrowser;
                self.scroll_offset = 0;
            }
            AppAction::OpenFilter => {
                self.active_filter = self.active_filter.next();
                self.clamp_scroll();
            }
            AppAction::OpenSearch => {
                self.search_active = true;
                self.search_query.clear();
            }
            AppAction::OpenSettings => {
                self.screen = Screen::Settings;
                self.scroll_offset = 0;
            }
            AppAction::OpenStream => {
                if let Some(m) = self.selected_match() {
                    if let Some(url) = &m.stream_url {
                        let _ = open::that(url);
                    }
                }
            }
            AppAction::ShowGroups => {
                self.tournament_detail_tab = TournamentTab::Overview;
            }
            AppAction::ShowMatches => {
                self.tournament_detail_tab = TournamentTab::Matches;
            }
            AppAction::ShowStandings => {
                self.tournament_detail_tab = TournamentTab::Info;
            }
            AppAction::ToggleBroadcast => {
                self.broadcast_mode = !self.broadcast_mode;
                if self.broadcast_mode {
                    self.screen = Screen::Broadcast;
                    self.ticker_offset = 0;
                } else {
                    self.screen = Screen::Dashboard;
                }
                self.scroll_offset = 0;
            }
            AppAction::SearchInput(c) => {
                self.search_query.push(c);
                self.clamp_scroll();
            }
            AppAction::SearchBackspace => {
                self.search_query.pop();
                self.clamp_scroll();
            }
            AppAction::SearchConfirm => {
                self.search_active = false;
                // query stays active as filter
            }
            AppAction::SearchCancel => {
                self.search_active = false;
                self.search_query.clear();
                self.clamp_scroll();
            }
            AppAction::ShowBracket => {
                self.tournament_detail_tab = TournamentTab::Bracket;
                self.bracket_round_offset = 0;
                self.bracket_match_offset = 0;
            }
            AppAction::ToggleBracketView => {
                if self.tournament_detail_tab == TournamentTab::Bracket {
                    if let Some(id) = &self.selected_tournament_id {
                        if let Some(bracket) = self.bracket_cache.get(id) {
                            if bracket.bracket_type == BracketType::SingleElim {
                                self.bracket_view_mode = match self.bracket_view_mode {
                                    BracketViewMode::Column => BracketViewMode::AsciiTree,
                                    BracketViewMode::AsciiTree => BracketViewMode::Column,
                                };
                            }
                        }
                    }
                }
            }
            AppAction::ToggleAllTournaments => {
                self.show_all_tournaments = !self.show_all_tournaments;
                self.scroll_offset = 0;
            }
            AppAction::NextGame => {
                if self.screen == Screen::MatchDetail {
                    if let Some(mid) = &self.selected_match_id {
                        let game_count = self
                            .match_detail_cache
                            .get(mid)
                            .map(|(d, _)| d.games.len())
                            .unwrap_or(0);
                        if game_count > 1 {
                            self.selected_game = (self.selected_game + 1) % game_count;
                        }
                    }
                }
            }
        }
    }

    fn handle_toggle_favorite(&mut self) {
        match self.screen {
            Screen::Dashboard => {
                match self.active_panel {
                    0 | 1 => {
                        // Open team picker dialog for the selected match
                        if let Some(m) = self.selected_match() {
                            self.favorite_picker = Some(FavoritePicker {
                                team_a: m.team_a.name.clone(),
                                team_b: m.team_b.name.clone(),
                                selected: 0,
                            });
                        }
                    }
                    2 => {
                        // Toggle tournament
                        if let Some(t) =
                            self.upcoming_tournaments().get(self.scroll_offset).copied()
                        {
                            let name = t.name.clone();
                            let was_fav =
                                self.config.favorite_tournaments.iter().any(|f| f == &name);
                            self.config.toggle_favorite_tournament(&name);
                            let msg = if was_fav {
                                format!("Removed {} from favorites", name)
                            } else {
                                format!("★ Added {} to favorites", name)
                            };
                            self.status_message = Some((msg, self.tick_count));
                            if let Err(e) = self.config.save() {
                                self.error_message = Some(format!("Failed to save: {}", e));
                            }
                        }
                    }
                    3 => {
                        // Open picker for favorites panel too
                        if let Some(m) = self
                            .favorite_teams_matches()
                            .get(self.scroll_offset)
                            .copied()
                        {
                            self.favorite_picker = Some(FavoritePicker {
                                team_a: m.team_a.name.clone(),
                                team_b: m.team_b.name.clone(),
                                selected: 0,
                            });
                        }
                    }
                    _ => {}
                }
            }
            Screen::TournamentBrowser => {
                if let Some(t) = self
                    .browsable_tournaments()
                    .get(self.scroll_offset)
                    .copied()
                {
                    let name = t.name.clone();
                    let was_fav = self.config.favorite_tournaments.iter().any(|f| f == &name);
                    self.config.toggle_favorite_tournament(&name);
                    let msg = if was_fav {
                        format!("Removed {} from favorites", name)
                    } else {
                        format!("★ Added {} to favorites", name)
                    };
                    self.status_message = Some((msg, self.tick_count));
                    if let Err(e) = self.config.save() {
                        self.error_message = Some(format!("Failed to save: {}", e));
                    }
                }
            }
            Screen::TournamentDetail => {
                if let Some(t) = self
                    .selected_tournament_id
                    .as_ref()
                    .and_then(|id| self.tournaments.iter().find(|t| &t.id == id))
                {
                    let name = t.name.clone();
                    let was_fav = self.config.favorite_tournaments.iter().any(|f| f == &name);
                    self.config.toggle_favorite_tournament(&name);
                    let msg = if was_fav {
                        format!("Removed {} from favorites", name)
                    } else {
                        format!("★ Added {} to favorites", name)
                    };
                    self.status_message = Some((msg, self.tick_count));
                    if let Err(e) = self.config.save() {
                        self.error_message = Some(format!("Failed to save: {}", e));
                    }
                }
            }
            _ => {} // Broadcast, Settings — no-op
        }
    }

    fn clamp_scroll(&mut self) {
        let max = self.current_panel_len().saturating_sub(1);
        if self.scroll_offset > max {
            self.scroll_offset = max;
        }
    }

    /// Returns the visible items for the live panel, respecting active_filter.
    pub fn visible_live(&self) -> Vec<&Match> {
        match self.active_filter {
            MatchFilter::UpcomingOnly => vec![],
            MatchFilter::FavoritesOnly => self
                .favorite_teams_matches()
                .into_iter()
                .filter(|m| m.status.is_live())
                .collect(),
            _ => self.live_matches(),
        }
    }

    /// Returns the visible items for the upcoming panel, respecting active_filter.
    pub fn visible_upcoming(&self) -> Vec<&Match> {
        match self.active_filter {
            MatchFilter::LiveOnly => vec![],
            MatchFilter::FavoritesOnly => self
                .favorite_teams_matches()
                .into_iter()
                .filter(|m| m.status == MatchStatus::Upcoming)
                .collect(),
            _ => self.upcoming_matches(),
        }
    }

    pub fn current_panel_len(&self) -> usize {
        match self.screen {
            Screen::Dashboard => match self.active_panel {
                0 => self.visible_live().len(),
                1 => self.visible_upcoming().len(),
                2 => self.upcoming_tournaments().len(),
                3 => self.favorite_teams_matches().len(),
                _ => 0,
            },
            Screen::TournamentBrowser => self.browsable_tournaments().len(),
            _ => 0,
        }
    }

    pub fn last_refresh_display(&self) -> String {
        let elapsed = self.last_refresh.elapsed().as_secs();
        if elapsed > 9000 {
            "never".to_string()
        } else if elapsed < 60 {
            format!("{}s ago", elapsed)
        } else {
            format!("{}m ago", elapsed / 60)
        }
    }

    pub fn data_source(&self) -> &str {
        if self.config.pandascore_api_key.is_some() {
            "PandaScore"
        } else {
            "Liquipedia"
        }
    }

    pub fn needs_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= Duration::from_secs(self.config.refresh_interval)
    }

    pub fn mark_refreshed(&mut self) {
        self.last_refresh = Instant::now();
    }

    pub fn live_matches(&self) -> Vec<&Match> {
        self.matches
            .iter()
            .filter(|m| m.status.is_live())
            .filter(|m| self.matches_search(m))
            .collect()
    }

    pub fn upcoming_matches(&self) -> Vec<&Match> {
        let mut upcoming: Vec<&Match> = self
            .matches
            .iter()
            .filter(|m| m.status == MatchStatus::Upcoming)
            .filter(|m| self.matches_search(m))
            .collect();
        upcoming.sort_by_key(|m| m.start_time);
        upcoming
    }

    pub fn upcoming_tournaments(&self) -> Vec<&Tournament> {
        let mut t: Vec<&Tournament> = self
            .tournaments
            .iter()
            .filter(|t| t.status != TournamentStatus::Completed)
            .filter(|t| {
                if self.search_query.is_empty() {
                    return true;
                }
                let q = self.search_query.to_lowercase();
                t.name.to_lowercase().contains(&q)
            })
            .collect();
        t.sort_by_key(|t| t.start_date);
        t
    }

    pub fn browsable_tournaments(&self) -> Vec<&Tournament> {
        let mut t: Vec<&Tournament> = self
            .tournaments
            .iter()
            .filter(|t| self.show_all_tournaments || t.status != TournamentStatus::Completed)
            .filter(|t| {
                if self.search_query.is_empty() {
                    return true;
                }
                let q = self.search_query.to_lowercase();
                t.name.to_lowercase().contains(&q)
            })
            .collect();
        t.sort_by_key(|t| t.start_date);
        t
    }

    pub fn favorite_teams_matches(&self) -> Vec<&Match> {
        self.matches
            .iter()
            .filter(|m| {
                let team_fav = self
                    .config
                    .favorite_teams
                    .iter()
                    .any(|fav| m.involves_team(fav));
                let tournament_fav = self
                    .config
                    .favorite_tournaments
                    .iter()
                    .any(|fav| m.tournament_name.eq_ignore_ascii_case(fav));
                team_fav || tournament_fav
            })
            .filter(|m| self.matches_search(m))
            .collect()
    }

    fn matches_search(&self, m: &Match) -> bool {
        if self.search_query.is_empty() {
            return true;
        }
        let q = self.search_query.to_lowercase();
        m.team_a.name.to_lowercase().contains(&q)
            || m.team_a.tag.to_lowercase().contains(&q)
            || m.team_b.name.to_lowercase().contains(&q)
            || m.team_b.tag.to_lowercase().contains(&q)
            || m.tournament_name.to_lowercase().contains(&q)
    }

    pub fn pending_notifications(&mut self) -> Vec<(String, NotificationEvent)> {
        if !self.config.enable_notifications {
            return Vec::new();
        }

        let now = chrono::Utc::now();
        let today = now.date_naive();
        let mut notifications = Vec::new();

        for m in &self.matches {
            let is_fav = self
                .config
                .favorite_teams
                .iter()
                .any(|fav| m.involves_team(fav));
            if !is_fav {
                continue;
            }

            // Match soon: fires when 14-15 min before start
            let secs_until = (m.start_time - now).num_seconds();
            if (840..900).contains(&secs_until) {
                let key = NotificationKey {
                    match_or_tournament_id: m.id.clone(),
                    event: NotificationEvent::MatchSoon,
                };
                if self.notified_events.insert(key) {
                    notifications.push((
                        format!(
                            "{} vs {} starts in 15 minutes! ({})",
                            m.team_a.name, m.team_b.name, m.tournament_name
                        ),
                        NotificationEvent::MatchSoon,
                    ));
                }
            }

            // Match live
            if m.status.is_live() {
                let key = NotificationKey {
                    match_or_tournament_id: m.id.clone(),
                    event: NotificationEvent::MatchLive,
                };
                if self.notified_events.insert(key) {
                    notifications.push((
                        format!(
                            "{} vs {} is LIVE! ({})",
                            m.team_a.name, m.team_b.name, m.tournament_name
                        ),
                        NotificationEvent::MatchLive,
                    ));
                }
            }
        }

        // Tournament today
        for t in &self.tournaments {
            if t.start_date.date_naive() == today {
                let key = NotificationKey {
                    match_or_tournament_id: t.id.clone(),
                    event: NotificationEvent::TournamentToday,
                };
                if self.notified_events.insert(key) {
                    notifications.push((
                        format!("{} starts today!", t.name),
                        NotificationEvent::TournamentToday,
                    ));
                }
            }
        }

        notifications
    }

    pub fn selected_match(&self) -> Option<&Match> {
        match self.screen {
            Screen::Dashboard => match self.active_panel {
                0 => self.visible_live().get(self.scroll_offset).copied(),
                1 => self.visible_upcoming().get(self.scroll_offset).copied(),
                3 => self
                    .favorite_teams_matches()
                    .get(self.scroll_offset)
                    .copied(),
                _ => None,
            },
            Screen::TournamentDetail if self.tournament_detail_tab == TournamentTab::Matches => {
                self.selected_tournament_id.as_ref().and_then(|tid| {
                    let matches: Vec<&Match> = self
                        .matches
                        .iter()
                        .filter(|m| m.tournament_id == *tid)
                        .collect();
                    matches.get(self.scroll_offset).copied()
                })
            }
            _ => None,
        }
    }

    fn bracket_round_count(&self) -> Option<usize> {
        self.selected_tournament_id
            .as_ref()
            .and_then(|id| self.bracket_cache.get(id))
            .map(|b| b.upper_rounds.len())
    }

    fn bracket_round_match_count(&self) -> Option<usize> {
        self.selected_tournament_id
            .as_ref()
            .and_then(|id| self.bracket_cache.get(id))
            .and_then(|b| b.upper_rounds.get(self.bracket_round_offset))
            .map(|r| r.matches.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        App::new(Config::default())
    }

    #[test]
    fn tick_count_starts_at_zero() {
        let app = test_app();
        assert_eq!(app.tick_count, 0);
    }

    #[test]
    fn toggle_broadcast_mode() {
        let mut app = test_app();
        assert!(!app.broadcast_mode);
        assert_eq!(app.screen, Screen::Dashboard);

        app.handle_action(AppAction::ToggleBroadcast);
        assert!(app.broadcast_mode);
        assert_eq!(app.screen, Screen::Broadcast);

        app.handle_action(AppAction::ToggleBroadcast);
        assert!(!app.broadcast_mode);
        assert_eq!(app.screen, Screen::Dashboard);
    }

    #[test]
    fn notification_dedup() {
        let mut app = test_app();
        app.config.enable_notifications = true;
        app.config.favorite_teams = vec!["Team A".into()];
        app.matches.push(Match {
            id: "m1".into(),
            team_a: Team {
                name: "Team A".into(),
                tag: "TA".into(),
                region: None,
            },
            team_b: Team {
                name: "Team B".into(),
                tag: "TB".into(),
                region: None,
            },
            score_a: 0,
            score_b: 0,
            status: MatchStatus::Live,
            series_format: SeriesFormat::Bo3,
            tournament_name: "Test Cup".into(),
            tournament_id: "t1".into(),
            start_time: chrono::Utc::now(),
            stream_url: None,
            game_time_secs: None,
            stage: None,
        });
        let first = app.pending_notifications();
        assert_eq!(first.len(), 1);
        let second = app.pending_notifications();
        assert_eq!(second.len(), 0); // deduped
    }

    fn test_tournament() -> Tournament {
        Tournament {
            id: "esl-one-2026".into(),
            name: "ESL One 2026".into(),
            start_date: chrono::Utc::now() + chrono::Duration::days(1),
            end_date: chrono::Utc::now() + chrono::Duration::days(5),
            status: TournamentStatus::Upcoming,
            tier: "1".into(),
            location: None,
            prize_pool: None,
        }
    }

    fn test_match(status: MatchStatus) -> Match {
        Match {
            id: "m1".into(),
            team_a: Team {
                name: "Team Liquid".into(),
                tag: "TL".into(),
                region: None,
            },
            team_b: Team {
                name: "OG".into(),
                tag: "OG".into(),
                region: None,
            },
            score_a: 1,
            score_b: 0,
            status,
            series_format: SeriesFormat::Bo3,
            tournament_name: "ESL".into(),
            tournament_id: "t1".into(),
            start_time: chrono::Utc::now() + chrono::Duration::hours(1),
            stream_url: None,
            game_time_secs: None,
            stage: None,
        }
    }

    #[test]
    fn scroll_offset_clamped() {
        let mut app = test_app();
        app.matches.push(test_match(MatchStatus::Live));
        // Panel 0 has 1 live match, scroll should clamp at 0
        app.active_panel = 0;
        app.handle_action(AppAction::ScrollDown);
        assert_eq!(app.scroll_offset, 0); // clamped at max (1-1=0)
    }

    #[test]
    fn filter_cycling() {
        let mut app = test_app();
        assert_eq!(app.active_filter, MatchFilter::All);
        app.handle_action(AppAction::OpenFilter);
        assert_eq!(app.active_filter, MatchFilter::LiveOnly);
        app.handle_action(AppAction::OpenFilter);
        assert_eq!(app.active_filter, MatchFilter::UpcomingOnly);
        app.handle_action(AppAction::OpenFilter);
        assert_eq!(app.active_filter, MatchFilter::FavoritesOnly);
        app.handle_action(AppAction::OpenFilter);
        assert_eq!(app.active_filter, MatchFilter::All);
    }

    #[test]
    fn search_input_and_cancel() {
        let mut app = test_app();
        app.handle_action(AppAction::OpenSearch);
        assert!(app.search_active);
        assert!(app.search_query.is_empty());

        app.handle_action(AppAction::SearchInput('l'));
        app.handle_action(AppAction::SearchInput('i'));
        assert_eq!(app.search_query, "li");

        app.handle_action(AppAction::SearchBackspace);
        assert_eq!(app.search_query, "l");

        app.handle_action(AppAction::SearchCancel);
        assert!(!app.search_active);
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn search_confirm_keeps_query() {
        let mut app = test_app();
        app.handle_action(AppAction::OpenSearch);
        app.handle_action(AppAction::SearchInput('t'));
        app.handle_action(AppAction::SearchConfirm);
        assert!(!app.search_active);
        assert_eq!(app.search_query, "t");
    }

    #[test]
    fn search_filters_matches() {
        let mut app = test_app();
        app.matches.push(test_match(MatchStatus::Live));
        assert_eq!(app.live_matches().len(), 1);

        app.search_query = "nonexistent".into();
        assert_eq!(app.live_matches().len(), 0);

        app.search_query = "liquid".into();
        assert_eq!(app.live_matches().len(), 1);
    }

    #[test]
    fn favorite_toggle_opens_picker() {
        let mut app = test_app();
        app.matches.push(test_match(MatchStatus::Live));
        app.active_panel = 0;
        // First press opens the picker
        app.handle_action(AppAction::ToggleFavorite);
        assert!(app.favorite_picker.is_some());
        assert_eq!(app.favorite_picker.as_ref().unwrap().team_a, "Team Liquid");
        // Select confirms (toggles team_a)
        app.handle_action(AppAction::Select);
        assert!(app.favorite_picker.is_none());
        assert!(app.status_message.is_some());
        assert!(app
            .config
            .favorite_teams
            .contains(&"Team Liquid".to_string()));
    }

    #[test]
    fn favorite_picker_selects_team_b() {
        let mut app = test_app();
        app.matches.push(test_match(MatchStatus::Live));
        app.active_panel = 0;
        app.handle_action(AppAction::ToggleFavorite);
        app.handle_action(AppAction::ScrollDown); // move to team_b
        assert_eq!(app.favorite_picker.as_ref().unwrap().selected, 1);
        app.handle_action(AppAction::Select);
        assert!(app.config.favorite_teams.contains(&"OG".to_string()));
        assert!(!app
            .config
            .favorite_teams
            .contains(&"Team Liquid".to_string()));
    }

    #[test]
    fn favorite_picker_esc_cancels() {
        let mut app = test_app();
        app.matches.push(test_match(MatchStatus::Live));
        app.active_panel = 0;
        app.handle_action(AppAction::ToggleFavorite);
        assert!(app.favorite_picker.is_some());
        app.handle_action(AppAction::Back);
        assert!(app.favorite_picker.is_none());
        assert!(app.config.favorite_teams.is_empty());
    }

    #[test]
    fn backspace_on_empty_search_is_noop() {
        let mut app = test_app();
        app.handle_action(AppAction::OpenSearch);
        app.handle_action(AppAction::SearchBackspace);
        assert!(app.search_query.is_empty()); // no panic
    }

    #[test]
    fn tournament_tab_switching() {
        let mut app = test_app();
        assert_eq!(app.tournament_detail_tab, TournamentTab::Overview);
        app.handle_action(AppAction::ShowMatches);
        assert_eq!(app.tournament_detail_tab, TournamentTab::Matches);
        app.handle_action(AppAction::ShowStandings);
        assert_eq!(app.tournament_detail_tab, TournamentTab::Info);
    }

    #[test]
    fn no_notifications_when_disabled() {
        let mut app = test_app();
        app.config.enable_notifications = false;
        app.config.favorite_teams = vec!["Team A".into()];
        app.matches.push(Match {
            id: "m1".into(),
            team_a: Team {
                name: "Team A".into(),
                tag: "TA".into(),
                region: None,
            },
            team_b: Team {
                name: "Team B".into(),
                tag: "TB".into(),
                region: None,
            },
            score_a: 0,
            score_b: 0,
            status: MatchStatus::Live,
            series_format: SeriesFormat::Bo3,
            tournament_name: "Test Cup".into(),
            tournament_id: "t1".into(),
            start_time: chrono::Utc::now(),
            stream_url: None,
            game_time_secs: None,
            stage: None,
        });
        assert_eq!(app.pending_notifications().len(), 0);
    }

    #[test]
    fn select_tournament_stores_id() {
        let mut app = test_app();
        app.tournaments.push(test_tournament());
        app.screen = Screen::TournamentBrowser;
        app.scroll_offset = 0;
        app.handle_action(AppAction::Select);
        assert_eq!(app.screen, Screen::TournamentDetail);
        assert_eq!(app.selected_tournament_id, Some("esl-one-2026".to_string()));
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn selected_tournament_survives_reorder() {
        let mut app = test_app();
        // Add two tournaments — the second one sorts first by start_date
        app.tournaments.push(test_tournament()); // starts in 1 day
        app.tournaments.push(Tournament {
            id: "ti-2026".into(),
            name: "TI 2026".into(),
            start_date: chrono::Utc::now() + chrono::Duration::days(3),
            end_date: chrono::Utc::now() + chrono::Duration::days(10),
            status: TournamentStatus::Upcoming,
            tier: "S".into(),
            location: None,
            prize_pool: None,
        });
        // Select the first item in the browser (esl-one-2026, earliest start_date)
        app.screen = Screen::TournamentBrowser;
        app.scroll_offset = 0;
        app.handle_action(AppAction::Select);
        assert_eq!(app.selected_tournament_id, Some("esl-one-2026".to_string()));
        // Simulate a data refresh that reverses the order in self.tournaments
        app.tournaments.reverse();
        // The ID-based lookup should still find the correct tournament
        let found = app
            .selected_tournament_id
            .as_ref()
            .and_then(|id| app.tournaments.iter().find(|t| &t.id == id));
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "ESL One 2026");
    }

    #[test]
    fn bracket_scroll_horizontal() {
        let mut app = test_app();
        app.screen = Screen::TournamentDetail;
        app.tournament_detail_tab = TournamentTab::Bracket;
        app.selected_tournament_id = Some("t1".into());

        app.bracket_cache.insert(
            "t1".into(),
            Bracket {
                bracket_type: BracketType::SingleElim,
                upper_rounds: vec![
                    BracketRound {
                        round: 1,
                        name: "R1".into(),
                        matches: vec![],
                    },
                    BracketRound {
                        round: 2,
                        name: "SF".into(),
                        matches: vec![],
                    },
                    BracketRound {
                        round: 3,
                        name: "F".into(),
                        matches: vec![],
                    },
                ],
                lower_rounds: None,
                grand_final: None,
            },
        );

        assert_eq!(app.bracket_round_offset, 0);
        app.handle_action(AppAction::NextPanel);
        assert_eq!(app.bracket_round_offset, 1);
        app.handle_action(AppAction::NextPanel);
        assert_eq!(app.bracket_round_offset, 2);
        app.handle_action(AppAction::NextPanel);
        assert_eq!(app.bracket_round_offset, 0); // wraps
    }

    #[test]
    fn all_tournaments_includes_completed() {
        let mut app = test_app();
        let mut t1 = test_tournament();
        t1.status = TournamentStatus::Completed;
        t1.id = "completed-1".into();
        let t2 = test_tournament();
        app.tournaments = vec![t1, t2];
        assert_eq!(app.browsable_tournaments().len(), 1);
        app.show_all_tournaments = true;
        assert_eq!(app.browsable_tournaments().len(), 2);
    }

    // === Match Detail Tests ===

    #[test]
    fn select_opens_match_detail_from_live_panel() {
        let mut app = test_app();
        app.matches.push(test_match(MatchStatus::Live));
        app.active_panel = 0;
        app.handle_action(AppAction::Select);
        assert_eq!(app.screen, Screen::MatchDetail);
        assert_eq!(app.selected_match_id, Some("m1".to_string()));
        assert_eq!(app.previous_screen, Some(Screen::Dashboard));
    }

    #[test]
    fn select_opens_match_detail_from_upcoming_panel() {
        let mut app = test_app();
        app.matches.push(test_match(MatchStatus::Upcoming));
        app.active_panel = 1;
        app.handle_action(AppAction::Select);
        assert_eq!(app.screen, Screen::MatchDetail);
        assert_eq!(app.selected_match_id, Some("m1".to_string()));
    }

    #[test]
    fn select_opens_match_detail_from_favorites_panel() {
        let mut app = test_app();
        app.config.favorite_teams = vec!["Team Liquid".into()];
        app.matches.push(test_match(MatchStatus::Live));
        app.active_panel = 3;
        app.handle_action(AppAction::Select);
        assert_eq!(app.screen, Screen::MatchDetail);
        assert_eq!(app.selected_match_id, Some("m1".to_string()));
    }

    #[test]
    fn select_noop_when_no_match() {
        let mut app = test_app();
        // No matches loaded, panel 0
        app.active_panel = 0;
        app.handle_action(AppAction::Select);
        assert_eq!(app.screen, Screen::Dashboard); // unchanged
        assert_eq!(app.selected_match_id, None);
    }

    #[test]
    fn next_game_cycles() {
        let mut app = test_app();
        app.screen = Screen::MatchDetail;
        app.selected_match_id = Some("m1".into());
        app.match_detail_cache.insert(
            "m1".into(),
            (
                MatchDetailData {
                    games: vec![
                        GameDetail {
                            game_number: 1,
                            status: MatchStatus::Completed,
                            winner: Some("TL".into()),
                            duration: None,
                        },
                        GameDetail {
                            game_number: 2,
                            status: MatchStatus::Completed,
                            winner: Some("OG".into()),
                            duration: None,
                        },
                        GameDetail {
                            game_number: 3,
                            status: MatchStatus::Live,
                            winner: None,
                            duration: None,
                        },
                    ],
                    fetch_status: FetchStatus::Ready,
                },
                Instant::now(),
            ),
        );
        assert_eq!(app.selected_game, 0);
        app.handle_action(AppAction::NextGame);
        assert_eq!(app.selected_game, 1);
        app.handle_action(AppAction::NextGame);
        assert_eq!(app.selected_game, 2);
        app.handle_action(AppAction::NextGame);
        assert_eq!(app.selected_game, 0); // wraps
    }

    #[test]
    fn next_game_noop_on_bo1() {
        let mut app = test_app();
        app.screen = Screen::MatchDetail;
        app.selected_match_id = Some("m1".into());
        app.match_detail_cache.insert(
            "m1".into(),
            (
                MatchDetailData {
                    games: vec![GameDetail {
                        game_number: 1,
                        status: MatchStatus::Live,
                        winner: None,
                        duration: None,
                    }],
                    fetch_status: FetchStatus::Ready,
                },
                Instant::now(),
            ),
        );
        app.handle_action(AppAction::NextGame);
        assert_eq!(app.selected_game, 0); // stays at 0
    }

    #[test]
    fn back_restores_previous_screen() {
        let mut app = test_app();
        app.screen = Screen::MatchDetail;
        app.previous_screen = Some(Screen::TournamentDetail);
        app.selected_match_id = Some("m1".into());
        app.handle_action(AppAction::Back);
        assert_eq!(app.screen, Screen::TournamentDetail);
        assert_eq!(app.selected_match_id, None);
        assert_eq!(app.previous_screen, None); // consumed
    }

    #[test]
    fn selected_match_works_from_favorites() {
        let mut app = test_app();
        app.config.favorite_teams = vec!["Team Liquid".into()];
        app.matches.push(test_match(MatchStatus::Live));
        app.active_panel = 3;
        let m = app.selected_match();
        assert!(m.is_some());
        assert_eq!(m.unwrap().team_a.name, "Team Liquid");
    }

    #[test]
    fn select_opens_match_detail_from_tournament_detail() {
        let mut app = test_app();
        app.tournaments.push(test_tournament());
        app.selected_tournament_id = Some("esl-one-2026".into());
        let mut m = test_match(MatchStatus::Live);
        m.tournament_id = "esl-one-2026".into();
        app.matches.push(m);
        app.screen = Screen::TournamentDetail;
        app.tournament_detail_tab = TournamentTab::Matches;
        app.scroll_offset = 0;
        app.handle_action(AppAction::Select);
        assert_eq!(app.screen, Screen::MatchDetail);
        assert_eq!(app.selected_match_id, Some("m1".to_string()));
        assert_eq!(app.previous_screen, Some(Screen::TournamentDetail));
    }

    #[test]
    fn refresh_invalidates_match_detail_cache() {
        let mut app = test_app();
        app.screen = Screen::MatchDetail;
        app.selected_match_id = Some("m1".into());
        app.selected_game = 2;
        app.match_detail_cache.insert(
            "m1".into(),
            (
                MatchDetailData {
                    games: vec![],
                    fetch_status: FetchStatus::Ready,
                },
                Instant::now(),
            ),
        );
        assert!(app.match_detail_cache.contains_key("m1"));
        app.handle_action(AppAction::Refresh);
        assert!(!app.match_detail_cache.contains_key("m1"));
        assert_eq!(app.selected_game, 0);
    }

    #[test]
    fn back_from_match_detail_resets_loading_flag() {
        let mut app = test_app();
        app.screen = Screen::MatchDetail;
        app.previous_screen = Some(Screen::Dashboard);
        app.selected_match_id = Some("m1".into());
        app.match_detail_loading = true;
        app.handle_action(AppAction::Back);
        assert_eq!(app.screen, Screen::Dashboard);
        assert!(!app.match_detail_loading);
    }
}
