//! HTTP handlers: health, CRUD, search, graph, maintenance, threads, conversation, skills, snapshots, import/export.
//!
//! REVIEW-10: extracted from `routing.rs` — all request handlers that execute
//! SDK operations under `run_db_op` (pool + spawn_blocking).

use crate::audit::AuditEvent;
use crate::connection_pool::PoolError;
use crate::error::Result;
use crate::metrics;
use crate::sdk::{
    VantaEmbedded, VantaMemoryFilter, VantaMemoryInput, VantaMemoryListOptions,
    VantaMemoryListPage, VantaMemoryRecord, VantaMemorySearchHit, VantaMemorySearchRequest,
    VantaNamespaceStatsMap, VantaOperationalMetrics,
};
use crate::server::errors::{
    not_found_response, panic_error_response, pool_error_response, query_error_response,
    thread_not_found_response, vanta_error_response,
};
use crate::server::state::{NodeDTO, QueryRequest, QueryResponse, RequestId, ServerState};
use crate::VantaError;
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
#[tracing::instrument]
pub async fn health_check() -> Json<QueryResponse> {
    Json(QueryResponse {
        success: true,
        data: "OK".to_string(),
        node_id: None,
        nodes: None,
    })
}

#[tracing::instrument]
pub async fn metrics_endpoint() -> impl IntoResponse {
    let metrics_text = metrics::export_metrics_text();
    match Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(metrics_text)
    {
        Ok(resp) => resp.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to build metrics response: {e}"),
        )
            .into_response(),
    }
}

/// JSON wire shape for `GET /api/v2/metrics` (REST-02): the operational
/// snapshot (same `VantaOperationalMetrics` shape the desktop `vanta_metrics`
/// wrapper consumes) plus per-namespace collection counts for the
/// Índices/salud surface (FEAT-02). Both fields reuse existing SDK types.
#[derive(Serialize)]
struct MetricsV2Response {
    metrics: VantaOperationalMetrics,
    namespaces: VantaNamespaceStatsMap,
}

/// `GET /api/v2/metrics` — engine metrics as JSON for the web console.
///
/// Runs under the connection pool like every `/api/v2` console op and inherits
/// the same auth, rate-limit and CORS layers as the other protected routes.
#[tracing::instrument(skip(state))]
pub async fn metrics_v2(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| {
        Ok(MetricsV2Response {
            metrics: db.operational_metrics(),
            namespaces: db.namespace_stats(None)?,
        })
    })
    .await
    {
        Ok(resp) => Json(resp).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn execute_query(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<QueryRequest>,
) -> Response {
    use crate::executor::{ExecutionResult, Executor};

    let _permit = match state.pool.acquire().await {
        Ok(p) => p,
        Err(e) => {
            let msg = match e {
                PoolError::Closed => "Server query pool closed".to_string(),
                PoolError::Timeout => "Server concurrency limit reached; retry shortly".to_string(),
            };
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                [(header::RETRY_AFTER, "1")],
                Json(QueryResponse {
                    success: false,
                    data: msg,
                    node_id: None,
                    nodes: None,
                }),
            )
                .into_response();
        }
    };

    let storage = state.storage.clone();
    let query = payload.query.clone();

    let start = Instant::now();
    let join_res = tokio::task::spawn_blocking(move || {
        let executor = Executor::new(&storage);
        executor.execute_hybrid(&query)
    })
    .await;
    // FND-07: feed the canonical query latency histogram (vanta_query_latency_ms)
    // with real server-side execution time — no-op without the prometheus feature.
    metrics::record_query_latency(start.elapsed().as_millis() as u64);

    let execution_result = match join_res {
        Ok(r) => r,
        Err(e) => return panic_error_response(&e),
    };

    match execution_result {
        Ok(ExecutionResult::Read(nodes)) => {
            let dtos: Vec<NodeDTO> = nodes.iter().map(NodeDTO::from).collect();
            Json(QueryResponse {
                success: true,
                data: format!("Read {} nodes.", nodes.len()),
                node_id: None,
                nodes: Some(dtos),
            })
            .into_response()
        }
        Ok(ExecutionResult::Write {
            affected_nodes,
            message,
            node_id,
        }) => Json(QueryResponse {
            success: true,
            data: format!("Mutated {} nodes: {}", affected_nodes, message),
            node_id,
            nodes: None,
        })
        .into_response(),
        Ok(ExecutionResult::StaleContext(summary_id)) => Json(QueryResponse {
            success: true,
            data: format!(
                "STALE_CONTEXT: Confidence Score critical. Rehydration available for summary {}",
                summary_id
            ),
            node_id: Some(summary_id),
            nodes: None,
        })
        .into_response(),
        Err(e) => query_error_response(&e),
    }
}

// ─── /api/v2 console surface (WEB-01) ───────────────────────────────────────
//
// Endpoints map 1:1 to the embedded SDK (`VantaEmbedded`) so the wire format
// is the SDK's own serde. Errors are `{success: false, error}` with the status
// from `vanta_error_status` — the same shape the auth middleware and circuit
// breaker already emit. All engine work runs under a pool permit in
// `spawn_blocking` (never on the Tokio runtime, R-2 server-mcp).

/// Run a blocking SDK operation under a connection-pool permit.
///
/// Pool, panic, and `VantaError` failures become HTTP responses; success
/// returns the raw SDK value for the handler to serialize.
pub(crate) async fn run_db_op<T>(
    state: &ServerState,
    op: impl FnOnce(&VantaEmbedded) -> Result<T> + Send + 'static,
) -> std::result::Result<T, Response>
where
    T: Send + 'static,
{
    let _permit = state.pool.acquire().await.map_err(pool_error_response)?;
    let db = state.db.clone();
    match tokio::task::spawn_blocking(move || op(&db)).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(vanta_error_response(&e)),
        Err(e) => Err(panic_error_response(&e)),
    }
}

/// Health report shape for `GET /api/v2/health` (mirrors the desktop
/// `HealthReport` wire contract).
#[derive(Serialize)]
struct HealthReportV2 {
    status: &'static str,
    backend: String,
    latency_ms: u64,
    checked_at_ms: u64,
    message: Option<String>,
}

