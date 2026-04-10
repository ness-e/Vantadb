use parking_lot::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const DEFAULT_BLOOM_BITS: usize = 100_000; 
const K_SALTS: [u64; 3] = [0x5A5A5A5A5A5A5A5A, 0x3C3C3C3C3C3C3C3C, 0x1E1E1E1E1E1E1E1E];

/// AdmissionFilter prevents the ingestion of previously rejected records
/// and blocked agent roles via a probabilistic Bloom Filter.
pub struct AdmissionFilter {
    bit_array: RwLock<Vec<u8>>,
    bits_count: usize,
}

impl AdmissionFilter {
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
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            salt.hash(&mut hasher);
            idxs[i] = (hasher.finish() as usize) % self.bits_count;
        }
        idxs
    }

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

    pub fn block_record(&self, record_id: u64) {
        let idxs = self.calculate_hashes_u64(record_id);
        self.set_bits(&idxs);
    }

    pub fn is_blocked(&self, record_id: u64) -> bool {
        let idxs = self.calculate_hashes_u64(record_id);
        self.check_bits(&idxs)
    }

    pub fn block_role(&self, owner_role: &str) {
        let idxs = self.calculate_hashes_str(owner_role);
        self.set_bits(&idxs);
    }

    pub fn is_role_blocked(&self, owner_role: &str) -> bool {
        let idxs = self.calculate_hashes_str(owner_role);
        self.check_bits(&idxs)
    }
}
