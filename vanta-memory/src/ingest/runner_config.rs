//! Local-first ingest runner config (IMPL-112-S1).
//!
//! Server-side config for the wiki ingest LLM runner: minimal TOML (no
//! secrets, ever) + `VANTADB_INGEST_*` env with precedence over TOML over the
//! inherited `VANTADB_LLM_*` values over code defaults. Builds the concrete
//! [`ConcreteRunner`] moved per-call into [`crate::ingest::worker::execute`]
//! via `vantadb-mcp::start_ingest::<ConcreteRunner>` — no global, no secret in
//! a static (spec FIND-112 §(b)+§(c), decisions S1-S3).
//!
//! Default (no env, no TOML, no model) degrades exactly like `NoLlm`
//! ([`LlmError::NotConfigured`] per chunk → `sources_skipped`, never a hard
//! error — contract P4 §(e)).

use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::adapters::standalone::llm_runner::{LlmConfig, StandaloneLlmRunner};
use crate::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use crate::ingest::{clamp_llm_concurrency, IngestConfig};

/// Default request timeout when unset (mirrors `StandaloneLlmRunner`).
pub const DEFAULT_TIMEOUT_SECS: u64 = 120;
/// Timeout clamp bounds (contract-first: never hang, never spin).
pub const MIN_TIMEOUT_SECS: u64 = 5;
pub const MAX_TIMEOUT_SECS: u64 = 600;
/// Max-tokens clamp bounds when explicitly set (`0` = runner default).
pub const MAX_MAX_TOKENS: u32 = 8000;

/// Ingest runner provider (spec §(a) matrix, S1: local honest + standalone reuse).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IngestRunnerProvider {
    /// Zero network, zero secrets. S1 has no local chat engine, so this
    /// always degrades to [`LlmError::NotConfigured`] — bit-for-bit the
    /// current `NoLlm` behaviour (contract §(e)).
    #[default]
    Local,
    /// HTTP via `StandaloneLlmRunner` + feature `llm-driver` (S2 wires G2).
    Ollama,
    /// HTTPS via `StandaloneLlmRunner` + feature `llm-driver`, key from env
    /// only (S2 wires G2).
    OpenAi,
}

impl IngestRunnerProvider {
    /// Parse a raw provider name: unknown → `Local` with a warn (R-5
    /// core-engine: warn + safe default, never panic).
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(str::trim).map(str::to_lowercase).as_deref() {
            None | Some("") | Some("local") => Self::Local,
            Some("ollama") => Self::Ollama,
            Some("openai") => Self::OpenAi,
            Some(other) => {
                tracing::warn!(provider = %other, "unknown ingest provider, falling back to local");
                Self::Local
            }
        }
    }
}

/// Server-side ingest runner config (spec §(b)). `openai_api_key` is populated
/// ONLY from env (`VANTADB_OPENAI_API_KEY`) — never from TOML (G0).
#[derive(Debug, Clone)]
pub struct IngestRunnerCfg {
    pub provider: IngestRunnerProvider,
    /// `""` = default per provider (resolved in [`Self::apply_env`]).
    pub model: String,
    /// `""` = default per provider.
    pub base_url: String,
    /// `0` = runner default; else clamped 1..=[`MAX_MAX_TOKENS`].
    pub max_tokens: u32,
    /// `0` = [`DEFAULT_TIMEOUT_SECS`]; else clamped 5..=600.
    pub timeout_secs: u64,
    /// Pipeline knobs (`[ingest.pipeline]`, reuses [`clamp_llm_concurrency`]).
    pub pipeline: IngestConfig,
    /// OpenAI key presence, env-only. `#[allow(dead_code)]` is NOT used: this
    /// field is read by [`Self::build_runner`] (S2 path included).
    pub openai_api_key: Option<String>,
}

/// Concrete runner enum (S1: delegation, NO `Box<dyn>` — the `start_ingest<R>`
/// generic monomorphises per `R` without object-safety).
///
/// S1 omits a `Local` variant on purpose: no local chat engine exists in the
/// codebase (embed-local is embeddings, not `LlmRunner::run` text), so
/// local-without-model is `None` — a dead variant would trip the workspace
/// `deny(warnings)` dead-code lint. The variant arrives with a real engine.
#[derive(Debug)]
pub enum ConcreteRunner {
    Ollama(StandaloneLlmRunner),
    OpenAi(StandaloneLlmRunner),
    /// Explicit LLM-free (S4: `NoLlm` lives on as a variant + as the facade
    /// struct for tests). `run` = [`LlmError::NotConfigured`].
    None,
}

