use std::sync::atomic::{AtomicUsize, Ordering};

static AUTO_TUNE: AutoTune = AutoTune::new();

pub struct AutoTune {
    ef_search: AtomicUsize,
    hit_streak: AtomicUsize,
}

impl AutoTune {
    const MIN_EF: usize = 10;
    const MAX_EF: usize = 2000;

    const fn new() -> Self {
        Self {
            ef_search: AtomicUsize::new(50),
            hit_streak: AtomicUsize::new(0),
        }
    }

    pub fn current_ef() -> usize {
        AUTO_TUNE.ef_search.load(Ordering::Relaxed)
    }

    pub fn report_brute_fallback() {
        let current = AUTO_TUNE.ef_search.load(Ordering::Relaxed);
        let new = (current * 2).min(Self::MAX_EF);
        AUTO_TUNE.ef_search.store(new, Ordering::Relaxed);
        AUTO_TUNE.hit_streak.store(0, Ordering::Relaxed);
    }

    pub fn report_success() {
        let streak = AUTO_TUNE.hit_streak.fetch_add(1, Ordering::Relaxed);
        if streak > 0 && streak % 10 == 0 {
            let current = AUTO_TUNE.ef_search.load(Ordering::Relaxed);
            let new = (current / 2).max(Self::MIN_EF);
            AUTO_TUNE.ef_search.store(new, Ordering::Relaxed);
        }
    }
}
