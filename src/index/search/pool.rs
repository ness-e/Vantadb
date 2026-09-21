//! Per-thread pool of `NeighborVec` lists for reuse across `search_layer`
//! calls — reduces SmallVec heap allocations when neighbor lists exceed 32.

use crate::index::graph::NeighborVec;

// E2: Per-thread pool of `NeighborVec` for reuse in `search_layer`.
// Reduces SmallVec heap allocations when neighbor lists exceed 32 elements.
thread_local! {
    static NL_POOL: std::cell::RefCell<Vec<NeighborVec>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Return a `NeighborVec` to the pool for reuse. Clears contents first.
pub(super) fn give_nl(mut v: NeighborVec) {
    v.clear();
    NL_POOL.with(|pool| pool.borrow_mut().push(v));
}
