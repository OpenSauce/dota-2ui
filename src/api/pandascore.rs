use super::{ApiError, ApiResult, FetchAllResult, MatchProvider};
use crate::models::*;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

const BASE_URL: &str = "https://api.pandascore.co/dota2";

pub struct PandaScoreProvider {
    client: Client,
    api_key: String,
}

#[derive(Deserialize)]
struct PsMatch {
    id: u64,
    status: String,
    number_of_games: u8,
    scheduled_at: Option<String>,
    opponents: Vec<PsOpponent>,
    results: Vec<PsResult>,
    league: Option<PsLeague>,
    tournament_id: Option<u64>,
    streams_list: Option<Vec<PsStream>>,
    #[serde(default)]
    round: Option<i32>,
    #[serde(default)]
    position: Option<u32>,
    #[serde(default)]
    previous_matches: Option<Vec<PsPreviousMatch>>,
}

#[derive(Deserialize)]
struct PsPreviousMatch {
    match_id: u64,
    #[serde(rename = "type")]
    link_type: String,
}

#[derive(Deserialize)]
struct PsOpponent {
    opponent: PsTeam,
}

#[derive(Deserialize)]
struct PsTeam {
    name: String,
    acronym: Option<String>,
    location: Option<String>,
}

#[derive(Deserialize)]
struct PsResult {
    score: u8,
}

#[derive(Deserialize)]
struct PsLeague {
    name: String,
}

#[derive(Deserialize)]
struct PsStream {
    raw_url: Option<String>,
}

#[derive(Deserialize)]
struct PsTournament {
    id: u64,
    name: String,
    begin_at: Option<String>,
    end_at: Option<String>,
    tier: Option<String>,
    prizepool: Option<String>,
    league: Option<PsLeague>,
    serie: Option<PsSerie>,
}

#[derive(Deserialize)]
struct PsSerie {
    full_name: Option<String>,
}

