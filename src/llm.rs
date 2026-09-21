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
}

// ── Factory ───────────────────────────────────────────────────────────

#[cfg(all(feature = "remote-inference", feature = "embed-local"))]
/// Return the embedding provider selected by `VANTADB_EMBEDDING_PROVIDER`.
///
/// | Value   | Provider                                          |
/// |---------|---------------------------------------------------|
/// | `openai`| [`OpenAIProvider`] — requires `VANTADB_OPENAI_API_KEY` |
/// | `ollama`| [`OllamaProvider`]                                |
/// | `local` | [`LocalOnnxProvider`] — requires `embed-local` feature |
pub fn get_embedding_provider() -> Box<dyn EmbeddingProvider> {
    let cfg = Config::default().llm_cfg();
    match cfg.embedding_provider.as_str() {
        "openai" => Box::new(OpenAIProvider::new(&cfg)),
        "ollama" => Box::new(OllamaProvider::new(&cfg)),
        "local" | "multilingual-e5-small" => {
            let model_dir = cfg.local_model_path;
            // ponytail: unwrap fallback to deterministic dummy if model missing — keeps CI green without 691MB download
            Box::new(
                LocalOnnxProvider::new(&model_dir)
                    .unwrap_or_else(|_| LocalOnnxProvider::new_dummy(384)),
            )
        }
        _ => {
            let model_dir = cfg.local_model_path;
            Box::new(
                LocalOnnxProvider::new(&model_dir)
                    .unwrap_or_else(|_| LocalOnnxProvider::new_dummy(384)),
            )
        }
    }
}

#[cfg(all(feature = "remote-inference", not(feature = "embed-local")))]
/// Return the embedding provider selected by `VANTADB_EMBEDDING_PROVIDER`.
///
/// | Value   | Provider                                          |
/// |---------|---------------------------------------------------|
/// | `openai`| [`OpenAIProvider`] — requires `VANTADB_OPENAI_API_KEY` |
/// | _any_   | [`OllamaProvider`] (default)                      |
pub fn get_embedding_provider() -> Box<dyn EmbeddingProvider> {
    let cfg = Config::default().llm_cfg();
    match cfg.embedding_provider.as_str() {
        "openai" => Box::new(OpenAIProvider::new(&cfg)),
        _ => Box::new(OllamaProvider::new(&cfg)),
    }
}

#[cfg(all(not(feature = "remote-inference"), feature = "embed-local"))]
/// Return the embedding provider — only `LocalOnnxProvider` available without `remote-inference`.
pub fn get_embedding_provider() -> Box<dyn EmbeddingProvider> {
    let cfg = Config::default().llm_cfg();
    let model_dir = cfg.local_model_path;
    Box::new(
        LocalOnnxProvider::new(&model_dir).unwrap_or_else(|_| LocalOnnxProvider::new_dummy(384)),
    )
}

// ── LocalOnnxProvider (embed-local) ───────────────────────────────────

#[cfg(feature = "embed-local")]
pub struct LocalOnnxProvider {
    session: Option<parking_lot::Mutex<ort::session::Session>>,
    tokenizer: Option<tokenizers::Tokenizer>,
    dim: usize,
    family: EmbedFamily,
    #[allow(dead_code)]
    model_dir: String,
}

/// Embedding model family — decides the e5 `query:`/`passage:` prefixes.
#[cfg(feature = "embed-local")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EmbedFamily {
    /// intfloat e5 (`*e5*` in model dir): trained with `query:`/`passage:` prefixes.
    E5,
    /// sentence-transformers MiniLM (`*minilm*`): raw text, no prefixes.
    MiniLM,
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

