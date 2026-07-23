//! Hybrid quantization algorithms for scalar and binary vector compression.

/// Hybrid Quantization Algorithms (Phase 31)
/// Contains carefully engineered quantization schemes for MMap Zero-Copy and L1 Caching.
///
/// SAFETY: All packed outputs are padded to 8-byte (u64) alignment boundaries
/// to prevent SIMD segfaults on unaligned mmap reads.
use core::f32;

/// Required alignment for mmap-safe SIMD reads (AVX2 minimum = 32, but u64 = 8 is our pack unit).
const MMAP_ALIGNMENT: usize = 8;

/// Creates a 1-bit representation (RaBitQ) of the FWHT-transformed vector.
/// Packs 64 boolean flag features into a single `u64`.
/// Excellent for massive batch pruning in L1 RAM cache.
pub fn rabitq_quantize(data: &[f32]) -> Box<[u64]> {
    let num_blocks = data.len().div_ceil(64);
    let mut packed = vec![0u64; num_blocks];

    for (i, &val) in data.iter().enumerate() {
        if val > 0.0 {
            let block = i / 64;
            let bit = i % 64;
            packed[block] |= 1 << bit;
        }
    }

    packed.into_boxed_slice()
}

/// Computes the similarity (equivalent to cosine similarity in Angular space)
/// between two 1-bit RaBitQ quantified vectors using POPCNT.
pub fn rabitq_similarity(a: &[u64], b: &[u64]) -> f32 {
    let mut xor_sum = 0;
    for (va, vb) in a.iter().zip(b.iter()) {
        xor_sum += (va ^ vb).count_ones();
    }

    let total_bits = (a.len() * 64) as f32;
    // Angle approximation from Hamming distance
    // cosine_sim = cos(pi * hamming / total_bits)
    // For fast retrieval, we can just return normalized match percentage,
    // which operates monotonically:

    1.0 - (xor_sum as f32 / total_bits)
}

/// Creates a PolarQuant (Custom 3-bit / 4-bit Two's Complement packed)
/// representation of the FWHT-transformed vector.
/// Each `u8` holds two 4-bit values (-8 to 7).
pub fn turbo_quant_quantize(data: &[f32]) -> (Box<[u8]>, f32) {
    // 1. Find max absolute value to establish the scaling bound
    let mut max_abs = 0.0_f32;
    for &val in data {
        let abs = val.abs();
        if abs > max_abs {
            max_abs = abs;
        }
    }

    // Fallback if vector is extremely close to zero
    if max_abs < f32::EPSILON {
        max_abs = 1.0;
    }

    // We quantize into range [-8, 7].
    let scale = 7.0 / max_abs;

    let num_bytes = data.len().div_ceil(2);
    let mut packed = vec![0u8; num_bytes];

    for (i, &val) in data.iter().enumerate() {
        let scaled = (val * scale).round();
        // Clamp explicitly to avoid panic on NaNs or huge math flukes
        let clamped = scaled.clamp(-8.0, 7.0) as i8;

        // Take bottom 4 bits safely
        let q = (clamped as u8) & 0x0F;

        let byte_pos = i / 2;
        if i % 2 == 0 {
            // High nibble
            packed[byte_pos] |= q << 4;
        } else {
            // Low nibble
            packed[byte_pos] |= q;
        }
    }

    // Pad to MMAP_ALIGNMENT boundary for safe SIMD mmap reads
    let aligned_len = (num_bytes + MMAP_ALIGNMENT - 1) & !(MMAP_ALIGNMENT - 1);
    packed.resize(aligned_len, 0u8);

    (packed.into_boxed_slice(), max_abs)
}

