//! Multi-connection contract for the VantaDB desktop app.
//!
//! [`VantaConnection`] is the trait every backend adapter (native / server / HTTP / MCP)
//! implements. Shared serde DTOs live in [`types`]; the unified error lives in
//! [`crate::error`].
//!
//! > NOTE (DESKTOP-04): DESK-02/DESKTOP-08 had in-flight work in this module. The
//! > multi-connection contract and the HTTP `server_client` coexist: DESK-08's wire
//! > DTOs were relocated verbatim to [`wire_types`]; DESK-02's `server_client.rs`
//! > (not present yet) should use `crate::connections::wire_types`.

pub mod child_process;
pub mod manager;
pub mod native;
pub mod server;
pub mod server_client;
mod r#trait;
pub mod types;
pub mod wire_types;

pub use child_process::McpSpawn;
pub use manager::ConnectionManager;
pub use r#trait::VantaConnection;
pub use server::ServerConnection;
pub use server_client::ServerClient;
pub use types::{
    Bm25Term, Capability, ConnectionInfo, ConnectionStatus, ExplanationHit, ExportReport,
    HealthReport, HealthStatus, IngestItem, ListPage, MemoryFilterItem, MemoryRecord,
    NamespaceStats, NamespaceStatsMap, SearchQuery, SearchResult, VantaGraphEdgeInfo,
    VantaGraphNodeInfo, VantaGraphTraversalResult, VantaQueryResult,
};
pub use wire_types::ServerClientConfig;