impl PandaScoreProvider {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("failed to build HTTP client");
        Self { client, api_key }
    }

    pub fn parse_matches(json: &str) -> Result<Vec<Match>, ApiError> {
        let ps_matches: Vec<PsMatch> =
            serde_json::from_str(json).map_err(|e| ApiError::Parse(e.to_string()))?;
        let mut matches = Vec::new();
        for m in ps_matches {
            if m.opponents.len() < 2 {
                continue;
            }
            let team_a = &m.opponents[0].opponent;
            let team_b = &m.opponents[1].opponent;
            let score_a = m.results.first().map(|r| r.score).unwrap_or(0);
            let score_b = m.results.get(1).map(|r| r.score).unwrap_or(0);

            let status = match m.status.as_str() {
                "running" => MatchStatus::Live,
                "finished" => MatchStatus::Completed,
                _ => MatchStatus::Upcoming,
            };

            let series_format = match m.number_of_games {
                2 => SeriesFormat::Bo2,
                3 => SeriesFormat::Bo3,
                5 => SeriesFormat::Bo5,
                _ => SeriesFormat::Bo1,
            };

            let start_time = m
                .scheduled_at
                .as_deref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            let stream_url = m
                .streams_list
                .as_ref()
                .and_then(|s| s.first())
                .and_then(|s| s.raw_url.clone());

            let league_name = m
                .league
                .as_ref()
                .map(|l| l.name.clone())
                .unwrap_or_default();

            matches.push(Match {
                id: m.id.to_string(),
                team_a: Team {
                    name: team_a.name.clone(),
                    tag: team_a
                        .acronym
                        .clone()
                        .unwrap_or_else(|| team_a.name.clone()),
                    region: team_a.location.clone(),
                },
                team_b: Team {
                    name: team_b.name.clone(),
                    tag: team_b
                        .acronym
                        .clone()
                        .unwrap_or_else(|| team_b.name.clone()),
                    region: team_b.location.clone(),
                },
                score_a,
                score_b,
                status,
                series_format,
                tournament_name: league_name.clone(),
                tournament_id: m
                    .tournament_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| league_name.to_lowercase().replace(' ', "-")),
                start_time,
                stream_url,
                game_time_secs: None,
                stage: None,
            });
        }
        Ok(matches)
    }

    pub fn parse_tournaments(json: &str) -> Result<Vec<Tournament>, ApiError> {
        let ps_tournaments: Vec<PsTournament> =
            serde_json::from_str(json).map_err(|e| ApiError::Parse(e.to_string()))?;
        let now = Utc::now();
        let mut tournaments = Vec::new();
        for t in ps_tournaments {
            let start_date = t
                .begin_at
                .as_deref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(now);
            let end_date = t
                .end_at
                .as_deref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(now);

            let status = if now < start_date {
                TournamentStatus::Upcoming
            } else if now > end_date {
                TournamentStatus::Completed
            } else {
                TournamentStatus::Live
            };

            let tier = match t.tier.as_deref() {
                Some("s") => "S-Tier",
                Some("a") => "A-Tier",
                Some("b") => "B-Tier",
                Some("c") => "C-Tier",
                Some(other) => other,
                None => "Unknown",
            }
            .to_string();

            // Build display name: "League Serie" (e.g. "ESL One 2026") or fall back to tournament name
            let display_name = match (&t.league, &t.serie) {
                (Some(league), Some(serie)) if serie.full_name.is_some() => {
                    format!("{} {}", league.name, serie.full_name.as_ref().unwrap())
                }
                (Some(league), _) => league.name.clone(),
                _ => t.name.clone(),
            };

            tournaments.push(Tournament {
                id: t.id.to_string(),
                name: display_name,
                start_date,
                end_date,
                status,
                tier,
                location: None,
                prize_pool: t.prizepool,
            });
        }

        // Deduplicate tournaments with the same name (PandaScore returns stages as separate entries).
        // Merge by taking the earliest start, latest end, most "active" status, and highest tier.
        let mut merged: std::collections::HashMap<String, Tournament> =
            std::collections::HashMap::new();
        for t in tournaments {
            if let Some(existing) = merged.get_mut(&t.name) {
                if t.start_date < existing.start_date {
                    existing.start_date = t.start_date;
                }
                if t.end_date > existing.end_date {
                    existing.end_date = t.end_date;
                }
                // Prefer Live > Upcoming > Completed
                if t.status == TournamentStatus::Live {
                    existing.status = TournamentStatus::Live;
                } else if t.status == TournamentStatus::Upcoming
                    && existing.status == TournamentStatus::Completed
                {
                    existing.status = TournamentStatus::Upcoming;
                }
                // Keep higher tier
                if t.tier.starts_with("S") && !existing.tier.starts_with("S") {
                    existing.tier = t.tier;
                }
                // Merge prize pool if missing
                if existing.prize_pool.is_none() {
                    existing.prize_pool = t.prize_pool;
                }
            } else {
                merged.insert(t.name.clone(), t);
            }
        }

        Ok(merged.into_values().collect())
    }

    async fn get(&self, endpoint: &str) -> ApiResult<String> {
        let url = format!("{}{}", BASE_URL, endpoint);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        if resp.status() == 429 {
            return Err(ApiError::RateLimit);
        }
        Ok(resp.text().await?)
    }

    /// Reconstruct a bracket from PandaScore match JSON data.
    pub fn reconstruct_bracket(json: &str) -> Result<Bracket, ApiError> {
        let ps_matches: Vec<PsMatch> =
            serde_json::from_str(json).map_err(|e| ApiError::Parse(e.to_string()))?;

        // Detect bracket type: if any match has a "loser" previous_match link → DoubleElim
        let has_loser_link = ps_matches.iter().any(|m| {
            m.previous_matches
                .as_ref()
                .map(|prev| prev.iter().any(|pm| pm.link_type == "loser"))
                .unwrap_or(false)
        });
        let bracket_type = if has_loser_link {
            BracketType::DoubleElim
        } else {
            BracketType::SingleElim
        };

        // Group by round number
        let mut round_map: BTreeMap<i32, Vec<&PsMatch>> = BTreeMap::new();
        for m in &ps_matches {
            let round = m.round.unwrap_or(1);
            round_map.entry(round).or_default().push(m);
        }

        // Separate upper (positive/zero rounds) and lower (negative rounds)
        let upper_round_nums: Vec<i32> = round_map.keys().filter(|&&r| r >= 0).copied().collect();
        let lower_round_nums: Vec<i32> = round_map.keys().filter(|&&r| r < 0).copied().collect();

        let max_upper = upper_round_nums.iter().copied().max().unwrap_or(0);

        let upper_rounds: Vec<BracketRound> = upper_round_nums
            .iter()
            .map(|&round_num| {
                let mut matches_in_round: Vec<&PsMatch> =
                    round_map.get(&round_num).cloned().unwrap_or_default();
                matches_in_round.sort_by_key(|m| m.position.unwrap_or(0));

                let bracket_matches: Vec<BracketMatch> = matches_in_round
                    .iter()
                    .map(|m| {
                        let team_a = m.opponents.first().map(|o| o.opponent.name.clone());
                        let team_b = m.opponents.get(1).map(|o| o.opponent.name.clone());
                        let score_a = m.results.first().map(|r| r.score).unwrap_or(0);
                        let score_b = m.results.get(1).map(|r| r.score).unwrap_or(0);
                        let status = match m.status.as_str() {
                            "running" => MatchStatus::Live,
                            "finished" => MatchStatus::Completed,
                            _ => MatchStatus::Upcoming,
                        };
                        let winner_to = find_next_match(&ps_matches, m.id, "winner");
                        let loser_to = find_next_match(&ps_matches, m.id, "loser");

                        BracketMatch {
                            match_id: m.id.to_string(),
                            round: round_num as usize,
                            position: m.position.unwrap_or(0) as usize,
                            team_a,
                            team_b,
                            score_a,
                            score_b,
                            status,
                            winner_to,
                            loser_to,
                        }
                    })
                    .collect();

                BracketRound {
                    round: round_num as usize,
                    name: name_round(round_num, max_upper),
                    matches: bracket_matches,
                }
            })
            .collect();

        let lower_rounds: Option<Vec<BracketRound>> = if lower_round_nums.is_empty() {
            None
        } else {
            let max_lower = lower_round_nums.iter().copied().max().unwrap_or(0);
            Some(
                lower_round_nums
                    .iter()
                    .map(|&round_num| {
                        let mut matches_in_round: Vec<&PsMatch> =
                            round_map.get(&round_num).cloned().unwrap_or_default();
                        matches_in_round.sort_by_key(|m| m.position.unwrap_or(0));

                        let bracket_matches: Vec<BracketMatch> = matches_in_round
                            .iter()
                            .map(|m| {
                                let team_a = m.opponents.first().map(|o| o.opponent.name.clone());
                                let team_b = m.opponents.get(1).map(|o| o.opponent.name.clone());
                                let score_a = m.results.first().map(|r| r.score).unwrap_or(0);
                                let score_b = m.results.get(1).map(|r| r.score).unwrap_or(0);
                                let status = match m.status.as_str() {
                                    "running" => MatchStatus::Live,
                                    "finished" => MatchStatus::Completed,
                                    _ => MatchStatus::Upcoming,
                                };
                                let winner_to = find_next_match(&ps_matches, m.id, "winner");
                                let loser_to = find_next_match(&ps_matches, m.id, "loser");

                                BracketMatch {
                                    match_id: m.id.to_string(),
                                    round: round_num.unsigned_abs() as usize,
                                    position: m.position.unwrap_or(0) as usize,
                                    team_a,
                                    team_b,
                                    score_a,
                                    score_b,
                                    status,
                                    winner_to,
                                    loser_to,
                                }
                            })
                            .collect();

                        BracketRound {
                            round: round_num.unsigned_abs() as usize,
                            name: name_round(round_num.abs(), max_lower.abs()),
                            matches: bracket_matches,
                        }
                    })
                    .collect(),
            )
        };

        Ok(Bracket {
            bracket_type,
            upper_rounds,
            lower_rounds,
            grand_final: None,
        })
    }
}

