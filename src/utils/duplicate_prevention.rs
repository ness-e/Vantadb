//! Duplicate prevention utilities for multi-writer scenarios.
//!
//! Bloom filter-based duplicate detection to prevent redundant inserts
//! in concurrent multi-writer environments.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;

const DEFAULT_BLOOM_BITS: usize = 100_000;

/// Maximum Bloom filter size in bits (64 MB = 512 Mbits) to prevent unbounded memory allocation.
pub const MAX_BLOOM_BITS: usize = 64 * 1024 * 1024 * 8;

const K_SALTS: [u64; 3] = [0x5A5A5A5A5A5A5A5A, 0x3C3C3C3C3C3C3C3C, 0x1E1E1E1E1E1E1E1E];

/// Bloom filter for preventing duplicate record ingestion.
///
/// Useful in multi-writer concurrent scenarios where multiple agents
/// may attempt to insert the same record simultaneously.
///
/// # Example
/// ```no_run
/// use vantadb::utils::duplicate_prevention::DuplicatePreventionFilter;
///
/// let filter = DuplicatePreventionFilter::new(100_000);
/// let record_id = 12345u128;
///
/// if !filter.is_duplicate(record_id) {
///     filter.mark_processed(record_id);
///     // Proceed with insert
/// }
/// ```
pub struct DuplicatePreventionFilter {
    bit_array: RwLock<Vec<u8>>,
    bits_count: usize,
    exact_registry: RwLock<std::collections::HashMap<(String, u64), String>>,
}

