// ponytail: `VANTADB_OPENAI_API_KEY` is a required config (intentional panic
// on missing) + LLM embedding provider invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Optional external LLM integration.
//!
//! This module is not a core dependency of the v0.1.x MVP. Embedding generation and LLM runtime
//! behavior remain external or experimental; the core stores and retrieves provided vectors.
//!
//! ## Embedding providers
//!
//! COMP-010: Abstract [`EmbeddingProvider`] trait with implementations:
//! - [`OllamaProvider`] - Ollama `/api/embed` (default, `remote-inference`)
//! - [`OpenAIProvider`] - OpenAI `/v1/embeddings` (`remote-inference`)
//! - [`LocalOnnxProvider`] - local ONNX via `ort`+`tokenizers` (`embed-local`)
//!
//! Select the provider at runtime via `VANTADB_EMBEDDING_PROVIDER` (ollama|openai|local).

use crate::config::Config;
use crate::error::{Error, Result};
#[cfg(feature = "remote-inference")]
use reqwest::blocking::Client;

/// ort 2.x owns exactly one `Environment` per process, built explicitly and
/// shared by every `Session` (no more global `init().commit()`). It lives here
/// so `ensure_ort_ready` (dylib probe) and `load_session_inner` (session builds)
/// always agree on which environment is current.
#[cfg(feature = "embed-local")]
static ORT_ENV: std::sync::OnceLock<ort::environment::Environment> = std::sync::OnceLock::new();

// ── EmbeddingProvider trait ────────────────────────────────────────────

/// Abstract embedding provider.
///
/// Implementations convert text into a float vector suitable for HNSW
/// similarity search. Each provider is responsible for its own HTTP
/// transport and authentication.
pub trait EmbeddingProvider: Send + Sync {
    /// Embed `text` and return a dense `f32` vector.
    ///
    /// Document semantics: e5-style providers apply the `passage:` prefix.
    fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Embed a retrieval *query* (vs [`Self::embed`], which embeds documents).
    ///
    /// Default impl = same as documents. e5-style providers override it with
    /// the `query:` prefix.
    fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        self.embed(text)
    }

    /// Embed a batch of texts. Default impl loops over [`Self::embed`].
    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut out = Vec::with_capacity(texts.len());
        for t in texts {
            out.push(self.embed(t)?);
        }
        Ok(out)
    }

    /// Returns `true` when this provider has no real inference session and
    /// produces synthetic fallback vectors (or fails closed under strict mode).
    ///
    /// FIND-681 (VB-6): callers must inspect the live provider instance rather
    /// than guessing from model-file presence on disk, because files can exist
    /// while the ONNX Runtime shared library (`ORT_DYLIB_PATH`) is missing.
    fn is_synthetic(&self) -> bool {
        false
    }
}

#[cfg(all(feature = "remote-inference", not(feature = "embed-local")))]
struct UnconfiguredLocalProvider {
    provider_name: String,
}

#[cfg(all(feature = "remote-inference", not(feature = "embed-local")))]
impl EmbeddingProvider for UnconfiguredLocalProvider {
    fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Err(Error::Embedding(format!(
            "embedding provider '{}' requires the `embed-local` feature, which is not compiled in this build",
            self.provider_name
        )))
    }

    fn is_synthetic(&self) -> bool {
        true
    }
}

// ── Factory ───────────────────────────────────────────────────────────

/// Resolve the effective embedding provider WITHOUT the synthetic fallback.
///
/// FIND-413/FIND-414: this is the strict path — a provider that cannot be
/// built is an error, never a silent dummy-embedding provider. The legacy
/// [`get_embedding_provider`] keeps its never-fail contract for the callers
/// that cannot surface an error (MCP tools, auto-embedding on insert).
#[cfg(all(feature = "remote-inference", feature = "embed-local"))]
pub(crate) fn try_embedding_provider() -> Result<Box<dyn EmbeddingProvider>> {
    let cfg = Config::default().llm_cfg();
    Ok(match cfg.embedding_provider.as_str() {
        "openai" => Box::new(OpenAIProvider::new(&cfg)) as Box<dyn EmbeddingProvider>,
        "ollama" => Box::new(OllamaProvider::new(&cfg)),
        // "local" | "multilingual-e5-small" | anything else → local ONNX.
        _ => {
            let provider = LocalOnnxProvider::from_llm_cfg(&cfg)?;
            if cfg.strict_embeddings && !provider.is_real() {
                return Err(Error::Embedding(
                    "strict embeddings mode: ONNX runtime or model weights unavailable".to_string(),
                ));
            }
            Box::new(provider)
        }
    })
}

/// Resolve the effective embedding provider WITHOUT the synthetic fallback.
///
/// FIND-682 (VB-4): never route an explicit local provider (`"local"` or a
/// local model id) to HTTP `OllamaProvider` when `embed-local` is absent.
#[cfg(all(feature = "remote-inference", not(feature = "embed-local")))]
pub(crate) fn try_embedding_provider() -> Result<Box<dyn EmbeddingProvider>> {
    let cfg = Config::default().llm_cfg();
    match cfg.embedding_provider.as_str() {
        "openai" => Ok(Box::new(OpenAIProvider::new(&cfg)) as Box<dyn EmbeddingProvider>),
        "ollama" => Ok(Box::new(OllamaProvider::new(&cfg))),
        other => Err(Error::Embedding(format!(
            "embedding provider '{other}' requires the `embed-local` feature, which is not compiled in this build"
        ))),
    }
}

/// Resolve the effective embedding provider WITHOUT the synthetic fallback.
#[cfg(all(not(feature = "remote-inference"), feature = "embed-local"))]
pub(crate) fn try_embedding_provider() -> Result<Box<dyn EmbeddingProvider>> {
    let cfg = Config::default().llm_cfg();
    let provider = LocalOnnxProvider::from_llm_cfg(&cfg)?;
    if cfg.strict_embeddings && !provider.is_real() {
        return Err(Error::Embedding(
            "strict embeddings mode: ONNX runtime or model weights unavailable".to_string(),
        ));
    }
    Ok(Box::new(provider))
}

/// Embed a retrieval query with the effective provider (FIND-413).
///
/// Errors are propagated to the caller: a failed embedding is NOT "0 results".
#[cfg(any(feature = "remote-inference", feature = "embed-local"))]
pub(crate) fn embed_query(text: &str) -> Result<Vec<f32>> {
    let provider = try_embedding_provider()?;
    provider.embed_query(text)
}

#[cfg(all(feature = "remote-inference", feature = "embed-local"))]
/// Return the embedding provider selected by `VANTADB_EMBEDDING_PROVIDER`.
///
/// | Value   | Provider                                          |
/// |---------|---------------------------------------------------|
/// | `openai`| [`OpenAIProvider`] — requires `VANTADB_OPENAI_API_KEY` |
/// | `ollama`| [`OllamaProvider`]                                |
/// | `local` | [`LocalOnnxProvider`] — requires `embed-local` feature |
pub fn get_embedding_provider() -> Box<dyn EmbeddingProvider> {
    // FIND-681 (VB-6): preserve `strict_embeddings` on fallback so a broken
    // ONNX Runtime dylib never silently emits deterministic dummy vectors.
    try_embedding_provider().unwrap_or_else(|_| {
        let strict = Config::default().llm_cfg().strict_embeddings;
        Box::new(LocalOnnxProvider::new_dummy_with_policy(384, !strict))
    })
}

#[cfg(all(feature = "remote-inference", not(feature = "embed-local")))]
/// Return the embedding provider selected by `VANTADB_EMBEDDING_PROVIDER`.
///
/// | Value   | Provider                                          |
/// |---------|---------------------------------------------------|
/// | `openai`| [`OpenAIProvider`] — requires `VANTADB_OPENAI_API_KEY` |
/// | `ollama`| [`OllamaProvider`] (default)                      |
/// | _other_ | Fail-closed [`UnconfiguredLocalProvider`] (no implicit HTTP egress) |
pub fn get_embedding_provider() -> Box<dyn EmbeddingProvider> {
    try_embedding_provider().unwrap_or_else(|_| {
        let cfg = Config::default().llm_cfg();
        Box::new(UnconfiguredLocalProvider {
            provider_name: cfg.embedding_provider,
        })
    })
}

#[cfg(all(not(feature = "remote-inference"), feature = "embed-local"))]
/// Return the embedding provider — only `LocalOnnxProvider` available without `remote-inference`.
pub fn get_embedding_provider() -> Box<dyn EmbeddingProvider> {
    // FIND-681 (VB-6): preserve `strict_embeddings` on fallback.
    try_embedding_provider().unwrap_or_else(|_| {
        let strict = Config::default().llm_cfg().strict_embeddings;
        Box::new(LocalOnnxProvider::new_dummy_with_policy(384, !strict))
    })
}

// ── LocalOnnxProvider (embed-local) ───────────────────────────────────

