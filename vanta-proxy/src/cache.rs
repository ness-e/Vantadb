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
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::CacheConfig;

/// Max response body eligible for caching (4 MiB — bounds memory per entry).
pub const MAX_CACHEABLE_BODY_BYTES: u64 = 4 * 1024 * 1024;

/// TTL sentinel: entries never expire.
pub const TTL_DISABLED: Duration = Duration::ZERO;

/// Default similarity threshold (cosine over normalized prompt TF).
pub const DEFAULT_SIMILARITY_THRESHOLD: f32 = 0.90;

/// Dense-vector embedding hook for the similarity path (PRX-09-embeddings).
///
/// Returning `None` (provider failure, empty text) must never break the
/// lookup — callers degrade to the lexical TF-cosine path (P4 fail-open).
/// The bundled [`OllamaEmbedProvider`] talks to an Ollama-compatible
/// `/api/embed` endpoint; tests use a deterministic in-memory fake.
pub trait EmbedProvider: Send + Sync {
    /// Embed `text` into a dense vector, or `None` when unavailable.
    fn embed(&self, text: &str) -> Option<Vec<f32>>;
}

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
pub struct ExactCache {
    enabled: bool,
    max_entries: usize,
    ttl: Duration,
    semantic_enabled: bool,
    similarity_threshold: f32,
    /// Optional dense-vector hook (PRX-09-embeddings). `None` (default) keeps
    /// the slice-2 lexical path byte-for-byte; `Some` tries embeddings first
    /// and degrades to lexical on any failure.
    embedder: Option<Arc<dyn EmbedProvider>>,
    map: HashMap<Key, TimedEntry>,
    order: VecDeque<Key>,
}