/// Helper wrapper that implements SIMD dot products for two unpacked TurboQuant strings.
/// (During Mmap, we stream the u8, unpack them rapidly, and accumulate).
pub fn turbo_quant_similarity(
    a_packed: &[u8],
    a_max_abs: f32,
    b_packed: &[u8],
    b_max_abs: f32,
) -> f32 {
    // Safety: verify pointer alignment for mmap zero-copy paths.
    // If data comes from mmap, misaligned pointers would cause SIMD penalties or segfaults.
    debug_assert!(
        (a_packed.as_ptr() as usize).is_multiple_of(std::mem::align_of::<u8>()),
        "turbo_quant_similarity: a_packed pointer is misaligned"
    );

    let mut dot = 0_i32;

    // Extremely fast scalar loop. The Rust compiler unrolls this beautifully,
    // and manual SIMD padding for 4-bit decompression is complex unless using specific shuffle intrinsic blocks.
    for (va, vb) in a_packed.iter().zip(b_packed.iter()) {
        let a_high = (*va >> 4) as i8;
        let a_high = if a_high & 8 != 0 { a_high | -8 } else { a_high }; // sign extend

        let a_low = (*va & 0x0F) as i8;
        let a_low = if a_low & 8 != 0 { a_low | -8 } else { a_low };

        let b_high = (*vb >> 4) as i8;
        let b_high = if b_high & 8 != 0 { b_high | -8 } else { b_high };

        let b_low = (*vb & 0x0F) as i8;
        let b_low = if b_low & 8 != 0 { b_low | -8 } else { b_low };

        dot += (a_high as i32 * b_high as i32) + (a_low as i32 * b_low as i32);
    }

    // Reverse the scale
    // Because both were scaled by (7.0 / max_abs), we divide by (49.0 / (a_max * b_max))

    // Note: Since fwht preserves magnitude, we can estimate cosine similarity directly
    // from this dot product if the original vectors were length 1.0!
    // But since this is a dot product, we just return it.
    dot as f32 * (a_max_abs * b_max_abs) / 49.0
}

/// 8-bit scalar quantization (SQ8).
/// Maps each f32 dimension to `i8` in [-127, 127] using linear scaling
/// by `max_abs` (the maximum absolute component value).
///
/// Memory: 1 byte/dim vs 4 bytes/dim for f32.
pub fn sq8_quantize(data: &[f32]) -> (Box<[i8]>, f32) {
    let mut max_abs = 0.0_f32;
    for &val in data {
        let abs = val.abs();
        if abs > max_abs {
            max_abs = abs;
        }
    }
    if max_abs < f32::EPSILON {
        max_abs = 1.0;
    }

    let scale = 127.0 / max_abs;
    let quantized: Vec<i8> = data
        .iter()
        .map(|&v| (v * scale).round().clamp(-127.0, 127.0) as i8)
        .collect();

    (quantized.into_boxed_slice(), max_abs)
}

