use anyhow::{Context, Result};
use tokio::fs;
use std::path::PathBuf;

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub async fn new(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir).await.context("Failed to create cache directory")?;
        Ok(Self { cache_dir })
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let path = self.cache_path(key);
        fs::read_to_string(path).await.ok()
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let path = self.cache_path(key);
        fs::write(path, value).await.context("Failed to write to cache")?;
        Ok(())
    }

    fn cache_path(&self, key: &str) -> PathBuf {
        let hash = format!("{:x}", md5::compute(key));
        self.cache_dir.join(hash)
    }
}
