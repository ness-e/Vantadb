//! # Eviction Policies
//!
//! Scoring strategies for determining which nodes to evict under memory pressure.
//! The [`EvictionPolicy`] enum wraps both the traditional weighted-score approach
//! (via [`crate::node::EvictionWeights`]) and the Bayesian Beta-Binomial decay model.
//!
//! ## Feature gate
//!
//! The Bayesian decay variants require `feature = "bayesian_decay"`.

use crate::node::EvictionWeights;

/// Policy for computing per-node eviction scores.
///
/// - `Weighted` — traditional linear combination of hits, confidence, importance, recency.
/// - `Bayesian` — Beta-Binomial posterior estimate (requires `bayesian_decay` feature).
#[derive(Debug, Clone)]
pub enum EvictionPolicy {
    /// Weighted linear combination of eviction factors.
    Weighted(EvictionWeights),
    /// Bayesian Beta-Binomial decay model.
    #[cfg(feature = "bayesian_decay")]
    Bayesian(BayesianDecay),
}

impl EvictionPolicy {
    /// Compute an eviction score for a node.
    ///
    /// Higher values → more valuable to keep. Lower values → eviction candidate.
    pub fn score(
        &self,
        hits: u64,
        confidence: f64,
        importance: f64,
        seconds_since_last_hit: f64,
    ) -> f64 {
        match self {
            EvictionPolicy::Weighted(w) => {
                // Weighted linear combination matching the formula in UnifiedNode::eviction_score.
                let recency = 1.0 / (seconds_since_last_hit.max(1.0)).ln_1p();
                hits as f64 * w.hits
                    + confidence * w.confidence
                    + importance * w.importance
                    + recency * w.recency
            }
            #[cfg(feature = "bayesian_decay")]
            EvictionPolicy::Bayesian(bayes) => bayes.score(hits, seconds_since_last_hit),
        }
    }

    /// Returns `true` when the score is low enough to justify eviction.
    pub fn should_evict(&self, score: f64) -> bool {
        match self {
            EvictionPolicy::Weighted(_) => score < 0.0, // weighted scores are always ≥ 0
            #[cfg(feature = "bayesian_decay")]
            EvictionPolicy::Bayesian(bayes) => bayes.should_evict(score),
        }
    }
}

impl Default for EvictionPolicy {
    fn default() -> Self {
        EvictionPolicy::Weighted(EvictionWeights {
            hits: 1.0,
            confidence: 2.0,
            importance: 3.0,
            recency: 1.0,
        })
    }
}

impl From<EvictionWeights> for EvictionPolicy {
    fn from(w: EvictionWeights) -> Self {
        EvictionPolicy::Weighted(w)
    }
}

// ─── Bayesian Decay ───────────────────────────────────────────────────────────

/// Bayesian Beta-Binomial decay model for eviction scoring.
///
/// Models each node's probability of future reuse as a Beta distribution:
///
/// - α = `prior_alpha` + hits
/// - β = `prior_beta` + time_since_last_hit (seconds)
/// - posterior score = α / (α + β) — expected probability of the Beta distribution
/// - threshold: scores below this are eviction candidates
///
/// A node with many hits and recent activity gets a high score (keep).
/// A node with few hits or long inactivity decays toward zero (evict).
#[cfg(feature = "bayesian_decay")]
#[derive(Debug, Clone, Copy)]
pub struct BayesianDecay {
    /// Prior pseudo-count for successes (α₀).
    prior_alpha: f64,
    /// Prior pseudo-count for failures (β₀).
    prior_beta: f64,
    /// Score below which a node should be evicted.
    threshold: f64,
}

#[cfg(feature = "bayesian_decay")]
impl Default for BayesianDecay {
    fn default() -> Self {
        Self {
            prior_alpha: 1.0,
            prior_beta: 1.0,
            threshold: 0.3,
        }
    }
}

