//! Local embedding IPC commands (DESKTOP-EMBED-01).
//!
//! Exposes the [`vantadb::llm::LocalOnnxProvider`] to the frontend so the user
//! can ingest plain text without pasting pre-computed vectors. The provider is
//! gated behind the `embed-local` Cargo feature — when that feature is OFF the
//! command falls back to a deterministic 384-dim dummy that mirrors the
//! `sanity_embed.py` reference behaviour. This keeps the default build lean
//! (no `ort`/`tokenizers` linkage) while letting the desktop ship local
//! embeddings when the operator opts in.
//!
//! Provider lifecycle: a single `LocalOnnxProvider` is cached per
//! `(model_dir)` string and reused across IPC calls so the ort session + HF
//! tokenizer are not re-loaded on every embed. The cache is process-local and
//! grows monotonically (bounded by the number of distinct model dirs the user
//! configures in the Settings page — typically one).
//!
//! # Build variants
//!
//! | Cargo command | Behaviour |
//! |---|---|
//! | `cargo tauri dev` (default) | Dummy provider; UI shows a banner explaining why |
//! | `cargo tauri dev --features embed-local` | Real ONNX inference from `embeddings/models/*` |
//!
//! # IPC contract (DESKTOP-EMBED-01)
//!
//! Request: `{ text: string, model_dir?: string }`
//! Response: `{ vector: number[], dim: number, model: string, source: "real"|"dummy" }`

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::State;

use crate::connections::ConnectionManager;
use crate::error::VantaError;

#[cfg(feature = "embed-local")]
use vantadb::llm::{EmbeddingProvider as _, LocalOnnxProvider};

/// Wire DTO returned by [`vanta_embed_text`].
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingResult {
    /// Dense vector (dummy or real). Length matches `dim`.
    pub vector: Vec<f32>,
    /// Dimensionality of the vector. Real provider: from `manifest.json`.
    /// Dummy: 384 (the default for `multilingual-e5-small`).
    pub dim: usize,
    /// Human-readable model identifier (e.g. `"multilingual-e5-small"`).
    pub model: String,
    /// `"real"` if the ONNX inference actually ran, `"dummy"` if the build was
    /// compiled without `--features embed-local` (or the model files are
    /// missing on disk).
    pub source: &'static str,
}

/// Process-wide cache of loaded ONNX providers, keyed by canonical model id.
///
/// The frontend passes either a path (`"embeddings/models/multilingual-e5-small/onnx"`)
/// or an id (`"multilingual-e5-small"`). The cache stores under both keys so
/// the second call does not re-parse the manifest.
#[derive(Default)]
pub struct EmbeddingCache {
    /// Keyed by canonical model id (e.g. `"multilingual-e5-small"`).
    by_id: Mutex<HashMap<String, Arc<dyn EmbedBackend>>>,
    /// Keyed by absolute path string, for callers that pass a path directly.
    by_path: Mutex<HashMap<PathBuf, Arc<dyn EmbedBackend>>>,
}

/// Backend abstraction so the dummy build keeps compiling.
pub(crate) trait EmbedBackend: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, String>;
    fn dim(&self) -> usize;
    fn model_id(&self) -> &str;
}

// ── Real backend (cfg embed-local) ──────────────────────────────────────

#[cfg(feature = "embed-local")]
struct OnnxBackend {
    provider: LocalOnnxProvider,
    model_id: String,
    dim: usize,
}

#[cfg(feature = "embed-local")]
impl EmbedBackend for OnnxBackend {
    fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        self.provider
            .embed(text)
            .map_err(|e| format!("onnx embed failed: {e}"))
    }
    fn dim(&self) -> usize {
        self.dim
    }
    fn model_id(&self) -> &str {
        &self.model_id
    }
}

// ── Dummy backend (no feature) ───────────────────────────────────────────
// Used only when the desktop is built WITHOUT `--features embed-local`.
// With the feature on, `OnnxBackend` takes the path and these stay
// `dead_code` for the build — hence `#[allow(dead_code)]`.

#[cfg(not(feature = "embed-local"))]
struct DummyBackend {
    dim: usize,
    model_id: String,
}

#[cfg(not(feature = "embed-local"))]
impl EmbedBackend for DummyBackend {
    fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        // Deterministic hash-based vector (mirrors `sanity_embed.py:dummy_embed`).
        // Stable across calls so two ingestions of the same text get the same vector.
        let h = simple_hash(text, self.dim);
        Ok(h)
    }
    fn dim(&self) -> usize {
        self.dim
    }
    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[cfg(not(feature = "embed-local"))]
