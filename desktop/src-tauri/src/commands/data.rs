//! Data-facing IPC commands (DESK-06): ingest/search/get/delete/list.
//!
//! These are thin wrappers that route to the active connection held by
//! [`ConnectionManager`](crate::connections::ConnectionManager). Ids and
//! namespaces are accepted as owned `String`/`Option<String>` because Tauri
//! deserializes command args into owned values (no borrowing across the IPC
//! boundary). Each command propagates [`VantaError`] straight back to the
//! frontend.

use tauri::State;

use crate::connections::{
    ExportReport, IngestItem, ListPage, MemoryFilterItem, MemoryRecord, SearchQuery, SearchResult,
    VantaGraphNodeInfo, VantaGraphTraversalResult, VantaQueryResult,
};
use crate::error::VantaError;
use crate::AppState;

/// Upsert a single record by key on the active connection (create or replace),
/// optionally pinning an absolute unix-ms expiry. Returns the stored record.
#[tauri::command]
pub async fn vanta_put(
    state: State<'_, AppState>,
    namespace: Option<String>,
    key: String,
    payload: String,
    metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    expires_at_ms: Option<u64>,
) -> Result<MemoryRecord, VantaError> {
    let item = IngestItem {
        id: Some(key),
        namespace: namespace.unwrap_or_else(|| "default".into()),
        text: payload,
        embedding: None,
        metadata: metadata.unwrap_or_default(),
    };
    state.manager.put(item, expires_at_ms).await
}

/// Store one or more records on the active connection, returning assigned/kept ids.
#[tauri::command]
pub async fn vanta_ingest(
    state: State<'_, AppState>,
    records: Vec<IngestItem>,
) -> Result<Vec<String>, VantaError> {
    state.manager.ingest_batch(records).await
}

/// Batch alias of [`vanta_ingest`] — same semantics, explicit name for the UI.
#[tauri::command]
pub async fn vanta_ingest_batch(
    state: State<'_, AppState>,
    records: Vec<IngestItem>,
) -> Result<Vec<String>, VantaError> {
    state.manager.ingest_batch(records).await
}

/// Semantic / text search over the active connection, ordered by descending score.
#[tauri::command]
pub async fn vanta_search(
    state: State<'_, AppState>,
    query: SearchQuery,
) -> Result<Vec<SearchResult>, VantaError> {
    state.manager.search(query).await
}

/// Fetch a single record by key on the active connection.
#[tauri::command]
pub async fn vanta_get(
    state: State<'_, AppState>,
    key: String,
    namespace: Option<String>,
) -> Result<MemoryRecord, VantaError> {
    state.manager.get(&key, namespace.as_deref()).await
}

/// Fetch a record as it was at a specific version (VS-CORE-07). Only the
/// native (embedded) connection implements version history; other transports
/// reject with `Unsupported`.
#[tauri::command]
pub async fn vanta_get_version(
    state: State<'_, AppState>,
    key: String,
    version: u64,
    namespace: Option<String>,
) -> Result<MemoryRecord, VantaError> {
    state
        .manager
        .get_version(&key, version, namespace.as_deref())
        .await
}

/// List every retained version of a record, ascending v1..vN (VS-CORE-07).
/// Only the native (embedded) connection implements version history; other
/// transports reject with `Unsupported`.
#[tauri::command]
pub async fn vanta_versions(
    state: State<'_, AppState>,
    key: String,
    namespace: Option<String>,
) -> Result<Vec<MemoryRecord>, VantaError> {
    state.manager.versions(&key, namespace.as_deref()).await
}

/// Delete a record by key on the active connection. Idempotent.
#[tauri::command]
pub async fn vanta_delete(
    state: State<'_, AppState>,
    key: String,
    namespace: Option<String>,
) -> Result<(), VantaError> {
    state.manager.delete(&key, namespace.as_deref()).await
}

/// List a page of records on the active connection, capped at `limit`
/// (default 100). `cursor` comes from a previous page's `next_cursor` and
/// continues pagination; a page with `next_cursor: None` is the last one.
#[tauri::command]
pub async fn vanta_list(
    state: State<'_, AppState>,
    namespace: Option<String>,
    limit: Option<usize>,
    cursor: Option<usize>,
) -> Result<ListPage, VantaError> {
    state
        .manager
        .list_records(namespace.as_deref(), limit, cursor)
        .await
}

