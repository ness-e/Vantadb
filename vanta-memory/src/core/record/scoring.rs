//! MEMG-21 — Composite L1 scoring: recency + relevance + importance.
//!
//! Opt-in re-ranking of recall candidates on top of the existing dual-pool
//! relevance machinery (D38: keyword overlap + cosine fused via RRF):
//!
//! ```text
//! composite = w_rel · relevance_norm + w_rec · recency + w_imp · importance
//! ```
//!
//! - **relevance_norm** — the caller's raw pool score (keyword overlap or
//!   cosine) or the fused RRF score when both pools contribute, min-max
//!   normalized over the candidate set (Park et al. §4.1: "normalize the
//!   recency, relevance, and importance scores … using min-max scaling").
//!   RRF is rank-based and scale-free, so heterogeneous arms never compete
//!   on raw scales.
//! - **recency** — MEMG-07's [`retention_factor`] (`2^(−age/half_life)`,
//!   `age` since `updated_at`), consumed as-is. CrewAI's composite `decay =
//!   0.5^(age_days/half_life_days)` is the same exponential family.
//! - **importance** — the record's declared `priority` (0-100) → `[0,1]`;
//!   `priority < 0` (strict global instruction) → `1.0`. Set at encoding
//!   time, mirroring Park ("generated at the time the memory object is
//!   created") and CrewAI ("set at encoding time").
//!
//! Sources (policy, not calibration — N-09):
//! - Park et al., *Generative Agents* — arXiv:2304.03442v2 §4.1 (weighted
//!   combination of recency/importance/relevance with min-max scaling; all
//!   α = 1 in their implementation).
//! - CrewAI unified memory docs §Composite Scoring
//!   (<https://docs.crewai.com/en/concepts/memory>) — exact weighted form and
//!   declared defaults `semantic 0.5 / recency 0.3 / importance 0.2`.
//!
//! The defaults below are declared policy, tunable per deployment; they carry
//! no calibrated-accuracy claim.

use serde::{Deserialize, Serialize};

use crate::core::abstractions::MemoryRecord;
use crate::core::record::lifecycle::{retention_factor, DecayPolicy};

/// Declared weights of the composite score, in per-mille (integer: the config
/// stays `Eq`-comparable — no floats in policy structs).
///
/// Defaults mirror CrewAI's documented composite-scoring defaults
/// (`semantic_weight=0.5`, `recency_weight=0.3`, `importance_weight=0.2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoringWeights {
    /// Weight of the (min-max normalized) relevance signal.
    pub relevance: u16,
    /// Weight of the recency signal (MEMG-07 retention).
    pub recency: u16,
    /// Weight of the importance signal (declared `priority`).
    pub importance: u16,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            relevance: 500,
            recency: 300,
            importance: 200,
        }
    }
}

/// Opt-in composite-scoring configuration: declared weights + the MEMG-07
/// per-type forgetting policy used for the recency signal.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositeScoring {
    /// Signal weights (per-mille; normalized by their sum at scoring time).
    pub weights: ScoringWeights,
    /// Decay policy backing `recency` (`retention_factor`).
    pub decay: DecayPolicy,
}

/// Importance of a record in `[0,1]` from its declared `priority` (0-100).
///
/// `priority < 0` is the strict-global-instruction sentinel
/// ([`crate::core::abstractions::MemoryType::Instruction`]) → `1.0`: a strict
/// instruction is maximally important. Values above 100 clamp to `1.0`.
pub fn importance_score(record: &MemoryRecord) -> f64 {
    if record.priority < 0 {
        return 1.0;
    }
    f64::from(record.priority.min(100)) / 100.0
}

/// Composite score of one candidate. `relevance` is expected already
/// normalized to `[0,1]` (see [`composite_rank`] for the min-max pass).
///
/// Degenerate weights (all zero) fall back to `relevance` — the most
/// conservative signal, preserving the existing relevance-driven order.
pub fn composite_score(
    record: &MemoryRecord,
    relevance: f64,
    scoring: &CompositeScoring,
    now_ms: u64,
) -> f64 {
    let weights = &scoring.weights;
    let total =
        f64::from(weights.relevance) + f64::from(weights.recency) + f64::from(weights.importance);
    if total <= 0.0 {
        return relevance;
    }
    let recency = retention_factor(record, &scoring.decay, now_ms);
    let importance = importance_score(record);
    (f64::from(weights.relevance) * relevance
        + f64::from(weights.recency) * recency
        + f64::from(weights.importance) * importance)
        / total
}