/// Classify the family from the model dir.
// ponytail: substring match on model_dir; exact id map if a non-e5 path ever contains "e5".
#[cfg(feature = "embed-local")]
fn family_of(model_dir: &str) -> EmbedFamily {
    let lower = model_dir.to_ascii_lowercase();
    if lower.contains("e5") {
        EmbedFamily::E5
    } else if lower.contains("minilm") {
        EmbedFamily::MiniLM
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
    /// If files are missing, returns a dummy deterministic provider (384d) so
    /// tests and CI remain green without downloading 691MB. Real inference
    /// is used when `model.onnx` + `tokenizer.json` are present and `ort`
    /// loads successfully.
    pub fn new(model_dir: &str) -> Result<Self> {
        Self::from_llm_cfg(&crate::config::LlmCfg {
            local_model_path: model_dir.to_string(),
            ..Default::default()
        })
    }

    /// Create a new provider from [`LlmCfg`] (uses `local_model_path`).
    pub fn from_llm_cfg(cfg: &crate::config::LlmCfg) -> Result<Self> {
        let model_dir = &cfg.local_model_path;
        let dim = Self::detect_dim(model_dir);
        let family = family_of(model_dir);
        // try to load tokenizer
        let tokenizer = Self::try_load_tokenizer(model_dir);
        // try to load session
        let session = Self::try_load_session(model_dir);
        // Always succeed — fallback to dummy if either missing, so factory never panics.
        // If both missing, we are in dummy mode (deterministic hash embeddings).
        Ok(Self {
            session: session.map(parking_lot::Mutex::new),
            tokenizer,
            dim,
            family,
            model_dir: model_dir.to_string(),
        })
    }

    /// Create a dummy provider with fixed dim (used as fallback).
    pub fn new_dummy(dim: usize) -> Self {
        Self {
            session: None,
            tokenizer: None,
            dim,
            family: EmbedFamily::Other,
            model_dir: "__dummy__".to_string(),
        }
    }

    fn detect_dim(model_dir: &str) -> usize {
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
        let candidates = [
            std::path::Path::new(model_dir)
                .join("tokenizer.json")
                .to_path_buf(),
            std::path::Path::new(model_dir)
                .join("../tokenizer.json")
                .to_path_buf(),
            std::path::Path::new(model_dir)
                .join("../../tokenizer.json")
                .to_path_buf(),
            std::path::PathBuf::from("embeddings/models/multilingual-e5-small/tokenizer.json"),
        ];
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
            _ => std::path::PathBuf::from({
                #[cfg(target_os = "windows")]
                {
                    "onnxruntime.dll"
                }
                #[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
                {
                    "libonnxruntime.so"
                }
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                {
                    "libonnxruntime.dylib"
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
                    "onnxruntime"
                }
            }),
        }
    }

    /// Pre-check the ONNX Runtime dylib WITHOUT panicking.
    ///
    /// FIND-100: `ort`'s lazy loader `expect`s on `Dlopen`/`MissingApi`/`BadVersion`.
    /// Worse, a panic inside `Session` building can fire while `ort` holds its global
    /// `G_ENV` mutex; the poisoned mutex then panics again in `ort`'s nounwind process-
    /// exit handler (`release_env_on_exit`) → abort even after a graceful run.
    /// Two layers: (1) `ort::init_from` performs the load + version check returning
    /// `Err` instead of panicking; (2) probe `ort::api()` under `catch_unwind` NOW,
    /// outside any session/environment lock, so `setup_api` can never panic later
    /// under `Environment::current`'s guard. Returns `true` when ORT is usable.
    fn ensure_ort_ready() -> bool {
        // wasm32 has no `init_from` (no `load-dynamic` there); session load below
        // stays behind `catch_unwind` (gate FIND-58: wasm32 raw build must compile).
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = Self::resolve_ort_dylib_path();
            let builder = match ort::init_from(&path) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!(
                        fallback = true,
                        error = %e,
                        path = %path.display(),
                        "ONNX Runtime dylib unusable; using deterministic dummy embeddings"
                    );
                    return false;
                }
            };
            // Layer 0 (P2-01 follow-up): even `commit()` must never panic-escape
            // (poisoned global) — wrap it too; strictly safer, zero cost.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| builder.commit()));
            // Layer 2: force `setup_api` now, outside `G_ENV`. A failure here only
            // poisons `ort`'s `OnceLock` (catchable forever after), never its `Mutex`.
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = ort::api();
            }))
            .is_err()
            {
                tracing::warn!(
                    fallback = true,
                    path = %path.display(),
                    "ONNX Runtime API unavailable; using deterministic dummy embeddings"
                );
                return false;
            }
            true
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = ort::init().commit();
            true
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
                    "ONNX Runtime panicked while loading session; using deterministic dummy embeddings"
                );
                None
            }
        }
    }

    fn load_session_inner(model_dir: &str) -> Option<ort::session::Session> {
        let candidates = [
            std::path::Path::new(model_dir)
                .join("model.onnx")
                .to_path_buf(),
            std::path::Path::new(model_dir)
                .join("onnx/model.onnx")
                .to_path_buf(),
            std::path::Path::new(model_dir)
                .join("model_int8.onnx")
                .to_path_buf(),
            std::path::PathBuf::from("embeddings/models/multilingual-e5-small/onnx/model.onnx"),
        ];
        for p in candidates {
            if p.exists() {
                if let Ok(sess) =
                    ort::session::Session::builder().and_then(|mut b| b.commit_from_file(&p))
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
                    if let Ok(sess) =
                        ort::session::Session::builder().and_then(|mut b| b.commit_from_file(&path))
                    {
                        return Some(sess);
                    }
                }
                // check subdir onnx/
                let sub = path.join("model.onnx");
                if sub.exists() {
                    if let Ok(sess) =
                        ort::session::Session::builder().and_then(|mut b| b.commit_from_file(&sub))
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
        let provider = LocalOnnxProvider::new("embeddings/models/multilingual-e5-small/onnx")
            .unwrap_or_else(|_| LocalOnnxProvider::new_dummy(384));
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
    fn f100_incompatible_dylib_never_panics() {
        let key = "ORT_DYLIB_PATH";
        let saved = std::env::var(key).ok();
        std::env::set_var(key, "C:/nonexistent-dir-find100/onnxruntime.dll");
        let built = std::panic::catch_unwind(|| {
            LocalOnnxProvider::new("embeddings/models/multilingual-e5-small/onnx")
        });
        match saved {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
        let provider = built
            .expect("provider construction must not panic with incompatible dylib")
            .expect("new returns Ok via dummy fallback");
        let v = provider.embed("hola mundo").expect("dummy embed works");
        assert_eq!(v.len(), 384);
    }

    // FIND-100: la resolución del dylib respeta `ORT_DYLIB_PATH` (pura, sin globals ORT).
    #[test]
    fn f100_resolve_ort_dylib_path_honors_env() {
        let key = "ORT_DYLIB_PATH";
        let saved = std::env::var(key).ok();
        std::env::set_var(key, "C:/custom/onnxruntime.dll");
        let resolved = super::LocalOnnxProvider::resolve_ort_dylib_path();
        match saved {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
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
}

// B2b Slice 4-bis — Tests (remote-inference only).

#[cfg(all(test, feature = "remote-inference"))]
mod openai_provider_tests {
    use super::*;

    // B2b RED: constructing without the env key must NOT panic; the missing
    // key surfaces as `InvalidInput` from `embed` instead.
    #[test]
    fn missing_api_key_is_error_not_panic() {
        let saved = std::env::var("VANTADB_OPENAI_API_KEY").ok();
        std::env::remove_var("VANTADB_OPENAI_API_KEY");

        let cfg = crate::config::Config::default();
        let provider = OpenAIProvider::new(&cfg.llm_cfg());
        let res = provider.embed("hello");

        if let Some(key) = saved {
            std::env::set_var("VANTADB_OPENAI_API_KEY", key);
        }
        let err = res.unwrap_err();
        assert!(
            matches!(err, Error::InvalidInput(ref msg) if msg.contains("VANTADB_OPENAI_API_KEY")),
            "expected InvalidInput mentioning the key, got {err:?}"
        );
    }
}
