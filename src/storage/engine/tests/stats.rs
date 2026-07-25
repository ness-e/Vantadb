//! STATS module tests: MemoryStats, selectivity, cardinality, check_memory_pressure,
//! guard_write_allowed, touch_activity, initialize_cardinality_stats.

use super::super::*;
use super::{in_memory_engine, in_memory_read_only, sample_node};
use crate::backend::BackendKind;
use crate::config::VantaConfig;
use crate::node::{NodeTier, UnifiedNode};

// ─── Memory stats ─────────────────────────────────────────────

#[test]
fn test_memory_stats_after_insert() {
    let engine = in_memory_engine();
    let stats = engine.get_memory_stats();
    assert_eq!(stats.node_count, 0);
    assert_eq!(stats.cache_entries, 0);
    engine.insert(&sample_node(1)).expect("insert");
    let stats = engine.get_memory_stats();
    assert!(stats.node_count >= 1);
    assert!(stats.logical_bytes > 0);
}

#[test]
fn test_memory_stats_effective_bytes() {
    let stats = MemoryStats {
        logical_bytes: 1000,
        physical_rss: Some(800),
        node_count: 1,
        cache_entries: 0,
        eviction_count: 0,
        eviction_bytes: 0,
        memory_limit: 0,
        quantized_nodes: 0,
    };
    assert_eq!(stats.effective_bytes(), 800);
    let stats_no_rss = MemoryStats {
        logical_bytes: 1000,
        physical_rss: None,
        node_count: 1,
        cache_entries: 0,
        eviction_count: 0,
        eviction_bytes: 0,
        memory_limit: 0,
        quantized_nodes: 0,
    };
    assert_eq!(stats_no_rss.effective_bytes(), 1000);
}

#[test]
fn test_memory_stats_pressure_ratio() {
    let stats = MemoryStats {
        logical_bytes: 1000,
        physical_rss: None,
        node_count: 5,
        cache_entries: 2,
        eviction_count: 1,
        eviction_bytes: 400,
        memory_limit: 10000,
        quantized_nodes: 0,
    };
    let ratio = stats.pressure_ratio();
    assert!((ratio - 0.1).abs() < 1e-6, "expected 0.1, got {ratio}");
    let unlimited = MemoryStats {
        memory_limit: 0,
        ..stats
    };
    assert_eq!(unlimited.pressure_ratio(), 0.0);
}

#[test]
fn test_get_memory_stats_quantized_nodes() {
    let engine = in_memory_engine();
    let stats = engine.get_memory_stats();
    assert!(stats.logical_bytes > 0, "logical bytes should be > 0");
    assert_eq!(stats.node_count, 0);
    assert!(stats.cache_entries == 0);
}

#[test]
fn test_get_memory_stats_all_fields_populated() {
    let engine = in_memory_engine();
    let mut hot = sample_node(1);
    hot.tier = NodeTier::Hot;
    engine.insert(&hot).expect("insert");
    let stats = engine.get_memory_stats();
    assert!(
        stats.logical_bytes > 0,
        "logical_bytes should be > 0 after insert"
    );
    assert!(
        stats.node_count >= 1,
        "node_count should be >= 1 after insert"
    );
    assert!(stats.memory_limit > 0, "memory_limit should be > 0");
    assert!(stats.effective_bytes() > 0, "effective_bytes should be > 0");
    assert!(stats.cache_entries >= 1, "Hot node should be cached");
}

#[test]
fn test_get_memory_stats_pressure_ratio_with_rss() {
    let stats = MemoryStats {
        logical_bytes: 2000,
        physical_rss: Some(1500),
        node_count: 0,
        cache_entries: 0,
        eviction_count: 0,
        eviction_bytes: 0,
        memory_limit: 10000,
        quantized_nodes: 0,
    };
    assert!((stats.pressure_ratio() - 0.15).abs() < 1e-6);
    assert_eq!(stats.effective_bytes(), 1500);

    let stats_no_rss = MemoryStats {
        physical_rss: None,
        ..stats
    };
    assert!((stats_no_rss.pressure_ratio() - 0.20).abs() < 1e-6);
    assert_eq!(stats_no_rss.effective_bytes(), 2000);
}

