use crate::api::{ApiError, ApiResult, MatchProvider};
use crate::models::*;
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;

const BASE_URL: &str = "https://liquipedia.net/dota2/api.php";
const USER_AGENT: &str = "dota-2ui/0.1 (https://github.com/OpenSauce/dota-2ui)";

pub struct LiquipediaProvider {
    client: Client,
}

#[derive(Deserialize)]
struct CargoResponse {
    cargoquery: Vec<CargoRow>,
}

#[derive(Deserialize)]
struct CargoRow {
    title: serde_json::Value,
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

    pub fn parse_tournaments(json: &str) -> Result<Vec<Tournament>, ApiError> {
        let resp: CargoResponse = serde_json::from_str(json).map_err(|e| ApiError::Parse(e.to_string()))?;
        let mut tournaments = Vec::new();
        for row in resp.cargoquery {
            let t = &row.title;
            let name = t["Name"].as_str().unwrap_or_default().to_string();
            if name.is_empty() { continue; }

            let start_str = t["DateStart"].as_str().unwrap_or_default();
            let end_str = t["Date"].as_str().unwrap_or_default();

            let start_date = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
                .map(|d| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()))
                .unwrap_or_else(|_| Utc::now());

            let end_date = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
                .map(|d| Utc.from_utc_datetime(&d.and_hms_opt(23, 59, 59).unwrap()))
                .unwrap_or_else(|_| Utc::now());

            let now = Utc::now();
            let status = if now < start_date { TournamentStatus::Upcoming }
                else if now > end_date { TournamentStatus::Completed }
                else { TournamentStatus::Live };

            let id = name.to_lowercase().replace(' ', "-");
            tournaments.push(Tournament {
                id, name, start_date, end_date, status,
                tier: t["Tier"].as_str().unwrap_or("Unknown").to_string(),
                location: t["Location"].as_str().map(|s| s.to_string()).filter(|s| !s.is_empty()),
                prize_pool: t["Prizepool"].as_str().map(|s| s.to_string()).filter(|s| !s.is_empty()),
            });
        }
        Ok(tournaments)
    }

    pub fn parse_matches(json: &str) -> Result<Vec<Match>, ApiError> {
        let resp: CargoResponse = serde_json::from_str(json).map_err(|e| ApiError::Parse(e.to_string()))?;
        let mut matches = Vec::new();
        for row in resp.cargoquery {
            let t = &row.title;
            let team1 = t["Team1"].as_str().unwrap_or_default().to_string();
            let team2 = t["Team2"].as_str().unwrap_or_default().to_string();
            if team1.is_empty() || team2.is_empty() { continue; }

            let score_a: u8 = t["Team1Score"].as_str().unwrap_or("0").parse().unwrap_or(0);
            let score_b: u8 = t["Team2Score"].as_str().unwrap_or("0").parse().unwrap_or(0);

            let datetime_str = t["DateTime UTC"].as_str().unwrap_or_default();
            let start_time = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
                .map(|dt| Utc.from_utc_datetime(&dt))
                .unwrap_or_else(|_| Utc::now());

            let best_of: u8 = t["BestOf"].as_str().unwrap_or("1").parse().unwrap_or(1);
            let series_format = match best_of {
                2 => SeriesFormat::Bo2,
                3 => SeriesFormat::Bo3,
                5 => SeriesFormat::Bo5,
                _ => SeriesFormat::Bo1,
            };

            let now = Utc::now();
            let wins_needed = match best_of {
                2 => 2,
                3 => 2,
                5 => 3,
                _ => 1,
            };
            let status = if start_time > now {
                MatchStatus::Upcoming
            } else if (score_a >= wins_needed || score_b >= wins_needed)
                || (best_of == 2 && score_a + score_b >= 2) {
                MatchStatus::Completed
            } else {
                MatchStatus::Live
            };

            let tournament_name = t["Tournament"].as_str().unwrap_or_default().to_string();
            let id = format!("{}-vs-{}-{}", team1, team2, datetime_str).to_lowercase().replace(' ', "-");

            matches.push(Match {
                id,
                team_a: Team { name: team1.clone(), tag: team1, region: None },
                team_b: Team { name: team2.clone(), tag: team2, region: None },
                score_a, score_b, status, series_format,
                tournament_name: tournament_name.clone(),
                tournament_id: tournament_name.to_lowercase().replace(' ', "-"),
                start_time, stream_url: None, game_time_secs: None,
            });
        }
        Ok(matches)
    }

    async fn fetch_cargo(&self, tables: &str, fields: &str, conditions: &str, limit: u32) -> ApiResult<String> {
        let url = format!(
            "{}?action=cargoquery&tables={}&fields={}&where={}&limit={}&format=json",
            BASE_URL, tables, fields, conditions, limit
        );
        let resp = self.client.get(&url).send().await?;
        if resp.status() == 429 { return Err(ApiError::RateLimit); }
        Ok(resp.text().await?)
    }
}

impl MatchProvider for LiquipediaProvider {
    fn fetch_matches(&self) -> Pin<Box<dyn Future<Output = ApiResult<Vec<Match>>> + Send + '_>> {
        Box::pin(async move {
            let now = Utc::now().format("%Y-%m-%d").to_string();
            let conditions = format!("MatchSchedule.DateTime_UTC>'{}'", now);
            let json = self.fetch_cargo(
                "MatchSchedule",
                "MatchSchedule.Team1,MatchSchedule.Team2,MatchSchedule.Team1Score,MatchSchedule.Team2Score,MatchSchedule.DateTime_UTC,MatchSchedule.BestOf,MatchSchedule.Tournament",
                &conditions, 50,
            ).await?;
            Self::parse_matches(&json)
        })
    }

    fn fetch_tournaments(&self) -> Pin<Box<dyn Future<Output = ApiResult<Vec<Tournament>>> + Send + '_>> {
        Box::pin(async move {
            let past = (Utc::now() - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
            let conditions = format!("Tournaments.DateStart>'{}'", past);
            let json = self.fetch_cargo(
                "Tournaments",
                "Tournaments.Name,Tournaments.DateStart,Tournaments.Date,Tournaments.Tier,Tournaments.Location,Tournaments.Prizepool",
                &conditions, 30,
            ).await?;
            Self::parse_tournaments(&json)
        })
    }
}
