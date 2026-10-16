// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;
use vantadb::sdk::*;
use vantadb::DistanceMetric;
use vantadb::Value;

#[test]
fn test_vanta_value_roundtrip() {
    let val = Value::String("hello".into());
    let json = serde_json::to_string(&val).unwrap();
    let back: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val, back);
}

#[test]
fn test_vanta_value_all_variants_serialize() {
    let variants = vec![
        Value::String("test".into()),
        Value::Int(42),
        Value::Float(std::f64::consts::PI),
        Value::Bool(true),
        Value::Null,
        Value::ListString(vec!["a".into(), "b".into()]),
        Value::ListInt(vec![1, 2, 3]),
        Value::ListFloat(vec![1.1, 2.2]),
        Value::ListBool(vec![true, false]),
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }
}

#[test]
fn test_memory_input_serialize_roundtrip() {
    let mut meta = BTreeMap::new();
    meta.insert("source".into(), Value::String("test".into()));
    let input = MemoryInput {
        namespace: "ns1".into(),
        key: "k1".into(),
        payload: "hello world".into(),
        metadata: meta,
        vector: Some(vec![0.1, 0.2, 0.3]),
        sparse_vector: None,
        ttl_ms: Some(60000),
        ..Default::default()
    };
    let json = serde_json::to_string(&input).unwrap();
    let back: MemoryInput = serde_json::from_str(&json).unwrap();
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
    meta.insert("lang".into(), Value::String("en".into()));
    let record = MemoryRecord {
        namespace: "ns1".into(),
        key: "k1".into(),
        payload: "data".into(),
        metadata: meta,
        created_at_ms: 1000,
        updated_at_ms: 2000,
        version: 1,
        node_id: 42,
        vector: None,
        sparse_vector: None,
        expires_at_ms: None,
        superseded_by: None,
        superseded_at_ms: None,
        ..Default::default()
    };
    let json = serde_json::to_string(&record).unwrap();
    let back: MemoryRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(record.namespace, back.namespace);
    assert_eq!(record.node_id, back.node_id);
}

