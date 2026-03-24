use dota_2ui::config::Config;
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let c = Config::default();
    assert_eq!(c.refresh_interval, 120);
    assert!(c.pandascore_api_key.is_none());
}

#[test]
fn test_config_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    let mut c = Config {
        refresh_interval: 30,
        pandascore_api_key: Some("test-key".to_string()),
        ..Config::default()
    };
    c.favorite_teams.push("Team Liquid".to_string());
    c.save_to(&path).unwrap();
    let loaded = Config::load_from(&path).unwrap();
    assert_eq!(loaded.refresh_interval, 30);
    assert_eq!(loaded.favorite_teams, vec!["Team Liquid"]);
    assert_eq!(loaded.pandascore_api_key.as_deref(), Some("test-key"));
}

#[test]
fn test_config_load_missing_returns_default() {
    let c = Config::load_from("/tmp/nonexistent-dota-2ui-test.toml").unwrap();
    assert_eq!(c.refresh_interval, 120);
}

#[test]
fn test_toggle_favorite_team() {
    let mut c = Config::default();
    c.toggle_favorite_team("Liquid");
    assert!(c.favorite_teams.contains(&"Liquid".to_string()));
    c.toggle_favorite_team("Liquid");
    assert!(!c.favorite_teams.contains(&"Liquid".to_string()));
}
