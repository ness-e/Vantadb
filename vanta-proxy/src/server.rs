//! Router assembly (TDAM parity: server.ts:307,312 — primary agent-prefixed routes).
//!
//! Every wire handler runs the MEM-26 pipeline BEFORE forwarding:
//! auth (D34) → session (D26) → inject (D29) → forward.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, Method, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use vantadb::sdk::Embedded;
use vantadb::storage::StorageEngine;

use crate::auth::AuthDb;
use crate::cache::{self, CachedEntry, ExactCache};
use crate::capture;
use crate::config::{ProxyConfig, UpstreamConfig};
use crate::context::{self, ContextOptimizer};
use crate::cost::{self, BudgetDecision, CostTracker, VirtualKey};
use crate::forward::Forwarder;
use crate::guardrails::GuardrailDecision;
use crate::handlers;
use crate::inject::{self, Protocol};
use crate::mem_command;
use crate::memory_tools;
use crate::rate_limit::{self, RateDecision, RateLimiter, UpstreamHealth};
use crate::redact::{self, Redactor};
use crate::report::{model_from_body, now_ms_u64, Reporter, TurnReport, TurnTimer};
use crate::routing::{tier_of, ResolvedRoute};
use crate::session::claude_code::CcRequestKind;
use crate::session::{parse_stage, session_key_from_headers, SessionStore};
use crate::sse_intercept;
use crate::translate::{self, TranslateSource};
use crate::writeback::WriteBack;

/// Hard iteration cap of the agentic memory-tool loop (D48): at most 3
/// tool-execution rounds per client request (so at most 4 upstream forwards).
const MAX_MEMORY_TOOL_ITERATIONS: usize = 3;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ProxyConfig>,
    pub forwarder: Arc<Forwarder>,
    /// Local RBAC entity store handle (D25/D34).
    pub auth: Arc<AuthDb>,
    /// Embedded memory handle over the SAME storage (persona/scene injection).
    pub memory: Arc<Embedded>,
    /// Local session state machine store (D26).
    pub sessions: Arc<SessionStore>,
    /// In-process sliding-window rate limiter (D24/D35).
    pub limiter: Arc<RateLimiter>,
    /// Upstream health tracker (PRX-01 S4): flips `limiter` degraded on
    /// 3×429/5xx, recovers on 5×éxito.
    pub upstream_health: Arc<UpstreamHealth>,
    /// L0 write-back coordinator (MEM-27).
    pub writeback: Arc<WriteBack>,
    /// Per-turn structured reporting (MEM-27).
    pub reporter: Arc<Reporter>,
    /// Exact + semantic response cache (PRX-09 slice 1+2, opt-in).
    pub cache: Arc<std::sync::Mutex<ExactCache>>,
    /// Cost ledger + virtual-key budgets (PRX-03). In-memory, fail-open.
    pub cost: Arc<CostTracker>,
    /// Egress PII/secret redaction (PRX-07). Disabled by default; compiled
    /// once at startup so per-request work is only the scan.
    pub redactor: Arc<Redactor>,
    /// Context optimization in transit (PRX-13). Disabled by default;
    /// stateless over config, built once at startup.
    pub optimizer: Arc<ContextOptimizer>,
}

/// PRX-01: true when an Anthropic request is a standalone CC sidequery
/// (TITLE / verify_api_key — no marker, empty tools, thinking disabled).
/// Sidequeries bypass session resolution, injection AND turn capture:
/// they are not conversation and must not pollute memory. Malformed or
/// non-Anthropic bodies never bypass (fail onto the full pipeline).
fn is_cc_sidequery(protocol: Protocol, body: &[u8]) -> bool {
    if !matches!(protocol, Protocol::Anthropic) {
        return false;
    }
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return false;
    };
    crate::session::claude_code::classify_cc_request(&value) == CcRequestKind::Sidequery
}

impl AppState {
    /// Build state from a loaded configuration, opening the local store at
    /// `config.auth.db_path`.
    ///
    /// # Errors
    /// - [`crate::error::ProxyError::Config`] if the HTTP client cannot be built.
    /// - [`crate::error::ProxyError::Storage`] if the local database cannot open.
    pub fn new(config: ProxyConfig) -> Result<Self, crate::error::ProxyError> {
        let db = AuthDb::open(&config.auth.db_path)?;
        Self::from_engine(config, db.engine())
    }

