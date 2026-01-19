use backon::ExponentialBuilder;
use std::time::Duration;

pub struct RetryPolicy {
    initial_interval: Duration,
}

impl RetryPolicy {
    pub fn new(initial_interval: Duration) -> Self {
        Self { initial_interval }
    }

    pub fn backoff(&self) -> ExponentialBuilder {
        ExponentialBuilder::default()
            .with_min_delay(self.initial_interval)
            .with_max_times(10)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}
