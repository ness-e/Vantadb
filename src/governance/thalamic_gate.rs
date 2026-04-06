use parking_lot::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const DEFAULT_BLOOM_BITS: usize = 100_000; // ~12.5 KB to target FP < 0.01 for 10k items
const K_SALTS: [u64; 3] = [0x5A5A5A5A5A5A5A5A, 0x3C3C3C3C3C3C3C3C, 0x1E1E1E1E1E1E1E1E];

/// ThalamicGate filters the reingestion of known rejected/hallucinated nodes.
/// Implemented using a Minimalist in-house Bloom Filter (Zero-Dependencies)
/// based on Rust's DefaultHasher and 3 unique salt seeds for k-hashing.
pub struct ThalamicGate {
    bit_array: RwLock<Vec<u8>>,
    bits_count: usize,
}

impl ThalamicGate {
    pub fn new(capacity_hint: usize) -> Self {
        // Adjust bits dynamically if needed, keeping default at least 100_000
        // Optimal m = -n*ln(p) / (ln(2)^2). For n=10,000, p=0.01 => m ≈ 95850 bits.
        let optimal_bits = ((capacity_hint as f64) * 9.585).ceil() as usize;
        let bits_count = optimal_bits.max(DEFAULT_BLOOM_BITS);
        let size_bytes = (bits_count + 7) / 8;
        
        Self {
            bit_array: RwLock::new(vec![0; size_bytes]),
            bits_count,
        }
    }

    /// Evaluates the 3 hash indexes for a given node_id
    fn calculate_hashes(&self, node_id: u64) -> [usize; 3] {
        let mut idxs = [0; 3];
        for (i, &salt) in K_SALTS.iter().enumerate() {
            let mut hasher = DefaultHasher::new();
            node_id.hash(&mut hasher);
            salt.hash(&mut hasher);
            idxs[i] = (hasher.finish() as usize) % self.bits_count;
        }
        idxs
    }

    /// Mark an ID as rejected with K k-hash insertions
    pub fn record_rejection(&self, node_id: u64) {
        let idxs = self.calculate_hashes(node_id);
        let mut bit_array = self.bit_array.write();
        
        for idx in idxs {
            let byte_idx = idx / 8;
            let bit_pos = idx % 8;
            bit_array[byte_idx] |= 1 << bit_pos;
        }
    }

    /// Fast probabilistic check if a node is blocked.
    pub fn is_rejected(&self, node_id: u64) -> bool {
        let idxs = self.calculate_hashes(node_id);
        let bit_array = self.bit_array.read();
        
        for idx in idxs {
            let byte_idx = idx / 8;
            let bit_pos = idx % 8;
            if (bit_array[byte_idx] & (1 << bit_pos)) == 0 {
                return false;
            }
        }
        true
    }

    /// Standard Bloom Filter cannot remove items without explicit counting (Counting Bloom Filter).
    /// Amnesty is generally ignored here, which is an accepted tradeoff for the performance.
    pub fn grant_amnesty(&self, _node_id: u64) {
        // No-OP
    }
}