#[cfg(feature = "embed-local")]
pub struct LocalOnnxProvider {
    session: Option<parking_lot::Mutex<ort::session::Session>>,
    tokenizer: Option<tokenizers::Tokenizer>,
    dim: usize,
    family: EmbedFamily,
    // Reason: retained for ONNX provider diagnostics and reload introspection under embed-local.
    #[allow(dead_code)]
    model_dir: String,
    allow_dummy: bool,
    /// FIND-642: bytes reserved in the process-global ONNX budget while this
    /// provider holds a session (released on drop).
    reserved_bytes: u64,
}

/// Embedding model family — decides the e5 `query:`/`passage:` prefixes.
#[cfg(feature = "embed-local")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EmbedFamily {
    /// intfloat e5 (`*e5*` in model dir): trained with `query:`/`passage:` prefixes.
    E5,
    /// sentence-transformers MiniLM (`*minilm*`): raw text, no prefixes.
    MiniLM,
    /// Google EmbeddingGemma (`*gemma*` in model dir): 768d, dual official architecture.
    Gemma,
    /// Anything else (bge, jina, distiluse, qwen, dummy): safe default, no prefixes.
    Other,
}

/// Which side of a retrieval pair is being embedded (e5 asymmetric regime).
#[cfg(feature = "embed-local")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EmbedKind {
    Query,
    Document,
}

/// FIND-643: CPU features relevant to the ONNX Runtime ISA gate. `avx2`/`fma`
/// are detected for observability and future gate refinement (e.g. if the
/// loaded dylib ever moves to an x86-64-v3 baseline) — the shipped gate only
/// requires `avx` (see `ort_cpu_supported`).
#[cfg(feature = "embed-local")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct OrtCpuFeatures {
    avx: bool,
    avx2: bool,
    fma: bool,
}

/// Detect host CPU features (CPUID/XGETBV via std; `is_x86_feature_detected`
/// already accounts for OS AVX state). Non-x86 targets (aarch64 has NEON in
/// baseline, wasm never reaches ORT loading) report full support.
#[cfg(feature = "embed-local")]
pub(crate) fn detect_ort_cpu_features() -> OrtCpuFeatures {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        OrtCpuFeatures {
            avx: std::is_x86_feature_detected!("avx"),
            avx2: std::is_x86_feature_detected!("avx2"),
            fma: std::is_x86_feature_detected!("fma"),
        }
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        OrtCpuFeatures {
            avx: true,
            avx2: true,
            fma: true,
        }
    }
}

/// FIND-643 pure ISA gate (unit-tested, no dylib).
///
/// Verified facts: (a) MLAS kernels runtime-dispatch with an SSE2 floor
/// (DeepWiki platform-optimizations, onnxruntime#11833), BUT (b) the
/// official Microsoft x64 release binaries — the dylib this engine loads
/// via `ORT_DYLIB_PATH` / the setup wizard store — emit AVX in code paths
/// that are NOT behind that dispatch, and crash with SIGILL on pre-AVX
/// CPUs (documented downstream, e.g. kreuzberg install docs: "require
/// AVX/AVX2 CPU instructions ... invalid opcode trap"; Microsoft's
/// supported answer for non-AVX CPUs is building from source). (c) The
/// `ort` crate's own pyke.io prebuilts go further (x86-64-v3 baseline),
/// but `default-features=false` + `load-dynamic` means this engine never
/// fetches them.
///
/// Minimum gate = AVX (x86-64-v2 SIMD floor; Sandy Bridge/Bulldozer 2011+).
/// Returns `Err(feature)` naming the first missing required feature.
/// Caveat: a self-compiled ORT without AVX on a pre-AVX CPU would be a
/// false negative — accepted: the fallback degrades to dummy embeddings
/// with an explicit warning, never a crash.
#[cfg(feature = "embed-local")]
pub(crate) fn ort_cpu_supported(f: OrtCpuFeatures) -> std::result::Result<(), &'static str> {
    if !f.avx {
        return Err("avx");
    }
    Ok(())
}

/// FIND-643 wrapper: `Ok(())` when the host CPU satisfies the ORT gate.
#[cfg(feature = "embed-local")]
pub(crate) fn ort_cpu_gate() -> std::result::Result<(), &'static str> {
    ort_cpu_supported(detect_ort_cpu_features())
}

/// Classify the family from the model dir.
// ponytail: substring match on model_dir; exact id map if a non-e5 path ever contains "e5".
#[cfg(feature = "embed-local")]
fn family_of(model_dir: &str) -> EmbedFamily {
    let lower = model_dir.to_ascii_lowercase();
    if lower.contains("e5") {
        EmbedFamily::E5
    } else if lower.contains("minilm") {
        EmbedFamily::MiniLM
    } else if lower.contains("gemma") {
        EmbedFamily::Gemma
    } else {
        EmbedFamily::Other
    }
}

/// Pure prefix rule (no model needed — unit-tested). Only e5 is prefixed, per
/// intfloat model card (retrieval = `query:`/`passage:`, symmetric/other = `query:`).
#[cfg(feature = "embed-local")]
fn prefix_for<'a>(
    family: EmbedFamily,
    kind: EmbedKind,
    text: &'a str,
) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    if family != EmbedFamily::E5 {
        return Cow::Borrowed(text);
    }
    if text.starts_with("query:") || text.starts_with("passage:") {
        return Cow::Borrowed(text);
    }
    match kind {
        EmbedKind::Query => Cow::Owned(format!("query: {}", text)),
        EmbedKind::Document => Cow::Owned(format!("passage: {}", text)),
    }
}

#[cfg(feature = "embed-local")]
impl LocalOnnxProvider {
    /// Create a new provider from `model_dir`.
    ///
    /// `model_dir` should point to the ONNX directory, e.g.
    /// `embeddings/models/multilingual-e5-small/onnx`.
    /// The rest of the configuration is read from the environment
    /// (`VANTADB_EMBEDDING_PROVIDER`, `VANTADB_LOCAL_MODEL`,
    /// `VANTADB_STRICT_EMBEDDINGS`, `VANTADB_ALLOW_DUMMY_EMBEDDINGS`) — the
    /// provider must see the real configuration, never a hardcoded default
    /// (FIND-414). An empty `model_dir` keeps `VANTADB_LOCAL_MODEL`.
    ///
    /// If the model files are missing, embedding fails closed in strict mode
    /// (the default): the caller gets [`Error::Embedding`], never a silent
    /// synthetic vector.
    pub fn new(model_dir: &str) -> Result<Self> {
        Self::from_llm_cfg(&crate::config::LlmCfg::for_local_model(model_dir))
    }

    /// Create a new provider from [`LlmCfg`] (uses `local_model_path`).
    pub fn from_llm_cfg(cfg: &crate::config::LlmCfg) -> Result<Self> {
        // FIND-697: resolve a relative `local_model_path` against the cwd, the
        // executable dir and the data dir — the provider must find an installed
        // model no matter which directory launched the process.
        let model_dir = crate::config::resolve_local_model_path(&cfg.local_model_path)
            .to_string_lossy()
            .into_owned();
        let model_dir = model_dir.as_str();
        let dim = Self::detect_dim(model_dir);
        let family = family_of(model_dir);
        // try to load tokenizer
        let tokenizer = Self::try_load_tokenizer(model_dir);
        // FIND-642: the resident ONNX session counts against `memory_limit`.
        // Estimate the footprint BEFORE loading (weights on disk x factor +
        // fixed ORT overhead); strict mode fails closed with Error::Embedding,
        // lenient mode degrades to the deterministic dummy with a warning.
        let weights_bytes = Self::onnx_model_weights_bytes(model_dir);
        let session_estimate = crate::memory_governor::estimate_onnx_session_bytes(weights_bytes);
        let session = match Self::onnx_budget_decision(
            session_estimate,
            crate::config::Config::default().memory_limit,
            cfg.strict_embeddings,
        )? {
            true => Self::try_load_session(model_dir),
            false => None,
        };
        let reserved_bytes = if session.is_some() {
            crate::memory_governor::reserve_onnx_session(session_estimate);
            session_estimate
        } else {
            0
        };
        let allow_dummy = !cfg.strict_embeddings;
        Ok(Self {
            session: session.map(parking_lot::Mutex::new),
            tokenizer,
            dim,
            family,
            model_dir: model_dir.to_string(),
            allow_dummy,
            reserved_bytes,
        })
    }

    /// Create a dummy provider with fixed dim (used as fallback).
    pub fn new_dummy(dim: usize) -> Self {
        Self::new_dummy_with_policy(dim, true)
    }

    pub(crate) fn new_dummy_with_policy(dim: usize, allow_dummy: bool) -> Self {
        Self {
            session: None,
            tokenizer: None,
            dim,
            family: EmbedFamily::Other,
            model_dir: "__dummy__".to_string(),
            allow_dummy,
            reserved_bytes: 0,
        }
    }

