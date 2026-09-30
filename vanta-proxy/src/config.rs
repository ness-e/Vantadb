//! Proxy configuration loaded from a TOML file (decision D31).

use std::path::Path;

use serde::Deserialize;

use crate::error::ProxyError;

/// Default forward timeout in seconds (TDAM parity: config.ts:10 — 600_000 ms).
pub const DEFAULT_FORWARD_TIMEOUT_SECS: u64 = 600;
/// Default listen port (TDAM parity).
pub const DEFAULT_PORT: u16 = 8096;

/// Top-level proxy configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ProxyConfig {
    /// Listen address and port.
    pub server: ServerConfig,
    /// Upstream LLM endpoint.
    pub upstream: UpstreamConfig,
    /// extra upstream endpoints for PRX-02 failover, tried in order after
    /// `upstream`. Empty (default) → single-upstream legacy behavior, so old
    /// TOML files without this key parse unchanged.
    pub upstreams: Vec<UpstreamConfig>,
    /// Local auth/session store (D25/D34).
    pub auth: AuthConfig,
    /// In-band `mem:` commands (D33) — disabled by default (TDAM parity).
    pub mem_command: MemCommandConfig,
    /// L0 write-back persistence settings.
    pub writeback: WritebackConfig,
    /// Exact response cache (PRX-09 slice 1). Disabled by default (opt-in).
    pub cache: CacheConfig,
    /// Optional per-turn span export to Langfuse/OTel over OTLP-JSON
    /// (MEM-56). Disabled by default (empty endpoint).
    pub report: ReportConfig,
    /// Cost tracking + virtual keys (PRX-03). Tracking on, enforcement off
    /// by default (log-first — enforcement never blocks legitimate traffic
    /// unless explicitly configured).
    pub cost: CostConfig,
    /// Task-aware routing por tier (PRX-06). Disabled by default so the
    /// wire stays a transparent proxy unless explicitly opted in.
    pub routing: crate::routing::TierRoutingConfig,
    /// PII/secret redaction on egress (PRX-07). Disabled by default so the
    /// wire stays a transparent proxy unless explicitly opted in.
    pub redact: crate::redact::RedactConfig,
    /// Context optimization in transit (PRX-13). Disabled by default so
    /// the wire stays a transparent proxy unless explicitly opted in.
    pub context: crate::context::ContextConfig,
    /// Per-key model allowlists (PRX-10 guardrails). Disabled by default
    /// so the wire stays a transparent proxy unless explicitly opted in.
    pub guardrails: crate::guardrails::GuardrailConfig,
    /// Anthropic↔OpenAI translation (PRX-11 slice 3). Disabled by default
    /// so the wire stays byte-identical unless explicitly opted in.
    pub translate: crate::translate::TranslateConfig,
    /// Memory-block injection budget (WIRE-01). Caps `<vanta-memory>` tokens;
    /// defaults keep existing behavior (seeds are tiny against the default).
    pub injection: InjectionConfig,
    /// Per-namespace AEAD envelope for captured originals (VER-03). Disabled
    /// by default: the redacted text is the only copy unless opted in.
    pub envelope: crate::envelope::EnvelopeConfig,
}

impl ProxyConfig {
    /// Load configuration from a TOML file.
    pub fn load(path: &Path) -> Result<Self, ProxyError> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| ProxyError::Config(format!("cannot read {}: {e}", path.display())))?;
        toml::from_str(&raw).map_err(|e| ProxyError::Config(format!("invalid TOML: {e}")))
    }

    /// Ordered upstream list for PRX-02 failover: `upstreams` when configured,
    /// otherwise the legacy single `upstream` (TOML compat — old files keep
    /// working with zero changes). Pure and total.
    pub fn upstreams_resolved(&self) -> Vec<UpstreamConfig> {
        if self.upstreams.is_empty() {
            vec![self.upstream.clone()]
        } else {
            self.upstreams.clone()
        }
    }
}

/// Default `<vanta-memory>` budget in tokens (WIRE-01): persona + scene +
/// a few recent turns fit comfortably; runaway histories get truncated by
/// section priority instead of growing the prompt unbounded.
pub const DEFAULT_INJECTION_MAX_TOKENS: u64 = 2000;

/// VER-04: injection-audit rotation defaults (mirror the core audit config).
pub const DEFAULT_INJECTION_AUDIT_MAX_BYTES: u64 = 10 * 1024 * 1024;
/// VER-04: max rotated injection-audit archives kept.
pub const DEFAULT_INJECTION_AUDIT_MAX_FILES: u32 = 5;