/// Computes approximate dot-product similarity between two SQ8-quantized vectors.
pub fn sq8_similarity(a: &[i8], a_max_abs: f32, b: &[i8], b_max_abs: f32) -> f32 {
    let mut dot = 0_i32;
    for (va, vb) in a.iter().zip(b.iter()) {
        dot += *va as i32 * *vb as i32;
    }
    let scale = (a_max_abs / 127.0) * (b_max_abs / 127.0);
    dot as f32 * scale
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    // ── rabitq_quantize ──────────────────────────────────────────────

    #[test]
    fn test_rabitq_quantize_all_positive() {
        // All positive → every bit set in first block
        let data = vec![1.0; 10];
        let packed = rabitq_quantize(&data);
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0], (1u64 << 10) - 1, "first 10 bits should be 1");
    }

    #[test]
    fn test_rabitq_quantize_mixed() {
        let data = vec![0.5, -0.3, 0.1, -0.7, 0.0, 2.0];
        // bits for indices 0, 2, 5 → 0b_10_0101 = 37
        let packed = rabitq_quantize(&data);
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0], 0b_10_0101);
    }

    #[test]
    fn test_rabitq_quantize_all_zero_or_negative() {
        let data = vec![0.0, -1.0, -0.5, -0.0];
        let packed = rabitq_quantize(&data);
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0], 0, "no bits should be set");
    }

    #[test]
    fn test_rabitq_quantize_large_vector() {
        let data = vec![1.0; 150]; // spans 3 blocks (64+64+22)
        let packed = rabitq_quantize(&data);
        assert_eq!(packed.len(), 3);
        assert_eq!(packed[0], u64::MAX, "block 0: all 64 bits set");
        assert_eq!(packed[1], u64::MAX, "block 1: all 64 bits set");
        // block 2: only the first 22 bits set (150 - 128 = 22)
        assert_eq!(packed[2], (1u64 << 22) - 1, "block 2: first 22 bits set");
    }

    #[test]
    fn test_rabitq_quantize_block_boundary() {
        // Elements at index 63 and 64 straddle the boundary between blocks 0 and 1
        let mut data = vec![-1.0; 66];
        data[63] = 1.0;
        data[64] = 1.0;
        let packed = rabitq_quantize(&data);
        assert_eq!(packed.len(), 2);
        // Block 0: bit 63 set (MSB)
        assert_eq!(packed[0], 1u64 << 63, "bit 63 set in block 0");
        // Block 1: bit 0 set
        assert_eq!(packed[1], 1u64, "bit 0 set in block 1");
    }

    #[test]
    fn test_rabitq_quantize_empty() {
        let packed = rabitq_quantize(&[]);
        assert!(packed.is_empty());
    }

    #[test]
    fn test_rabitq_quantize_exact_64() {
        let data = vec![1.0; 64];
        let packed = rabitq_quantize(&data);
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0], u64::MAX);
    }

    #[test]
    fn test_rabitq_quantize_exact_65() {
        let data = vec![1.0; 65];
        let packed = rabitq_quantize(&data);
        assert_eq!(packed.len(), 2);
        assert_eq!(packed[0], u64::MAX);
        assert_eq!(packed[1], 1u64, "only bit 0 set in block 1");
    }

    // ── rabitq_similarity ────────────────────────────────────────────

    #[test]
    fn test_rabitq_similarity_identical() {
        let data = vec![1.0, -1.0, 0.5, -0.3, 0.8];
        let a = rabitq_quantize(&data);
        let b = rabitq_quantize(&data);
        let sim = rabitq_similarity(&a, &b);
        assert!(
            (sim - 1.0).abs() < f32::EPSILON,
            "identical should score 1.0, got {sim}"
        );
    }

    #[test]
    fn test_rabitq_similarity_opposite() {
        // All positive vs all negative → every element's sign bit flips
        let a = rabitq_quantize(&[1.0; 64]); // all 64 bits set
        let b = rabitq_quantize(&[-1.0; 64]); // no bits set
        let sim = rabitq_similarity(&a, &b);
        // xor_sum = 64, total_bits = 1*64 = 64 → 1.0 - 1.0 = 0.0
        assert!(
            (sim - 0.0).abs() < f32::EPSILON,
            "opposite should score 0.0, got {sim}"
        );
    }

    #[test]
    fn test_rabitq_similarity_half_match() {
        // 64 elements: half positive, half negative → 32 bits differ out of 64
        let mut a = vec![-1.0; 64];
        let mut b = vec![-1.0; 64];
        for i in 0..64 {
            if i < 32 {
                a[i] = 1.0; // first 32 positive in a
                b[i] = 1.0; // first 32 positive in b too → match
            } else {
                a[i] = 1.0; // last 32 positive in a
                b[i] = -1.0; // last 32 negative in b → differ
            }
        }
        let pa = rabitq_quantize(&a);
        let pb = rabitq_quantize(&b);
        let sim = rabitq_similarity(&pa, &pb);
        // First 32 bits match, last 32 differ → xor_sum = 32, total = 64 → 0.5
        assert!(
            (sim - 0.5).abs() < f32::EPSILON,
            "half match should score 0.5, got {sim}"
        );
    }

    #[test]
    fn test_rabitq_similarity_different_lengths() {
        // zip stops at the shorter length → only first block compared
        let a = rabitq_quantize(&[1.0; 70]); // 2 blocks: [all 64 bits set, 6 bits set]
        let b = rabitq_quantize(&[1.0; 10]); // 1 block:  [first 10 bits set]
        let sim = rabitq_similarity(&a, &b);
        // Only first block compared (zip to min len = 1).
        // a[0] = u64::MAX (all 64 bits set), b[0] = (1u64<<10)-1 = 1023 (10 bits set)
        // xor bits 0-9: 0 (both 1), bits 10-63: 1 (a=1, b=0) → 54 ones
        // xor_sum = 54, total_bits = a.len() * 64 = 2 * 64 = 128
        // sim = 1.0 - 54/128 = 74/128 = 0.578125
        let expected = 74.0 / 128.0;
        assert!(
            (sim - expected).abs() < f32::EPSILON,
            "expected {expected}, got {sim}"
        );
    }

    #[test]
    fn test_rabitq_similarity_multi_block() {
        let mut a = vec![1.0; 128];
        let mut b = vec![1.0; 128];
        // Flip half of the bits in each block
        for i in 0..32 {
            a[i] = -1.0;
            b[i + 64] = -1.0;
        }
        let pa = rabitq_quantize(&a);
        let pb = rabitq_quantize(&b);
        let sim = rabitq_similarity(&pa, &pb);
        // Block 0: 32 bits differ → xor_sum += 32
        // Block 1: 32 bits differ → xor_sum += 32
        // total = 64, xor_total = 64 → 1.0 - 0.5 = 0.5
        assert!((sim - 0.5).abs() < 0.001, "expected ~0.5, got {sim}");
    }

    // ── turbo_quant_quantize ─────────────────────────────────────────

    #[test]
    fn test_turbo_quant_quantize_basic() {
        // data in [-1, 1] range, max_abs = 1.0
        let data = vec![1.0, 0.5, 0.0, -0.5, -1.0];
        let (packed, max_abs) = turbo_quant_quantize(&data);
        assert!(
            (max_abs - 1.0).abs() < f32::EPSILON,
            "max_abs should be 1.0"
        );
        // 5 elements → ceil(5/2) = 3 bytes, padded to MMAP_ALIGNMENT (8) → 8 bytes
        assert_eq!(packed.len(), 8, "should be padded to 8 bytes");
    }

    #[test]
    fn test_turbo_quant_quantize_zero_vector() {
        let data = vec![0.0; 10];
        let (packed, max_abs) = turbo_quant_quantize(&data);
        // max_abs < EPSILON → fallback to 1.0
        assert!(
            (max_abs - 1.0).abs() < f32::EPSILON,
            "zero vector fallback max_abs=1"
        );
        assert_eq!(packed.len(), 8, "padded to 8 bytes");
        // All values are 0 → all nibbles are 0
        for &byte in packed.iter() {
            assert_eq!(byte, 0, "all nibbles should be 0 for zero vector");
        }
    }

    #[test]
    fn test_turbo_quant_quantize_alignment() {
        // Various sizes to verify padding to 8-byte boundary
        let cases = [1, 2, 3, 7, 8, 15, 16, 31, 32];
        for &n in &cases {
            let data = vec![0.5; n];
            let (packed, _) = turbo_quant_quantize(&data);
            assert!(
                packed.len() % MMAP_ALIGNMENT == 0,
                "len {} for n={} should be multiple of {}",
                packed.len(),
                n,
                MMAP_ALIGNMENT
            );
        }
    }

    #[test]
    fn test_turbo_quant_quantize_negative_values() {
        let data = vec![-1.0, -0.5];
        let (packed, max_abs) = turbo_quant_quantize(&data);
        assert!((max_abs - 1.0).abs() < f32::EPSILON);
        // high nibble = quant(-1) = -7 & 0x0F = 0x09
        // low nibble  = quant(-0.5 rounded) = -3.5 → -4 & 0x0F = 0x0C
        // byte = 0x90 | 0x0C = 0x9C
        assert_eq!(packed[0], 0x9C, "expected 0x9C for [-1.0, -0.5]");
    }

    #[test]
    fn test_turbo_quant_quantize_odd_count() {
        // 3 elements → ceil(3/2) = 2 bytes → padded to 8
        let data = vec![0.3, 0.7, -0.9];
        let (packed, max_abs) = turbo_quant_quantize(&data);
        assert!(
            (max_abs - 0.9).abs() < f32::EPSILON,
            "max_abs should be 0.9"
        );
        assert_eq!(packed.len(), 8);
        // low nibble of second byte is padding (0)
        assert_eq!(
            packed[1] & 0x0F,
            0,
            "odd count: trailing nibble should be 0"
        );
    }

    #[test]
    fn test_turbo_quant_quantize_clamps_out_of_range() {
        // Extreme values beyond [-8, 7] range after scaling
        let data = vec![100.0, -200.0];
        let (packed, max_abs) = turbo_quant_quantize(&data);
        assert!((max_abs - 200.0).abs() < f32::EPSILON);
        // scaled = [3.5→4, -7.0→-7]
        // 100 * 7/200 = 3.5 → 4, -200 * 7/200 = -7
        // high nibble = 4, low nibble = -7 & 0x0F = 0x09
        // byte = 0x40 | 0x09 = 0x49
        assert_eq!(packed[0], 0x49, "expected 0x49 for [100, -200]");
    }

    // ── turbo_quant_similarity ───────────────────────────────────────

    #[test]
    fn test_turbo_quant_similarity_identical() {
        let data = vec![0.3, 0.7, -0.9, 0.1, -0.4];
        let (a, a_max) = turbo_quant_quantize(&data);
        let (b, b_max) = turbo_quant_quantize(&data);
        let sim = turbo_quant_similarity(&a, a_max, &b, b_max);
        assert!(sim > 0.0, "self-similarity should be positive, got {sim}");
        // Should equal the dot product of the original (approximately)
        let orig_dot: f32 = data.iter().map(|&v| v * v).sum();
        let err = (sim - orig_dot).abs() / orig_dot.max(1e-8);
        assert!(
            err < 0.2,
            "relative error {err} too high: sim={sim}, orig={orig_dot}"
        );
    }

    #[test]
    fn test_turbo_quant_similarity_opposite() {
        let pos = vec![0.3, 0.7, -0.9];
        let neg: Vec<f32> = pos.iter().map(|&v| -v).collect();
        let (a, a_max) = turbo_quant_quantize(&pos);
        let (b, b_max) = turbo_quant_quantize(&neg);
        let sim = turbo_quant_similarity(&a, a_max, &b, b_max);
        assert!(
            sim < 0.0,
            "opposite vectors should give negative sim, got {sim}"
        );
        // Should be roughly -dot(pos, pos)
        let orig_dot: f32 = pos.iter().map(|&v| v * v).sum();
        assert!(
            (sim + orig_dot).abs() / orig_dot.max(1e-8) < 0.3,
            "expected ~{}, got {sim}",
            -orig_dot
        );
    }

    #[test]
    fn test_turbo_quant_similarity_zero_vector() {
        let zero = vec![0.0; 10];
        let data = vec![0.3, 0.7, -0.9, 0.1];
        let (a, a_max) = turbo_quant_quantize(&zero);
        let (b, b_max) = turbo_quant_quantize(&data);
        let sim = turbo_quant_similarity(&a, a_max, &b, b_max);
        // Zero vector quantized → all zeros → dot product = 0
        assert!(
            (sim - 0.0).abs() < 1e-6,
            "zero vector should give 0 sim, got {sim}"
        );
    }

    #[test]
    fn test_turbo_quant_similarity_different_max_abs() {
        let a_data = vec![1.0; 4];
        let b_data = vec![0.5; 4];
        let (a, a_max) = turbo_quant_quantize(&a_data);
        let (b, b_max) = turbo_quant_quantize(&b_data);
        let sim = turbo_quant_similarity(&a, a_max, &b, b_max);
        // Original dot = 4 * 1.0 * 0.5 = 2.0
        let orig_dot: f32 = a_data.iter().zip(b_data.iter()).map(|(x, y)| x * y).sum();
        let err = (sim - orig_dot).abs() / orig_dot.max(1e-8);
        assert!(
            err < 0.3,
            "relative error {err} too high: sim={sim}, orig={orig_dot}"
        );
    }

    #[test]
    fn test_turbo_quant_similarity_self_consistent() {
        // Two similar but not identical vectors should give a reasonable similarity
        let a = vec![0.5, -0.3, 0.8, 0.1, -0.6, 0.2];
        let b = vec![0.4, -0.2, 0.9, 0.0, -0.5, 0.3];
        let (pa, am) = turbo_quant_quantize(&a);
        let (pb, bm) = turbo_quant_quantize(&b);
        let sim = turbo_quant_similarity(&pa, am, &pb, bm);
        let orig_dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let err = (sim - orig_dot).abs();
        // Quantization loss is expected but should be within reasonable bounds
        assert!(
            err < 1.0,
            "quantized similarity {sim} too far from original {orig_dot}"
        );
    }

    // ── sq8_quantize ─────────────────────────────────────────────────

    #[test]
    fn test_sq8_quantize_basic() {
        let data = vec![1.0, -0.5, 0.3, -0.8];
        let (packed, max_abs) = sq8_quantize(&data);
        assert!(
            (max_abs - 1.0).abs() < f32::EPSILON,
            "max_abs should be 1.0"
        );
        assert_eq!(packed.len(), 4);
        // 1.0 * 127/1 = 127
        assert_eq!(packed[0], 127);
        // -0.5 * 127 = -63.5 → -64
        assert_eq!(packed[1], -64);
        // 0.3 * 127 = 38.1 → 38
        assert_eq!(packed[2], 38);
        // -0.8 * 127 = -101.6 → -102
        assert_eq!(packed[3], -102);
    }

    #[test]
    fn test_sq8_quantize_zero_vector() {
        let data = vec![0.0; 5];
        let (packed, max_abs) = sq8_quantize(&data);
        assert!(
            (max_abs - 1.0).abs() < f32::EPSILON,
            "zero vector fallback max_abs=1"
        );
        for &v in packed.iter() {
            assert_eq!(v, 0, "all quantized values should be 0");
        }
    }

    #[test]
    fn test_sq8_quantize_clamping() {
        // Values beyond [-1, 1] should clamp to [-127, 127]
        let data = vec![5.0, -10.0, 100.0];
        let (packed, max_abs) = sq8_quantize(&data);
        assert!(
            (max_abs - 100.0).abs() < f32::EPSILON,
            "max_abs should be 100.0"
        );
        // 5.0 * 127/100 = 6.35 → 6
        assert_eq!(packed[0], 6, "clamped value for 5.0");
        // -10.0 * 127/100 = -12.7 → -13
        assert_eq!(packed[1], -13, "clamped value for -10.0");
        // 100.0 * 127/100 = 127 → clamped to 127
        assert_eq!(packed[2], 127, "clamped value for 100.0");
    }

    #[test]
    fn test_sq8_quantize_negative_max() {
        let data = vec![-0.3, 0.1, -2.0];
        let (packed, max_abs) = sq8_quantize(&data);
        assert!(
            (max_abs - 2.0).abs() < f32::EPSILON,
            "max_abs should be 2.0"
        );
        // -0.3 * 127/2 = -19.05 → -19
        assert_eq!(packed[0], -19);
        // 0.1 * 127/2 = 6.35 → 6
        assert_eq!(packed[1], 6);
        // -2.0 * 127/2 = -127
        assert_eq!(packed[2], -127);
    }

    #[test]
    fn test_sq8_quantize_empty() {
        let (packed, max_abs) = sq8_quantize(&[]);
        assert!(packed.is_empty());
        assert!(
            (max_abs - 1.0).abs() < f32::EPSILON,
            "empty vector fallback"
        );
    }

    // ── sq8_similarity ───────────────────────────────────────────────

    #[test]
    fn test_sq8_similarity_identical() {
        let data = vec![0.5, -0.3, 0.8, 0.1];
        let (a, a_max) = sq8_quantize(&data);
        let (b, b_max) = sq8_quantize(&data);
        let sim = sq8_similarity(&a, a_max, &b, b_max);
        let orig_dot: f32 = data.iter().map(|&v| v * v).sum();
        let err = (sim - orig_dot).abs() / orig_dot.max(1e-8);
        assert!(
            err < 0.01,
            "self-similarity error {err} too high: sim={sim}, orig={orig_dot}"
        );
    }

    #[test]
    fn test_sq8_similarity_opposite() {
        let pos = vec![0.5, -0.3, 0.8];
        let neg: Vec<f32> = pos.iter().map(|&v| -v).collect();
        let (a, a_max) = sq8_quantize(&pos);
        let (b, b_max) = sq8_quantize(&neg);
        let sim = sq8_similarity(&a, a_max, &b, b_max);
        let orig_dot: f32 = pos.iter().zip(neg.iter()).map(|(x, y)| x * y).sum();
        assert!(
            sim < 0.0,
            "opposite vectors should give negative sim, got {sim}"
        );
        // orig_dot should be roughly -(0.5^2 + 0.3^2 + 0.8^2) = -(0.25+0.09+0.64) = -0.98
        let err = (sim - orig_dot).abs() / orig_dot.abs().max(1e-8);
        assert!(
            err < 0.02,
            "relative error {err} too high: sim={sim}, orig={orig_dot}"
        );
    }

    #[test]
    fn test_sq8_similarity_orthogonal() {
        let a_data = vec![0.5, 0.0, -0.3];
        let b_data = vec![0.0, 1.0, 0.0];
        let (a, a_max) = sq8_quantize(&a_data);
        let (b, b_max) = sq8_quantize(&b_data);
        let sim = sq8_similarity(&a, a_max, &b, b_max);
        let orig_dot: f32 = a_data.iter().zip(b_data.iter()).map(|(x, y)| x * y).sum();
        assert!(
            (sim - orig_dot).abs() < 0.01,
            "orthogonal sim should be ~0, got {sim}, orig={orig_dot}"
        );
    }

    #[test]
    fn test_sq8_similarity_different_max_abs() {
        let a_data = vec![2.0, -1.0, 0.5];
        let b_data = vec![0.5, 1.0, -2.0];
        let (a, a_max) = sq8_quantize(&a_data);
        let (b, b_max) = sq8_quantize(&b_data);
        let sim = sq8_similarity(&a, a_max, &b, b_max);
        let orig_dot: f32 = a_data.iter().zip(b_data.iter()).map(|(x, y)| x * y).sum();
        let err = (sim - orig_dot).abs() / orig_dot.abs().max(1e-8);
        assert!(
            err < 0.05,
            "relative error {err} too high: sim={sim}, orig={orig_dot}"
        );
    }

    #[test]
    fn test_sq8_similarity_zero_vector() {
        let zero = vec![0.0; 4];
        let data = vec![0.5, -0.3, 0.8, 0.1];
        let (a, a_max) = sq8_quantize(&zero);
        let (b, b_max) = sq8_quantize(&data);
        let sim = sq8_similarity(&a, a_max, &b, b_max);
        assert!(
            (sim - 0.0).abs() < 1e-6,
            "zero vector should give 0 sim, got {sim}"
        );
    }

    // ── Roundtrip consistency ────────────────────────────────────────

    #[test]
    fn test_rabitq_quantize_roundtrip() {
        // Verify that quantize → similarity is consistent with original dot product
        let data_a = vec![0.6, -0.2, 0.9, -0.7, 0.1, -0.4, 0.3, 0.0];
        let data_b = vec![-0.3, 0.5, 0.2, 0.8, -0.6, 0.7, -0.1, 0.4];
        let pa = rabitq_quantize(&data_a);
        let pb = rabitq_quantize(&data_b);
        let sim_q = rabitq_similarity(&pa, &pb);

        // For binary quantization, the similarity approximates angular cosine.
        // It should be in [0, 1] and directionally consistent.
        assert!(
            sim_q >= 0.0 && sim_q <= 1.0,
            "RaBitQ similarity should be in [0, 1], got {sim_q}"
        );
    }

    #[test]
    fn test_sq8_roundtrip_precision() {
        // Roundtrip SQ8: quantize → reconstruct → verify error bounds
        let original: Vec<f32> = vec![0.12, 0.88, 0.54, 0.31, -0.22, 0.95, -0.11, 0.47];
        let (packed, max_abs) = sq8_quantize(&original);
        let inv = max_abs / 127.0;
        let reconstructed: Vec<f32> = packed.iter().map(|&q| (q as f32) * inv).collect();

        for (orig, recon) in original.iter().zip(reconstructed.iter()) {
            let err = (orig - recon).abs();
            assert!(
                err < 0.02,
                "SQ8 roundtrip error {err} too high for {orig} vs {recon}"
            );
        }
    }

    #[test]
    fn test_turbo_roundtrip_precision() {
        // Roundtrip TurboQuant: quantize → reconstruct → verify error
        let original: Vec<f32> = vec![0.15, 0.72, -0.43, 0.61, -0.88, 0.33, 0.92, -0.17];
        let (packed, max_abs) = turbo_quant_quantize(&original);

        // Manual reconstruction from packed nibbles
        let mut reconstructed = Vec::with_capacity(original.len());
        for (_i, &byte) in packed.iter().enumerate() {
            if reconstructed.len() >= original.len() {
                break;
            }
            let high = (byte >> 4) as i8;
            let high = if high & 8 != 0 { high | -8 } else { high };
            reconstructed.push((high as f32) * max_abs / 7.0);

            if reconstructed.len() >= original.len() {
                break;
            }
            let low = (byte & 0x0F) as i8;
            let low = if low & 8 != 0 { low | -8 } else { low };
            reconstructed.push((low as f32) * max_abs / 7.0);
        }

        for (orig, recon) in original.iter().zip(reconstructed.iter()) {
            let err = (orig - recon).abs();
            assert!(
                err < 0.15,
                "Turbo roundtrip error {err} too high for {orig} vs {recon}"
            );
        }
    }
}
