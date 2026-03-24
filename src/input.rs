use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    TournamentBrowser,
    TournamentDetail,
    Settings,
    Broadcast,
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
}

pub fn map_key(key: KeyEvent, screen: &Screen) -> Option<AppAction> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(AppAction::Quit);
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
        KeyCode::Char('b') if *screen == Screen::Dashboard || *screen == Screen::Broadcast => {
            Some(AppAction::ToggleBroadcast)
        }
        _ => None,
    }
}
