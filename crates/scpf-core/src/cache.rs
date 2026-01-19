use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;
        Ok(Self { cache_dir })
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let path = self.cache_path(key);
        fs::read_to_string(path).ok()
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let path = self.cache_path(key);
        fs::write(path, value).context("Failed to write to cache")?;
        Ok(())
    }

    fn cache_path(&self, key: &str) -> PathBuf {
        let hash = format!("{:x}", md5::compute(key));
        self.cache_dir.join(hash)
    }
}
