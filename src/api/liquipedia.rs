use super::{ApiError, ApiResult, FetchAllResult, MatchProvider};
use crate::models::*;
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::pin::Pin;

const BASE_URL: &str = "https://liquipedia.net/dota2/api.php";
const USER_AGENT: &str = "dota-2ui/0.1 (https://github.com/OpenSauce/dota-2ui)";

pub struct LiquipediaProvider {
    client: Client,
}

#[derive(Deserialize)]
struct ParseResponse {
    parse: ParseData,
}

#[derive(Deserialize)]
struct ParseData {
    text: ParseText,
}

#[derive(Deserialize)]
struct ParseText {
    #[serde(rename = "*")]
    content: String,
}

impl Default for LiquipediaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LiquipediaProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .gzip(true)
            .build()
            .expect("failed to build HTTP client");
        Self { client }
    }

    /// Parse matches from the Liquipedia:Matches page HTML (returned via MediaWiki parse API).
    /// Each match is in a `<div class="match-info">` block containing:
    /// - Team names in `<span class="name"><a>TeamName</a></span>`
    /// - Scores in `<span class="match-info-header-scoreholder-score">N</span>`
    /// - Format in `<span class="match-info-header-scoreholder-lower">(BoN)</span>`
    /// - Timestamp in `<span class="timer-object" data-timestamp="UNIX">`
    /// - Tournament in `<span class="match-info-tournament-name"><a title="...">Name</a></span>`
    /// - Completed matches have `match-info-header-winner` / `match-info-header-loser` classes
    pub fn parse_matches_html(html: &str) -> Result<Vec<Match>, ApiError> {
        let re_name = Regex::new(r#"<span class="name"[^>]*><a[^>]*>([^<]+)</a></span>"#).unwrap();
        let re_score = Regex::new(r#"match-info-header-scoreholder-score">(\d+)</span>"#).unwrap();
        let re_format = Regex::new(r#"scoreholder-lower">\(([^)]+)\)</span>"#).unwrap();
        let re_timestamp = Regex::new(r#"data-timestamp="(\d+)""#).unwrap();
        let re_tournament =
            Regex::new(r#"match-info-tournament-name"><a[^>]*title="([^"]+)""#).unwrap();

        let blocks: Vec<&str> = html.split("<div class=\"match-info\">").skip(1).collect();
        let now = Utc::now();
        let mut matches = Vec::new();

        for block in blocks {
            // Only take up to the next match-info div
            let block = block
                .split("<div class=\"match-info\">")
                .next()
                .unwrap_or(block);

            let names: Vec<String> = re_name
                .captures_iter(block)
                .map(|c| c[1].to_string())
                .collect();
            if names.len() < 2 {
                continue;
            }

            let scores: Vec<u8> = re_score
                .captures_iter(block)
                .map(|c| c[1].parse().unwrap_or(0))
                .collect();
            let score_a = scores.first().copied().unwrap_or(0);
            let score_b = scores.get(1).copied().unwrap_or(0);

            let series_format = re_format
                .captures(block)
                .and_then(|c| match c[1].to_string().as_str() {
                    "Bo1" => Some(SeriesFormat::Bo1),
                    "Bo2" => Some(SeriesFormat::Bo2),
                    "Bo3" => Some(SeriesFormat::Bo3),
                    "Bo5" => Some(SeriesFormat::Bo5),
                    _ => None,
                })
                .unwrap_or(SeriesFormat::Bo1);

            let start_time = re_timestamp
                .captures(block)
                .and_then(|c| c[1].parse::<i64>().ok())
                .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
                .unwrap_or(now);

            let tournament_raw = re_tournament
                .captures(block)
                .map(|c| c[1].to_string())
                .unwrap_or_default();
            // Clean up tournament name and extract stage
            let (tournament_name, stage) = clean_tournament_name(&tournament_raw);

            let has_winner = block.contains("match-info-header-winner");
            let _has_scores = scores.len() >= 2 && (score_a > 0 || score_b > 0);

            let status = if has_winner {
                MatchStatus::Completed
            } else if start_time > now {
                MatchStatus::Upcoming
            } else {
                // start_time <= now: live regardless of scores
                MatchStatus::Live
            };

            let id = format!("{}-vs-{}-{}", names[0], names[1], start_time.timestamp())
                .to_lowercase()
                .replace(' ', "-");

            matches.push(Match {
                id,
                team_a: Team {
                    name: names[0].clone(),
                    tag: names[0].clone(),
                    region: None,
                },
                team_b: Team {
                    name: names[1].clone(),
                    tag: names[1].clone(),
                    region: None,
                },
                score_a,
                score_b,
                status,
                series_format,
                tournament_name: tournament_name.clone(),
                tournament_id: tournament_raw.to_lowercase().replace(' ', "-"),
                start_time,
                stream_url: None,
                game_time_secs: None,
                stage: stage.clone(),
            });
        }

        Ok(matches)
    }

    /// Derive tournaments from match data — group by tournament name and infer dates.
    pub fn derive_tournaments(matches: &[Match]) -> Vec<Tournament> {
        let mut tournament_map: HashMap<String, (DateTime<Utc>, DateTime<Utc>)> = HashMap::new();

        for m in matches {
            if m.tournament_name.is_empty() {
                continue;
            }
            let entry = tournament_map
                .entry(m.tournament_name.clone())
                .or_insert((m.start_time, m.start_time));
            if m.start_time < entry.0 {
                entry.0 = m.start_time;
            }
            if m.start_time > entry.1 {
                entry.1 = m.start_time;
            }
        }

        let now = Utc::now();
        tournament_map
            .into_iter()
            .map(|(name, (earliest, latest))| {
                let status = if latest < now
                    && !matches
                        .iter()
                        .any(|m| m.tournament_name == name && m.status == MatchStatus::Live)
                {
                    TournamentStatus::Completed
                } else if earliest > now {
                    TournamentStatus::Upcoming
                } else {
                    TournamentStatus::Live
                };

                let id = name.to_lowercase().replace(' ', "-");
                Tournament {
                    id,
                    name,
                    start_date: earliest,
                    end_date: latest,
                    status,
                    tier: String::new(),
                    location: None,
                    prize_pool: None,
                }
            })
            .collect()
    }

    /// Build a bracket by grouping matches by their stage field.
    /// This is a fallback for when no proper bracket data is available.
    pub fn build_stage_bracket(matches: &[Match]) -> Option<Bracket> {
        let mut stage_map: BTreeMap<String, Vec<&Match>> = BTreeMap::new();
        for m in matches {
            let stage = m.stage.clone().unwrap_or_else(|| "Matches".into());
            stage_map.entry(stage).or_default().push(m);
        }
        if stage_map.is_empty() {
            return None;
        }
        let upper_rounds: Vec<BracketRound> = stage_map
            .into_iter()
            .enumerate()
            .map(|(i, (stage_name, stage_matches))| {
                let bracket_matches: Vec<BracketMatch> = stage_matches
                    .iter()
                    .enumerate()
                    .map(|(pos, m)| BracketMatch {
                        match_id: m.id.clone(),
                        round: i + 1,
                        position: pos,
                        team_a: Some(m.team_a.name.clone()),
                        team_b: Some(m.team_b.name.clone()),
                        score_a: m.score_a,
                        score_b: m.score_b,
                        status: m.status,
                        winner_to: None,
                        loser_to: None,
                    })
                    .collect();
                BracketRound {
                    round: i + 1,
                    name: stage_name,
                    matches: bracket_matches,
                }
            })
            .collect();
        Some(Bracket {
            bracket_type: BracketType::Unknown,
            upper_rounds,
            lower_rounds: None,
            grand_final: None,
        })
    }

    async fn fetch_parse_page(&self, page: &str) -> ApiResult<String> {
        let url = format!("{}?action=parse&page={}&format=json", BASE_URL, page);
        let resp = self.client.get(&url).send().await?;
        if resp.status() == 429 {
            return Err(ApiError::RateLimit);
        }
        let text = resp.text().await?;
        let parsed: ParseResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        Ok(parsed.parse.text.content)
    }
}

/// Clean tournament name from Liquipedia wiki path format.
/// "ESL One/Birmingham/2026/Group Stage" -> ("ESL One Birmingham 2026", Some("Group Stage"))
/// Extracts stage suffixes like "Group Stage", "Playoffs", etc.
fn clean_tournament_name(raw: &str) -> (String, Option<String>) {
    let stage_suffixes = [
        "Group Stage",
        "Playoffs",
        "Main Event",
        "Qualifier",
        "Open Qualifier",
        "Closed Qualifier",
    ];
    let parts: Vec<&str> = raw.split('/').collect();
    let mut stage: Option<String> = None;
    let mut cleaned: Vec<&str> = Vec::new();
    for part in &parts {
        if let Some(suffix) = stage_suffixes.iter().find(|s| part.contains(*s)) {
            stage = Some((*suffix).to_string());
        } else {
            cleaned.push(part);
        }
    }
    // Also strip anything after '#'
    let name = cleaned.join(" ");
    let name = name.split('#').next().unwrap_or(&name).trim().to_string();
    (name, stage)
}

impl MatchProvider for LiquipediaProvider {
    fn fetch_all(&self) -> Pin<Box<dyn Future<Output = ApiResult<FetchAllResult>> + Send + '_>> {
        Box::pin(async move {
            let html = self.fetch_parse_page("Liquipedia:Matches").await?;
            let matches = Self::parse_matches_html(&html)?;
            let tournaments = Self::derive_tournaments(&matches);
            Ok(FetchAllResult {
                matches,
                tournaments,
            })
        })
    }
}
