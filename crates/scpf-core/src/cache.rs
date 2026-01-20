use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;
use xxhash_rust::xxh3::xxh3_64;

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub async fn new(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)
            .await
            .context("Failed to create cache directory")?;
        Ok(Self { cache_dir })
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let path = self.cache_path(key);
        fs::read_to_string(path).await.ok()
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let path = self.cache_path(key);
        let temp_path = path.with_extension("tmp");

        fs::write(&temp_path, value)
            .await
            .context("Failed to write to temporary cache file")?;

        fs::rename(&temp_path, &path)
            .await
            .context("Failed to atomically move cache file")?;

        Ok(())
    }

    fn cache_path(&self, key: &str) -> PathBuf {
        let hash = xxh3_64(key.as_bytes());
        self.cache_dir.join(format!("{:x}", hash))
    }
}