impl DuplicatePreventionFilter {
    /// Create a new bloom filter with capacity hint for expected number of records.
    ///
    /// # Arguments
    /// * `capacity_hint` - Expected number of unique records
    pub fn new(capacity_hint: usize) -> Self {
        let optimal_bits = ((capacity_hint as f64) * 9.585).ceil() as usize;
        let bits_count = optimal_bits.max(DEFAULT_BLOOM_BITS).min(MAX_BLOOM_BITS);
        let size_bytes = bits_count.div_ceil(8);

        Self {
            bit_array: RwLock::new(vec![0; size_bytes]),
            bits_count,
            exact_registry: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Create a Bloom filter with an explicit number of bits (useful for testing collisions).
    pub fn new_with_bits(bits_count: usize) -> Self {
        let bits_count = bits_count.max(8).min(MAX_BLOOM_BITS);
        let size_bytes = bits_count.div_ceil(8);
        Self {
            bit_array: RwLock::new(vec![0; size_bytes]),
            bits_count,
            exact_registry: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Register exact content hash mapping to confirmed record key.
    pub fn register_exact(&self, namespace: &str, content_hash: u64, key: &str) {
        let mut map = self.exact_registry.write();
        map.insert((namespace.to_string(), content_hash), key.to_string());
    }

    /// Look up confirmed record key by content hash in namespace.
    pub fn lookup_exact(&self, namespace: &str, content_hash: u64) -> Option<String> {
        let map = self.exact_registry.read();
        map.get(&(namespace.to_string(), content_hash)).cloned()
    }

    fn calculate_hashes(&self, key: u128) -> [usize; 3] {
        let mut idxs = [0; 3];
        for (i, &salt) in K_SALTS.iter().enumerate() {
            let mut hasher = XxHash64::with_seed(0);
            key.hash(&mut hasher);
            salt.hash(&mut hasher);
            idxs[i] = (hasher.finish() as usize) % self.bits_count;
        }
        idxs
    }

    fn calculate_hashes_str(&self, key: &str) -> [usize; 3] {
        let mut idxs = [0; 3];
        for (i, &salt) in K_SALTS.iter().enumerate() {
            let mut hasher = XxHash64::with_seed(0);
            key.hash(&mut hasher);
            salt.hash(&mut hasher);
            idxs[i] = (hasher.finish() as usize) % self.bits_count;
        }
        idxs
    }

    fn calculate_dual_hashes_bytes(&self, bytes: &[u8]) -> [usize; 3] {
        let mut h1 = XxHash64::with_seed(0);
        bytes.hash(&mut h1);
        let hash1 = h1.finish();

        let mut h2 = XxHash64::with_seed(K_SALTS[0]);
        bytes.hash(&mut h2);
        let hash2 = h2.finish();

        let mut idxs = [0; 3];
        for (i, item) in idxs.iter_mut().enumerate() {
            let combined = hash1.wrapping_add((i as u64).wrapping_mul(hash2));
            *item = (combined as usize) % self.bits_count;
        }
        idxs
    }

    fn set_bits(&self, idxs: &[usize; 3]) {
        let mut bit_array = self.bit_array.write();
        for &idx in idxs {
            let byte_idx = idx / 8;
            let bit_pos = idx % 8;
            bit_array[byte_idx] |= 1 << bit_pos;
        }
    }

    fn check_bits(&self, idxs: &[usize; 3]) -> bool {
        let bit_array = self.bit_array.read();
        for &idx in idxs {
            let byte_idx = idx / 8;
            let bit_pos = idx % 8;
            if (bit_array[byte_idx] & (1 << bit_pos)) == 0 {
                return false;
            }
        }
        true
    }

    /// Mark a record ID as processed to prevent future duplicates.
    pub fn mark_processed(&self, record_id: u128) {
        let idxs = self.calculate_hashes(record_id);
        self.set_bits(&idxs);
    }

    /// Check if a record ID has already been processed.
    ///
    /// Returns `true` if the record is likely a duplicate (false positives possible).
    pub fn is_duplicate(&self, record_id: u128) -> bool {
        let idxs = self.calculate_hashes(record_id);
        self.check_bits(&idxs)
    }

    /// Check if content has already been seen using dual XxHash64, and mark it
    /// atomically in a single write lock acquisition.
    ///
    /// Returns `true` if it was already marked (duplicate), `false` if it was fresh and is now marked.
    pub fn check_and_mark(&self, content: &[u8]) -> bool {
        let idxs = self.calculate_dual_hashes_bytes(content);
        let mut bit_array = self.bit_array.write();
        let mut is_dup = true;
        for &idx in &idxs {
            let byte_idx = idx / 8;
            let bit_pos = idx % 8;
            if (bit_array[byte_idx] & (1 << bit_pos)) == 0 {
                is_dup = false;
                break;
            }
        }
        if is_dup {
            return true;
        }
        for &idx in &idxs {
            let byte_idx = idx / 8;
            let bit_pos = idx % 8;
            bit_array[byte_idx] |= 1 << bit_pos;
        }
        false
    }

    /// Mark an agent role as blocked to prevent future operations from that role.
    pub fn block_role(&self, owner_role: &str) {
        let idxs = self.calculate_hashes_str(owner_role);
        self.set_bits(&idxs);
    }

    /// Check if an agent role is blocked.
    pub fn is_role_blocked(&self, owner_role: &str) -> bool {
        let idxs = self.calculate_hashes_str(owner_role);
        self.check_bits(&idxs)
    }
}

impl Default for DuplicatePreventionFilter {
    fn default() -> Self {
        Self::new(DEFAULT_BLOOM_BITS)
    }
}

/// Fast token Jaccard similarity over normalized tokens.
/// Splits on non-alphanumeric/non-underscore characters and lowercases.
pub fn token_jaccard_similarity(a: &str, b: &str) -> f32 {
    let extract = |s: &str| -> HashSet<String> {
        s.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|t| !t.is_empty())
            .map(str::to_lowercase)
            .collect()
    };
    let a_tokens = extract(a);
    let b_tokens = extract(b);
    if a_tokens.is_empty() && b_tokens.is_empty() {
        return 1.0;
    }
    if a_tokens.is_empty() || b_tokens.is_empty() {
        return 0.0;
    }
    let intersection = a_tokens.intersection(&b_tokens).count();
    let union = a_tokens.union(&b_tokens).count();
    if union == 0 {
        1.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Fast-gate pre-LLM decision band based on semantic (cosine) and lexical (Jaccard) similarities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FastGateDecision {
    /// Highly similar: no-op / reinforce existing memory without calling LLM.
    /// (cosine >= 0.92 or jaccard >= 0.85)
    NoopReinforce,
    /// Moderately similar: supersedes existing memory.
    /// (0.78 <= cosine < 0.92)
    Supersede,
    /// Dissimilar / new memory: add as new entry.
    /// (< 0.78)
    Add,
}

impl FastGateDecision {
    /// Classify candidate similarities according to the Q17 approved thresholds:
    /// - `exact_dup` or `cosine >= 0.92` or `jaccard >= 0.85` -> `FastGateDecision::NoopReinforce`
    /// - `0.78 <= cosine < 0.92` -> `FastGateDecision::Supersede`
    /// - `< 0.78` -> `FastGateDecision::Add`
    pub fn classify_with_exact(exact_dup: bool, cosine: f32, jaccard: f32) -> Self {
        if exact_dup || cosine >= 0.92 || jaccard >= 0.85 {
            FastGateDecision::NoopReinforce
        } else if cosine >= 0.78 {
            FastGateDecision::Supersede
        } else {
            FastGateDecision::Add
        }
    }

    /// Classify candidate similarities according to the Q17 approved thresholds:
    /// - `cosine >= 0.92` or `jaccard >= 0.85` -> `FastGateDecision::NoopReinforce`
    /// - `0.78 <= cosine < 0.92` -> `FastGateDecision::Supersede`
    /// - `< 0.78` -> `FastGateDecision::Add`
    pub fn classify(cosine: f32, jaccard: f32) -> Self {
        Self::classify_with_exact(false, cosine, jaccard)
    }
}

/// Convenience function for `FastGateDecision::classify`.
pub fn fast_gate_classify(cosine: f32, jaccard: f32) -> FastGateDecision {
    FastGateDecision::classify(cosine, jaccard)
}

/// Normalize content for deduplication (trim and lowercase).
pub fn normalize_content(content: &str) -> String {
    content.trim().to_lowercase()
}

/// Compute 64-bit content hash of normalized content.
pub fn hash_content(content: &str) -> u64 {
    let mut hasher = XxHash64::with_seed(0);
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_detection() {
        let filter = DuplicatePreventionFilter::new(1000);
        let record_id = 42u128;

        assert!(!filter.is_duplicate(record_id));
        filter.mark_processed(record_id);
        assert!(filter.is_duplicate(record_id));
    }

    #[test]
    fn test_check_and_mark_atomic() {
        let filter = DuplicatePreventionFilter::new(1000);
        let content = b"user likes dark mode";

        assert!(!filter.check_and_mark(content));
        assert!(filter.check_and_mark(content));
    }

    #[test]
    fn test_max_bloom_bits_bound() {
        let filter = DuplicatePreventionFilter::new(usize::MAX / 2);
        assert!(filter.bits_count <= MAX_BLOOM_BITS);
    }

    #[test]
    fn test_token_jaccard_similarity() {
        assert_eq!(token_jaccard_similarity("", ""), 1.0);
        assert_eq!(token_jaccard_similarity("hello world", ""), 0.0);
        assert_eq!(
            token_jaccard_similarity("user prefers dark mode", "user prefers dark mode"),
            1.0
        );
        let sim = token_jaccard_similarity("user prefers dark mode", "user prefers light theme");
        assert!(sim > 0.3 && sim < 0.6, "sim was {sim}");
    }

    #[test]
    fn test_fast_gate_decision_thresholds() {
        // cosine >= 0.92 -> NoopReinforce
        assert_eq!(
            FastGateDecision::classify(0.95, 0.2),
            FastGateDecision::NoopReinforce
        );
        // jaccard >= 0.85 -> NoopReinforce
        assert_eq!(
            FastGateDecision::classify(0.5, 0.9),
            FastGateDecision::NoopReinforce
        );
        // 0.78 <= cosine < 0.92 -> Supersede
        assert_eq!(
            FastGateDecision::classify(0.85, 0.4),
            FastGateDecision::Supersede
        );
        assert_eq!(
            FastGateDecision::classify(0.78, 0.1),
            FastGateDecision::Supersede
        );
        // < 0.78 -> Add
        assert_eq!(FastGateDecision::classify(0.77, 0.6), FastGateDecision::Add);
        assert_eq!(FastGateDecision::classify(0.1, 0.1), FastGateDecision::Add);
    }

    #[test]
    fn test_role_blocking() {
        let filter = DuplicatePreventionFilter::new(1000);
        let role = "test_agent";

        assert!(!filter.is_role_blocked(role));
        filter.block_role(role);
        assert!(filter.is_role_blocked(role));
    }

    #[test]
    fn test_default_capacity() {
        let filter = DuplicatePreventionFilter::default();
        assert!(!filter.is_duplicate(123u128));
        filter.mark_processed(123u128);
        assert!(filter.is_duplicate(123u128));
    }
}
