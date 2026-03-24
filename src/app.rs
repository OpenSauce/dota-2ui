use crate::config::Config;
use crate::input::{AppAction, Screen};
use crate::models::*;
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

pub struct App {
    pub screen: Screen,
    pub config: Config,
    pub matches: Vec<Match>,
    pub tournaments: Vec<Tournament>,
    pub selected_tournament: Option<usize>,
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
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            screen: Screen::Dashboard,
            config,
            matches: Vec::new(),
            tournaments: Vec::new(),
            selected_tournament: None,
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
        }
    }

    pub fn handle_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => {
                if self.search_active {
                    self.search_active = false;
                    self.search_query.clear();
                } else {
                    match self.screen {
                        Screen::Dashboard => self.should_quit = true,
                        _ => {
                            self.screen = Screen::Dashboard;
                            self.scroll_offset = 0;
                        }
                    }
                }
            }
            AppAction::ScrollUp => { self.scroll_offset = self.scroll_offset.saturating_sub(1); }
            AppAction::ScrollDown => { self.scroll_offset += 1; }
            AppAction::NextPanel => {
                self.active_panel = (self.active_panel + 1) % 4;
                self.scroll_offset = 0;
            }
            AppAction::Select => {
                if self.screen == Screen::TournamentBrowser {
                    self.selected_tournament = Some(self.scroll_offset);
                    self.screen = Screen::TournamentDetail;
                    self.scroll_offset = 0;
                }
            }
            AppAction::ToggleFavorite => {}
            AppAction::Refresh => {
                self.last_refresh = Instant::now() - Duration::from_secs(9999);
            }
            AppAction::OpenTournaments => {
                self.screen = Screen::TournamentBrowser;
                self.scroll_offset = 0;
            }
            AppAction::OpenFilter => {}
            AppAction::OpenSearch => {
                self.search_active = !self.search_active;
                if !self.search_active { self.search_query.clear(); }
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
            AppAction::ShowGroups | AppAction::ShowMatches | AppAction::ShowStandings => {}
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
        self.matches.iter().filter(|m| m.status.is_live()).collect()
    }

    pub fn upcoming_matches(&self) -> Vec<&Match> {
        let mut upcoming: Vec<&Match> = self.matches.iter()
            .filter(|m| m.status == MatchStatus::Upcoming).collect();
        upcoming.sort_by_key(|m| m.start_time);
        upcoming
    }

    pub fn upcoming_tournaments(&self) -> Vec<&Tournament> {
        let mut t: Vec<&Tournament> = self.tournaments.iter()
            .filter(|t| t.status != TournamentStatus::Completed).collect();
        t.sort_by_key(|t| t.start_date);
        t
    }

    pub fn favorite_teams_matches(&self) -> Vec<&Match> {
        self.matches.iter().filter(|m| {
            self.config.favorite_teams.iter().any(|fav| {
                m.team_a.name.eq_ignore_ascii_case(fav) || m.team_b.name.eq_ignore_ascii_case(fav)
            })
        }).collect()
    }

    pub fn pending_notifications(&mut self) -> Vec<(String, NotificationEvent)> {
        if !self.config.enable_notifications { return Vec::new(); }

        let now = chrono::Utc::now();
        let today = now.date_naive();
        let mut notifications = Vec::new();

        for m in &self.matches {
            let is_fav = self.config.favorite_teams.iter().any(|fav| {
                m.team_a.name.eq_ignore_ascii_case(fav) || m.team_b.name.eq_ignore_ascii_case(fav)
            });
            if !is_fav { continue; }

            // Match soon: fires when 14-15 min before start
            let secs_until = (m.start_time - now).num_seconds();
            if secs_until >= 840 && secs_until < 900 {
                let key = NotificationKey {
                    match_or_tournament_id: m.id.clone(),
                    event: NotificationEvent::MatchSoon,
                };
                if self.notified_events.insert(key) {
                    notifications.push((
                        format!("{} vs {} starts in 15 minutes! ({})", m.team_a.name, m.team_b.name, m.tournament_name),
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
                        format!("{} vs {} is LIVE! ({})", m.team_a.name, m.team_b.name, m.tournament_name),
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

    fn selected_match(&self) -> Option<&Match> {
        match self.active_panel {
            0 => self.live_matches().get(self.scroll_offset).copied(),
            1 => self.upcoming_matches().get(self.scroll_offset).copied(),
            _ => None,
        }
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
            team_a: Team { name: "Team A".into(), tag: "TA".into(), region: None },
            team_b: Team { name: "Team B".into(), tag: "TB".into(), region: None },
            score_a: 0, score_b: 0,
            status: MatchStatus::Live,
            series_format: SeriesFormat::Bo3,
            tournament_name: "Test Cup".into(),
            tournament_id: "t1".into(),
            start_time: chrono::Utc::now(),
            stream_url: None,
            game_time_secs: None,
        });
        let first = app.pending_notifications();
        assert_eq!(first.len(), 1);
        let second = app.pending_notifications();
        assert_eq!(second.len(), 0); // deduped
    }

    #[test]
    fn no_notifications_when_disabled() {
        let mut app = test_app();
        app.config.enable_notifications = false;
        app.config.favorite_teams = vec!["Team A".into()];
        app.matches.push(Match {
            id: "m1".into(),
            team_a: Team { name: "Team A".into(), tag: "TA".into(), region: None },
            team_b: Team { name: "Team B".into(), tag: "TB".into(), region: None },
            score_a: 0, score_b: 0,
            status: MatchStatus::Live,
            series_format: SeriesFormat::Bo3,
            tournament_name: "Test Cup".into(),
            tournament_id: "t1".into(),
            start_time: chrono::Utc::now(),
            stream_url: None,
            game_time_secs: None,
        });
        assert_eq!(app.pending_notifications().len(), 0);
    }
}