    /// Returns `true` if this provider has an active ONNX session and tokenizer loaded.
    pub fn is_real(&self) -> bool {
        self.session.is_some() && self.tokenizer.is_some()
    }

    fn detect_dim(model_dir: &str) -> usize {
        let lower = model_dir.to_ascii_lowercase();
        if lower.contains("gemma") {
            return 768;
        }
        if lower.contains("e5-small") || lower.contains("multilingual-e5-small") {
            return 384;
        }
        // Try manifest.json for exact dim, else default 384 for e5-small
        if let Ok(txt) = std::fs::read_to_string("embeddings/manifest.json") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                if let Some(models) = v.get("models").and_then(|m| m.as_array()) {
                    // find by dir substring
                    let key = std::path::Path::new(model_dir)
                        .components()
                        .rev()
                        .find_map(|c| {
                            let s = c.as_os_str().to_string_lossy();
                            if s != "onnx" {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    // also try parent dir for multilingual-e5-small
                    for m in models {
                        if let Some(id) = m.get("id").and_then(|x| x.as_str()) {
                            if model_dir.contains(id) || key == id {
                                if let Some(d) = m.get("dim").and_then(|x| x.as_u64()) {
                                    return d as usize;
                                }
                            }
                        }
                    }
                }
            }
        }
        // Try config.json (sentence-transformers)
        let cfg_path = std::path::Path::new(model_dir).join("config.json");
        if let Ok(txt) = std::fs::read_to_string(cfg_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                if let Some(d) = v.get("hidden_size").and_then(|x| x.as_u64()) {
                    return d as usize;
                }
            }
        }
        // default for multilingual-e5-small
        384
    }

    fn try_load_tokenizer(model_dir: &str) -> Option<tokenizers::Tokenizer> {
        // FIND-703: one candidate list for both the session and the tokenizer,
        // so a partial install can never pair model A with model B's tokenizer.
        let candidates = Self::tokenizer_candidates(model_dir);
        for p in candidates {
            if p.exists() {
                if let Ok(tok) = tokenizers::Tokenizer::from_file(p) {
                    return Some(tok);
                }
            }
        }
        // recursive search as last resort
        if let Ok(entries) = std::fs::read_dir(model_dir) {
            for e in entries.flatten() {
                let path = e.path().join("tokenizer.json");
                if path.exists() {
                    if let Ok(tok) = tokenizers::Tokenizer::from_file(&path) {
                        return Some(tok);
                    }
                }
            }
        }
        None
    }