impl std::fmt::Debug for ExactCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExactCache")
            .field("enabled", &self.enabled)
            .field("max_entries", &self.max_entries)
            .field("ttl", &self.ttl)
            .field("semantic_enabled", &self.semantic_enabled)
            .field("similarity_threshold", &self.similarity_threshold)
            .field("embedder", &self.embedder.is_some())
            .field("len", &self.map.len())
            .finish()
    }
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
    /// Dense vector of the prompt (`None` = no embedder or hook failed —
    /// that entry stays lexical-only).
    prompt_vec: Option<Vec<f32>>,
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
            embedder: None,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// Attach a dense-vector hook (builder — [`Self::new`] stays lexical).
    /// Server-side wiring lives in `AppState::from_engine` (gated on
    /// `CacheConfig::semantic_enabled`); until then this is also the opt-in
    /// contract for embed search in isolation.
    pub fn with_embedder(mut self, provider: Arc<dyn EmbedProvider>) -> Self {
        self.embedder = Some(provider);
        self
    }

    /// True when a dense-vector hook is attached (wiring probe + ops/debug).
    pub fn has_embedder(&self) -> bool {
        self.embedder.is_some()
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
    ///
    /// With an embedder attached ([`Self::with_embedder`]) the dense-vector
    /// cosine runs first under the same template gate and the same
    /// `similarity_threshold` (vectors are L2-normalized, so 0.90 stays
    /// conservative); any embed failure or miss degrades to the lexical
    /// TF-cosine path, which always runs — an embed hit can only *add* hits,
    /// never remove the ones slice 2 already served.
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
        let template = body_template(body);
        let threshold = self.similarity_threshold;
        let ttl = self.ttl;

        // Embed path: dense cosine over precomputed vectors (fail-open —
        // `None` anywhere drops to the lexical scan below).
        if let Some(provider) = self.embedder.clone() {
            if let Some(query_vec) = provider.embed(&prompt) {
                let mut best: Option<(Key, f32)> = None;
                for (key, timed) in &self.map {
                    if key.protocol != protocol || key.path != path {
                        continue;
                    }
                    if is_expired(timed.inserted_at, ttl) {
                        continue;
                    }
                    let Some(stored_vec) = timed.prompt_vec.as_ref() else {
                        continue;
                    };
                    if timed.template != template {
                        continue;
                    }
                    let sim = cosine_f32(&query_vec, stored_vec);
                    let better = match &best {
                        None => true,
                        Some((_, s)) => sim > *s,
                    };
                    if sim >= threshold && better {
                        best = Some((key.clone(), sim));
                    }
                }
                if let Some((winner, _)) = best {
                    let entry = self.map.get(&winner)?.entry.clone();
                    self.touch(&winner);
                    return Some(entry);
                }
            }
        }

        self.lookup_similar_lexical(protocol, path, &prompt, &template, threshold, ttl)
    }

    /// Slice-2 lexical scan: TF-cosine over the normalized prompt.
    /// Unchanged behavior — the embed path above only adds hits.
    fn lookup_similar_lexical(
        &mut self,
        protocol: &str,
        path: &str,
        prompt: &str,
        template: &[u8],
        threshold: f32,
        ttl: Duration,
    ) -> Option<CachedEntry> {
        let query_tf = term_freq(&normalize(prompt));
        if query_tf.is_empty() {
            return None;
        }
        let query_norm = norm(&query_tf);

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
                let (template, prompt_tf, prompt_norm, prompt_vec) =
                    similarity_material(body, self.embedder.as_ref());
                timed.template = template;
                timed.prompt_tf = prompt_tf;
                timed.prompt_norm = prompt_norm;
                timed.prompt_vec = prompt_vec;
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
                let (template, prompt_tf, prompt_norm, prompt_vec) =
                    similarity_material(body, self.embedder.as_ref());
                self.map.insert(
                    owned,
                    TimedEntry {
                        entry,
                        inserted_at: Instant::now(),
                        template,
                        prompt_tf,
                        prompt_norm,
                        prompt_vec,
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
/// prompt squared-norm, prompt dense vector). Unparseable/prompt-less bodies
/// yield empty TF and no vector and never participate in similarity
/// (exact-only). A failing hook yields `None` vector (lexical-only entry).
fn similarity_material(
    body: &[u8],
    embedder: Option<&Arc<dyn EmbedProvider>>,
) -> (Vec<u8>, HashMap<String, u32>, u32, Option<Vec<f32>>) {
    let Some(prompt) = extract_prompt_text(body) else {
        return (body.to_vec(), HashMap::new(), 0, None);
    };
    let tf = term_freq(&normalize(&prompt));
    let n = norm(&tf);
    let vec = embedder.and_then(|p| p.embed(&prompt));
    (body_template(body), tf, n, vec)
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

/// Dense-vector cosine (embed path). Zero-norm or dim mismatch → 0.0
/// (fail-open: never a hit on garbage, never a panic).
fn cosine_f32(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na * nb)
}

/// Ollama-compatible embedding provider (`POST {base_url}/api/embed`).
///
/// Zero new crates: `reqwest/blocking` is already a vanta-proxy dependency.
/// Every failure (no server, bad status, bad shape) returns `None` so the
/// cache degrades to lexical — embeddings are best-effort, never blocking.
/// Configure via [`Self::from_env`] (`VANTA_EMBED_BASE_URL`,
/// `VANTA_EMBED_MODEL`, default `http://localhost:11434` + `nomic-embed-text`).
///
/// The blocking client lives on a dedicated worker thread (created AND
/// dropped there): dropping a tokio Runtime inside an async context panics,
/// and blocking HTTP on an executor thread would starve the pipeline.
#[derive(Clone)]
pub struct OllamaEmbedProvider {
    base_url: String,
    model: String,
    tx: std::sync::mpsc::Sender<EmbedJob>,
}

impl std::fmt::Debug for OllamaEmbedProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OllamaEmbedProvider")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .finish()
    }
}

/// One embedding request handed to the worker thread.
struct EmbedJob {
    text: String,
    reply: std::sync::mpsc::Sender<Option<Vec<f32>>>,
}

impl OllamaEmbedProvider {
    /// Build for an explicit endpoint + model.
    pub fn new(base_url: String, model: String) -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<EmbedJob>();
        let worker_url = base_url.clone();
        let worker_model = model.clone();
        std::thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new());
            for job in rx {
                let out = embed_once(&client, &worker_url, &worker_model, &job.text);
                let _ = job.reply.send(out);
            }
        });
        Self {
            base_url,
            model,
            tx,
        }
    }

    /// Build from env (`VANTA_EMBED_BASE_URL`, `VANTA_EMBED_MODEL`).
    pub fn from_env() -> Self {
        let base_url = std::env::var("VANTA_EMBED_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model =
            std::env::var("VANTA_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());
        Self::new(base_url, model)
    }
}

/// Single Ollama `/api/embed` round-trip (worker thread only).
fn embed_once(
    client: &reqwest::blocking::Client,
    base_url: &str,
    model: &str,
    text: &str,
) -> Option<Vec<f32>> {
    if text.trim().is_empty() {
        return None;
    }
    #[derive(serde::Serialize)]
    struct EmbedRequest<'a> {
        model: &'a str,
        input: &'a str,
    }
    #[derive(serde::Deserialize)]
    struct EmbedResponse {
        #[serde(default)]
        embeddings: Vec<Vec<f32>>,
    }
    let url = format!("{base_url}/api/embed");
    let response = client
        .post(url)
        .json(&EmbedRequest { model, input: text })
        .send()
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let parsed: EmbedResponse = response.json().ok()?;
    parsed.embeddings.into_iter().next()
}

