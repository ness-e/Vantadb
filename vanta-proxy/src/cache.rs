//! Exact response cache (PRX-09 slice 1: exact-only; semantic is DEFER).
//!
//! Key = (protocol label, wire path, post-injection request bytes). Because
//! the key holds the *injected* body, any memory change (new PRX-04 prefix)
//! is a different key — invalidation is implicit; [`ExactCache::invalidate_all`]
//! is the explicit escape hatch. Only small JSON 2xx responses are stored;
//! SSE streams, `stream:true` requests and unknown-length bodies always bypass.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};

use crate::config::CacheConfig;

/// Max response body eligible for caching (4 MiB — bounds memory per entry).
pub const MAX_CACHEABLE_BODY_BYTES: u64 = 4 * 1024 * 1024;

/// A stored exact response (replayed byte-for-byte on hit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedEntry {
    pub status: u16,
    pub content_type: String,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    protocol: String,
    path: String,
    body: Vec<u8>,
}

/// Exact-match response cache with bounded FIFO eviction.
#[derive(Debug, Default)]
pub struct ExactCache {
    enabled: bool,
    max_entries: usize,
    map: HashMap<Key, CachedEntry>,
    order: VecDeque<Key>,
}

impl ExactCache {
    /// Build from config (disabled → every lookup misses, stores are no-ops).
    pub fn new(cfg: CacheConfig) -> Self {
        Self {
            enabled: cfg.enabled,
            max_entries: cfg.max_entries,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// False when the cache is disabled — callers must not store either.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Number of entries currently held.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// True when no entry is held.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Exact lookup over the post-injection request bytes.
    pub fn lookup(&self, protocol: &str, path: &str, body: &[u8]) -> Option<CachedEntry> {
        if !self.enabled {
            return None;
        }
        let key = Key {
            protocol: protocol.to_string(),
            path: path.to_string(),
            body: body.to_vec(),
        };
        self.map.get(&key).cloned()
    }

    /// Store an exact response. Same key overwrites in place (keeps its
    /// FIFO position); a new key evicts oldest-first past `max_entries`.
    pub fn store(&mut self, protocol: &str, path: &str, body: &[u8], entry: CachedEntry) {
        if !self.enabled || self.max_entries == 0 {
            return;
        }
        let key = Key {
            protocol: protocol.to_string(),
            path: path.to_string(),
            body: body.to_vec(),
        };
        match self.map.entry(key) {
            Entry::Occupied(mut slot) => {
                slot.insert(entry);
            }
            Entry::Vacant(slot) => {
                let owned = slot.key().clone();
                // ponytail: FIFO eviction O(1) amortized — upgrade to LRU only
                // if hit-rate data shows recency matters more than insertion order.
                while self.map.len() >= self.max_entries {
                    match self.order.pop_front() {
                        Some(old) => {
                            self.map.remove(&old);
                        }
                        None => break,
                    }
                }
                self.order.push_back(owned.clone());
                self.map.insert(owned, entry);
            }
        }
    }

    /// Drop every entry (explicit invalidation; implicit invalidation comes
    /// free from the post-injection key).
    pub fn invalidate_all(&mut self) {
        self.map.clear();
        self.order.clear();
    }
}

/// True when a request body is eligible for exact caching: a JSON object
/// without `"stream": true` (streaming responses are unbounded event streams).
pub fn is_cacheable_request(body: &[u8]) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return false;
    };
    if !value.is_object() {
        return false;
    }
    !value
        .get("stream")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

/// True when a response is eligible for exact caching: 2xx JSON. Body
/// length is enforced at collection time ([`MAX_CACHEABLE_BODY_BYTES`] cap) —
/// `forward.rs` strips `content-length` for streaming safety, so no length
/// gate lives here. Non-JSON (SSE et al.) always bypasses.
pub fn is_cacheable_response(status: u16, content_type: Option<&str>) -> bool {
    if !(200..300).contains(&status) {
        return false;
    }
    content_type.is_some_and(|ct| ct.contains("json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(body: &[u8]) -> CachedEntry {
        CachedEntry {
            status: 200,
            content_type: "application/json".to_string(),
            body: body.to_vec(),
        }
    }

    fn enabled(n: usize) -> ExactCache {
        ExactCache::new(CacheConfig {
            enabled: true,
            max_entries: n,
        })
    }

    #[test]
    fn disabled_cache_never_hits_nor_stores() {
        let mut cache = ExactCache::new(CacheConfig {
            enabled: false,
            max_entries: 8,
        });
        cache.store("openai", "/p", b"{}", entry(b"{}"));
        assert!(cache.is_empty());
        assert!(cache.lookup("openai", "/p", b"{}").is_none());
    }

    #[test]
    fn same_key_overwrites_zero_max_stores_nothing() {
        let mut cache = enabled(8);
        cache.store("openai", "/p", b"{}", entry(b"1"));
        cache.store("openai", "/p", b"{}", entry(b"2"));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.lookup("openai", "/p", b"{}").expect("hit").body, b"2");

        let mut zero = enabled(0);
        zero.store("openai", "/p", b"{}", entry(b"1"));
        assert!(zero.is_empty());
    }

    #[test]
    fn fifo_evicts_oldest_first() {
        let mut cache = enabled(2);
        cache.store("openai", "/p", b"a", entry(b"a"));
        cache.store("openai", "/p", b"b", entry(b"b"));
        cache.store("openai", "/p", b"c", entry(b"c"));
        assert_eq!(cache.len(), 2);
        assert!(cache.lookup("openai", "/p", b"a").is_none());
        assert!(cache.lookup("openai", "/p", b"b").is_some());
        assert!(cache.lookup("openai", "/p", b"c").is_some());
        cache.invalidate_all();
        assert!(cache.is_empty());
    }

    #[test]
    fn request_gates() {
        assert!(is_cacheable_request(br#"{"model":"m"}"#));
        assert!(!is_cacheable_request(br#"{"model":"m","stream":true}"#));
        assert!(!is_cacheable_request(b"not-json"));
        assert!(!is_cacheable_request(b"[1,2]"));
    }

    #[test]
    fn response_gates() {
        assert!(is_cacheable_response(200, Some("application/json")));
        assert!(is_cacheable_response(
            200,
            Some("application/json; charset=utf-8")
        ));
        assert!(!is_cacheable_response(500, Some("application/json")));
        assert!(!is_cacheable_response(200, Some("text/event-stream")));
        assert!(!is_cacheable_response(200, None));
    }
}
