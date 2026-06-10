//! Duplicate prevention utilities for multi-writer scenarios.
//!
//! Bloom filter-based duplicate detection to prevent redundant inserts
//! in concurrent multi-writer environments.

use parking_lot::RwLock;
use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;

const DEFAULT_BLOOM_BITS: usize = 100_000;
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
/// let record_id = 12345u64;
/// 
/// if !filter.is_duplicate(record_id) {
///     filter.mark_processed(record_id);
///     // Proceed with insert
/// }
/// ```
pub struct DuplicatePreventionFilter {
    bit_array: RwLock<Vec<u8>>,
    bits_count: usize,
}

impl DuplicatePreventionFilter {
    /// Create a new bloom filter with capacity hint for expected number of records.
    /// 
    /// # Arguments
    /// * `capacity_hint` - Expected number of unique records
    pub fn new(capacity_hint: usize) -> Self {
        let optimal_bits = ((capacity_hint as f64) * 9.585).ceil() as usize;
        let bits_count = optimal_bits.max(DEFAULT_BLOOM_BITS);
        let size_bytes = bits_count.div_ceil(8);

        Self {
            bit_array: RwLock::new(vec![0; size_bytes]),
            bits_count,
        }
    }

    fn calculate_hashes_u64(&self, key: u64) -> [usize; 3] {
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
    pub fn mark_processed(&self, record_id: u64) {
        let idxs = self.calculate_hashes_u64(record_id);
        self.set_bits(&idxs);
    }

    /// Check if a record ID has already been processed.
    /// 
    /// Returns `true` if the record is likely a duplicate (false positives possible).
    pub fn is_duplicate(&self, record_id: u64) -> bool {
        let idxs = self.calculate_hashes_u64(record_id);
        self.check_bits(&idxs)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_detection() {
        let filter = DuplicatePreventionFilter::new(1000);
        let record_id = 42u64;

        assert!(!filter.is_duplicate(record_id));
        filter.mark_processed(record_id);
        assert!(filter.is_duplicate(record_id));
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
        assert!(!filter.is_duplicate(123u64));
        filter.mark_processed(123u64);
        assert!(filter.is_duplicate(123u64));
    }
}