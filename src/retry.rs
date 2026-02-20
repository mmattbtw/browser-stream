use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff: Duration,
}

impl RetryPolicy {
    pub fn new(max_retries: u32, backoff: Duration) -> Self {
        Self {
            max_retries,
            backoff,
        }
    }

    pub fn should_retry(&self, failures_so_far: u32) -> bool {
        failures_so_far <= self.max_retries
    }
}
