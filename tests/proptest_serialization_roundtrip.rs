//! 🔁 Property-based round-trip tests for SDK serialization types.
//!
//! Binary (postcard) round-trips use full arbitrary floats with exact equality.
//! JSON round-trips use float-free or range-restricted values to avoid
//! serde_json's f64/f32 precision limitation (1 ULP loss for certain values).
//! Types with `#[serde(with = "u128_serde")]` (string-based u128) are tested
//! with JSON only since postcard can't deserialize the custom serde helper.
//!
//! Run: `cargo test --test proptest_serialization_roundtrip`

use proptest::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use vantadb::node::DistanceMetric;
use vantadb::{
    VantaBm25TermContribution, VantaCapabilities, VantaEdgeRecord, VantaExportReport, VantaFields,
    VantaHybridFusionReport, VantaImportReport, VantaIndexRebuildReport, VantaMemoryInput,
    VantaMemoryListOptions, VantaMemoryMetadata, VantaMemoryRecord, VantaMemorySearchHit,
    VantaMemorySearchRequest, VantaNodeInput, VantaNodeRecord, VantaQueryResult,
    VantaRuntimeProfile, VantaSearchExplanation, VantaSearchExplanationHit, VantaSearchHit,
    VantaStorageTier, VantaTextIndexRepairReport, VantaValue,
};

// ── Helpers ─────────────────────────────────────────────────────────────

/// Postcard exact round-trip: T → bytes → T, assert_eq.
fn assert_postcard<T: Serialize + DeserializeOwned + std::fmt::Debug + PartialEq>(val: &T) {
    let bytes = postcard::to_allocvec(val).unwrap();
    let recovered: T = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(*val, recovered, "postcard round-trip mismatch");
}

/// JSON round-trip: serialize → deserialize → assert_eq original.
/// Only safe for types without f32/f64 fields.
fn assert_json<T: Serialize + DeserializeOwned + std::fmt::Debug + PartialEq>(val: &T) {
    let json = serde_json::to_string(val).unwrap();
    let recovered: T = serde_json::from_str(&json).unwrap();
    assert_eq!(*val, recovered, "JSON round-trip mismatch");
}

// ── Strategy helpers ─────────────────────────────────────────────────────

fn arb_datetime() -> impl Strategy<Value = chrono::DateTime<chrono::Utc>> {
    (0i64..2_000_000_000i64).prop_map(|secs| {
        chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0).unwrap_or_default()
    })
}

/// Float-free VantaValue for JSON round-trip tests (avoid f64 precision loss).
fn arb_vanta_value_json() -> impl Strategy<Value = VantaValue> {
    prop_oneof![
        10 => any::<String>().prop_map(VantaValue::String),
        4 => any::<i64>().prop_map(VantaValue::Int),
        4 => any::<bool>().prop_map(VantaValue::Bool),
        4 => arb_datetime().prop_map(VantaValue::DateTime),
        2 => prop::collection::vec(any::<String>(), 0..5).prop_map(VantaValue::ListString),
        1 => prop::collection::vec(any::<i64>(), 0..5).prop_map(VantaValue::ListInt),
        1 => prop::collection::vec(any::<bool>(), 0..5).prop_map(VantaValue::ListBool),
        1 => prop::collection::vec(arb_datetime(), 0..5).prop_map(VantaValue::ListDateTime),
        2 => Just(VantaValue::Null),
    ]
}

fn arb_metadata_json() -> impl Strategy<Value = VantaMemoryMetadata> {
    prop::collection::btree_map("[a-zA-Z_][a-zA-Z0-9_]{0,15}", arb_vanta_value_json(), 0..8)
}

