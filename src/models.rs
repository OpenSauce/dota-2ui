use chrono::{DateTime, Utc};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Team {
    pub name: String,
    pub tag: String,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MatchStatus { Upcoming, Live, Completed }

impl MatchStatus {
    pub fn is_live(&self) -> bool { matches!(self, MatchStatus::Live) }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SeriesFormat { Bo1, Bo2, Bo3, Bo5 }

impl fmt::Display for SeriesFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeriesFormat::Bo1 => write!(f, "Bo1"),
            SeriesFormat::Bo2 => write!(f, "Bo2"),
            SeriesFormat::Bo3 => write!(f, "Bo3"),
            SeriesFormat::Bo5 => write!(f, "Bo5"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub team_a: Team,
    pub team_b: Team,
    pub score_a: u8,
    pub score_b: u8,
    pub status: MatchStatus,
    pub series_format: SeriesFormat,
    pub tournament_name: String,
    pub tournament_id: String,
    pub start_time: DateTime<Utc>,
    pub stream_url: Option<String>,
    pub game_time_secs: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TournamentStatus { Upcoming, Live, Completed }

impl TournamentStatus {
    pub fn is_live(&self) -> bool { matches!(self, TournamentStatus::Live) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub id: String,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: TournamentStatus,
    pub tier: String,
    pub location: Option<String>,
    pub prize_pool: Option<String>,
}

impl Tournament {
    pub fn tier_color(&self) -> Color {
        match self.tier.as_str() {
            "1" | "S-Tier" | "Major" => Color::Yellow,
            "2" | "A-Tier" | "Minor" => Color::Gray,
            "3" | "B-Tier" | "Qualifier" => Color::White,
            _ => Color::DarkGray,
        }
    }

    pub fn countdown_ratio(&self) -> f64 {
        if self.status != TournamentStatus::Upcoming { return 1.0; }
        let secs_until = (self.start_date - Utc::now()).num_seconds();
        if secs_until <= 0 { return 1.0; }
        let ratio = (86400.0 - secs_until as f64) / 86400.0;
        ratio.clamp(0.0, 1.0)
    }

    pub fn countdown(&self) -> Option<(i64, i64, i64, i64)> {
        if self.status != TournamentStatus::Upcoming { return None; }
        let diff = self.start_date - Utc::now();
        if diff.num_seconds() <= 0 { return None; }
        let total_secs = diff.num_seconds();
        let days = total_secs / 86400;
        let hours = (total_secs % 86400) / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        Some((days, hours, minutes, seconds))
    }
}
