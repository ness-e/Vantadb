use crate::governance::ResolutionResult;
use crate::node::{AccessTracker, UnifiedNode};
use parking_lot::RwLock;
use std::collections::HashMap;

// ─── Conflict Resolver (Legacy: Devil's Advocate) ─────────────────────────────

/// Tracks collision counts and confidence scores per unique origin (`_owner_role`).
/// Used by ConflictResolver to compute the logarithmic friction metric F_ax.
pub struct OriginCollisionTracker {
    /// Map: owner_role → (collision_count, confidence_score_of_origin)
    origins: HashMap<String, (u64, f32)>,
}

impl Default for OriginCollisionTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl OriginCollisionTracker {
    pub fn new() -> Self {
        Self {
            origins: HashMap::new(),
        }
    }

    pub fn record_collision(&mut self, owner_role: &str, challenger_confidence: f32) {
        let entry = self
            .origins
            .entry(owner_role.to_string())
            .or_insert((0, challenger_confidence));
        entry.0 += 1;
        entry.1 = entry.1 * 0.8 + challenger_confidence * 0.2;
    }

    pub fn compute_friction(&self) -> f32 {
        self.origins
            .iter()
            .map(|(_, (count, confidence))| ((*count as f32 + 1.0).log2()) * confidence)
            .sum()
    }

    pub fn unique_origins(&self) -> usize {
        self.origins.len()
    }

    pub fn slash_origin(&mut self, owner_role: &str) {
        if let Some(entry) = self.origins.get_mut(owner_role) {
            entry.1 = 0.0;
        } else {
            self.origins.insert(owner_role.to_string(), (0, 0.0));
        }
    }

    pub fn is_slashed(&self, owner_role: &str) -> bool {
        self.origins
            .get(owner_role)
            .is_some_and(|(_, confidence)| *confidence <= 0.0)
    }

    pub fn reset(&mut self) {
        self.origins.clear();
    }
}

pub struct ConflictResolver {
    pub collision_tracker: RwLock<OriginCollisionTracker>,
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictResolver {
    pub fn new() -> Self {
        Self {
            collision_tracker: RwLock::new(OriginCollisionTracker::new()),
        }
    }
}

pub trait ConfidenceArbiter {
    fn evaluate_conflict(
        &self,
        incumbent: &UnifiedNode,
        challenger: &UnifiedNode,
    ) -> ResolutionResult;
}

impl ConfidenceArbiter for ConflictResolver {
    fn evaluate_conflict(
        &self,
        incumbent: &UnifiedNode,
        challenger: &UnifiedNode,
    ) -> ResolutionResult {
        let challenger_role = challenger
            .relational
            .get("_owner_role")
            .and_then(|v| v.as_str())
            .unwrap_or("anonymous");

        {
            let tracker = self.collision_tracker.read();
            if tracker.is_slashed(challenger_role) {
                return ResolutionResult::Reject(format!(
                    "Slashing Policy: agent '{}' has Confidence Score 0.0 (banned)",
                    challenger_role
                ));
            }
        }

        if let Some(sim) = incumbent.vector.cosine_similarity(&challenger.vector) {
            if sim > 0.95 {
                if incumbent.is_pinned() && incumbent.importance >= 0.8 {
                    let mut tracker = self.collision_tracker.write();
                    tracker.record_collision(challenger_role, challenger.confidence_score());

                    let friction = tracker.compute_friction();
                    let threshold = incumbent.importance * 10.0;

                    if friction < threshold {
                        return ResolutionResult::Reject(
                            format!(
                                "Consistency Barrier: Insufficient friction (F_ax={:.2} < threshold={:.2}). Unique origins: {}",
                                friction, threshold, tracker.unique_origins()
                            )
                        );
                    }
                }

                if challenger.confidence_score() < incumbent.confidence_score() {
                    return ResolutionResult::Superposition(
                        crate::governance::consistency::ConsistencyRecord::new_superposition(
                            incumbent.clone(),
                            challenger.clone(),
                            10000,
                        ),
                    );
                }
            }
        }
        ResolutionResult::Accept
    }
}