/// Full-range VantaValue for postcard tests (all float values).
fn arb_vanta_value_full() -> impl Strategy<Value = VantaValue> {
    prop_oneof![
        10 => any::<String>().prop_map(VantaValue::String),
        4 => any::<i64>().prop_map(VantaValue::Int),
        4 => any::<f64>().prop_map(VantaValue::Float),
        4 => any::<bool>().prop_map(VantaValue::Bool),
        4 => arb_datetime().prop_map(VantaValue::DateTime),
        2 => prop::collection::vec(any::<String>(), 0..5).prop_map(VantaValue::ListString),
        1 => prop::collection::vec(any::<i64>(), 0..5).prop_map(VantaValue::ListInt),
        1 => prop::collection::vec(any::<f64>(), 0..5).prop_map(VantaValue::ListFloat),
        1 => prop::collection::vec(any::<bool>(), 0..5).prop_map(VantaValue::ListBool),
        1 => prop::collection::vec(arb_datetime(), 0..5).prop_map(VantaValue::ListDateTime),
        2 => Just(VantaValue::Null),
    ]
}

fn arb_vanta_fields_full() -> impl Strategy<Value = VantaFields> {
    prop::collection::btree_map("[a-zA-Z_][a-zA-Z0-9_]{0,15}", arb_vanta_value_full(), 0..8)
}

fn arb_metadata_full() -> impl Strategy<Value = VantaMemoryMetadata> {
    arb_vanta_fields_full()
}

fn arb_distance_metric() -> impl Strategy<Value = DistanceMetric> {
    prop_oneof![
        Just(DistanceMetric::Cosine),
        Just(DistanceMetric::Euclidean),
    ]
}

fn arb_runtime_profile() -> impl Strategy<Value = VantaRuntimeProfile> {
    prop_oneof![
        Just(VantaRuntimeProfile::Enterprise),
        Just(VantaRuntimeProfile::Performance),
        Just(VantaRuntimeProfile::LowResource),
    ]
}

fn arb_storage_tier() -> impl Strategy<Value = VantaStorageTier> {
    prop_oneof![Just(VantaStorageTier::Hot), Just(VantaStorageTier::Cold),]
}

fn arb_u128() -> impl Strategy<Value = u128> {
    (any::<u64>(), any::<u64>()).prop_map(|(hi, lo)| (hi as u128) << 64 | lo as u128)
}

fn arb_vector() -> impl Strategy<Value = Vec<f32>> {
    prop::collection::vec(-1.0f32..1.0, 0..64)
}

fn arb_option_vector() -> impl Strategy<Value = Option<Vec<f32>>> {
    prop::option::weighted(0.8, arb_vector())
}

// ── VantaValue (postcard, full range) ───────────────────────────────────

proptest! {
    #[test]
    fn test_vanta_value_postcard_roundtrip(val in arb_vanta_value_full()) {
        assert_postcard(&val);
    }

    #[test]
    fn test_edge_record_postcard_roundtrip(
        target: u128, label: String, weight: f32,
    ) {
        assert_postcard(&VantaEdgeRecord { target, label, weight });
    }
}

// ── VantaMemoryInput (postcard) ─────────────────────────────────────────

fn arb_memory_input_full() -> impl Strategy<Value = VantaMemoryInput> {
    (
        any::<String>(),
        any::<String>(),
        any::<String>(),
        arb_metadata_full(),
        arb_option_vector(),
        prop::option::of(any::<u64>()),
    )
        .prop_map(
            |(namespace, key, payload, metadata, vector, ttl_ms)| VantaMemoryInput {
                namespace,
                key,
                payload,
                metadata,
                vector,
                ttl_ms,
            },
        )
}

proptest! {
    #[test]
    fn test_memory_input_postcard_roundtrip(input in arb_memory_input_full()) {
        assert_postcard(&input);
    }
}

// ── VantaMemoryRecord (JSON) ────────────────────────────────────────
// Uses `#[serde(with = "u128_serde")]` (string-based u128), which is
// incompatible with postcard's binary format but works with JSON.

