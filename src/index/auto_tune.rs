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

#[cfg(test)]
mod tests {
    use super::*;

    /// Run f with the global state temporarily reset to 50/0.
    fn with_reset(f: impl FnOnce()) {
        AUTO_TUNE.ef_search.store(50, Ordering::Relaxed);
        AUTO_TUNE.hit_streak.store(0, Ordering::Relaxed);
        f();
    }

    #[test]
    fn brute_fallback_doubles_ef() {
        with_reset(|| {
            let before = AutoTune::current_ef();
            AutoTune::report_brute_fallback();
            assert_eq!(AutoTune::current_ef(), before * 2);
            AutoTune::report_brute_fallback();
            assert_eq!(AutoTune::current_ef(), before * 4);
        });
    }

    #[test]
    fn ten_successes_halves_ef() {
        with_reset(|| {
            let before = AutoTune::current_ef();
            AutoTune::report_brute_fallback();
            AutoTune::report_brute_fallback();
            let doubled = AutoTune::current_ef();
            assert_eq!(doubled, before * 4);
            for _ in 0..11 {
                AutoTune::report_success();
            }
            assert_eq!(AutoTune::current_ef(), before * 2);
        });
    }

    #[test]
    fn ef_bounded_by_max() {
        with_reset(|| {
            for _ in 0..20 {
                AutoTune::report_brute_fallback();
            }
            assert_eq!(AutoTune::current_ef(), AutoTune::MAX_EF);
            for _ in 0..100 {
                AutoTune::report_success();
            }
            assert_eq!(AutoTune::current_ef(), AutoTune::MIN_EF);
        });
    }
}
