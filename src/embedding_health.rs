//! DEF-08 code-follow-up — visible fallback for local embeddings.
//!
//! Contract: `docs/dev/strategy/DISTRIBUTION.md` §7 and `SPEC.md` L74 —
//! "fallback avisado (`fallback: true`), nunca silencioso". This module is the
//! single probe behind every *status* surface (CLI `status`, MCP
//! `capabilities`, the `setup-embeddings.ps1` live test) so the notice text is
//! never duplicated per binding.
//!
//! Both documented triggers are covered:
//!
//! - **(a) ONNX Runtime unusable** — [`ort::init_from`] loads the dylib and
//!   checks its version, returning `Err` instead of panicking: the exact
//!   pre-check `src/llm.rs:341-419` runs before building a session (FIND-100).
//! - **(b) model files missing** (`model.onnx` + `tokenizer.json`) — today the
//!   provider degrades to deterministic dummy vectors in silence
//!   (`src/llm.rs:206`, `src/llm.rs:526-531`).
//!
//! Approximation (deliberate, documented): files present ⇒ assumed real. A
//! present-but-corrupt model still degrades silently inside the provider;
//! detecting that needs a real inference run, which a status probe must not do.
//!
//! Invariant: this module never mutates `src/llm.rs` state — it calls the same
//! public `ort` API the provider calls and mirrors the provider's dylib
//! resolution (`src/llm.rs:306-339`). `ort::init_from` only fills the same
//! cached dylib handle a real provider run would fill, and its failure mode
//! (plain `Err`, no global poisoning) is idempotent.

use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Remedy printed with every degraded notice (DISTRIBUTION §7).
pub const REMEDY: &str =
    "pwsh setup-embeddings.ps1, then verify: python embeddings/verify.py --check";

/// Snapshot of the local-embedding setup, shaped for status surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingHealth {
    /// Effective provider (`VANTADB_EMBEDDING_PROVIDER`; default `ollama`).
    pub provider: String,
    /// `true` = deterministic dummy embeddings confirmed; `false` = local setup
    /// verified; `null` = not applicable (remote provider, or this build has no
    /// `embed-local` feature) — remote degradation is reported per call (EMB-13).
    pub fallback: Option<bool>,
    /// Degradation trigger, from the DISTRIBUTION §6 vocabulary: `"model"` or
    /// `"dylib"`. `null` when not degraded / not applicable.
    pub reason: Option<String>,
    /// Evidence: missing files or the `ort` load error.
    pub detail: Option<String>,
    /// Model directory probed (local provider only).
    pub model_dir: Option<String>,
    /// Whether `model.onnx` + `tokenizer.json` were found (local provider only).
    pub model_present: Option<bool>,
    /// User-facing notice — present only when `fallback == true`.
    pub notice: Option<String>,
}

/// Probe the effective embedding setup (env-aware, cheap, never panics).
pub fn embedding_health() -> EmbeddingHealth {
    let cfg = Config::default().llm_cfg();
    #[cfg(feature = "embed-local")]
    {
        health_for(&cfg.embedding_provider, &cfg.local_model_path)
    }
    #[cfg(not(feature = "embed-local"))]
    {
        // Local ONNX is not compiled in: nothing to probe (no dummy provider to
        // route to), so the status is "not applicable" rather than a guess.
        not_applicable(
            &cfg.embedding_provider,
            "this build has no `embed-local` feature: local ONNX is not compiled",
        )
    }
}

/// Build the health snapshot for a given provider/model dir (pure routing).
#[cfg(feature = "embed-local")]
fn health_for(provider: &str, model_dir: &str) -> EmbeddingHealth {
    // Mirrors `get_embedding_provider()` feature routing: with `remote-inference`
    // only `openai`/`ollama` are remote; without it, everything routes local.
    if routes_to_local(provider) {
        return local_health(
            provider,
            model_dir,
            model_files_present(model_dir),
            ort_load_error(),
        );
    }
    not_applicable(
        provider,
        &format!(
            "provider '{provider}' is remote: a failed call degrades per call (EMB-13), \
             not to a local dummy provider"
        ),
    )
}

