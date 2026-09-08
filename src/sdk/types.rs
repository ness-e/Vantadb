//! Stable public types for the VantaDB SDK boundary.
//! All types in this module are serializable and designed for third-party bindings.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod graph;
mod record;
mod search;

pub use graph::{VantaEdgeRecord, VantaNodeInput, VantaNodeRecord, VantaQueryResult};
pub use record::{
    VantaExportReport, VantaFilterOp, VantaImportReport, VantaMemoryExportLine, VantaMemoryFilter,
    VantaMemoryFilterItem, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryListPage,
    VantaMemoryRecord, VantaNamespaceStats, VantaNamespaceStatsMap,
    DEFAULT_EXPIRING_SOON_WINDOW_MS,
};
#[cfg(debug_assertions)]
pub use search::VantaMemorySearchDebugReport;
pub(crate) use search::{
    DerivedIndexRebuildReport, DerivedIndexState, ExpectedTextIndexEntries, SparseIndexCounts,
    SparseIndexRebuildReport, SparseIndexState, TextIndexCounts, TextIndexMutationReport,
    TextIndexRebuildReport, TextIndexState,
};
pub use search::{
    SearchProfileConfig, SearchProfileMode, VantaBm25TermContribution, VantaHybridFusionReport,
    VantaIndexRebuildReport, VantaMemorySearchHit, VantaMemorySearchRequest,
    VantaSearchExplanation, VantaSearchExplanationHit, VantaSearchHit, VantaTextIndexAuditReport,
    VantaTextIndexRepairReport,
};

pub(crate) mod u128_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(val: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&val.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum U128 {
            Str(String),
            Num(u64),
        }
        match U128::deserialize(deserializer)? {
            U128::Str(s) => s.parse().map_err(serde::de::Error::custom),
            U128::Num(n) => Ok(n as u128),
        }
    }
}

/// Stable runtime profile exposed to SDKs without leaking hardware internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VantaRuntimeProfile {
    /// High-resource profile for enterprise-class hardware (AVX-512, 16+ GB RAM).
    Enterprise,
    /// Standard server profile (AVX2/NEON, 4+ GB RAM).
    Performance,
    /// Constrained profile for low-resource devices.
    LowResource,
}

/// Stable storage tier view for external SDKs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VantaStorageTier {
    /// Hot tier for frequently accessed nodes.
    Hot,
    /// Cold tier for infrequently accessed nodes.
    Cold,
}

/// Stable field value representation for external SDKs.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum VantaValue {
    /// UTF-8 string value.
    String(String),
    /// Signed 64-bit integer.
    Int(i64),
    /// 64-bit floating point number.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// RFC 3339 datetime with timezone.
    DateTime(chrono::DateTime<chrono::Utc>),
    /// List of UTF-8 strings.
    ListString(Vec<String>),
    /// List of signed 64-bit integers.
    ListInt(Vec<i64>),
    /// List of 64-bit floating point numbers.
    ListFloat(Vec<f64>),
    /// List of booleans.
    ListBool(Vec<bool>),
    /// List of RFC 3339 datetimes with timezone.
    ListDateTime(Vec<chrono::DateTime<chrono::Utc>>),
    /// Explicit null value.
    Null,
}

impl VantaValue {
    /// Flatten list variants into individual scalar values for index storage.
    /// Non-list variants return a single-element vector containing a clone of self.
    pub fn to_index_values(&self) -> Vec<VantaValue> {
        match self {
            VantaValue::ListString(vec) => {
                vec.iter().map(|s| VantaValue::String(s.clone())).collect()
            }
            VantaValue::ListInt(vec) => vec.iter().map(|&i| VantaValue::Int(i)).collect(),
            VantaValue::ListFloat(vec) => vec.iter().map(|&f| VantaValue::Float(f)).collect(),
            VantaValue::ListBool(vec) => vec.iter().map(|&b| VantaValue::Bool(b)).collect(),
            VantaValue::ListDateTime(vec) => {
                vec.iter().map(|&dt| VantaValue::DateTime(dt)).collect()
            }
            other => vec![other.clone()],
        }
    }
}

