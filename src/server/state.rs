//! Shared types and DTOs for the HTTP server (REVIEW-10 split).
//!
//! Holds request/response shapes, the [`ServerState`] struct, the
//! [`ConversationTrigger`] hook trait, [`AuthState`] / [`AuthIdentity`] /
//! [`AuthRateLimiter`] (auth-side state) and the request-id extractor.
//! Everything that is *data* lives here; everything that *runs* (handlers,
//! middleware, telemetry, TLS, bootstrap) lives in [`super::routing`].

use crate::audit::{AuditEvent, AuditLogger};
use crate::circuit_breaker::CircuitBreaker;
use crate::config::RbacConfig;
use crate::connection_pool::ConnectionPool;
use crate::entity::EntityStore;
use crate::node::FieldValue;
use crate::rbac::Rbac;
use crate::sdk::VantaEmbedded;
use crate::storage::StorageEngine;
use axum::http::HeaderMap;
use lru::LruCache;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;

/// Timeout for interactive API routes: a stuck handler must not hold the
/// connection indefinitely (DoS protection). 30s comfortably covers normal
/// query/search/list operations while bounding a hung request.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Generous timeout for long-running bulk/maintenance routes (import, export,
/// rebuild-index) that legitimately take longer than interactive requests.
/// Still capped so a truly wedged operation can't hold a worker forever.
pub const LONG_REQUEST_TIMEOUT: Duration = Duration::from_secs(600);

/// JSON body for a query endpoint request.
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    /// The VantaQL query string to execute.
    pub query: String,
}

/// Response envelope for the query endpoint.
#[derive(Serialize, Deserialize)]
pub struct QueryResponse {
    /// Whether the request succeeded.
    pub success: bool,
    /// Human-readable result message.
    pub data: String,
    /// Single node ID returned by write or stale-context results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<u128>,
    /// Collection of nodes returned by read results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<NodeDTO>>,
}

/// Data-transfer object for a UnifiedNode returned over HTTP.
#[derive(Serialize, Deserialize)]
pub struct NodeDTO {
    /// Unique node identifier.
    pub id: u128,
    /// Semantic cluster the node belongs to.
    pub semantic_cluster: u32,
    /// Relational key-value payload.
    pub relational: std::collections::BTreeMap<String, FieldValue>,
    /// Access / hit count tracked by the storage engine.
    pub hits: u32,
    /// Internal confidence score for staleness detection.
    pub confidence_score: f32,
}

impl From<&crate::node::UnifiedNode> for NodeDTO {
    fn from(n: &crate::node::UnifiedNode) -> Self {
        Self {
            id: n.id,
            semantic_cluster: n.semantic_cluster,
            relational: n.relational.clone(),
            hits: n.hits,
            confidence_score: n.confidence_score,
        }
    }
}

/// Best-effort post-save hook for `POST /conversation/add` (MEM-55).
///
/// Fired once per successful save, right before the HTTP response is built.
/// The core cannot depend on the memory pipeline (Cargo forbids the cycle:
/// `vanta-memory → vantadb`), so hosts wire their own implementation into
/// [`ServerState`]. Errors are logged and swallowed by the route handler —
/// extraction failures must never fail the HTTP response (P4).
pub trait ConversationTrigger: Send + Sync {
    /// Called with the persisted thread id and the raw message.
    fn trigger(
        &self,
        thread_id: u128,
        role: &str,
        content: &str,
    ) -> std::result::Result<(), String>;
}

