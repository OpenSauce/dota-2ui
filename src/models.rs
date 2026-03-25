use chrono::{DateTime, Utc};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Team {
    pub name: String,
    pub tag: String,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MatchStatus {
    Upcoming,
    Live,
    Completed,
}

impl MatchStatus {
    pub fn is_live(&self) -> bool {
        matches!(self, MatchStatus::Live)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SeriesFormat {
    Bo1,
    Bo2,
    Bo3,
    Bo5,
}

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
    #[serde(default)]
    pub stage: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TournamentStatus {
    Upcoming,
    Live,
    Completed,
}

impl TournamentStatus {
    #[allow(dead_code)]
    pub fn is_live(&self) -> bool {
        matches!(self, TournamentStatus::Live)
    }
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

impl Match {
    /// Human-readable relative time string for this match.
    pub fn relative_time(&self) -> String {
        match self.status {
            MatchStatus::Live => "LIVE".to_string(),
            MatchStatus::Completed => "Final".to_string(),
            MatchStatus::Upcoming => {
                let diff = self.start_time - Utc::now();
                let secs = diff.num_seconds();
                if secs < 0 {
                    "starting soon".to_string()
                } else if secs < 60 {
                    "now".to_string()
                } else if secs < 3600 {
                    format!("in {}m", secs / 60)
                } else if secs < 86400 {
                    format!("in {}h {}m", secs / 3600, (secs % 3600) / 60)
                } else if secs < 604800 {
                    format!("in {}d {}h", secs / 86400, (secs % 86400) / 3600)
                } else {
                    format!("in {}d", secs / 86400)
                }
            }
        }
    }

    /// Color representing urgency of this match's timing.
    pub fn urgency_color(&self) -> Color {
        match self.status {
            MatchStatus::Live => Color::Red,
            MatchStatus::Completed => Color::DarkGray,
            MatchStatus::Upcoming => {
                let secs = (self.start_time - Utc::now()).num_seconds();
                if secs < 900 {
                    Color::Red
                } else if secs < 7200 {
                    Color::Yellow
                } else {
                    Color::DarkGray
                }
            }
        }
    }

    /// Check if this match involves a team (case-insensitive, checks name and tag).
    pub fn involves_team(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        self.team_a.name.to_lowercase() == lower
            || self.team_a.tag.to_lowercase() == lower
            || self.team_b.name.to_lowercase() == lower
            || self.team_b.tag.to_lowercase() == lower
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BracketType {
    SingleElim,
    DoubleElim,
    GroupStage,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BracketViewMode {
    Column,
    AsciiTree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BracketMatch {
    pub match_id: String,
    pub round: usize,
    pub position: usize,
    pub team_a: Option<String>,
    pub team_b: Option<String>,
    pub score_a: u8,
    pub score_b: u8,
    pub status: MatchStatus,
    pub winner_to: Option<(usize, usize)>,
    pub loser_to: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BracketRound {
    pub round: usize,
    pub name: String,
    pub matches: Vec<BracketMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bracket {
    pub bracket_type: BracketType,
    pub upper_rounds: Vec<BracketRound>,
    pub lower_rounds: Option<Vec<BracketRound>>,
    pub grand_final: Option<BracketMatch>,
}

#[derive(Debug, Clone)]
pub enum FetchStatus {
    Loading,
    Ready,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct GameDetail {
    pub game_number: u8,
    pub status: MatchStatus,
    pub winner: Option<String>,
    pub duration: Option<Duration>,
}

/// Holds only data NOT already in Match.
/// Match provides: team_a, team_b, score_a, score_b, series_format,
/// tournament_name, start_time, stream_url, stage.
#[derive(Debug, Clone)]
pub struct MatchDetailData {
    pub games: Vec<GameDetail>,
    pub fetch_status: FetchStatus,
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
        if self.status != TournamentStatus::Upcoming {
            return 1.0;
        }
        let secs_until = (self.start_date - Utc::now()).num_seconds();
        if secs_until <= 0 {
            return 1.0;
        }
        let ratio = (86400.0 - secs_until as f64) / 86400.0;
        ratio.clamp(0.0, 1.0)
    }

    pub fn countdown(&self) -> Option<(i64, i64, i64, i64)> {
        if self.status != TournamentStatus::Upcoming {
            return None;
        }
        let diff = self.start_date - Utc::now();
        if diff.num_seconds() <= 0 {
            return None;
        }
        let total_secs = diff.num_seconds();
        let days = total_secs / 86400;
        let hours = (total_secs % 86400) / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        Some((days, hours, minutes, seconds))
    }
}
