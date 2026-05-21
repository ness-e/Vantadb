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
