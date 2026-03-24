use dota_2ui::api::liquipedia::LiquipediaProvider;
use dota_2ui::models::*;

#[test]
fn test_parse_tournament_response() {
    let json = r#"{"cargoquery": [{"title": {"Name": "ESL One Birmingham 2026", "DateStart": "2026-03-22", "Date": "2026-03-30", "Tier": "Tier 1", "Location": "Birmingham, United Kingdom", "Prizepool": "1000000"}}]}"#;
    let tournaments = LiquipediaProvider::parse_tournaments(json).unwrap();
    assert_eq!(tournaments.len(), 1);
    assert_eq!(tournaments[0].name, "ESL One Birmingham 2026");
    assert_eq!(tournaments[0].tier, "Tier 1");
    assert_eq!(tournaments[0].location.as_deref(), Some("Birmingham, United Kingdom"));
}

#[test]
fn test_parse_match_response() {
    let json = r#"{"cargoquery": [{"title": {"Team1": "Team Liquid", "Team2": "Gaimin Gladiators", "Team1Score": "1", "Team2Score": "0", "DateTime UTC": "2026-03-24 12:00:00", "BestOf": "2", "Tournament": "ESL One Birmingham 2026"}}]}"#;
    let matches = LiquipediaProvider::parse_matches(json).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].team_a.name, "Team Liquid");
    assert_eq!(matches[0].team_b.name, "Gaimin Gladiators");
    assert_eq!(matches[0].score_a, 1);
    assert_eq!(matches[0].score_b, 0);
}

#[test]
fn test_parse_empty_response() {
    let json = r#"{"cargoquery": []}"#;
    assert!(LiquipediaProvider::parse_tournaments(json).unwrap().is_empty());
    assert!(LiquipediaProvider::parse_matches(json).unwrap().is_empty());
}
