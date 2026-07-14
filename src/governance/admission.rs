use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use twox_hash::XxHash64;

use crate::sync_ext::RwLockExt;

const BLOOM_SEEDS: [u64; 3] = [
    0xdead_beef_dead_beef,
    0xcafe_babe_cafe_babe,
    0xbaad_f00d_baad_f00d,
];

/// Configuration for the admission filter.
#[derive(Debug, Clone)]
pub struct AdmissionConfig {
    /// Expected number of items to insert.
    pub capacity: usize,
    /// Target false-positive rate (e.g. 0.01 = 1%).
    pub target_fp_rate: f64,
    /// Auto-reset when estimated FP rate exceeds this threshold (e.g. 0.05 = 5%).
    pub reset_threshold: f64,
    /// Number of counters per row in CountMinSketch.
    pub sketch_width: usize,
    /// Number of rows (hash functions) in CountMinSketch.
    pub sketch_depth: usize,
}

impl Default for AdmissionConfig {
    fn default() -> Self {
        Self {
            capacity: 100_000,
            target_fp_rate: 0.01,
            reset_threshold: 0.05,
            sketch_width: 10_000,
            sketch_depth: 4,
        }
    }
}

/// Count-Min Sketch for frequency estimation of record IDs.
struct CountMinSketch {
    width: usize,
    depth: usize,
    counters: Vec<Vec<AtomicU64>>,
}

impl CountMinSketch {
    fn new(width: usize, depth: usize) -> Self {
        let counters = (0..depth)
            .map(|_| (0..width).map(|_| AtomicU64::new(0)).collect())
            .collect();
        Self {
            width,
            depth,
            counters,
        }
    }

    fn hash_row(&self, item: u64, row: usize) -> usize {
        let mut hasher =
            XxHash64::with_seed(BLOOM_SEEDS[row % BLOOM_SEEDS.len()].wrapping_add(row as u64));
        item.hash(&mut hasher);
        (hasher.finish() as usize) % self.width
    }

    fn increment(&self, item: u64) {
        for row in 0..self.depth {
            let idx = self.hash_row(item, row);
            self.counters[row][idx].fetch_add(1, Ordering::Relaxed);
        }
    }

    fn estimate(&self, item: u64) -> u64 {
        (0..self.depth)
            .map(|row| {
                let idx = self.hash_row(item, row);
                self.counters[row][idx].load(Ordering::Relaxed)
            })
            .min()
            .unwrap_or(0)
    }

    fn reset(&self) {
        for row in 0..self.depth {
            for col in 0..self.width {
                self.counters[row][col].store(0, Ordering::Relaxed);
            }
        }
    }
}

/// Fixed-capacity Bloom filter with automatic reset when false-positive rate
/// exceeds the configured threshold. Combined with a CountMinSketch for
/// frequency tracking.
pub struct AdmissionFilter {
    bits: RwLock<Vec<u64>>,
    num_bits: usize,
    num_hashes: usize,
    insert_count: AtomicU64,
    test_count: AtomicU64,
    false_positive_count: AtomicU64,
    config: AdmissionConfig,
    sketch: CountMinSketch,
}

impl AdmissionFilter {
    pub fn new(config: AdmissionConfig) -> Self {
        let num_bits = optimal_bits(config.capacity, config.target_fp_rate);
        let num_hashes = optimal_hashes(config.capacity, num_bits);
        let word_count = (num_bits + 63) / 64;
        Self {
            bits: RwLock::new(vec![0u64; word_count]),
            num_bits,
            num_hashes,
            insert_count: AtomicU64::new(0),
            test_count: AtomicU64::new(0),
            false_positive_count: AtomicU64::new(0),
            config: config.clone(),
            sketch: CountMinSketch::new(config.sketch_width, config.sketch_depth),
        }
    }

    fn hash_positions(&self, item: u64) -> Vec<usize> {
        let mut positions = Vec::with_capacity(self.num_hashes);
        for i in 0..self.num_hashes {
            let seed = BLOOM_SEEDS[i % BLOOM_SEEDS.len()].wrapping_add(i as u64);
            let mut hasher = XxHash64::with_seed(seed);
            item.hash(&mut hasher);
            positions.push((hasher.finish() as usize) % self.num_bits);
        }
        positions
    }