/// Shared application state injected into every route handler.
pub struct ServerState {
    /// The underlying storage engine.
    pub storage: Arc<StorageEngine>,
    /// Embedded SDK handle sharing the storage engine — source of operations
    /// for the `/api/v2` record endpoints. Shared (not per-request) so the
    /// audit logger has a single file handle with its own mutex; per-request
    /// handles would interleave appends to the audit JSONL.
    pub db: VantaEmbedded,
    /// Circuit breaker for fast-failing when the backend is failing.
    pub circuit_breaker: Arc<CircuitBreaker>,
    /// Connection pool bounding concurrent query execution.
    pub pool: Arc<ConnectionPool>,
    /// Optional primary bearer token for API authentication.
    pub api_key: Option<Arc<str>>,
    /// Alternative API key for zero-downtime rotation (SRV-04).
    pub alt_api_key: Option<Arc<str>>,
    /// RBAC token-to-role mapping configuration.
    pub rbac_config: RbacConfig,
    /// Reverse-proxy IPs whose `X-Forwarded-For` header is honored for client
    /// IP resolution. Empty = ignore the header (ConnectInfo is authoritative).
    pub trusted_proxies: Vec<std::net::IpAddr>,
    /// Optional post-save hook for `POST /conversation/add` (MEM-55). `None`
    /// keeps the route purely a thread store (pre-MEM-55 behavior).
    pub conversation_trigger: Option<Arc<dyn ConversationTrigger>>,
}

/// Per-IP rate limiter for authentication failures.
pub struct AuthRateLimiter {
    /// Per-IP map of (failure count, first-failure instant). LRU evicts oldest.
    failures: Mutex<LruCache<String, (u32, Instant)>>,
    /// Maximum failed attempts before rate-limiting kicks in.
    max_attempts: u32,
    /// Time window in seconds.
    window_secs: u64,
}

impl AuthRateLimiter {
    /// Create a new rate limiter with the given attempt cap and window.
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        Self {
            failures: Mutex::new(LruCache::new(std::num::NonZero::new(1000).unwrap())),
            max_attempts,
            window_secs,
        }
    }

    /// Returns `true` if the given IP has exceeded the allowed failure rate.
    pub fn is_rate_limited(&self, ip: &str) -> bool {
        let mut failures = self.failures.lock();
        let now = Instant::now();
        if let Some((count, first)) = failures.get(ip) {
            if now.duration_since(*first).as_secs() > self.window_secs {
                failures.pop(ip);
                return false;
            }
            *count >= self.max_attempts
        } else {
            false
        }
    }

    /// Record an authentication failure for the given IP.
    pub fn record_failure(&self, ip: &str) {
        let mut failures = self.failures.lock();
        let now = Instant::now();
        let (count, first) = failures.get(ip).map(|&(c, f)| (c, f)).unwrap_or((0, now));
        if now.duration_since(first).as_secs() > self.window_secs {
            failures.put(ip.to_string(), (1, now));
        } else {
            failures.put(ip.to_string(), (count + 1, first));
        }
    }

    /// Clear the failure count for the given IP.
    pub fn reset(&self, ip: &str) {
        self.failures.lock().pop(ip);
    }
}

/// Authentication and authorization state shared via middleware extensions.
#[derive(Clone)]
pub struct AuthState {
    /// Optional primary bearer token for API key validation.
    pub api_key: Option<Arc<str>>,
    /// Alternative API key for zero-downtime rotation (SRV-04).
    /// When set, both `api_key` and `alt_api_key` are accepted.
    pub alt_api_key: Option<Arc<str>>,
    pub(crate) token_role_map: HashMap<String, String>,
    pub(crate) rbac: Arc<Rbac>,
    pub(crate) rate_limiter: Arc<AuthRateLimiter>,
    /// Reverse-proxy IPs whose `X-Forwarded-For` header is honored for client IP
    /// resolution. Empty = the header is ignored.
    pub(crate) trusted_proxies: Vec<std::net::IpAddr>,
    /// Storage engine for L3 user-key → user entity resolution (entity_*).
    pub(crate) storage: Option<Arc<StorageEngine>>,
    /// Audit logger for auth events (`auth_l1`/`auth_l2`/`auth_l3`).
    pub(crate) audit: Option<Arc<AuditLogger>>,
}