impl LlmRunner for ConcreteRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        match self {
            Self::Ollama(r) | Self::OpenAi(r) => r.run(params),
            Self::None => Err(LlmError::NotConfigured),
        }
    }
}

/// Build the per-call runner (S3): `None` (Option) is reserved for a future
/// `off` kill-switch; every S1 provider maps to `Some` — degraded providers
/// to `Some(ConcreteRunner::None)` so the worker takes its existing P4 path.
pub fn build_ingest_runner(cfg: &IngestRunnerCfg) -> Option<ConcreteRunner> {
    cfg.build_runner()
}

impl IngestRunnerCfg {
    pub fn defaults() -> Self {
        Self {
            provider: IngestRunnerProvider::Local,
            model: String::new(),
            base_url: String::new(),
            max_tokens: 0,
            timeout_secs: 0,
            pipeline: IngestConfig::default(),
            openai_api_key: None,
        }
    }

    /// Parse a TOML doc (`[ingest]` + `[ingest.pipeline]`); unparseable →
    /// defaults with a warn (never panic on operator config).
    pub fn from_toml_str(s: &str) -> Self {
        let mut cfg = Self::defaults();
        let file: TomlFile = match toml::from_str(s) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(error = %e, "ignoring unparseable ingest TOML, using defaults");
                return cfg;
            }
        };
        let t = file.ingest;
        cfg.provider = IngestRunnerProvider::parse(t.provider.as_deref());
        if let Some(v) = t.model {
            cfg.model = v;
        }
        if let Some(v) = t.base_url {
            cfg.base_url = v;
        }
        if let Some(v) = t.max_tokens {
            cfg.max_tokens = v;
        }
        if let Some(v) = t.timeout_secs {
            cfg.timeout_secs = v;
        }
        let p = t.pipeline;
        cfg.pipeline = IngestConfig {
            global_llm_concurrency: clamp_llm_concurrency(p.global_llm_concurrency),
            chunk_target_chars: p
                .chunk_target_chars
                .filter(|&n| n > 0)
                .unwrap_or(vantadb::wiki::DEFAULT_TARGET_CHARS),
            chunk_overlap_chars: p
                .chunk_overlap_chars
                .unwrap_or(vantadb::wiki::DEFAULT_OVERLAP_CHARS),
        };
        cfg.validate();
        cfg
    }

    /// Apply env overrides (`VANTADB_INGEST_*` win over TOML; legacy
    /// `VANTADB_LLM_*` fill provider defaults). `get` is injected so tests
    /// stay pure (no process-global env mutation under parallel harness).
    pub fn apply_env(&mut self, get: impl Fn(&str) -> Option<String>) {
        if let Some(raw) = get("VANTADB_INGEST_PROVIDER") {
            self.provider = IngestRunnerProvider::parse(Some(&raw));
        }
        if let Some(v) = get("VANTADB_INGEST_MODEL") {
            self.model = v;
        }
        if let Some(v) = get("VANTADB_INGEST_BASE_URL") {
            self.base_url = v;
        }
        if let Some(raw) = get("VANTADB_INGEST_MAX_TOKENS") {
            match raw.trim().parse::<u32>() {
                Ok(n) => self.max_tokens = n,
                Err(_) => {
                    tracing::warn!(value = %raw, "invalid VANTADB_INGEST_MAX_TOKENS, keeping previous")
                }
            }
        }
        if let Some(raw) = get("VANTADB_INGEST_TIMEOUT_SECS") {
            match raw.trim().parse::<u64>() {
                Ok(n) => self.timeout_secs = n,
                Err(_) => {
                    tracing::warn!(value = %raw, "invalid VANTADB_INGEST_TIMEOUT_SECS, keeping previous")
                }
            }
        }
        // Inherited provider defaults (only when unset by TOML/env-specific).
        match self.provider {
            IngestRunnerProvider::Ollama => {
                if self.model.is_empty() {
                    self.model = get("VANTADB_LLM_MODEL").unwrap_or_else(|| "all-minilm".into());
                }
                if self.base_url.is_empty() {
                    self.base_url =
                        get("VANTADB_LLM_URL").unwrap_or_else(|| "http://localhost:11434".into());
                }
            }
            IngestRunnerProvider::OpenAi => {
                if self.model.is_empty() {
                    self.model = get("VANTADB_INGEST_OPENAI_MODEL")
                        .or_else(|| get("VANTADB_OPENAI_MODEL"))
                        .unwrap_or_else(|| "gpt-4o-mini".into());
                }
                if self.base_url.is_empty() {
                    self.base_url = "https://api.openai.com/v1".to_string();
                }
                self.openai_api_key =
                    get("VANTADB_OPENAI_API_KEY").filter(|k| !k.trim().is_empty());
            }
            IngestRunnerProvider::Local => {}
        }
        self.validate();
    }

    /// Full server-side load: TOML at `path` (missing/unreadable → defaults +
    /// warn) + real process env + validation.
    pub fn from_env_toml(path: &Path) -> Self {
        let mut cfg = match std::fs::read_to_string(path) {
            Ok(s) => Self::from_toml_str(&s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::defaults(),
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "unreadable ingest TOML, using defaults");
                Self::defaults()
            }
        };
        cfg.apply_env(|k| std::env::var(k).ok());
        cfg
    }

    /// Boundary validation (api-and-interface-design §3): clamps + safe
    /// defaults, never panics.
    pub fn validate(&mut self) {
        if self.timeout_secs == 0 {
            self.timeout_secs = DEFAULT_TIMEOUT_SECS;
        } else {
            self.timeout_secs = self.timeout_secs.clamp(MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS);
        }
        if self.max_tokens > MAX_MAX_TOKENS {
            tracing::warn!(max_tokens = self.max_tokens, "clamping ingest max_tokens");
            self.max_tokens = MAX_MAX_TOKENS;
        }
    }

    /// Pipeline section as the worker's [`IngestConfig`].
    pub fn pipeline_config(&self) -> IngestConfig {
        self.pipeline.clone()
    }

    /// Per-call constructor (S3): pure config mapping, no I/O.
    pub fn build_runner(&self) -> Option<ConcreteRunner> {
        match self.provider {
            IngestRunnerProvider::Local => Some(ConcreteRunner::None),
            IngestRunnerProvider::Ollama => Some(ConcreteRunner::Ollama(self.standalone_cfg())),
            IngestRunnerProvider::OpenAi if self.openai_api_key.is_some() => {
                Some(ConcreteRunner::OpenAi(self.standalone_cfg()))
            }
            // No key in env → deferred NotConfigured (pattern B2b `llm.rs`:
            // error at `run`, never panic at construction).
            IngestRunnerProvider::OpenAi => Some(ConcreteRunner::None),
        }
    }

    /// Shared standalone config (ollama/openai differ only by resolved
    /// base_url/model/key — one constructor, no branching at call sites).
    fn standalone_cfg(&self) -> StandaloneLlmRunner {
        let mut cfg = self.clone();
        cfg.validate();
        StandaloneLlmRunner::new(LlmConfig {
            base_url: cfg.base_url.clone(),
            api_key: cfg.openai_api_key.clone().unwrap_or_default(),
            model: cfg.model.clone(),
            max_tokens: (cfg.max_tokens > 0).then_some(cfg.max_tokens),
            timeout: Some(Duration::from_secs(cfg.timeout_secs)),
        })
    }
}

