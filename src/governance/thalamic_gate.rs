use parking_lot::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const DEFAULT_BLOOM_BITS: usize = 100_000; // ~12.5 KB to target FP < 0.01 for 10k items
const K_SALTS: [u64; 3] = [0x5A5A5A5A5A5A5A5A, 0x3C3C3C3C3C3C3C3C, 0x1E1E1E1E1E1E1E1E];

/// ThalamicGate filters the reingestion of known rejected/hallucinated nodes
/// and slashed agent roles.
///
/// Implemented using a Minimalist in-house Bloom Filter (Zero-Dependencies)
/// based on Rust's DefaultHasher and 3 unique salt seeds for k-hashing.
///
/// Phase 36: Extended with role-based banning (Epistemic Apoptosis).
/// Once an agent's `_owner_role` is banned, ALL future mutations from that
/// agent are rejected at the L1 level (XOR/POPCNT speed).
/// The ban is irreversible until engine restart (standard Bloom Filter).
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

    /// Evaluates the 3 hash indexes for a given u64 key
    fn calculate_hashes_u64(&self, key: u64) -> [usize; 3] {
        let mut idxs = [0; 3];
        for (i, &salt) in K_SALTS.iter().enumerate() {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            salt.hash(&mut hasher);
            idxs[i] = (hasher.finish() as usize) % self.bits_count;
        }
        idxs
    }

    /// Evaluates the 3 hash indexes for a string key (Phase 36: role-based banning)
    fn calculate_hashes_str(&self, key: &str) -> [usize; 3] {
        let mut idxs = [0; 3];
        for (i, &salt) in K_SALTS.iter().enumerate() {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            salt.hash(&mut hasher);
            idxs[i] = (hasher.finish() as usize) % self.bits_count;
        }
        idxs
    }

    /// Set bits in the bloom filter for given hash indexes
    fn set_bits(&self, idxs: &[usize; 3]) {
        let mut bit_array = self.bit_array.write();
        for &idx in idxs {
            let byte_idx = idx / 8;
            let bit_pos = idx % 8;
            bit_array[byte_idx] |= 1 << bit_pos;
        }
    }

    /// Check if all bits are set for given hash indexes
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

    // ─── Node-level rejection (existing API) ────────────────────

    /// Mark a node ID as rejected with K k-hash insertions
    pub fn record_rejection(&self, node_id: u64) {
        let idxs = self.calculate_hashes_u64(node_id);
        self.set_bits(&idxs);
    }

    /// Fast probabilistic check if a node is blocked.
    pub fn is_rejected(&self, node_id: u64) -> bool {
        let idxs = self.calculate_hashes_u64(node_id);
        self.check_bits(&idxs)
    }

    // ─── Phase 36: Role-level banning (Epistemic Apoptosis) ─────

    /// Ban an entire `_owner_role` identity from all future mutations.
    /// This is the L1 Hard-Filter: O(1) constant-time rejection at POPCNT speed.
    /// Irreversible until engine restart.
    pub fn record_role_ban(&self, owner_role: &str) {
        let idxs = self.calculate_hashes_str(owner_role);
        self.set_bits(&idxs);
    }

    /// Check if an `_owner_role` has been banned (Epistemic Apoptosis).
    /// Returns true if the role is probably banned (Bloom FP rate < 0.01).
    pub fn is_role_banned(&self, owner_role: &str) -> bool {
        let idxs = self.calculate_hashes_str(owner_role);
        self.check_bits(&idxs)
    }

    /// Standard Bloom Filter cannot remove items without explicit counting (Counting Bloom Filter).
    /// Amnesty is generally ignored here, which is an accepted tradeoff for the performance.
    pub fn grant_amnesty(&self, _node_id: u64) {
        // No-OP
    }
}
