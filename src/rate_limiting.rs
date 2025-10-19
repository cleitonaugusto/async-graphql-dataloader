use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::error::RateLimitError;

#[derive(Clone)]
pub struct RateLimiter {
    limits: Arc<RwLock<HashMap<String, RateLimit>>>,
}

struct RateLimit {
    requests: u64,
    window_start: Instant,
    max_requests: u64,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_limit(
        &self,
        key: &str,
        max_requests: u64,
        window: Duration
    ) -> Result<(), RateLimitError> {
        let mut limits = self.limits.write().await;
        let now = Instant::now();

        let limit = limits.entry(key.to_string()).or_insert(RateLimit {
            requests: 0,
            window_start: now,
            max_requests,
            window_duration: window,
        });

        // Reset window if expired or configuration changed
        if now.duration_since(limit.window_start) > limit.window_duration
            || limit.max_requests != max_requests
            || limit.window_duration != window {

            limit.requests = 0;
            limit.window_start = now;
            limit.max_requests = max_requests;
            limit.window_duration = window;
        }

        if limit.requests >= limit.max_requests {
            let reset_in = limit.window_duration - now.duration_since(limit.window_start);
            Err(RateLimitError::LimitExceeded {
                key: key.to_string(),
                max_requests,
                reset_in,
                retry_after: reset_in.as_secs(),
            })
        } else {
            limit.requests += 1;
            Ok(())
        }
    }

    pub async fn get_usage(&self, key: &str) -> Option<RateLimitUsage> {
        let limits = self.limits.read().await;
        let limit = limits.get(key)?;

        let now = Instant::now();
        let window_elapsed = now.duration_since(limit.window_start);
        let window_remaining = limit.window_duration.checked_sub(window_elapsed)
            .unwrap_or(Duration::from_secs(0));

        Some(RateLimitUsage {
            current_requests: limit.requests,
            max_requests: limit.max_requests,
            window_remaining,
            reset_in: window_remaining,
        })
    }

    pub async fn clear_limits(&self) {
        self.limits.write().await.clear();
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitUsage {
    pub current_requests: u64,
    pub max_requests: u64,
    pub window_remaining: Duration,
    pub reset_in: Duration,
}