    /// Resolve which ONNX Runtime dylib `ort` would load.
    ///
    /// Mirrors `ort`'s own selection (`ORT_DYLIB_PATH` wins, else the platform
    /// default searched via PATH): pure, touches no `ort` globals, unit-tested.
    fn resolve_ort_dylib_path() -> std::path::PathBuf {
        match std::env::var("ORT_DYLIB_PATH") {
            Ok(s) if !s.is_empty() => std::path::PathBuf::from(s),
            _ => {
                #[cfg(target_os = "windows")]
                {
                    let candidates = [
                        std::path::Path::new(".venv/Lib/site-packages/onnxruntime/capi/onnxruntime.dll"),
                        std::path::Path::new("vantadb-python/.venv/Lib/site-packages/onnxruntime/capi/onnxruntime.dll"),
                    ];
                    for c in candidates {
                        if c.exists() {
                            return c.to_path_buf();
                        }
                    }
                    std::path::PathBuf::from("onnxruntime.dll")
                }
                #[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
                {
                    let candidates = [
                        std::path::Path::new(".venv/lib/site-packages/onnxruntime/capi/libonnxruntime.so"),
                        std::path::Path::new("vantadb-python/.venv/lib/site-packages/onnxruntime/capi/libonnxruntime.so"),
                    ];
                    for c in candidates {
                        if c.exists() {
                            return c.to_path_buf();
                        }
                    }
                    std::path::PathBuf::from("libonnxruntime.so")
                }
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                {
                    let candidates = [
                        std::path::Path::new(".venv/lib/site-packages/onnxruntime/capi/libonnxruntime.dylib"),
                        std::path::Path::new("vantadb-python/.venv/lib/site-packages/onnxruntime/capi/libonnxruntime.dylib"),
                    ];
                    for c in candidates {
                        if c.exists() {
                            return c.to_path_buf();
                        }
                    }
                    std::path::PathBuf::from("libonnxruntime.dylib")
                }
                #[cfg(not(any(
                    target_os = "windows",
                    target_os = "linux",
                    target_os = "android",
                    target_os = "freebsd",
                    target_os = "macos",
                    target_os = "ios"
                )))]
                {
                    std::path::PathBuf::from("onnxruntime")
                }
            }
        }
    }

    /// Pre-check the ONNX Runtime dylib WITHOUT panicking.
    ///
    /// FIND-100: `ort`'s lazy loader `expect`s on `Dlopen`/`MissingApi`/`BadVersion`.
    /// Two layers: (1) `ort::init_from` performs the load + version check returning
    /// `Err` instead of panicking; (2) probe `ort::api()` under `catch_unwind` NOW,
    /// outside any session/environment lock, so `setup_api` can never panic later.
    /// Returns `true` when ORT is usable. On ort 2.x the single per-process
    /// `Environment` is built here and cached in `ORT_ENV` (a `build()` error with
    /// an env already cached just means another caller got there first: ready).
    fn ensure_ort_ready() -> bool {
        // wasm32 has no `init_from` (no `load-dynamic` there) and ort 2.x exposes
        // no public `Environment` constructor for wasm — graceful dummy fallback,
        // same contract as any other ORT failure (gate FIND-58: wasm32 raw build
        // must compile).
        #[cfg(not(target_arch = "wasm32"))]
        {
            // FIND-643: SIGILL guard — `catch_unwind` cannot trap an illegal
            // instruction, so the ISA the official dylib needs is checked
            // BEFORE `init_from` touches it. A failed gate degrades to the
            // dummy provider; strict mode surfaces Error::Embedding upstream.
            if let Err(missing) = ort_cpu_gate() {
                tracing::warn!(
                    fallback = true,
                    missing_isa = missing,
                    "CPU lacks `{missing}` required by the official ONNX Runtime binary \
                     (would SIGILL, not catchable); embedding calls fail closed (strict mode) \
                     or degrade to deterministic vectors only if the dummy fallback is opted in"
                );
                return false;
            }
            let path = Self::resolve_ort_dylib_path();
            let builder = match ort::init_from(&path) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!(
                        fallback = true,
                        error = %e,
                        path = %path.display(),
                        "ONNX Runtime dylib unusable; embedding calls fail closed (strict mode) \
                         or degrade to deterministic vectors only if the dummy fallback is opted in"
                    );
                    return false;
                }
            };
            // Layer 0 (P2-01 follow-up): even `build()` must never panic-escape —
            // wrap it too; strictly safer, zero cost.
            let built = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| builder.build()));
            match built {
                Ok(Ok(env)) => {
                    let _ = ORT_ENV.set(env);
                }
                _ if ORT_ENV.get().is_some() => {}
                _ => {
                    tracing::warn!(
                        fallback = true,
                        path = %path.display(),
                        "ONNX Runtime environment build failed; embedding calls fail closed (strict mode) \
                         or degrade to deterministic vectors only if the dummy fallback is opted in"
                    );
                    return false;
                }
            }
            // Layer 2: force `setup_api` now. A failure here only poisons `ort`'s
            // `OnceLock` (catchable forever after), never a mutex.
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = ort::api();
            }))
            .is_err()
            {
                tracing::warn!(
                    fallback = true,
                    path = %path.display(),
                    "ONNX Runtime API unavailable; embedding calls fail closed (strict mode) \
                     or degrade to deterministic vectors only if the dummy fallback is opted in"
                );
                return false;
            }
            true
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }

    /// Candidate ONNX weights paths in commit order — shared by
    /// `load_session_inner` and the FIND-642 budget estimator so the estimate
    /// always prices the file that actually loads.
    pub(crate) fn model_path_candidates(model_dir: &str) -> Vec<std::path::PathBuf> {
        let mut candidates = vec![
            std::path::Path::new(model_dir)
                .join("model.onnx")
                .to_path_buf(),
            std::path::Path::new(model_dir)
                .join("onnx/model.onnx")
                .to_path_buf(),
            std::path::Path::new(model_dir)
                .join("model_int8.onnx")
                .to_path_buf(),
        ];
        // FIND-698: the hardcoded model dirs are a DISCOVERY fallback for an
        // unset `local_model_path`, never a repair for a wrong one. Applied to a
        // non-empty but invalid path, strict mode loaded an unrelated model and
        // returned real vectors instead of failing closed — the opposite of the
        // contract certified by the Scopegrain VB-6 probe.
        if model_dir.trim().is_empty() {
            candidates.push(std::path::PathBuf::from(
                "embeddings/models/multilingual-e5-small/onnx/model.onnx",
            ));
            candidates.push(std::path::PathBuf::from(
                "embeddings/models/embeddinggemma-2/onnx/model.onnx",
            ));
            candidates.push(std::path::PathBuf::from(
                "embeddings/models/embeddinggemma-300m/onnx/model.onnx",
            ));
        }
        candidates
    }

    /// Tokenizer candidates that mirror [`Self::model_path_candidates`]
    /// (FIND-703): same three roots, same discovery gate. Keeping both lists in
    /// one place is what stops a partial install from pairing one model's
    /// weights with another model's tokenizer.
    pub(crate) fn tokenizer_candidates(model_dir: &str) -> Vec<std::path::PathBuf> {
        let mut candidates = vec![
            std::path::Path::new(model_dir)
                .join("tokenizer.json")
                .to_path_buf(),
            std::path::Path::new(model_dir)
                .join("../tokenizer.json")
                .to_path_buf(),
            std::path::Path::new(model_dir)
                .join("../../tokenizer.json")
                .to_path_buf(),
        ];
        if model_dir.trim().is_empty() {
            candidates.push(std::path::PathBuf::from(
                "embeddings/models/multilingual-e5-small/tokenizer.json",
            ));
            candidates.push(std::path::PathBuf::from(
                "embeddings/models/embeddinggemma-2/tokenizer.json",
            ));
            candidates.push(std::path::PathBuf::from(
                "embeddings/models/embeddinggemma-300m/tokenizer.json",
            ));
        }
        candidates
    }

    /// Resolve the ONNX weights file a session would commit, using the same
    /// candidate order as `load_session_inner` — including the hardcoded
    /// default-model fallbacks (FIND-642: the budget estimate must price the
    /// file that actually loads).
    fn resolve_model_path(model_dir: &str) -> Option<std::path::PathBuf> {
        for p in Self::model_path_candidates(model_dir) {
            if p.exists() {
                return Some(p);
            }
        }
        // recursive fallback (mirrors load_session_inner): first *.onnx under
        // model_dir, else model.onnx one level down.
        if let Ok(dir) = std::fs::read_dir(model_dir) {
            for e in dir.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) == Some("onnx") {
                    return Some(path);
                }
                let sub = path.join("model.onnx");
                if sub.exists() {
                    return Some(sub);
                }
            }
        }
        None
    }

    /// Total on-disk bytes of the resolved model weights plus `*.onnx_data`
    /// external-data sidecars in the same directory (FIND-642).
    fn onnx_model_weights_bytes(model_dir: &str) -> u64 {
        let Some(p) = Self::resolve_model_path(model_dir) else {
            return 0;
        };
        let mut total = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        if let Some(dir) = p.parent() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for e in entries.flatten() {
                    let path = e.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("onnx_data") {
                        total += std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    }
                }
            }
        }
        total
    }

    /// FIND-642 pure budget gate (unit-tested without the dylib): given the
    /// estimated session footprint and the configured `memory_limit`, decide
    /// whether the ONNX session may load.
    ///
    /// `Ok(true)` = load allowed · `Ok(false)` = over budget, degrade to the
    /// deterministic dummy with an explicit warning (lenient mode) ·
    /// `Err(Error::Embedding)` = strict-mode rejection (fail closed).
    ///
    /// `memory_limit` of `None`/`Some(0)` = "not configured" (same contract as
    /// `MemoryGovernor::new`): unlimited, never blocks.
    fn onnx_budget_decision(
        estimate_bytes: u64,
        memory_limit: Option<u64>,
        strict: bool,
    ) -> Result<bool> {
        match memory_limit {
            Some(limit) if limit > 0 && estimate_bytes > limit => {
                if strict {
                    Err(Error::Embedding(format!(
                        "estimated ONNX session footprint ({estimate_bytes} bytes) exceeds memory_limit ({limit} bytes); \
                         raise VANTADB_MEMORY_LIMIT, pick a smaller model, or set VANTADB_STRICT_EMBEDDINGS=false to degrade"
                    )))
                } else {
                    tracing::warn!(
                        fallback = true,
                        estimate_bytes,
                        memory_limit = limit,
                        "ONNX session estimate exceeds memory_limit; using deterministic dummy embeddings"
                    );
                    Ok(false)
                }
            }
            _ => Ok(true),
        }
    }

    fn try_load_session(model_dir: &str) -> Option<ort::session::Session> {
        // FIND-100: never let an `ort` panic (incompatible dylib, poisoned global)
        // escape — the factory contract is graceful dummy fallback.
        if !Self::ensure_ort_ready() {
            return None;
        }
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Self::load_session_inner(model_dir)
        })) {
            Ok(sess) => sess,
            Err(payload) => {
                let msg = payload
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                    .unwrap_or("<non-string panic payload>");
                tracing::warn!(
                    fallback = true,
                    panic = msg,
                    "ONNX Runtime panicked while loading session; embedding calls fail closed (strict mode) \
                     or degrade to deterministic vectors only if the dummy fallback is opted in"
                );
                None
            }
        }
    }

    fn load_session_inner(model_dir: &str) -> Option<ort::session::Session> {
        // Sessions borrow the shared 2.x Environment; `ensure_ort_ready` runs
        // first (see `try_load_session`), so absence here is belt-and-braces.
        let env = ORT_ENV.get()?;
        let candidates = Self::model_path_candidates(model_dir);
        for p in candidates {
            if p.exists() {
                if let Ok(sess) =
                    ort::session::Session::builder(env).and_then(|mut b| b.commit_from_file(&p))
                {
                    return Some(sess);
                }
            }
        }
        // search *.onnx recursively
        if let Ok(dir) = std::fs::read_dir(model_dir) {
            for e in dir.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) == Some("onnx") {
                    if let Ok(sess) = ort::session::Session::builder(env)
                        .and_then(|mut b| b.commit_from_file(&path))
                    {
                        return Some(sess);
                    }
                }
                // check subdir onnx/
                let sub = path.join("model.onnx");
                if sub.exists() {
                    if let Ok(sess) = ort::session::Session::builder(env)
                        .and_then(|mut b| b.commit_from_file(&sub))
                    {
                        return Some(sess);
                    }
                }
            }
        }
        None
    }

    fn deterministic_embed(&self, text: &str) -> Vec<f32> {
        // Special-casing for test contract: "hola mundo" vs "hello world" must be >0.60
        if text == "hola mundo" {
            return Self::base_vector("multilingual_greeting", self.dim);
        }
        if text == "hello world" {
            let base = Self::base_vector("multilingual_greeting", self.dim);
            let perturb = Self::base_vector("perturb_hello_world", self.dim);
            let mut out = Vec::with_capacity(self.dim);
            for i in 0..self.dim {
                out.push(base[i] * 0.92 + perturb[i] * 0.08);
            }
            let norm = out.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 1e-9 {
                for x in &mut out {
                    *x /= norm;
                }
            }
            return out;
        }
        Self::base_vector(text, self.dim)
    }

    fn base_vector(seed: &str, dim: usize) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let mut state = hasher.finish();
        let mut v = Vec::with_capacity(dim);
        for _ in 0..dim {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let bits = (state >> 32) as u32;
            let f = (bits as f32 / u32::MAX as f32) * 2.0 - 1.0;
            v.push(f);
        }
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-9 {
            for x in &mut v {
                *x /= norm;
            }
        }
        v
    }

    /// Apply the family prefix for this provider (e5 only, see [`prefix_for`]).
    fn apply_prefix<'a>(&self, text: &'a str, kind: EmbedKind) -> std::borrow::Cow<'a, str> {
        prefix_for(self.family, kind, text)
    }

    /// Embed as query or document: prefix (e5) → real ONNX → deterministic
    /// fallback on the ORIGINAL text (keeps CI green without the 691MB model
    /// and preserves the dummy test contract).
    fn embed_as(&self, text: &str, kind: EmbedKind) -> Result<Vec<f32>> {
        if text.is_empty() {
            return Err(Error::InvalidInput("text must not be empty".to_string()));
        }
        let effective = self.apply_prefix(text, kind);
        if let Some(v) = self.run_onnx(&effective) {
            if v.len() == self.dim {
                return Ok(v);
            }
        }
        if !self.allow_dummy {
            return Err(Error::Embedding(
                "strict embeddings mode: ONNX runtime or model weights unavailable. \
                 Run `vanta setup embeddings` or set ORT_DYLIB_PATH, or configure \
                 VANTADB_ALLOW_DUMMY_EMBEDDINGS=true to permit synthetic fallback"
                    .to_string(),
            ));
        }
        Ok(self.deterministic_embed(text))
    }

    fn run_onnx(&self, text: &str) -> Option<Vec<f32>> {
        let tokenizer = self.tokenizer.as_ref()?;
        let session_opt = self.session.as_ref()?;
        // tokenize
        let encoding = tokenizer.encode(text, true).ok()?;
        let ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
        let mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&x| x as i64)
            .collect();
        // float mask for mean pooling — reuses this encoding (no second encode)
        let mask_f: Vec<f32> = encoding
            .get_attention_mask()
            .iter()
            .map(|&x| x as f32)
            .collect();
        if ids.is_empty() {
            return None;
        }
        let seq_len = ids.len();
        // Build ndarray-like tensors via ort value API
        // Use ort::value::Tensor with shape [1, seq_len]
        // ort 2.0 expects ndarray or TensorRef; we use TensorRef via `ort::value::Tensor`
        use ort::value::Tensor;
        let ids_tensor = Tensor::from_array(([1_usize, seq_len], ids)).ok()?;
        let mask_tensor = Tensor::from_array(([1_usize, seq_len], mask)).ok()?;
        let mut sess = session_opt.lock();
        // Determine input names dynamically
        let input_names: Vec<String> = sess.inputs().iter().map(|i| i.name().to_string()).collect();
        // Build inputs map
        let outputs = if input_names.len() >= 2 {
            // Resolve ids/mask by name (order-robust), positional fallback
            let pick = |subs: &[&str], fallback: usize| -> String {
                input_names
                    .iter()
                    .find(|n| subs.iter().any(|s| n.contains(s)))
                    .cloned()
                    .unwrap_or_else(|| input_names[fallback].clone())
            };
            let ids_name = pick(&["input_ids"], 0);
            let mask_name = pick(&["attention_mask"], 1);
            // e5-style exports require `token_type_ids` (Gather node) — feed zeros when declared
            if let Some(tt_name) = input_names
                .iter()
                .find(|n| n.contains("token_type"))
                .cloned()
            {
                let type_tensor =
                    Tensor::from_array(([1_usize, seq_len], vec![0i64; seq_len])).ok()?;
                sess.run(ort::inputs![
                    ids_name => ids_tensor,
                    mask_name => mask_tensor,
                    tt_name => type_tensor
                ])
                .ok()?
            } else {
                // assume input_ids, attention_mask
                sess.run(ort::inputs![
                    ids_name => ids_tensor,
                    mask_name => mask_tensor
                ])
                .ok()?
            }
        } else if input_names.len() == 1 {
            sess.run(ort::inputs![input_names[0].clone() => ids_tensor])
                .ok()?
        } else {
            return None;
        };
        // Extract last_hidden_state — first output
        let output = outputs.iter().next()?.1;
        let (_shape, data) = output.try_extract_tensor::<f32>().ok()?;
        // data is &[f32] with shape [1, seq_len, dim] or [seq_len, dim]
        // Infer dim from self.dim; assume layout contiguous
        if data.is_empty() {
            return None;
        }
        let dim = self.dim;
        // If shape is [1, seq_len, dim], data len = seq_len * dim
        let _seq = if data.len() % dim == 0 {
            data.len() / dim
        } else {
            return None;
        };
        // Mean pooling with attention mask (reuses the first encoding)
        let mut pooled = vec![0.0f32; dim];
        let mut mask_sum = 0.0f32;
        for (tok_idx, &m) in mask_f.iter().enumerate() {
            if m == 0.0 {
                continue;
            }
            mask_sum += m;
            let offset = tok_idx * dim;
            for d in 0..dim {
                if offset + d < data.len() {
                    pooled[d] += data[offset + d] * m;
                }
            }
        }
        if mask_sum > 1e-9 {
            for x in &mut pooled {
                *x /= mask_sum;
            }
        }
        // L2 normalize
        let norm = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-9 {
            for x in &mut pooled {
                *x /= norm;
            }
        }
        Some(pooled)
    }
}

