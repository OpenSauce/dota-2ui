use dota_2ui::cache::DiskCache;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_cache_write_and_read() {
    let dir = TempDir::new().unwrap();
    let cache = DiskCache::new(dir.path().to_path_buf());
    cache.write("matches", r#"[{"id":"1"}]"#).unwrap();
    let result = cache.read("matches", Duration::from_secs(300)).unwrap();
    assert_eq!(result.unwrap(), r#"[{"id":"1"}]"#);
}

#[test]
fn test_cache_expired() {
    let dir = TempDir::new().unwrap();
    let cache = DiskCache::new(dir.path().to_path_buf());
    cache.write("old", "data").unwrap();
    let result = cache.read("old", Duration::from_secs(0)).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_cache_missing_key() {
    let dir = TempDir::new().unwrap();
    let cache = DiskCache::new(dir.path().to_path_buf());
    let result = cache.read("nonexistent", Duration::from_secs(300)).unwrap();
    assert!(result.is_none());
}
