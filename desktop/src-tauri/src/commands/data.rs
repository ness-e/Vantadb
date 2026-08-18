//! Data-facing IPC commands (DESK-06): ingest/search/get/delete/list.
//!
//! These are thin wrappers that route to the active connection held by
//! [`ConnectionManager`](crate::connections::ConnectionManager). Ids and
//! namespaces are accepted as owned `String`/`Option<String>` because Tauri
//! deserializes command args into owned values (no borrowing across the IPC
//! boundary). Each command propagates [`VantaError`] straight back to the
//! frontend.

use tauri::State;

use crate::connections::{IngestItem, ListPage, MemoryRecord, SearchQuery, SearchResult};
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
