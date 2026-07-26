//! Integration tests for JOIN and subquery execution.
//!
//! Tests end-to-end through the executor via `Statement::Select`,
//! verifying that NestedLoopJoin and subquery filtering produce
//! correct combined results across entity types.

use std::collections::BTreeMap;

use vantadb::config::VantaConfig;
use vantadb::executor::Executor;
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::query::{Condition, FromClause, RelOp, SelectStatement, Statement, SubqueryCondition};
use vantadb::storage::{BackendKind, StorageEngine};

fn setup() -> (StorageEngine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
        .expect("Failed to open StorageEngine");
    (storage, dir)
}

fn insert_person(storage: &StorageEngine, id: u128, name: &str, addr_id: u128) {
    let mut node = UnifiedNode::new(id);
    node.relational
        .insert("type".into(), FieldValue::String("Person".into()));
    node.relational
        .insert("name".into(), FieldValue::String(name.into()));
    node.relational
        .insert("addr_id".into(), FieldValue::Int(addr_id as i64));
    storage.insert(&node).unwrap();
}

fn insert_address(storage: &StorageEngine, id: u128, city: &str) {
    let mut node = UnifiedNode::new(id);
    node.relational
        .insert("type".into(), FieldValue::String("Address".into()));
    node.relational
        .insert("city".into(), FieldValue::String(city.into()));
    node.relational
        .insert("id".into(), FieldValue::Int(id as i64));
    storage.insert(&node).unwrap();
}

fn insert_product(storage: &StorageEngine, id: u128, name: &str, price: i64) {
    let mut node = UnifiedNode::new(id);
    node.relational
        .insert("type".into(), FieldValue::String("Product".into()));
    node.relational
        .insert("name".into(), FieldValue::String(name.into()));
    node.relational
        .insert("price".into(), FieldValue::Int(price));
    storage.insert(&node).unwrap();
}

// ─── JOIN between two entity types via Statement::Select ─────────

#[test]
fn test_join_two_entities_returns_combined_results() {
    let (storage, _dir) = setup();
    let ex = Executor::new(&storage);

    // Insert Persons
    insert_person(&storage, 1, "Alice", 10);
    insert_person(&storage, 2, "Bob", 20);

    // Insert Addresses
    insert_address(&storage, 10, "New York");
    insert_address(&storage, 20, "London");

    // Build: SELECT * FROM Person p JOIN Address a ON p.addr_id = a.id
    let stmt = Statement::Select(SelectStatement {
        projections: vec![],
        from: FromClause::Join {
            left: Box::new(FromClause::Single {
                entity: "*".into(),
                alias: "p".into(),
            }),
            right: Box::new(FromClause::Single {
                entity: "*".into(),
                alias: "a".into(),
            }),
            left_field: "p.addr_id".into(),
            right_field: "a.id".into(),
        },
        where_clause: None,
        subquery_conditions: vec![],
        temperature: None,
    });

    let result = ex.execute_statement(stmt).unwrap();
    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
            // Should have 2 joined results (Alice→New York, Bob→London)
            assert_eq!(nodes.len(), 2, "JOIN should produce 2 combined rows");

            // Each result should have fields from both sides
            let alice_row = nodes
                .iter()
                .find(|n| n.relational.get("name") == Some(&FieldValue::String("Alice".into())))
                .expect("Alice should be in results");

            assert_eq!(
                alice_row.relational.get("city"),
                Some(&FieldValue::String("New York".into())),
                "Alice should be joined with New York"
            );

            let bob_row = nodes
                .iter()
                .find(|n| n.relational.get("name") == Some(&FieldValue::String("Bob".into())))
                .expect("Bob should be in results");

            assert_eq!(
                bob_row.relational.get("city"),
                Some(&FieldValue::String("London".into())),
                "Bob should be joined with London"
            );
        }
        _ => panic!("Expected Read result"),
    }
}