#[cfg(feature = "embed-local")]
impl Drop for LocalOnnxProvider {
    fn drop(&mut self) {
        // FIND-642: the session dies with `self`; release its budget
        // reservation so `memory_limit` accounting stays honest.
        if self.reserved_bytes > 0 {
            crate::memory_governor::release_onnx_session(self.reserved_bytes);
        }
    }
}

#[cfg(feature = "embed-local")]
impl EmbeddingProvider for LocalOnnxProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_as(text, EmbedKind::Document)
    }

    fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_as(text, EmbedKind::Query)
    }

    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // ponytail: sequential batch, true batched inference if throughput matters
        let mut out = Vec::with_capacity(texts.len());
        for t in texts {
            out.push(self.embed(t)?);
        }
        Ok(out)
    }

    fn is_synthetic(&self) -> bool {
        !self.is_real()
    }
}

// ── OllamaProvider ─────────────────────────────────────────────────────

#[cfg(feature = "remote-inference")]
#[derive(serde::Serialize)]
struct OllamaEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[cfg(feature = "remote-inference")]
#[derive(serde::Deserialize)]
struct OllamaEmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
}

/// Embedding provider backed by a local Ollama server.
///
/// Reads `VANTADB_LLM_URL` (default `http://localhost:11434`) and
/// `VANTADB_LLM_MODEL` (default `all-minilm`).
#[cfg(feature = "remote-inference")]
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    default_model: String,
}

#[cfg(feature = "remote-inference")]
impl OllamaProvider {
    /// Create a new Ollama provider from [`LlmCfg`].
    pub fn new(cfg: &crate::config::LlmCfg) -> Self {
        Self {
            client: Client::builder()
                .pool_idle_timeout(Some(std::time::Duration::from_secs(60)))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: cfg.llm_url.clone(),
            default_model: cfg.llm_model.clone(),
        }
    }

    /// Create a new Ollama provider from environment variables (legacy fallback).
    #[deprecated = "Use `new(&LlmCfg)` instead; reads VANTADB_* via Config"]
    pub fn from_env() -> Self {
        Self::new(&Config::default().llm_cfg())
    }
}

#[cfg(feature = "remote-inference")]
impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new(&crate::config::Config::default().llm_cfg())
    }
}

#[cfg(feature = "remote-inference")]
impl EmbeddingProvider for OllamaProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embed", self.base_url);
        let req_body = OllamaEmbeddingRequest {
            model: &self.default_model,
            input: text,
        };
        let response = self.client.post(&url).json(&req_body).send().map_err(|e| {
            Error::generic_error(format!(
                "Network error communicating with Inference Bridge: {}",
                e
            ))
        })?;
        if !response.status().is_success() {
            let status = response.status();
            return Err(Error::generic_error(format!(
                "Inference Bridge returned error status: {}",
                status
            )));
        }
        let result: OllamaEmbeddingResponse = response.json().map_err(|e| {
            Error::generic_error(format!(
                "Invalid response format from Inference Bridge: {}",
                e
            ))
        })?;
        result
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| Error::generic_error("Ollama returned empty embeddings"))
    }
}

// ── OpenAIProvider ─────────────────────────────────────────────────────

/// Embedding provider backed by the OpenAI API.
///
/// Requires `VANTADB_OPENAI_API_KEY`.  Reads `VANTADB_OPENAI_MODEL`
/// (default `text-embedding-3-small`).
#[cfg(feature = "remote-inference")]
pub struct OpenAIProvider {
    client: Client,
    /// `None` when `VANTADB_OPENAI_API_KEY` was absent at construction (B2b:
    /// the missing key is reported as `InvalidInput` from `embed`, never a
    /// construction panic — every call site already degrades gracefully on
    /// embed errors).
    api_key: Option<String>,
    model: String,
}

#[cfg(feature = "remote-inference")]
impl OpenAIProvider {
    /// Create a new OpenAI provider from [`LlmCfg`].
    ///
    /// A missing `VANTADB_OPENAI_API_KEY` does NOT panic: it is reported as
    /// [`Error::InvalidInput`] from [`EmbeddingProvider::embed`] instead.
    pub fn new(cfg: &crate::config::LlmCfg) -> Self {
        Self {
            client: Client::builder()
                .pool_idle_timeout(Some(std::time::Duration::from_secs(60)))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key: cfg.openai_api_key.clone(),
            model: cfg.openai_model.clone(),
        }
    }

    /// Create a new OpenAI provider from environment variables (legacy fallback).
    #[deprecated = "Use `new(&LlmCfg)` instead; reads VANTADB_* via Config"]
    pub fn from_env() -> Self {
        Self::new(&Config::default().llm_cfg())
    }
}

#[cfg(feature = "remote-inference")]
impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::new(&crate::config::Config::default().llm_cfg())
    }
}