    /// Build state over an already-open storage engine (tests / shared handles).
    ///
    /// # Errors
    /// Returns [`crate::error::ProxyError::Config`] if the HTTP client cannot be built.
    pub fn from_engine(
        config: ProxyConfig,
        engine: Arc<StorageEngine>,
    ) -> Result<Self, crate::error::ProxyError> {
        // PRX-08 S2: fail fast on self-forwarding loops (default upstream
        // 127.0.0.1:8096 == default listen port) instead of recursing to timeout.
        if config.upstream.points_at_self(config.server.port) {
            return Err(crate::error::ProxyError::Config(format!(
                "upstream {} points at this proxy itself (listen port {})",
                config.upstream.url, config.server.port
            )));
        }
        let forwarder = Forwarder::new(&config.upstream)?;
        let persist_path = (!config.writeback.persist_path.is_empty())
            .then(|| std::path::PathBuf::from(&config.writeback.persist_path));
        let reporter = Reporter::new();
        // MEM-56: optional OTLP span export — None (no hook) unless an
        // endpoint is configured in `[report]`.
        if let Some(hook) = crate::langfuse::langfuse_hook(&config.report) {
            reporter.add_hook(hook);
        }
        let cache = ExactCache::new(config.cache.clone());
        // PRX-09-wiring: attach the Ollama embed hook only when semantic
        // search is opted in (`semantic_enabled`). Default-off keeps the wire
        // byte-identical: without the hook the slice-2 lexical path runs
        // alone. Offline-safe: `from_env` only reads env + builds a client —
        // no I/O until the first `embed`, and every embed failure degrades
        // to lexical (fail-open, hits only ever added).
        let cache = if config.cache.semantic_enabled {
            cache.with_embedder(Arc::new(cache::OllamaEmbedProvider::from_env()))
        } else {
            cache
        };
        // PRX-03: ledger over the configured price table + default budget.
        let cost = CostTracker::new(
            config.cost.prices.clone(),
            config.cost.default_budget_usd,
            config.cost.enforce,
        );
        // PRX-07: compile custom patterns once — invalid patterns fail
        // closed here (proxy refuses to start) instead of per-request.
        let redactor = Redactor::new(&config.redact)?;
        // PRX-13: stateless over config — infallible build.
        let optimizer = ContextOptimizer::new(&config.context);
        Ok(Self {
            limiter: RateLimiter::new(config.server.rate_limit_per_minute).into(),
            upstream_health: UpstreamHealth::new().into(),
            writeback: WriteBack::new(persist_path).into(),
            reporter: reporter.into(),
            cache: Arc::new(std::sync::Mutex::new(cache)),
            cost: cost.into(),
            redactor: redactor.into(),
            optimizer: optimizer.into(),
            config: Arc::new(config),
            forwarder: Arc::new(forwarder),
            auth: AuthDb::new(engine.clone()).into(),
            memory: Embedded::from_engine(engine).into(),
            sessions: SessionStore::new().into(),
        })
    }

    /// PRX-06: resuelve el routing por tier para una request.
    ///
    /// Solo Anthropic clasifica (el CC classifier no vale en otros
    /// protocolos); el resto es passthrough. Corre ANTES del pipeline pero
    /// solo lee la identidad para `bypass_keys` — D34 sigue siendo el gate
    /// en `process_inner`, esto nunca autoriza nada.
    fn resolve_route(
        &self,
        protocol: Protocol,
        headers: &HeaderMap,
        body: &Bytes,
    ) -> ResolvedRoute {
        let upstreams = self.config.upstreams_resolved();
        let Some(tier) = tier_of(protocol, body) else {
            return ResolvedRoute::passthrough(body.clone(), upstreams);
        };
        let user_key = self
            .auth
            .authenticate(headers)
            .map(|id| id.user_id)
            .unwrap_or_default();
        let Some(d) = self
            .config
            .routing
            .resolve(tier, &user_key, upstreams.len())
        else {
            return ResolvedRoute::passthrough(body.clone(), upstreams);
        };
        if d.shadow {
            tracing::info!(tier = ?d.tier, model_override = ?d.model_override, upstream_index = d.upstream_index, "routing shadow — decision logged, wire untouched");
            return ResolvedRoute::passthrough(body.clone(), upstreams);
        }
        let out_body = match &d.model_override {
            Some(m) => Bytes::from(crate::routing::rewrite_model(body, m)),
            None => body.clone(),
        };
        tracing::info!(tier = ?d.tier, model_override = ?d.model_override, upstream_index = d.upstream_index, "routing enforce");
        ResolvedRoute {
            body: out_body,
            upstreams: ResolvedRoute::prioritize(upstreams, d.upstream_index),
        }
    }

    /// The MEM-26/27 pipeline:
    /// auth (D34) → rate-limit (D24) → session (D26) → mem-command (D33)
    /// → inject (D29) → forward, wrapped in per-turn reporting.
    ///
    /// Any pipeline failure returns a typed error response; only a fully
    /// authorized request ever reaches the upstream.
    pub(crate) async fn process(
        &self,
        protocol: Protocol,
        wire_path: &str,
        headers: &HeaderMap,
        body: Bytes,
        space_id: &str,
    ) -> Response<Body> {
        let timer = TurnTimer::start();
        // PRX-06: tier routing BEFORE model extraction so reports/rate-limit
        // keys ven el modelo ruteado (shadow/enforce loguean la decisión).
        let route = self.resolve_route(protocol, headers, &body);
        let body = route.body.clone();
        let model = model_from_body(&body);
        let response = self
            .process_inner(
                protocol,
                wire_path,
                headers,
                body.clone(),
                space_id,
                &model,
                &route.upstreams,
            )
            .await;
        // D47/MEM-50: completed request → L0 turn capture. Fire-and-forget
        // AFTER the response is built — a slow or failing memory write can
        // never delay or break the forward. Sidequeries bypass capture:
        // they are not conversation turns (PRX-01).
        if response.status().is_success() && !is_cc_sidequery(protocol, &body) {
            self.capture_turn(protocol, headers, space_id, &model, &body);
        }
        // PRX-03: request-side cost accounting. Best-effort and fail-open:
        // an unresolvable identity simply records nothing — the wire
        // already ran. Output side stays 0 until a buffered body exists
        // (`record_response_usage`, wired by a future SSE drain).
        let mut virtual_key = String::new();
        let mut session = String::new();
        let mut input_tokens = 0u64;
        let mut turn_cost = 0.0;
        if self.config.cost.enabled {
            if let Ok(identity) = self.auth.authenticate(headers) {
                virtual_key = identity.user_id;
                session = session_key_from_headers(headers).unwrap_or_default();
                let usage = cost::Usage {
                    input_tokens: cost::tokens_from_request_body(body.as_ref()),
                    output_tokens: 0,
                };
                input_tokens = usage.input_tokens;
                turn_cost = self.cost.record(&virtual_key, &session, &model, &usage);
            }
        }
        self.reporter.emit(&TurnReport {
            timestamp_ms: now_ms_u64(),
            space_id: space_id.to_string(),
            protocol: protocol_name(protocol).to_string(),
            model,
            status: response.status().as_u16(),
            duration_ms: timer.elapsed_ms(),
            virtual_key,
            session,
            input_tokens,
            output_tokens: 0,
            cost_usd: turn_cost,
        });
        response
    }

