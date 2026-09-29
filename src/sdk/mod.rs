//! Public SDK surface for the VantaDB embedded client.
//! Re-exports the core types, builder, and serialization helpers.

mod api;
pub(crate) mod builder;
pub mod connect;
mod gds;
mod graph;
pub(crate) mod search;
pub(crate) mod serialization;
pub(crate) mod types;
pub(crate) mod version_history;

pub use api::BulkImportReport;
pub use builder::Embedded;
pub use connect::connect;
#[allow(deprecated)]
pub use serialization::memory_record_from_node;
pub use serialization::{
    export_line_from_record, record_from_export_line, record_from_node, FIELD_CREATED_AT_MS,
    FIELD_EXPIRES_AT_MS, FIELD_KEY, FIELD_NAMESPACE, FIELD_PAYLOAD, FIELD_UPDATED_AT_MS,
    FIELD_VERSION,
};
pub use types::{
    default_confidence, Bm25TermContribution, Capabilities, ConfidenceClass, EdgeRecord,
    EntityBoost, EntityBoostProvenance, EntityBoostReport, EntityBoostedSearch, ExportReport,
    Fields, FilterOp, GroupByConfig, HybridFusionReport, ImportReport, IndexRebuildReport,
    MemoryExportLine, MemoryFilter, MemoryFilterItem, MemoryInput, MemoryListOptions,
    MemoryListPage, MemoryMetadata, MemoryRecord, MemorySearchHit, MemorySearchPage,
    MemorySearchRequest, MmrConfig, NamespaceStats, NamespaceStatsMap, NodeInput, NodeRecord,
    OperationalMetrics, QueryResult, RangeFilter, RuntimeProfile, SearchExplanation,
    SearchExplanationHit, SearchHit, SearchProfileConfig, SearchProfileMode, SkillCreateInput,
    SkillListOptions, SkillListPage, SkillPatchInput, SkillRecord, SkillUpdateInput,
    SkillWriteResult, StorageTier, TextIndexAuditReport, TextIndexRepairReport, ValidWindow, Value,
    DERIVATION_DISCOUNT, MAX_DERIVATION_DEPTH,
};