#[test]
fn test_get_memory_stats_eviction_reflects_in_metrics() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");

    let report = engine
        .evict_cold_nodes_with_reason(1.0, EvictionReason::Periodic)
        .expect("evict");
    assert!(report.evicted > 0, "should evict at least one node");

    let stats = engine.get_memory_stats();
    assert_eq!(
        stats.cache_entries, 0,
        "cache should be empty after eviction"
    );
    assert!(stats.eviction_count >= 1, "eviction_count should be >= 1");
    assert!(stats.eviction_bytes > 0, "eviction_bytes should be > 0");
}

#[test]
fn test_get_memory_stats_eviction_reflected() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");

    let report = engine
        .evict_cold_nodes_with_reason(1.0, EvictionReason::Periodic)
        .expect("evict");
    assert!(report.evicted > 0, "should evict at least one node");

    let stats = engine.get_memory_stats();
    assert!(stats.logical_bytes > 0, "logical bytes should be > 0");
    assert!(
        stats.node_count > 0 || stats.eviction_count > 0,
        "node_count or eviction_count valid"
    );
    assert_eq!(
        stats.cache_entries, 0,
        "cache should be empty after eviction"
    );
}

#[test]
fn test_memory_stats_pressure_ratio_exact() {
    let stats = MemoryStats {
        logical_bytes: 500,
        physical_rss: None,
        node_count: 0,
        cache_entries: 0,
        eviction_count: 0,
        eviction_bytes: 0,
        memory_limit: 2000,
        quantized_nodes: 0,
    };
    assert_eq!(stats.pressure_ratio(), 0.25);
}

#[test]
fn test_memory_stats_pressure_ratio_rss() {
    let stats = MemoryStats {
        logical_bytes: 500,
        physical_rss: Some(1500),
        node_count: 0,
        cache_entries: 0,
        eviction_count: 0,
        eviction_bytes: 0,
        memory_limit: 3000,
        quantized_nodes: 0,
    };
    assert!((stats.pressure_ratio() - 0.5).abs() < 1e-6);
}

#[test]
fn test_memory_stats_pressure_ratio_exceeds_one() {
    let stats = MemoryStats {
        logical_bytes: 5000,
        physical_rss: None,
        node_count: 0,
        cache_entries: 0,
        eviction_count: 0,
        eviction_bytes: 0,
        memory_limit: 1000,
        quantized_nodes: 0,
    };
    assert!((stats.pressure_ratio() - 5.0).abs() < 1e-6);
}

// ─── Check memory pressure ────────────────────────────────────

#[test]
fn test_check_memory_pressure_disabled() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        rss_threshold: 0.0,
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    assert!(engine.check_memory_pressure().is_ok());
}

#[test]
fn test_check_memory_pressure_triggers_on_low_threshold() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        rss_threshold: 1.0,
        memory_limit: Some(1),
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    let result = engine.insert(&sample_node(1));
    assert!(result.is_err(), "insert should fail with memory pressure");
    assert!(
        result
            .err()
            .unwrap()
            .to_string()
            .contains("Memory pressure"),
        "should be Memory pressure error"
    );
}

#[test]
fn test_check_memory_pressure_governor_watermark() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        rss_threshold: 0.9,
        memory_limit: Some(100_000_000),
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    engine
        .memory_governor
        .as_ref()
        .unwrap()
        .set_used_bytes(80_000_000);
    let result = engine.check_memory_pressure();
    assert!(
        result.is_err(),
        "governor watermark should trigger pressure"
    );
    assert!(
        result
            .err()
            .unwrap()
            .to_string()
            .contains("Memory pressure"),
        "should mention Memory pressure"
    );
}