    async fn process_inner(
        &self,
        protocol: Protocol,
        wire_path: &str,
        headers: &HeaderMap,
        body: Bytes,
        space_id: &str,
        model: &str,
        upstreams: &[UpstreamConfig],
    ) -> Response<Body> {
        // 1) D34: every request needs a valid user key — no open mode. Auth
        // runs BEFORE the limiter so unauthenticated traffic never burns
        // another identity's quota slots (invalid keys always get 401).
        if let Err(e) = self.auth.authenticate(headers) {
            tracing::debug!(error = %e, "request rejected by auth");
            return e.into_response();
        }

        // 1b) PRX-03: virtual-key budget gate (log-first default). Runs
        // right after D34 so only authenticated identities are checked —
        // it can never bypass auth, and without enforce it only warns.
        if self.config.cost.enabled {
            if let Ok(identity) = self.auth.authenticate(headers) {
                let key = VirtualKey {
                    id: identity.user_id,
                    budget_usd: self.config.cost.default_budget_usd,
                    enforce: self.config.cost.enforce,
                };
                if let BudgetDecision::Limited {
                    spent_usd,
                    budget_usd,
                } = self.cost.check_budget(&key)
                {
                    tracing::warn!(
                        key = %key.id, spent_usd, budget_usd,
                        "virtual key over budget — rejecting 429"
                    );
                    return crate::error::ProxyError::BudgetExceeded {
                        key: key.id,
                        spent_usd,
                        budget_usd,
                    }
                    .into_response();
                }
            }
        }

        // 1c) PRX-10: per-key model allowlist (opt-in policy). Runs after
        // 1b so `user_id` is already authenticated — the allowlist never
        // authorizes, it only denies models outside the key's policy.
        // Disabled default / unknown key / empty entry all allow (fail-open
        // additive). Extension point: a pluggable moderation provider
        // would hook this same gate.
        if let Ok(identity) = self.auth.authenticate(headers) {
            if let GuardrailDecision::Denied {
                key,
                model: denied_model,
            } = self.config.guardrails.check(&identity.user_id, model)
            {
                tracing::warn!(
                    key = %key, model = %denied_model,
                    "model not allowed for virtual key — rejecting 403"
                );
                return crate::error::ProxyError::GuardrailBlocked {
                    key,
                    model: denied_model,
                }
                .into_response();
            }
        }

        // 2) D24/D35: sliding-window limit keyed by spaceId×model.
        match self.limiter.check(space_id, model) {
            RateDecision::Allowed { .. } => {}
            limited @ RateDecision::Limited { .. } => {
                tracing::warn!(space_id = %space_id, model = %model, "rate limit exceeded");
                return rate_limit::limited_response(
                    space_id,
                    model,
                    self.config.server.rate_limit_per_minute,
                    limited,
                );
            }
        }

        // 3) D33: in-band `mem:` command interception (opt-in). Runs BEFORE
        // session resolution so commands work on sessions without context.
        if self.config.mem_command.enabled {
            if let Some(cmd) = mem_command::parse(&body) {
                tracing::info!(command = %cmd.command, "mem: command intercepted");
                // Commands run pre-session (work without context): scope to the
                // header key when present, else to all turns (PRX-01).
                let session_key = session_key_from_headers(headers).unwrap_or_default();
                return mem_command::respond(&mem_command::execute(
                    &self.memory,
                    &session_key,
                    &cmd,
                ));
            }
        }

        // 3b) PRX-01: CC sidequeries are standalone (TITLE/verify_api_key)
        // — forward verbatim, skipping session resolution + injection.
        // Auth, rate-limit and mem-command above still apply.
        if is_cc_sidequery(protocol, &body) {
            tracing::debug!("cc sidequery — verbatim forward");
            return self.forward_raw(wire_path, headers, body, upstreams).await;
        }

        // 4) D26: resolve/create the session and refresh its TTL clock.
        let Some(key) = session_key_from_headers(headers) else {
            // No session context → clean init: forward verbatim (D29 applies
            // to sessions only).
            return self.forward_raw(wire_path, headers, body, upstreams).await;
        };
        self.sessions.ensure(&key);

        // 5) D29: system-prompt injection + L0/L1 tools. Non-JSON bodies
        // pass through untouched.
        let memory_block = inject::build_memory_block(&self.memory, &key);
        let body = match inject::inject_into(&body, protocol, &memory_block) {
            Ok(Some(modified)) => Bytes::from(modified),
            Ok(None) => body,
            Err(e) => return e.into_response(),
        };

        // 5a) PRX-07: egress PII/secret redaction over the post-injection
        // bytes (what actually leaves toward the upstream). Disabled by
        // default → identity. Block → 422 kinds-only; Mask → forward
        // scrubbed bytes; Log → warn + forward verbatim. Verbatim paths
        // above (sidequery, no-session) intentionally bypass.
        let body = match self.redactor.apply(&body) {
            redact::ApplyOutcome::Pass(scrubbed) => Bytes::from(scrubbed),
            redact::ApplyOutcome::Block(kinds) => {
                return crate::error::ProxyError::RedactionBlocked { kinds }.into_response();
            }
        };

        // PRX-13: context optimization in transit over the post-redaction
        // bytes (what gets cached and forwarded). Disabled by default →
        // identity. Never blocks: over-budget histories are trimmed
        // (system + last turns survive); tool/image bodies pass through in
        // Performance/Balanced (pre-mortem). Verbatim paths above
        // (sidequery, no-session) intentionally bypass.
        let user_key = self
            .auth
            .authenticate(headers)
            .map(|id| id.user_id)
            .unwrap_or_default();
        let context::ApplyOutcome::Pass(trimmed) = self.optimizer.apply(&body, protocol, &user_key);
        let body = Bytes::from(trimmed);

        // 5b) PRX-09 slice 1+2: exact cache over the POST-INJECTION bytes
        // (the key carries the PRX-04 prefix, so memory changes invalidate
        // implicitly). Runs AFTER auth/session — a hit never bypasses D34.
        // Slice 2: on exact miss, a similarity hit over the same template
        // replays the entry (opt-in via `semantic_enabled`).
        // 5d) PRX-11 slice 3: opt-in Anthropic→OpenAI translation runs here
        // (AFTER auth/gates, BEFORE cache+forward) so translated bodies share
        // cache entries with native OpenAI requests. Disabled default →
        // byte-identical wire (tested).
        let (eff_protocol, eff_wire_path, body, translated) =
            match translate_request(&self.config.translate, protocol, wire_path, &body) {
                Some((p, w, b)) => (p, w, b, true),
                None => (protocol, wire_path.to_string(), body, false),
            };
        let cacheable = self.cache_enabled() && cache::is_cacheable_request(body.as_ref());
        if cacheable {
            if let Some(entry) = self.cache_lookup(eff_protocol, &eff_wire_path, &body) {
                return cached_response(&entry);
            }
            if let Some(entry) = self.cache_lookup_similar(eff_protocol, &eff_wire_path, &body) {
                return cached_response(&entry);
            }
        }

        let response = self
            .forward_with_tool_loop(
                eff_protocol,
                &eff_wire_path,
                headers,
                body.clone(),
                space_id,
                &key,
                model,
                upstreams,
            )
            .await;
        let response = map_translated_response(response, translated).await;

        // 5c) Store small JSON 2xx for the next identical request.
        // SSE/chunked/unknown-length responses bypass — never buffered.
        if cacheable {
            return self
                .maybe_store(eff_protocol, &eff_wire_path, body, response)
                .await;
        }
        response
    }

