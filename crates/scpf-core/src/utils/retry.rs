use backoff::ExponentialBackoff;
use std::time::Duration;

pub struct RetryPolicy {
    initial_interval: Duration,
}

impl RetryPolicy {
    pub fn new(initial_interval: Duration) -> Self {
        Self { initial_interval }
    }

    pub fn backoff(&self) -> ExponentialBackoff {
        ExponentialBackoff {
            initial_interval: self.initial_interval,
            max_elapsed_time: Some(Duration::from_secs(60)),
            ..Default::default()
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}