/// Rank candidates (record + raw relevance) best-first by composite score.
///
/// The raw relevance values are min-max normalized over the candidate set
/// (Park §4.1) before weighting. Ties break deterministically: `updated_at`
/// descending, then `id` ascending. Returns indices into `candidates`.
///
/// # ponytail: full sort per recall pass, opt-in only
/// O(n log n) over the candidate list already held in memory; the legacy path
/// (no scoring) never enters here. Upgrade path: partial select if candidate
/// pools grow past session-sized lists.
pub fn composite_rank(
    candidates: &[(&MemoryRecord, f64)],
    scoring: &CompositeScoring,
    now_ms: u64,
) -> Vec<usize> {
    let relevance = min_max(candidates.iter().map(|(_, score)| *score));
    let mut scored: Vec<(usize, f64)> = candidates
        .iter()
        .zip(&relevance)
        .enumerate()
        .map(|(idx, ((record, _), rel))| (idx, composite_score(record, *rel, scoring, now_ms)))
        .collect();
    scored.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| {
                candidates[b.0]
                    .0
                    .updated_at
                    .cmp(&candidates[a.0].0.updated_at)
            })
            .then_with(|| candidates[a.0].0.id.cmp(&candidates[b.0].0.id))
    });
    scored.into_iter().map(|(idx, _)| idx).collect()
}

