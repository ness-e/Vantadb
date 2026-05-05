//! Embedded SDK boundary certification.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use tempfile::tempdir;
use vantadb::sdk::{VantaEmbedded, VantaNodeInput, VantaOpenOptions, VantaValue};

#[test]
fn python_bridge_certification() {
    let mut harness = VantaHarness::new("API LAYER (PYTHON SDK)");

    harness.execute("Embedded SDK Boundary: CRUD + Search + Restart", || {
        let dir = tempdir().expect("Failed to create temp dir");
        let path = dir.path();
        let sdk = VantaEmbedded::open_with_options(
            path,
            VantaOpenOptions {
                memory_limit_bytes: Some(128 * 1024 * 1024),
                read_only: false,
            },
        )
        .expect("Failed to open embedded SDK");

        let mut input = VantaNodeInput::new(42);
        input.content = Some("sdk boundary".to_string());
        input.vector = Some(vec![1.0, 0.0, 0.0]);
        input
            .fields
            .insert("category".into(), VantaValue::String("python-sdk".into()));
        sdk.insert_node(input).expect("Insert failed");

        let node = sdk
            .get_node(42)
            .expect("Get failed")
            .expect("Node should exist");
        assert_eq!(node.id, 42);
        assert_eq!(
            node.fields.get("content"),
            Some(&VantaValue::String("sdk boundary".into()))
        );

        let hits = sdk
            .search_vector(&[1.0, 0.0, 0.0], 1)
            .expect("Search failed");
        assert_eq!(hits[0].node_id, 42);

        sdk.flush().expect("Flush failed");
        sdk.close().expect("Close failed");
        assert!(
            sdk.get_node(42).is_err(),
            "Closed embedded handle must reject further operations"
        );

        let reopened = VantaEmbedded::open(path).expect("Reopen failed");
        let reopened_node = reopened
            .get_node(42)
            .expect("Reopened get failed")
            .expect("Reopened node should exist");
        assert_eq!(reopened_node.id, 42);
        TerminalReporter::success("Stable embedded SDK boundary verified.");
    });

    harness.execute("Capabilities Surface", || {
        let dir = tempdir().expect("Failed to create temp dir");
        let sdk = VantaEmbedded::open(dir.path()).expect("Failed to open embedded SDK");
        let caps = sdk.capabilities();
        assert!(caps.persistence);
        assert!(caps.vector_search);
        assert!(!caps.iql_queries);
        assert!(!caps.read_only);
        TerminalReporter::success("Capabilities surface is stable and additive.");
    });
}
