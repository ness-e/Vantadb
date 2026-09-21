#![allow(clippy::expect_used, clippy::unwrap_used)]
// Standalone generator for benches/data/synthetic_dataset.bin.
// Mirrors next_u64/gen_f32 in benches/common/mod.rs so regeneration is
// byte-identical (verify by re-hashing the output).
//
// Build & run from the workspace root:
//   rustc benches/common/gen_dataset.rs -O -o /tmp/gen_dataset && /tmp/gen_dataset
fn next_u64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn main() {
    let dim = 256usize;
    let count = 2000usize;
    let seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut state = seed;
    let mut buf = Vec::with_capacity(count * dim * 4);
    for _ in 0..count {
        for _ in 0..dim {
            let bits = next_u64(&mut state);
            let v: f32 = ((bits >> 11) as f32) / ((1u64 << 21) as f32);
            buf.extend_from_slice(&v.to_le_bytes());
        }
    }
    std::fs::write("benches/data/synthetic_dataset.bin", &buf).unwrap();
    println!("wrote benches/data/synthetic_dataset.bin: {} bytes", buf.len());
}