/// Stable relational fields map for external SDKs.
pub type VantaFields = BTreeMap<String, VantaValue>;

/// Stable metadata map for persistent memory records.
pub type VantaMemoryMetadata = VantaFields;

/// Stable snapshot of operational metrics used for validation and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaOperationalMetrics {
    /// Engine startup duration in milliseconds.
    pub startup_ms: u64,
    /// WAL replay duration in milliseconds.
    pub wal_replay_ms: u64,
    /// Number of records replayed from the WAL during startup.
    pub wal_records_replayed: u64,
    /// ANN index rebuild duration in milliseconds.
    pub ann_rebuild_ms: u64,
    /// Number of nodes scanned during the last ANN rebuild.
    pub ann_rebuild_scanned_nodes: u64,
    /// Derived (namespace/payload) index rebuild duration in milliseconds.
    pub derived_rebuild_ms: u64,
    /// Text index rebuild duration in milliseconds.
    pub text_index_rebuild_ms: u64,
    /// Total text index postings written.
    pub text_postings_written: u64,
    /// Total text index repairs triggered.
    pub text_index_repairs: u64,
    /// Total BM25 lexical queries executed.
    pub text_lexical_queries: u64,
    /// Cumulative time spent on BM25 lexical queries in milliseconds.
    pub text_lexical_query_ms: u64,
    /// Total BM25 candidates scored across all queries.
    pub text_candidates_scored: u64,
    /// Total text index consistency audits performed.
    pub text_consistency_audits: u64,
    /// Total text index consistency audits that detected drift.
    pub text_consistency_audit_failures: u64,
    /// Cumulative time spent on hybrid queries in milliseconds.
    pub hybrid_query_ms: u64,
    /// Total unique candidates fused across all hybrid queries.
    pub hybrid_candidates_fused: u64,
    /// Total queries planned as hybrid (text+vector).
    pub planner_hybrid_queries: u64,
    /// Total queries planned as text-only.
    pub planner_text_only_queries: u64,
    /// Total queries planned as vector-only.
    pub planner_vector_only_queries: u64,
    /// Total records exported.
    pub records_exported: u64,
    /// Total records imported.
    pub records_imported: u64,
    /// Total import errors encountered.
    pub import_errors: u64,
    /// Total derived index prefix scans performed.
    pub derived_prefix_scans: u64,
    /// Total fallbacks to full scan when derived index was absent.
    pub derived_full_scan_fallbacks: u64,
    /// Process resident set size in bytes (OS-reported).
    pub process_rss_bytes: u64,
    /// Process virtual memory in bytes (OS-reported).
    pub process_virtual_bytes: u64,
    /// Number of nodes in the HNSW index.
    pub hnsw_nodes_count: u64,
    /// Estimated logical footprint of HNSW allocations.
    pub hnsw_logical_bytes: u64,
    /// OS-reported resident bytes for mmap-backed files when available.
    pub mmap_resident_bytes: Option<u64>,
    /// Number of entries in the volatile hot-node cache.
    pub volatile_cache_entries: u64,
    /// Maximum capacity in bytes for the volatile cache.
    pub volatile_cache_cap_bytes: u64,
    /// Bytes allocated by jemalloc, if available.
    pub jemalloc_allocated_bytes: Option<u64>,
    /// Bytes in active pages allocated by jemalloc, if available.
    pub jemalloc_active_bytes: Option<u64>,
    /// Bytes dedicated to jemalloc metadata, if available.
    pub jemalloc_metadata_bytes: Option<u64>,
    /// Bytes in resident pages allocated by jemalloc, if available.
    pub jemalloc_resident_bytes: Option<u64>,
    /// Bytes mapped by jemalloc, if available.
    pub jemalloc_mapped_bytes: Option<u64>,
    /// Bytes in retained pages by jemalloc, if available.
    pub jemalloc_retained_bytes: Option<u64>,
    /// L1 extraction latency in milliseconds (last observed; TDAM metric-tracking-l1).
    pub l1_extraction_latency_ms: u64,
    /// L1 dedup latency in milliseconds (last observed).
    pub l1_dedup_latency_ms: u64,
    /// L2 extraction latency in milliseconds (last observed; TDAM metric-tracking-l2).
    pub l2_extraction_latency_ms: u64,
    /// L2 LLM call duration in milliseconds (last observed).
    pub l2_llm_duration_ms: u64,
    /// L3 generation latency in milliseconds (last observed; TDAM metric-tracking-l3).
    pub l3_generation_latency_ms: u64,
    /// Persona context length before L3 update.
    pub persona_length_before: u64,
    /// Persona context length after L3 update.
    pub persona_length_after: u64,
    /// Persona drift ratio scaled by 10_000 (basis points; 10_000 == 1.0).
    pub persona_drift_ratio: u64,
    /// Total recall queries that produced at least one hit.
    pub recall_hit_count: u64,
    /// Best recall hit score scaled by 10_000 (basis points; 10_000 == 1.0).
    pub recall_top_score: u64,
    /// Recall query latency in milliseconds (last observed; TDAM metric-tracking-recall).
    pub recall_latency_ms: u64,
    /// Recall strategy code used by the last query: 0=skipped, 1=keyword, 2=embedding, 3=hybrid.
    pub recall_strategy: u64,
    /// Offload (memory compaction) latency in milliseconds (last observed).
    pub offload_latency_ms: u64,
}