fn name_round(round: i32, max_round: i32) -> String {
    let from_end = max_round - round;
    match from_end {
        0 => "Final".into(),
        1 => "Semifinals".into(),
        2 => "Quarterfinals".into(),
        _ => format!("Round {}", round),
    }
}

fn find_next_match(
    all_matches: &[PsMatch],
    source_id: u64,
    link_type: &str,
) -> Option<(usize, usize)> {
    for m in all_matches {
        if let Some(prev) = &m.previous_matches {
            for pm in prev {
                if pm.match_id == source_id && pm.link_type == link_type {
                    let round = m.round.unwrap_or(1).unsigned_abs() as usize;
                    let pos = m.position.unwrap_or(0) as usize;
                    return Some((round, pos));
                }
            }
        }
    }
    None
}

impl MatchProvider for PandaScoreProvider {
    fn fetch_bracket(
        &self,
        tournament_id: &str,
    ) -> Pin<Box<dyn Future<Output = ApiResult<Option<crate::models::Bracket>>> + Send + '_>> {
        let tid = tournament_id.to_string();
        Box::pin(async move {
            let json = self
                .get(&format!("/tournaments/{}/matches?per_page=100", tid))
                .await?;
            let bracket = Self::reconstruct_bracket(&json)?;
            if bracket.upper_rounds.is_empty() {
                Ok(None)
            } else {
                Ok(Some(bracket))
            }
        })
    }

    fn fetch_all(&self) -> Pin<Box<dyn Future<Output = ApiResult<FetchAllResult>> + Send + '_>> {
        Box::pin(async move {
            let running_m = self
                .get("/matches/running")
                .await
                .unwrap_or_else(|_| "[]".to_string());
            let upcoming_m = self
                .get("/matches/upcoming?per_page=20")
                .await
                .unwrap_or_else(|_| "[]".to_string());
            let mut matches = Self::parse_matches(&running_m)?;
            matches.extend(Self::parse_matches(&upcoming_m)?);

            let running_t = self
                .get("/tournaments/running")
                .await
                .unwrap_or_else(|_| "[]".to_string());
            let upcoming_t = self
                .get("/tournaments/upcoming?per_page=20")
                .await
                .unwrap_or_else(|_| "[]".to_string());
            let mut tournaments = Self::parse_tournaments(&running_t)?;
            tournaments.extend(Self::parse_tournaments(&upcoming_t)?);

            Ok(FetchAllResult {
                matches,
                tournaments,
            })
        })
    }
}
