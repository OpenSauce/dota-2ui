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
    fn from(e: reqwest::Error) -> Self {
        ApiError::Http(e)
    }
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

/// Combined result from a single fetch operation.
pub struct FetchAllResult {
    pub matches: Vec<Match>,
    pub tournaments: Vec<Tournament>,
}

pub trait MatchProvider: Send + Sync {
    /// Fetch all data in as few API requests as possible.
    fn fetch_all(&self) -> Pin<Box<dyn Future<Output = ApiResult<FetchAllResult>> + Send + '_>>;
}

pub fn provider_from_config(api_key: Option<&str>) -> Box<dyn MatchProvider> {
    match api_key {
        Some(key) => Box::new(pandascore::PandaScoreProvider::new(key.to_string())),
        None => Box::new(liquipedia::LiquipediaProvider::new()),
    }
}