#[test]
fn test_join_with_where_filter() {
    let (storage, _dir) = setup();
    let ex = Executor::new(&storage);

    insert_person(&storage, 1, "Alice", 10);
    insert_person(&storage, 2, "Bob", 20);
    insert_person(&storage, 3, "Charlie", 10);
    insert_address(&storage, 10, "New York");
    insert_address(&storage, 20, "London");

    // SELECT * FROM Person p JOIN Address a ON p.addr_id = a.id WHERE p.name = "Alice"
    let stmt = Statement::Select(SelectStatement {
        projections: vec![],
        from: FromClause::Join {
            left: Box::new(FromClause::Single {
                entity: "*".into(),
                alias: "p".into(),
            }),
            right: Box::new(FromClause::Single {
                entity: "*".into(),
                alias: "a".into(),
            }),
            left_field: "p.addr_id".into(),
            right_field: "a.id".into(),
        },
        where_clause: Some(vec![Condition::Relational(
            "name".into(),
            RelOp::Eq,
            FieldValue::String("Alice".into()),
        )]),
        subquery_conditions: vec![],
        temperature: None,
    });

    let result = ex.execute_statement(stmt).unwrap();
    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
            assert_eq!(nodes.len(), 1, "Filtered JOIN should return 1 row");
            assert_eq!(
                nodes[0].relational.get("city"),
                Some(&FieldValue::String("New York".into())),
                "Alice should be joined with New York"
            );
        }
        _ => panic!("Expected Read result"),
    }
}

#[test]
fn test_join_no_matches() {
    let (storage, _dir) = setup();
    let ex = Executor::new(&storage);

    // Insert Person with addr_id that doesn't match any Address
    insert_person(&storage, 1, "Ghost", 999);
    insert_address(&storage, 10, "Nowhere");

    let stmt = Statement::Select(SelectStatement {
        projections: vec![],
        from: FromClause::Join {
            left: Box::new(FromClause::Single {
                entity: "*".into(),
                alias: "p".into(),
            }),
            right: Box::new(FromClause::Single {
                entity: "*".into(),
                alias: "a".into(),
            }),
            left_field: "p.addr_id".into(),
            right_field: "a.id".into(),
        },
        where_clause: None,
        subquery_conditions: vec![],
        temperature: None,
    });

    let result = ex.execute_statement(stmt).unwrap();
    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
            assert_eq!(nodes.len(), 0, "No matching JOIN should return 0 rows");
        }
        _ => panic!("Expected Read result"),
    }
}

// ─── Scalar subquery in WHERE ───────────────────────────────────

#[test]
fn test_subquery_where_scalar() {
    let (storage, _dir) = setup();
    let ex = Executor::new(&storage);

    // Insert products
    insert_product(&storage, 1, "Cheap Widget", 5);
    insert_product(&storage, 2, "Mid Widget", 15);
    insert_product(&storage, 3, "Priced Widget", 25);
    insert_product(&storage, 4, "Expensive Widget", 100);

    // Build the subquery: SELECT price FROM Product (will get first result's price)
    let subq = SelectStatement {
        projections: vec!["price".into()],
        from: FromClause::Single {
            entity: "*".into(),
            alias: "sq".into(),
        },
        where_clause: Some(vec![Condition::Relational(
            "name".into(),
            RelOp::Eq,
            FieldValue::String("Priced Widget".into()),
        )]),
        subquery_conditions: vec![],
        temperature: None,
    };

    // Outer query: SELECT * FROM Product WHERE price >= (SELECT price FROM ...)
    // This uses the subquery_conditions field for scalar subquery comparison
    let stmt = Statement::Select(SelectStatement {
        projections: vec![],
        from: FromClause::Single {
            entity: "*".into(),
            alias: "p".into(),
        },
        where_clause: None,
        subquery_conditions: vec![SubqueryCondition {
            field: "price".into(),
            op: RelOp::Gte,
            subquery: Box::new(subq),
        }],
        temperature: None,
    });

    let result = ex.execute_statement(stmt).unwrap();
    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
            // The subquery returns Priced Widget (price=25).
            // So outer query should return products with price >= 25.
            assert_eq!(
                nodes.len(),
                2,
                "Should match Priced Widget (25) and Expensive Widget (100)"
            );
            let prices: Vec<i64> = nodes
                .iter()
                .filter_map(|n| n.relational.get("price"))
                .filter_map(|v| {
                    if let FieldValue::Int(i) = v {
                        Some(*i)
                    } else {
                        None
                    }
                })
                .collect();
            assert!(prices.contains(&25), "Should include price 25");
            assert!(prices.contains(&100), "Should include price 100");
        }
        _ => panic!("Expected Read result"),
    }
}

