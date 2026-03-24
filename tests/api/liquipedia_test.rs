use chrono::Utc;
use dota_2ui::api::liquipedia::LiquipediaProvider;
use dota_2ui::models::*;

#[test]
fn test_parse_matches_html() {
    let html = r#"
    <div class="match-info">
        <span class="timer-object" data-timestamp="1774373100">March 24, 2026</span>
        <div class="match-info-header">
            <div class="match-info-header-opponent match-info-header-opponent-left">
                <div class="block-team flipped">
                    <span class="name"><a href="/dota2/Aurora" title="Aurora">Aurora</a></span>
                </div>
            </div>
            <div class="match-info-header-scoreholder">
                <span class="match-info-header-scoreholder-score">1</span>
                <span class="match-info-header-scoreholder-divider">:</span>
                <span class="match-info-header-scoreholder-score">0</span>
                <span class="match-info-header-scoreholder-lower">(Bo2)</span>
            </div>
            <div class="match-info-header-opponent">
                <div class="block-team">
                    <span class="name"><a href="/dota2/OG" title="OG">OG</a></span>
                </div>
            </div>
        </div>
        <div class="match-info-tournament">
            <span class="match-info-tournament-name"><a href="/dota2/ESL" title="ESL One/Birmingham/2026/Group Stage">ESL One Birmingham 2026</a></span>
        </div>
    </div>
    "#;

    let matches = LiquipediaProvider::parse_matches_html(html).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].team_a.name, "Aurora");
    assert_eq!(matches[0].team_b.name, "OG");
    assert_eq!(matches[0].score_a, 1);
    assert_eq!(matches[0].score_b, 0);
    assert_eq!(matches[0].series_format, SeriesFormat::Bo2);
    assert_eq!(matches[0].tournament_name, "ESL One Birmingham 2026");
}

#[test]
fn test_parse_completed_match() {
    let html = r#"
    <div class="match-info">
        <span class="timer-object" data-timestamp="1774300000">March 24, 2026</span>
        <div class="match-info-header">
            <div class="match-info-header-opponent match-info-header-opponent-left match-info-header-winner">
                <div class="block-team flipped">
                    <span class="name"><a title="Tundra">Tundra</a></span>
                </div>
            </div>
            <div class="match-info-header-scoreholder">
                <span class="match-info-header-scoreholder-score">2</span>
                <span class="match-info-header-scoreholder-divider">:</span>
                <span class="match-info-header-scoreholder-score">1</span>
                <span class="match-info-header-scoreholder-lower">(Bo3)</span>
            </div>
            <div class="match-info-header-opponent match-info-header-loser">
                <div class="block-team">
                    <span class="name"><a title="Entity">Entity</a></span>
                </div>
            </div>
        </div>
        <div class="match-info-tournament">
            <span class="match-info-tournament-name"><a title="Stampede Champions S2">Stampede</a></span>
        </div>
    </div>
    "#;

    let matches = LiquipediaProvider::parse_matches_html(html).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].status, MatchStatus::Completed);
    assert_eq!(matches[0].score_a, 2);
    assert_eq!(matches[0].score_b, 1);
}

#[test]
fn test_parse_empty_html() {
    let html = "<div>no matches here</div>";
    let matches = LiquipediaProvider::parse_matches_html(html).unwrap();
    assert!(matches.is_empty());
}

#[test]
fn test_derive_tournaments() {
    let html = r#"
    <div class="match-info">
        <span class="timer-object" data-timestamp="1774373100"></span>
        <div class="match-info-header">
            <div class="match-info-header-opponent">
                <span class="name"><a>Team A</a></span>
            </div>
            <div class="match-info-header-scoreholder">
                <span class="match-info-header-scoreholder-score">0</span>
                <span class="match-info-header-scoreholder-score">0</span>
                <span class="match-info-header-scoreholder-lower">(Bo3)</span>
            </div>
            <div class="match-info-header-opponent">
                <span class="name"><a>Team B</a></span>
            </div>
        </div>
        <div class="match-info-tournament">
            <span class="match-info-tournament-name"><a title="ESL One/Birmingham/2026/Group Stage">ESL</a></span>
        </div>
    </div>
    "#;

    let matches = LiquipediaProvider::parse_matches_html(html).unwrap();
    let tournaments = LiquipediaProvider::derive_tournaments(&matches);
    assert_eq!(tournaments.len(), 1);
    assert!(tournaments[0].name.contains("ESL One Birmingham 2026"));
}

#[test]
fn build_stage_bracket_groups_by_stage() {
    let matches = vec![
        Match {
            id: "m1".into(),
            team_a: Team { name: "OG".into(), tag: "OG".into(), region: None },
            team_b: Team { name: "Nigma".into(), tag: "NGX".into(), region: None },
            score_a: 2, score_b: 0, status: MatchStatus::Completed,
            series_format: SeriesFormat::Bo3,
            tournament_name: "Test Cup".into(), tournament_id: "t1".into(),
            start_time: Utc::now(), stream_url: None, game_time_secs: None,
            stage: Some("Group Stage".into()),
        },
        Match {
            id: "m2".into(),
            team_a: Team { name: "Liquid".into(), tag: "TL".into(), region: None },
            team_b: Team { name: "Spirit".into(), tag: "TS".into(), region: None },
            score_a: 1, score_b: 2, status: MatchStatus::Completed,
            series_format: SeriesFormat::Bo3,
            tournament_name: "Test Cup".into(), tournament_id: "t1".into(),
            start_time: Utc::now(), stream_url: None, game_time_secs: None,
            stage: Some("Playoffs".into()),
        },
    ];

    let bracket = LiquipediaProvider::build_stage_bracket(&matches).unwrap();
    assert_eq!(bracket.bracket_type, BracketType::Unknown);
    assert_eq!(bracket.upper_rounds.len(), 2);
    assert_eq!(bracket.upper_rounds[0].name, "Group Stage");
    assert_eq!(bracket.upper_rounds[1].name, "Playoffs");
}
