use connectomedb::storage::StorageEngine;
use connectomedb::node::{UnifiedNode, NodeFlags, NeuronType, FieldValue, VectorRepresentations, CognitiveUnit};
use connectomedb::executor::{Executor, ExecutionResult};
use tempfile::tempdir;

#[tokio::test]
async fn test_rehydration_core() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("db");
    let storage = StorageEngine::open(path.to_str().unwrap()).unwrap();

    // 1. Create Summary Node (Stale context, trust < 0.4)
    let summary_id: u64 = 999;
    let mut summary = UnifiedNode::new(summary_id);
    summary.neuron_type = NeuronType::STNeuron;
    summary.trust_score = 0.3;
    summary.set_field("type", FieldValue::String("NeuralSummary".to_string()));
    storage.insert(&summary).unwrap();

    // 2. Create Component Nodes and then tombstone them to shadow_kernel
    let mut comp1 = UnifiedNode::new(1001);
    comp1.add_edge(summary_id, "belonged_to");
    comp1.vector = VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
    comp1.flags.set(NodeFlags::HAS_VECTOR);

    let mut comp2 = UnifiedNode::new(1002);
    comp2.add_edge(summary_id, "belonged_to");
    comp2.vector = VectorRepresentations::Full(vec![0.4, 0.5, 0.6]);
    comp2.flags.set(NodeFlags::HAS_VECTOR);

    storage.insert(&comp1).unwrap();
    storage.insert(&comp2).unwrap();

    // 3. Simulate Bayesian Forgetting: push to Shadow Archive
    storage.delete(1001, "Bayesian Forgetting").unwrap();
    storage.delete(1002, "Bayesian Forgetting").unwrap();

    // Verification: They are dead in the main index
    assert!(storage.get(1001).unwrap().is_none(), "Node 1001 should be dead");
    assert!(storage.get(1002).unwrap().is_none(), "Node 1002 should be dead");

    // 4. Rehydration: Recover archaeological memories from shadow_kernel
    let resurrected = storage.rehydrate(summary_id).expect("Rehydration failed");

    assert_eq!(resurrected.len(), 2, "Should rehydrate exactly 2 forgotten nodes");

    for r in &resurrected {
        // Flags: TOMBSTONE cleared, REHYDRATED set, ACTIVE
        assert!(!r.flags.is_tombstone(), "Tombstone flag must be cleared");
        assert!(r.flags.is_set(NodeFlags::REHYDRATED), "Must carry REHYDRATED provenance");
        assert!(r.flags.is_active(), "Must be active");
        assert_eq!(r.neuron_type, NeuronType::STNeuron, "Must be promoted to STNeuron");

        // Verify they are now in cortex_ram 
        let from_ram = storage.get(r.id).unwrap();
        assert!(from_ram.is_some(), "Rehydrated node must be findable in cortex_ram");
    }

    // 5. StaleContext trigger via IQL (correct syntax: FROM Node#ID)
    let executor = Executor::new(&storage);
    let query = format!("FROM Node#{}", summary_id);
    let result = executor.execute_hybrid(&query).await.unwrap();

    match result {
        ExecutionResult::StaleContext(id) => {
            assert_eq!(id, summary_id, "StaleContext must reference the low-trust summary");
        }
        ExecutionResult::Read(nodes) => {
            // If the query returned the node, manually verify trust is low
            assert!(!nodes.is_empty(), "Should have found the summary node");
            let node = &nodes[0];
            assert!(
                node.trust_score() >= 0.4,
                "If Read was returned instead of StaleContext, trust_score must have been >= 0.4 (got {})",
                node.trust_score()
            );
        }
        _ => panic!("Unexpected result variant"),
    }

    // Windows: RocksDB mantiene file handles abiertos.
    // drop(storage) libera el DB antes de limpiar el tempdir.
    drop(storage);
    let _ = dir.close();
}
