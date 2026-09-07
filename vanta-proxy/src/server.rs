//! Router assembly (TDAM parity: server.ts:307,312 — primary agent-prefixed routes).
//!
//! Every wire handler runs the MEM-26 pipeline BEFORE forwarding:
//! auth (D34) → session (D26) → inject (D29) → forward.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, Method, Response};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::StorageEngine;

use crate::auth::AuthDb;
use crate::capture;
use crate::config::ProxyConfig;
use crate::forward::Forwarder;
use crate::handlers;
use crate::inject::{self, Protocol};
use crate::mem_command;
use crate::memory_tools;
use crate::rate_limit::{self, RateDecision, RateLimiter, UpstreamHealth};
use crate::report::{model_from_body, now_ms_u64, Reporter, TurnReport, TurnTimer};
use crate::session::claude_code::CcRequestKind;
use crate::session::{parse_stage, session_key_from_headers, SessionStore};
use crate::sse_intercept;
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
    pub memory: Arc<VantaEmbedded>,
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
        let forwarder = Forwarder::new(&config.upstream)?;
        let persist_path = (!config.writeback.persist_path.is_empty())
            .then(|| std::path::PathBuf::from(&config.writeback.persist_path));
        let reporter = Reporter::new();
        // MEM-56: optional OTLP span export — None (no hook) unless an
        // endpoint is configured in `[report]`.
        if let Some(hook) = crate::langfuse::langfuse_hook(&config.report) {
            reporter.add_hook(hook);
        }
        Ok(Self {
            limiter: RateLimiter::new(config.server.rate_limit_per_minute).into(),
            upstream_health: UpstreamHealth::new().into(),
            writeback: WriteBack::new(persist_path).into(),
            reporter: reporter.into(),
            config: Arc::new(config),
            forwarder: Arc::new(forwarder),
            auth: AuthDb::new(engine.clone()).into(),
            memory: VantaEmbedded::from_engine(engine).into(),
            sessions: SessionStore::new().into(),
        })
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
        let model = model_from_body(&body);
        let response = self
            .process_inner(protocol, wire_path, headers, body.clone(), space_id, &model)
            .await;
        // D47/MEM-50: completed request → L0 turn capture. Fire-and-forget
        // AFTER the response is built — a slow or failing memory write can
        // never delay or break the forward. Sidequeries bypass capture:
        // they are not conversation turns (PRX-01).
        if response.status().is_success() && !is_cc_sidequery(protocol, &body) {
            self.capture_turn(protocol, headers, space_id, &model, &body);
        }
        self.reporter.emit(&TurnReport {
            timestamp_ms: now_ms_u64(),
            space_id: space_id.to_string(),
            protocol: protocol_name(protocol).to_string(),
            model,
            status: response.status().as_u16(),
            duration_ms: timer.elapsed_ms(),
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
    ) -> Response<Body> {
        // 1) D34: every request needs a valid user key — no open mode. Auth
        // runs BEFORE the limiter so unauthenticated traffic never burns
        // another identity's quota slots (invalid keys always get 401).
        if let Err(e) = self.auth.authenticate(headers) {
            tracing::debug!(error = %e, "request rejected by auth");
            return e.into_response();
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
            return self.forward_raw(wire_path, headers, body).await;
        }

        // 4) D26: resolve/create the session and refresh its TTL clock.
        let Some(key) = session_key_from_headers(headers) else {
            // No session context → clean init: forward verbatim (D29 applies
            // to sessions only).
            return self.forward_raw(wire_path, headers, body).await;
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

        self.forward_with_tool_loop(protocol, wire_path, headers, body, space_id, &key, model)
            .await
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
    ) -> Response<Body> {
        match self
            .forwarder
            .forward(
                &self.config.upstream,
                Method::POST,
                wire_path,
                headers,
                body,
            )
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
    /// without our tools, Responses protocol) forwards verbatim.
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
    ) -> Response<Body> {
        // Zero-overhead gate: only OpenAI/Anthropic shapes with our tools
        // announced pay for interception; everything else is byte-identical
        // passthrough.
        if !matches!(protocol, Protocol::OpenAI | Protocol::Anthropic)
            || !memory_tools::announces(&body)
        {
            return self.forward_raw(wire_path, headers, body).await;
        }
        let protocol_label = protocol_name(protocol);
        let mut current = body;
        let mut executed = 0usize;
        loop {
            let response = self
                .forwarder
                .forward(
                    &self.config.upstream,
                    Method::POST,
                    wire_path,
                    headers,
                    current.clone(),
                )
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
                Protocol::OpenAI | Protocol::Responses => sse_intercept::openai_message(&events),
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
}
