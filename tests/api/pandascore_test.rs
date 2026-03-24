use dota_2ui::api::pandascore::PandaScoreProvider;
use dota_2ui::models::*;

#[test]
fn test_parse_pandascore_match() {
    let json = r#"[{"id": 12345, "name": "Team Liquid vs OG", "status": "running", "number_of_games": 3, "scheduled_at": "2026-03-24T12:00:00Z", "opponents": [{"opponent": {"name": "Team Liquid", "acronym": "Liquid", "location": "EU"}}, {"opponent": {"name": "OG", "acronym": "OG", "location": "EU"}}], "results": [{"team_id": 1, "score": 1}, {"team_id": 2, "score": 0}], "league": {"name": "ESL One Birmingham 2026"}, "streams_list": [{"raw_url": "https://twitch.tv/esl"}]}]"#;
    let matches = PandaScoreProvider::parse_matches(json).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].team_a.name, "Team Liquid");
    assert_eq!(matches[0].team_a.tag, "Liquid");
    assert_eq!(matches[0].score_a, 1);
    assert_eq!(matches[0].status, MatchStatus::Live);
    assert_eq!(matches[0].series_format, SeriesFormat::Bo3);
    assert_eq!(matches[0].stream_url.as_deref(), Some("https://twitch.tv/esl"));
}

#[test]
fn test_parse_pandascore_tournament() {
    let json = r#"[{"id": 999, "name": "ESL One Birmingham 2026", "begin_at": "2026-03-22T00:00:00Z", "end_at": "2026-03-30T23:59:59Z", "tier": "s", "prizepool": "$1,000,000"}]"#;
    let tournaments = PandaScoreProvider::parse_tournaments(json).unwrap();
    assert_eq!(tournaments.len(), 1);
    assert_eq!(tournaments[0].name, "ESL One Birmingham 2026");
    assert_eq!(tournaments[0].tier, "S-Tier");
}

#[test]
fn test_parse_pandascore_empty() {
    assert!(PandaScoreProvider::parse_matches("[]").unwrap().is_empty());
    assert!(PandaScoreProvider::parse_tournaments("[]").unwrap().is_empty());
}
