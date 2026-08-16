//! Per-search profiling — vfile I/O reads, compute time, and candidate
//! counters. Debug builds track real counters; release builds are a
//! zero-cost no-op type.

#[cfg(debug_assertions)]
pub(crate) struct SearchProfile {
    vfile_reads: u64,
    unique_pages: std::collections::HashSet<u64>,
    compute_ns: u64,
    candidates_seen: u64,
    start: std::time::Instant,
    compute_start: std::time::Instant,
}

#[cfg(debug_assertions)]
impl SearchProfile {
    pub(crate) fn new() -> Self {
        Self {
            vfile_reads: 0,
            unique_pages: std::collections::HashSet::new(),
            compute_ns: 0,
            candidates_seen: 0,
            start: std::time::Instant::now(),
            compute_start: std::time::Instant::now(),
        }
    }

    pub(super) fn start_compute(&mut self) {
        self.compute_start = std::time::Instant::now();
    }

    pub(super) fn end_compute(&mut self) {
        self.compute_ns += self.compute_start.elapsed().as_nanos() as u64;
    }

    pub(super) fn record_vfile_candidate(&mut self, storage_offset: u64) {
        self.vfile_reads += 2;
        self.candidates_seen += 1;
        self.unique_pages.insert(storage_offset >> 12);
    }

    pub(super) fn record_vfile_entry(&mut self, storage_offset: u64) {
        self.vfile_reads += 1;
        self.unique_pages.insert(storage_offset >> 12);
    }

    pub(super) fn log(&self, ef_search: usize, top_k: usize) {
        let elapsed = self.start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let compute_ms = self.compute_ns as f64 / 1_000_000.0;
        let io_ms = (elapsed_ms - compute_ms).max(0.0);
        tracing::debug!(
            "search_profile: ef={} top_k={} {:.2}ms total ({:.2}ms compute, {:.2}ms io), \
             {} vfile_reads, {} unique_pages, {} candidates",
            ef_search,
            top_k,
            elapsed_ms,
            compute_ms,
            io_ms,
            self.vfile_reads,
            self.unique_pages.len(),
            self.candidates_seen,
        );
    }
}

#[cfg(not(debug_assertions))]
pub(crate) struct SearchProfile;

#[cfg(not(debug_assertions))]
impl SearchProfile {
    pub(crate) fn new() -> Self {
        Self
    }
    pub(super) fn start_compute(&mut self) {}
    pub(super) fn end_compute(&mut self) {}
    pub(super) fn record_vfile_candidate(&mut self, _storage_offset: u64) {}
    pub(super) fn record_vfile_entry(&mut self, _storage_offset: u64) {}
    pub(super) fn log(&self, _ef_search: usize, _top_k: usize) {}
}