/// Human label for the configured storage backend.
fn backend_label(kind: &crate::backend::BackendKind) -> &'static str {
    match kind {
        crate::backend::BackendKind::Fjall => "fjall",
        crate::backend::BackendKind::RocksDb => "rocksdb",
        crate::backend::BackendKind::InMemory => "in-memory",
    }
}

#[tracing::instrument(skip(state))]
pub async fn health_v2(State(state): State<Arc<ServerState>>) -> Response {
    let start = Instant::now();
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || db.list_namespaces()).await;
    let latency_ms = start.elapsed().as_millis() as u64;
    let checked_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let (status, message) = match result {
        Ok(Ok(_)) => ("healthy", None),
        Ok(Err(e)) => ("degraded", Some(e.to_string())),
        Err(e) => ("degraded", Some(format!("execution task panicked: {e}"))),
    };
    Json(HealthReportV2 {
        status,
        backend: backend_label(&state.db.config.backend_kind).to_string(),
        latency_ms,
        checked_at_ms,
        message,
    })
    .into_response()
}

#[tracing::instrument(skip(state))]
pub async fn records_put(
    State(state): State<Arc<ServerState>>,
    Json(input): Json<VantaMemoryInput>,
) -> Response {
    match run_db_op(&state, move |db| db.put(input)).await {
        Ok(record) => (StatusCode::CREATED, Json(record)).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn records_put_batch(
    State(state): State<Arc<ServerState>>,
    Json(inputs): Json<Vec<VantaMemoryInput>>,
) -> Response {
    match run_db_op(&state, move |db| db.put_batch(inputs)).await {
        Ok(records) => (StatusCode::CREATED, Json(records)).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn records_get(
    State(state): State<Arc<ServerState>>,
    AxumPath((ns, key)): AxumPath<(String, String)>,
) -> Response {
    let key_label = key.clone();
    match run_db_op(&state, move |db| db.get(&ns, &key)).await {
        Ok(Some(record)) => Json(record).into_response(),
        Ok(None) => not_found_response(&key_label),
        Err(resp) => resp,
    }
}

/// Query params for `GET /api/v2/records/{ns}/{key}/versions`.
#[derive(Deserialize, Debug)]
pub struct RecordsVersionsParams {
    /// When present, returns only that version instead of the full list.
    version: Option<u64>,
}

#[tracing::instrument(skip(state))]
pub async fn records_versions(
    State(state): State<Arc<ServerState>>,
    AxumPath((ns, key)): AxumPath<(String, String)>,
    Query(params): Query<RecordsVersionsParams>,
) -> Response {
    match params.version {
        Some(version) => {
            let key_label = key.clone();
            match run_db_op(&state, move |db| db.get_version(&ns, &key, version)).await {
                Ok(Some(record)) => Json(record).into_response(),
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("version {version} not found for key {key_label}"),
                    })),
                )
                    .into_response(),
                Err(resp) => resp,
            }
        }
        None => match run_db_op(&state, move |db| db.versions(&ns, &key)).await {
            Ok(records) => Json(records).into_response(),
            Err(resp) => resp,
        },
    }
}

#[tracing::instrument(skip(state))]
pub async fn records_delete(
    State(state): State<Arc<ServerState>>,
    AxumPath((ns, key)): AxumPath<(String, String)>,
) -> Response {
    let key_label = key.clone();
    match run_db_op(&state, move |db| db.delete(&ns, &key)).await {
        Ok(true) => Json(serde_json::json!({ "deleted": true })).into_response(),
        Ok(false) => not_found_response(&key_label),
        Err(resp) => resp,
    }
}

/// Query params for `DELETE /api/v2/records?namespace=&filter=`.
#[derive(Deserialize, Debug)]
pub struct DeleteByFilterParams {
    namespace: String,
    /// JSON array of `VantaMemoryFilterItem` (e.g.
    /// `[{"field":"kind","op":"Eq","value":{"String":"note"}}]`).
    filter: String,
}

#[tracing::instrument(skip(state))]
pub async fn records_delete_by_filter(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<DeleteByFilterParams>,
) -> Response {
    let filter: VantaMemoryFilter = match serde_json::from_str(&params.filter) {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("invalid filter JSON: {e}"),
                })),
            )
                .into_response();
        }
    };
    let ns = params.namespace;
    match run_db_op(&state, move |db| db.delete_by_filter(&ns, filter)).await {
        Ok(deleted) => Json(serde_json::json!({ "deleted": deleted })).into_response(),
        Err(resp) => resp,
    }
}

/// AUD-046: merge per-namespace pages (stable namespace-name order) for the
/// `/api/v2/list` all-namespaces fan-out. A namespace whose page still has a
/// `next_cursor` was capped at `NS_CAP` mid-listing — it is reported in the
/// returned `truncated_namespaces` so the client never sees silent truncation.
fn merge_all_namespaces_pages(
    pages: Vec<(String, VantaMemoryListPage)>,
) -> (Vec<VantaMemoryRecord>, Vec<String>) {
    let mut records = Vec::new();
    let mut truncated_namespaces = Vec::new();
    for (ns, page) in pages {
        if page.next_cursor.is_some() {
            truncated_namespaces.push(ns);
        }
        records.extend(page.records);
    }
    (records, truncated_namespaces)
}

/// Query params for `GET /api/v2/list`.
#[derive(Deserialize, Debug)]
pub struct ListParams {
    // Option: la consola web lista sin namespace → default a "default" (igual
    // que el bridge nativo). Un campo String requerido 400ea en axum antes del handler.
    namespace: Option<String>,
    limit: Option<usize>,
    cursor: Option<usize>,
    /// JSON array of `VantaMemoryFilterItem`.
    filter_ops: Option<String>,
}