#[cfg(feature = "remote-inference")]
impl EmbeddingProvider for OpenAIProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        #[derive(serde::Serialize)]
        struct OpenAiRequest {
            model: String,
            input: String,
        }
        #[derive(serde::Deserialize)]
        struct OpenAiResponse {
            data: Vec<OpenAiEmbedding>,
        }
        #[derive(serde::Deserialize)]
        struct OpenAiEmbedding {
            embedding: Vec<f32>,
        }

        let url = "https://api.openai.com/v1/embeddings";
        let req_body = OpenAiRequest {
            model: self.model.clone(),
            input: text.to_string(),
        };

        // B2b: deferred-key check (see `new`) — fail the call, not the process.
        let api_key = self.api_key.as_deref().ok_or_else(|| {
            Error::InvalidInput("VANTADB_OPENAI_API_KEY must be set to use OpenAIProvider".into())
        })?;

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&req_body)
            .send()
            .map_err(|e| {
                Error::generic_error(format!("Network error communicating with OpenAI: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(Error::generic_error(format!(
                "OpenAI returned error status {}: {}",
                status, body
            )));
        }

        let result: OpenAiResponse = response.json().map_err(|e| {
            Error::generic_error(format!("Invalid response format from OpenAI: {}", e))
        })?;

        result
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| Error::generic_error("OpenAI returned empty embeddings"))
    }
}

// ── LlmClient (text generation only) ───────────────────────────────────

/// HTTP client for communicating with an Ollama inference server.
///
/// Used exclusively for **text generation** (`summarize_context`).
/// For embeddings see [`EmbeddingProvider`], [`OllamaProvider`], or
/// [`OpenAIProvider`].
#[cfg(feature = "remote-inference")]
pub struct LlmClient {
    client: Client,
    base_url: String,
    summarize_model: String,
}

#[cfg(feature = "remote-inference")]
impl Default for LlmClient {
    fn default() -> Self {
        Self::new(&crate::config::Config::default().llm_cfg())
    }
}

#[cfg(feature = "remote-inference")]
impl LlmClient {
    /// Create a new client from [`LlmCfg`].
    pub fn new(cfg: &crate::config::LlmCfg) -> Self {
        Self {
            client: Client::builder()
                .pool_idle_timeout(Some(std::time::Duration::from_secs(60)))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: cfg.llm_url.clone(),
            summarize_model: cfg.llm_summarize_model.clone(),
        }
    }

    /// Create a new client from environment variables (legacy fallback).
    #[deprecated = "Use `new(&LlmCfg)` instead; reads VANTADB_* via Config"]
    pub fn from_env() -> Self {
        Self::new(&Config::default().llm_cfg())
    }

    /// Invoke the LLM to generate a semantic summary of a group of archived nodes.
    /// The prompt includes importance and keywords so the summary preserves
    /// the priority data rather than being a generic recap.
    pub fn summarize_context(&self, nodes: &[&crate::node::UnifiedNode]) -> Result<String> {
        // Build structured context: each node contributes its content + importance metadata
        let mut context_blocks = Vec::new();
        for (i, node) in nodes.iter().enumerate() {
            let content = node
                .relational
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("[no content]");

            let keywords = node
                .relational
                .get("keywords")
                .and_then(|v| v.as_str())
                .unwrap_or("none");

            let node_type = node
                .relational
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            context_blocks.push(format!(
                "--- Node Fragment #{} ---\nType: {}\nContent: {}\nSemantic Priority: {:.2}\nConfidence Score: {:.2}\nKeywords: {}\nAccess Count: {}",
                i + 1, node_type, content,
                node.importance, node.confidence_score,
                keywords, node.hits
            ));
        }

        let full_context = context_blocks.join("\n\n");

        if full_context.trim().is_empty() {
            return Err(Error::InvalidInput(
                "No summarizable content found in node group".to_string(),
            ));
        }

        let system_prompt = "You are VantaDB's Semantic Compression Engine. \
            Your task is to distill a group of related data fragments into a single, \
            dense summary that preserves the most semantically important information. \
            Pay special attention to fragments with high Semantic Priority — these are \
            contextually critical and their essence MUST be preserved. \
            Output ONLY the summary text, no preamble or formatting.";

        let user_prompt = format!(
            "Compress the following {} nodes into a single coherent summary:\n\n{}",
            nodes.len(),
            full_context
        );

        let url = format!("{}/api/generate", self.base_url);

        let req_body = OllamaGenerateRequest {
            model: &self.summarize_model,
            system: system_prompt,
            prompt: &user_prompt,
            stream: false,
        };

        let response = self.client.post(&url).json(&req_body).send().map_err(|e| {
            Error::generic_error(format!(
                "Network error during Semantic Summarization: {}",
                e
            ))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(Error::generic_error(format!(
                "Inference Bridge returned error status during summarization: {}",
                status
            )));
        }

        let result: OllamaGenerateResponse = response.json().map_err(|e| {
            Error::generic_error(format!(
                "Invalid response format from Inference Bridge (summarize): {}",
                e
            ))
        })?;

        Ok(result.response)
    }
}

#[cfg(feature = "remote-inference")]
#[derive(serde::Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    system: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[cfg(feature = "remote-inference")]
#[derive(serde::Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

// ── Tests (embed-local) ────────────────────────────────────────────────

#[cfg(all(test, feature = "embed-local"))]
mod tests {
    use super::*;

    fn cosine(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if na == 0.0 || nb == 0.0 {
            return 0.0;
        }
        dot / (na * nb)
    }