    /// True when the exact cache is enabled. Sync-only lock section —
    /// the guard never crosses `.await` (concurrency-async R-2).
    fn cache_enabled(&self) -> bool {
        self.cache.lock().is_ok_and(|guard| guard.enabled())
    }

    /// Sync-only lookup: lock, clone the entry, drop the guard.
    fn cache_lookup(&self, protocol: Protocol, path: &str, body: &Bytes) -> Option<CachedEntry> {
        let mut guard = self.cache.lock().ok()?;
        guard.lookup(protocol_name(protocol), path, body.as_ref())
    }

    /// Sync-only similarity lookup (slice 2): same session-path template +
    /// cosine over the prompt. Unparseable bodies fail open (None → upstream).
    fn cache_lookup_similar(
        &self,
        protocol: Protocol,
        path: &str,
        body: &Bytes,
    ) -> Option<CachedEntry> {
        let mut guard = self.cache.lock().ok()?;
        guard.lookup_similar(protocol_name(protocol), path, body.as_ref())
    }

    /// Buffer a small JSON 2xx and store it; anything else flows through
    /// untouched. The body is only consumed after the cacheability gate
    /// passed (known content-length within budget).
    async fn maybe_store(
        &self,
        protocol: Protocol,
        path: &str,
        request: Bytes,
        response: Response<Body>,
    ) -> Response<Body> {
        let (parts, body) = response.into_parts();
        let status = parts.status.as_u16();
        let content_type = parts
            .headers
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        // NOTE: `forward.rs` strips `content-length` for streaming safety, so
        // length is enforced here by the collection cap: JSON chat responses
        // are small and complete; anything past the cap is a typed 502
        // (documented ceiling — chat completions never legitimately exceed it).
        if !cache::is_cacheable_response(status, Some(&content_type)) {
            return Response::from_parts(parts, body);
        }
        let limit = usize::try_from(cache::MAX_CACHEABLE_BODY_BYTES + 1).unwrap_or(usize::MAX);
        match axum::body::to_bytes(body, limit).await {
            Ok(bytes) => {
                let entry = CachedEntry {
                    status,
                    content_type,
                    body: bytes.to_vec(),
                };
                // Sync-only store section — no `.await` under the guard.
                if let Ok(mut guard) = self.cache.lock() {
                    guard.store(protocol_name(protocol), path, request.as_ref(), entry);
                }
                Response::from_parts(parts, Body::from(bytes))
            }
            // Upstream lied about content-length (declared small, sent big):
            // the consumed body is unrecoverable — typed 502, never a truncation.
            Err(e) => {
                crate::error::ProxyError::Forward(format!("cache buffer: {e}")).into_response()
            }
        }
    }

