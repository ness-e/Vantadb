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
pub use builder::VantaEmbedded;
pub use connect::connect;
pub use serialization::{
    export_line_from_record, memory_record_from_node, FIELD_CREATED_AT_MS, FIELD_EXPIRES_AT_MS,
    FIELD_KEY, FIELD_NAMESPACE, FIELD_PAYLOAD, FIELD_UPDATED_AT_MS, FIELD_VERSION,
};
pub use types::{
    VantaBm25TermContribution, VantaCapabilities, VantaEdgeRecord, VantaExportReport, VantaFields,
    VantaFilterOp, VantaHybridFusionReport, VantaImportReport, VantaIndexRebuildReport,
    VantaMemoryFilter, VantaMemoryFilterItem, VantaMemoryInput, VantaMemoryListOptions,
    VantaMemoryListPage, VantaMemoryMetadata, VantaMemoryRecord, VantaMemorySearchHit,
    VantaMemorySearchRequest, VantaNamespaceStats, VantaNamespaceStatsMap, VantaNodeInput,
    VantaNodeRecord, VantaOperationalMetrics, VantaQueryResult, VantaRuntimeProfile,
    VantaSearchExplanation, VantaSearchExplanationHit, VantaSearchHit, VantaStorageTier,
    VantaTextIndexAuditReport, VantaTextIndexRepairReport, VantaValue,
};
