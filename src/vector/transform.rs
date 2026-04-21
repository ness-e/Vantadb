use crate::hardware::{HardwareCapabilities, InstructionSet};
use wide::f32x8;

/// Fast Walsh-Hadamard Transform (FWHT)
///
/// Distributes the variance of the vector components across all dimensions,
/// which is critical to minimizing error before 1-bit and 3-bit quantization.
/// Mutates the input slice in place. Requires `data.len()` to be a power of 2.
pub fn fwht(data: &mut [f32]) {
    let n = data.len();
    if !n.is_power_of_two() {
        return; // Must handle padding horizontally before calling
    }

    let caps = HardwareCapabilities::global();
    match caps.instructions {
        InstructionSet::Fallback => fwht_scalar(data),
        _ => fwht_simd(data),
    }
}

pub fn fwht_scalar(data: &mut [f32]) {
    let n = data.len();
    let mut h = 1;
    while h < n {
        for i in (0..n).step_by(h * 2) {
            for j in i..i + h {
                let x = data[j];
                let y = data[j + h];
                data[j] = x + y;
                data[j + h] = x - y;
            }
        }
        h *= 2;
    }

    // Normalize to preserve magnitude
    let scale = 1.0 / (n as f32).sqrt();
    for x in data.iter_mut() {
        *x *= scale;
    }
}

pub fn fwht_simd(data: &mut [f32]) {
    let n = data.len();
    let mut h = 1;

    // For strides smaller than 8, we do scalar, because f32x8 cannot easily
    // interleave elements across 1, 2, 4 distance natively without complex swizzles.
    // Given the cache locality, scalar is extremely fast here anyway.
    while h < 8 && h < n {
        for i in (0..n).step_by(h * 2) {
            for j in i..i + h {
                let x = data[j];
                let y = data[j + h];
                data[j] = x + y;
                data[j + h] = x - y;
            }
        }
        h *= 2;
    }

    // SIMD for h >= 8
    while h < n {
        for i in (0..n).step_by(h * 2) {
            for j in (i..i + h).step_by(8) {
                // Ensure we don't go out of bounds
                if j + 8 <= i + h {
                    let x_slice = &data[j..j + 8];
                    let y_slice = &data[j + h..j + h + 8];

                    let x = f32x8::new([
                        x_slice[0], x_slice[1], x_slice[2], x_slice[3], x_slice[4], x_slice[5],
                        x_slice[6], x_slice[7],
                    ]);
                    let y = f32x8::new([
                        y_slice[0], y_slice[1], y_slice[2], y_slice[3], y_slice[4], y_slice[5],
                        y_slice[6], y_slice[7],
                    ]);

                    let new_x = x + y;
                    let new_y = x - y;

                    let arr_x: [f32; 8] = new_x.into();
                    let arr_y: [f32; 8] = new_y.into();

                    data[j..j + 8].copy_from_slice(&arr_x);
                    data[j + h..j + h + 8].copy_from_slice(&arr_y);
                } else {
                    // Scalar fallback for remainder
                    for k in j..i + h {
                        let x = data[k];
                        let y = data[k + h];
                        data[k] = x + y;
                        data[k + h] = x - y;
                    }
                }
            }
        }
        h *= 2;
    }

    // Normalize
    let scale = 1.0 / (n as f32).sqrt();
    let scale_v = f32x8::splat(scale);
    let mut chunks = data.chunks_exact_mut(8);
    for chunk in &mut chunks {
        let x = f32x8::new([
            chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
        ]);
        let v = x * scale_v;
        let arr: [f32; 8] = v.into();
        chunk.copy_from_slice(&arr);
    }
    for x in chunks.into_remainder() {
        *x *= scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fwht_scalar() {
        let mut data = vec![1.0, 0.0, 1.0, 0.0];
        fwht_scalar(&mut data);
        let expected = [1.0, 1.0, 0.0, 0.0];
        for (a, b) in data.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 1e-5);
        }
    }

    #[test]
    fn test_fwht_simd_vs_scalar() {
        let mut d1 = vec![0.5f32; 1024];
        for (i, item) in d1.iter_mut().enumerate() {
            *item = (i as f32).sin();
        }
        let mut d2 = d1.clone();

        fwht_scalar(&mut d1);
        fwht_simd(&mut d2);

        for (a, b) in d1.iter().zip(d2.iter()) {
            assert!((a - b).abs() < 1e-4);
        }
    }
}
