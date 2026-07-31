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

    /// Reset to default state (50/0). Useful for benchmarks.
    pub fn reset() {
        AUTO_TUNE.ef_search.store(50, Ordering::Relaxed);
        AUTO_TUNE.hit_streak.store(0, Ordering::Relaxed);
    }

    /// Set ef_search to any value. Used by benchmarks to bypass auto-tuning.
    pub fn set_ef(v: usize) {
        AUTO_TUNE.ef_search.store(v, Ordering::Relaxed);
        AUTO_TUNE.hit_streak.store(0, Ordering::Relaxed);
    }

    pub fn report_brute_fallback() {
        let current = AUTO_TUNE.ef_search.load(Ordering::Relaxed);
        let new = (current + current / 2).min(Self::MAX_EF);
        AUTO_TUNE.ef_search.store(new, Ordering::Relaxed);
        AUTO_TUNE.hit_streak.store(0, Ordering::Relaxed);
        crate::metrics::core::record_auto_tune_ef(new);
    }

    pub fn report_success() {
        let streak = AUTO_TUNE.hit_streak.fetch_add(1, Ordering::Relaxed);
        if streak > 0 && streak % 10 == 0 {
            let current = AUTO_TUNE.ef_search.load(Ordering::Relaxed);
            let new = (current / 2).max(Self::MIN_EF);
            AUTO_TUNE.ef_search.store(new, Ordering::Relaxed);
            crate::metrics::core::record_auto_tune_ef(new);
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
    fn brute_fallback_increases_ef() {
        with_reset(|| {
            let before = AutoTune::current_ef();
            AutoTune::report_brute_fallback();
            assert_eq!(AutoTune::current_ef(), before + before / 2); // 50 + 25 = 75
            AutoTune::report_brute_fallback();
            let second = 75 + 75 / 2; // 75 + 37 = 112
            assert_eq!(AutoTune::current_ef(), second);
        });
    }

    #[test]
    fn ten_successes_halves_ef() {
        with_reset(|| {
            AutoTune::report_brute_fallback(); // 50 → 75
            AutoTune::report_brute_fallback(); // 75 → 112
            let doubled = AutoTune::current_ef();
            assert_eq!(doubled, 112);
            // 11 successes: 10th triggers half (112/2=56), 11th does nothing
            for _ in 0..11 {
                AutoTune::report_success();
            }
            assert_eq!(AutoTune::current_ef(), 56);
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

    /// Integration-style: simulate consecutive fallback→success cycles
    /// as the real code path would trigger them (no engine needed).
    #[test]
    fn repeated_fallbacks_increase_ef() {
        with_reset(|| {
            let initial = AutoTune::current_ef();
            assert_eq!(initial, 50);

            // Simulate 5 consecutive ANN fallbacks
            for _ in 0..5 {
                AutoTune::report_brute_fallback();
            }
            let after_fallbacks = AutoTune::current_ef();
            // 50→75→112→168→252→378
            assert!(
                after_fallbacks > initial * 5,
                "ef should increase significantly after 5 fallbacks, got {after_fallbacks}"
            );

            // Simulate 55 successful queries (triggers 5 halvings)
            for _ in 0..55 {
                AutoTune::report_success();
            }
            let after_successes = AutoTune::current_ef();
            // 378 → 189 → 94 → 47 → 23 → 11
            assert!(
                after_successes < after_fallbacks,
                "ef should decrease after successes, was {after_fallbacks}, now {after_successes}"
            );
            assert!(
                after_successes >= AutoTune::MIN_EF,
                "ef should not drop below MIN_EF, got {after_successes}"
            );
        });
    }
}