/// Execute an IQL statement against the active connection (VS-CORE-06).
///
/// Only the native (embedded) connection implements IQL; other transports
/// reject with `Unsupported`. The result is a `Read` (records), `Write`
/// (affected count), or `StaleContext` marker, as a discriminated union.
#[tauri::command]
pub async fn vanta_query(
    state: State<'_, AppState>,
    iql: String,
) -> Result<VantaQueryResult, VantaError> {
    state.manager.query(&iql).await
}

/// IQL editor autocomplete candidates for the token being typed (VS-CORE-06).
///
/// Pure string shim over the parser's keyword/identifier table — no
/// connection or backend access, so it is synchronous and always available.
#[tauri::command]
pub fn vanta_iql_autocomplete(prefix: String) -> Vec<String> {
    vantadb::parser::autocomplete_prefix(&prefix)
}

/// Export a namespace to a JSONL file on the active connection (VS-CORE-04).
///
/// `filter` is an optional AND-combined metadata filter (e.g. from the query
/// builder); `None` (or empty) exports the full namespace. Only the native
/// (embedded) connection implements file export; other transports reject with
/// `Unsupported`.
#[tauri::command]
pub async fn vanta_export_namespace(
    state: State<'_, AppState>,
    namespace: String,
    path: String,
    filter: Option<Vec<MemoryFilterItem>>,
) -> Result<ExportReport, VantaError> {
    state
        .manager
        .export_namespace(&namespace, &path, filter)
        .await
}

/// Delete every record in a namespace matching an AND-combined metadata filter
/// on the active connection (VS-CORE-05), returning the number deleted.
///
/// The core rejects an empty filter to prevent accidental full-namespace
/// deletion — that error propagates to the UI unchanged. Only the native
/// (embedded) connection implements batch delete; other transports reject with
/// `Unsupported`.
#[tauri::command]
pub async fn vanta_delete_by_filter(
    state: State<'_, AppState>,
    namespace: String,
    filter: Vec<MemoryFilterItem>,
) -> Result<u64, VantaError> {
    state.manager.delete_by_filter(&namespace, filter).await
}

/// Breadth-first graph traversal from root node ids on the active connection
/// (GRAFO-01). Returns the visited nodes plus their outgoing edges.
///
/// `roots` are node ids (u128, string-serialized on the wire). `direction` is
/// `"Forward"` / `"Reverse"` / `"Both"`. `limit` caps the result (default 50).
/// Only the native (embedded) connection implements graph traversal; other
/// transports reject with `Unsupported`.
#[tauri::command]
pub async fn vanta_graph_bfs(
    state: State<'_, AppState>,
    roots: Vec<String>,
    max_depth: usize,
    direction: String,
    limit: Option<usize>,
) -> Result<VantaGraphTraversalResult, VantaError> {
    state
        .manager
        .graph_bfs(roots, max_depth, direction, limit)
        .await
}

/// Depth-first graph traversal from root node ids on the active connection
/// (GRAFO-01). Same contract as [`vanta_graph_bfs`].
#[tauri::command]
pub async fn vanta_graph_dfs(
    state: State<'_, AppState>,
    roots: Vec<String>,
    max_depth: usize,
    direction: String,
    limit: Option<usize>,
) -> Result<VantaGraphTraversalResult, VantaError> {
    state
        .manager
        .graph_dfs(roots, max_depth, direction, limit)
        .await
}

/// Degree centrality (in+out) for every node in `namespace` on the active
/// connection (GRAFO-01), up to `limit` (default 50). An empty/unknown
/// namespace returns an empty list, not an error.
#[tauri::command]
pub async fn vanta_graph_degree(
    state: State<'_, AppState>,
    namespace: String,
    limit: Option<usize>,
) -> Result<Vec<VantaGraphNodeInfo>, VantaError> {
    state.manager.graph_degree(&namespace, limit).await
}
