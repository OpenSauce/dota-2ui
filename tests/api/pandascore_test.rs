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
    assert_eq!(
        matches[0].stream_url.as_deref(),
        Some("https://twitch.tv/esl")
    );
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
fn match_uses_real_tournament_id() {
    let json = r#"[{
        "id": 100,
        "status": "running",
        "number_of_games": 3,
        "scheduled_at": "2026-03-24T12:00:00Z",
        "opponents": [
            {"opponent": {"name": "OG", "acronym": "OG", "location": "EU"}},
            {"opponent": {"name": "Nigma", "acronym": "NGX", "location": "EU"}}
        ],
        "results": [{"score": 1}, {"score": 0}],
        "league": {"name": "ESL One"},
        "tournament_id": 9876,
        "tournament": {"id": 9876, "name": "ESL One Birmingham 2026"},
        "streams_list": []
    }]"#;
    let matches = dota_2ui::api::pandascore::PandaScoreProvider::parse_matches(json).unwrap();
    assert_eq!(matches[0].tournament_id, "9876");
}

#[test]
fn reconstruct_bracket_from_matches() {
    let json = r#"[
        {
            "id": 1, "status": "finished", "number_of_games": 3,
            "scheduled_at": "2026-03-24T10:00:00Z",
            "opponents": [
                {"opponent": {"name": "OG", "acronym": "OG", "location": null}},
                {"opponent": {"name": "Nigma", "acronym": "NGX", "location": null}}
            ],
            "results": [{"score": 2}, {"score": 0}],
            "league": {"name": "Test Cup"},
            "tournament_id": 100,
            "streams_list": [],
            "round": 1, "position": 1, "previous_matches": []
        },
        {
            "id": 2, "status": "finished", "number_of_games": 3,
            "scheduled_at": "2026-03-24T13:00:00Z",
            "opponents": [
                {"opponent": {"name": "Team Liquid", "acronym": "TL", "location": null}},
                {"opponent": {"name": "Team Spirit", "acronym": "TS", "location": null}}
            ],
            "results": [{"score": 2}, {"score": 1}],
            "league": {"name": "Test Cup"},
            "tournament_id": 100,
            "streams_list": [],
            "round": 1, "position": 2, "previous_matches": []
        },
        {
            "id": 3, "status": "running", "number_of_games": 5,
            "scheduled_at": "2026-03-24T16:00:00Z",
            "opponents": [
                {"opponent": {"name": "OG", "acronym": "OG", "location": null}},
                {"opponent": {"name": "Team Liquid", "acronym": "TL", "location": null}}
            ],
            "results": [{"score": 1}, {"score": 1}],
            "league": {"name": "Test Cup"},
            "tournament_id": 100,
            "streams_list": [],
            "round": 2, "position": 1,
            "previous_matches": [{"match_id": 1, "type": "winner"}, {"match_id": 2, "type": "winner"}]
        }
    ]"#;

    let bracket = dota_2ui::api::pandascore::PandaScoreProvider::reconstruct_bracket(json).unwrap();
    assert_eq!(
        bracket.bracket_type,
        dota_2ui::models::BracketType::SingleElim
    );
    assert_eq!(bracket.upper_rounds.len(), 2);
    assert_eq!(bracket.upper_rounds[0].matches.len(), 2);
    assert_eq!(bracket.upper_rounds[1].matches.len(), 1);
    assert_eq!(bracket.upper_rounds[0].matches[0].team_a, Some("OG".into()));
    assert_eq!(bracket.upper_rounds[1].matches[0].winner_to, None);
}

#[test]
fn test_parse_pandascore_empty() {
    assert!(PandaScoreProvider::parse_matches("[]").unwrap().is_empty());
    assert!(PandaScoreProvider::parse_tournaments("[]")
        .unwrap()
        .is_empty());
}