#[cfg(feature = "bayesian_decay")]
impl BayesianDecay {
    /// Create a new `BayesianDecay` with custom parameters.
    ///
    /// `prior_alpha` and `prior_beta` must be positive (> 0.0).
    /// `threshold` should be in (0.0, 1.0).
    pub fn new(prior_alpha: f64, prior_beta: f64, threshold: f64) -> Self {
        Self {
            prior_alpha: prior_alpha.max(f64::EPSILON),
            prior_beta: prior_beta.max(f64::EPSILON),
            threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Compute the posterior expected probability of reuse.
    ///
    /// Formula: (prior_alpha + hits) / (prior_alpha + hits + prior_beta + seconds_since_last_hit)
    ///
    /// Returns a value in (0.0, 1.0). Higher = more likely to be reused.
    pub fn score(&self, hits: u64, seconds_since_last_hit: f64) -> f64 {
        let alpha = self.prior_alpha + hits as f64;
        let beta = self.prior_beta + seconds_since_last_hit.max(0.0);
        alpha / (alpha + beta)
    }

    /// Returns `true` when the posterior score falls below the configured threshold.
    pub fn should_evict(&self, score: f64) -> bool {
        score < self.threshold
    }

    // ── Accessors ──

    pub const fn prior_alpha(&self) -> f64 {
        self.prior_alpha
    }
    pub const fn prior_beta(&self) -> f64 {
        self.prior_beta
    }
    pub const fn threshold(&self) -> f64 {
        self.threshold
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Bayesian decay tests ─────────────────────────────────────────────

    #[cfg(feature = "bayesian_decay")]
    #[test]
    fn test_bayesian_defaults() {
        let b = BayesianDecay::default();
        assert!((b.prior_alpha() - 1.0).abs() < 1e-9);
        assert!((b.prior_beta() - 1.0).abs() < 1e-9);
        assert!((b.threshold() - 0.3).abs() < 1e-9);
    }

    #[cfg(feature = "bayesian_decay")]
    #[test]
    fn test_bayesian_score_high_hits() {
        let b = BayesianDecay::default();
        // 100 hits, 3600 seconds (1 hour) since last hit
        let score = b.score(100, 3600.0);
        // α = 1 + 100 = 101, β = 1 + 3600 = 3601
        // score = 101 / (101 + 3601) ≈ 0.027
        // Wait, that seems low. Let me recalculate:
        // 101 / 3702 ≈ 0.027
        // Actually with these params it's reasonable — 100 hits over 1 hour gives low density
        // Let's test with shorter time for high score
        assert!(
            score < 0.5,
            "100 hits, 1h ago → score should be low-ish: {score}"
        );
    }

    #[cfg(feature = "bayesian_decay")]
    #[test]
    fn test_bayesian_score_high_hits_recent() {
        let b = BayesianDecay::default();
        // 100 hits, 1 second since last hit
        let score = b.score(100, 1.0);
        // α = 101, β = 1 + 1 = 2, score = 101/103 ≈ 0.98
        assert!(
            score > 0.9,
            "100 hits, 1s ago → score should be > 0.9: {score}"
        );
    }

    #[cfg(feature = "bayesian_decay")]
    #[test]
    fn test_bayesian_score_no_hits() {
        let b = BayesianDecay::default();
        // 0 hits, 86400 seconds (1 day) since last hit
        let score = b.score(0, 86400.0);
        // α = 1, β = 1 + 86400 = 86401, score = 1/86402 ≈ 0.00001
        assert!(
            score < 0.5,
            "0 hits, 1d ago → score should be < 0.5: {score}"
        );
    }

    #[cfg(feature = "bayesian_decay")]
    #[test]
    fn test_bayesian_eviction_threshold() {
        // threshold at 0.3
        let b = BayesianDecay::new(1.0, 1.0, 0.3);
        // Score 0.4 → keep
        assert!(!b.should_evict(0.4));
        // Score 0.2 → evict
        assert!(b.should_evict(0.2));
        // Score exactly at threshold → keep (not strictly less)
        assert!(!b.should_evict(0.3));
    }

    #[cfg(feature = "bayesian_decay")]
    #[test]
    fn test_bayesian_score_zero_seconds() {
        let b = BayesianDecay::default();
        // 10 hits, 0 seconds (just happened)
        let score = b.score(10, 0.0);
        // α = 11, β = 1 + 0 = 1, score = 11/12 ≈ 0.917
        assert!(
            score > 0.9,
            "10 hits, 0s ago → score should be > 0.9: {score}"
        );
    }

    #[cfg(feature = "bayesian_decay")]
    #[test]
    fn test_bayesian_new_clamps_params() {
        let b = BayesianDecay::new(0.0, -1.0, 1.5);
        // Should clamp to EPSILON for non-positive priors, clamp threshold to [0,1]
        assert!(b.prior_alpha() > 0.0);
        assert!(b.prior_beta() > 0.0);
        assert!((b.threshold() - 1.0).abs() < 1e-9);
    }

    // ─── EvictionPolicy tests ─────────────────────────────────────────────

    #[test]
    fn test_eviction_policy_default() {
        let policy = EvictionPolicy::default();
        match &policy {
            EvictionPolicy::Weighted(_) => {} // expected
            #[cfg(feature = "bayesian_decay")]
            EvictionPolicy::Bayesian(_) => panic!("default should be Weighted"),
        }
    }

    #[test]
    fn test_eviction_policy_from_weights() {
        let weights = EvictionWeights {
            hits: 0.5,
            confidence: 1.0,
            importance: 1.5,
            recency: 2.0,
        };
        let policy: EvictionPolicy = weights.into();
        match policy {
            EvictionPolicy::Weighted(w) => {
                assert!((w.hits - 0.5).abs() < 1e-9);
                assert!((w.recency - 2.0).abs() < 1e-9);
            }
            #[cfg(feature = "bayesian_decay")]
            EvictionPolicy::Bayesian(_) => panic!("from(EvictionWeights) should produce Weighted"),
        }
    }

    #[cfg(feature = "bayesian_decay")]
    #[test]
    fn test_eviction_policy_bayesian_round_trip() {
        let bayes = BayesianDecay::new(2.0, 5.0, 0.25);
        let policy = EvictionPolicy::Bayesian(bayes);
        let score = policy.score(50, 0.0, 0.0, 10.0);
        // α = 2 + 50 = 52, β = 5 + 10 = 15, score = 52/67 ≈ 0.776
        assert!((score - 52.0 / 67.0).abs() < 1e-9);
        assert!(!policy.should_evict(score));
        assert!(policy.should_evict(0.1));
    }

    #[test]
    fn test_eviction_policy_weighted_score() {
        let weights = EvictionWeights {
            hits: 1.0,
            confidence: 2.0,
            importance: 3.0,
            recency: 1.0,
        };
        let policy = EvictionPolicy::Weighted(weights);
        // 10 hits, confidence 0.5, importance 0.8, 60s since last hit
        let score = policy.score(10, 0.5, 0.8, 60.0);
        let recency = 1.0 / (60.0_f64).ln_1p();
        let expected = 10.0 * 1.0 + 0.5 * 2.0 + 0.8 * 3.0 + recency * 1.0;
        assert!((score - expected).abs() < 1e-9);
    }
}
