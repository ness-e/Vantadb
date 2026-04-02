use iadbms::storage::StorageEngine;
use iadbms::node::UnifiedNode;
use iadbms::graph::GraphTraverser;

#[test]
fn test_bfs_traversal() {
    let storage = StorageEngine::open("tests_graph_db").unwrap();
    
    // root -> 2 -> 3
    //   |----> 4
    let mut node1 = UnifiedNode::new(1);
    node1.add_edge(2, "relates_to");
    node1.add_edge(4, "relates_to");

    let mut node2 = UnifiedNode::new(2);
    node2.add_edge(3, "relates_to");

    let node3 = UnifiedNode::new(3);
    let node4 = UnifiedNode::new(4);

    storage.insert(&node1).unwrap();
    storage.insert(&node2).unwrap();
    storage.insert(&node3).unwrap();
    storage.insert(&node4).unwrap();

    let traverser = GraphTraverser::new(&storage);
    
    // Depth 1: Should reach 1, 2, 4 but not 3
    let res_d1 = traverser.bfs_traverse(&[1], 1).unwrap();
    assert!(res_d1.contains(&1));
    assert!(res_d1.contains(&2));
    assert!(res_d1.contains(&4));
    assert!(!res_d1.contains(&3));

    // Depth 2: Should reach 3 as well
    let res_d2 = traverser.bfs_traverse(&[1], 2).unwrap();
    assert_eq!(res_d2.len(), 4);
    assert!(res_d2.contains(&3));
}