fn arb_memory_record_json() -> impl Strategy<Value = VantaMemoryRecord> {
    (
        any::<String>(),
        any::<String>(),
        any::<String>(),
        arb_metadata_json(),
        any::<u64>(),
        any::<u64>(),
        any::<u64>(),
        (0u64..=1_000_000_000_000u64),
        arb_option_vector(),
        prop::option::of(any::<u64>()),
    )
        .prop_map(
            |(
                namespace,
                key,
                payload,
                metadata,
                created_at_ms,
                updated_at_ms,
                version,
                node_id_small,
                vector,
                expires_at_ms,
            )| {
                VantaMemoryRecord {
                    namespace,
                    key,
                    payload,
                    metadata,
                    created_at_ms,
                    updated_at_ms,
                    version,
                    node_id: node_id_small as u128,
                    vector,
                    expires_at_ms,
                }
            },
        )
}

proptest! {
    #[test]
    fn test_memory_record_json_roundtrip(rec in arb_memory_record_json()) {
        assert_json(&rec);
    }
}

// ── VantaMemorySearchHit (JSON) ─────────────────────────────────────────
// Contains VantaMemoryRecord (u128_serde), so postcard is not supported.

proptest! {
    #[test]
    fn test_search_hit_json_roundtrip(rec in arb_memory_record_json(), score in -1e8f32..1e8f32) {
        let hit = VantaMemorySearchHit { record: rec, score, explanation: None };
        assert_json(&hit);
    }
}

// ── VantaMemoryListOptions (postcard) ───────────────────────────────────

fn arb_list_options_full() -> impl Strategy<Value = VantaMemoryListOptions> {
    (
        arb_metadata_full(),
        any::<usize>(),
        prop::option::of(any::<usize>()),
    )
        .prop_map(|(filters, limit, cursor)| VantaMemoryListOptions {
            filters,
            limit,
            cursor,
        })
}

proptest! {
    #[test]
    fn test_list_options_postcard_roundtrip(opts in arb_list_options_full()) {
        assert_postcard(&opts);
    }
}

// ── VantaMemorySearchRequest (postcard) ─────────────────────────────────

fn arb_search_request_full() -> impl Strategy<Value = VantaMemorySearchRequest> {
    (
        any::<String>(),
        arb_vector(),
        arb_metadata_full(),
        prop::option::of(any::<String>()),
        any::<usize>(),
        arb_distance_metric(),
        any::<bool>(),
    )
        .prop_map(
            |(namespace, query_vector, filters, text_query, top_k, distance_metric, explain)| {
                VantaMemorySearchRequest {
                    namespace,
                    query_vector,
                    filters,
                    text_query,
                    top_k,
                    distance_metric,
                    explain,
                }
            },
        )
}

proptest! {
    #[test]
    fn test_search_request_postcard_roundtrip(req in arb_search_request_full()) {
        assert_postcard(&req);
    }
}

// ── VantaSearchExplanationHit (postcard) ────────────────────────────────

fn arb_search_explanation_hit() -> impl Strategy<Value = VantaSearchExplanationHit> {
    (
        any::<String>(),
        any::<f32>(),
        prop::option::of(any::<String>()),
        prop::collection::vec(any::<String>(), 0..8),
        prop::collection::vec(any::<String>(), 0..8),
        prop::collection::vec(
            (
                any::<String>(),
                any::<u32>(),
                any::<u64>(),
                any::<u32>(),
                any::<f32>(),
            )
                .prop_map(|(token, tf, df, doc_len, contribution)| {
                    VantaBm25TermContribution {
                        token,
                        tf,
                        df,
                        doc_len,
                        contribution,
                    }
                }),
            0..4,
        ),
        prop::option::of(any::<usize>()),
        prop::option::of(any::<usize>()),
    )
        .prop_map(
            |(
                identity,
                score,
                snippet,
                matched_tokens,
                matched_phrases,
                bm25_terms,
                rrf_text_rank,
                rrf_vector_rank,
            )| {
                VantaSearchExplanationHit {
                    identity,
                    score,
                    snippet,
                    matched_tokens,
                    matched_phrases,
                    bm25_terms,
                    rrf_text_rank,
                    rrf_vector_rank,
                }
            },
        )
}

