// SPDX-License-Identifier: AGPL-3.0-or-later

use bytes::Bytes;
use moka::sync::Cache as MokaCache;
use std::time::Duration;

#[derive(Debug)]
pub struct Cache {
    enabled: bool,
    max_entry_bytes: usize,
    inner: MokaCache<String, Bytes>,
}

impl Cache {
    pub fn new(capacity_bytes: usize, max_entry_bytes: usize, ttl_ms: u64) -> Self {
        let enabled = capacity_bytes > 0 && max_entry_bytes > 0 && ttl_ms > 0;
        Self {
            enabled,
            max_entry_bytes: max_entry_bytes.min(capacity_bytes),
            inner: MokaCache::builder()
                .max_capacity(capacity_bytes as u64)
                .weigher(|_key: &String, value: &Bytes| -> u32 {
                    value.len().min(u32::MAX as usize) as u32
                })
                .time_to_live(Duration::from_millis(ttl_ms))
                .build(),
        }
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        if !self.enabled {
            return None;
        }
        self.inner.get(key)
    }

    pub fn put(&self, key: impl Into<String>, data: Bytes) {
        if !self.enabled || data.is_empty() || data.len() > self.max_entry_bytes {
            return;
        }
        self.inner.insert(key.into(), data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_cache_roundtrips_returning_shared_handles() {
        let cache = Cache::new(64, 64, 60_000);
        cache.put("a", Bytes::from_static(b"abc"));
        let a1 = cache.get("a").unwrap();
        assert_eq!(b"abc", a1.as_ref());
        let a2 = cache.get("a").unwrap();
        assert_eq!(a1.as_ptr(), a2.as_ptr());
    }

    #[test]
    fn byte_cache_bounds_total_weighted_size() {
        let cache = Cache::new(8, 8, 60_000);
        for i in 0..50 {
            cache.put(format!("k{i}"), Bytes::from(vec![0u8; 4]));
        }
        cache.inner.run_pending_tasks();
        assert!(cache.inner.weighted_size() <= 8);
    }

    #[test]
    fn byte_cache_skips_entries_over_max_entry_size() {
        let cache = Cache::new(32, 4, 60_000);
        cache.put("large", Bytes::from_static(b"12345"));
        assert_eq!(None, cache.get("large"));
    }

    #[test]
    fn byte_cache_disabled_when_capacity_or_ttl_is_zero() {
        let no_capacity = Cache::new(0, 4, 60_000);
        no_capacity.put("a", Bytes::from_static(b"abc"));
        assert_eq!(None, no_capacity.get("a"));

        let no_ttl = Cache::new(32, 4, 0);
        no_ttl.put("a", Bytes::from_static(b"abc"));
        assert_eq!(None, no_ttl.get("a"));
    }
}