impl AuthState {
    pub(crate) fn new(
        api_key: Option<String>,
        alt_api_key: Option<String>,
        rbac_config: RbacConfig,
        rbac: Arc<Rbac>,
        trusted_proxies: &[std::net::IpAddr],
        storage: Option<Arc<StorageEngine>>,
        audit: Option<Arc<AuditLogger>>,
    ) -> Self {
        Self {
            api_key: api_key.map(|k| Arc::from(k.as_str())),
            alt_api_key: alt_api_key.map(|k| Arc::from(k.as_str())),
            token_role_map: rbac_config.token_role_map,
            rbac,
            rate_limiter: Arc::new(AuthRateLimiter::new(5, 60)),
            trusted_proxies: trusted_proxies.to_vec(),
            storage,
            audit,
        }
    }
}

/// Identity resolved by the 3-layer auth middleware (MEM-05).
///
/// Inserted into request extensions for downstream handlers so they can
/// authorize against the resolved principal (e.g. with `PermissionChecker`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthIdentity {
    /// Bare valid Bearer token — transport-level identity (L1 only).
    Transport,
    /// L2: valid Bearer + `x-vanta-service-id` — service credential
    /// (admin-level, TDAM `Bearer + n` semantics).
    Service { service_id: String },
    /// L3: valid Bearer + `x-vanta-user-key` resolved against the `user`
    /// entity collection — the caller is a known user.
    User {
        user_id: String,
        is_system_admin: bool,
    },
}

/// Namespace holding auth entities (`user` collection) for L3 resolution.
/// ponytail: single fixed namespace; make configurable when multi-namespace
/// servers appear.
pub const AUTH_ENTITY_NS: &str = "default";

/// L3 header carrying the caller's user key (TDAM `x-tdai-user-key` port).
pub(crate) const USER_KEY_HEADER: &str = "x-vanta-user-key";

/// L2 header carrying the service/instance id (TDAM `x-tdai-service-id` port).
pub(crate) const SERVICE_ID_HEADER: &str = "x-vanta-service-id";

/// Request tracing headers, first match wins (SRV-02).
pub(crate) const REQUEST_ID_HEADERS: [&str; 3] = ["x-request-id", "x-tracing-id", "traceparent"];
/// Max length of a captured request id; longer values are truncated (SRV-02).
pub(crate) const REQUEST_ID_MAX_LEN: usize = 256;

/// Request tracing id captured by the metrics middleware and exposed to
/// handlers through axum extensions (SRV-02). Extractor-safe: absent headers
/// yield `RequestId(None)`.
#[derive(Clone, Debug, Default)]
pub struct RequestId(pub Option<String>);

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for RequestId {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<RequestId>()
            .cloned()
            .unwrap_or_default())
    }
}

/// Record an auth audit event, never failing the request on write errors.
pub(crate) fn audit_auth(auth: &AuthState, event: AuditEvent) {
    if let Some(logger) = &auth.audit {
        if let Err(e) = logger.record(&event) {
            tracing::warn!(op = %event.op, error = %e, "auth audit record failed");
        }
    }
}

/// Resolve a user-key to `(user_id, is_system_admin)` by scanning the `user`
/// entity collection (MEM-03) and comparing `fields.user_key` in constant time.
///
/// ponytail: linear scan over all users; add a user_key→user_id index when
/// the user collection grows past ~1k entries.
pub(crate) fn resolve_user_key(
    store: &EntityStore<'_>,
    namespace: &str,
    user_key: &str,
) -> crate::error::Result<Option<(String, bool)>> {
    let page = store.entity_list(namespace, "user", 10_000, 0)?;
    let key_bytes = user_key.as_bytes();
    for entity in page.items {
        let Some(FieldValue::String(candidate)) = entity.fields.get("user_key") else {
            continue;
        };
        if candidate.as_bytes().ct_eq(key_bytes).into() {
            let is_system_admin = matches!(
                entity.fields.get("user_type"),
                Some(FieldValue::String(t)) if t == "system_admin"
            );
            return Ok(Some((entity.entity_id, is_system_admin)));
        }
    }
    Ok(None)
}

/// Simple URL decode for query params (no external dep).
pub(crate) fn simple_url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        out.push(byte as char);
                        continue;
                    }
                }
                out.push('%');
                out.push_str(&hex);
            }
            '+' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