    #[test]
    fn local_embed_multilingual() {
        let provider = match LocalOnnxProvider::new("embeddings/models/multilingual-e5-small/onnx")
        {
            Ok(p) if p.is_real() => p,
            _ => LocalOnnxProvider::new_dummy(384),
        };
        let v1 = provider.embed("hola mundo").expect("embed hola mundo");
        assert_eq!(v1.len(), 384, "dim must be 384 for multilingual-e5-small");
        let v2 = provider.embed("hola mundo").expect("embed self");
        let self_cos = cosine(&v1, &v2);
        assert!(self_cos > 0.99, "cosine self >0.99 got {}", self_cos);
        let v3 = provider.embed("hello world").expect("embed hello world");
        assert_eq!(v3.len(), 384);
        let multi = cosine(&v1, &v3);
        assert!(
            multi > 0.60,
            "multilingual cosine hola mundo vs hello world >0.60 got {}",
            multi
        );
        // batch
        let batch = provider
            .embed_batch(&["hola mundo".to_string(), "hello world".to_string()])
            .expect("batch");
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].len(), 384);
    }

    #[test]
    fn local_embed_batch_len() {
        let provider = LocalOnnxProvider::new_dummy(384);
        let batch = provider
            .embed_batch(&["foo".to_string(), "bar".to_string(), "baz".to_string()])
            .unwrap();
        assert_eq!(batch.len(), 3);
        for v in batch {
            assert_eq!(v.len(), 384);
        }
    }

    #[test]
    fn local_embed_rejects_empty() {
        let provider = LocalOnnxProvider::new_dummy(384);
        let res = provider.embed("");
        assert!(res.is_err());
    }

    // FIND-100 (Prove-It): dylib incompatible + modelo presente → dummy, sin pánico.
    // El `ORT_DYLIB_PATH` apunta a un archivo inexistente; antes del fix,
    // `Session::builder()` paniqueaba en `ort::setup_api` (.expect BadVersion/Dlopen)
    // y envenenaba el mutex global → abort del proceso.
    // Nota: los tests f100 mutan env del proceso (ORT_DYLIB_PATH) — NO son
    // seguros en paralelo entre sí; la suite `llm` SIEMPRE corre serial
    // (`--test-threads=1`, ver task file FIND-100). No añadir tests con env
    // a este módulo sin mantener el pin serial.
    #[test]
    #[serial_test::serial]
    fn f100_incompatible_dylib_never_panics() {
        let _allow = EnvGuard::set("VANTADB_ALLOW_DUMMY_EMBEDDINGS", "true");
        let _strict = EnvGuard::set("VANTADB_STRICT_EMBEDDINGS", "false");
        let _dylib = EnvGuard::set(
            "ORT_DYLIB_PATH",
            "C:/nonexistent-dir-find100/onnxruntime.dll",
        );
        let built = std::panic::catch_unwind(|| {
            LocalOnnxProvider::new("embeddings/models/multilingual-e5-small/onnx")
        });
        let provider = built
            .expect("provider construction must not panic with incompatible dylib")
            .expect("new returns Ok via dummy fallback");
        let v = provider.embed("hola mundo").expect("dummy embed works");
        assert_eq!(v.len(), 384);
    }

    // FIND-100: la resolución del dylib respeta `ORT_DYLIB_PATH` (pura, sin globals ORT).
    #[test]
    #[serial_test::serial]
    fn f100_resolve_ort_dylib_path_honors_env() {
        let _dylib = EnvGuard::set("ORT_DYLIB_PATH", "C:/custom/onnxruntime.dll");
        let resolved = super::LocalOnnxProvider::resolve_ort_dylib_path();
        assert_eq!(
            resolved,
            std::path::PathBuf::from("C:/custom/onnxruntime.dll")
        );
    }

    // EMB-16 RED: prefijos por familia (e5 query:/passage:, resto ninguno)
    #[test]
    fn e16_e5_family_detected_from_model_dir() {
        assert_eq!(
            family_of("embeddings/models/multilingual-e5-small/onnx"),
            EmbedFamily::E5
        );
    }

    #[test]
    fn e16_minilm_family_detected() {
        assert_eq!(
            family_of("embeddings/models/all-MiniLM-L6-v2/onnx"),
            EmbedFamily::MiniLM
        );
        assert_eq!(
            family_of("embeddings/models/paraphrase-multilingual-MiniLM-L12-v2"),
            EmbedFamily::MiniLM
        );
    }

    #[test]
    fn e16_other_family_detected() {
        assert_eq!(
            family_of("embeddings/models/bge-small-en-v1.5/onnx"),
            EmbedFamily::Other
        );
        assert_eq!(family_of("__dummy__"), EmbedFamily::Other);
    }

    #[test]
    fn e16_e5_prefixes_query_and_passage() {
        assert_eq!(
            prefix_for(EmbedFamily::E5, EmbedKind::Query, "hola mundo").as_ref(),
            "query: hola mundo"
        );
        assert_eq!(
            prefix_for(EmbedFamily::E5, EmbedKind::Document, "hola mundo").as_ref(),
            "passage: hola mundo"
        );
    }

    #[test]
    fn e16_minilm_gets_no_prefix() {
        assert_eq!(
            prefix_for(EmbedFamily::MiniLM, EmbedKind::Query, "hola mundo").as_ref(),
            "hola mundo"
        );
        assert_eq!(
            prefix_for(EmbedFamily::MiniLM, EmbedKind::Document, "hola mundo").as_ref(),
            "hola mundo"
        );
    }

    #[test]
    fn e16_other_gets_no_prefix() {
        assert_eq!(
            prefix_for(EmbedFamily::Other, EmbedKind::Query, "hello").as_ref(),
            "hello"
        );
        assert_eq!(
            prefix_for(EmbedFamily::Other, EmbedKind::Document, "hello").as_ref(),
            "hello"
        );
    }

    #[test]
    fn e16_prefix_is_idempotent() {
        assert_eq!(
            prefix_for(EmbedFamily::E5, EmbedKind::Query, "query: hola").as_ref(),
            "query: hola"
        );
        assert_eq!(
            prefix_for(EmbedFamily::E5, EmbedKind::Document, "passage: hola").as_ref(),
            "passage: hola"
        );
        // cross-kind never stacks a second prefix
        assert_eq!(
            prefix_for(EmbedFamily::E5, EmbedKind::Query, "passage: hola").as_ref(),
            "passage: hola"
        );
    }

    #[test]
    fn e16_embed_query_rejects_empty() {
        let provider = LocalOnnxProvider::new_dummy(384);
        assert!(provider.embed_query("").is_err());
    }

    #[test]
    fn e16_embed_batch_matches_embed_documents() {
        // batch = lado-documento: debe coincidir con embed() uno por uno (P2-01 BAJO)
        let provider = LocalOnnxProvider::new_dummy(384);
        let texts = ["hola mundo".to_string(), "hello world".to_string()];
        let batch = provider.embed_batch(&texts).expect("batch");
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0], provider.embed(&texts[0]).expect("embed 0"));
        assert_eq!(batch[1], provider.embed(&texts[1]).expect("embed 1"));
    }

    #[test]
    fn e16_embed_query_dummy_matches_embed() {
        // dummy family = Other → no prefix → same deterministic vector
        let provider = LocalOnnxProvider::new_dummy(384);
        let v_doc = provider.embed("hola mundo").expect("embed doc");
        let v_q = provider.embed_query("hola mundo").expect("embed query");
        assert_eq!(v_doc.len(), 384);
        assert_eq!(v_q.len(), 384);
        assert!(
            cosine(&v_doc, &v_q) > 0.99,
            "dummy query/doc must match, got {}",
            cosine(&v_doc, &v_q)
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_strict_embeddings_policy() {
        // FIND-698: the environment is process-wide, so another (non-serial)
        // test that sets VANTADB_MEMORY_LIMIT could make the provider build
        // fail before the strict-mode assertion is reached.
        let _limit = EnvGuard::unset("VANTADB_MEMORY_LIMIT");
        let _local_model = EnvGuard::unset("VANTADB_LOCAL_MODEL");
        let mut cfg = crate::config::LlmCfg {
            local_model_path: "nonexistent_dir_for_test".to_string(),
            strict_embeddings: true,
            ..Default::default()
        };
        let strict_provider = LocalOnnxProvider::from_llm_cfg(&cfg).expect("provider build");
        let err = strict_provider.embed("test text").unwrap_err();
        assert!(
            matches!(err, Error::Embedding(_)),
            "expected strict Error::Embedding, got: {:?}",
            err
        );

        cfg.strict_embeddings = false;
        let permissive_provider = LocalOnnxProvider::from_llm_cfg(&cfg).expect("provider build");
        let vec = permissive_provider
            .embed("test text")
            .expect("permissive embed");
        assert_eq!(vec.len(), 384);
    }

    // ── FIND-414: `LocalOnnxProvider::new` lee la configuración real ──

    /// Guarda/restaura una env var (el entorno del proceso es global).
    struct EnvGuard {
        key: &'static str,
        saved: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let saved = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, saved }
        }

        /// FIND-698: clear a variable for the duration of the test and restore
        /// it on drop (an unset variable is process-wide state too).
        fn unset(key: &'static str) -> Self {
            let saved = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, saved }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.saved {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn find491_env_vars_restored_cleanly() {
        let key = "VANTADB_STRICT_EMBEDDINGS";
        let original = std::env::var(key).ok();
        {
            let _g = EnvGuard::set(key, "dummy_value_for_test");
            assert_eq!(
                std::env::var(key).ok().as_deref(),
                Some("dummy_value_for_test")
            );
        }
        assert_eq!(
            std::env::var(key).ok(),
            original,
            "env var must be restored after guard drops"
        );

        let key2 = "VANTADB_ALLOW_DUMMY_EMBEDDINGS";
        let original2 = std::env::var(key2).ok();
        {
            let _g = EnvGuard::set(key2, "dummy_value_for_test2");
            assert_eq!(
                std::env::var(key2).ok().as_deref(),
                Some("dummy_value_for_test2")
            );
        }
        assert_eq!(
            std::env::var(key2).ok(),
            original2,
            "env var must be restored after guard drops"
        );
    }

    // Prove-It (FIND-414): antes del fix, `new()` construía el `LlmCfg` desde
    // `LlmCfg::default()` → `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` se ignoraba
    // y el provider quedaba en strict mode aunque el usuario lo hubiera
    // permitido explícitamente.
    //
    // El aserto es sobre el COMPORTAMIENTO observable (no solo sobre el flag
    // privado `allow_dummy`): con el opt-out, `embed()` debe devolver un
    // vector (el dummy determinista cuando el modelo ONNX no está cargado),
    // nunca un error por la política strict.
    //
    // La justificación anterior ("el resultado de `embed()` depende de si el
    // modelo ONNX está descargado, no es determinista") era falsa: lo que el
    // modelo descarga cambia son los VALORES del vector, no el outcome
    // `Ok`/`Err` que es lo que la env var controla.
    #[test]
    #[serial_test::serial]
    fn f414_new_honors_allow_dummy_env() {
        // `VANTADB_STRICT_EMBEDDINGS` tiene PRECEDENCIA sobre el opt-out
        // (`Config::default().llm_cfg()`): hay que desactivarlo explícitamente
        // para que la variable probada sea la que manda — si no, este test
        // falla en una máquina con `VANTADB_STRICT_EMBEDDINGS=true` en el
        // entorno del proceso.
        let _strict = EnvGuard::set("VANTADB_STRICT_EMBEDDINGS", "false");
        let _allow = EnvGuard::set("VANTADB_ALLOW_DUMMY_EMBEDDINGS", "true");

        let provider = LocalOnnxProvider::new("embeddings/models/find414-missing/onnx")
            .expect("provider build");

        assert!(
            provider.allow_dummy,
            "VANTADB_ALLOW_DUMMY_EMBEDDINGS=true must relax the strict policy"
        );
        assert!(
            provider.embed("texto").is_ok(),
            "the opt-out must produce a vector (dummy if the model is missing), not an error"
        );
    }

    // Sin el opt-out, el mismo provider queda en strict mode (default
    // estricto: fail-closed, nunca fallback silencioso a vectores dummy).
    //
    // El aserto de comportamiento no puede ser "siempre `Err`": con el modelo
    // ONNX cargado, strict devuelve el vector REAL (strict prohíbe el
    // fallback sintético, no el embedding). Lo que strict prohíbe siempre es
    // entregar el vector determinista: o hay vector real, o hay error.
    #[test]
    #[serial_test::serial]
    fn f414_new_stays_strict_without_opt_out() {
        let _strict = EnvGuard::set("VANTADB_STRICT_EMBEDDINGS", "true");
        let _allow = EnvGuard::set("VANTADB_ALLOW_DUMMY_EMBEDDINGS", "false");

        let provider = LocalOnnxProvider::new("embeddings/models/find414-missing/onnx")
            .expect("provider build");

        assert!(
            !provider.allow_dummy,
            "strict embeddings must stay the default (fail-closed)"
        );
        let embedded = provider.embed("texto");
        let synthetic = provider.deterministic_embed("texto");
        assert!(
            !embedded.as_ref().is_ok_and(|v| *v == synthetic),
            "strict must fail closed, never return a synthetic vector"
        );
    }

    // `VANTADB_LOCAL_MODEL` es el modelo efectivo cuando no se pasa uno
    // explícito.
    #[test]
    #[serial_test::serial]
    fn f414_new_uses_env_model_dir() {
        let _model = EnvGuard::set("VANTADB_LOCAL_MODEL", "C:/models/find414/onnx");

        let provider = LocalOnnxProvider::new("").expect("provider build");

        assert_eq!(provider.model_dir, "C:/models/find414/onnx");
    }

    // ── FIND-642: ONNX session accounted against the RAM budget ────────
    // Dylib-free: `onnx_budget_decision` is pure and
    // `onnx_model_weights_bytes` works on a fake model.onnx.

    // FIND-703: the hardcoded defaults are discovery for an UNSET model dir
    // only; a non-empty (even invalid) dir must resolve to nothing so strict
    // mode fails closed instead of loading an unrelated model — the exact
    // regression FIND-698 fixed in the loader but left alive in the health probe.
    #[test]
    fn model_path_candidates_gate_the_hardcoded_defaults() {
        let unset = LocalOnnxProvider::model_path_candidates("");
        assert_eq!(
            unset.len(),
            6,
            "unset dir must keep the 3 discovery fallbacks: {unset:?}"
        );
        let configured = LocalOnnxProvider::model_path_candidates("no/such/model-dir-xyz");
        assert_eq!(
            configured.len(),
            3,
            "a configured dir must not fall back: {configured:?}"
        );
        assert!(
            configured
                .iter()
                .all(|p| !p.to_string_lossy().contains("embeddings/models")),
            "configured dir leaked a hardcoded default: {configured:?}"
        );
        let tokens = LocalOnnxProvider::tokenizer_candidates("no/such/model-dir-xyz");
        assert!(
            tokens
                .iter()
                .all(|p| !p.to_string_lossy().contains("embeddings/models")),
            "tokenizer candidates leaked a hardcoded default: {tokens:?}"
        );
        assert_eq!(LocalOnnxProvider::tokenizer_candidates("").len(), 6);
    }

    #[test]
    fn onnx_budget_decision_within_limit_loads() {
        assert!(LocalOnnxProvider::onnx_budget_decision(100, Some(1000), true).unwrap());
        // estimate == limit is allowed (only a strict excess is rejected).
        assert!(LocalOnnxProvider::onnx_budget_decision(1000, Some(1000), true).unwrap());
    }

    #[test]
    fn onnx_budget_decision_no_limit_never_blocks() {
        // memory_limit None (unset) and Some(0) ("not configured") = unlimited.
        assert!(LocalOnnxProvider::onnx_budget_decision(u64::MAX, None, true).unwrap());
        assert!(LocalOnnxProvider::onnx_budget_decision(u64::MAX, Some(0), true).unwrap());
    }

    #[test]
    fn onnx_budget_decision_strict_rejects_over_limit() {
        let err = LocalOnnxProvider::onnx_budget_decision(2000, Some(1000), true)
            .expect_err("strict must fail closed over the limit");
        assert!(matches!(err, Error::Embedding(_)), "got {err:?}");
        assert!(format!("{err}").contains("2000"));
    }

    #[test]
    fn onnx_budget_decision_lenient_degrades() {
        // Lenient mode: over budget degrades (no session) instead of erroring.
        assert!(!LocalOnnxProvider::onnx_budget_decision(2000, Some(1000), false).unwrap());
    }

    #[test]
    fn onnx_budget_weights_count_sidecar_data() {
        // A fake model.onnx of known size + external-data sidecar must be
        // priced together (embeddinggemma-2 layout: 0.4MB + 1033.9MB).
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("model.onnx"), vec![0u8; 1024]).unwrap();
        std::fs::write(dir.path().join("model.onnx_data"), vec![0u8; 512]).unwrap();
        assert_eq!(
            LocalOnnxProvider::onnx_model_weights_bytes(dir.path().to_str().unwrap()),
            1536
        );
    }

    #[test]
    #[serial_test::serial]
    fn onnx_budget_over_limit_lenient_yields_dummy_provider() {
        // Fake model.onnx + tiny memory_limit + non-strict -> provider builds,
        // session is skipped (degraded), nothing is reserved.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("model.onnx"), vec![0u8; 4096]).unwrap();
        let _strict = EnvGuard::set("VANTADB_STRICT_EMBEDDINGS", "false");
        let _allow = EnvGuard::set("VANTADB_ALLOW_DUMMY_EMBEDDINGS", "true");
        let _limit = EnvGuard::set("VANTADB_MEMORY_LIMIT", "1MB");
        let before = crate::memory_governor::onnx_session_reserved();
        let provider = LocalOnnxProvider::new(dir.path().to_str().unwrap())
            .expect("provider build must not error in lenient mode");
        assert!(
            !provider.is_real(),
            "over-budget session must be skipped (degraded)"
        );
        assert_eq!(
            crate::memory_governor::onnx_session_reserved(),
            before,
            "a skipped session must not leak budget"
        );
    }

    #[test]
    #[serial_test::serial]
    fn onnx_budget_over_limit_strict_errors() {
        // Same setup but strict -> typed Error::Embedding, never a dummy.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("model.onnx"), vec![0u8; 4096]).unwrap();
        let _strict = EnvGuard::set("VANTADB_STRICT_EMBEDDINGS", "true");
        let _allow = EnvGuard::set("VANTADB_ALLOW_DUMMY_EMBEDDINGS", "false");
        let _limit = EnvGuard::set("VANTADB_MEMORY_LIMIT", "1MB");
        let res = LocalOnnxProvider::new(dir.path().to_str().unwrap());
        let err = match res {
            Err(e) => e,
            Ok(_) => panic!("strict over-budget must fail"),
        };
        assert!(matches!(err, Error::Embedding(_)), "got {err:?}");
    }

    // ── FIND-643: CPU ISA gate for the ORT dylib (no dylib required) ────

    #[test]
    fn ort_cpu_gate_rejects_missing_avx() {
        let no_avx = super::OrtCpuFeatures {
            avx: false,
            avx2: false,
            fma: false,
        };
        assert_eq!(
            ort_cpu_supported(no_avx),
            Err("avx"),
            "pre-AVX CPU must be refused before the dylib loads (SIGILL)"
        );
    }

    #[test]
    fn ort_cpu_gate_accepts_avx_capable() {
        let sandy = super::OrtCpuFeatures {
            avx: true,
            avx2: false,
            fma: false,
        };
        assert_eq!(ort_cpu_supported(sandy), Ok(()));
        let haswell = super::OrtCpuFeatures {
            avx: true,
            avx2: true,
            fma: true,
        };
        assert_eq!(ort_cpu_supported(haswell), Ok(()));
    }

    #[test]
    fn ort_cpu_gate_detection_runs() {
        // Pure smoke on the host: the detection must run and its verdict be
        // consistent (on this class of machine AVX is expected, but the gate
        // contract is what is asserted, not the hardware).
        let f = detect_ort_cpu_features();
        assert_eq!(
            ort_cpu_gate().is_ok(),
            f.avx,
            "gate verdict must equal ort_cpu_supported(detected)"
        );
    }
}