/// Stable capabilities summary exposed to external SDKs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaCapabilities {
    /// Current runtime performance profile.
    pub runtime_profile: VantaRuntimeProfile,
    /// Whether the database persists data to disk.
    pub persistence: bool,
    /// Whether vector search via HNSW is available.
    pub vector_search: bool,
    /// Whether IQL query parsing and execution is available.
    pub iql_queries: bool,
    /// Whether the database is in read-only mode.
    pub read_only: bool,
}

/// A single immutable version of a skill (agent skill / memory skill).
///
/// Skills are versioned: every successful `update`/`patch` appends a new
/// version and flips `is_head` on the previous head. `content_hash` enables
/// idempotent writes (same content → no-op), `expires_at` drives TTL cleanup
/// that keeps the most recent non-head versions (KEEP_RECENT = 3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillRecord {
    /// Stable skill identifier (e.g. `skl-...`), immutable across versions.
    pub skill_id: String,
    /// Monotonic version number, starting at 1.
    pub version: u64,
    /// Whether this version is the current head of the skill.
    pub is_head: bool,
    /// Owning agent identifier — part of the unique `(owner_agent, name)` key.
    pub owner_agent: String,
    /// Skill name — part of the unique `(owner_agent, name)` key. Immutable.
    pub name: String,
    /// Human-readable skill description.
    pub description: String,
    /// Skill body content (e.g. the SKILL.md text).
    pub content: String,
    /// Non-cryptographic content hash (FNV-1a 64-bit, hex) for idempotency.
    pub content_hash: String,
    /// Arbitrary skill metadata.
    pub metadata: BTreeMap<String, String>,
    /// Unix seconds when this version was created.
    pub created_at: u64,
    /// Unix seconds when this version was last written.
    pub updated_at: u64,
    /// Unix seconds after which this version is eligible for TTL cleanup
    /// (`None` = never expires).
    pub expires_at: Option<u64>,
}

/// Input for creating a new skill (version 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillCreateInput {
    /// Skill name — unique per `owner_agent`. Immutable after creation.
    pub name: String,
    /// Human-readable skill description.
    #[serde(default)]
    pub description: String,
    /// Skill body content.
    pub content: String,
    /// Owning agent identifier.
    pub owner_agent: String,
    /// Arbitrary skill metadata.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    /// Optional TTL: when set, this and future versions expire after `ttl_secs`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_secs: Option<u64>,
}

/// Input for updating a skill: replaces description and content, appends a
/// new version. `metadata: None` keeps the previous metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillUpdateInput {
    /// New description (replaces the previous one).
    pub description: String,
    /// New content (replaces the previous one).
    pub content: String,
    /// New metadata, or `None` to keep the previous metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, String>>,
}