    /// D47: single L0 write path — track the conversation turn through
    /// [`WriteBack::track`]. Requires a session key (no session → no capture,
    /// matching the verbatim-forward path) and a non-empty user text.
    fn capture_turn(
        &self,
        protocol: Protocol,
        headers: &HeaderMap,
        space_id: &str,
        model: &str,
        body: &[u8],
    ) {
        let Some(session) = session_key_from_headers(headers) else {
            return;
        };
        let Some(text) = capture::last_user_text(body).filter(|t| !t.is_empty()) else {
            return;
        };
        let job = capture::turn_job(
            self.memory.as_ref().clone(),
            &session,
            protocol_name(protocol),
            space_id,
            model,
            &text,
        );
        self.writeback.track(format!("turn:{session}"), job);
    }

    /// PRX-01 S4: observe one upstream status and flip the limiter's
    /// degraded flag on enter/recover edges. Transport errors
    /// (timeout/unreachable) count as 503-class failures — the upstream
    /// didn't serve the request.
    fn note_upstream(&self, status: u16) {
        match self.upstream_health.observe(status) {
            Some(true) => {
                tracing::warn!(status, "upstream degraded — fail-open rate limiting");
                self.limiter.set_degraded(true);
            }
            Some(false) => {
                tracing::info!("upstream recovered — rate limiting enforced");
                self.limiter.set_degraded(false);
            }
            None => {}
        }
    }

    /// Verbatim forward (streaming passthrough).
    async fn forward_raw(
        &self,
        wire_path: &str,
        headers: &HeaderMap,
        body: Bytes,
        upstreams: &[UpstreamConfig],
    ) -> Response<Body> {
        // PRX-02: failover across the upstream list (PRX-06: tier-first
        // order when routing enforced; legacy single upstream otherwise).
        match self
            .forwarder
            .forward_with_failover(upstreams, Method::POST, wire_path, headers, body)
            .await
        {
            Ok(resp) => {
                self.note_upstream(resp.status().as_u16());
                resp
            }
            Err(e) => {
                self.note_upstream(503);
                e.into_response()
            }
        }
    }

    /// O2 agentic loop (D46/D48): forward, buffer the upstream SSE response,
    /// and while it invokes one of OUR memory tools, execute them server-side
    /// and re-request with synthesized tool results appended. Only the FINAL
    /// response reaches the client. Everything else (non-SSE, errors, bodies
    /// without our tools) forwards verbatim. PRX-06: Responses protocol
    /// participates (history appends to the `input` array).
    ///
    /// Trade-off accepted by design D46: turns where our tools are announced
    /// lose incremental streaming for buffered rounds.
    async fn forward_with_tool_loop(
        &self,
        protocol: Protocol,
        wire_path: &str,
        headers: &HeaderMap,
        body: Bytes,
        space_id: &str,
        session_key: &str,
        model: &str,
        upstreams: &[UpstreamConfig],
    ) -> Response<Body> {
        // Zero-overhead gate: only shapes with our tools announced pay for
        // interception (PRX-06: Responses included); everything else is
        // byte-identical passthrough.
        if !matches!(
            protocol,
            Protocol::OpenAI | Protocol::Anthropic | Protocol::Responses
        ) || !memory_tools::announces(&body)
        {
            return self.forward_raw(wire_path, headers, body, upstreams).await;
        }
        let protocol_label = protocol_name(protocol);
        let mut current = body;
        let mut executed = 0usize;
        loop {
            // PRX-02: each tool-loop round also fails over across upstreams
            // (PRX-06: tier-first order when routing enforced).
            let response = self
                .forwarder
                .forward_with_failover(upstreams, Method::POST, wire_path, headers, current.clone())
                .await;
            let (parts, body) = match response {
                Ok(response) => {
                    self.note_upstream(response.status().as_u16());
                    response.into_parts()
                }
                Err(e) => {
                    self.note_upstream(503);
                    return e.into_response();
                }
            };
            // Only successful SSE responses are interceptable; anything else
            // flows through untouched.
            let is_sse = parts
                .headers
                .get(sse_intercept::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.starts_with("text/event-stream"));
            if !parts.status.is_success() || !is_sse {
                return Response::from_parts(parts, body);
            }

            let captured = match sse_intercept::drain(body).await {
                Ok(captured) => captured,
                Err(e) => return e.into_response(),
            };
            let events = sse_intercept::data_events(&captured.full);
            let message = match protocol {
                Protocol::OpenAI => sse_intercept::openai_message(&events),
                Protocol::Responses => sse_intercept::responses_message(&events),
                Protocol::Anthropic => sse_intercept::anthropic_message(&events),
            };
            let calls = memory_tools::extract(&message);
            // No memory tool this round → final response. Cap reached → hand
            // the last response back verbatim (D48 hard stop).
            if calls.is_empty() || executed >= MAX_MEMORY_TOOL_ITERATIONS {
                tracing::debug!(
                    executed,
                    remaining_calls = calls.len(),
                    "memory-tool loop finished"
                );
                return sse_intercept::replay(parts, captured.chunks);
            }

            // Unparseable current body → cannot rebuild history; replay.
            let Ok(mut request) = serde_json::from_slice::<serde_json::Value>(current.as_ref())
            else {
                return sse_intercept::replay(parts, captured.chunks);
            };
            let results: Vec<(String, String)> = calls
                .iter()
                .map(|call| {
                    let text = memory_tools::execute(
                        &self.memory,
                        &self.writeback,
                        session_key,
                        protocol_label,
                        space_id,
                        model,
                        call,
                    );
                    (call.id.clone(), text)
                })
                .collect();
            executed += 1;
            memory_tools::append_exchange(protocol, &mut request, &message, &results);
            current = match serde_json::to_vec(&request) {
                Ok(bytes) => Bytes::from(bytes),
                Err(_) => return sse_intercept::replay(parts, captured.chunks),
            };
        }
    }
}