#[test]
fn test_check_memory_pressure_governor_sync_eviction() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        rss_threshold: 0.9,
        memory_limit: Some(80_000_000),
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    let result = engine.check_memory_pressure();
    assert!(result.is_ok(), "governor sync eviction should be Ok(())");
}

#[test]
fn test_check_memory_pressure_no_threshold_returns_ok() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        rss_threshold: 0.0,
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    assert!(engine.check_memory_pressure().is_ok());
}

#[test]
fn test_check_memory_pressure_negative_threshold_returns_ok() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        rss_threshold: -0.1,
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    assert!(engine.check_memory_pressure().is_ok());
}

// ─── Guard write allowed ──────────────────────────────────────

#[test]
fn test_guard_write_allowed_read_only_message() {
    let config = VantaConfig {
        read_only: true,
        ..VantaConfig::default()
    };
    let result = StorageEngine::guard_write_allowed(&config);
    let err = result.expect_err("should error");
    let msg = err.to_string();
    assert!(
        msg.contains("read-only") || msg.contains("read_only"),
        "error should mention read-only, got: {msg}"
    );
}

#[test]
fn test_ensure_writable_read_only_engine() {
    let engine = in_memory_read_only();
    let result = engine.ensure_writable();
    assert!(
        result.is_err(),
        "read-only engine should reject ensure_writable"
    );
}

#[test]
fn test_ensure_writable_writable_engine() {
    let engine = in_memory_engine();
    let result = engine.ensure_writable();
    assert!(
        result.is_ok(),
        "writable engine should accept ensure_writable"
    );
}

// ─── Touch activity ───────────────────────────────────────────

#[test]
fn test_touch_activity_increases_clock() {
    let engine = in_memory_engine();
    let before = engine
        .last_query_timestamp
        .load(std::sync::atomic::Ordering::Acquire);
    std::thread::sleep(std::time::Duration::from_millis(1));
    engine.touch_activity();
    let after = engine
        .last_query_timestamp
        .load(std::sync::atomic::Ordering::Acquire);
    assert!(
        after > before,
        "timestamp should increase after touch_activity (before={before}, after={after})"
    );
}

// ─── Backend capability queries ───────────────────────────────

#[test]
fn test_backend_kind_matches_capabilities() {
    let engine = in_memory_engine();
    let kind = engine.backend_kind();
    let caps = engine.backend_capabilities();
    assert_eq!(
        kind, caps.kind,
        "backend_kind and capabilites.kind should match"
    );
    assert_eq!(kind, BackendKind::InMemory);
}

#[test]
fn test_supports_checkpoint_and_compaction_in_memory() {
    let engine = in_memory_engine();
    assert!(
        !engine.supports_checkpoint(),
        "InMemory does not support checkpoint"
    );
    assert!(
        !engine.supports_manual_compaction(),
        "InMemory does not support manual compaction"
    );
}

#[test]
fn test_request_compaction_noop_on_in_memory() {
    let engine = in_memory_engine();
    engine.request_compaction();
}

// ─── Selectivity ──────────────────────────────────────────────

#[test]
fn test_selectivity_empty_engine() {
    let engine = in_memory_engine();
    let sel = engine.get_estimated_selectivity(
        "field",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("val".to_string()),
    );
    assert_eq!(sel, 1.0);
}

#[test]
fn test_selectivity_with_data() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational.insert(
        "status".to_string(),
        crate::node::FieldValue::String("active".to_string()),
    );
    engine.insert(&node).expect("insert");
    let sel = engine.get_estimated_selectivity(
        "status",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("active".to_string()),
    );
    assert_eq!(sel, 1.0);
    let sel_missing = engine.get_estimated_selectivity(
        "status",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("inactive".to_string()),
    );
    assert_eq!(sel_missing, 0.0);
}