/// Memory-block injection budget (WIRE-01). Caps the `<vanta-memory>` system
/// prompt block via the canonical `estimate_text_tokens` heuristic (~4
/// chars/token — guardrail precision, not billing). `0` disables memory
/// injection entirely (empty block → prompt untouched).
///
/// VER-04 adds the two governance knobs (both opt-in; defaults keep the wire
/// byte-identical):
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct InjectionConfig {
    /// Max tokens for the assembled block (wrapper tags included).
    pub max_tokens: u64,
    /// Injection ACL (VER-04): namespace prefixes the block may read from
    /// (`persona/`, `scene/`, `l1/`, ...). Empty = allow-all (default);
    /// namespaces not matched are skipped and recorded in the audit.
    pub namespace_allow_prefixes: Vec<String>,
    /// Injection audit (VER-04): JSONL path for `injection` events (which
    /// memory fed which prompt, under which budget/ACL decision). Empty =
    /// disabled. Append-only + rotated by the core `AuditLogger`.
    pub audit_log_path: String,
}

impl Default for InjectionConfig {
    fn default() -> Self {
        Self {
            max_tokens: DEFAULT_INJECTION_MAX_TOKENS,
            namespace_allow_prefixes: Vec::new(),
            audit_log_path: String::new(),
        }
    }
}

/// In-band `mem:` command interception (D33). Disabled by default so the
/// wire stays a transparent proxy unless explicitly opted in (TDAM parity).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MemCommandConfig {
    /// When false (default), `mem:*` messages are forwarded verbatim to the
    /// upstream LLM.
    pub enabled: bool,
}

/// L0 write-back queue persistence.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WritebackConfig {
    /// File holding labels of writes that exhausted retries (crash audit
    /// trail). Empty string disables persistence.
    pub persist_path: String,
}

impl Default for WritebackConfig {
    fn default() -> Self {
        Self {
            persist_path: "vanta-proxy-writeback-pending.json".to_string(),
        }
    }
}

/// Exact + semantic response cache (PRX-09 slice 1: exact-only; slice 2:
/// TTL + LRU + similarity). Disabled by default so the wire stays a
/// transparent proxy unless explicitly opted in.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// When false (default), every lookup misses and stores are no-ops.
    pub enabled: bool,
    /// Max entries held (oldest-first eviction past the cap; lookups refresh
    /// recency, so hot entries survive).
    pub max_entries: usize,
    /// Per-entry TTL in seconds (slice 2). Caching is active only when
    /// `enabled` AND `ttl_secs > 0`; 0 (default) means no TTL configured →
    /// cache disabled (API-05 X5 — the old "0 = never expires" convention
    /// was inverted).
    pub ttl_secs: u64,
    /// Similarity hits on near-duplicate prompts (slice 2). Off by default.
    pub semantic_enabled: bool,
    /// Cosine threshold over normalized prompt TF (slice 2).
    pub similarity_threshold: f32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_entries: 128,
            ttl_secs: 0,
            semantic_enabled: false,
            similarity_threshold: crate::cache::DEFAULT_SIMILARITY_THRESHOLD,
        }
    }
}

/// Optional per-turn span export to a Langfuse/OTel endpoint (MEM-56).
/// Disabled by default so an unconfigured proxy pays zero overhead.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ReportConfig {
    /// OTLP/HTTP JSON endpoint receiving one span per turn (e.g. Langfuse
    /// `https://cloud.langfuse.com/api/public/otel/v1/traces` or an OTel
    /// collector). Empty (default) disables export entirely.
    pub langfuse_endpoint: String,
    /// Raw value of the `Authorization` header sent with each export
    /// (e.g. Langfuse basic auth). Empty omits the header.
    pub langfuse_auth_header: String,
}

impl ReportConfig {
    /// Export is on only when an endpoint is configured.
    pub fn enabled(&self) -> bool {
        !self.langfuse_endpoint.is_empty()
    }
}

/// Cost tracking + virtual keys (PRX-03). All keys default so a TOML
/// without `[cost]` parses unchanged (legacy compat).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CostConfig {
    /// When false, no budget check runs and no turn is recorded.
    pub enabled: bool,
    /// Fallback budget in USD applied to keys without their own budget
    /// (`None` = untracked spend, always allowed). Per-key budgets
    /// (PRX-10 allowlists) override this.
    pub default_budget_usd: Option<f64>,
    /// Fallback enforcement: over budget + enforce → 429; over budget
    /// without enforce → warn + allow (log-first default).
    pub enforce: bool,
    /// Per-model USD/1K prices + `__default__` fallback for unknown models.
    pub prices: crate::cost::PriceTable,
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_budget_usd: None,
            enforce: false,
            prices: crate::cost::PriceTable::default(),
        }
    }
}

