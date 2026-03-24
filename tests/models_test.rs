use dota_2ui::models::*;
use chrono::{Utc, Duration};

#[test]
fn test_match_status_is_live() {
    assert!(MatchStatus::Live.is_live());
    assert!(!MatchStatus::Upcoming.is_live());
    assert!(!MatchStatus::Completed.is_live());
}

#[test]
fn test_series_format_display() {
    assert_eq!(SeriesFormat::Bo1.to_string(), "Bo1");
    assert_eq!(SeriesFormat::Bo3.to_string(), "Bo3");
}

#[test]
fn test_tournament_countdown_future() {
    let future = Utc::now() + Duration::days(5) + Duration::hours(12);
    let t = Tournament {
        id: "t".into(), name: "T".into(), start_date: future,
        end_date: future + Duration::days(10), status: TournamentStatus::Upcoming,
        tier: "S".into(), location: None, prize_pool: None,
    };
    let (days, hours, _, _) = t.countdown().unwrap();
    assert_eq!(days, 5);
    assert!(hours >= 11 && hours <= 12);
}

#[test]
fn test_tournament_countdown_live_returns_none() {
    let t = Tournament {
        id: "t".into(), name: "T".into(),
        start_date: Utc::now() - Duration::days(1),
        end_date: Utc::now() + Duration::days(1),
        status: TournamentStatus::Live,
        tier: "S".into(), location: None, prize_pool: None,
    };
    assert!(t.countdown().is_none());
}