fn simple_hash(text: &str, dim: usize) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let seed = hasher.finish();
    let mut v: Vec<f32> = (0..dim)
        .map(|i| {
            // mulberry32-style mix of (seed, i) → [-1, 1)
            let mut x = seed.wrapping_add(i as u64).wrapping_mul(0x6D2B79F5);
            x ^= x >> 15;
            x = x.wrapping_mul(0x85EBCA6B);
            x ^= x >> 13;
            x = x.wrapping_mul(0xC2B2AE35);
            x ^= x >> 16;
            (x as f32 / u32::MAX as f32) * 2.0 - 1.0
        })
        .collect();
    // L2 normalize so cosine works downstream.
    let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
    v
}

// ── Model resolution ────────────────────────────────────────────────────

/// Resolve the model directory + canonical id.
///
/// Accepts:
/// - `model_id` (e.g. `"multilingual-e5-small"`) — looked up against
///   `embeddings/manifest.json` (relative to CWD).
/// - absolute or `embeddings/models/<id>/onnx` path — used as-is, the id is
///   the path's parent (i.e. `<id>`).
///
/// Defaults to `multilingual-e5-small` when the caller passes nothing.
fn resolve_model(model: Option<&str>) -> Result<(PathBuf, String, usize), VantaError> {
    // Default: multilingual-e5-small (the project's DEFAULT per manifest.json).
    let requested = model.unwrap_or("multilingual-e5-small");

    // Try as a path first.
    let p = PathBuf::from(requested);
    if p.is_absolute() || p.exists() {
        let dir = if p.ends_with("onnx") {
            p.clone()
        } else {
            p.join("onnx")
        };
        let id = dir
            .components()
            .rev()
            .nth(2) // .../<id>/onnx
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".to_string());
        let dim = detect_dim_or_default(&dir);
        return Ok((dir, id, dim));
    }

    // Otherwise look up by id in manifest.json (relative to CWD).
    let manifest = PathBuf::from("embeddings/manifest.json");
    if let Ok(txt) = std::fs::read_to_string(&manifest) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
            if let Some(models) = v.get("models").and_then(|m| m.as_array()) {
                for m in models {
                    if let Some(id) = m.get("id").and_then(|x| x.as_str()) {
                        if id == requested {
                            let dim = m
                                .get("dim")
                                .and_then(|x| x.as_u64())
                                .map(|x| x as usize)
                                .unwrap_or(384);
                            let dir =
                                PathBuf::from(format!("embeddings/models/{}/onnx", requested));
                            return Ok((dir, requested.to_string(), dim));
                        }
                    }
                }
            }
        }
    }

    Err(VantaError::Other(format!(
        "unknown embedding model: {requested:?}; pass a manifest id (e.g. \
         \"multilingual-e5-small\") or an absolute path to the onnx directory"
    )))
}

fn detect_dim_or_default(dir: &std::path::Path) -> usize {
    if let Ok(txt) = std::fs::read_to_string(dir.join("config.json")) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
            if let Some(d) = v.get("hidden_size").and_then(|x| x.as_u64()) {
                return d as usize;
            }
        }
    }
    384
}

// ── IPC command ─────────────────────────────────────────────────────────

/// Generate a dense embedding vector for `text` using the local ONNX provider.
///
/// `model` is optional: when `None` we resolve `multilingual-e5-small` from
/// `embeddings/manifest.json`. See module docs for the build-variant matrix.
///
/// Frontend usage (TypeScript):
///
/// ```ts
/// import { invoke } from "@tauri-apps/api/core";
/// const { vector, dim, source } = await invoke<EmbeddingResult>("vanta_embed_text", {
///     text: "hola mundo",
///     model: "multilingual-e5-small", // optional
/// });
/// ```
#[tauri::command]
pub fn vanta_embed_text(
    text: String,
    model: Option<String>,
    cache: State<'_, EmbeddingCache>,
) -> Result<EmbeddingResult, VantaError> {
    if text.is_empty() {
        return Err(VantaError::Other("text must be non-empty".into()));
    }
    let (model_dir, model_id, default_dim) = resolve_model(model.as_deref())?;

    // Cache lookup: by path first (exact hit), then by id.
    if let Some(backend) = cache.by_path.lock().unwrap().get(&model_dir).cloned() {
        let vec = backend.embed(&text).map_err(VantaError::Other)?;
        return Ok(EmbeddingResult {
            dim: backend.dim(),
            vector: vec,
            model: backend.model_id().to_string(),
            source: backend_source(backend.as_ref()),
        });
    }

    // Build the backend.
    let backend: Arc<dyn EmbedBackend> = build_backend(&model_dir, &model_id, default_dim)?;
    let result = EmbeddingResult {
        dim: backend.dim(),
        vector: backend.embed(&text).map_err(VantaError::Other)?,
        model: backend.model_id().to_string(),
        source: backend_source(backend.as_ref()),
    };
    cache
        .by_path
        .lock()
        .unwrap()
        .insert(model_dir.clone(), backend.clone());
    cache.by_id.lock().unwrap().insert(model_id, backend);
    Ok(result)
}