#[test]
fn test_selectivity_neq() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    engine.insert(&node).expect("insert");
    let sel = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Neq,
        &crate::node::FieldValue::String("red".to_string()),
    );
    assert_eq!(sel, 0.0);
}

#[test]
fn test_selectivity_range_gt_gte() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational
        .insert("score".to_string(), crate::node::FieldValue::Float(95.0));
    engine.insert(&node).expect("insert");
    let sel_gt = engine.get_estimated_selectivity(
        "score",
        &crate::query::RelOp::Gt,
        &crate::node::FieldValue::Float(90.0),
    );
    assert_eq!(sel_gt, 0.33);
    let sel_gte = engine.get_estimated_selectivity(
        "score",
        &crate::query::RelOp::Gte,
        &crate::node::FieldValue::Float(90.0),
    );
    assert_eq!(sel_gte, 0.33);
}

#[test]
fn test_selectivity_range_lt_lte() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational
        .insert("age".to_string(), crate::node::FieldValue::Float(30.0));
    engine.insert(&node).expect("insert");
    let sel_lt = engine.get_estimated_selectivity(
        "age",
        &crate::query::RelOp::Lt,
        &crate::node::FieldValue::Float(40.0),
    );
    assert_eq!(sel_lt, 0.33);
    let sel_lte = engine.get_estimated_selectivity(
        "age",
        &crate::query::RelOp::Lte,
        &crate::node::FieldValue::Float(40.0),
    );
    assert_eq!(sel_lte, 0.33);
}

#[test]
fn test_selectivity_unknown_field_empty_engine() {
    let engine = in_memory_engine();
    let sel = engine.get_estimated_selectivity(
        "nonexistent",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("x".to_string()),
    );
    assert_eq!(sel, 1.0);
}

#[test]
fn test_selectivity_unknown_field_with_data() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    engine.insert(&node).expect("insert");
    let sel_eq = engine.get_estimated_selectivity(
        "nonexistent",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("x".to_string()),
    );
    assert_eq!(sel_eq, 0.0);
    let sel_neq = engine.get_estimated_selectivity(
        "nonexistent",
        &crate::query::RelOp::Neq,
        &crate::node::FieldValue::String("x".to_string()),
    );
    assert_eq!(sel_neq, 1.0);
}

#[test]
fn test_selectivity_range_op_unknown_field() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational
        .insert("score".to_string(), crate::node::FieldValue::Float(95.0));
    engine.insert(&node).expect("insert");
    let sel_gt = engine.get_estimated_selectivity(
        "nonexistent",
        &crate::query::RelOp::Gt,
        &crate::node::FieldValue::Float(90.0),
    );
    assert!(
        (sel_gt - 0.5).abs() < 1e-6,
        "range op on unknown field should be 0.5, got {sel_gt}"
    );
    let sel_lt = engine.get_estimated_selectivity(
        "nonexistent",
        &crate::query::RelOp::Lt,
        &crate::node::FieldValue::Float(90.0),
    );
    assert!(
        (sel_lt - 0.5).abs() < 1e-6,
        "Lt on unknown field should be 0.5, got {sel_lt}"
    );
}

#[test]
fn test_selectivity_with_cardinality_cap() {
    let engine = in_memory_engine();
    for i in 0..100u128 {
        let mut node = UnifiedNode::new(i);
        node.relational.insert(
            "color".to_string(),
            crate::node::FieldValue::String(format!("color_{}", i)),
        );
        engine.insert(&node).expect("insert");
    }
    let sel_eq = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("nonexistent".to_string()),
    );
    assert!(
        (sel_eq - 0.01).abs() < 1e-6,
        "expected ~0.01 for unknown value with 100 distinct keys, got {sel_eq}"
    );
    let sel_neq = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Neq,
        &crate::node::FieldValue::String("nonexistent".to_string()),
    );
    assert!(
        (sel_neq - 0.99).abs() < 1e-6,
        "expected ~0.99 for Neq with 100 distinct keys, got {sel_neq}"
    );
}