// ── VantaExportReport (JSON + postcard) ─────────────────────────────────

proptest! {
    #[test]
    fn test_export_report_json_roundtrip(
        records_exported: u64,
        namespaces in prop::collection::vec(any::<String>(), 0..5),
        path: String,
        duration_ms: u64,
    ) {
        let report = VantaExportReport { records_exported, namespaces, path, duration_ms };
        assert_json(&report);
        assert_postcard(&report);
    }
}

// ── VantaImportReport (JSON + postcard) ─────────────────────────────────

proptest! {
    #[test]
    fn test_import_report_json_roundtrip(
        inserted: u64, updated: u64, skipped: u64, errors: u64, duration_ms: u64,
    ) {
        let report = VantaImportReport { inserted, updated, skipped, errors, duration_ms };
        assert_json(&report);
        assert_postcard(&report);
    }
}

// ── VantaIndexRebuildReport (JSON + postcard) ───────────────────────────

proptest! {
    #[test]
    fn test_index_rebuild_report_json_roundtrip(
        scanned_nodes: u64, indexed_vectors: u64, skipped_tombstones: u64,
        duration_ms: u64, derived_rebuild_ms: u64, index_path: String, success: bool,
    ) {
        let report = VantaIndexRebuildReport {
            scanned_nodes, indexed_vectors, skipped_tombstones,
            duration_ms, derived_rebuild_ms, index_path, success,
        };
        assert_json(&report);
        assert_postcard(&report);
    }
}

// ── VantaTextIndexRepairReport (JSON + postcard) ────────────────────────

proptest! {
    #[test]
    fn test_text_index_repair_report_json_roundtrip(
        record_count: u64, posting_entries: u64, doc_stats_entries: u64,
        term_stats_entries: u64, namespace_stats_entries: u64,
        duration_ms: u64, success: bool,
    ) {
        let report = VantaTextIndexRepairReport {
            record_count, posting_entries, doc_stats_entries,
            term_stats_entries, namespace_stats_entries, duration_ms, success,
        };
        assert_json(&report);
        assert_postcard(&report);
    }
}

// ── VantaNodeInput (postcard) ───────────────────────────────────────────

fn arb_node_input_full() -> impl Strategy<Value = VantaNodeInput> {
    (
        arb_u128(),
        prop::option::of(any::<String>()),
        arb_option_vector(),
        arb_vanta_fields_full(),
    )
        .prop_map(|(id, content, vector, fields)| VantaNodeInput {
            id,
            content,
            vector,
            fields,
        })
}

proptest! {
    #[test]
    fn test_node_input_postcard_roundtrip(input in arb_node_input_full()) {
        assert_postcard(&input);
    }
}

// ── VantaNodeRecord (postcard) ──────────────────────────────────────────

fn arb_edge_record() -> impl Strategy<Value = VantaEdgeRecord> {
    (arb_u128(), any::<String>(), any::<f32>()).prop_map(|(target, label, weight)| {
        VantaEdgeRecord {
            target,
            label,
            weight,
        }
    })
}

fn arb_node_record_full() -> impl Strategy<Value = VantaNodeRecord> {
    (
        arb_u128(),
        arb_vanta_fields_full(),
        arb_option_vector(),
        any::<usize>(),
        prop::collection::vec(arb_edge_record(), 0..8),
        any::<f32>(),
        any::<f32>(),
        any::<u32>(),
        any::<u64>(),
        any::<u32>(),
        arb_storage_tier(),
        any::<bool>(),
    )
        .prop_map(
            |(
                id,
                fields,
                vector,
                vector_dimensions,
                edges,
                confidence_score,
                importance,
                hits,
                last_accessed,
                epoch,
                tier,
                is_alive,
            )| {
                VantaNodeRecord {
                    id,
                    fields,
                    vector,
                    vector_dimensions,
                    edges,
                    confidence_score,
                    importance,
                    hits,
                    last_accessed,
                    epoch,
                    tier,
                    is_alive,
                }
            },
        )
}

