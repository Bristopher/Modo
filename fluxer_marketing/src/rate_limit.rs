// SPDX-License-Identifier: AGPL-3.0-or-later

use moka::future::Cache;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Cache<String, Arc<AtomicU32>>,
    max_requests: u32,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            buckets: Cache::builder()
                .max_capacity(16_384)
                .time_to_live(window)
                .build(),
            max_requests,
        }
    }

    pub async fn check(&self, key: &str) -> bool {
        let counter = self
            .buckets
            .get_with(key.to_owned(), async { Arc::new(AtomicU32::new(0)) })
            .await;
        let count = counter.fetch_add(1, Ordering::Relaxed).saturating_add(1);
        count <= self.max_requests
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allows_up_to_the_limit_then_blocks() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));
        assert!(limiter.check("1.2.3.4").await);
        assert!(limiter.check("1.2.3.4").await);
        assert!(limiter.check("1.2.3.4").await);
        assert!(!limiter.check("1.2.3.4").await);
    }

    #[tokio::test]
    async fn tracks_keys_independently() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.check("a").await);
        assert!(!limiter.check("a").await);
        assert!(limiter.check("b").await);
    }
}