/// HTTP listener settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// Rate-limits per minute per `space_id`×model. Enforced in ONE place:
    /// the in-process sliding window in `server.rs` (`process_inner` step 2);
    /// `/snapshot` only reports its telemetry.
    pub rate_limit_per_minute: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: DEFAULT_PORT,
            rate_limit_per_minute: 60,
        }
    }
}

impl ProxyConfig {
    /// Refuse-to-start gate (WIRE-09, FIND-07 parity with the main server's
    /// `validate_auth_config`; API-05: auth now covers every route).
    ///
    /// 1. Every resolved upstream needs an explicit URL — the legacy code
    ///    default self-forwarded into this proxy (X4), so an empty URL
    ///    refuses with an actionable message instead of a silent loop.
    /// 2. Trust boundary: binding a non-loopback host with zero provisioned
    ///    user keys is refused — since API-05 `/snapshot` (like every route)
    ///    requires a valid `x-vanta-user-key`, a keyless non-loopback bind
    ///    could not serve anyone. Loopback binds remain the documented
    ///    local-dev exception. Deliberately no `--allow-insecure` override:
    ///    both remedies already exist, so no new config surface is needed.
    ///
    /// `pub(crate)` — startup wiring only, not public API (zero new symbols).
    pub(crate) fn validate_startup(&self, provisioned_user_keys: usize) -> Result<(), ProxyError> {
        if self
            .upstreams_resolved()
            .iter()
            .any(|upstream| upstream.url.trim().is_empty())
        {
            return Err(ProxyError::Config(
                "upstream.url is empty — set [upstream].url in config.toml \
                 (e.g. https://api.anthropic.com); no default is applied because \
                 the legacy default self-forwarded into this proxy"
                    .to_string(),
            ));
        }
        if provisioned_user_keys == 0 && !is_loopback_host(&self.server.host) {
            return Err(ProxyError::Config(format!(
                "refusing to start: non-loopback bind '{}' with no provisioned user keys — \
                 every route (including GET /snapshot) requires a valid user key, so this \
                 proxy could not serve anyone. Fix either way: (1) provision at least one \
                 `user` entity with a `user_key` in the auth store at '{}', or (2) bind a \
                 loopback host (127.0.0.1/localhost/::1)",
                self.server.host, self.auth.db_path
            )));
        }
        Ok(())
    }
}

/// Whether `host` binds only the loopback interface (`127.0.0.0/8`,
/// `::1`, or the literal name `localhost`). Unresolvable hostnames are
/// treated as non-loopback (fail closed). Copy of the FIND-07 helper in
/// `vantadb::server::bootstrap` (kept local: no shared util crate exists;
/// see WIRE-07 `ffi-core` plans).
fn is_loopback_host(host: &str) -> bool {
    let h = host.trim();
    let h = h.strip_prefix('[').unwrap_or(h);
    let h = h.strip_suffix(']').unwrap_or(h);
    h.eq_ignore_ascii_case("localhost")
        || h.parse::<std::net::IpAddr>()
            .map(|ip| ip.is_loopback())
            .unwrap_or(false)
}

/// Local auth/session store settings (D25: RBAC local entity_*, no remote gateway).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    /// Path of the local VantaDB store holding the `user`/`team`/`agent`/`task`
    /// entity collections used for auth (D34) and session validation (D26).
    pub db_path: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            db_path: "vantadb_data".to_string(),
        }
    }
}

/// Upstream LLM endpoint settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UpstreamConfig {
    /// Base URL of the upstream (e.g. `https://api.anthropic.com`). The wire
    /// path (`/v1/...`) is appended as received from the client.
    pub url: String,
    /// If non-empty, overrides the `Authorization` header sent upstream;
    /// otherwise the incoming header is passed through verbatim.
    pub api_key: String,
    /// Total forward timeout in seconds (D31/TDAM: default 600).
    pub forward_timeout_secs: u64,
    /// Model IDs served by `GET /v1/models` (PRX-05: derived from config,
    /// never hardcoded — a stale list breaks the Claude Code picker).
    /// Empty (default) → `{"object":"list","data":[]}`.
    pub models: Vec<String>,
}