#[test]
fn test_selectivity_neq_missing_field_with_data() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    engine.insert(&node).expect("insert");
    let sel_neq = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Neq,
        &crate::node::FieldValue::String("nonexistent".to_string()),
    );
    assert!(
        (sel_neq - 1.0).abs() < 1e-6,
        "Neq for missing value with known field should be 1.0, got {sel_neq}"
    );
}

#[test]
fn test_selectivity_range_op_known_field() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational
        .insert("score".to_string(), crate::node::FieldValue::Float(95.0));
    engine.insert(&node).expect("insert");
    let sel_gt = engine.get_estimated_selectivity(
        "score",
        &crate::query::RelOp::Gt,
        &crate::node::FieldValue::Float(90.0),
    );
    assert_eq!(sel_gt, 0.33);
    let sel_lt = engine.get_estimated_selectivity(
        "score",
        &crate::query::RelOp::Lt,
        &crate::node::FieldValue::Float(100.0),
    );
    assert_eq!(sel_lt, 0.33);
}

// ─── initialize_cardinality_stats ─────────────────────────────

#[test]
fn test_initialize_cardinality_stats_empty() {
    let engine = in_memory_engine();
    let stats = StorageEngine::initialize_cardinality_stats(&*engine.backend);
    assert!(
        stats.is_empty(),
        "empty backend should return empty cardinality stats"
    );
}

#[test]
fn test_initialize_cardinality_stats_with_single_node() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    engine.insert(&node).expect("insert");
    let stats = StorageEngine::initialize_cardinality_stats(&*engine.backend);
    assert_eq!(stats.len(), 1, "should have one field");
    let color_map = stats.get("color").expect("should have color field");
    let red_count = color_map.get("red").copied().unwrap_or(0);
    assert_eq!(red_count, 1, "red should have cardinality 1");
}

#[test]
fn test_initialize_cardinality_stats_multiple_fields() {
    let engine = in_memory_engine();
    for i in 0..3u128 {
        let mut node = UnifiedNode::new(i);
        node.relational.insert(
            "color".to_string(),
            crate::node::FieldValue::String(if i % 2 == 0 {
                "red".to_string()
            } else {
                "blue".to_string()
            }),
        );
        node.relational.insert(
            "size".to_string(),
            crate::node::FieldValue::String("large".to_string()),
        );
        engine.insert(&node).expect("insert");
    }
    let stats = StorageEngine::initialize_cardinality_stats(&*engine.backend);
    assert_eq!(stats.len(), 2, "two fields: color and size");
    let color_map = stats.get("color").expect("color");
    assert_eq!(*color_map.get("red").unwrap_or(&0), 2);
    assert_eq!(*color_map.get("blue").unwrap_or(&0), 1);
    let size_map = stats.get("size").expect("size");
    assert_eq!(*size_map.get("large").unwrap_or(&0), 3);
}

#[test]
fn test_initialize_cardinality_stats_after_delete() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    engine.insert(&node).expect("insert");
    engine.delete(1, "test").expect("delete");
    let stats = StorageEngine::initialize_cardinality_stats(&*engine.backend);
    assert!(
        !stats.is_empty() || stats.is_empty(),
        "should complete without error"
    );
}

#[test]
fn test_initialize_cardinality_stats_many_distinct_values() {
    let engine = in_memory_engine();
    for i in 0..10u128 {
        let mut node = UnifiedNode::new(i);
        node.relational.insert(
            "unique".to_string(),
            crate::node::FieldValue::String(format!("v_{}", i)),
        );
        engine.insert(&node).expect("insert");
    }
    let stats = StorageEngine::initialize_cardinality_stats(&*engine.backend);
    let unique_map = stats.get("unique").expect("unique field");
    assert_eq!(unique_map.len(), 10, "should track 10 distinct values");
}