/// Routing mirror of `get_embedding_provider()` (src/llm.rs:57-111).
#[cfg(feature = "embed-local")]
fn routes_to_local(provider: &str) -> bool {
    #[cfg(feature = "remote-inference")]
    {
        !matches!(provider, "openai" | "ollama")
    }
    #[cfg(not(feature = "remote-inference"))]
    {
        let _ = provider;
        true
    }
}

/// Verdict for the local ONNX path from already-probed facts (pure — testable
/// without touching the environment).
#[cfg(feature = "embed-local")]
fn local_health(
    provider: &str,
    model_dir: &str,
    model_present: bool,
    ort_err: Option<String>,
) -> EmbeddingHealth {
    let mut problems = Vec::new();
    if !model_present {
        problems.push(format!(
            "model files missing under '{model_dir}' (model.onnx + tokenizer.json)"
        ));
    }
    if let Some(err) = ort_err.as_deref() {
        problems.push(format!("ONNX Runtime unusable: {err}"));
    }
    let degraded = !problems.is_empty();
    let reason = if degraded {
        // Closed vocabulary (DISTRIBUTION §6): prefer the actionable trigger.
        Some(if model_present { "dylib" } else { "model" }.to_string())
    } else {
        None
    };
    let detail = if degraded {
        Some(problems.join("; "))
    } else {
        None
    };
    EmbeddingHealth {
        provider: provider.to_string(),
        fallback: Some(degraded),
        reason,
        detail: detail.clone(),
        model_dir: Some(model_dir.to_string()),
        model_present: Some(model_present),
        notice: detail.map(|d| build_notice(&d)),
    }
}

/// The notice contract (DISTRIBUTION §7): machine-readable flag + consequence
/// + remedy, in one line.
#[cfg(feature = "embed-local")]
fn build_notice(detail: &str) -> String {
    format!(
        "[vantadb] Local embeddings degraded (fallback: true): {detail}. \
         VantaDB serves deterministic dummy embeddings — recall is NOT semantic \
         until restored. Fix: {REMEDY}"
    )
}

/// "Not applicable" snapshot (remote provider / no `embed-local` build).
fn not_applicable(provider: &str, detail: &str) -> EmbeddingHealth {
    EmbeddingHealth {
        provider: provider.to_string(),
        fallback: None,
        reason: None,
        detail: Some(detail.to_string()),
        model_dir: None,
        model_present: None,
        notice: None,
    }
}

/// `true` when `dir` looks like it can feed a real ONNX session.
///
/// Mirrors the candidate lists of `try_load_tokenizer` / `load_session_inner`
/// (`src/llm.rs:272-304`, `src/llm.rs:421-466`), including their one-level scan
/// over the model dir.
#[cfg(feature = "embed-local")]
fn model_files_present(dir: &str) -> bool {
    use std::path::{Path, PathBuf};

    let base = Path::new(dir);
    let onnx_candidates = [
        base.join("model.onnx"),
        base.join("onnx/model.onnx"),
        base.join("model_int8.onnx"),
        PathBuf::from("embeddings/models/multilingual-e5-small/onnx/model.onnx"),
    ];
    let tokenizer_candidates = [
        base.join("tokenizer.json"),
        base.join("../tokenizer.json"),
        base.join("../../tokenizer.json"),
        PathBuf::from("embeddings/models/multilingual-e5-small/tokenizer.json"),
    ];
    let entries = entries_one_level(dir);
    let has_onnx = onnx_candidates.iter().any(|p| p.exists())
        || entries.iter().any(|p| {
            matches!(p.extension().and_then(|e| e.to_str()), Some("onnx"))
                || p.join("onnx/model.onnx").exists()
        });
    let has_tokenizer = tokenizer_candidates.iter().any(|p| p.exists())
        || entries.iter().any(|p| p.join("tokenizer.json").exists());
    has_onnx && has_tokenizer
}

#[cfg(feature = "embed-local")]
fn entries_one_level(dir: &str) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .map(|entries| entries.flatten().map(|e| e.path()).collect())
        .unwrap_or_default()
}

