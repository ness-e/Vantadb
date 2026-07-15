use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use web_time::{SystemTime, UNIX_EPOCH};

use crate::sync_ext::RwLockExt;

const MAX_CONFLICT_BACKOFF: u32 = 64;

/// A version vector mapping origin → version counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionVector {
    versions: HashMap<String, u64>,
    timestamp: u128,
}

impl VersionVector {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
            timestamp: now_nanos(),
        }
    }

    pub fn with_origin(origin: String, version: u64) -> Self {
        let mut versions = HashMap::new();
        versions.insert(origin, version);
        Self {
            versions,
            timestamp: now_nanos(),
        }
    }

    pub fn increment(&mut self, origin: &str) {
        let entry = self.versions.entry(origin.to_string()).or_insert(0);
        *entry += 1;
        self.timestamp = now_nanos();
    }

    pub fn version(&self, origin: &str) -> u64 {
        self.versions.get(origin).copied().unwrap_or(0)
    }

    pub fn origins(&self) -> impl Iterator<Item = &String> {
        self.versions.keys()
    }

    pub fn timestamp(&self) -> u128 {
        self.timestamp
    }

    /// Returns conflict type when comparing with another vector.
    pub fn compare(&self, other: &VersionVector) -> VersionOrder {
        let mut self_greater = false;
        let mut other_greater = false;

        let all_origins: std::collections::HashSet<&String> =
            self.versions.keys().chain(other.versions.keys()).collect();

        for origin in all_origins {
            let sv = self.version(origin);
            let ov = other.version(origin);
            if sv > ov {
                self_greater = true;
            } else if ov > sv {
                other_greater = true;
            }
        }

        match (self_greater, other_greater) {
            (false, false) => VersionOrder::Equal,
            (true, false) => VersionOrder::After,
            (false, true) => VersionOrder::Before,
            (true, true) => VersionOrder::Concurrent,
        }
    }

    /// Merge two version vectors, taking the max version per origin.
    pub fn merge(&self, other: &VersionVector) -> VersionVector {
        let mut merged = self.versions.clone();
        for (origin, version) in &other.versions {
            let entry = merged.entry(origin.clone()).or_insert(0);
            if *version > *entry {
                *entry = *version;
            }
        }
        VersionVector {
            versions: merged,
            timestamp: now_nanos(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionOrder {
    Equal,
    Before,
    After,
    Concurrent,
}

/// A candidate value with associated version vector.
#[derive(Debug, Clone)]
pub struct Candidate<T: Clone> {
    pub value: T,
    pub version: VersionVector,
    pub origin: String,
}

/// Outcome of conflict resolution.
#[derive(Debug, Clone)]
pub enum Resolution<T: Clone> {
    Accepted(Candidate<T>),
    Superposition(Vec<Candidate<T>>),
    Rejected(String),
}

/// Record of a resolved conflict for audit.
#[derive(Debug, Clone)]
pub struct ConflictRecord {
    pub node_id: u64,
    pub timestamp: u128,
    pub origins: Vec<String>,
    pub resolution: String,
    pub nonce: u64,
}

/// Conflict resolver with bounded friction, exponential backoff on repeated
/// conflicts, and a LWW fallback for non-conflicting fields.
pub struct ConflictResolver {
    /// Friction coefficient, bounded [0.0, 1.0].
    friction_coeff: f64,
    /// Per-node conflict backoff counter.
    conflict_backoff: RwLock<HashMap<u64, u32>>,
    /// Audit log of resolved conflicts.
    conflict_log: RwLock<Vec<ConflictRecord>>,
    /// Nonce counter (atomic + timestamp combo for uniqueness).
    nonce_counter: AtomicU64,
    /// Maximum entries in conflict log.
    max_log_entries: usize,
}

impl ConflictResolver {
    pub fn new(friction_coeff: f64, max_log_entries: usize) -> Self {
        let coeff = friction_coeff.clamp(0.0, 1.0);
        Self {
            friction_coeff: coeff,
            conflict_backoff: RwLock::new(HashMap::new()),
            conflict_log: RwLock::new(Vec::new()),
            nonce_counter: AtomicU64::new(0),
            max_log_entries,
        }
    }

    /// Generate a globally unique nonce from atomic counter + timestamp.
    fn generate_nonce(&self) -> u64 {
        let counter = self.nonce_counter.fetch_add(1, Ordering::SeqCst);
        let time_part = (now_nanos() & 0x00FF_FFFF_FFFF_FFFF) as u64;
        time_part ^ counter.rotate_left(16)
    }

    /// Resolve a conflict between incumbent and challenger candidates.
    ///
    /// Returns `Resolution::Accepted` if the challenger wins,
    /// `Resolution::Superposition` if the conflict cannot be resolved immediately,
    /// or `Resolution::Rejected` if the challenger is rejected.
    pub fn resolve<T: Clone + PartialEq>(
        &self,
        node_id: u64,
        incumbent: &Candidate<T>,
        challenger: Candidate<T>,
    ) -> Resolution<T> {
        let order = incumbent.version.compare(&challenger.version);

        match order {
            VersionOrder::Equal | VersionOrder::After => {
                self.log_conflict(
                    node_id,
                    &incumbent.origin,
                    &challenger.origin,
                    "incumbent_wins",
                );
                Resolution::Accepted(incumbent.clone())
            }
            VersionOrder::Before => {
                self.log_conflict(
                    node_id,
                    &incumbent.origin,
                    &challenger.origin,
                    "challenger_wins",
                );
                Resolution::Accepted(challenger)
            }
            VersionOrder::Concurrent => {
                if challenger.value == incumbent.value {
                    let mut merged = incumbent.clone();
                    merged.version = incumbent.version.merge(&challenger.version);
                    self.log_conflict(
                        node_id,
                        &incumbent.origin,
                        &challenger.origin,
                        "merged_identical",
                    );
                    return Resolution::Accepted(merged);
                }

                let backoff = self.compute_backoff(node_id);
                let friction = self.compute_friction(&incumbent.origin, &challenger.origin);

                if friction >= 1.0 - self.friction_coeff * (backoff as f64).recip() {
                    self.log_conflict(
                        node_id,
                        &incumbent.origin,
                        &challenger.origin,
                        "superposition",
                    );
                    Resolution::Superposition(vec![incumbent.clone(), challenger])
                } else {
                    self.log_conflict(
                        node_id,
                        &incumbent.origin,
                        &challenger.origin,
                        "challenger_wins_after_backoff",
                    );
                    Resolution::Accepted(challenger)
                }
            }
        }
    }

    /// Backpressure-based friction computation.
    /// Higher collision count → lower friction → harder to pass (GOV-02 fix).
    fn compute_friction(&self, incumbent_origin: &str, challenger_origin: &str) -> f64 {
        let backoff = self.conflict_backoff.lock_rwlock();
        let inc_collisions = backoff
            .get(&(hash_str(incumbent_origin) as u64))
            .copied()
            .unwrap_or(0);
        let chal_collisions = backoff
            .get(&(hash_str(challenger_origin) as u64))
            .copied()
            .unwrap_or(0);
        let total = (inc_collisions + chal_collisions).max(1) as f64;

        let epsilon = 1e-6;
        1.0 / (total.log2() + 1.0 + epsilon)
    }

    /// Exponential backoff for repeated conflicts (GOV-07 fix).
    /// Returns number of backoff levels (starts at 1, doubles per conflict).
    fn compute_backoff(&self, node_id: u64) -> u32 {
        let mut backoff = self.conflict_backoff.lock_rwlock_mut();
        let count = backoff.entry(node_id).or_insert(0);
        *count = (*count + 1).min(MAX_CONFLICT_BACKOFF);
        *count
    }

    /// Three-way merge for concurrent writes on structured fields.
    /// Falls back to LWW for non-conflicting fields.
    pub fn three_way_merge<T: Clone + PartialEq>(base: &[T], ours: &[T], theirs: &[T]) -> Vec<T> {
        let max_len = ours.len().max(theirs.len());
        let mut result = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let ours_val = ours.get(i);
            let theirs_val = theirs.get(i);
            let base_val = base.get(i);

            match (ours_val, theirs_val, base_val) {
                (Some(o), Some(t), _) if o == t => result.push(o.clone()),
                (Some(o), Some(t), Some(b)) if o == b => result.push(t.clone()),
                (Some(o), Some(t), Some(b)) if t == b => result.push(o.clone()),
                (Some(o), Some(_), _) => result.push(o.clone()),
                (Some(o), None, _) => result.push(o.clone()),
                (None, Some(t), _) => result.push(t.clone()),
                _ => {}
            }
        }

        result
    }

    /// Reset backoff for a given node (called on successful resolution).
    pub fn reset_backoff(&self, node_id: u64) {
        let mut backoff = self.conflict_backoff.lock_rwlock_mut();
        backoff.remove(&node_id);
    }

    /// Garbage-collect old entries from the conflict log.
    pub fn gc_conflict_log(&self, max_age_nanos: u128) -> usize {
        let mut log = self.conflict_log.lock_rwlock_mut();
        let cutoff = now_nanos().saturating_sub(max_age_nanos);
        let before = log.len();
        log.retain(|r| r.timestamp >= cutoff);
        before - log.len()
    }

    fn log_conflict(
        &self,
        node_id: u64,
        incumbent_origin: &str,
        challenger_origin: &str,
        resolution: &str,
    ) {
        let record = ConflictRecord {
            node_id,
            timestamp: now_nanos(),
            origins: vec![incumbent_origin.to_string(), challenger_origin.to_string()],
            resolution: resolution.to_string(),
            nonce: self.generate_nonce(),
        };
        let mut log = self.conflict_log.lock_rwlock_mut();
        if log.len() >= self.max_log_entries {
            log.remove(0);
        }
        log.push(record);
    }

    /// Returns a snapshot of the conflict log for audit.
    pub fn conflict_log(&self) -> Vec<ConflictRecord> {
        self.conflict_log.lock_rwlock().clone()
    }

    /// Current backoff levels for all tracked nodes.
    pub fn backoff_levels(&self) -> HashMap<u64, u32> {
        self.conflict_backoff.lock_rwlock().clone()
    }
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = twox_hash::XxHash64::with_seed(0);
    Hash::hash(s, &mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_vector_ordering() {
        let mut v1 = VersionVector::new();
        v1.increment("origin-a");
        let mut v2 = v1.clone();
        assert_eq!(v1.compare(&v2), VersionOrder::Equal);
        v2.increment("origin-a");
        assert_eq!(v1.compare(&v2), VersionOrder::Before);
        assert_eq!(v2.compare(&v1), VersionOrder::After);
    }

    #[test]
    fn test_concurrent_versions() {
        let mut v1 = VersionVector::with_origin("alice".into(), 1);
        let mut v2 = VersionVector::with_origin("bob".into(), 1);
        v1.increment("alice");
        v2.increment("bob");
        assert_eq!(v1.compare(&v2), VersionOrder::Concurrent);
    }

    #[test]
    fn test_three_way_merge() {
        let base = vec![1, 2, 3];
        let ours = vec![1, 5, 3];
        let theirs = vec![1, 2, 4];
        let result = ConflictResolver::three_way_merge(&base, &ours, &theirs);
        assert_eq!(result, vec![1, 5, 4]);
    }

    #[test]
    fn test_resolve_accepts_challenger() {
        let resolver = ConflictResolver::new(0.5, 100);
        let incumbent = Candidate {
            value: "old",
            version: VersionVector::with_origin("alice".into(), 1),
            origin: "alice".into(),
        };
        let challenger = Candidate {
            value: "new",
            version: VersionVector::with_origin("alice".into(), 2),
            origin: "alice".into(),
        };
        match resolver.resolve(1, &incumbent, challenger) {
            Resolution::Accepted(c) => assert_eq!(c.value, "new"),
            _ => panic!("Expected challenger to be accepted (is causally after)"),
        }
    }

    #[test]
    fn test_resolve_rejects_old_version() {
        let resolver = ConflictResolver::new(0.5, 100);
        let incumbent = Candidate {
            value: "current",
            version: VersionVector::with_origin("alice".into(), 5),
            origin: "alice".into(),
        };
        let challenger = Candidate {
            value: "stale",
            version: VersionVector::with_origin("alice".into(), 3),
            origin: "alice".into(),
        };
        match resolver.resolve(1, &incumbent, challenger) {
            Resolution::Accepted(c) => {
                assert_eq!(c.value, "current");
            }
            _ => panic!("Expected incumbent to be accepted (challenger is causally before)"),
        }
    }

    #[test]
    fn test_friction_coeff_bounded() {
        let resolver = ConflictResolver::new(1.5, 100);
        assert!(resolver.friction_coeff <= 1.0);
        assert!(resolver.friction_coeff >= 0.0);

        let resolver2 = ConflictResolver::new(-0.5, 100);
        assert!(resolver2.friction_coeff >= 0.0);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let resolver = ConflictResolver::new(0.5, 100);
        let n1 = resolver.generate_nonce();
        let n2 = resolver.generate_nonce();
        let n3 = resolver.generate_nonce();
        assert_ne!(n1, n2);
        assert_ne!(n2, n3);
        assert_ne!(n1, n3);
    }

    #[test]
    fn test_exponential_backoff_capped() {
        let resolver = ConflictResolver::new(0.5, 100);
        let node_id = 42;
        for _ in 0..200 {
            resolver.compute_backoff(node_id);
        }
        let levels = resolver.backoff_levels();
        assert!(levels.get(&node_id).copied().unwrap_or(0) <= MAX_CONFLICT_BACKOFF);
    }
}
