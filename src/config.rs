use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pandascore_api_key: Option<String>,
    #[serde(default)]
    pub favorite_teams: Vec<String>,
    #[serde(default)]
    pub favorite_tournaments: Vec<String>,
    #[serde(default)]
    pub enable_notifications: bool,
}

fn default_refresh_interval() -> u64 {
    120
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_interval: 120,
            pandascore_api_key: None,
            favorite_teams: Vec::new(),
            favorite_tournaments: Vec::new(),
            enable_notifications: false,
        }
    }
}

impl Config {
    pub fn load() -> io::Result<Self> {
        Self::load_from(Self::config_path())
    }

    pub fn save(&self) -> io::Result<()> {
        self.save_to(Self::config_path())
    }

    pub fn load_from<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        // Re-read the on-disk config to preserve fields the user may have
        // edited externally (e.g. pandascore_api_key added by hand).
        let mut to_save = self.clone();
        if to_save.pandascore_api_key.is_none() {
            if let Ok(disk) = Self::load_from(path) {
                to_save.pandascore_api_key = disk.pandascore_api_key;
            }
        }
        let content = toml::to_string_pretty(&to_save).map_err(io::Error::other)?;
        fs::write(path, content)
    }

    pub fn toggle_favorite_team(&mut self, name: &str) {
        if let Some(pos) = self.favorite_teams.iter().position(|t| t == name) {
            self.favorite_teams.remove(pos);
        } else {
            self.favorite_teams.push(name.to_string());
        }
    }

    pub fn toggle_favorite_tournament(&mut self, name: &str) {
        if let Some(pos) = self.favorite_tournaments.iter().position(|t| t == name) {
            self.favorite_tournaments.remove(pos);
        } else {
            self.favorite_tournaments.push(name.to_string());
        }
    }

    fn config_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("dota-tui")
            .join("config.toml")
    }
}