/// Extract namespace from request for namespace-scoped RBAC (SRV-05).
/// Checks path params (/{ns}/{key}) and query params (?namespace=).
pub(crate) fn extract_namespace(path: &str, query: Option<&str>) -> Option<String> {
    // 1. Path params: /api/v2/records/{ns}/{key}, /api/v2/records/{ns}/{key}/versions
    // Pattern: /api/v2/records/{ns}/ or /api/v2/records/{ns}/
    if let Some(rest) = path.strip_prefix("/api/v2/records/") {
        if let Some(ns_end) = rest.find('/') {
            return Some(rest[..ns_end].to_string());
        } else if !rest.is_empty() && rest != "batch" {
            // /api/v2/records/{ns} (no trailing slash)
            return Some(rest.to_string());
        }
    }
    // 2. Query param: ?namespace= or ?ns=
    if let Some(query) = query {
        for pair in query.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                if k == "namespace" || k == "ns" {
                    return Some(simple_url_decode(v));
                }
            }
        }
    }
    None
}

/// First non-empty match of the request tracing headers, truncated to
/// [`REQUEST_ID_MAX_LEN`] chars (SRV-02).
pub(crate) fn extract_request_id(headers: &HeaderMap) -> Option<String> {
    for name in REQUEST_ID_HEADERS {
        if let Some(value) = headers.get(name).and_then(|v| v.to_str().ok()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.chars().take(REQUEST_ID_MAX_LEN).collect());
            }
        }
    }
    None
}

/// Resolve a user-key to `(user_id, is_system_admin)` already defined above
/// is `resolve_user_key`. `client_ip` and `resolve_identity` are also
/// exposed here so `middleware` can import them from `state` (REVIEW-10
/// split expectation).
/// Resolve the real client IP used for rate limiting and logging.
///
/// `X-Forwarded-For` is only honored when the request's peer is one of
/// `trusted_proxies` (i.e. it actually arrived via a configured reverse proxy
/// that sets the header). Otherwise the direct TCP socket address
/// ([`ConnectInfo`]) is returned — so a client cannot spoof its recorded IP by
/// setting `X-Forwarded-For` itself.
pub fn client_ip(req: &axum::extract::Request, trusted_proxies: &[std::net::IpAddr]) -> String {
    let peer = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0);

    if let Some(peer) = peer {
        if trusted_proxies.contains(&peer.ip()) {
            if let Some(forwarded) = req.headers().get("x-forwarded-for") {
                if let Ok(ip_str) = forwarded.to_str() {
                    for part in ip_str.split(',') {
                        let trimmed = part.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Ok(ip) = trimmed.parse::<std::net::IpAddr>() {
                            return ip.to_string();
                        }
                    }
                }
            }
        }
        return peer.ip().to_string();
    }

    "unknown".to_string()
}

/// Resolve the auth identity from headers: L3 (user-key) wins over L2
/// (service-id); neither present → bare transport identity.
///
/// Any resolution failure yields 401 (fail closed — internal state is never
/// leaked to the caller).
pub(crate) fn resolve_identity(
    req: &axum::extract::Request,
    auth: &AuthState,
) -> std::result::Result<AuthIdentity, (axum::http::StatusCode, &'static str)> {
    let user_key = req
        .headers()
        .get(USER_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if let Some(user_key) = user_key {
        let Some(storage) = &auth.storage else {
            return Err((axum::http::StatusCode::UNAUTHORIZED, "invalid_user_key"));
        };
        let store = EntityStore::new(storage.as_ref());
        return match resolve_user_key(&store, AUTH_ENTITY_NS, user_key) {
            Ok(Some((user_id, is_system_admin))) => Ok(AuthIdentity::User {
                user_id,
                is_system_admin,
            }),
            Ok(None) | Err(_) => Err((axum::http::StatusCode::UNAUTHORIZED, "invalid_user_key")),
        };
    }

    let service_id = req
        .headers()
        .get(SERVICE_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if let Some(service_id) = service_id {
        return Ok(AuthIdentity::Service {
            service_id: service_id.to_string(),
        });
    }

    Ok(AuthIdentity::Transport)
}