#[test]
fn test_search_request_serialize() {
    let req = MemorySearchRequest {
        namespace: "ns1".into(),

        query_vector: vec![0.1, 0.2],
        filters: BTreeMap::new(),

        text_query: Some("hello".into()),
        top_k: 5,
        distance_metric: DistanceMetric::Cosine,
        explain: true,
        query_sparse: None,
        exclude_superseded: false,
        min_confidence: None,
        as_of_ms: None,
        valid_window: None,
        include_quarantined: false,
        search_profile: None,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: MemorySearchRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(req.namespace, back.namespace);
    assert_eq!(req.top_k, back.top_k);
    assert_eq!(req.text_query, back.text_query);
}

#[test]
fn test_search_hit_serialize() {
    let record = MemoryRecord {
        namespace: "ns".into(),
        key: "k".into(),
        payload: "p".into(),
        metadata: BTreeMap::new(),
        created_at_ms: 0,
        updated_at_ms: 0,
        version: 1,
        node_id: 1,
        vector: None,
        sparse_vector: None,
        expires_at_ms: None,
        superseded_by: None,
        superseded_at_ms: None,
        ..Default::default()
    };
    let hit = MemorySearchHit {
        record: record.clone(),
        score: 0.95,
        explanation: None,
    };
    let json = serde_json::to_string(&hit).unwrap();
    let back: MemorySearchHit = serde_json::from_str(&json).unwrap();
    assert_eq!(hit.score, back.score);
    assert_eq!(hit.record.node_id, back.record.node_id);
}

// ── API-10 slice 1: schema-first parity (Rust → wire-schema.json → TS) ────
// The canonical wire schema lives in docs/api/wire-schema.json (hand-written
// on purpose — real codegen is slice 2, post-1.0). These tests pin the
// contract: every wire DTO serializes to EXACTLY the key set the schema
// declares (no silent field additions/removals per binding), u128 ids travel
// as decimal strings, and the TS mirror declares the same v2 fields.

mod wire_schema_parity {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn load_schema() -> serde_json::Value {
        let raw = fs::read_to_string(
            repo_root()
                .join("docs")
                .join("api")
                .join("wire-schema.json"),
        )
        .expect("read docs/api/wire-schema.json");
        serde_json::from_str(&raw).expect("wire-schema.json must be valid JSON")
    }

    fn schema_keys(schema: &serde_json::Value, dto: &str) -> BTreeSet<String> {
        schema["properties"][dto]["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("schema must declare properties for {dto}"))
            .keys()
            .cloned()
            .collect()
    }

    fn schema_required(schema: &serde_json::Value, dto: &str) -> BTreeSet<String> {
        schema["properties"][dto]["required"]
            .as_array()
            .unwrap_or_else(|| panic!("schema must declare required for {dto}"))
            .iter()
            .map(|v| {
                v.as_str()
                    .expect("required entries are strings")
                    .to_string()
            })
            .collect()
    }

    fn value_keys(value: &serde_json::Value, what: &str) -> BTreeSet<String> {
        value
            .as_object()
            .unwrap_or_else(|| panic!("{what} must serialize to a JSON object"))
            .keys()
            .cloned()
            .collect()
    }

    /// Exact key-set parity: the Rust serde shape IS the schema shape.
    /// A new field anywhere (core or binding) fails here until the schema
    /// is updated in the same PR — that is the anti-drift contract.
    fn assert_exact_parity(schema: &serde_json::Value, dto: &str, value: serde_json::Value) {
        let want = schema_keys(schema, dto);
        let got = value_keys(&value, dto);
        let missing: Vec<_> = want.difference(&got).collect();
        let extra: Vec<_> = got.difference(&want).collect();
        assert!(
            missing.is_empty() && extra.is_empty(),
            "{dto} drift: missing from Rust wire = {missing:?}, extra in Rust wire = {extra:?} \
             (update docs/api/wire-schema.json in the same PR)"
        );
        let required = schema_required(schema, dto);
        let absent: Vec<_> = required.difference(&got).collect();
        assert!(
            absent.is_empty(),
            "{dto}: schema-required keys absent from Rust wire = {absent:?}"
        );
    }

    #[test]
    fn wire_dto_key_sets_match_canonical_schema() {
        use vantadb::sdk::*;
        use vantadb::Value;

        let schema = load_schema();

        // MemoryInput — write path (all-optional v2 fields present as null/false).
        let input = MemoryInput::new("ns", "k", "p");
        assert_exact_parity(
            &schema,
            "MemoryInput",
            serde_json::to_value(&input).expect("serialize MemoryInput"),
        );

        // MemoryRecord — read path; node_id must be a decimal string (u128 > 2^53).
        let record = MemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "p".into(),
            metadata: Default::default(),
            created_at_ms: 1,
            updated_at_ms: 2,
            version: 1,
            node_id: (1u128 << 63) + 7,
            ..Default::default()
        };
        let record_json = serde_json::to_value(&record).expect("serialize MemoryRecord");
        assert!(
            record_json["node_id"].is_string(),
            "MemoryRecord.node_id must travel as a decimal string"
        );
        assert_exact_parity(&schema, "MemoryRecord", record_json);

        // MemorySearchHit — explanation present so the key exists regardless
        // of any future skip_serializing_if on the Option.
        let hit = MemorySearchHit {
            record,
            score: 0.5,
            explanation: Some(SearchExplanationHit {
                identity: "ns\x00k".into(),
                score: 0.5,
                snippet: None,
                matched_tokens: vec![],
                matched_phrases: vec![],
                bm25_terms: vec![],
                rrf_text_rank: None,
                rrf_vector_rank: None,
            }),
        };
        assert_exact_parity(
            &schema,
            "MemorySearchHit",
            serde_json::to_value(&hit).expect("serialize MemorySearchHit"),
        );

        // MemorySearchPage — abstention signal populated (SCH-07 wire).
        let page = MemorySearchPage {
            hits: vec![],
            next_cursor: None,
            abstained: true,
            abstention_reason: Some(AbstentionReason::NoCandidatesAboveThreshold),
        };
        assert_exact_parity(
            &schema,
            "MemorySearchPage",
            serde_json::to_value(&page).expect("serialize MemorySearchPage"),
        );

        // MemoryListPage.
        let list = MemoryListPage {
            records: vec![],
            next_cursor: None,
        };
        assert_exact_parity(
            &schema,
            "MemoryListPage",
            serde_json::to_value(&list).expect("serialize MemoryListPage"),
        );

        // NodeRecord + EdgeRecord — graph wire; id/target as decimal strings.
        let node = NodeRecord {
            id: (1u128 << 63) + 7,
            fields: Default::default(),
            vector: Some(vec![0.5]),
            vector_dimensions: 1,
            edges: vec![EdgeRecord {
                target: 2,
                label: "related".into(),
                weight: 0.8,
                reverse: false,
                created_at_ms: 0,
            }],
            confidence_score: 0.9,
            importance: 0.5,
            hits: 10,
            last_accessed: 1000,
            epoch: 0,
            tier: StorageTier::Hot,
            is_alive: true,
        };
        let node_json = serde_json::to_value(&node).expect("serialize NodeRecord");
        assert!(
            node_json["id"].is_string(),
            "NodeRecord.id must travel as a decimal string"
        );
        assert_exact_parity(&schema, "NodeRecord", node_json);
        assert_exact_parity(
            &schema,
            "EdgeRecord",
            serde_json::to_value(&EdgeRecord {
                target: 2,
                label: "related".into(),
                weight: 0.8,
                reverse: false,
                created_at_ms: 0,
            })
            .expect("serialize EdgeRecord"),
        );

        // Filter + window DTOs.
        assert_exact_parity(
            &schema,
            "MemoryFilterItem",
            serde_json::to_value(&MemoryFilterItem {
                field: "kind".into(),
                op: FilterOp::Eq,
                value: Value::String("note".into()),
            })
            .expect("serialize MemoryFilterItem"),
        );
        assert_exact_parity(
            &schema,
            "ValidWindow",
            serde_json::to_value(ValidWindow {
                from_ms: 1,
                to_ms: 2,
            })
            .expect("serialize ValidWindow"),
        );

        // MemorySearchRequest — required-subset contract (the request carries
        // many optional-with-default fields; the schema pins which are required).
        let req = MemorySearchRequest {
            namespace: "ns".into(),
            query_vector: vec![0.1],
            query_sparse: None,
            filters: Default::default(),
            text_query: None,
            top_k: 5,
            distance_metric: vantadb::DistanceMetric::Cosine,
            explain: false,
            exclude_superseded: false,
            min_confidence: None,
            as_of_ms: None,
            valid_window: None,
            include_quarantined: false,
            search_profile: None,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
        };
        let req_json = serde_json::to_value(&req).expect("serialize MemorySearchRequest");
        let req_keys = value_keys(&req_json, "MemorySearchRequest");
        for key in schema_required(&schema, "MemorySearchRequest") {
            assert!(
                req_keys.contains(&key),
                "MemorySearchRequest: schema-required key '{key}' missing from Rust wire"
            );
        }
        for key in req_keys {
            assert!(
                schema_keys(&schema, "MemorySearchRequest").contains(&key),
                "MemorySearchRequest: Rust wire key '{key}' not declared in schema \
                 (update docs/api/wire-schema.json in the same PR)"
            );
        }
    }

    #[test]
    fn ts_mirror_declares_same_wire_fields() {
        // Rust→TS parity without codegen (slice 1): the TS mirror must declare
        // the same wire field names the schema pins — above all the v2 fields
        // (confidence, quarantine, valid-time) and the u128-as-string ids.
        // Python .pyi is excluded: vantadb-python/** is foreign WIP (see task file).
        let ts = fs::read_to_string(repo_root().join("vantadb-ts").join("src").join("types.ts"))
            .expect("read vantadb-ts/src/types.ts");
        for field in [
            "node_id",
            "superseded_by",
            "valid_at_ms",
            "invalid_at_ms",
            "confidence",
            "confidence_class",
            "quarantined_at_ms",
            "quarantine_reason",
            "include_quarantined",
            "min_confidence",
        ] {
            assert!(
                ts.contains(field),
                "vantadb-ts/src/types.ts must declare wire field '{field}' \
                 (drift vs docs/api/wire-schema.json)"
            );
        }
    }
}

#[test]
fn test_list_page_serialize() {
    let page = MemoryListPage {
        records: vec![],
        next_cursor: None,
    };
    let json = serde_json::to_string(&page).unwrap();
    let back: MemoryListPage = serde_json::from_str(&json).unwrap();
    assert!(back.records.is_empty());
    assert!(back.next_cursor.is_none());
}

#[test]
fn test_search_page_serialize_with_abstention_signal() {
    // SCH-07 (ADR-046 §D2): the page wire carries the selective-abstention
    // signal with stable snake_case codes — the same shape every page-shaped
    // transport (HTTP `SearchPageV2`, MCP structuredContent) mirrors.
    let page = MemorySearchPage {
        hits: vec![],
        next_cursor: None,
        abstained: true,
        abstention_reason: Some(AbstentionReason::NoCandidatesAboveThreshold),
    };
    let json = serde_json::to_value(&page).unwrap();
    assert_eq!(json["abstained"], serde_json::json!(true));
    assert_eq!(
        json["abstention_reason"],
        serde_json::json!("no_candidates_above_threshold")
    );
    let back: MemorySearchPage = serde_json::from_value(json).unwrap();
    assert!(back.abstained);
    assert_eq!(
        back.abstention_reason,
        Some(AbstentionReason::NoCandidatesAboveThreshold)
    );

    // Default (no threshold / no quarantine drain): both fields at defaults,
    // and a v1-style payload without them still deserializes (serde default).
    let page = MemorySearchPage {
        hits: vec![],
        next_cursor: None,
        abstained: false,
        abstention_reason: None,
    };
    let json = serde_json::to_value(&page).unwrap();
    assert_eq!(json["abstention_reason"], serde_json::Value::Null);
    let legacy: MemorySearchPage =
        serde_json::from_value(serde_json::json!({"hits": [], "next_cursor": null})).unwrap();
    assert!(!legacy.abstained);
    assert_eq!(legacy.abstention_reason, None);
}

#[test]
fn test_node_record_serialize() {
    let record = NodeRecord {
        id: 1,
        fields: BTreeMap::new(),
        vector: None,
        vector_dimensions: 0,
        edges: vec![EdgeRecord {
            target: 2,
            label: "related".into(),
            weight: 0.8,
            reverse: false,
            created_at_ms: 0,
        }],
        confidence_score: 0.9,
        importance: 0.5,
        hits: 10,
        last_accessed: 1000,
        epoch: 0,
        tier: StorageTier::Hot,
        is_alive: true,
    };
    let json = serde_json::to_string(&record).unwrap();
    let back: NodeRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(record.id, back.id);
    assert_eq!(record.edges.len(), back.edges.len());
    assert_eq!(record.edges[0].target, back.edges[0].target);
}

#[test]
fn test_query_result_serialize() {
    let result = QueryResult::Read(vec![]);
    let json = serde_json::to_string(&result).unwrap();
    let back: QueryResult = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, QueryResult::Read(_)));

    let write = QueryResult::Write {
        affected_nodes: 1,
        message: "ok".into(),
        node_id: Some(42),
    };
    let json = serde_json::to_string(&write).unwrap();
    let back: QueryResult = serde_json::from_str(&json).unwrap();
    match back {
        QueryResult::Write {
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
fn test_query_result_write_missing_node_id_defaults_none() {
    // API-01 R1 (review P2-01): producers that predate the `u128_serde`
    // change (or omit `node_id`) must still deserialize. `deserialize_with`
    // bypasses serde_derive's `missing_field` default path, so the field
    // needs an explicit `#[serde(default)]` — without it this payload fails
    // with "missing field `node_id`".
    let json = r#"{"Write":{"affected_nodes":1,"message":"old"}}"#;
    let parsed: QueryResult =
        serde_json::from_str(json).expect("missing node_id must default to None");
    match parsed {
        QueryResult::Write {
            affected_nodes,
            message,
            node_id,
        } => {
            assert_eq!(affected_nodes, 1);
            assert_eq!(message, "old");
            assert_eq!(node_id, None);
        }
        other => panic!("expected Write variant, got {other:?}"),
    }
}

#[test]
fn test_capabilities_serialize() {
    let caps = Capabilities {
        runtime_profile: RuntimeProfile::LowResource,
        persistence: false,
        vector_search: true,
        iql_queries: true,
        read_only: false,
    };
    let json = serde_json::to_string(&caps).unwrap();
    let back: Capabilities = serde_json::from_str(&json).unwrap();
    assert_eq!(caps.vector_search, back.vector_search);
    assert_eq!(caps.persistence, back.persistence);
    assert_eq!(back.runtime_profile, RuntimeProfile::LowResource);
}

#[test]
fn test_export_report_serialize() {
    let report = ExportReport {
        records_exported: 100,
        namespaces: vec!["ns1".into()],
        path: "/tmp/export.jsonl".into(),
        duration_ms: 50,
        sha256: String::new(),
        manifest_path: String::new(),
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: ExportReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report.records_exported, back.records_exported);
}

#[test]
fn test_import_report_serialize() {
    let report = ImportReport {
        inserted: 10,
        updated: 2,
        skipped: 0,
        errors: 0,
        quarantined: 3,
        duration_ms: 30,
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: ImportReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report.inserted, back.inserted);
}

#[test]
fn test_index_rebuild_report_serialize() {
    let report = IndexRebuildReport {
        scanned_nodes: 100,
        indexed_vectors: 95,
        skipped_tombstones: 5,
        duration_ms: 200,
        derived_rebuild_ms: 50,
        index_path: "/tmp/index".into(),
        success: true,
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: IndexRebuildReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report.scanned_nodes, back.scanned_nodes);
    assert!(back.success);
}

#[test]
fn test_text_index_audit_report_serialize() {
    let report = TextIndexAuditReport {
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
    let back: TextIndexAuditReport = serde_json::from_str(&json).unwrap();
    assert!(back.passed);
}

#[test]
fn test_operational_metrics_serialize() {
    let metrics = OperationalMetrics {
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
        jemalloc_allocated_bytes: None,
        jemalloc_active_bytes: None,
        jemalloc_metadata_bytes: None,
        jemalloc_resident_bytes: None,
        jemalloc_mapped_bytes: None,
        jemalloc_retained_bytes: None,
        l1_extraction_latency_ms: 15,
        l1_dedup_latency_ms: 5,
        l2_extraction_latency_ms: 25,
        l2_llm_duration_ms: 80,
        l3_generation_latency_ms: 45,
        persona_length_before: 120,
        persona_length_after: 150,
        persona_drift_ratio: 2500,
        recall_hit_count: 7,
        recall_top_score: 9500,
        recall_latency_ms: 30,
        recall_strategy: 3,
        offload_latency_ms: 60,
    };
    let json = serde_json::to_string(&metrics).unwrap();
    let back: OperationalMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(metrics.startup_ms, back.startup_ms);
    assert_eq!(metrics.hnsw_nodes_count, back.hnsw_nodes_count);
    assert_eq!(back.l1_extraction_latency_ms, 15);
    assert_eq!(back.recall_strategy, 3);
}

#[test]
fn test_search_explanation_serialize() {
    let explanation = SearchExplanation {
        route: "hybrid".into(),
        hits: vec![],
        fusion_report: None,
    };
    let json = serde_json::to_string(&explanation).unwrap();
    let back: SearchExplanation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.route, "hybrid");
}

#[test]
fn test_query_result_write_node_id_u128_wire_string() {
    // Wire contract (API-01): u128 ids travel as decimal strings so ids > 2^53
    // survive JSON — consistent with MemoryRecord/StaleContext (`u128_serde`).
    let big: u128 = (1u128 << 63) + 7;
    let write = QueryResult::Write {
        affected_nodes: 1,
        message: "created".into(),
        node_id: Some(big),
    };
    let json = serde_json::to_string(&write).unwrap();
    assert!(
        json.contains("\"node_id\":\""),
        "node_id must serialize as a decimal string, got: {json}"
    );
    assert!(
        json.contains(&big.to_string()),
        "decimal string must carry the full u128 value: {json}"
    );
    let back: QueryResult = serde_json::from_str(&json).unwrap();
    match back {
        QueryResult::Write { node_id, .. } => assert_eq!(node_id, Some(big)),
        other => panic!("expected Write, got {other:?}"),
    }

    // `None` stays null and roundtrips.
    let none = QueryResult::Write {
        affected_nodes: 0,
        message: "noop".into(),
        node_id: None,
    };
    let json_none = serde_json::to_string(&none).unwrap();
    let back_none: QueryResult = serde_json::from_str(&json_none).unwrap();
    match back_none {
        QueryResult::Write { node_id, .. } => assert_eq!(node_id, None),
        other => panic!("expected Write, got {other:?}"),
    }

    // Legacy numeric payloads (<= u64) still deserialize (backward-compatible read).
    let legacy = r#"{"Write":{"affected_nodes":1,"message":"old","node_id":42}}"#;
    let back_legacy: QueryResult = serde_json::from_str(legacy).unwrap();
    match back_legacy {
        QueryResult::Write { node_id, .. } => assert_eq!(node_id, Some(42)),
        other => panic!("expected Write, got {other:?}"),
    }
}

// ── SCH-04: confidence fields on the serde wire (HTTP/MCP propagate the
// record/hit JSON by serde; these tests pin the D2 contract) ────────────────

#[test]
fn memory_record_confidence_fields_serialize_with_declared_values() {
    let record = MemoryRecord {
        confidence_class: ConfidenceClass::Derived,
        confidence: 0.45,
        last_validated_at_ms: Some(1_700_000_000_000),
        derived_from: vec!["parent-1".into(), "parent-2".into()],
        ..Default::default()
    };

    let json = serde_json::to_value(&record).unwrap();
    assert_eq!(json["confidence_class"], serde_json::json!("Derived"));
    // serde_json widens f32 → f64 on the wire (JSON has no f32): compare
    // against the exact widened value, not the decimal literal.
    assert_eq!(json["confidence"].as_f64(), Some(f64::from(0.45f32)));
    assert_eq!(
        json["last_validated_at_ms"],
        serde_json::json!(1_700_000_000_000u64)
    );
    assert_eq!(
        json["derived_from"],
        serde_json::json!(["parent-1", "parent-2"])
    );

    let back: MemoryRecord = serde_json::from_value(json).unwrap();
    assert_eq!(back.confidence_class, ConfidenceClass::Derived);
    assert_eq!(back.confidence, 0.45);
    assert_eq!(back.last_validated_at_ms, Some(1_700_000_000_000));
    assert_eq!(
        back.derived_from,
        vec!["parent-1".to_string(), "parent-2".into()]
    );
}

#[test]
fn memory_record_v1_json_without_confidence_defaults_to_d_a() {
    // v1 payload (pre-0.8.0) has none of the confidence fields: serde defaults
    // must normalize it to the ratified D_a semantics (asserted, 1.0, never
    // validated, no parents).
    let json = serde_json::json!({
        "namespace": "ns",
        "key": "k",
        "payload": "p",
        "metadata": {},
        "created_at_ms": 1,
        "updated_at_ms": 2,
        "version": 1,
        "node_id": 7,
        "vector": null,
        "expires_at_ms": null,
        "superseded_by": null,
        "superseded_at_ms": null,
    });

    let record: MemoryRecord = serde_json::from_value(json).unwrap();
    assert_eq!(record.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(record.confidence, 1.0);
    assert_eq!(record.last_validated_at_ms, None);
    assert!(record.derived_from.is_empty());
}

// EGO-05: `Value` acepta escalares JS planos (NAPI/serde_json entregan f64,
// string, bool, null sin tag). El formato tagged existente sigue válido
// (retrocompatible); la serialización NO cambia.
#[test]
fn test_vanta_value_accepts_plain_scalars() {
    let cases: &[(&str, Value)] = &[
        // strings planos
        (r#""hello""#, Value::String("hello".into())),
        // enteros planos (i64 y f64 integral tipo Date.now())
        ("42", Value::Int(42)),
        ("-7", Value::Int(-7)),
        ("1753728000000", Value::Int(1753728000000)),
        // floats no-integrales
        ("2.5", Value::Float(2.5)),
        // bool y null planos
        ("true", Value::Bool(true)),
        ("false", Value::Bool(false)),
        ("null", Value::Null),
    ];
    for (json, expected) in cases {
        let got: Value = serde_json::from_str(json).unwrap();
        assert_eq!(&got, expected, "plain scalar {json}");
    }
}

#[test]
fn test_vanta_value_accepts_plain_homogeneous_arrays() {
    let cases: &[(&str, Value)] = &[
        (
            r#"["a","b"]"#,
            Value::ListString(vec!["a".into(), "b".into()]),
        ),
        ("[1,2,3]", Value::ListInt(vec![1, 2, 3])),
        ("[1.5,2.5]", Value::ListFloat(vec![1.5, 2.5])),
        ("[true,false]", Value::ListBool(vec![true, false])),
    ];
    for (json, expected) in cases {
        let got: Value = serde_json::from_str(json).unwrap();
        assert_eq!(&got, expected, "plain array {json}");
    }
}

#[test]
fn test_vanta_value_bare_null_string_roundtrips() {
    // serde serializa el unit variant sin wrapper: Value::Null ⇄ `"Null"`.
    // El bare `"Null"` debe leerse como Null (como el derived), no como String.
    let json = serde_json::to_string(&Value::Null).unwrap();
    assert_eq!(json, r#""Null""#);
    let back: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Value::Null);
}

#[test]
fn test_vanta_value_tagged_form_still_parses() {
    let cases: &[(&str, Value)] = &[
        (r#"{"String":"hi"}"#, Value::String("hi".into())),
        (r#"{"Int":42}"#, Value::Int(42)),
        (r#"{"Float":2.5}"#, Value::Float(2.5)),
        (r#"{"Bool":true}"#, Value::Bool(true)),
        (r#"{"Null":null}"#, Value::Null),
        (r#"{"ListInt":[1,2]}"#, Value::ListInt(vec![1, 2])),
    ];
    for (json, expected) in cases {
        let got: Value = serde_json::from_str(json).unwrap();
        assert_eq!(&got, expected, "tagged {json}");
    }
}

#[test]
fn test_vanta_value_rejects_mixed_arrays() {
    let got: Result<Value, _> = serde_json::from_str(r#"[1,"a"]"#);
    assert!(got.is_err(), "mixed arrays must fail, got: {:?}", got.ok());
}
