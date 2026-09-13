//! Neutral search-profile leaf (C2M3): breaks the `sdk ↔ planner` and
//! `sdk ↔ query` module cycles.
//!
//! `SearchProfileMode` / `SearchProfileConfig` were defined in
//! `sdk::types::search` while `query.rs`, `parser/grammar.rs` and `planner.rs`
//! imported them from `sdk` — and `sdk/search/*` called back into
//! `crate::planner::*`. This leaf owns the profile types plus the RRF /
//! candidate-budget constants so every consumer depends **downward** on a
//! module with zero intra-crate deps (std + serde only).
//!
//! Backward compatibility (BND-04): the old `crate::sdk::...` paths
//! (`SearchProfileConfig`, `SearchProfileMode`) stay alive as `pub use`
//! re-exports for one minor version. Cycle participants import from here
//! directly; `crate::planner::RRF_K` is intentionally NOT re-exported
//! (its sole consumer `api::scores` uses this leaf).

use serde::{Deserialize, Serialize};

/// Reciprocal Rank Fusion smoothing constant (standard literature value: 60).
pub const RRF_K: f32 = 60.0;

/// Multiplier applied to `top_k` to derive the per-arm candidate budget.
pub const CANDIDATE_MULTIPLIER: usize = 4;

/// Minimum candidates fetched per arm in hybrid mode.
pub const MIN_CANDIDATE_BUDGET: usize = 32;

/// Maximum candidates fetched per arm in hybrid mode (guards against
/// unbounded lexical scan at large `top_k`).
pub const MAX_CANDIDATE_BUDGET: usize = 256;

/// Modo de fusión híbrida para un [`SearchProfileConfig`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchProfileMode {
    /// Solo búsqueda BM25 lexical: ignora el vector denso y el sparse.
    Keyword,
    /// Solo búsqueda por similitud vectorial: ignora el texto (mantiene sparse si viene).
    Vector,
    /// Fusión híbrida completa (texto + vector + sparse según los inputs). Default.
    #[default]
    Hybrid,
}

/// Perfil de búsqueda configurable por request/namespace (MEM-01).
///
/// Los campos `None` delegan en las constantes core (`RRF_K`,
/// `hybrid_candidate_budget`) de `sdk::search::fusion`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SearchProfileConfig {
    /// Modo de búsqueda. Default: `Hybrid`.
    #[serde(default)]
    pub mode: SearchProfileMode,
    /// Parámetro `k` de fusión RRF. `None` usa `RRF_K` (60).
    #[serde(default)]
    pub rrf_k: Option<usize>,
    /// Presupuesto de candidatos por canal. `None` usa `hybrid_candidate_budget`.
    #[serde(default)]
    pub candidate_k: Option<usize>,
}