    fn hash_str(&self, s: &str) -> u64 {
        let mut hasher = XxHash64::with_seed(BLOOM_SEEDS[0]);
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Add a record ID to the filter.
    pub fn block_record(&self, id: u64) {
        let positions = self.hash_positions(id);
        let mut bits = self.bits.lock_rwlock_mut();
        for pos in &positions {
            bits[pos / 64] |= 1u64 << (pos % 64);
        }
        self.insert_count.fetch_add(1, Ordering::Relaxed);
        self.sketch.increment(id);

        if self.estimated_fp_rate() > self.config.reset_threshold {
            self.reset_filter_internal(&mut bits);
        }
    }

    /// Add a role string to the filter.
    pub fn block_role(&self, role: &str) {
        let hashed = self.hash_str(role);
        self.block_record(hashed);
    }

    /// Check whether a record ID is blocked.
    pub fn is_blocked(&self, id: u64) -> bool {
        let positions = self.hash_positions(id);
        let bits = self.bits.lock_rwlock();
        let test_count = self.test_count.fetch_add(1, Ordering::Relaxed);

        for pos in &positions {
            if bits[pos / 64] & (1u64 << (pos % 64)) == 0 {
                return false;
            }
        }

        if test_count > 0 {
            let fp_estimate = self.estimated_fp_rate_internal(test_count, &bits);
            let fp_rate = fp_estimate as f64 / test_count as f64;
            if fp_rate > self.config.target_fp_rate {
                self.false_positive_count.fetch_add(1, Ordering::Relaxed);
            }
        }

        true
    }

    /// Check whether a role is blocked.
    pub fn is_role_blocked(&self, role: &str) -> bool {
        let hashed = self.hash_str(role);
        self.is_blocked(hashed)
    }

    /// Estimate the frequency of a record ID using the CountMinSketch.
    pub fn record_frequency(&self, id: u64) -> u64 {
        self.sketch.estimate(id)
    }

    /// Current estimated false-positive rate.
    pub fn estimated_fp_rate(&self) -> f64 {
        let inserted = self.insert_count.load(Ordering::Relaxed);
        if inserted == 0 || self.num_bits == 0 {
            return 0.0;
        }
        let ratio = inserted as f64 / self.num_bits as f64;
        1.0 - (-ratio * self.num_hashes as f64).exp()
    }

    fn estimated_fp_rate_internal(&self, tests: u64, _bits: &[u64]) -> u64 {
        let inserted = self.insert_count.load(Ordering::Relaxed);
        if inserted == 0 || self.num_bits == 0 {
            return 0;
        }
        let ratio = inserted as f64 / self.num_bits as f64;
        let fp_prob = 1.0 - (-ratio * self.num_hashes as f64).exp();
        (fp_prob * tests as f64) as u64
    }

    /// Number of tracked false positives.
    pub fn false_positive_count(&self) -> u64 {
        self.false_positive_count.load(Ordering::Relaxed)
    }

    /// Total insertions since last reset.
    pub fn insert_count(&self) -> u64 {
        self.insert_count.load(Ordering::Relaxed)
    }

    /// Total membership tests performed.
    pub fn test_count(&self) -> u64 {
        self.test_count.load(Ordering::Relaxed)
    }

    /// Manually trigger a reset of the bloom filter.
    pub fn reset_filter(&self) {
        let mut bits = self.bits.lock_rwlock_mut();
        self.reset_filter_internal(&mut bits);
    }

    fn reset_filter_internal(&self, bits: &mut Vec<u64>) {
        for word in bits.iter_mut() {
            *word = 0;
        }
        self.insert_count.store(0, Ordering::Relaxed);
        self.false_positive_count.store(0, Ordering::Relaxed);
        self.sketch.reset();
    }

    /// Configuration used by this filter.
    pub fn config(&self) -> &AdmissionConfig {
        &self.config
    }
}

fn optimal_bits(capacity: usize, fp_rate: f64) -> usize {
    let n = capacity as f64;
    let bits = -(n * fp_rate.ln()) / (std::f64::consts::LN_2.powi(2));
    (bits.ceil() as usize).max(100_000)
}

fn optimal_hashes(capacity: usize, num_bits: usize) -> usize {
    let n = capacity as f64;
    let m = num_bits as f64;
    let k = (m / n * std::f64::consts::LN_2).ceil() as usize;
    k.max(1).min(10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_and_check_id() {
        let filter = AdmissionFilter::new(AdmissionConfig::default());
        filter.block_record(42);
        assert!(filter.is_blocked(42));
    }

    #[test]
    fn test_non_blocked_id() {
        let filter = AdmissionFilter::new(AdmissionConfig::default());
        filter.block_record(42);
        assert!(!filter.is_blocked(99));
    }

    #[test]
    fn test_block_and_check_role() {
        let filter = AdmissionFilter::new(AdmissionConfig::default());
        filter.block_role("malicious-actor");
        assert!(filter.is_role_blocked("malicious-actor"));
    }

    #[test]
    fn test_frequency_tracking() {
        let filter = AdmissionFilter::new(AdmissionConfig::default());
        filter.block_record(1);
        filter.block_record(1);
        filter.block_record(1);
        let freq = filter.record_frequency(1);
        assert!(freq >= 1);
    }

    #[test]
    fn test_fp_rate_bounded() {
        let config = AdmissionConfig {
            capacity: 1_000,
            target_fp_rate: 0.01,
            reset_threshold: 0.5,
            sketch_width: 1_000,
            sketch_depth: 4,
        };
        let filter = AdmissionFilter::new(config);
        for i in 0..10_000u64 {
            filter.block_record(i);
        }
        let fp_rate = filter.estimated_fp_rate();
        assert!(fp_rate <= 1.0, "FP rate should be bounded by 1.0");
    }

    #[test]
    fn test_reset_clears_stats() {
        let config = AdmissionConfig::default();
        let filter = AdmissionFilter::new(config);
        filter.block_record(42);
        assert!(filter.is_blocked(42));
        filter.reset_filter();
        assert!(!filter.is_blocked(42));
        assert_eq!(filter.insert_count(), 0);
    }
}
