use dota_2ui::models::*;
use chrono::{Utc, Duration};
use ratatui::style::Color;

#[test]
fn test_countdown_ratio() {
    let far_away = Tournament {
        id: "1".into(), name: "Test".into(),
        start_date: Utc::now() + Duration::days(5),
        end_date: Utc::now() + Duration::days(7),
        status: TournamentStatus::Upcoming, tier: "1".into(),
        location: None, prize_pool: None,
    };
    assert_eq!(far_away.countdown_ratio(), 0.0);

    let soon = Tournament {
        id: "1".into(), name: "Test".into(),
        start_date: Utc::now() + Duration::hours(12),
        end_date: Utc::now() + Duration::days(7),
        status: TournamentStatus::Upcoming, tier: "1".into(),
        location: None, prize_pool: None,
    };
    let ratio = soon.countdown_ratio();
    assert!(ratio > 0.4 && ratio < 0.6, "12h away should be ~0.5, got {}", ratio);

    let started = Tournament {
        id: "1".into(), name: "Test".into(),
        start_date: Utc::now() + Duration::days(5),
        end_date: Utc::now() + Duration::days(7),
        status: TournamentStatus::Live, tier: "1".into(),
        location: None, prize_pool: None,
    };
    assert_eq!(started.countdown_ratio(), 1.0);
}

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

#[test]
fn test_tier_color_mapping() {
    let make_tournament = |tier: &str| Tournament {
        id: "t".into(),
        name: "T".into(),
        start_date: Utc::now(),
        end_date: Utc::now(),
        status: TournamentStatus::Upcoming,
        tier: tier.into(),
        location: None,
        prize_pool: None,
    };

    // Tier 1 / S-Tier / Major → Yellow
    assert_eq!(make_tournament("1").tier_color(), Color::Yellow);
    assert_eq!(make_tournament("S-Tier").tier_color(), Color::Yellow);
    assert_eq!(make_tournament("Major").tier_color(), Color::Yellow);

    // Tier 2 / A-Tier / Minor → Gray
    assert_eq!(make_tournament("2").tier_color(), Color::Gray);
    assert_eq!(make_tournament("A-Tier").tier_color(), Color::Gray);
    assert_eq!(make_tournament("Minor").tier_color(), Color::Gray);

    // Tier 3 / B-Tier / Qualifier → White
    assert_eq!(make_tournament("3").tier_color(), Color::White);
    assert_eq!(make_tournament("B-Tier").tier_color(), Color::White);
    assert_eq!(make_tournament("Qualifier").tier_color(), Color::White);

    // Unknown → DarkGray
    assert_eq!(make_tournament("unknown").tier_color(), Color::DarkGray);
    assert_eq!(make_tournament("").tier_color(), Color::DarkGray);
}
