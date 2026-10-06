//! STRAT-04 slice 1 — lock-free parallel scoring kernel for wasm32 threads.
//!
//! Built for `wasm32-unknown-unknown` with `+atomics` + shared memory (see
//! `build.ps1`). Each wasm instance (one per Web Worker / Node `worker_thread`)
//! imports the SAME `WebAssembly.Memory`, so every instance operates on one
//! shared linear memory. Work is distributed with a lock-free chunk cursor
//! (`AtomicU32::fetch_add` on shared memory) — no mutexes, no host-side
//! coordination.
//!
//! The kernel is `no_std` in wasm builds: no allocator, no TLS, no
//! `std::sync` — the smallest surface that links with `--shared-memory`.
//!
//! Host contract (JS side, see `benchmarks/wasm_threads_bench.mjs`):
//! - one `WebAssembly.Memory({ shared: true, maximum: ... })` created once;
//! - every worker instantiates this module with `{ env: { memory } }` and
//!   assigns `__stack_pointer` to its own disjoint stack region;
//! - if a future build emits `__wasm_init_memory` (shared-memory build with
//!   passive data segments), exactly one instance must call it once — the
//!   current build emits no such export (verified: module exports are
//!   `__stack_pointer` + `score_batch_chunked` only);
//! - the f32 arena layout and the raw pointers are computed by the host.

#![cfg_attr(all(not(test), target_arch = "wasm32"), no_std)]
#![warn(missing_docs)]
#![forbid(unsafe_op_in_unsafe_fn)]

use core::sync::atomic::{AtomicU32, Ordering};

/// Kernel return code: success.
pub const STATUS_OK: u32 = 0;
/// Kernel return code: a required pointer was null (nothing was dereferenced).
pub const STATUS_NULL: u32 = 1;
/// Kernel return code: `dims == 0` or `chunk == 0`.
pub const STATUS_BAD_ARGS: u32 = 2;

/// Cosine similarity between two f32 slices (compares up to the shorter one).
///
/// Returns `0.0` when either vector has zero norm (fail-safe: never NaN).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    let mut i = 0usize;
    while i < n {
        let x = a[i];
        let y = b[i];
        dot += x * y;
        na += x * x;
        nb += y * y;
        i += 1;
    }
    let denom = libm::sqrtf(na) * libm::sqrtf(nb);
    if denom > 0.0 {
        dot / denom
    } else {
        0.0
    }
}

/// Score `count` vectors (flat `[count x dims]` f32 arena) against `query`,
/// writing one cosine score per vector into `scores`.
///
/// Work is claimed in chunks of `chunk` vectors from the shared atomic
/// `cursor`: multiple wasm instances may call this concurrently over the same
/// arena, and every index is processed exactly once (lock-free chunk claim).
///
/// Returns a `STATUS_*` code; never panics across the FFI boundary.
///
/// # Safety
///
/// Caller guarantees (host contract):
/// - `vectors` points to `count * dims` initialized, 4-byte aligned `f32`;
/// - `query` points to `dims` initialized, 4-byte aligned `f32`;
/// - `scores` points to `count` writable, 4-byte aligned `f32` slots;
/// - `cursor` points to one writable, 4-byte aligned `u32`, shared by every
///   concurrent caller of this batch and initialized to `0` before the first
///   call. `count + (concurrent callers x chunk)` must stay below `u32::MAX`
///   (chunk claiming wraps otherwise);
/// - all pointers remain valid for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn score_batch_chunked(
    vectors: *const f32,
    count: u32,
    dims: u32,
    query: *const f32,
    scores: *mut f32,
    cursor: *mut u32,
    chunk: u32,
) -> u32 {
    if vectors.is_null() || query.is_null() || scores.is_null() || cursor.is_null() {
        return STATUS_NULL;
    }
    if dims == 0 || chunk == 0 {
        return STATUS_BAD_ARGS;
    }
    let dims = dims as usize;
    let count = count as usize;

    // SAFETY: per the documented contract, `cursor` is a valid, 4-byte
    // aligned, writable u32 shared by every concurrent caller of this batch;
    // `AtomicU32` has the same layout and alignment as `u32`.
    let cursor = unsafe { &*cursor.cast::<AtomicU32>() };

    // SAFETY: `query` is valid for `dims` reads (contract). Read-only shared
    // access: concurrent readers are fine.
    let query = unsafe { core::slice::from_raw_parts(query, dims) };

    loop {
        // Lock-free chunk claim: each iteration of each caller gets a
        // disjoint chunk; `Relaxed` is sufficient (the claim is the only
        // cross-thread ordering that matters, and it is enforced by the
        // atomic RMW itself).
        let start = cursor.fetch_add(chunk, Ordering::Relaxed) as usize;
        if start >= count {
            break;
        }
        let end = start.saturating_add(chunk as usize).min(count);
        for i in start..end {
            // SAFETY: `vectors` is valid for `count * dims` reads and `i` is
            // within `count` (contract); `scores` is valid for `count` writes,
            // and chunk claiming guarantees each index is written by exactly
            // one caller. Writes go through raw pointers (no `&mut` aliasing
            // across callers).
            let score = unsafe {
                let v = core::slice::from_raw_parts(vectors.add(i * dims), dims);
                cosine_similarity(v, query)
            };
            unsafe { scores.add(i).write(score) };
        }
    }
    STATUS_OK
}

#[cfg(all(not(test), target_arch = "wasm32"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