#[tracing::instrument(skip(state))]
pub async fn records_list(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<ListParams>,
) -> Response {
    // Sin namespace → agregar TODOS los namespaces (orden estable: nombre asc).
    // Antes defaulteaba a "default" y el grid/paleta de la consola mostraba
    // "Sin registros" con datos presentes en otros namespaces.
    let all_namespaces = params
        .namespace
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .is_none();
    let filter_ops = match params.filter_ops.as_deref() {
        None => None,
        Some(raw) => match serde_json::from_str::<VantaMemoryFilter>(raw) {
            Ok(f) => Some(f),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("invalid filter_ops JSON: {e}"),
                    })),
                )
                    .into_response();
            }
        },
    };
    let limit = params.limit.unwrap_or(100);
    let cursor = params.cursor;
    if all_namespaces {
        // FIND-24: fan-out by namespace now respects the client's `limit`
        // (instead of NS_CAP) — list(limit=100) used to materialize
        // NS_CAP=10_000 records per namespace and slice in memory, blowing
        // past REQUEST_TIMEOUT=30s for ≥10k records total. The SDK's
        // `indexed_ids_by_namespace` early-exits at `limit`, so a per-ns
        // `limit`-sized scan is O(limit) per namespace. Cross-namespace
        // pagination walks namespaces in stable name order; the returned
        // `next_cursor` is the cumulative offset within the merged window
        // and remains backward-compatible with single-namespace clients.
        //
        // NS_CAP (10_000) was the previous fan-out ceiling per ns and has been
        // removed: the SDK now enforces the `limit` early-exit natively, so
        // there is no in-memory NS_CAP cost. Truncation at the namespace
        // boundary is still detected via `next_cursor` from each per-ns
        // `VantaMemoryListPage` (see `merge_all_namespaces_pages`).

        /// Fan-out response: same shape as `VantaMemoryListPage` plus an
        /// additive signal listing namespaces whose listing is still paginating
        /// (they may hold more records than this response contains).
        #[derive(Serialize)]
        struct AllNamespacesListPage {
            records: Vec<VantaMemoryRecord>,
            next_cursor: Option<usize>,
            /// Namespaces still paginating during the fan-out (their
            /// per-ns `VantaMemoryListPage.next_cursor` was `Some`).
            truncated_namespaces: Vec<String>,
        }

        let options_for = move |_ns: String| VantaMemoryListOptions {
            filter_ops: filter_ops.clone(),
            limit,
            cursor,
            ..Default::default()
        };
        return match run_db_op(&state, move |db| {
            let mut names: Vec<String> = db.namespace_stats(None)?.keys().cloned().collect();
            names.sort();
            let mut pages = Vec::new();
            for ns in names {
                let page = db.list(&ns, options_for(ns.clone()))?;
                pages.push((ns, page));
            }
            let (records, truncated_namespaces) = merge_all_namespaces_pages(pages);
            let start = cursor.unwrap_or(0).min(records.len());
            let end = (start + limit).min(records.len());
            let window = records[start..end].to_vec();
            let next_cursor = (end < records.len()).then_some(end);
            Ok::<_, VantaError>(AllNamespacesListPage {
                records: window,
                next_cursor,
                truncated_namespaces,
            })
        })
        .await
        {
            Ok(page) => Json(page).into_response(),
            Err(resp) => resp,
        };
    }
    let ns = params.namespace.unwrap_or_default();
    let options = VantaMemoryListOptions {
        filter_ops,
        limit,
        cursor,
        ..Default::default()
    };
    match run_db_op(&state, move |db| db.list(&ns, options)).await {
        Ok(page) => Json(page).into_response(),
        Err(resp) => resp,
    }
}

/// JSON body for `POST /api/v2/search`: the SDK search request plus optional
/// offset pagination (REST-04). `cursor`/`limit` are server-only — the core
/// `search()` is a top_k window without its own cursor, so the wire pages by
/// offset over the same score-ranked result set.
#[derive(Debug, Deserialize)]
pub struct SearchPageRequest {
    #[serde(flatten)]
    request: VantaMemorySearchRequest,
    /// Zero-based offset into the ranked result set.
    #[serde(default)]
    cursor: Option<usize>,
    /// Page size; defaults to `top_k`.
    #[serde(default)]
    limit: Option<usize>,
}

/// Page-shaped search response mirroring `VantaMemoryListPage` so the web
/// console paginates search the same way it paginates list (REST-04).
#[derive(Serialize)]
struct SearchPageV2 {
    records: Vec<VantaMemorySearchHit>,
    next_cursor: Option<usize>,
}

#[tracing::instrument(skip(state))]
pub async fn records_search(
    State(state): State<Arc<ServerState>>,
    Json(page_request): Json<SearchPageRequest>,
) -> Response {
    // La Topbar de la consola web busca sin namespace → search_all (todos los
    // namespaces, merge por score). Antes defaulteaba a "default" y la búsqueda
    // global ignoraba silenciosamente todo lo ingerido en otros namespaces.
    let mut request = page_request.request;
    let all_namespaces = request.namespace.trim().is_empty();
    // Paginación offset (REST-04): el core `search()` es una ventana top_k sin
    // cursor propio, así que el server traduce cursor/limit → top_k+1 (un extra
    // para saber si hay más página) y recorta. Los resultados se ordenan por
    // score, así que offset sobre el mismo ranking es estable entre páginas.
    let page_size = page_request.limit.unwrap_or(request.top_k.max(1));
    let cursor = page_request.cursor.unwrap_or(0);
    request.top_k = cursor.saturating_add(page_size).saturating_add(1);
    match run_db_op(&state, move |db| {
        if all_namespaces {
            db.search_all(request)
        } else {
            db.search(request)
        }
    })
    .await
    {
        Ok(hits) => {
            let start = cursor.min(hits.len());
            let end = (start + page_size).min(hits.len());
            let records = hits[start..end].to_vec();
            let next_cursor = (end < hits.len()).then_some(end);
            Json(SearchPageV2 {
                records,
                next_cursor,
            })
            .into_response()
        }
        Err(resp) => resp,
    }
}

/// Query params for `GET /api/v2/autocomplete`.
#[derive(Deserialize, Debug)]
pub struct AutocompleteParams {
    prefix: Option<String>,
}

#[tracing::instrument]
pub async fn iql_autocomplete(Query(params): Query<AutocompleteParams>) -> Json<Vec<String>> {
    let prefix = params.prefix.unwrap_or_default();
    Json(crate::parser::autocomplete_prefix(&prefix))
}