/// Assemble the proxy router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/snapshot", get(snapshot))
        .route("/session/advance", post(session_advance))
        .route(
            "/{agent}/{spaceId}/v1/chat/completions",
            post(handlers::openai::chat_completions),
        )
        .route(
            "/{agent}/{spaceId}/v1/messages",
            post(handlers::anthropic::messages),
        )
        .route("/v1/responses", post(handlers::responses::responses))
        // PRX-05: model discovery (Claude Code picker) + token counting.
        // Both the plain and `{agent}/{spaceId}`-prefixed shapes (parity
        // with responses vs chat/completions).
        .route("/v1/models", get(handlers::auxiliary::models))
        .route(
            "/{agent}/{spaceId}/v1/models",
            get(handlers::auxiliary::models),
        )
        .route(
            "/v1/messages/count_tokens",
            post(handlers::auxiliary::count_tokens),
        )
        .route(
            "/{agent}/{spaceId}/v1/messages/count_tokens",
            post(handlers::auxiliary::count_tokens),
        )
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

/// `POST /session/advance` (PRX-01): explicit session trigger alongside the
/// `x-vanta-session` header. Body: `{ "target": "team"|"agent"|"task",
/// "entity_id": "<id>" }`. Requires auth (401); missing key / bad target →
/// 400; unknown entity or illegal transition → [`crate::error::ProxyError`]
/// mapping (400).
async fn session_advance(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, crate::error::ProxyError> {
    use crate::error::ProxyError;
    state.auth.authenticate(&headers)?;
    let Some(key) = session_key_from_headers(&headers) else {
        return Err(ProxyError::InvalidRequest(
            "missing session key: send `x-vanta-session` (or another session alias)".into(),
        ));
    };
    let target = body
        .get("target")
        .and_then(serde_json::Value::as_str)
        .and_then(parse_stage)
        .ok_or_else(|| {
            ProxyError::InvalidRequest("`target` must be one of: team, agent, task".into())
        })?;
    let entity_id = body
        .get("entity_id")
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| ProxyError::InvalidRequest("`entity_id` must be non-empty".into()))?;
    let stage = state
        .sessions
        .advance(&state.auth, &key, target, entity_id)?;
    Ok(Json(serde_json::json!({ "stage": stage.label() })))
}

/// Live operational snapshot (DESKTOP-38): recent TurnReports, active
/// sessions, pending write-back queue and rate-limit telemetry. Read-only
/// over state that already exists — nothing here fabricates data.
async fn snapshot(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<serde_json::Value> {
    let sessions = state.sessions.snapshot();
    let pending_labels = state.writeback.pending_labels();
    Json(serde_json::json!({
        "turns": state.reporter.recent_reports(),
        "sessions": sessions,
        "sessions_active": sessions.len(),
        "writeback": {
            "pending_labels": pending_labels,
            "pending_count": state.writeback.pending_count(),
        },
        "rate_limit": {
            "limit_per_minute": state.limiter.limit(),
            "hits_total": state.limiter.hits_total(),
            "degraded": state.limiter.is_degraded(),
        },
        "cost": {
            "snapshot": state.cost.snapshot(),
            "default_budget_usd": state.config.cost.default_budget_usd,
            "enforce": state.config.cost.enforce,
        },
    }))
}

/// Stable protocol label for per-turn reports.
fn protocol_name(protocol: Protocol) -> &'static str {
    match protocol {
        Protocol::OpenAI => "openai",
        Protocol::Anthropic => "anthropic",
        Protocol::Responses => "responses",
    }
}