/// Input for patching a skill: only the provided fields change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillPatchInput {
    /// New description, or `None` to keep the previous one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New content, or `None` to keep the previous one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// New metadata, or `None` to keep the previous one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, String>>,
}

/// Options for listing skills (heads only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillListOptions {
    /// Only list skills owned by this agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_agent: Option<String>,
    /// Only list skills whose name starts with this prefix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_prefix: Option<String>,
    /// Maximum number of items to return (default 50).
    #[serde(default = "default_skill_list_limit")]
    pub limit: usize,
    /// Number of items to skip.
    #[serde(default)]
    pub offset: usize,
}

fn default_skill_list_limit() -> usize {
    50
}

impl Default for SkillListOptions {
    fn default() -> Self {
        Self {
            owner_agent: None,
            name_prefix: None,
            limit: default_skill_list_limit(),
            offset: 0,
        }
    }
}

/// A page of skills returned by [`SkillListOptions`]-based listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillListPage {
    /// The listed skill heads.
    pub items: Vec<SkillRecord>,
    /// Total number of matching skills (before pagination).
    pub total: usize,
}

/// Result of a skill write. `idempotent = true` means the write was a no-op
/// because the content hash already matched the head (no new version appended).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillWriteResult {
    /// The head version after the write.
    pub record: SkillRecord,
    /// Whether the write was skipped as idempotent (no version appended).
    pub idempotent: bool,
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    // ΓöÇΓöÇ VantaRuntimeProfile ΓöÇΓöÇ

    #[test]
    fn test_runtime_profile_variants() {
        assert_ne!(
            VantaRuntimeProfile::Enterprise,
            VantaRuntimeProfile::Performance
        );
        assert_ne!(
            VantaRuntimeProfile::LowResource,
            VantaRuntimeProfile::Enterprise
        );
    }

    #[test]
    fn test_runtime_profile_clone_copy() {
        let p = VantaRuntimeProfile::Performance;
        let copied = p;
        assert_eq!(p, copied);
    }

    #[test]
    fn test_runtime_profile_debug() {
        let d = format!("{:?}", VantaRuntimeProfile::LowResource);
        assert_eq!(d, "LowResource");
    }

    // ΓöÇΓöÇ VantaStorageTier ΓöÇΓöÇ

    #[test]
    fn test_storage_tier_variants() {
        assert_ne!(VantaStorageTier::Hot, VantaStorageTier::Cold);
    }

    #[test]
    fn test_storage_tier_debug() {
        let h = format!("{:?}", VantaStorageTier::Hot);
        assert_eq!(h, "Hot");
    }

    // ΓöÇΓöÇ VantaValue ΓöÇΓöÇ

    #[test]
    fn test_vanta_value_string() {
        let v = VantaValue::String("hello".into());
        assert_eq!(
            v.to_index_values(),
            vec![VantaValue::String("hello".into())]
        );
    }

    #[test]
    fn test_vanta_value_int() {
        let v = VantaValue::Int(42);
        assert_eq!(v.to_index_values(), vec![VantaValue::Int(42)]);
    }

    #[test]
    fn test_vanta_value_float() {
        let v = VantaValue::Float(42.5);
        assert_eq!(v.to_index_values(), vec![VantaValue::Float(42.5)]);
    }

    #[test]
    fn test_vanta_value_bool() {
        let v = VantaValue::Bool(true);
        assert_eq!(v.to_index_values(), vec![VantaValue::Bool(true)]);
    }

    #[test]
    fn test_vanta_value_null() {
        let v = VantaValue::Null;
        assert_eq!(v.to_index_values(), vec![VantaValue::Null]);
    }

    #[test]
    fn test_vanta_value_datetime() {
        let dt: chrono::DateTime<chrono::Utc> = "2025-01-01T00:00:00Z".parse().unwrap();
        let v = VantaValue::DateTime(dt);
        let values = v.to_index_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], VantaValue::DateTime(dt));
    }

    #[test]
    fn test_vanta_value_to_index_list_string() {
        let v = VantaValue::ListString(vec!["a".into(), "b".into(), "c".into()]);
        let values = v.to_index_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], VantaValue::String("a".into()));
        assert_eq!(values[2], VantaValue::String("c".into()));
    }

    #[test]
    fn test_vanta_value_to_index_list_int() {
        let v = VantaValue::ListInt(vec![1, 2, 3]);
        let values = v.to_index_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values[1], VantaValue::Int(2));
    }

    #[test]
    fn test_vanta_value_to_index_list_float() {
        let v = VantaValue::ListFloat(vec![1.0, 2.0]);
        let values = v.to_index_values();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_vanta_value_to_index_list_bool() {
        let v = VantaValue::ListBool(vec![true, false, true]);
        let values = v.to_index_values();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_vanta_value_to_index_list_datetime() {
        let dt: chrono::DateTime<chrono::Utc> = "2025-06-15T12:00:00Z".parse().unwrap();
        let v = VantaValue::ListDateTime(vec![dt]);
        let values = v.to_index_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], VantaValue::DateTime(dt));
    }

    #[test]
    fn test_vanta_value_to_index_empty_list() {
        let v = VantaValue::ListString(vec![]);
        let values = v.to_index_values();
        assert!(values.is_empty());
    }

    #[test]
    fn test_vanta_value_clone() {
        let v = VantaValue::String("test".into());
        let cloned = v.clone();
        assert_eq!(v, cloned);
    }

    #[test]
    fn test_vanta_value_debug() {
        let d = format!("{:?}", VantaValue::Bool(false));
        assert!(d.contains("Bool") || d.contains("false"));
    }

    // ΓöÇΓöÇ VantaCapabilities ΓöÇΓöÇ

    #[test]
    fn test_capabilities_default() {
        let caps = VantaCapabilities {
            runtime_profile: VantaRuntimeProfile::Performance,
            persistence: true,
            vector_search: true,
            iql_queries: false,
            read_only: false,
        };
        assert_eq!(caps.runtime_profile, VantaRuntimeProfile::Performance);
        assert!(caps.persistence);
        assert!(caps.vector_search);
        assert!(!caps.iql_queries);
        assert!(!caps.read_only);
    }

    // ΓöÇΓöÇ VantaOperationalMetrics ΓöÇΓöÇ

    #[test]
    fn test_operational_metrics_defaults() {
        let m = VantaOperationalMetrics {
            startup_ms: 100,
            wal_replay_ms: 50,
            wal_records_replayed: 200,
            ann_rebuild_ms: 300,
            ann_rebuild_scanned_nodes: 1000,
            derived_rebuild_ms: 80,
            text_index_rebuild_ms: 150,
            text_postings_written: 5000,
            text_index_repairs: 1,
            text_lexical_queries: 42,
            text_lexical_query_ms: 120,
            text_candidates_scored: 10000,
            text_consistency_audits: 3,
            text_consistency_audit_failures: 0,
            hybrid_query_ms: 200,
            hybrid_candidates_fused: 500,
            planner_hybrid_queries: 10,
            planner_text_only_queries: 5,
            planner_vector_only_queries: 8,
            records_exported: 100,
            records_imported: 50,
            import_errors: 2,
            derived_prefix_scans: 30,
            derived_full_scan_fallbacks: 1,
            process_rss_bytes: 1_000_000,
            process_virtual_bytes: 2_000_000,
            hnsw_nodes_count: 500,
            hnsw_logical_bytes: 10_000_000,
            mmap_resident_bytes: Some(500_000),
            volatile_cache_entries: 100,
            volatile_cache_cap_bytes: 1_000_000,
            jemalloc_allocated_bytes: Some(2_000_000),
            jemalloc_active_bytes: Some(1_500_000),
            jemalloc_metadata_bytes: Some(100_000),
            jemalloc_resident_bytes: Some(1_800_000),
            jemalloc_mapped_bytes: Some(3_000_000),
            jemalloc_retained_bytes: Some(500_000),
            l1_extraction_latency_ms: 15,
            l1_dedup_latency_ms: 5,
            l2_extraction_latency_ms: 25,
            l2_llm_duration_ms: 80,
            l3_generation_latency_ms: 45,
            persona_length_before: 120,
            persona_length_after: 150,
            persona_drift_ratio: 2_500,
            recall_hit_count: 7,
            recall_top_score: 9_500,
            recall_latency_ms: 30,
            recall_strategy: 3,
            offload_latency_ms: 60,
        };
        assert_eq!(m.startup_ms, 100);
        assert_eq!(m.hnsw_nodes_count, 500);
        assert_eq!(m.jemalloc_allocated_bytes, Some(2_000_000));
    }

    #[test]
    fn test_operational_metrics_clone_debug() {
        let m = VantaOperationalMetrics {
            startup_ms: 1,
            wal_replay_ms: 2,
            wal_records_replayed: 3,
            ann_rebuild_ms: 4,
            ann_rebuild_scanned_nodes: 5,
            derived_rebuild_ms: 6,
            text_index_rebuild_ms: 7,
            text_postings_written: 8,
            text_index_repairs: 9,
            text_lexical_queries: 10,
            text_lexical_query_ms: 11,
            text_candidates_scored: 12,
            text_consistency_audits: 13,
            text_consistency_audit_failures: 14,
            hybrid_query_ms: 15,
            hybrid_candidates_fused: 16,
            planner_hybrid_queries: 17,
            planner_text_only_queries: 18,
            planner_vector_only_queries: 19,
            records_exported: 20,
            records_imported: 21,
            import_errors: 22,
            derived_prefix_scans: 23,
            derived_full_scan_fallbacks: 24,
            process_rss_bytes: 25,
            process_virtual_bytes: 26,
            hnsw_nodes_count: 27,
            hnsw_logical_bytes: 28,
            mmap_resident_bytes: None,
            volatile_cache_entries: 29,
            volatile_cache_cap_bytes: 30,
            jemalloc_allocated_bytes: None,
            jemalloc_active_bytes: None,
            jemalloc_metadata_bytes: None,
            jemalloc_resident_bytes: None,
            jemalloc_mapped_bytes: None,
            jemalloc_retained_bytes: None,
            l1_extraction_latency_ms: 31,
            l1_dedup_latency_ms: 32,
            l2_extraction_latency_ms: 33,
            l2_llm_duration_ms: 34,
            l3_generation_latency_ms: 35,
            persona_length_before: 36,
            persona_length_after: 37,
            persona_drift_ratio: 38,
            recall_hit_count: 39,
            recall_top_score: 40,
            recall_latency_ms: 41,
            recall_strategy: 42,
            offload_latency_ms: 43,
        };
        let cloned = m.clone();
        assert_eq!(m, cloned);
        let dbg = format!("{:?}", m);
        assert!(dbg.contains("startup_ms"));
    }

    // ΓöÇΓöÇ VantaCapabilities clone/debug ΓöÇΓöÇ

    #[test]
    fn test_capabilities_clone() {
        let caps = VantaCapabilities {
            runtime_profile: VantaRuntimeProfile::Enterprise,
            persistence: true,
            vector_search: false,
            iql_queries: true,
            read_only: false,
        };
        let cloned = caps.clone();
        assert_eq!(caps, cloned);
    }

    #[test]
    fn test_capabilities_debug() {
        let caps = VantaCapabilities {
            runtime_profile: VantaRuntimeProfile::Performance,
            persistence: false,
            vector_search: true,
            iql_queries: false,
            read_only: true,
        };
        let dbg = format!("{:?}", caps);
        assert!(dbg.contains("Performance"));
        assert!(dbg.contains("read_only"));
    }

    // ΓöÇΓöÇ VantaValue Debug variant coverage ΓöÇΓöÇ

    #[test]
    fn test_vanta_value_debug_variants() {
        assert!(format!("{:?}", VantaValue::String("a".into())).contains("String"));
        assert!(format!("{:?}", VantaValue::Int(1)).contains("Int"));
        assert!(format!("{:?}", VantaValue::Float(1.0)).contains("Float"));
        assert!(format!("{:?}", VantaValue::Null).contains("Null"));
        assert!(format!("{:?}", VantaValue::ListString(vec!["a".into()])).contains("List"));
    }
}