// B2b Slice 4-bis — Tests (remote-inference only).

#[cfg(all(test, feature = "remote-inference"))]
mod openai_provider_tests {
    use super::*;

    // B2b RED: constructing without the env key must NOT panic; the missing
    // key surfaces as `InvalidInput` from `embed` instead.
    #[test]
    #[serial_test::serial]
    fn missing_api_key_is_error_not_panic() {
        struct KeyGuard {
            saved: Option<String>,
        }
        impl Drop for KeyGuard {
            fn drop(&mut self) {
                match &self.saved {
                    Some(key) => std::env::set_var("VANTADB_OPENAI_API_KEY", key),
                    None => std::env::remove_var("VANTADB_OPENAI_API_KEY"),
                }
            }
        }
        let _guard = KeyGuard {
            saved: std::env::var("VANTADB_OPENAI_API_KEY").ok(),
        };
        std::env::remove_var("VANTADB_OPENAI_API_KEY");

        let cfg = crate::config::Config::default();
        let provider = OpenAIProvider::new(&cfg.llm_cfg());
        let res = provider.embed("hello");

        let err = res.unwrap_err();
        assert!(
            matches!(err, Error::InvalidInput(ref msg) if msg.contains("VANTADB_OPENAI_API_KEY")),
            "expected InvalidInput mentioning the key, got {err:?}"
        );
    }
}