/// Min-max normalize to `[0,1]`. All-equal (or single) values → `1.0` each
/// (the whole set is "top of its pool"). Non-finite inputs are ignored for the
/// min/max span and map to `0.0` when a finite span exists; a set with no
/// finite values falls into the all-equal branch → `1.0`.
fn min_max(values: impl Iterator<Item = f64>) -> Vec<f64> {
    let values: Vec<f64> = values.collect();
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for value in values.iter().filter(|v| v.is_finite()) {
        min = min.min(*value);
        max = max.max(*value);
    }
    if !min.is_finite() || !max.is_finite() || max - min <= 0.0 {
        return vec![1.0; values.len()];
    }
    values
        .into_iter()
        .map(|v| {
            if v.is_finite() {
                (v.clamp(min, max) - min) / (max - min)
            } else {
                0.0
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        composite_rank, composite_score, importance_score, CompositeScoring, ScoringWeights,
    };
    use crate::core::abstractions::{MemoryRecord, MemoryType};
    use crate::core::record::lifecycle::DecayPolicy;

    /// Fixed instants as RFC3339 + epoch ms (no chrono in this test module).
    /// `T0` = 2026-01-01T00:00:00.000Z (30d before NOW), `NOW` =
    /// 2026-01-31T00:00:00.000Z, `T60` = 2025-12-02T00:00:00.000Z (60d before).
    const T0: &str = "2026-01-01T00:00:00.000Z";
    const T0_MS: u64 = 1_767_225_600_000;
    const NOW_ISO: &str = "2026-01-31T00:00:00.000Z";
    const NOW_MS: u64 = 1_769_817_600_000;
    const T60: &str = "2025-12-02T00:00:00.000Z";
    const SEVEN_DAYS_MS: u64 = 604_800_000;

    /// Fixture with explicit type/priority/updated_at (age + importance control).
    fn record(id: &str, memory_type: MemoryType, priority: i32, updated_at: &str) -> MemoryRecord {
        MemoryRecord {
            id: id.into(),
            content: format!("content of {id}"),
            memory_type,
            priority,
            scene_name: "s".into(),
            source_message_ids: vec![],
            metadata: serde_json::Value::Null,
            timestamps: vec![updated_at.into()],
            created_at: updated_at.into(),
            updated_at: updated_at.into(),
            version: 1,
            session_key: "sk".into(),
            session_id: "".into(),
            task_id: None,
            team_id: None,
            user_id: None,
            agent_id: None,
            vector: None,
            heat: 0,
            superseded_by: None,
        }
    }

    fn aged(id: &str, priority: i32, updated_at: &str) -> MemoryRecord {
        record(id, MemoryType::Episodic, priority, updated_at)
    }

    // ── declared defaults (policy, not calibration) ──

    #[test]
    fn default_weights_are_declared() {
        let weights = ScoringWeights::default();
        assert_eq!(weights.relevance, 500);
        assert_eq!(weights.recency, 300);
        assert_eq!(weights.importance, 200);
        // CompositeScoring default bundles the declared weights + MEMG-07 policy.
        let scoring = CompositeScoring::default();
        assert_eq!(scoring.weights, weights);
        assert_eq!(scoring.decay, DecayPolicy::default());
    }

    // ── importance signal ──

    #[test]
    fn importance_maps_priority_range() {
        assert!((importance_score(&aged("a", 0, NOW_ISO)) - 0.0).abs() < 1e-12);
        assert!((importance_score(&aged("b", 50, NOW_ISO)) - 0.5).abs() < 1e-12);
        assert!((importance_score(&aged("c", 100, NOW_ISO)) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn importance_strict_instruction_is_max() {
        // priority < 0 = strict global instruction (types.rs) → max importance.
        assert!((importance_score(&aged("a", -1, NOW_ISO)) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn importance_clamps_above_100() {
        assert!((importance_score(&aged("a", 150, NOW_ISO)) - 1.0).abs() < 1e-12);
    }

    // ── composite score: known values ──

    #[test]
    fn composite_score_known_values_zero_age() {
        // relevance 1.0, zero age → recency 1.0, priority 50 → importance 0.5:
        // 0.5·1 + 0.3·1 + 0.2·0.5 = 0.9 (exact, defaults).
        let scoring = CompositeScoring::default();
        let r = aged("a", 50, NOW_ISO);
        let score = composite_score(&r, 1.0, &scoring, NOW_MS);
        assert!((score - 0.9).abs() < 1e-12, "expected 0.9, got {score}");
    }

    #[test]
    fn composite_score_recency_at_one_half_life() {
        // Episodic half-life = 7d: age 7d → recency 0.5 (MEMG-07 consumed).
        // relevance 0.0 + priority 0 → 0.3·0.5 = 0.15.
        let scoring = CompositeScoring::default();
        let r = aged("a", 0, T0);
        let score = composite_score(&r, 0.0, &scoring, T0_MS + SEVEN_DAYS_MS);
        assert!((score - 0.15).abs() < 1e-12, "expected 0.15, got {score}");
    }

    #[test]
    fn composite_score_zero_weights_fall_back_to_relevance() {
        let scoring = CompositeScoring {
            weights: ScoringWeights {
                relevance: 0,
                recency: 0,
                importance: 0,
            },
            ..CompositeScoring::default()
        };
        let r = aged("a", 50, NOW_ISO);
        let score = composite_score(&r, 0.42, &scoring, NOW_MS);
        assert!(
            (score - 0.42).abs() < 1e-12,
            "degenerate weights → relevance"
        );
    }

    // ── composite rank: ordering ──

    #[test]
    fn composite_rank_promotes_recency_and_importance() {
        // Legacy relevance order: A (3.0) > B (2.5) > C (2.0). Composite
        // re-ranks: B (fresh + important) first; C (stale + mid) last.
        let a = aged("a", 10, T0);
        let b = aged("b", 90, NOW_ISO);
        let c = aged("c", 50, T60);
        let candidates = [(&a, 3.0), (&b, 2.5), (&c, 2.0)];
        let order = composite_rank(&candidates, &CompositeScoring::default(), NOW_MS);
        assert_eq!(order, vec![1, 0, 2], "B fresh+important, then A, then C");
    }

    #[test]
    fn composite_rank_min_max_is_scale_invariant() {
        // Same ages/priorities, raw relevance scaled ×10 → same order.
        let a = aged("a", 50, NOW_ISO);
        let b = aged("b", 50, NOW_ISO);
        let c = aged("c", 50, NOW_ISO);
        let scoring = CompositeScoring::default();
        let raw = composite_rank(&[(&a, 10.0), (&b, 5.0), (&c, 0.0)], &scoring, NOW_MS);
        let scaled = composite_rank(&[(&a, 1.0), (&b, 0.5), (&c, 0.0)], &scoring, NOW_MS);
        assert_eq!(raw, vec![0, 1, 2]);
        assert_eq!(raw, scaled);
    }

    #[test]
    fn composite_rank_uniform_signals_preserve_input_order() {
        // Identical age/priority, decreasing relevance in input order → the
        // composite order equals the legacy relevance order (neutrality).
        let a = aged("a", 50, NOW_ISO);
        let b = aged("b", 50, NOW_ISO);
        let c = aged("c", 50, NOW_ISO);
        let order = composite_rank(
            &[(&a, 3.0), (&b, 2.0), (&c, 1.0)],
            &CompositeScoring::default(),
            NOW_MS,
        );
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn composite_rank_ties_are_deterministic() {
        // Same score → updated_at desc, then id asc.
        let old = aged("z", 50, T0);
        let new = aged("a", 50, NOW_ISO);
        let order = composite_rank(
            &[(&old, 1.0), (&new, 1.0)],
            &CompositeScoring::default(),
            NOW_MS,
        );
        assert_eq!(order, vec![1, 0], "newer first on tie");
    }

    #[test]
    fn composite_rank_empty_and_single() {
        let scoring = CompositeScoring::default();
        assert!(composite_rank(&[], &scoring, NOW_MS).is_empty());
        let a = aged("a", 50, NOW_ISO);
        assert_eq!(composite_rank(&[(&a, 1.0)], &scoring, NOW_MS), vec![0]);
    }

    // ── serde round-trip (config surface) ──

    #[test]
    fn composite_scoring_round_trips_serde() {
        let scoring = CompositeScoring::default();
        let json = serde_json::to_string(&scoring).expect("serialize");
        let back: CompositeScoring = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, scoring);
    }
}
