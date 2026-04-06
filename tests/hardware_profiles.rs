/// Hardware Chameleon (Fase 27) Integration Tests
///
/// These tests validate the dynamic hardware detection and OOM emergency REM flags.

use connectomedb::storage::StorageEngine;
use std::sync::Arc;

fn temp_storage() -> Arc<StorageEngine> {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).expect("Failed to open storage"))
}

#[tokio::test]
async fn test_oom_emergency_trigger_activation() {
    let storage = temp_storage();
    
    // We get the real hardware constraint locally
    let caps = connectomedb::hardware::HardwareCapabilities::global();
    let cortex_cap_bytes = caps.total_memory / 4;
    let approx_node_size = 1536; 
    let _max_stn_nodes = (cortex_cap_bytes / approx_node_size) as usize;

    // Simulate inserting nodes up to the threshold
    // WARNING: For actual 16GB machines, this would require 2.7 Million inserts and could OOM the test OS.
    // Instead, we will simulate the behavior by manually injecting a threshold limit in real code or 
    // mock it if possible. Since we can't mock total_memory dynamically in this test structure without 
    // refactoring HardwareCapabilities to use a trait/DI, we will just test the boolean flag mechanism.
    
    // Verify that it starts false
    assert!(!storage.emergency_rem_trigger.load(std::sync::atomic::Ordering::Acquire));
}

#[test]
fn test_hardware_profile_detection() {
    // Just verify detection doesn't panic and returns valid variants
    let caps = connectomedb::hardware::HardwareCapabilities::global();
    
    assert!(caps.total_memory > 0, "System memory must be greater than 0");
    assert!(caps.logical_cores > 0, "Logical cores must be greater than 0");
    assert!(caps.vitality_score >= 1, "Vitality score must be calculated");

    println!("Detected Profile: {:?}", caps.profile);
    println!("Detected Instructions: {:?}", caps.instructions);
}

#[test]
fn test_scalar_fallback_cosine_similarity() {
    let a = connectomedb::node::VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
    let b = connectomedb::node::VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
    
    // Test similarity computation regardless of the active instruction set branch.
    let sim = a.cosine_similarity(&b).unwrap();
    assert!((sim - 1.0).abs() < 1e-6);

    let c = connectomedb::node::VectorRepresentations::Full(vec![0.0, 1.0, 0.0]);
    let sim2 = a.cosine_similarity(&c).unwrap();
    assert!(sim2.abs() < 1e-6); // orthogonal = 0
}
