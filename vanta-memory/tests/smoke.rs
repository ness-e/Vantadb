//! Smoke test: the crate links and the LLM-free default holds.
//! (Contract MEM-08a, D19 — dedicated tests per task.)

#[test]
fn crate_links() {
    // Trivial: proves the crate compiles/links as a workspace member.
    assert_eq!(vanta_memory::name(), "vanta-memory");
}

#[test]
fn llm_driver_feature_is_opt_in() {
    // Default build must NOT enable llm-driver (LLM-free guarantee).
    assert!(!cfg!(feature = "llm-driver"));
}