#[cfg(feature = "embed-local")]
fn build_backend(
    model_dir: &std::path::Path,
    model_id: &str,
    default_dim: usize,
) -> Result<Arc<dyn EmbedBackend>, VantaError> {
    let dir_str = model_dir.to_string_lossy().to_string();
    let provider = LocalOnnxProvider::new(&dir_str)
        .map_err(|e| VantaError::Other(format!("LocalOnnxProvider::new({dir_str}): {e}")))?;
    // `LocalOnnxProvider::new` always succeeds — it falls back to dummy when
    // the model files are missing. Detect that and surface a clear error to
    // the UI instead of silently serving dummy vectors.
    let onnx_file = model_dir.join("model.onnx");
    if !onnx_file.exists() {
        return Err(VantaError::Other(format!(
            "ONNX model not found at {}; run `python embeddings/download.py --only {model_id}`",
            onnx_file.display()
        )));
    }
    let dim = if default_dim > 0 {
        default_dim
    } else {
        provider_embed_dim().unwrap_or(384)
    };
    Ok(Arc::new(OnnxBackend {
        provider,
        model_id: model_id.to_string(),
        dim,
    }))
}

#[cfg(feature = "embed-local")]
fn provider_embed_dim() -> Option<usize> {
    // `LocalOnnxProvider` sets `dim` internally during `new()` (see
    // `src/llm.rs:detect_dim`). We trust the manifest-resolved `default_dim`
    // passed into `build_backend`; this stub stays for future overrides.
    None
}

#[cfg(not(feature = "embed-local"))]
fn build_backend(
    model_dir: &std::path::Path,
    model_id: &str,
    default_dim: usize,
) -> Result<Arc<dyn EmbedBackend>, VantaError> {
    // Default build: dummy backend. Honest about it via `source = "dummy"` in
    // the wire response so the UI can show a banner.
    let _ = (model_dir, model_id, default_dim);
    Ok(Arc::new(DummyBackend {
        dim: 384,
        model_id: model_id.to_string(),
    }))
}

fn backend_source(backend: &dyn EmbedBackend) -> &'static str {
    // Cheap type-id trick would need Any; keep it explicit via a marker.
    // Real backends always have a loaded session; dummy never. We use the
    // presence of model_dir sentinel as the disambiguator.
    if backend.model_id().is_empty() {
        "dummy"
    } else {
        #[cfg(feature = "embed-local")]
        {
            "real"
        }
        #[cfg(not(feature = "embed-local"))]
        {
            "dummy"
        }
    }
}

// ── Active-connection hint for the UI ───────────────────────────────────

/// Whether the active connection is using `embed-local` for vector search.
/// Surfaced to the UI so it can warn the user when they paste vectors into
/// the IngestForm that don't match the active connection's dim.
#[tauri::command]
pub fn vanta_embed_capabilities(_manager: State<'_, ConnectionManager>) -> serde_json::Value {
    serde_json::json!({
        "embed_local_compiled": cfg!(feature = "embed-local"),
        "default_model": "multilingual-e5-small",
        "manifest": "embeddings/manifest.json",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_is_deterministic() {
        let a = simple_hash("hola mundo", 384);
        let b = simple_hash("hola mundo", 384);
        assert_eq!(a, b);
        // L2 normalized.
        let n: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (n - 1.0).abs() < 1e-4,
            "vector must be L2-normalized, got norm={n}"
        );
    }

    #[test]
    fn dummy_distinguishes_inputs() {
        let a = simple_hash("hola", 16);
        let b = simple_hash("hello", 16);
        assert_ne!(a, b);
    }

    #[test]
    fn resolve_default_model() {
        // No manifest on test cwd? Resolve still returns the default if `embeddings/manifest.json` is reachable
        // — otherwise we get an UnknownModel error. Either branch is fine; the contract is "doesn't panic".
        let _ = resolve_model(None);
    }

    #[test]
    fn build_backend_does_not_panic() {
        // Default build: Dummy succeeds by contract.
        // Real build (embed-local): LocalOnnxProvider::new always succeeds (dummy
        // fallback) but our `build_backend` validates that `model.onnx` exists
        // and errors otherwise. Either branch is acceptable — the test only
        // proves we don't panic.
        let r = build_backend(
            std::path::Path::new("embeddings/models/multilingual-e5-small/onnx"),
            "multilingual-e5-small",
            384,
        );
        // Don't assert ok/err — both are valid. We only care about no-panic.
        drop(r);
    }
}
