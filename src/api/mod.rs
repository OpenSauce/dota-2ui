pub mod liquipedia;
pub mod pandascore;

use crate::models::{Match, Tournament};
use std::future::Future;
use std::pin::Pin;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    Http(reqwest::Error),
    Parse(String),
    RateLimit,
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self { ApiError::Http(e) }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Http(e) => write!(f, "HTTP error: {}", e),
            ApiError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ApiError::RateLimit => write!(f, "Rate limited"),
        }
    }
}

pub trait MatchProvider: Send + Sync {
    fn fetch_matches(&self) -> Pin<Box<dyn Future<Output = ApiResult<Vec<Match>>> + Send + '_>>;
    fn fetch_tournaments(&self) -> Pin<Box<dyn Future<Output = ApiResult<Vec<Tournament>>> + Send + '_>>;
}

pub fn provider_from_config(api_key: Option<&str>) -> Box<dyn MatchProvider> {
    match api_key {
        Some(key) => Box::new(pandascore::PandaScoreProvider::new(key.to_string())),
        None => Box::new(liquipedia::LiquipediaProvider::new()),
    }
}