// ── TOML schema (private; `[ingest]` + `[ingest.pipeline]`, no secrets) ──

#[derive(Debug, Default, Deserialize)]
struct TomlFile {
    #[serde(default)]
    ingest: TomlIngest,
}

#[derive(Debug, Default, Deserialize)]
struct TomlIngest {
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    timeout_secs: Option<u64>,
    // NOTE: no `api_key` field on purpose — a key in TOML must never be
    // honoured (G0, test 4). Unknown keys are ignored by serde default.
    #[serde(default)]
    pipeline: TomlPipeline,
}

#[derive(Debug, Default, Deserialize)]
struct TomlPipeline {
    #[serde(default)]
    global_llm_concurrency: Option<usize>,
    #[serde(default)]
    chunk_target_chars: Option<usize>,
    #[serde(default)]
    chunk_overlap_chars: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_empty(_: &str) -> Option<String> {
        None
    }

    fn get_ollama_provider(k: &str) -> Option<String> {
        (k == "VANTADB_INGEST_PROVIDER").then(|| "ollama".to_string())
    }

    fn get_watson_provider(k: &str) -> Option<String> {
        (k == "VANTADB_INGEST_PROVIDER").then(|| "watson".to_string())
    }

    #[test]
    fn ingest_runner_cfg_defaults_to_local_p4() {
        let cfg = IngestRunnerCfg::defaults();
        assert_eq!(cfg.provider, IngestRunnerProvider::Local);
        // Default local has no chat engine: degrades like NoLlm.
        let runner = cfg.build_runner().expect("S1 always returns Some");
        assert!(matches!(runner, ConcreteRunner::None));
        let err = runner
            .run(&LlmRunParams::new("hello", "ingest-extract"))
            .expect_err("must degrade");
        assert!(matches!(err, LlmError::NotConfigured));
    }

