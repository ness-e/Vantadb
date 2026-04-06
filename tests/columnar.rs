use connectomedb::node::{UnifiedNode, VectorRepresentations};
use connectomedb::columnar::nodes_to_record_batch;

#[test]
fn test_arrow_conversion() {
    let mut node1 = UnifiedNode::new(1);
    node1.vector = VectorRepresentations::Full(vec![4.2]);
    let mut node2 = UnifiedNode::new(2);
    node2.vector = VectorRepresentations::Full(vec![7.1]);

    let nodes = vec![node1, node2];
    let batch = nodes_to_record_batch(&nodes).unwrap();

    assert_eq!(batch.num_columns(), 2);
    assert_eq!(batch.num_rows(), 2);
}