/// Resolve which dylib `ort` would load — mirror of
/// `LocalOnnxProvider::resolve_ort_dylib_path` (`src/llm.rs:306-339`).
#[cfg(feature = "embed-local")]
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

/// `Some(error)` when `ort` cannot load a usable runtime from `path`.
///
/// Uses `ort::init_from` (public API): it loads + version-checks the dylib and
/// returns `Err` for `Dlopen` / `MissingApi` / `BadVersion` instead of the
/// panic `ort::api()` would raise (FIND-100).
#[cfg(feature = "embed-local")]
fn ort_load_error_at(path: &std::path::Path) -> Option<String> {
    ort::init_from(path).err().map(|e| e.to_string())
}

#[cfg(feature = "embed-local")]
fn ort_load_error() -> Option<String> {
    ort_load_error_at(&resolve_ort_dylib_path())
}

#[cfg(all(test, feature = "embed-local"))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn no_such_dir() -> String {
        std::env::temp_dir()
            .join("vantadb-def08-no-such-model-dir")
            .to_string_lossy()
            .to_string()
    }

    // DEF-08 (b): modelo ausente → aviso presente (flag + consecuencia + remedio).
    #[test]
    fn def08_missing_model_reports_fallback_notice() {
        let health = local_health("local", &no_such_dir(), false, None);
        assert_eq!(health.fallback, Some(true));
        assert_eq!(health.reason.as_deref(), Some("model"));
        assert_eq!(health.model_present, Some(false));
        let notice = health.notice.expect("degraded status must carry a notice");
        assert!(notice.contains("fallback: true"), "notice: {notice}");
        assert!(
            notice.contains("deterministic dummy embeddings"),
            "notice: {notice}"
        );
        assert!(notice.contains("NOT semantic"), "notice: {notice}");
        assert!(notice.contains("setup-embeddings.ps1"), "notice: {notice}");
        assert!(
            notice.contains("embeddings/verify.py --check"),
            "notice: {notice}"
        );
    }

    // DEF-08 (a): ORT no usable → reason dylib + aviso (nunca silencio).
    #[test]
    fn def08_unusable_ort_reports_fallback_notice() {
        let health = local_health(
            "local",
            "embeddings/models/multilingual-e5-small/onnx",
            true,
            Some("failed to load `x`: Dlopen".to_string()),
        );
        assert_eq!(health.fallback, Some(true));
        assert_eq!(health.reason.as_deref(), Some("dylib"));
        let notice = health.notice.expect("degraded status must carry a notice");
        assert!(notice.contains("fallback: true"), "notice: {notice}");
        assert!(notice.contains("ONNX Runtime unusable"), "notice: {notice}");
        assert!(notice.contains("setup-embeddings.ps1"), "notice: {notice}");
    }

    // Setup sano → sin aviso (no false alarms).
    #[test]
    fn def08_healthy_setup_is_not_degraded() {
        let health = local_health("local", "anywhere", true, None);
        assert_eq!(health.fallback, Some(false));
        assert!(health.reason.is_none());
        assert!(health.notice.is_none());
    }

    // Prove-It (patrón f100): dylib inexistente → Err tipado, sin pánico.
    #[test]
    fn def08_ort_probe_never_panics_and_reports_error() {
        let bad = std::path::PathBuf::from("C:/nonexistent-def08/onnxruntime.dll");
        let probed = std::panic::catch_unwind(|| ort_load_error_at(&bad));
        let err = probed.expect("ort probe must not panic on a bad dylib path");
        assert!(err.is_some(), "missing dylib must report an error");
        // Y el veredicto completo sigue siendo un aviso, nunca un pánico.
        let health = local_health("local", &no_such_dir(), false, err);
        assert_eq!(health.fallback, Some(true));
        assert!(health.notice.is_some());
    }

    // Remoto → no aplicable (no se inventa un veredicto local).
    #[test]
    fn def08_remote_provider_is_not_applicable() {
        let health = not_applicable("ollama", "remote provider");
        assert_eq!(health.fallback, None);
        assert!(health.notice.is_none());
    }
}
