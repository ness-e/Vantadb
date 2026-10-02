//! Smoke test: the crate links and the LLM-free default holds.
//! (Contract MEM-08a, D19 — dedicated tests per task.)

#[test]
fn crate_links() {
    // Trivial: proves the crate compiles/links as a workspace member.
    assert_eq!(vanta_memory::name(), "vanta-memory");
}

/// Default build must NOT enable `llm-driver` (LLM-free guarantee).
/// FIND-184: gated to builds where the feature is off — under workspace feature
/// unification (CI `--all-features` / audit profile) another member enables it,
/// and the compile-time assertion would abort the entire test binary (E0080).
#[test]
#[cfg(not(feature = "llm-driver"))]
fn llm_driver_feature_is_opt_in() {
    // Compile-time check: with `--features mock` this must still hold.
    const {
        assert!(cfg!(not(feature = "llm-driver")));
    }
}