#[test]
fn test_subquery_where_with_exact_match() {
    let (storage, _dir) = setup();
    let ex = Executor::new(&storage);

    insert_product(&storage, 1, "Widget A", 10);
    insert_product(&storage, 2, "Widget B", 20);
    insert_product(&storage, 3, "Target", 20);

    // Subquery: SELECT price FROM Product WHERE name = "Widget B" (price=20)
    let subq = SelectStatement {
        projections: vec!["price".into()],
        from: FromClause::Single {
            entity: "*".into(),
            alias: "sq".into(),
        },
        where_clause: Some(vec![Condition::Relational(
            "name".into(),
            RelOp::Eq,
            FieldValue::String("Widget B".into()),
        )]),
        subquery_conditions: vec![],
        temperature: None,
    };

    // Outer: SELECT * FROM Product WHERE price = (SELECT ...)
    let stmt = Statement::Select(SelectStatement {
        projections: vec![],
        from: FromClause::Single {
            entity: "*".into(),
            alias: "p".into(),
        },
        where_clause: None,
        subquery_conditions: vec![SubqueryCondition {
            field: "price".into(),
            op: RelOp::Eq,
            subquery: Box::new(subq),
        }],
        temperature: None,
    });

    let result = ex.execute_statement(stmt).unwrap();
    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
            // Widget B (price=20) and Target (price=20) both match
            assert_eq!(nodes.len(), 2, "Two products with price=20");
            assert!(
                nodes
                    .iter()
                    .any(|n| n.relational.get("name")
                        == Some(&FieldValue::String("Widget B".into()))),
                "Widget B should match"
            );
            assert!(
                nodes.iter().any(|n| n.relational.get("name") == Some(&FieldValue::String("Target".into()))),
                "Target should match"
            );
        }
        _ => panic!("Expected Read result"),
    }
}

// ─── Combined JOIN + WHERE + projection via execute_hybrid ──────

#[test]
fn test_execute_join_through_hybrid_parser() {
    let (storage, _dir) = setup();
    let ex = Executor::new(&storage);

    // Insert data using INSERT statements
    // Both use the same type "Entity" because the parser ident doesn't accept `*`.
    // This works: join ON addr_id = id correctly links Alice (addr_id=10, but no `id` field)
    // with the address node (id=10, city=Paris).
    ex.execute_statement(Statement::Insert(vantadb::query::InsertStatement {
        node_id: 1,
        node_type: "Entity".into(),
        fields: {
            let mut f = BTreeMap::new();
            f.insert("name".into(), FieldValue::String("Alice".into()));
            f.insert("addr_id".into(), FieldValue::Int(10));
            f
        },
        vector: None,
    }))
    .unwrap();

    ex.execute_statement(Statement::Insert(vantadb::query::InsertStatement {
        node_id: 10,
        node_type: "Entity".into(),
        fields: {
            let mut f = BTreeMap::new();
            f.insert("city".into(), FieldValue::String("Paris".into()));
            f.insert("id".into(), FieldValue::Int(10));
            f
        },
        vector: None,
    }))
    .unwrap();

    // Execute SELECT with JOIN via the IQL parser
    let result = ex
        .execute_hybrid("SELECT * FROM Entity p JOIN Entity a ON p.addr_id = a.id")
        .unwrap();

    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
            assert_eq!(nodes.len(), 1, "One joined result expected");
            assert_eq!(
                nodes[0].relational.get("name"),
                Some(&FieldValue::String("Alice".into()))
            );
            assert_eq!(
                nodes[0].relational.get("city"),
                Some(&FieldValue::String("Paris".into()))
            );
        }
        _ => panic!("Expected Read result"),
    }
}