/// Replay a cached exact entry byte-for-byte (PRX-09 slice 1).
/// PRX-11 slice 3: opt-in request translation. Returns the effective
/// (protocol, upstream path, body) for cache+forward, or `None` when the
/// gate is closed / the body isn't JSON (wire stays byte-identical).
/// Pure over its inputs (no self) so it unit-tests without AppState.
fn translate_request(
    cfg: &translate::TranslateConfig,
    protocol: Protocol,
    _wire_path: &str,
    body: &Bytes,
) -> Option<(Protocol, String, Bytes)> {
    let src = match protocol {
        Protocol::Anthropic => TranslateSource::Anthropic,
        Protocol::OpenAI => TranslateSource::OpenAI,
        _ => return None,
    };
    if !translate::should_translate(cfg, src) {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let out = translate::anthropic_to_openai(&value);
    let bytes = Bytes::from(serde_json::to_vec(&out).unwrap_or_else(|_| body.to_vec()));
    // The translated body targets the OpenAI endpoint, not the client's
    // Anthropic path — sharing the entry with native OpenAI requests.
    Some((Protocol::OpenAI, "/v1/chat/completions".to_string(), bytes))
}

/// Cap for buffering a translated response before mapping it back
/// (same order as the cache cap — chat completions are small).
const MAX_TRANSLATED_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

/// PRX-11 slice 3: map a translated upstream response back to the client
/// protocol. SSE streams pass through untouched (never buffered); small
/// buffered JSON bodies are translated; anything else flows through
/// unchanged (fail-open — a mapping failure must never break a valid
/// upstream response, except an over-cap body which becomes a typed 502).
async fn map_translated_response(response: Response<Body>, translated: bool) -> Response<Body> {
    if !translated {
        return response;
    }
    let (parts, body) = response.into_parts();
    let is_sse = parts
        .headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.contains("event-stream"));
    if is_sse {
        return Response::from_parts(parts, body);
    }
    let too_big = parts
        .headers
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .is_some_and(|len| len > MAX_TRANSLATED_RESPONSE_BYTES);
    if too_big {
        return Response::from_parts(parts, body);
    }
    let limit = usize::try_from(MAX_TRANSLATED_RESPONSE_BYTES + 1).unwrap_or(usize::MAX);
    let bytes = match axum::body::to_bytes(body, limit).await {
        Ok(b) => b,
        Err(_) => {
            return crate::error::ProxyError::Forward(
                "translate: response too large to map back".to_string(),
            )
            .into_response()
        }
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return Response::from_parts(parts, Body::from(bytes));
    };
    let mapped = translate::openai_to_anthropic(&value);
    let out = Bytes::from(serde_json::to_vec(&mapped).unwrap_or_else(|_| bytes.to_vec()));
    let mut builder = Response::builder().status(parts.status);
    if let Some(headers) = builder.headers_mut() {
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/json"),
        );
        if let Ok(value) = axum::http::HeaderValue::from_str(&out.len().to_string()) {
            headers.insert(axum::http::header::CONTENT_LENGTH, value);
        }
    }
    builder.body(Body::from(out)).unwrap_or_else(|_| {
        crate::error::ProxyError::Forward("translate: rebuild failed".to_string()).into_response()
    })
}