/// Query params for `GET /api/v2/audit`.
#[derive(Deserialize, Debug)]
pub struct AuditParams {
    namespace: Option<String>,
    op: Option<String>,
    outcome: Option<String>,
    limit: Option<usize>,
    cursor: Option<usize>,
}

/// Default page size when the caller omits `limit` (mirrors the desktop).
const AUDIT_DEFAULT_LIMIT: usize = 100;

/// A page of audit events ordered newest-first (mirrors the desktop `AuditPage`).
#[derive(Serialize)]
struct AuditPageV2 {
    events: Vec<AuditEvent>,
    next_cursor: Option<usize>,
}

/// Resolve the audit log path from the embedded config.
///
/// `None` means audit is not configured — the endpoint reports 404 rather than
/// inventing a path that would never be written (mirrors the desktop, which
/// errors with "audit log no configurado").
fn audit_log_path(state: &ServerState) -> Option<std::path::PathBuf> {
    state.db.config.audit_log_path.clone()
}

/// Read the audit JSONL at `path`, apply filters, and paginate newest-first.
///
/// `cursor` is a zero-based offset into the *filtered* newest-first list;
/// `next_cursor` is `Some(end)` when older events remain, `None` otherwise.
///
/// ponytail: whole-file read (fine for console-sized audit logs); a byte-offset
/// tail read is the upgrade if the log grows large.
fn read_audit_page(
    path: &std::path::Path,
    namespace: Option<&str>,
    op: Option<&str>,
    outcome: Option<&str>,
    limit: usize,
    cursor: Option<usize>,
) -> std::io::Result<AuditPageV2> {
    let content = std::fs::read_to_string(path)?;
    let mut matched: Vec<AuditEvent> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<AuditEvent>(line).ok())
        .filter(|e| namespace.is_none_or(|n| e.namespace == n))
        .filter(|e| op.is_none_or(|o| e.op == o))
        .filter(|e| outcome.is_none_or(|o| e.outcome == o))
        .collect();
    matched.reverse();
    let start = cursor.unwrap_or(0).min(matched.len());
    let end = (start + limit).min(matched.len());
    let events = matched[start..end].to_vec();
    let next_cursor = (end < matched.len()).then_some(end);
    Ok(AuditPageV2 {
        events,
        next_cursor,
    })
}

#[tracing::instrument(skip(state))]
pub async fn audit_events(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<AuditParams>,
) -> Response {
    let Some(path) = audit_log_path(&state) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "audit log no configurado",
            })),
        )
            .into_response();
    };
    let namespace = params.namespace;
    let op = params.op;
    let outcome = params.outcome;
    let limit = params.limit.unwrap_or(AUDIT_DEFAULT_LIMIT);
    let cursor = params.cursor;

    let join = tokio::task::spawn_blocking(move || {
        read_audit_page(
            &path,
            namespace.as_deref(),
            op.as_deref(),
            outcome.as_deref(),
            limit,
            cursor,
        )
    })
    .await;

    match join {
        Ok(Ok(page)) => Json(page).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": format!("failed to read audit log: {e}"),
            })),
        )
            .into_response(),
        Err(e) => panic_error_response(&e),
    }
}

/// 404 hint returned when no `--dashboard-dir` is configured (WEB-03).
pub async fn dashboard_disabled() -> Response {
    (
        StatusCode::NOT_FOUND,
        "Dashboard not enabled. Start the server with --dashboard-dir <path> to serve the Vanta Studio console at /dashboard.",
    )
        .into_response()
}

// ─── /api/v2 extended SDK surface (WEB-02) ─────────────────────────────────
//
// Second slice of the console API: export/import, graph traversal + GDS,
// maintenance, threads, and snapshots. Same rules as WEB-01: the wire format
// is the SDK's own serde, errors are `{success: false, error}` with the
// status from `vanta_error_status`, and all engine work runs under a pool
// permit in `spawn_blocking` via `run_db_op`.

/// Body for `POST /api/v2/export`.
#[derive(Deserialize, Debug)]
pub struct ExportRequest {
    /// Target path for the export file (JSONL).
    path: String,
    /// When present, exports only this namespace; otherwise exports all.
    namespace: Option<String>,
    /// Optional AND-combined filter applied to the exported records.
    filter: Option<VantaMemoryFilter>,
}

/// Body for `POST /api/v2/import`.
#[derive(Deserialize, Debug)]
pub struct ImportRequest {
    /// Inline records to import (export wire format). Mutually exclusive with `path`.
    records: Option<Vec<VantaMemoryRecord>>,
    /// Path to a JSONL export (default) or a `.vdbdump` bulk file (`format: "bulk"`).
    path: Option<String>,
    /// File format when `path` is set: `"jsonl"` (default) or `"bulk"`.
    format: Option<String>,
}

