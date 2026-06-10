//! Confidence metrics for multi-agent coordination.
//!
//! Utilities for tracking collision counts and confidence scores per unique origin.
//! Used in multi-agent scenarios to compute friction metrics and detect problematic agents.

use std::collections::HashMap;

/// Tracks collision counts and confidence scores per unique origin (`_owner_role`).
///
/// Used in multi-agent coordination to compute logarithmic friction metrics
/// that help identify agents with high conflict rates.
///
/// # Example
/// ```no_run
/// use vantadb::utils::confidence_metrics::OriginCollisionTracker;
///
/// let mut tracker = OriginCollisionTracker::new();
/// tracker.record_collision("agent_alpha", 0.95);
/// tracker.record_collision("agent_beta", 0.80);
///
/// let friction = tracker.compute_friction();
/// println!("Total friction: {}", friction);
/// ```
#[derive(Debug, Clone, Default)]
pub struct OriginCollisionTracker {
    /// Map: owner_role → (collision_count, confidence_score_of_origin)
    origins: HashMap<String, (u64, f32)>,
}

impl OriginCollisionTracker {
    /// Create a new collision tracker.
    pub fn new() -> Self {
        Self {
            origins: HashMap::new(),
        }
    }

    /// Record a collision event for a specific origin with its confidence score.
    ///
    /// # Arguments
    /// * `owner_role` - The agent/origin that caused the collision
    /// * `challenger_confidence` - The confidence score of the colliding operation
    pub fn record_collision(&mut self, owner_role: &str, challenger_confidence: f32) {
        let entry = self
            .origins
            .entry(owner_role.to_string())
            .or_insert((0, challenger_confidence));
        entry.0 += 1;
        // Exponential moving average for confidence smoothing
        entry.1 = entry.1 * 0.8 + challenger_confidence * 0.2;
    }

    /// Compute the logarithmic friction metric F_ax.
    ///
    /// The friction metric is calculated as:
    /// `Σ (log2(collision_count + 1) * confidence_score)` for all origins
    ///
    /// This metric helps identify problematic origins with high collision rates
    /// in multi-agent coordination scenarios.
    pub fn compute_friction(&self) -> f32 {
        self.origins
            .iter()
            .map(|(_, (count, confidence))| ((*count as f32 + 1.0).log2()) * confidence)
            .sum()
    }

    /// Get the number of unique origins tracked.
    pub fn unique_origins(&self) -> usize {
        self.origins.len()
    }

    /// Slash (set to zero) the confidence score for a specific origin.
    ///
    /// This is used to "ban" problematic agents by setting their confidence to 0.
    pub fn slash_origin(&mut self, owner_role: &str) {
        if let Some(entry) = self.origins.get_mut(owner_role) {
            entry.1 = 0.0;
        } else {
            self.origins.insert(owner_role.to_string(), (0, 0.0));
        }
    }

    /// Check if an origin has been slashed (confidence score <= 0).
    pub fn is_slashed(&self, owner_role: &str) -> bool {
        self.origins
            .get(owner_role)
            .is_some_and(|(_, confidence)| *confidence <= 0.0)
    }

    /// Reset all tracking data.
    pub fn reset(&mut self) {
        self.origins.clear();
    }
}

/// Compute the friction metric from a collision tracker map.
///
/// This is a functional alternative to the method-based approach,
/// useful for one-off calculations without maintaining state.
///
/// # Arguments
/// * `origins` - Map of owner_role → (collision_count, confidence_score)
///
/// # Returns
/// The logarithmic friction metric F_ax
pub fn compute_confidence_friction(origins: &HashMap<String, (u64, f32)>) -> f32 {
    origins
        .iter()
        .map(|(_, (count, confidence))| ((*count as f32 + 1.0).log2()) * confidence)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_tracking() {
        let mut tracker = OriginCollisionTracker::new();
        tracker.record_collision("agent_a", 0.9);
        tracker.record_collision("agent_a", 0.85);

        assert_eq!(tracker.unique_origins(), 1);
    }

    #[test]
    fn test_friction_computation() {
        let mut tracker = OriginCollisionTracker::new();
        tracker.record_collision("agent_a", 1.0);
        tracker.record_collision("agent_b", 0.5);

        let friction = tracker.compute_friction();
        assert!(friction > 0.0);
    }

    #[test]
    fn test_slashing() {
        let mut tracker = OriginCollisionTracker::new();
        tracker.record_collision("bad_agent", 0.9);

        assert!(!tracker.is_slashed("bad_agent"));
        tracker.slash_origin("bad_agent");
        assert!(tracker.is_slashed("bad_agent"));
    }

    #[test]
    fn test_functional_friction() {
        let mut origins = HashMap::new();
        origins.insert("agent_a".to_string(), (2, 0.9));
        origins.insert("agent_b".to_string(), (1, 0.5));

        let friction = compute_confidence_friction(&origins);
        assert!(friction > 0.0);
    }

    #[test]
    fn test_reset() {
        let mut tracker = OriginCollisionTracker::new();
        tracker.record_collision("agent_a", 0.9);
        tracker.reset();

        assert_eq!(tracker.unique_origins(), 0);
        assert_eq!(tracker.compute_friction(), 0.0);
    }
}
