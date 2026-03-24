use super::{ApiError, ApiResult, FetchAllResult, MatchProvider};
use crate::models::*;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
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
                tournament_id: m.tournament_id
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

            tournaments.push(Tournament {
                id: t.id.to_string(),
                name: t.name,
                start_date,
                end_date,
                status,
                tier,
                location: None,
                prize_pool: t.prizepool,
            });
        }
        Ok(tournaments)
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
}

impl MatchProvider for PandaScoreProvider {
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