    #[test]
    fn ingest_runner_cfg_env_beats_toml() {
        let mut cfg = IngestRunnerCfg::from_toml_str(
            "[ingest]\nprovider = \"openai\"\nmodel = \"toml-model\"\n",
        );
        assert_eq!(cfg.provider, IngestRunnerProvider::OpenAi);
        cfg.apply_env(get_ollama_provider);
        assert_eq!(cfg.provider, IngestRunnerProvider::Ollama);
        // TOML model survives when env does not override it.
        assert_eq!(cfg.model, "toml-model");
    }

    #[test]
    fn ingest_runner_cfg_unknown_provider_warns_and_falls_back() {
        let cfg = IngestRunnerCfg::from_toml_str("[ingest]\nprovider = \"watson\"\n");
        assert_eq!(cfg.provider, IngestRunnerProvider::Local);
        let mut cfg2 = IngestRunnerCfg::defaults();
        cfg2.apply_env(get_watson_provider);
        assert_eq!(cfg2.provider, IngestRunnerProvider::Local);
    }

    #[test]
    fn ingest_runner_cfg_never_reads_key_from_toml() {
        let mut cfg = IngestRunnerCfg::from_toml_str(
            "[ingest]\nprovider = \"openai\"\napi_key = \"sk-toml-must-be-ignored\"\n",
        );
        cfg.apply_env(get_empty);
        assert_eq!(
            cfg.openai_api_key, None,
            "TOML api_key must never be honoured (G0)"
        );
        // Without env key the openai provider degrades, never authenticates.
        let runner = cfg.build_runner().expect("Some");
        assert!(matches!(runner, ConcreteRunner::None));
    }

    #[test]
    fn ingest_concrete_runner_is_send_sync_static() {
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_send_sync_static::<ConcreteRunner>();
    }

    #[test]
    fn ingest_chunk_config_clamps() {
        let cfg = IngestRunnerCfg::from_toml_str(
            "[ingest.pipeline]\nglobal_llm_concurrency = 99\nchunk_target_chars = 0\n",
        );
        let pipeline = cfg.pipeline_config();
        assert_eq!(pipeline.global_llm_concurrency, 20);
        // 0 = unset → worker default, via the shared clamp (no logic dup).
        assert_eq!(
            pipeline.global_llm_concurrency,
            clamp_llm_concurrency(Some(99))
        );
        let cfg0 =
            IngestRunnerCfg::from_toml_str("[ingest.pipeline]\nglobal_llm_concurrency = 0\n");
        assert_eq!(
            cfg0.pipeline_config().global_llm_concurrency,
            clamp_llm_concurrency(Some(0))
        );
        // Timeout / max_tokens clamps.
        let mut cfg = IngestRunnerCfg::defaults();
        cfg.timeout_secs = 9999;
        cfg.max_tokens = 99999;
        cfg.validate();
        assert_eq!(cfg.timeout_secs, MAX_TIMEOUT_SECS);
        assert_eq!(cfg.max_tokens, MAX_MAX_TOKENS);
    }
}
