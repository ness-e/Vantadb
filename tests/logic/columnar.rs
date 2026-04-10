//! Columnar Engine & Arrow Integration Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{VantaHarness, TerminalReporter};
use vantadb::columnar::nodes_to_record_batch;
use vantadb::node::{UnifiedNode, VectorRepresentations};

#[test]
fn columnar_engine_certification() {
    let mut harness = VantaHarness::new("LOGIC LAYER (COLUMNAR ENGINE)");

    harness.execute("Arrow: UnifiedNode to RecordBatch Conversion", || {
        TerminalReporter::sub_step("Preparing heterogeneous node buffer...");
        let mut node1 = UnifiedNode::new(1);
        node1.vector = VectorRepresentations::Full(vec![4.2]);
        let mut node2 = UnifiedNode::new(2);
        node2.vector = VectorRepresentations::Full(vec![7.1]);

        let nodes = vec![node1, node2];
        let batch = nodes_to_record_batch(&nodes).expect("Arrow conversion failed");

        assert_eq!(batch.num_columns(), 2);
        assert_eq!(batch.num_rows(), 2);
        
        TerminalReporter::success("Apache Arrow record batch generated successfully.");
    });
}
