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
fn test_save_preserves_api_key() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");

    // Simulate user's config file with API key
    std::fs::write(
        &path,
        "refresh_interval = 120\nfavorite_teams = [\"OG\"]\nfavorite_tournaments = []\nenable_notifications = false\npandascore_api_key = \"my-secret-key\"\n",
    )
    .unwrap();

    // Load it
    let mut config = Config::load_from(&path).unwrap();
    assert_eq!(config.pandascore_api_key.as_deref(), Some("my-secret-key"));

    // Toggle a favorite (this triggers save in the app)
    config.toggle_favorite_team("Team Liquid");
    config.save_to(&path).unwrap();

    // Verify the saved file
    let content = std::fs::read_to_string(&path).unwrap();
    eprintln!("Saved config:\n{}", content);
    assert!(
        content.contains("my-secret-key"),
        "API key was lost! File content:\n{}",
        content
    );
    assert!(
        !content.contains("\n = "),
        "Empty key found in config! File content:\n{}",
        content
    );

    // Reload and verify
    let reloaded = Config::load_from(&path).unwrap();
    assert_eq!(
        reloaded.pandascore_api_key.as_deref(),
        Some("my-secret-key")
    );
    assert!(reloaded.favorite_teams.contains(&"OG".to_string()));
    assert!(reloaded.favorite_teams.contains(&"Team Liquid".to_string()));
}

#[test]
fn test_save_preserves_externally_added_api_key() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");

    // App starts without API key
    let mut config = Config::default();
    config.favorite_teams.push("OG".to_string());
    config.save_to(&path).unwrap();

    // User manually adds API key to the file
    std::fs::write(
        &path,
        "refresh_interval = 120\nfavorite_teams = [\"OG\"]\nfavorite_tournaments = []\nenable_notifications = false\npandascore_api_key = \"my-secret-key\"\n",
    )
    .unwrap();

    // App saves again (e.g. user toggled a favorite) — key should be preserved
    config.toggle_favorite_team("Team Liquid");
    config.save_to(&path).unwrap();

    let reloaded = Config::load_from(&path).unwrap();
    assert_eq!(
        reloaded.pandascore_api_key.as_deref(),
        Some("my-secret-key"),
        "API key was lost after save!"
    );
    assert!(reloaded.favorite_teams.contains(&"Team Liquid".to_string()));
}

#[test]
fn test_save_default_no_empty_keys() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");

    let config = Config::default();
    config.save_to(&path).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    eprintln!("Default config serialized:\n{}", content);
    assert!(
        !content.contains("\n = "),
        "Empty key found in default config! File content:\n{}",
        content
    );
}

#[test]
fn test_toggle_favorite_team() {
    let mut c = Config::default();
    c.toggle_favorite_team("Liquid");
    assert!(c.favorite_teams.contains(&"Liquid".to_string()));
    c.toggle_favorite_team("Liquid");
    assert!(!c.favorite_teams.contains(&"Liquid".to_string()));
}
