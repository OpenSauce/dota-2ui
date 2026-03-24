use crate::config::Config;
use crate::input::{AppAction, Screen};
use crate::models::*;
use std::time::{Duration, Instant};

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
}
