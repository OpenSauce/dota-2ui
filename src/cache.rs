use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

pub struct DiskCache {
    dir: PathBuf,
}

impl DiskCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn default_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dota-tui")
    }

    pub fn write(&self, key: &str, data: &str) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        fs::write(self.dir.join(format!("{}.json", key)), data)
    }

    pub fn read(&self, key: &str, ttl: Duration) -> io::Result<Option<String>> {
        let path = self.dir.join(format!("{}.json", key));
        if !path.exists() {
            return Ok(None);
        }
        let age = fs::metadata(&path)?
            .modified()?
            .elapsed()
            .unwrap_or(Duration::MAX);
        if age > ttl {
            return Ok(None);
        }
        Ok(Some(fs::read_to_string(&path)?))
    }
}