#[test]
fn test_subquery_through_hybrid_parser() {
    let (storage, _dir) = setup();
    let ex = Executor::new(&storage);

    // Insert products
    for (id, name, price) in [(1, "Hat", 15), (2, "Scarf", 25), (3, "Coat", 50)] {
        ex.execute_statement(Statement::Insert(vantadb::query::InsertStatement {
            node_id: id,
            node_type: "Item".into(),
            fields: {
                let mut f = BTreeMap::new();
                f.insert("name".into(), FieldValue::String(name.into()));
                f.insert("price".into(), FieldValue::Int(price));
                f
            },
            vector: None,
        }))
        .unwrap();
    }

    // Use "Item" as entity since the parser ident doesn't accept `*`.
    // Both outer and subquery scan the same entity type.
    let result = ex.execute_hybrid(
        "SELECT * FROM Item p WHERE price >= (SELECT price FROM Item sq WHERE name = \"Scarf\")"
    ).unwrap();

    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
            // Scarf (25) and Coat (50) have price >= 25
            assert_eq!(nodes.len(), 2, "Two products with price >= 25");
            let names: Vec<&str> = nodes
                .iter()
                .filter_map(|n| n.relational.get("name"))
                .filter_map(|v| {
                    if let FieldValue::String(s) = v {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            assert!(names.contains(&"Scarf"), "Scarf should match");
            assert!(names.contains(&"Coat"), "Coat should match");
            assert!(!names.contains(&"Hat"), "Hat (price=15) should not match");
        }
        _ => panic!("Expected Read result"),
    }
}

// ─── Edge cases ────────────────────────────────────────────────

#[test]
fn test_join_self_join() {
    let (storage, _dir) = setup();
    let ex = Executor::new(&storage);

    // Insert employees (id, name, manager_id)
    // Employee 1 manages 2
    // UnifiedNode.id is NOT automatically a relational field; the `id` field
    // must be set explicitly for ON condition matching.
    for (id, name, mgr_id) in [(1u128, "Alice", 0u128), (2, "Bob", 1)] {
        let mut node = UnifiedNode::new(id);
        node.relational
            .insert("id".into(), FieldValue::Int(id as i64));
        node.relational
            .insert("type".into(), FieldValue::String("Employee".into()));
        node.relational
            .insert("name".into(), FieldValue::String(name.into()));
        node.relational
            .insert("mgr_id".into(), FieldValue::Int(mgr_id as i64));
        storage.insert(&node).unwrap();
    }

    // Self-join: find employees and their managers
    // SELECT * FROM Employee e JOIN Employee m ON e.mgr_id = m.id
    let stmt = Statement::Select(SelectStatement {
        projections: vec![],
        from: FromClause::Join {
            left: Box::new(FromClause::Single {
                entity: "*".into(),
                alias: "e".into(),
            }),
            right: Box::new(FromClause::Single {
                entity: "*".into(),
                alias: "m".into(),
            }),
            left_field: "e.mgr_id".into(),
            right_field: "m.id".into(),
        },
        where_clause: None,
        subquery_conditions: vec![],
        temperature: None,
    });

    let result = ex.execute_statement(stmt).unwrap();
    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
            // Bob (mgr_id=1) joined with Alice (id=1)
            assert_eq!(nodes.len(), 1, "Only Bob has a matching manager");
            // The result has fields from both 'e' (Bob) and 'm' (Alice)
            // Bob's name takes priority on conflict, but they're different fields
        }
        _ => panic!("Expected Read result"),
    }
}

#[test]
fn test_join_empty_tables() {
    let (storage, _dir) = setup();
    let ex = Executor::new(&storage);

    // Both sides empty
    let stmt = Statement::Select(SelectStatement {
        projections: vec![],
        from: FromClause::Join {
            left: Box::new(FromClause::Single {
                entity: "*".into(),
                alias: "a".into(),
            }),
            right: Box::new(FromClause::Single {
                entity: "*".into(),
                alias: "b".into(),
            }),
            left_field: "a.x".into(),
            right_field: "b.y".into(),
        },
        where_clause: None,
        subquery_conditions: vec![],
        temperature: None,
    });

    let result = ex.execute_statement(stmt).unwrap();
    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
            assert_eq!(nodes.len(), 0, "JOIN on empty tables returns no rows");
        }
        _ => panic!("Expected Read result"),
    }
}

#[test]
fn test_select_basic_no_join() {
    let (storage, _dir) = setup();
    let ex = Executor::new(&storage);

    insert_person(&storage, 1, "Alice", 0);

    // Simple SELECT with no JOIN
    let stmt = Statement::Select(SelectStatement {
        projections: vec![],
        from: FromClause::Single {
            entity: "*".into(),
            alias: "p".into(),
        },
        where_clause: None,
        subquery_conditions: vec![],
        temperature: None,
    });

    let result = ex.execute_statement(stmt).unwrap();
    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
            assert_eq!(nodes.len(), 1, "Basic SELECT should return 1 row");
        }
        _ => panic!("Expected Read result"),
    }
}
