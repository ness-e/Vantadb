use std::collections::BTreeMap;
use vantadb::sdk::*;
use vantadb::DistanceMetric;
use vantadb::VantaValue;

#[test]
fn test_vanta_value_roundtrip() {
    let val = VantaValue::String("hello".into());
    let json = serde_json::to_string(&val).unwrap();
    let back: VantaValue = serde_json::from_str(&json).unwrap();
    assert_eq!(val, back);
}

#[test]
fn test_vanta_value_all_variants_serialize() {
    let variants = vec![
        VantaValue::String("test".into()),
        VantaValue::Int(42),
        VantaValue::Float(std::f64::consts::PI),
        VantaValue::Bool(true),
        VantaValue::Null,
        VantaValue::ListString(vec!["a".into(), "b".into()]),
        VantaValue::ListInt(vec![1, 2, 3]),
        VantaValue::ListFloat(vec![1.1, 2.2]),
        VantaValue::ListBool(vec![true, false]),
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: VantaValue = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }
}

#[test]
fn test_memory_input_serialize_roundtrip() {
    let mut meta = BTreeMap::new();
    meta.insert("source".into(), VantaValue::String("test".into()));
    let input = VantaMemoryInput {
        namespace: "ns1".into(),
        key: "k1".into(),
        payload: "hello world".into(),
        metadata: meta,
        vector: Some(vec![0.1, 0.2, 0.3]),
        ttl_ms: Some(60000),
    };
    let json = serde_json::to_string(&input).unwrap();
    let back: VantaMemoryInput = serde_json::from_str(&json).unwrap();
    assert_eq!(input.namespace, back.namespace);
    assert_eq!(input.key, back.key);
    assert_eq!(input.payload, back.payload);
    assert_eq!(input.metadata, back.metadata);
    assert_eq!(input.vector, back.vector);
    assert_eq!(input.ttl_ms, back.ttl_ms);
}

#[test]
fn test_memory_record_serialize() {
    let mut meta = BTreeMap::new();
    meta.insert("lang".into(), VantaValue::String("en".into()));
    let record = VantaMemoryRecord {
        namespace: "ns1".into(),
        key: "k1".into(),
        payload: "data".into(),
        metadata: meta,
        created_at_ms: 1000,
        updated_at_ms: 2000,
        version: 1,
        node_id: 42,
        vector: None,
        expires_at_ms: None,
    };
    let json = serde_json::to_string(&record).unwrap();
    let back: VantaMemoryRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(record.namespace, back.namespace);
    assert_eq!(record.node_id, back.node_id);
}

#[test]
fn test_search_request_serialize() {
    let req = VantaMemorySearchRequest {
        namespace: "ns1".into(),

        query_vector: vec![0.1, 0.2],
        filters: BTreeMap::new(),

        text_query: Some("hello".into()),
        top_k: 5,
        distance_metric: DistanceMetric::Cosine,
        explain: true,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: VantaMemorySearchRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req.namespace, back.namespace);
    assert_eq!(req.top_k, back.top_k);
    assert_eq!(req.text_query, back.text_query);
}

#[test]
fn test_search_hit_serialize() {
    let record = VantaMemoryRecord {
        namespace: "ns".into(),
        key: "k".into(),
        payload: "p".into(),
        metadata: BTreeMap::new(),
        created_at_ms: 0,
        updated_at_ms: 0,
        version: 1,
        node_id: 1,
        vector: None,
        expires_at_ms: None,
    };
    let hit = VantaMemorySearchHit {
        record: record.clone(),
        score: 0.95,
        explanation: None,
    };
    let json = serde_json::to_string(&hit).unwrap();
    let back: VantaMemorySearchHit = serde_json::from_str(&json).unwrap();
    assert_eq!(hit.score, back.score);
    assert_eq!(hit.record.node_id, back.record.node_id);
}

#[test]
fn test_list_page_serialize() {
    let page = VantaMemoryListPage {
        records: vec![],
        next_cursor: None,
    };
    let json = serde_json::to_string(&page).unwrap();
    let back: VantaMemoryListPage = serde_json::from_str(&json).unwrap();
    assert!(back.records.is_empty());
    assert!(back.next_cursor.is_none());
}

#[test]
fn test_node_record_serialize() {
    let record = VantaNodeRecord {
        id: 1,
        fields: BTreeMap::new(),
        vector: None,
        vector_dimensions: 0,
        edges: vec![VantaEdgeRecord {
            target: 2,
            label: "related".into(),
            weight: 0.8,
        }],
        confidence_score: 0.9,
        importance: 0.5,
        hits: 10,
        last_accessed: 1000,
        epoch: 0,
        tier: VantaStorageTier::Hot,
        is_alive: true,
    };
    let json = serde_json::to_string(&record).unwrap();
    let back: VantaNodeRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(record.id, back.id);
    assert_eq!(record.edges.len(), back.edges.len());
    assert_eq!(record.edges[0].target, back.edges[0].target);
}