impl UpstreamConfig {
    /// True when this upstream URL would route back into this proxy itself
    /// (loopback host + our own listen port) — a self-forwarding loop that
    /// recurses until the forward timeout (PRX-08 S2). Pure and total.
    pub fn points_at_self(&self, listen_port: u16) -> bool {
        let Some((host, port)) = split_host_port(&self.url) else {
            return false;
        };
        let loopback = matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1");
        loopback && port == listen_port
    }
}

/// Split `scheme://authority/rest` into lowercase (host, port). `None` when
/// unparseable or portless (portless → cannot be a self-loop on our port).
fn split_host_port(url: &str) -> Option<(String, u16)> {
    let authority = url.split("://").nth(1)?.split('/').next()?;
    let authority = authority.split('@').next_back()?;
    if let Some(rest) = authority.strip_prefix('[') {
        // [::1]:port
        let (host, port) = rest.split_once("]:")?;
        Some((host.to_ascii_lowercase(), port.parse().ok()?))
    } else {
        let (host, port) = authority.split_once(':')?;
        Some((host.to_ascii_lowercase(), port.parse().ok()?))
    }
}

impl Default for UpstreamConfig {
    fn default() -> Self {
        Self {
            // API-05 (X4): empty by default — `validate_startup` refuses to
            // start without an explicit URL. The old default
            // (`http://127.0.0.1:8096`) self-forwarded into this proxy.
            url: String::new(),
            api_key: String::new(),
            forward_timeout_secs: DEFAULT_FORWARD_TIMEOUT_SECS,
            models: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_upstream_is_empty_not_a_self_loop() {
        // API-05 (X4): the old default self-forwarded into this proxy; the
        // new default forces an explicit URL (`validate_startup` refuses
        // empty), so the shipped default can never loop.
        let cfg = UpstreamConfig::default();
        assert!(cfg.url.is_empty(), "default upstream URL must be empty");
        assert!(!cfg.points_at_self(DEFAULT_PORT));
    }

    #[test]
    fn explicit_self_loop_url_is_still_detected() {
        let cfg = UpstreamConfig {
            url: "http://127.0.0.1:8096".to_string(),
            ..UpstreamConfig::default()
        };
        assert!(cfg.points_at_self(DEFAULT_PORT));
        assert!(cfg.points_at_self(8096));
    }

    #[test]
    fn mock_upstream_on_other_port_is_not_self() {
        let cfg = UpstreamConfig {
            url: "http://127.0.0.1:51234".to_string(),
            ..UpstreamConfig::default()
        };
        assert!(!cfg.points_at_self(DEFAULT_PORT));
    }

    #[test]
    fn remote_upstream_is_not_self_even_on_same_port() {
        let cfg = UpstreamConfig {
            url: "https://api.anthropic.com".to_string(),
            ..UpstreamConfig::default()
        };
        assert!(!cfg.points_at_self(443));
    }

    #[test]
    fn resolved_falls_back_to_legacy_single_upstream() {
        // PRX-02: old TOML without `upstreams` → exactly the legacy upstream.
        let cfg = ProxyConfig::default();
        let list = cfg.upstreams_resolved();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].url, cfg.upstream.url);
    }

