use iadbms::governor::{ResourceGovernor, ALLOCATED_BYTES};
use std::sync::atomic::Ordering;

#[test]
fn test_oom_protection() {
    let governor = ResourceGovernor::new(1024 * 1024, 1000); // 1MB limit
    
    // Request 500KB - should succeed
    assert!(governor.request_allocation(512 * 1024).is_ok());
    assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 512 * 1024);

    // Request 600KB - should fail
    assert!(governor.request_allocation(600 * 1024).is_err());
    
    // Ensure memory wasn't leaked dynamically
    assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 512 * 1024);

    // Free 500KB
    governor.free_allocation(512 * 1024);
    assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 0);
}
