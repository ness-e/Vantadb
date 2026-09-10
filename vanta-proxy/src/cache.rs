//! Exact + semantic response cache (PRX-09 slice 1: exact; slice 2: TTL + LRU + similarity).
//!
//! Key = (protocol label, wire path, post-injection request bytes). Because
//! the key holds the *injected* body, any memory change (new PRX-04 prefix)
//! is a different key — invalidation is implicit; [`ExactCache::invalidate_all`]
//! is the explicit escape hatch. Only small JSON 2xx responses are stored;
//! SSE streams, `stream:true` requests and unknown-length bodies always bypass.
//!
//! Slice 2 adds, all opt-in and additive:
//! - TTL per entry (`CacheConfig::ttl_secs`, 0 = no expiry), lazy on lookup.
//! - LRU recency (lookup refreshes position; slice-1 FIFO behavior preserved
//!   when no lookups happen before eviction).
//! - Similarity hits: same (protocol, path, body-template) + cosine ≥
//!   `similarity_threshold` over the normalized last-user prompt. The template
//!   is the body with the prompt blanked, so a memory/prefix change is a
//!   different template (miss) — semantic never serves a stale context.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::config::CacheConfig;

/// Max response body eligible for caching (4 MiB — bounds memory per entry).
pub const MAX_CACHEABLE_BODY_BYTES: u64 = 4 * 1024 * 1024;

/// TTL sentinel: entries never expire.
pub const TTL_DISABLED: Duration = Duration::ZERO;

/// Default similarity threshold (cosine over normalized prompt TF).
pub const DEFAULT_SIMILARITY_THRESHOLD: f32 = 0.90;

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

/// Exact-match response cache with TTL + bounded LRU eviction + similarity.
#[derive(Debug)]
pub struct ExactCache {
    enabled: bool,
    max_entries: usize,
    ttl: Duration,
    semantic_enabled: bool,
    similarity_threshold: f32,
    map: HashMap<Key, TimedEntry>,
    order: VecDeque<Key>,
}

/// Stored entry with insertion time (TTL) and similarity material
/// (body template + prompt term frequencies, precomputed at store).
#[derive(Debug, Clone)]
struct TimedEntry {
    entry: CachedEntry,
    inserted_at: Instant,
    /// Body with the last-user prompt blanked — a memory/prefix change is a
    /// different template, so semantic never serves across contexts.
    template: Vec<u8>,
    /// Term frequencies of the normalized prompt (empty = not comparable).
    prompt_tf: HashMap<String, u32>,
    prompt_norm: u32,
}

/// True when `inserted_at` is older than `ttl` (`TTL_DISABLED` never expires).
fn is_expired(inserted_at: Instant, ttl: Duration) -> bool {
    ttl != TTL_DISABLED && inserted_at.elapsed() >= ttl
}