impl EmbedProvider for OllamaEmbedProvider {
    fn embed(&self, text: &str) -> Option<Vec<f32>> {
        if text.trim().is_empty() {
            return None;
        }
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        self.tx
            .send(EmbedJob {
                text: text.to_string(),
                reply: reply_tx,
            })
            .ok()?;
        // Worker gone or timed-out upstream → None → lexical fallback.
        reply_rx.recv().ok()?
    }
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
        // `Instant` es monotónico desde boot: `now - d` paniquea si uptime < d
        // (el `10_000s` original exigía 2.7h de uptime). `checked_sub` nunca paniquea.
        let old = Instant::now()
            .checked_sub(Duration::from_secs(10))
            .unwrap_or_else(Instant::now);
        // En hosts con uptime <10s `old` puede ser `now`: el sleep garantiza
        // `elapsed >= ttl` sin restas que paniqueen.
        std::thread::sleep(Duration::from_millis(10));
        assert!(is_expired(old, Duration::from_millis(1)));
        assert!(!is_expired(Instant::now(), Duration::from_secs(60)));
        assert!(!is_expired(
            Instant::now()
                .checked_sub(Duration::from_secs(10_000))
                .unwrap_or_else(Instant::now),
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

    // --- PRX-09-embeddings slice 3 RED: EmbedProvider inexistente ---

    use super::EmbedProvider;
    use std::sync::Arc;

    /// Fake semántico test-only: buckets por keywords (determinístico, offline).
    /// Misma familia → coseno 1.0; distinta → 0.0. Prueba el *wiring* del
    /// embed-path, no calidad semántica (esa la da el provider real en prod).
    struct BucketEmbed;

    impl EmbedProvider for BucketEmbed {
        fn embed(&self, text: &str) -> Option<Vec<f32>> {
            let t = text.to_lowercase();
            if t.contains("deploy") || t.contains("release") || t.contains("production") {
                Some(vec![1.0, 0.0, 0.0])
            } else if t.contains("quantum") || t.contains("entanglement") {
                Some(vec![0.0, 1.0, 0.0])
            } else {
                Some(vec![0.0, 0.0, 1.0])
            }
        }
    }

    /// Embedder que siempre falla — el lookup debe degradar al léxico (P4).
    struct FailEmbed;

    impl EmbedProvider for FailEmbed {
        fn embed(&self, _text: &str) -> Option<Vec<f32>> {
            None
        }
    }

    fn embed_cfg() -> CacheConfig {
        CacheConfig {
            enabled: true,
            max_entries: 8,
            ttl_secs: 0,
            semantic_enabled: true,
            similarity_threshold: 0.9,
        }
    }

    fn with_embed() -> ExactCache {
        ExactCache::new(embed_cfg()).with_embedder(Arc::new(BucketEmbed))
    }

    #[test]
    fn embed_hit_paraphrase_lexical_miss() {
        let stored = openai_body("How do I deploy the application to production?");
        // Paráfrasis: solapamiento léxico ~0.45 < 0.9 → el TF-coseno NO hitea.
        let paraphrase = openai_body("What are the steps to release to production?");

        let mut lexical = ExactCache::new(embed_cfg());
        lexical.store("openai", "/p", &stored, entry(b"deployed"));
        assert!(
            lexical
                .lookup_similar("openai", "/p", &paraphrase)
                .is_none(),
            "lexical must MISS the paraphrase (proves the test is semantic)"
        );

        let mut cache = with_embed();
        cache.store("openai", "/p", &stored, entry(b"deployed"));
        let hit = cache
            .lookup_similar("openai", "/p", &paraphrase)
            .expect("embed path must HIT the paraphrase");
        assert_eq!(hit.body, b"deployed");
    }

    #[test]
    fn embed_miss_unrelated() {
        let mut cache = with_embed();
        let stored = openai_body("How do I deploy the application to production?");
        cache.store("openai", "/p", &stored, entry(b"deployed"));
        let other = openai_body("Explain quantum entanglement in detail please");
        assert!(cache.lookup_similar("openai", "/p", &other).is_none());
    }

    #[test]
    fn embed_failure_falls_back_to_lexical() {
        let mut cache = ExactCache::new(embed_cfg()).with_embedder(Arc::new(FailEmbed));
        let stored = openai_body("What is the capital of France?");
        cache.store("openai", "/p", &stored, entry(b"paris"));
        // Embed falla (None) → el léxico igual hitea el near-duplicate.
        let near = openai_body("what is the capital of france");
        let hit = cache
            .lookup_similar("openai", "/p", &near)
            .expect("lexical fallback must hit on embed failure");
        assert_eq!(hit.body, b"paris");
    }

    #[test]
    fn embed_lookup_latency_budget() {
        use std::time::Instant;
        let mut cache = with_embed();
        for i in 0..128 {
            let body = openai_body(&format!("deploy runbook step {i} for production"));
            cache.store("openai", "/p", &body, entry(b"x"));
        }
        let probe = openai_body("How do I deploy the application to production?");
        let start = Instant::now();
        for _ in 0..50 {
            let _ = cache.lookup_similar("openai", "/p", &probe);
        }
        let elapsed = start.elapsed();
        eprintln!("embed lookup_similar 50x/128 entries: {elapsed:?}");
        assert!(
            elapsed.as_secs() < 5,
            "embed scan must stay far inside budget, got {elapsed:?}"
        );
    }
}