#[tracing::instrument(skip(state))]
pub async fn export_v2(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<ExportRequest>,
) -> Response {
    let namespace = req.namespace.clone();
    let filter = req.filter.clone();
    match run_db_op(&state, move |db| match namespace.as_deref() {
        Some(ns) => db.export_namespace(&req.path, ns, filter),
        None => db.export_all(&req.path),
    })
    .await
    {
        Ok(report) => Json(report).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn import_v2(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<ImportRequest>,
) -> Response {
    let records = req.records.clone();
    let path = req.path.clone();
    let format = req.format.clone();
    // The three import ops return two report types (VantaImportReport vs
    // BulkImportReport); normalize to a JSON value to keep one response path.
    match run_db_op(&state, move |db| -> Result<serde_json::Value> {
        let value = if let Some(records) = records {
            serde_json::to_value(db.import_records(records)?).map_err(VantaError::serialization)?
        } else if let Some(path) = path {
            if format.as_deref() == Some("bulk") {
                serde_json::to_value(db.bulk_import_file(&path)?)
                    .map_err(VantaError::serialization)?
            } else {
                serde_json::to_value(db.import_file(&path)?).map_err(VantaError::serialization)?
            }
        } else {
            return Err(VantaError::InvalidInput(
                "import requires `records` or `path`".into(),
            ));
        };
        Ok(value)
    })
    .await
    {
        Ok(report) => Json(report).into_response(),
        Err(resp) => resp,
    }
}

/// Direction wire enum — `TraversalDirection` (src/graph.rs) is not serde.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum GraphDirection {
    Forward,
    Reverse,
    Both,
}

impl From<GraphDirection> for crate::graph::TraversalDirection {
    fn from(d: GraphDirection) -> Self {
        match d {
            GraphDirection::Forward => crate::graph::TraversalDirection::Forward,
            GraphDirection::Reverse => crate::graph::TraversalDirection::Reverse,
            GraphDirection::Both => crate::graph::TraversalDirection::Both,
        }
    }
}

/// Body for `POST /api/v2/graph/bfs` and `/dfs`.
#[derive(Deserialize, Debug)]
pub struct GraphTraversalRequest {
    /// Node ids to start from.
    roots: Vec<u128>,
    /// Maximum hop depth from the roots.
    max_depth: usize,
    /// Edge direction: `"forward"` (default), `"reverse"`, or `"both"`.
    direction: Option<GraphDirection>,
}

/// Body for `POST /api/v2/graph/degree` and `/centrality`.
#[derive(Deserialize, Debug)]
pub struct GraphRootsRequest {
    /// Node ids to score.
    roots: Vec<u128>,
}

fn default_pagerank_iterations() -> usize {
    100
}
fn default_pagerank_damping() -> f64 {
    0.85
}
fn default_pagerank_tolerance() -> f64 {
    1e-6
}

/// Body for `POST /api/v2/graph/pagerank`.
#[derive(Deserialize, Debug)]
pub struct GraphPageRankRequest {
    /// Node ids to score.
    roots: Vec<u128>,
    #[serde(default = "default_pagerank_iterations")]
    max_iterations: usize,
    #[serde(default = "default_pagerank_damping")]
    damping: f64,
    #[serde(default = "default_pagerank_tolerance")]
    tolerance: f64,
}

#[tracing::instrument(skip(state))]
pub async fn graph_bfs(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphTraversalRequest>,
) -> Response {
    let roots = req.roots.clone();
    let max_depth = req.max_depth;
    let direction = req.direction.unwrap_or(GraphDirection::Forward).into();
    match run_db_op(&state, move |db| db.graph_bfs(&roots, max_depth, direction)).await {
        Ok(ids) => Json(ids).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn graph_dfs(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphTraversalRequest>,
) -> Response {
    let roots = req.roots.clone();
    let max_depth = req.max_depth;
    let direction = req.direction.unwrap_or(GraphDirection::Forward).into();
    match run_db_op(&state, move |db| db.graph_dfs(&roots, max_depth, direction)).await {
        Ok(ids) => Json(ids).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn graph_degree(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphRootsRequest>,
) -> Response {
    let roots = req.roots.clone();
    match run_db_op(&state, move |db| db.graph_degree_centrality(&roots)).await {
        Ok(scores) => Json(scores).into_response(),
        Err(resp) => resp,
    }
}

/// The GDS module exposes a single centrality op (`degree_centrality`), so
/// `/graph/centrality` maps to the same SDK call as `/graph/degree`.
#[tracing::instrument(skip(state))]
pub async fn graph_centrality(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphRootsRequest>,
) -> Response {
    let roots = req.roots.clone();
    match run_db_op(&state, move |db| db.graph_degree_centrality(&roots)).await {
        Ok(scores) => Json(scores).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn graph_pagerank(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphPageRankRequest>,
) -> Response {
    let roots = req.roots.clone();
    let max_iterations = req.max_iterations;
    let damping = req.damping;
    let tolerance = req.tolerance;
    match run_db_op(&state, move |db| {
        db.graph_page_rank(&roots, max_iterations, damping, tolerance)
    })
    .await
    {
        Ok(scores) => Json(scores).into_response(),
        Err(resp) => resp,
    }
}

// --- Graph v2 (REST-03): desktop-DTO wire with u128-safe string ids --------

/// Wire node — mirror of the desktop `VantaGraphNodeInfo`
/// (desktop/src-tauri/src/connections/types.rs).
#[derive(Serialize, Debug)]
struct GraphNodeDTO {
    /// Numeric node id, serialized as a string (u128 on the core side).
    id: String,
    /// Display label (content/text/__vanta_payload field, id fallback).
    label: String,
    /// Grouping key for coloring (namespace or node type), when known.
    group: Option<String>,
    /// In+out degree centrality (0 when not computed).
    degree: u64,
}

/// Wire edge — mirror of the desktop `VantaGraphEdgeInfo`.
#[derive(Serialize, Debug)]
struct GraphEdgeDTO {
    /// Source node id (string — u128 on the core side).
    source: String,
    /// Target node id (string — u128 on the core side).
    target: String,
    /// Edge label, when the backend exposes one.
    label: Option<String>,
    /// Edge weight, when the backend exposes one.
    weight: Option<f32>,
}

/// Wire traversal result — mirror of the desktop `VantaGraphTraversalResult`.
#[derive(Serialize, Debug, Default)]
struct GraphTraversalDTO {
    nodes: Vec<GraphNodeDTO>,
    edges: Vec<GraphEdgeDTO>,
}

/// Body for `POST /api/v2/graph/v2/bfs` and `/dfs`. Roots are decimal strings
/// so ids above u64::MAX survive the JSON wire (the legacy `/api/v2/graph/*`
/// endpoints take bare u128 numbers, which the browser cannot parse).
#[derive(Deserialize, Debug)]
pub struct GraphV2TraversalRequest {
    /// Node ids to start from (decimal u128 strings).
    roots: Vec<String>,
    /// Maximum hop depth from the roots.
    max_depth: usize,
    /// Edge direction: `"forward"` (default), `"reverse"`, or `"both"`.
    direction: Option<GraphDirection>,
    /// Cap on the returned node count (default 50).
    limit: Option<usize>,
}

/// Body for `POST /api/v2/graph/v2/degree`.
#[derive(Deserialize, Debug)]
pub struct GraphV2DegreeRequest {
    /// Namespace whose records are scored.
    namespace: String,
    /// Cap on the returned node count (default 50).
    limit: Option<usize>,
}

/// Parse a wire node-id string into the core's u128 id (native.rs
/// `parse_node_id`).
fn parse_node_id_str(id: &str) -> Result<u128> {
    id.parse::<u128>().map_err(|_| {
        VantaError::InvalidInput(format!(
            "invalid node id '{id}': expected a decimal u128 string"
        ))
    })
}

/// Label/group extraction mirror of native.rs `node_record_to_graph_node`.
fn node_record_to_graph_dto(n: &crate::sdk::VantaNodeRecord) -> GraphNodeDTO {
    let label = ["__vanta_payload", "text", "content"]
        .into_iter()
        .find_map(|k| match n.fields.get(k) {
            Some(crate::sdk::VantaValue::String(s)) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| n.id.to_string());
    let group = match n.fields.get("type") {
        Some(crate::sdk::VantaValue::String(s)) => Some(s.clone()),
        _ => None,
    };
    GraphNodeDTO {
        id: n.id.to_string(),
        label,
        group,
        degree: 0,
    }
}

/// Build the wire traversal result from visited node ids, mirror of native.rs
/// `graph_traversal_result`: capped at `cap` nodes; each node's outgoing edges
/// become the edge list (source = node, target = edge target).
fn graph_traversal_dto(db: &VantaEmbedded, ids: &[u128], cap: usize) -> Result<GraphTraversalDTO> {
    let mut result = GraphTraversalDTO::default();
    for id in ids.iter().take(cap) {
        if let Some(node) = db.get_node(*id)? {
            result.nodes.push(node_record_to_graph_dto(&node));
            for edge in &node.edges {
                result.edges.push(GraphEdgeDTO {
                    source: id.to_string(),
                    target: edge.target.to_string(),
                    label: Some(edge.label.clone()),
                    weight: Some(edge.weight),
                });
            }
        }
    }
    Ok(result)
}

/// POST `/api/v2/graph/v2/bfs` — desktop `VantaGraphTraversalResult` wire with
/// u128-safe string ids (REST-03).
#[tracing::instrument(skip(state))]
pub async fn graph_v2_bfs(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphV2TraversalRequest>,
) -> Response {
    let roots = match req
        .roots
        .iter()
        .map(|r| parse_node_id_str(r))
        .collect::<Result<Vec<u128>>>()
    {
        Ok(roots) => roots,
        Err(e) => return vanta_error_response(&e),
    };
    let max_depth = req.max_depth;
    let direction = req.direction.unwrap_or(GraphDirection::Forward).into();
    let cap = req.limit.unwrap_or(50);
    match run_db_op(&state, move |db| {
        let ids = db.graph_bfs(&roots, max_depth, direction)?;
        graph_traversal_dto(db, &ids, cap)
    })
    .await
    {
        Ok(dto) => Json(dto).into_response(),
        Err(resp) => resp,
    }
}

/// POST `/api/v2/graph/v2/dfs` — desktop `VantaGraphTraversalResult` wire with
/// u128-safe string ids (REST-03).
#[tracing::instrument(skip(state))]
pub async fn graph_v2_dfs(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphV2TraversalRequest>,
) -> Response {
    let roots = match req
        .roots
        .iter()
        .map(|r| parse_node_id_str(r))
        .collect::<Result<Vec<u128>>>()
    {
        Ok(roots) => roots,
        Err(e) => return vanta_error_response(&e),
    };
    let max_depth = req.max_depth;
    let direction = req.direction.unwrap_or(GraphDirection::Forward).into();
    let cap = req.limit.unwrap_or(50);
    match run_db_op(&state, move |db| {
        let ids = db.graph_dfs(&roots, max_depth, direction)?;
        graph_traversal_dto(db, &ids, cap)
    })
    .await
    {
        Ok(dto) => Json(dto).into_response(),
        Err(resp) => resp,
    }
}

/// POST `/api/v2/graph/v2/degree` — desktop `VantaGraphNodeInfo[]` wire with
/// u128-safe string ids (REST-03). Mirrors native.rs `graph_degree`; an
/// empty/unknown namespace resolves to an empty array, not an error (GRAFO-01).
#[tracing::instrument(skip(state))]
pub async fn graph_v2_degree(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphV2DegreeRequest>,
) -> Response {
    let ns = req.namespace;
    let cap = req.limit.unwrap_or(50);
    match run_db_op(&state, move |db| {
        let options = VantaMemoryListOptions {
            limit: cap,
            cursor: None,
            ..Default::default()
        };
        let page = db.list(&ns, options)?;
        if page.records.is_empty() {
            return Ok(Vec::new());
        }
        let node_ids: Vec<u128> = page.records.iter().map(|r| r.node_id).collect();
        let degrees = db.graph_degree_centrality(&node_ids)?;
        Ok(page
            .records
            .into_iter()
            .map(|r| GraphNodeDTO {
                id: r.node_id.to_string(),
                label: r.payload.clone(),
                group: Some(ns.clone()),
                degree: degrees
                    .get(&r.node_id)
                    .map(|(in_d, out_d)| (*in_d + *out_d) as u64)
                    .unwrap_or(0),
            })
            .collect())
    })
    .await
    {
        Ok(nodes) => Json(nodes).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn maintenance_purge(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| db.purge_expired()).await {
        Ok(purged) => Json(serde_json::json!({ "purged": purged })).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn maintenance_compact(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| db.compact_layout()).await {
        Ok(freed_bytes) => Json(serde_json::json!({ "freed_bytes": freed_bytes })).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn maintenance_flush(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| db.flush()).await {
        Ok(()) => Json(serde_json::json!({ "flushed": true })).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn maintenance_rebuild_index(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| db.rebuild_index()).await {
        Ok(report) => Json(report).into_response(),
        Err(resp) => resp,
    }
}

/// Query params for `GET /api/v2/threads`.
#[derive(Deserialize, Debug)]
pub struct ThreadsListParams {
    /// Maximum number of threads to return.
    #[serde(default = "default_threads_limit")]
    limit: usize,
    /// Offset into the thread list.
    #[serde(default)]
    offset: usize,
}

fn default_threads_limit() -> usize {
    100
}

/// Body for `POST /api/v2/threads`.
#[derive(Deserialize, Debug)]
pub struct ThreadCreateRequest {
    /// Human-readable thread title.
    title: String,
    /// Optional time-to-live in seconds for the thread.
    ttl_secs: Option<u64>,
}

/// Body for `POST /api/v2/threads/{id}` (send a message).
#[derive(Deserialize, Debug)]
pub struct ThreadMessageRequest {
    /// Message role (`user`, `assistant`, ...).
    role: String,
    /// Message content.
    content: String,
}

/// Wire view of a thread — `MessageThread.thread_id` is a bare `u128` that
/// serde cannot emit as a JSON number (out of u64 range), so it travels as a
/// string, consistent with `u128_serde` elsewhere in the SDK wire format.
#[derive(Serialize)]
struct ThreadDTO {
    thread_id: String,
    title: String,
    messages: Vec<crate::agentic::Message>,
    created_at: u64,
    updated_at: u64,
    metadata: std::collections::HashMap<String, String>,
}

impl From<crate::agentic::MessageThread> for ThreadDTO {
    fn from(t: crate::agentic::MessageThread) -> Self {
        Self {
            thread_id: t.thread_id.to_string(),
            title: t.title,
            messages: t.messages,
            created_at: t.created_at,
            updated_at: t.updated_at,
            metadata: t.metadata,
        }
    }
}

#[tracing::instrument(skip(state))]
pub async fn threads_list(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<ThreadsListParams>,
) -> Response {
    let limit = params.limit;
    let offset = params.offset;
    match run_db_op(&state, move |db| db.list_threads(limit, offset)).await {
        Ok(threads) => {
            let dtos: Vec<ThreadDTO> = threads.into_iter().map(ThreadDTO::from).collect();
            Json(dtos).into_response()
        }
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn threads_create(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<ThreadCreateRequest>,
) -> Response {
    let title = req.title.clone();
    let ttl_secs = req.ttl_secs;
    match run_db_op(&state, move |db| db.create_thread(&title, ttl_secs)).await {
        Ok(thread_id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "thread_id": thread_id.to_string() })),
        )
            .into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn threads_get(
    State(state): State<Arc<ServerState>>,
    AxumPath(thread_id): AxumPath<u128>,
) -> Response {
    match run_db_op(&state, move |db| db.get_thread(thread_id)).await {
        Ok(Some(thread)) => Json(ThreadDTO::from(thread)).into_response(),
        Ok(None) => thread_not_found_response(thread_id),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn threads_send_message(
    State(state): State<Arc<ServerState>>,
    AxumPath(thread_id): AxumPath<u128>,
    Json(req): Json<ThreadMessageRequest>,
) -> Response {
    let role = req.role.clone();
    let content = req.content.clone();
    match run_db_op(&state, move |db| {
        db.send_message(thread_id, &role, &content)
    })
    .await
    {
        Ok(()) => Json(serde_json::json!({ "sent": true })).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn threads_delete(
    State(state): State<Arc<ServerState>>,
    AxumPath(thread_id): AxumPath<u128>,
) -> Response {
    match run_db_op(&state, move |db| db.delete_thread(thread_id)).await {
        Ok(()) => Json(serde_json::json!({ "deleted": true })).into_response(),
        Err(resp) => resp,
    }
}

/// Body for `POST /conversation/add` (F3 data plane): record one message in a
/// conversation. When `thread_id` is absent, a new thread is created first —
/// the agent does not need to pre-create a thread to accumulate context.
#[derive(Deserialize, Debug)]
pub struct ConversationAddRequest {
    /// Existing thread id (u128 as decimal string). When absent, a thread is
    /// created with `title` (defaults to `"conversation"`) and `ttl_secs`.
    thread_id: Option<String>,
    /// Human-readable thread title, used only when creating a new thread.
    title: Option<String>,
    /// Message role (`user`, `assistant`, ...).
    role: String,
    /// Message content.
    content: String,
    /// Optional time-to-live in seconds for a newly created thread.
    ttl_secs: Option<u64>,
}

#[tracing::instrument(skip(state))]
pub async fn conversation_add(
    State(state): State<Arc<ServerState>>,
    request_id: RequestId,
    Json(req): Json<ConversationAddRequest>,
) -> Response {
    let thread_id = match req.thread_id.as_deref() {
        Some(raw) => match raw.parse::<u128>() {
            Ok(id) => Some(id),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("invalid thread_id: {raw:?}"),
                    })),
                )
                    .into_response();
            }
        },
        None => None,
    };
    let title = req
        .title
        .clone()
        .unwrap_or_else(|| "conversation".to_string());
    let ttl_secs = req.ttl_secs;
    let role = req.role.clone();
    let content = req.content.clone();
    // SRV-02: carry the caller's tracing id into the audit event.
    let rid = request_id.0;

    match run_db_op(&state, move |db| {
        let id = match thread_id {
            Some(id) => id,
            None => db.create_thread(&title, ttl_secs)?,
        };
        db.send_message(id, &role, &content)?;
        db.audit(
            AuditEvent::memory("conversation", "threads", &id.to_string(), "ok", None)
                .with_request_id_opt(rid),
        );
        Ok(id)
    })
    .await
    {
        Ok(id) => {
            // MEM-55: fire the memory-pipeline trigger best-effort. Any error
            // is logged and swallowed — the HTTP response reflects only the
            // thread save (P4: extraction failures never fail the request).
            if let Some(trigger) = &state.conversation_trigger {
                if let Err(err) = trigger.trigger(id, &req.role, &req.content) {
                    tracing::warn!(thread = %id, %err, "conversation trigger failed; ignoring");
                }
            }
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "success": true, "thread_id": id.to_string() })),
            )
                .into_response()
        }
        Err(resp) => resp,
    }
}

/// Query params for `GET /skill/listing` (F3 data plane): head rows of the
/// skill store with optional filters — enough for prompt-injection use cases.
#[derive(Deserialize, Debug)]
pub struct SkillListingParams {
    /// Only list skills owned by this agent.
    owner_agent: Option<String>,
    /// Only list skills whose name starts with this prefix.
    name_prefix: Option<String>,
    /// Maximum number of items to return (default 50, capped at 200).
    limit: Option<usize>,
    /// Number of items to skip.
    offset: Option<usize>,
}

/// Lean wire view of a skill head row — skill metadata without the content
/// body (the listing is for prompt injection, not for dumping full skills).
#[derive(Serialize)]
struct SkillListingItem {
    skill_id: String,
    version: u64,
    name: String,
    owner_agent: String,
    description: String,
}

#[tracing::instrument(skip(state))]
pub async fn skill_listing(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<SkillListingParams>,
) -> Response {
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);
    match run_db_op(&state, move |db| {
        let engine = db.engine_handle()?;
        let store = crate::skills::SkillStore::new(&engine);
        store.list(crate::sdk::SkillListOptions {
            owner_agent: params.owner_agent,
            name_prefix: params.name_prefix,
            limit,
            offset,
        })
    })
    .await
    {
        Ok(page) => {
            let items: Vec<SkillListingItem> = page
                .items
                .into_iter()
                .map(|r| SkillListingItem {
                    skill_id: r.skill_id,
                    version: r.version,
                    name: r.name,
                    owner_agent: r.owner_agent,
                    description: r.description,
                })
                .collect();
            Json(serde_json::json!({ "items": items, "total": page.total })).into_response()
        }
        Err(resp) => resp,
    }
}

/// Query params for the mutating skill endpoints (PUT/PATCH/DELETE).
///
/// `expected_version` is the optimistic lock (MEM-06 pattern): a stale value
/// surfaces as 409 via `VantaError::ExecutionConflict`. `owner_agent` is
/// checked against the head's owner — a mismatch returns the SAME 404 as a
/// missing skill (no existence oracle for other agents' skills).
#[derive(Deserialize, Debug)]
pub struct SkillMutationParams {
    owner_agent: String,
    expected_version: u64,
}

/// Resolve a skill head enforcing ownership. Missing skill and foreign-owned
/// skill are indistinguishable on the wire (both `NotFound` → 404).
fn require_owned_head(
    store: &crate::skills::SkillStore<'_>,
    skill_id: &str,
    owner_agent: &str,
) -> crate::error::Result<crate::sdk::SkillRecord> {
    match store.get_head(skill_id)? {
        Some(head) if head.owner_agent == owner_agent => Ok(head),
        _ => Err(VantaError::NotFound {
            kind: "skill".into(),
            id: skill_id.into(),
        }),
    }
}

/// `POST /api/v2/skills` — create a skill (version 1). Idempotent when the
/// same `(owner_agent, name)` + content already exists (`idempotent: true`).
#[tracing::instrument(skip(state))]
pub async fn skill_create(
    State(state): State<Arc<ServerState>>,
    Json(input): Json<crate::sdk::SkillCreateInput>,
) -> Response {
    match run_db_op(&state, move |db| {
        let engine = db.engine_handle()?;
        crate::skills::SkillStore::new(&engine).create(input)
    })
    .await
    {
        Ok(result) => (StatusCode::CREATED, Json(result)).into_response(),
        Err(resp) => resp,
    }
}

/// `PUT /api/v2/skills/{skill_id}?owner_agent=…&expected_version=…` — full
/// update of description+content, appending a new version.
#[tracing::instrument(skip(state))]
pub async fn skill_update(
    State(state): State<Arc<ServerState>>,
    AxumPath(skill_id): AxumPath<String>,
    Query(params): Query<SkillMutationParams>,
    Json(input): Json<crate::sdk::SkillUpdateInput>,
) -> Response {
    match run_db_op(&state, move |db| {
        let engine = db.engine_handle()?;
        let store = crate::skills::SkillStore::new(&engine);
        require_owned_head(&store, &skill_id, &params.owner_agent)?;
        store.update(&skill_id, params.expected_version, input)
    })
    .await
    {
        Ok(result) => Json(result).into_response(),
        Err(resp) => resp,
    }
}

/// `PATCH /api/v2/skills/{skill_id}?owner_agent=…&expected_version=…` —
/// partial update; only provided fields change.
#[tracing::instrument(skip(state))]
pub async fn skill_patch(
    State(state): State<Arc<ServerState>>,
    AxumPath(skill_id): AxumPath<String>,
    Query(params): Query<SkillMutationParams>,
    Json(input): Json<crate::sdk::SkillPatchInput>,
) -> Response {
    match run_db_op(&state, move |db| {
        let engine = db.engine_handle()?;
        let store = crate::skills::SkillStore::new(&engine);
        require_owned_head(&store, &skill_id, &params.owner_agent)?;
        store.patch(&skill_id, params.expected_version, input)
    })
    .await
    {
        Ok(result) => Json(result).into_response(),
        Err(resp) => resp,
    }
}

/// `DELETE /api/v2/skills/{skill_id}?owner_agent=…&expected_version=…` —
/// removes every version plus the head index row.
#[tracing::instrument(skip(state))]
pub async fn skill_delete(
    State(state): State<Arc<ServerState>>,
    AxumPath(skill_id): AxumPath<String>,
    Query(params): Query<SkillMutationParams>,
) -> Response {
    match run_db_op(&state, move |db| {
        let engine = db.engine_handle()?;
        let store = crate::skills::SkillStore::new(&engine);
        require_owned_head(&store, &skill_id, &params.owner_agent)?;
        store.delete(&skill_id, params.expected_version)
    })
    .await
    {
        Ok(deleted) => Json(serde_json::json!({ "deleted": deleted })).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
pub async fn snapshots_list(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| db.list_snapshots()).await {
        Ok(names) => Json(names).into_response(),
        Err(resp) => resp,
    }
}

/// `FsSnapshot` (storage/engine) is not serializable (`created_at` is a
/// monotonic `Instant`), so the wire shape carries name + path only.
#[tracing::instrument(skip(state))]
pub async fn snapshots_create(
    State(state): State<Arc<ServerState>>,
    AxumPath(name): AxumPath<String>,
) -> Response {
    let name_label = name.clone();
    match run_db_op(&state, move |db| db.create_snapshot(&name)).await {
        Ok(snapshot) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "name": name_label,
                "path": snapshot.path.to_string_lossy(),
            })),
        )
            .into_response(),
        Err(resp) => resp,
    }
}
