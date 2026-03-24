use chrono::{Duration, Utc};
use dota_2ui::models::*;
use ratatui::style::Color;

#[test]
fn test_countdown_ratio() {
    let far_away = Tournament {
        id: "1".into(),
        name: "Test".into(),
        start_date: Utc::now() + Duration::days(5),
        end_date: Utc::now() + Duration::days(7),
        status: TournamentStatus::Upcoming,
        tier: "1".into(),
        location: None,
        prize_pool: None,
    };
    assert_eq!(far_away.countdown_ratio(), 0.0);

    let soon = Tournament {
        id: "1".into(),
        name: "Test".into(),
        start_date: Utc::now() + Duration::hours(12),
        end_date: Utc::now() + Duration::days(7),
        status: TournamentStatus::Upcoming,
        tier: "1".into(),
        location: None,
        prize_pool: None,
    };
    let ratio = soon.countdown_ratio();
    assert!(
        ratio > 0.4 && ratio < 0.6,
        "12h away should be ~0.5, got {}",
        ratio
    );

    let started = Tournament {
        id: "1".into(),
        name: "Test".into(),
        start_date: Utc::now() + Duration::days(5),
        end_date: Utc::now() + Duration::days(7),
        status: TournamentStatus::Live,
        tier: "1".into(),
        location: None,
        prize_pool: None,
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
        id: "t".into(),
        name: "T".into(),
        start_date: future,
        end_date: future + Duration::days(10),
        status: TournamentStatus::Upcoming,
        tier: "S".into(),
        location: None,
        prize_pool: None,
    };
    let (days, hours, _, _) = t.countdown().unwrap();
    assert_eq!(days, 5);
    assert!((11..=12).contains(&hours));
}

#[test]
fn test_tournament_countdown_live_returns_none() {
    let t = Tournament {
        id: "t".into(),
        name: "T".into(),
        start_date: Utc::now() - Duration::days(1),
        end_date: Utc::now() + Duration::days(1),
        status: TournamentStatus::Live,
        tier: "S".into(),
        location: None,
        prize_pool: None,
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

fn make_match(status: MatchStatus, start_offset_secs: i64) -> Match {
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
        score_b: 2,
        status,
        series_format: SeriesFormat::Bo3,
        tournament_name: "ESL".into(),
        tournament_id: "t1".into(),
        start_time: Utc::now() + Duration::seconds(start_offset_secs),
        stream_url: None,
        game_time_secs: None,
        stage: None,
    }
}

#[test]
fn test_relative_time_upcoming() {
    assert_eq!(make_match(MatchStatus::Upcoming, 30).relative_time(), "now");
    // Use offsets well within bucket boundaries to avoid race conditions
    assert_eq!(
        make_match(MatchStatus::Upcoming, 600).relative_time(),
        "in 9m"
    );
    assert_eq!(
        make_match(MatchStatus::Upcoming, 7200).relative_time(),
        "in 1h 59m"
    );
    assert_eq!(
        make_match(MatchStatus::Upcoming, 93600).relative_time(),
        "in 1d 1h"
    );
    assert_eq!(
        make_match(MatchStatus::Upcoming, 700000).relative_time(),
        "in 8d"
    );
}

#[test]
fn test_relative_time_stale() {
    // Upcoming but start_time in the past
    assert_eq!(
        make_match(MatchStatus::Upcoming, -60).relative_time(),
        "starting soon"
    );
}

#[test]
fn test_relative_time_live_and_completed() {
    assert_eq!(make_match(MatchStatus::Live, 0).relative_time(), "LIVE");
    assert_eq!(
        make_match(MatchStatus::Completed, -3600).relative_time(),
        "Final"
    );
}

#[test]
fn test_urgency_color() {
    // Live → Red
    assert_eq!(make_match(MatchStatus::Live, 0).urgency_color(), Color::Red);
    // Completed → DarkGray
    assert_eq!(
        make_match(MatchStatus::Completed, -3600).urgency_color(),
        Color::DarkGray
    );
    // Upcoming <15m → Red
    assert_eq!(
        make_match(MatchStatus::Upcoming, 600).urgency_color(),
        Color::Red
    );
    // Upcoming <2h → Yellow
    assert_eq!(
        make_match(MatchStatus::Upcoming, 3600).urgency_color(),
        Color::Yellow
    );
    // Upcoming >=2h → DarkGray
    assert_eq!(
        make_match(MatchStatus::Upcoming, 10000).urgency_color(),
        Color::DarkGray
    );
}

#[test]
fn test_involves_team() {
    let m = make_match(MatchStatus::Live, 0);
    assert!(m.involves_team("Team Liquid"));
    assert!(m.involves_team("team liquid")); // case insensitive
    assert!(m.involves_team("TL")); // tag
    assert!(m.involves_team("tl")); // tag case insensitive
    assert!(m.involves_team("OG")); // team_b
    assert!(!m.involves_team("Secret")); // not involved
}

#[test]
fn bracket_type_default_is_unknown() {
    let bracket = Bracket {
        bracket_type: BracketType::Unknown,
        upper_rounds: vec![],
        lower_rounds: None,
        grand_final: None,
    };
    assert_eq!(bracket.bracket_type, BracketType::Unknown);
    assert!(bracket.upper_rounds.is_empty());
    assert!(bracket.lower_rounds.is_none());
}

#[test]
fn bracket_match_tbd_teams() {
    let bm = BracketMatch {
        match_id: "m1".into(),
        round: 1,
        position: 0,
        team_a: Some("OG".into()),
        team_b: None,
        score_a: 0,
        score_b: 0,
        status: MatchStatus::Upcoming,
        winner_to: Some((2, 0)),
        loser_to: None,
    };
    assert!(bm.team_b.is_none());
    assert_eq!(bm.winner_to, Some((2, 0)));
}

#[test]
fn bracket_serialization_roundtrip() {
    let bracket = Bracket {
        bracket_type: BracketType::SingleElim,
        upper_rounds: vec![BracketRound {
            round: 1,
            name: "Quarterfinals".into(),
            matches: vec![BracketMatch {
                match_id: "m1".into(),
                round: 1,
                position: 0,
                team_a: Some("OG".into()),
                team_b: Some("Nigma".into()),
                score_a: 2,
                score_b: 0,
                status: MatchStatus::Completed,
                winner_to: Some((2, 0)),
                loser_to: None,
            }],
        }],
        lower_rounds: None,
        grand_final: None,
    };
    let json = serde_json::to_string(&bracket).unwrap();
    let restored: Bracket = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.bracket_type, BracketType::SingleElim);
    assert_eq!(restored.upper_rounds[0].matches[0].team_a, Some("OG".into()));
}
