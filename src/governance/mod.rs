pub mod sleep_worker;
pub mod invalidations;
pub mod thalamic_gate;
pub mod uncertainty;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;

/// A permanent record of a node that has been logically deleted.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditableTombstone {
    pub id: u64,
    pub timestamp_deleted: u64,
    pub reason: String,
    pub original_hash: u64,
}

impl AuditableTombstone {
    pub fn new(id: u64, reason: impl Into<String>, original_hash: u64) -> Self {
        let timestamp_deleted = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            timestamp_deleted,
            reason: reason.into(),
            original_hash,
        }
    }
}

// ─── Soberanía Cognitiva (Devil's Advocate) ────────────────
use crate::node::{UnifiedNode, CognitiveUnit};

#[derive(Debug, Clone)]
pub enum ResolutionResult {
    Accept,
    Reject(String),               // Razón basada en Trust Score o gatekeep
    Superposition(crate::governance::uncertainty::QuantumNeuron), // Zona de incertidumbre con múltiples candidatos (Fase 32B)
    Merge { new_trust: f32 },     // Combinar aserciones bajando certeza
}

pub trait TrustArbiter {
    fn evaluate_conflict(&self, incumbent: &UnifiedNode, challenger: &UnifiedNode) -> ResolutionResult;
}

// ─── Fase 36: Origin Collision Tracker (Barrera Hematoencefálica Semántica) ──

/// Tracks collision counts and trust scores per unique origin (`_owner_role`).
/// Used by DevilsAdvocate to compute the logarithmic friction metric F_ax.
///
/// The axiomatic friction formula is:
///   F_ax = Σ_i [ log2(1 + c_i) × T_i ]
/// where c_i = collision count from origin i, T_i = trust score of origin i.
///
/// A single origin flooding N attacks grows logarithmically → flattened impact.
/// Only a diverse set of trusted origins can breach the axiom threshold.
pub struct OriginCollisionTracker {
    /// Map: owner_role → (collision_count, trust_score_of_origin)
    origins: HashMap<String, (u64, f32)>,
}

impl OriginCollisionTracker {
    pub fn new() -> Self {
        Self {
            origins: HashMap::new(),
        }
    }

    /// Record a collision from a specific origin.
    /// If the origin is new, initializes with the challenger's trust_score.
    /// If existing, increments collision count and updates trust via exponential moving average.
    pub fn record_collision(&mut self, owner_role: &str, challenger_trust: f32) {
        let entry = self.origins.entry(owner_role.to_string()).or_insert((0, challenger_trust));
        entry.0 += 1;
        // EMA blend: 80% existing + 20% new observation (prevents single-shot trust inflation)
        entry.1 = entry.1 * 0.8 + challenger_trust * 0.2;
    }

    /// Compute the Axiomatic Friction F_ax from all recorded collisions.
    ///
    /// F_ax = Σ_i [ log2(1 + c_i) × T_i ]
    ///
    /// Properties:
    /// - Single origin with 1000 collisions: log2(1001) × T ≈ 10 × T
    /// - 10 diverse origins with 10 collisions each: 10 × log2(11) × T ≈ 10 × 3.46 × T = 34.6 × T
    /// - Diversity wins by design.
    pub fn compute_friction(&self) -> f32 {
        self.origins.iter().map(|(_, (count, trust))| {
            ((*count as f32 + 1.0).log2()) * trust
        }).sum()
    }

    /// Returns the number of unique origins that have reported collisions.
    pub fn unique_origins(&self) -> usize {
        self.origins.len()
    }

    /// Slash an origin's trust to 0.0 (Epistemic Apoptosis).
    /// Called by SleepWorker when a hallucination is confirmed.
    pub fn slash_origin(&mut self, owner_role: &str) {
        if let Some(entry) = self.origins.get_mut(owner_role) {
            entry.1 = 0.0;
        } else {
            self.origins.insert(owner_role.to_string(), (0, 0.0));
        }
    }

    /// Check if a specific origin has been slashed (TrustScore == 0.0).
    pub fn is_slashed(&self, owner_role: &str) -> bool {
        self.origins.get(owner_role).map_or(false, |(_, trust)| *trust <= 0.0)
    }

    /// Reset tracker (used between test cycles or on engine restart).
    pub fn reset(&mut self) {
        self.origins.clear();
    }
}

// ─── DevilsAdvocate (Stateful) ─────────────────────────────

/// The cognitive sovereignty arbiter.
///
/// Phase 36: Now stateful — maintains an `OriginCollisionTracker` to compute
/// friction based on origin diversity, preventing single-agent gaslighting.
pub struct DevilsAdvocate {
    pub collision_tracker: RwLock<OriginCollisionTracker>,
}

impl DevilsAdvocate {
    pub fn new() -> Self {
        Self {
            collision_tracker: RwLock::new(OriginCollisionTracker::new()),
        }
    }
}

impl TrustArbiter for DevilsAdvocate {
    fn evaluate_conflict(&self, incumbent: &UnifiedNode, challenger: &UnifiedNode) -> ResolutionResult {
        // ── Phase 36: Hard L1 — Reject slashed agents immediately ──
        let challenger_role = challenger.relational.get("_owner_role")
            .and_then(|v| v.as_str())
            .unwrap_or("anonymous");

        {
            let tracker = self.collision_tracker.read();
            if tracker.is_slashed(challenger_role) {
                return ResolutionResult::Reject(
                    format!("Epistemic Apoptosis: agent '{}' has TrustScore 0.0 (slashed)", challenger_role)
                );
            }
        }

        // ── Vector similarity gate ──
        if let Some(sim) = incumbent.vector.cosine_similarity(&challenger.vector) {
            // Umbral del 95% de similitud para evaluar si hablan del mismo tema
            if sim > 0.95 {
                // ── Phase 36: Axiomatic Iron Wall ──
                // If incumbent is PINNED with high valence, apply logarithmic friction
                if incumbent.is_pinned() && incumbent.semantic_valence >= 0.8 {
                    let mut tracker = self.collision_tracker.write();
                    tracker.record_collision(challenger_role, challenger.trust_score());

                    let friction = tracker.compute_friction();
                    let threshold = incumbent.semantic_valence * 10.0;

                    if friction < threshold {
                        // Insufficient diversity to question axiom
                        return ResolutionResult::Reject(
                            format!(
                                "Barrera Hematoencefálica: Fricción axiomática insuficiente (F_ax={:.2} < threshold={:.2}). Orígenes únicos: {}",
                                friction, threshold, tracker.unique_origins()
                            )
                        );
                    }
                    // If friction >= threshold: diverse consensus reached → Superposition
                }

                // Standard conflict resolution (non-pinned or threshold breached)
                if challenger.trust_score() < incumbent.trust_score() {
                    return ResolutionResult::Superposition(
                        crate::governance::uncertainty::QuantumNeuron::new_superposition(
                            incumbent.clone(),
                            challenger.clone(),
                            10000 // 10s default collapse deadline
                        )
                    );
                }
            }
        }
        ResolutionResult::Accept
    }
}
