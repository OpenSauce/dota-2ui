use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    TournamentBrowser,
    TournamentDetail,
    Settings,
    Broadcast,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchFilter {
    All,
    LiveOnly,
    UpcomingOnly,
    FavoritesOnly,
}

impl MatchFilter {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::LiveOnly,
            Self::LiveOnly => Self::UpcomingOnly,
            Self::UpcomingOnly => Self::FavoritesOnly,
            Self::FavoritesOnly => Self::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::LiveOnly => "Live Only",
            Self::UpcomingOnly => "Upcoming Only",
            Self::FavoritesOnly => "Favorites Only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TournamentTab {
    Overview,
    Matches,
    Bracket,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppAction {
    Quit,
    Back,
    ScrollUp,
    ScrollDown,
    Select,
    NextPanel,
    ToggleFavorite,
    Refresh,
    OpenTournaments,
    OpenFilter,
    OpenSearch,
    OpenSettings,
    OpenStream,
    ShowGroups,
    ShowMatches,
    ShowStandings,
    ToggleBroadcast,
    SearchInput(char),
    SearchBackspace,
    SearchConfirm,
    SearchCancel,
    ShowBracket,
    ToggleBracketView,
    ToggleAllTournaments,
}

pub fn map_key(key: KeyEvent, screen: &Screen, search_active: bool) -> Option<AppAction> {
    // Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(AppAction::Quit);
    }

    // When search is active, route keys to search actions
    if search_active {
        return match key.code {
            KeyCode::Esc => Some(AppAction::SearchCancel),
            KeyCode::Enter => Some(AppAction::SearchConfirm),
            KeyCode::Backspace => Some(AppAction::SearchBackspace),
            KeyCode::Char(c) => Some(AppAction::SearchInput(c)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('q') => Some(AppAction::Quit),
        KeyCode::Esc | KeyCode::Backspace => Some(AppAction::Back),
        KeyCode::Char('j') | KeyCode::Down => Some(AppAction::ScrollDown),
        KeyCode::Char('k') | KeyCode::Up => Some(AppAction::ScrollUp),
        KeyCode::Enter => Some(AppAction::Select),
        KeyCode::Tab => Some(AppAction::NextPanel),
        KeyCode::Char('s') => Some(AppAction::ToggleFavorite),
        KeyCode::Char('r') => Some(AppAction::Refresh),
        KeyCode::Char('t') => Some(AppAction::OpenTournaments),
        KeyCode::Char('f') => Some(AppAction::OpenFilter),
        KeyCode::Char('/') => Some(AppAction::OpenSearch),
        KeyCode::Char('o') => Some(AppAction::OpenStream),
        KeyCode::Char(',') => Some(AppAction::OpenSettings),
        KeyCode::Char('g') if *screen == Screen::TournamentDetail => Some(AppAction::ShowGroups),
        KeyCode::Char('m') if *screen == Screen::TournamentDetail => Some(AppAction::ShowMatches),
        KeyCode::Char('d') if *screen == Screen::TournamentDetail => Some(AppAction::ShowStandings),
        KeyCode::Char('b') if *screen == Screen::TournamentDetail => Some(AppAction::ShowBracket),
        KeyCode::Char('v') if *screen == Screen::TournamentDetail => Some(AppAction::ToggleBracketView),
        KeyCode::Char('a') if *screen == Screen::TournamentBrowser => Some(AppAction::ToggleAllTournaments),
        KeyCode::Char('1') if *screen == Screen::TournamentDetail => Some(AppAction::ShowGroups),
        KeyCode::Char('2') if *screen == Screen::TournamentDetail => Some(AppAction::ShowMatches),
        KeyCode::Char('3') if *screen == Screen::TournamentDetail => Some(AppAction::ShowBracket),
        KeyCode::Char('4') if *screen == Screen::TournamentDetail => Some(AppAction::ShowStandings),
        KeyCode::Char('b') if *screen == Screen::Dashboard || *screen == Screen::Broadcast => {
            Some(AppAction::ToggleBroadcast)
        }
        _ => None,
    }
}
