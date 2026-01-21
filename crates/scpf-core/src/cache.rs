use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::fs;
use xxhash_rust::xxh3::xxh3_64;

const DEFAULT_TTL_SECS: u64 = 86400; // 24 hours

pub struct Cache {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl Cache {
    pub async fn new(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)
            .await
            .context("Failed to create cache directory")?;
        Ok(Self {
            cache_dir,
            ttl: Duration::from_secs(DEFAULT_TTL_SECS),
        })
    }

    pub async fn with_ttl(cache_dir: PathBuf, ttl: Duration) -> Result<Self> {
        fs::create_dir_all(&cache_dir)
            .await
            .context("Failed to create cache directory")?;
        Ok(Self { cache_dir, ttl })
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let path = self.cache_path(key);

        // Check if cache entry exists and is not expired
        if let Ok(metadata) = fs::metadata(&path).await {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed > self.ttl {
                        // Cache expired, remove it
                        let _ = fs::remove_file(&path).await;
                        return None;
                    }
                }
            }
        }

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