fn cached_response(entry: &CachedEntry) -> Response<Body> {
    let status = StatusCode::from_u16(entry.status).unwrap_or(StatusCode::OK);
    let mut builder = Response::builder().status(status);
    if let Some(headers) = builder.headers_mut() {
        if let Ok(value) = axum::http::HeaderValue::from_str(&entry.content_type) {
            headers.insert(axum::http::header::CONTENT_TYPE, value);
        }
        if let Ok(value) = axum::http::HeaderValue::from_str(&entry.body.len().to_string()) {
            headers.insert(axum::http::header::CONTENT_LENGTH, value);
        }
    }
    builder
        .body(Body::from(entry.body.clone()))
        .unwrap_or_else(|_| {
            crate::error::ProxyError::Forward("cache replay failed".to_string()).into_response()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn anthropic_body(
        messages: serde_json::Value,
        extra: Option<(&str, serde_json::Value)>,
    ) -> Vec<u8> {
        let mut b = json!({"model": "claude-x", "messages": messages});
        if let Some((k, v)) = extra {
            b[k] = v;
        }
        serde_json::to_vec(&b).expect("serialize")
    }

    fn user_msg(content: serde_json::Value) -> serde_json::Value {
        json!({"role": "user", "content": content})
    }

    #[test]
    fn sidequery_body_on_anthropic_bypasses_pipeline() {
        // PRX-01: standalone TITLE-style request (no marker, no tools,
        // thinking off) skips session/inject/capture.
        let body = anthropic_body(
            json!([user_msg(json!("title this"))]),
            Some(("thinking", json!({"type": "disabled"}))),
        );
        assert!(is_cc_sidequery(Protocol::Anthropic, &body));
    }

    #[test]
    fn main_and_fork_bodies_stay_on_pipeline() {
        let mut marked = user_msg(json!([{"type":"text","text":"latest"}]));
        marked["content"][0]["cache_control"] = json!({"type": "ephemeral"});
        let main = anthropic_body(json!([user_msg(json!("hi")), marked]), None);
        assert!(!is_cc_sidequery(Protocol::Anthropic, &main));

        let mut fork_marked = user_msg(json!([{"type":"text","text":"prefix"}]));
        fork_marked["content"][0]["cache_control"] = json!({"type": "ephemeral"});
        let fork = anthropic_body(json!([fork_marked, user_msg(json!("new tail"))]), None);
        assert!(!is_cc_sidequery(Protocol::Anthropic, &fork));
    }

    #[test]
    fn non_anthropic_and_garbage_never_bypass() {
        let body = anthropic_body(
            json!([user_msg(json!("title this"))]),
            Some(("thinking", json!({"type": "disabled"}))),
        );
        assert!(!is_cc_sidequery(Protocol::OpenAI, &body));
        assert!(!is_cc_sidequery(Protocol::Anthropic, b"not json"));
        assert!(!is_cc_sidequery(Protocol::Anthropic, b"{}"));
    }

    // PRX-11 slice 3: translate hook gate + response map-back.
    fn anth_req() -> Bytes {
        Bytes::from(
            serde_json::json!({"model": "claude-x", "max_tokens": 10, "messages": [{"role": "user", "content": "hi"}]})
                .to_string(),
        )
    }

    #[test]
    fn translate_disabled_stays_verbatim() {
        let cfg = crate::translate::TranslateConfig::default();
        assert!(
            translate_request(&cfg, Protocol::Anthropic, "/v1/messages", &anth_req()).is_none()
        );
    }

    #[test]
    fn translate_openai_client_never_rewritten() {
        let cfg = crate::translate::TranslateConfig { enabled: true };
        assert!(
            translate_request(&cfg, Protocol::OpenAI, "/v1/chat/completions", &anth_req())
                .is_none()
        );
    }

    #[test]
    fn translate_enabled_maps_to_openai_path() {
        let cfg = crate::translate::TranslateConfig { enabled: true };
        let (proto, path, body) =
            translate_request(&cfg, Protocol::Anthropic, "/v1/messages", &anth_req())
                .expect("gate open + valid JSON");
        assert!(matches!(proto, Protocol::OpenAI));
        assert_eq!(path, "/v1/chat/completions");
        let v: serde_json::Value = serde_json::from_slice(&body).expect("valid JSON");
        assert_eq!(v["messages"][0]["role"], "user");
    }

    #[test]
    fn translate_garbage_body_fails_open() {
        let cfg = crate::translate::TranslateConfig { enabled: true };
        assert!(translate_request(
            &cfg,
            Protocol::Anthropic,
            "/v1/messages",
            &Bytes::from("{{{")
        )
        .is_none());
    }

    fn json_response(body: serde_json::Value) -> Response<Body> {
        Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .expect("build")
    }

    #[tokio::test]
    async fn mapback_passthrough_when_not_translated() {
        let resp = json_response(serde_json::json!({"a": 1}));
        let out = map_translated_response(resp, false).await;
        assert_eq!(out.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(out.into_body(), 1024)
            .await
            .expect("read");
        assert_eq!(bytes.as_ref(), b"{\"a\":1}");
    }

    #[tokio::test]
    async fn mapback_sse_never_buffered() {
        let resp = Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "text/event-stream")
            .body(Body::from("data: {}\n\n"))
            .expect("build");
        let out = map_translated_response(resp, true).await;
        assert_eq!(
            out.headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("text/event-stream")
        );
        let bytes = axum::body::to_bytes(out.into_body(), 1024)
            .await
            .expect("read");
        assert_eq!(bytes.as_ref(), b"data: {}\n\n");
    }

    #[tokio::test]
    async fn mapback_json_gets_anthropic_shape() {
        let upstream = serde_json::json!({
            "id": "chatcmpl-1", "model": "gpt-x",
            "choices": [{"message": {"role": "assistant", "content": "hi"}, "finish_reason": "stop"}]
        });
        let out = map_translated_response(json_response(upstream), true).await;
        assert_eq!(out.status(), StatusCode::OK);
        assert_eq!(
            out.headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );
        let bytes = axum::body::to_bytes(out.into_body(), 4096)
            .await
            .expect("read");
        let v: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
        assert_eq!(v["type"], "message");
        assert_eq!(v["content"][0]["text"], "hi");
    }

    #[tokio::test]
    async fn mapback_invalid_json_flows_untouched() {
        let out = map_translated_response(json_response(serde_json::json!("oops")), true).await;
        // `"oops"` is valid JSON (a string) → maps to an empty-content message.
        let bytes = axum::body::to_bytes(out.into_body(), 4096)
            .await
            .expect("read");
        let v: serde_json::Value = serde_json::from_slice(&bytes).expect("valid JSON");
        assert_eq!(v["type"], "message");
        let raw = Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(Body::from("not json{{{"))
            .expect("build");
        let out = map_translated_response(raw, true).await;
        let bytes = axum::body::to_bytes(out.into_body(), 4096)
            .await
            .expect("read");
        assert_eq!(bytes.as_ref(), b"not json{{{");
    }
}
