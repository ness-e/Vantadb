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
use crate::config::ProxyConfig;
use crate::forward::Forwarder;
use crate::handlers;
use crate::inject::{self, Protocol};
use crate::mem_command;
use crate::rate_limit::{self, RateDecision, RateLimiter};
use crate::report::{model_from_body, now_ms_u64, Reporter, TurnReport, TurnTimer};
use crate::session::{session_key_from_headers, SessionStore};
use crate::writeback::WriteBack;

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
    /// L0 write-back coordinator (MEM-27).
    pub writeback: Arc<WriteBack>,
    /// Per-turn structured reporting (MEM-27).
    pub reporter: Arc<Reporter>,
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
        Ok(Self {
            limiter: RateLimiter::new(config.server.rate_limit_per_minute).into(),
            writeback: WriteBack::new(persist_path).into(),
            reporter: Reporter::new().into(),
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
            .process_inner(protocol, wire_path, headers, body, space_id, &model)
            .await;
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
                return mem_command::respond(&mem_command::execute(&cmd));
            }
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

        self.forward_raw(wire_path, headers, body).await
    }

    /// Verbatim forward (streaming passthrough).
    async fn forward_raw(
        &self,
        wire_path: &str,
        headers: &HeaderMap,
        body: Bytes,
    ) -> Response<Body> {
        self.forwarder
            .forward(
                &self.config.upstream,
                Method::POST,
                wire_path,
                headers,
                body,
            )
            .await
            .unwrap_or_else(IntoResponse::into_response)
    }
}

/// Assemble the proxy router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
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

/// Stable protocol label for per-turn reports.
fn protocol_name(protocol: Protocol) -> &'static str {
    match protocol {
        Protocol::OpenAI => "openai",
        Protocol::Anthropic => "anthropic",
        Protocol::Responses => "responses",
    }
}