impl ExactCache {
    /// Build from config (disabled → every lookup misses, stores are no-ops).
    pub fn new(cfg: CacheConfig) -> Self {
        Self {
            enabled: cfg.enabled,
            max_entries: cfg.max_entries,
            ttl: Duration::from_secs(cfg.ttl_secs),
            semantic_enabled: cfg.semantic_enabled,
            similarity_threshold: cfg.similarity_threshold,
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

    /// Exact lookup over the post-injection request bytes. Refreshes LRU
    /// recency on hit; lazily drops expired entries (returns None).
    pub fn lookup(&mut self, protocol: &str, path: &str, body: &[u8]) -> Option<CachedEntry> {
        if !self.enabled {
            return None;
        }
        let key = Key {
            protocol: protocol.to_string(),
            path: path.to_string(),
            body: body.to_vec(),
        };
        let expired = self
            .map
            .get(&key)
            .is_some_and(|t| is_expired(t.inserted_at, self.ttl));
        if expired {
            self.map.remove(&key);
            self.unlink(&key);
            return None;
        }
        let entry = self.map.get(&key)?.entry.clone();
        self.touch(&key);
        Some(entry)
    }

    /// Similarity lookup over the post-injection body: best cosine ≥
    /// threshold among entries with the same (protocol, path, template).
    /// None when disabled, unparseable, prompt-less, or below threshold.
    /// Refreshes LRU recency on hit; skips (and drops) expired entries.
    pub fn lookup_similar(
        &mut self,
        protocol: &str,
        path: &str,
        body: &[u8],
    ) -> Option<CachedEntry> {
        if !self.enabled || !self.semantic_enabled {
            return None;
        }
        let prompt = extract_prompt_text(body)?;
        let query_tf = term_freq(&normalize(&prompt));
        if query_tf.is_empty() {
            return None;
        }
        let query_norm = norm(&query_tf);
        let template = body_template(body);
        let threshold = self.similarity_threshold;
        let ttl = self.ttl;

        // Scan is O(entries) — bounded by max_entries (128 default).
        let mut best: Option<(Key, f32)> = None;
        let mut expired: Vec<Key> = Vec::new();
        for (key, timed) in &self.map {
            if key.protocol != protocol || key.path != path {
                continue;
            }
            if is_expired(timed.inserted_at, ttl) {
                expired.push(key.clone());
                continue;
            }
            if timed.prompt_tf.is_empty() || timed.template != template {
                continue;
            }
            let sim = cosine(&query_tf, query_norm, &timed.prompt_tf, timed.prompt_norm);
            let better = match &best {
                None => true,
                Some((_, s)) => sim > *s,
            };
            if sim >= threshold && better {
                best = Some((key.clone(), sim));
            }
        }
        for key in expired {
            self.map.remove(&key);
            self.unlink(&key);
        }
        let (winner, _) = best?;
        let entry = self.map.get(&winner)?.entry.clone();
        self.touch(&winner);
        Some(entry)
    }

    /// Move `key` to the back of the recency order.
    /// ponytail: O(n) touch — fine for max_entries ~128; switch to the `lru`
    /// crate (already in the lock file) if caps grow past ~10k.
    fn touch(&mut self, key: &Key) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
        self.order.push_back(key.clone());
    }

    fn unlink(&mut self, key: &Key) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
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
                let timed = slot.get_mut();
                timed.entry = entry;
                timed.inserted_at = Instant::now();
                let (template, prompt_tf, prompt_norm) = similarity_material(body);
                timed.template = template;
                timed.prompt_tf = prompt_tf;
                timed.prompt_norm = prompt_norm;
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
                let (template, prompt_tf, prompt_norm) = similarity_material(body);
                self.map.insert(
                    owned,
                    TimedEntry {
                        entry,
                        inserted_at: Instant::now(),
                        template,
                        prompt_tf,
                        prompt_norm,
                    },
                );
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

/// Precompute similarity material for a request body: (template, prompt TF,
/// prompt squared-norm). Unparseable/prompt-less bodies yield empty TF and
/// never participate in similarity (exact-only).
fn similarity_material(body: &[u8]) -> (Vec<u8>, HashMap<String, u32>, u32) {
    let Some(prompt) = extract_prompt_text(body) else {
        return (body.to_vec(), HashMap::new(), 0);
    };
    let tf = term_freq(&normalize(&prompt));
    let n = norm(&tf);
    (body_template(body), tf, n)
}

/// Last-user prompt text of a chat body (OpenAI `messages[].content: str`
/// or Anthropic `messages[].content[]: {type:"text",text}`).
/// None when unparseable or prompt-less — callers fail open (skip semantic).
fn extract_prompt_text(body: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let messages = value.get("messages")?.as_array()?;
    messages.iter().rev().find_map(|m| {
        if m.get("role").and_then(serde_json::Value::as_str) != Some("user") {
            return None;
        }
        match m.get("content") {
            Some(serde_json::Value::String(s)) => Some(s.clone()),
            Some(serde_json::Value::Array(blocks)) => {
                let texts: Vec<&str> = blocks
                    .iter()
                    .filter(|b| b.get("type").and_then(serde_json::Value::as_str) == Some("text"))
                    .filter_map(|b| b.get("text").and_then(serde_json::Value::as_str))
                    .collect();
                if texts.is_empty() {
                    None
                } else {
                    Some(texts.join("\n"))
                }
            }
            _ => None,
        }
    })
}

/// Body with the last-user prompt blanked — the similarity template.
/// Falls back to the raw body when extraction fails.
fn body_template(body: &[u8]) -> Vec<u8> {
    let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return body.to_vec();
    };
    let Some(messages) = value.get_mut("messages").and_then(|m| m.as_array_mut()) else {
        return body.to_vec();
    };
    let mut blanked = false;
    for m in messages.iter_mut().rev() {
        if m.get("role").and_then(serde_json::Value::as_str) != Some("user") {
            continue;
        }
        match m.get_mut("content") {
            Some(serde_json::Value::String(s)) => {
                s.clear();
                blanked = true;
            }
            Some(serde_json::Value::Array(blocks)) => {
                for b in blocks.iter_mut() {
                    let is_text = b.get("type").and_then(serde_json::Value::as_str) == Some("text");
                    if !is_text {
                        continue;
                    }
                    if let Some(serde_json::Value::String(t)) = b.get_mut("text") {
                        t.clear();
                        blanked = true;
                    }
                }
            }
            _ => {}
        }
        if blanked {
            break;
        }
    }
    if blanked {
        serde_json::to_vec(&value).unwrap_or_else(|_| body.to_vec())
    } else {
        body.to_vec()
    }
}

/// Lowercase alphanumeric token stream (punctuation/case-insensitive).
/// ponytail: lexical similarity, not embeddings — paraphrases with different
/// words miss by design; upgrade to embed-local cosine when a local embedder
/// lands in the workspace (none exists today).
fn normalize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

fn term_freq(tokens: &[String]) -> HashMap<String, u32> {
    let mut tf = HashMap::new();
    for t in tokens {
        *tf.entry(t.clone()).or_insert(0) += 1;
    }
    tf
}

fn norm(tf: &HashMap<String, u32>) -> u32 {
    tf.values().map(|c| c * c).sum()
}

fn cosine(a: &HashMap<String, u32>, a_norm: u32, b: &HashMap<String, u32>, b_norm: u32) -> f32 {
    if a_norm == 0 || b_norm == 0 {
        return 0.0;
    }
    let (small, big) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let dot: u32 = small
        .iter()
        .map(|(t, c)| c * big.get(t).unwrap_or(&0))
        .sum();
    dot as f32 / ((a_norm as f32).sqrt() * (b_norm as f32).sqrt())
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
            ..Default::default()
        })
    }

    #[test]
    fn disabled_cache_never_hits_nor_stores() {
        let mut cache = ExactCache::new(CacheConfig {
            enabled: false,
            max_entries: 8,
            ..Default::default()
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

    // --- PRX-09 slice 2 RED: TTL + LRU + similitud (API inexistente) ---

    fn slice2_cfg(max: usize) -> CacheConfig {
        CacheConfig {
            enabled: true,
            max_entries: max,
            ttl_secs: 0,
            semantic_enabled: true,
            similarity_threshold: 0.9,
        }
    }

    fn openai_body(prompt: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "model": "m",
            "messages": [{ "role": "user", "content": prompt }],
        }))
        .expect("json")
    }

    #[test]
    fn ttl_expiry_predicate() {
        use std::time::{Duration, Instant};
        // Entrada vieja con TTL corto → expirada; fresca → viva; TTL_ZERO nunca expira.
        assert!(is_expired(
            Instant::now() - Duration::from_secs(10),
            Duration::from_secs(5)
        ));
        assert!(!is_expired(Instant::now(), Duration::from_secs(60)));
        assert!(!is_expired(
            Instant::now() - Duration::from_secs(10_000),
            super::TTL_DISABLED
        ));
    }

    #[test]
    fn lru_touch_refreshes_recency() {
        let mut cache = ExactCache::new(slice2_cfg(2));
        cache.store("openai", "/p", b"a", entry(b"a"));
        cache.store("openai", "/p", b"b", entry(b"b"));
        // Tocar `a` la vuelve más reciente que `b`: la próxima evicción cae en `b`.
        assert!(cache.lookup("openai", "/p", b"a").is_some());
        cache.store("openai", "/p", b"c", entry(b"c"));
        assert_eq!(cache.len(), 2);
        assert!(
            cache.lookup("openai", "/p", b"a").is_some(),
            "touched stays"
        );
        assert!(
            cache.lookup("openai", "/p", b"b").is_none(),
            "untouched evicted"
        );
        assert!(cache.lookup("openai", "/p", b"c").is_some());
    }

    #[test]
    fn similar_hit_near_duplicate() {
        let mut cache = ExactCache::new(slice2_cfg(8));
        let stored = openai_body("What is the capital of France?");
        cache.store("openai", "/p", &stored, entry(b"paris"));
        // Mismo prompt normalizado (case/puntuación) → hit semántico sin exact-hit.
        let near = openai_body("what is the capital of france");
        assert!(
            cache.lookup("openai", "/p", &near).is_none(),
            "no exact hit"
        );
        let hit = cache
            .lookup_similar("openai", "/p", &near)
            .expect("semantic hit");
        assert_eq!(hit.body, b"paris");
    }

    #[test]
    fn similar_miss_different_prompt() {
        let mut cache = ExactCache::new(slice2_cfg(8));
        let stored = openai_body("What is the capital of France?");
        cache.store("openai", "/p", &stored, entry(b"paris"));
        let other = openai_body("Explain quantum entanglement in detail please");
        assert!(cache.lookup_similar("openai", "/p", &other).is_none());
    }

    #[test]
    fn similar_threshold_configurable() {
        // Prompts con solapamiento parcial: threshold bajo hitea, alto no.
        let a = openai_body("deploy the app to production now");
        let b = openai_body("deploy the app to staging now");
        let mut loose = ExactCache::new(CacheConfig {
            similarity_threshold: 0.5,
            ..slice2_cfg(8)
        });
        loose.store("openai", "/p", &a, entry(b"ok"));
        assert!(loose.lookup_similar("openai", "/p", &b).is_some());

        let mut strict = ExactCache::new(CacheConfig {
            similarity_threshold: 0.99,
            ..slice2_cfg(8)
        });
        strict.store("openai", "/p", &a, entry(b"ok"));
        assert!(strict.lookup_similar("openai", "/p", &b).is_none());
    }

    #[test]
    fn similar_disabled_by_default() {
        let mut cache = ExactCache::new(CacheConfig {
            enabled: true,
            max_entries: 8,
            ..Default::default()
        });
        let stored = openai_body("What is the capital of France?");
        cache.store("openai", "/p", &stored, entry(b"paris"));
        let near = openai_body("what is the capital of france");
        assert!(cache.lookup_similar("openai", "/p", &near).is_none());
    }

    #[test]
    fn extract_prompt_openai_and_anthropic() {
        let openai = openai_body("hello there");
        assert_eq!(extract_prompt_text(&openai).as_deref(), Some("hello there"));

        let anthropic = serde_json::to_vec(&serde_json::json!({
            "model": "c",
            "messages": [{
                "role": "user",
                "content": [{ "type": "text", "text": "hi anthropic" }],
            }],
        }))
        .expect("json");
        assert_eq!(
            extract_prompt_text(&anthropic).as_deref(),
            Some("hi anthropic")
        );

        assert!(extract_prompt_text(b"not-json").is_none());
        assert!(extract_prompt_text(br#"{"model":"m"}"#).is_none());
    }
}
