//! ScalarIndex.remove coverage — engine-level wiring for FIND-39.

use crate::node::FieldValue;

use super::{in_memory_engine, sample_node};

#[test]
fn test_scalar_remove() {
    let engine = in_memory_engine();
    let si = engine
        .scalar_index
        .as_ref()
        .expect("scalar_index should exist");

    // ── Direct ScalarIndex API ─────────────────────────────────
    si.insert("color", &FieldValue::String("red".into()), 1);
    si.insert("color", &FieldValue::String("red".into()), 2);
    si.insert("color", &FieldValue::String("blue".into()), 3);

    let reds = si.lookup("color", &FieldValue::String("red".into()));
    assert_eq!(reds.len(), 2);
    assert!(reds.contains(&1) && reds.contains(&2));

    // remove one of two sharing same value — leaves the other
    si.remove("color", &FieldValue::String("red".into()), 1);
    assert_eq!(
        si.lookup("color", &FieldValue::String("red".into())),
        vec![2]
    );

    // no-op: missing field / value / id must not panic and must leave state intact
    si.remove("missing_field", &FieldValue::String("x".into()), 999);
    si.remove("color", &FieldValue::String("nonexistent".into()), 2);
    si.remove("color", &FieldValue::String("red".into()), 999);
    assert_eq!(
        si.lookup("color", &FieldValue::String("red".into())),
        vec![2],
        "wrong-id remove must be no-op"
    );

    // cross-field isolation
    si.insert("size", &FieldValue::Int(10), 2);
    si.remove("color", &FieldValue::String("red".into()), 2);
    assert!(
        si.lookup("color", &FieldValue::String("red".into()))
            .is_empty(),
        "color:red should be empty after removing last holder"
    );
    assert_eq!(si.lookup("size", &FieldValue::Int(10)), vec![2]);
    assert_eq!(
        si.lookup("color", &FieldValue::String("blue".into())),
        vec![3]
    );

    // remove last holder of a value
    si.remove("color", &FieldValue::String("blue".into()), 3);
    assert!(si
        .lookup("color", &FieldValue::String("blue".into()))
        .is_empty());

    // ── Engine wiring: insert/overwrite/delete must keep index current ──
    let mut node10 = sample_node(10);
    node10
        .relational
        .insert("tag".to_string(), FieldValue::String("alpha".into()));
    engine.insert(&node10).expect("insert alpha");
    assert_eq!(
        si.lookup("tag", &FieldValue::String("alpha".into())),
        vec![10]
    );

    let mut node10b = sample_node(10);
    node10b
        .relational
        .insert("tag".to_string(), FieldValue::String("beta".into()));
    engine.insert(&node10b).expect("overwrite beta");
    assert!(
        si.lookup("tag", &FieldValue::String("alpha".into()))
            .is_empty(),
        "old value alpha must be removed on overwrite"
    );
    assert_eq!(
        si.lookup("tag", &FieldValue::String("beta".into())),
        vec![10]
    );

    engine.delete(10, "test").expect("delete");
    assert!(
        si.lookup("tag", &FieldValue::String("beta".into()))
            .is_empty(),
        "delete must clear scalar entry"
    );
}