#[test]
fn test_query_result_serialize() {
    let result = VantaQueryResult::Read(vec![]);
    let json = serde_json::to_string(&result).unwrap();
    let back: VantaQueryResult = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, VantaQueryResult::Read(_)));

    let write = VantaQueryResult::Write {
        affected_nodes: 1,
        message: "ok".into(),
        node_id: Some(42),
    };
    let json = serde_json::to_string(&write).unwrap();
    let back: VantaQueryResult = serde_json::from_str(&json).unwrap();
    match back {
        VantaQueryResult::Write {
            affected_nodes,
            message,
            node_id,
        } => {
            assert_eq!(affected_nodes, 1);
            assert_eq!(message, "ok");
            assert_eq!(node_id, Some(42));
        }
        _ => panic!("expected Write variant"),
    }
}

#[test]
fn test_capabilities_serialize() {
    let caps = VantaCapabilities {
        runtime_profile: VantaRuntimeProfile::LowResource,
        persistence: false,
        vector_search: true,
        iql_queries: true,
        read_only: false,
    };
    let json = serde_json::to_string(&caps).unwrap();
    let back: VantaCapabilities = serde_json::from_str(&json).unwrap();
    assert_eq!(caps.vector_search, back.vector_search);
    assert_eq!(caps.persistence, back.persistence);
    assert_eq!(back.runtime_profile, VantaRuntimeProfile::LowResource);
}

#[test]
fn test_export_report_serialize() {
    let report = VantaExportReport {
        records_exported: 100,
        namespaces: vec!["ns1".into()],
        path: "/tmp/export.jsonl".into(),
        duration_ms: 50,
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: VantaExportReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report.records_exported, back.records_exported);
}

#[test]
fn test_import_report_serialize() {
    let report = VantaImportReport {
        inserted: 10,
        updated: 2,
        skipped: 0,
        errors: 0,
        duration_ms: 30,
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: VantaImportReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report.inserted, back.inserted);
}

#[test]
fn test_index_rebuild_report_serialize() {
    let report = VantaIndexRebuildReport {
        scanned_nodes: 100,
        indexed_vectors: 95,
        skipped_tombstones: 5,
        duration_ms: 200,
        derived_rebuild_ms: 50,
        index_path: "/tmp/index".into(),
        success: true,
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: VantaIndexRebuildReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report.scanned_nodes, back.scanned_nodes);
    assert!(back.success);
}

#[test]
fn test_text_index_audit_report_serialize() {
    let report = VantaTextIndexAuditReport {
        schema_version: 1,
        tokenizer: "ascii_alnum".into(),
        tokenizer_version: 1,
        key_format: "vanta_text_v3".into(),
        namespace_filter: None,
        namespaces_audited: vec![],
        records_scanned: 0,
        expected_entries: 0,
        actual_entries: 0,
        missing_entries: 0,
        unexpected_entries: 0,
        value_mismatches: 0,
        unreadable_entries: 0,
        mismatches: 0,
        deep_audit: false,
        position_errors: 0,
        tf_errors: 0,
        df_errors: 0,
        doc_len_errors: 0,
        logical_corruptions: 0,
        state_valid: true,
        state_status: "ok".into(),
        duration_ms: 0,
        passed: true,
        status: "clean".into(),
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: VantaTextIndexAuditReport = serde_json::from_str(&json).unwrap();
    assert!(back.passed);
}

#[test]
fn test_operational_metrics_serialize() {
    let metrics = VantaOperationalMetrics {
        startup_ms: 100,
        wal_replay_ms: 20,
        wal_records_replayed: 50,
        ann_rebuild_ms: 500,
        ann_rebuild_scanned_nodes: 1000,
        derived_rebuild_ms: 100,
        text_index_rebuild_ms: 200,
        text_postings_written: 300,
        text_index_repairs: 1,
        text_lexical_queries: 10,
        text_lexical_query_ms: 5,
        text_candidates_scored: 100,
        text_consistency_audits: 2,
        text_consistency_audit_failures: 0,
        hybrid_query_ms: 3,
        hybrid_candidates_fused: 50,
        planner_hybrid_queries: 5,
        planner_text_only_queries: 3,
        planner_vector_only_queries: 8,
        records_exported: 200,
        records_imported: 150,
        import_errors: 0,
        derived_prefix_scans: 20,
        derived_full_scan_fallbacks: 0,
        process_rss_bytes: 1048576,
        process_virtual_bytes: 2097152,
        hnsw_nodes_count: 500,
        hnsw_logical_bytes: 65536,
        mmap_resident_bytes: None,
        volatile_cache_entries: 50,
        volatile_cache_cap_bytes: 1048576,
    };
    let json = serde_json::to_string(&metrics).unwrap();
    let back: VantaOperationalMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(metrics.startup_ms, back.startup_ms);
    assert_eq!(metrics.hnsw_nodes_count, back.hnsw_nodes_count);
}

#[test]
fn test_search_explanation_serialize() {
    let explanation = VantaSearchExplanation {
        route: "hybrid".into(),
        hits: vec![],
        fusion_report: None,
    };
    let json = serde_json::to_string(&explanation).unwrap();
    let back: VantaSearchExplanation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.route, "hybrid");
}
