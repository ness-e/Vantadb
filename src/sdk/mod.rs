mod api;
pub(crate) mod builder;
pub mod connect;
mod graph;
mod search;
mod serialization;
pub(crate) mod types;

pub use builder::VantaEmbedded;
pub use connect::connect;
pub use serialization::{
    export_line_from_record, FIELD_CREATED_AT_MS, FIELD_EXPIRES_AT_MS, FIELD_KEY, FIELD_NAMESPACE,
    FIELD_PAYLOAD, FIELD_UPDATED_AT_MS, FIELD_VERSION,
};
pub use types::{
    VantaBm25TermContribution, VantaCapabilities, VantaEdgeRecord, VantaExportReport, VantaFields,
    VantaHybridFusionReport, VantaImportReport, VantaIndexRebuildReport, VantaMemoryInput,
    VantaMemoryListOptions, VantaMemoryListPage, VantaMemoryMetadata, VantaMemoryRecord,
    VantaMemorySearchHit, VantaMemorySearchRequest, VantaNodeInput, VantaNodeRecord,
    VantaOperationalMetrics, VantaQueryResult, VantaRuntimeProfile, VantaSearchExplanation,
    VantaSearchExplanationHit, VantaSearchHit, VantaStorageTier, VantaTextIndexAuditReport,
    VantaTextIndexRepairReport, VantaValue,
};