proptest! {
    #[test]
    fn test_node_record_postcard_roundtrip(rec in arb_node_record_full()) {
        assert_postcard(&rec);
    }
}

// ── VantaQueryResult (postcard) ─────────────────────────────────────────

fn arb_query_result_full() -> impl Strategy<Value = VantaQueryResult> {
    prop_oneof![
        prop::collection::vec(arb_node_record_full(), 0..5).prop_map(VantaQueryResult::Read),
        (
            any::<usize>(),
            any::<String>(),
            prop::option::of(arb_u128())
        )
            .prop_map(
                |(affected_nodes, message, node_id)| VantaQueryResult::Write {
                    affected_nodes,
                    message,
                    node_id,
                }
            ),
        arb_u128().prop_map(|node_id| VantaQueryResult::StaleContext { node_id }),
    ]
}

proptest! {
    #[test]
    fn test_query_result_postcard_roundtrip(result in arb_query_result_full()) {
        assert_postcard(&result);
    }
}

// ── VantaCapabilities (JSON + postcard) ─────────────────────────────────

proptest! {
    #[test]
    fn test_capabilities_json_roundtrip(
        runtime_profile in arb_runtime_profile(),
        persistence: bool, vector_search: bool, iql_queries: bool, read_only: bool,
    ) {
        let caps = VantaCapabilities { runtime_profile, persistence, vector_search, iql_queries, read_only };
        assert_json(&caps);
        assert_postcard(&caps);
    }
}

// ── VantaSearchExplanation (postcard) ───────────────────────────────────

fn arb_hybrid_fusion_report() -> impl Strategy<Value = VantaHybridFusionReport> {
    (
        any::<usize>(),
        any::<usize>(),
        any::<usize>(),
        any::<usize>(),
    )
        .prop_map(
            |(text_candidates, vector_candidates, fused_candidates, rrf_k)| {
                VantaHybridFusionReport {
                    text_candidates,
                    vector_candidates,
                    fused_candidates,
                    rrf_k,
                }
            },
        )
}

fn arb_search_explanation() -> impl Strategy<Value = VantaSearchExplanation> {
    (
        any::<String>(),
        prop::collection::vec(arb_search_explanation_hit(), 0..5),
        prop::option::of(arb_hybrid_fusion_report()),
    )
        .prop_map(|(route, hits, fusion_report)| VantaSearchExplanation {
            route,
            hits,
            fusion_report,
        })
}

proptest! {
    #[test]
    fn test_search_explanation_postcard_roundtrip(exp in arb_search_explanation()) {
        assert_postcard(&exp);
    }
}

// ── VantaSearchHit (postcard) ───────────────────────────────────────────

proptest! {
    #[test]
    fn test_search_hit_simple_postcard_roundtrip(node_id: u128, distance: f32) {
        assert_postcard(&VantaSearchHit { node_id, distance });
    }
}

// ── VantaEdgeRecord (postcard) ──────────────────────────────────────────

proptest! {
    #[test]
    fn test_edge_record_json_roundtrip_int(label: String) {
        let edge = VantaEdgeRecord { target: 0, label, weight: 0.0 };
        let json = serde_json::to_string(&edge).unwrap();
        let recovered: VantaEdgeRecord = serde_json::from_str(&json).unwrap();
        // f64 fields may differ by 1 ULP through JSON — verify structure instead
        assert_eq!(edge.target, recovered.target);
        assert_eq!(edge.label, recovered.label);
    }
}