    #[test]
    fn resolved_prefers_upstreams_when_configured() {
        let cfg = ProxyConfig {
            upstreams: vec![
                UpstreamConfig {
                    url: "http://127.0.0.1:9001".to_string(),
                    ..UpstreamConfig::default()
                },
                UpstreamConfig {
                    url: "http://127.0.0.1:9002".to_string(),
                    ..UpstreamConfig::default()
                },
            ],
            ..ProxyConfig::default()
        };
        let list = cfg.upstreams_resolved();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].url, "http://127.0.0.1:9001");
        assert_eq!(list[1].url, "http://127.0.0.1:9002");
    }

    #[test]
    fn legacy_toml_without_upstreams_still_parses() {
        // PRX-02 compat: a pre-failover TOML file has no `upstreams` key.
        let cfg: ProxyConfig = toml::from_str(
            "[server]\nhost = \"127.0.0.1\"\nport = 8096\n\
             [upstream]\nurl = \"https://api.anthropic.com\"\napi_key = \"k\"\n",
        )
        .expect("legacy TOML must parse");
        assert!(cfg.upstreams.is_empty());
        assert_eq!(cfg.upstreams_resolved().len(), 1);
    }

    #[test]
    fn legacy_toml_without_cost_still_parses_with_tracking_defaults() {
        // PRX-03 compat: a pre-cost TOML file has no `[cost]` key —
        // tracking on, no budget, no enforcement (log-first).
        let cfg: ProxyConfig = toml::from_str(
            "[server]\nhost = \"127.0.0.1\"\nport = 8096\n\
             [upstream]\nurl = \"https://api.anthropic.com\"\napi_key = \"k\"\n",
        )
        .expect("legacy TOML must parse");
        assert!(cfg.cost.enabled);
        assert_eq!(cfg.cost.default_budget_usd, None);
        assert!(!cfg.cost.enforce);
        assert!(
            cfg.cost.prices.cost_usd(
                "gpt-4o",
                &crate::cost::Usage {
                    input_tokens: 1000,
                    output_tokens: 0,
                }
            ) > 0.0
        );
    }

    #[test]
    fn toml_with_cost_parses_budget_and_enforce() {
        let cfg: ProxyConfig = toml::from_str(
            "[upstream]\nurl = \"https://api.anthropic.com\"\n\
             [cost]\nenabled = true\ndefault_budget_usd = 5.0\nenforce = true\n",
        )
        .expect("cost TOML must parse");
        assert!(cfg.cost.enabled);
        assert_eq!(cfg.cost.default_budget_usd, Some(5.0));
        assert!(cfg.cost.enforce);
    }

    #[test]
    fn toml_with_upstreams_parses_in_order() {
        let cfg: ProxyConfig = toml::from_str(
            "[upstream]\nurl = \"http://127.0.0.1:9001\"\n\
             [[upstreams]]\nurl = \"http://127.0.0.1:9001\"\n\
             [[upstreams]]\nurl = \"http://127.0.0.1:9002\"\napi_key = \"b\"\n",
        )
        .expect("multi-upstream TOML must parse");
        let list = cfg.upstreams_resolved();
        assert_eq!(list.len(), 2);
        assert_eq!(list[1].api_key, "b");
    }

    // ─── WIRE-09: refuse-to-start (FIND-07 parity) ──────────────
    // RED: `validate_startup` does not exist yet — these fail to compile
    // pre-fix; post-fix they pin the gate. Contract: non-loopback bind
    // without provisioned user keys refuses; loopback or ≥1 key starts.

    fn proxy_cfg_with_host(host: &str) -> ProxyConfig {
        ProxyConfig {
            server: ServerConfig {
                host: host.into(),
                ..ServerConfig::default()
            },
            // API-05: explicit upstream so the empty-URL refusal stays quiet.
            upstream: UpstreamConfig {
                url: "https://api.example.com".to_string(),
                ..UpstreamConfig::default()
            },
            ..ProxyConfig::default()
        }
    }

    #[test]
    fn refuse_start_empty_upstream_url() {
        // API-05 (X4): no default upstream — empty URL refuses with an
        // actionable message instead of a silent self-loop.
        let mut cfg = proxy_cfg_with_host("127.0.0.1");
        cfg.upstream.url = String::new();
        let err = cfg.validate_startup(0).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("upstream.url is empty"),
            "message must name the missing upstream, got: {msg}"
        );
    }

    #[test]
    fn refuse_start_empty_upstream_in_failover_list() {
        // The failover list is checked too (it replaces `upstream` when set).
        let mut cfg = proxy_cfg_with_host("127.0.0.1");
        cfg.upstreams = vec![UpstreamConfig {
            url: "  ".to_string(),
            ..UpstreamConfig::default()
        }];
        assert!(cfg.validate_startup(0).is_err());
    }

    #[test]
    fn refuse_start_non_loopback_without_keys() {
        for host in ["0.0.0.0", "192.168.1.10", "example.com", "::"] {
            let cfg = proxy_cfg_with_host(host);
            let err = cfg.validate_startup(0).unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("refusing to start") && msg.contains("user_key"),
                "host {host}: message must name the refusal + remedy, got: {msg}"
            );
        }
    }

    #[test]
    fn refuse_start_loopback_without_keys_ok() {
        for host in ["127.0.0.1", "localhost", "::1", "[::1]"] {
            let cfg = proxy_cfg_with_host(host);
            assert!(
                cfg.validate_startup(0).is_ok(),
                "loopback host {host} must start without keys"
            );
        }
    }

    #[test]
    fn refuse_start_non_loopback_with_keys_ok() {
        let cfg = proxy_cfg_with_host("0.0.0.0");
        assert!(cfg.validate_startup(1).is_ok());
    }
}
