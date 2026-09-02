#![cfg_attr(target_arch = "wasm32", no_main)]
#![warn(missing_docs)]
// ERR-CORE-02: keep clippy deny on prod code; tests may `unwrap`/`expect`
// freely (same pattern as vantadb/src/lib.rs).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! WASM bindings for the VantaDB embedded vector database.
//!
//! This crate provides JavaScript-accessible types and functions via `wasm_bindgen`,
//! exposing VantaDB's core operations (put, get, delete, search, graph traversal, etc.)
//! to WebAssembly targets. It also includes an optional OPFS persistence layer and
//! a SIMD-accelerated cosine distance helper.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Global counter of NaN/Inf→0.0 sanitizations applied to outgoing WASM data.
///
/// WSM-12 (research-vantadb-wasm-20260825 H-15): Float32Array / JSON cannot
/// represent NaN or Infinity, so the WASM glue coerces them silently. This
/// counter makes the silent data alteration observable via
/// `operational_metrics().nan_sanitization_count`. Accumulated (not reset) so
/// the value reflects the lifetime of the WASM instance.
static NAN_SANITIZATION_COUNT: AtomicU64 = AtomicU64::new(0);

/// Increment the sanitization counter by `n` (typically 1, but vector coercion
/// passes the count of replaced elements in a single call).
#[inline]
fn record_nan_sanitization(n: u64) {
    NAN_SANITIZATION_COUNT.fetch_add(n, Ordering::Relaxed);
}

/// Global counter of metadata serializations that silently failed and were
/// dropped from the outgoing JS record.
///
/// WSM-11 (research-vantadb-wasm-20260825 H-14): `memory_record_to_js`
/// used `if let Ok(meta) = serde_wasm_bindgen::to_value(&rec.metadata)` —
/// on serialization failure the record was returned **without** metadata and
/// without any signal, a silent data-loss path. This counter makes the loss
/// observable via `operational_metrics().metadata_drop_count`. Accumulated
/// (not reset) so the value reflects the lifetime of the WASM instance.
static METADATA_DROP_COUNT: AtomicU64 = AtomicU64::new(0);

/// Increment the metadata-drop counter by `n` (always 1 per dropped record).
#[inline]
fn record_metadata_drop(n: u64) {
    METADATA_DROP_COUNT.fetch_add(n, Ordering::Relaxed);
}
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use vantadb::config::VantaConfig;
use vantadb::graph::TraversalDirection;
use vantadb::sdk::*;
use vantadb::{BackendKind, SparseVector, VantaError, MAX_BATCH_SIZE, MAX_F32_VEC_LEN, MAX_K};
use wasm_bindgen::prelude::*;

mod opfs;
/// OPFS file handle abstraction wrapping a JS `FileSystemFileHandle`.
pub use opfs::OpfsFile;
/// OPFS-based storage for persisting VantaDB state in the browser.
pub use opfs::OpfsStorage;

mod idb;
/// IndexedDB-based storage for browsers without OPFS support.
pub use idb::IdbStorage;

/// Web Worker bridge for offloading OPFS I/O to a dedicated thread.
///
/// Provides `OpfsWorkerProxy` for main-thread communication with a
/// background worker that handles all `opfs::OpfsStorage` operations.
#[cfg(feature = "opfs")]
pub mod worker;

// FFI guards (`MAX_F32_VEC_LEN`, `MAX_BATCH_SIZE`, `MAX_K`) now live in core
// (`vantadb::config`) — single source of truth across transports (WSM-09).

/// Minimal WASM-friendly config that maps to VantaConfig
#[derive(Deserialize)]
#[serde(default)]
struct WasmConfig {
    storage_path: String,
    read_only: bool,
    rss_threshold: f64,
    memory_limit: Option<u64>,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            storage_path: "vantadb_data".to_string(),
            read_only: false,
            rss_threshold: 0.80,
            memory_limit: None,
        }
    }
}

fn build_config(wasm: WasmConfig) -> VantaConfig {
    VantaConfig {
        storage_path: wasm.storage_path,
        read_only: wasm.read_only,
        rss_threshold: wasm.rss_threshold,
        memory_limit: wasm.memory_limit,
        backend_kind: BackendKind::InMemory,
        ..VantaConfig::default()
    }
}

/// Serializable wrapper for VantaMemoryInput
#[derive(Serialize, Deserialize)]
struct MemoryInput {
    namespace: String,
    key: String,
    payload: String,
    #[serde(default)]
    metadata: VantaMemoryMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    vector: Option<Vec<f32>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_sparse_vector"
    )]
    sparse_vector: Option<SparseVector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl_ms: Option<u64>,
}

/// serde adapter for `sparse_vector`: plain JS objects always carry string
/// keys, and serde-wasm-bindgen — unlike serde_json — does not coerce them to
/// the core's `BTreeMap<u32, f32>`. Accept numeric-string keys so the
/// documented `Record<number, number>` SDK shape round-trips.
fn deserialize_sparse_vector<'de, D>(deserializer: D) -> Result<Option<SparseVector>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;
    let raw: Option<std::collections::BTreeMap<String, f32>> =
        serde::Deserialize::deserialize(deserializer)?;
    let Some(raw) = raw else { return Ok(None) };
    let mut map = std::collections::BTreeMap::new();
    for (k, v) in raw {
        let id = k
            .parse::<u32>()
            .map_err(|_| D::Error::custom(format!("sparse_vector key '{k}' is not a u32")))?;
        map.insert(id, v);
    }
    Ok(Some(SparseVector(map)))
}

/// Search request
#[derive(Serialize, Deserialize)]
struct SearchRequest {
    namespace: String,
    query_vector: Vec<f32>,
    #[serde(default)]
    filters: VantaMemoryMetadata,
    /// Hide superseded records from results.
    #[serde(default)]
    exclude_superseded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_query: Option<String>,
    #[serde(default = "default_top_k")]
    top_k: usize,
    #[serde(default = "default_distance")]
    distance_metric: String,
    #[serde(default)]
    explain: bool,
}

fn default_top_k() -> usize {
    10
}
fn default_distance() -> String {
    "Cosine".to_string()
}

/// List/pagination options deserialized from JS.
#[derive(Serialize, Deserialize)]
struct ListOptions {
    #[serde(default)]
    filters: VantaMemoryMetadata,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default, deserialize_with = "deserialize_cursor")]
    cursor: Option<usize>,
    /// Hide superseded records from the listing (consistency with `search`).
    #[serde(default)]
    exclude_superseded: bool,
}

fn default_limit() -> usize {
    100
}

/// Deserialize a pagination cursor from JS.
///
/// Policy string-u64: cursors travel as decimal strings so values beyond
/// 2^53 are never lossy through f64. Plain numbers are still accepted for
/// backward compatibility with older callers.
fn deserialize_cursor<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde_json::Value as Json;
    match Option::<Json>::deserialize(deserializer)? {
        None | Some(Json::Null) => Ok(None),
        Some(Json::String(s)) => s
            .parse::<usize>()
            .map(Some)
            .map_err(|e| serde::de::Error::custom(format!("invalid cursor '{s}': {e}"))),
        Some(Json::Number(n)) => n
            .as_u64()
            .and_then(|u| usize::try_from(u).ok())
            .map(Some)
            .ok_or_else(|| serde::de::Error::custom("cursor value out of range")),
        Some(other) => Err(serde::de::Error::custom(format!(
            "cursor must be a decimal string or number, got {other}"
        ))),
    }
}

/// Serialize a page cursor for JS as a decimal string (policy string-u64:
/// avoids f64 precision loss above 2^53). `None` becomes `null`.
fn next_cursor_to_js(cursor: Option<u64>) -> JsValue {
    match cursor {
        Some(c) => JsValue::from_str(&c.to_string()),
        None => JsValue::NULL,
    }
}

// ── JS‑facing record types (u64 → String for JS Number safety) ──────────

#[derive(Serialize)]
struct JsNodeRecord {
    id: String,
    fields: VantaFields,
    vector: Option<Vec<f32>>,
    vector_dimensions: usize,
    edges: Vec<VantaEdgeRecord>,
    confidence_score: f32,
    importance: f32,
    hits: u32,
    last_accessed: String,
    epoch: u32,
    tier: VantaStorageTier,
    is_alive: bool,
}

impl From<VantaNodeRecord> for JsNodeRecord {
    fn from(n: VantaNodeRecord) -> Self {
        JsNodeRecord {
            id: n.id.to_string(),
            fields: n.fields,
            vector: n.vector,
            vector_dimensions: n.vector_dimensions,
            edges: n.edges,
            confidence_score: n.confidence_score,
            importance: n.importance,
            hits: n.hits,
            last_accessed: n.last_accessed.to_string(),
            epoch: n.epoch,
            tier: n.tier,
            is_alive: n.is_alive,
        }
    }
}

#[derive(Serialize)]
struct JsOperationalMetrics {
    startup_ms: String,
    wal_replay_ms: String,
    wal_records_replayed: String,
    ann_rebuild_ms: String,
    ann_rebuild_scanned_nodes: String,
    derived_rebuild_ms: String,
    text_index_rebuild_ms: String,
    text_postings_written: String,
    text_index_repairs: String,
    text_lexical_queries: String,
    text_lexical_query_ms: String,
    text_candidates_scored: String,
    text_consistency_audits: String,
    text_consistency_audit_failures: String,
    hybrid_query_ms: String,
    hybrid_candidates_fused: String,
    planner_hybrid_queries: String,
    planner_text_only_queries: String,
    planner_vector_only_queries: String,
    records_exported: String,
    records_imported: String,
    import_errors: String,
    derived_prefix_scans: String,
    derived_full_scan_fallbacks: String,
    process_rss_bytes: String,
    process_virtual_bytes: String,
    hnsw_nodes_count: String,
    hnsw_logical_bytes: String,
    mmap_resident_bytes: Option<String>,
    volatile_cache_entries: String,
    volatile_cache_cap_bytes: String,
    jemalloc_allocated_bytes: Option<String>,
    jemalloc_active_bytes: Option<String>,
    jemalloc_metadata_bytes: Option<String>,
    jemalloc_resident_bytes: Option<String>,
    jemalloc_mapped_bytes: Option<String>,
    jemalloc_retained_bytes: Option<String>,
    /// Cumulative count of NaN/Inf→0.0 sanitizations applied to outgoing
    /// vector/explanation payloads (WSM-12). Non-zero is a signal that
    /// upstream data (embeddings, scores) contains non-finite floats.
    nan_sanitization_count: String,
    /// Cumulative count of metadata fields that failed WASM serialization and
    /// were dropped from outgoing records (WSM-11). Non-zero means the
    /// metadata contained a value the JSON-compatible serializer couldn't
    /// represent (e.g. unsupported map key type, non-finite float). The record
    /// is still returned (sans metadata) to preserve backward compat — this
    /// counter exists so the silent data loss is observable.
    metadata_drop_count: String,
}

impl From<VantaOperationalMetrics> for JsOperationalMetrics {
    fn from(m: VantaOperationalMetrics) -> Self {
        JsOperationalMetrics {
            startup_ms: m.startup_ms.to_string(),
            wal_replay_ms: m.wal_replay_ms.to_string(),
            wal_records_replayed: m.wal_records_replayed.to_string(),
            ann_rebuild_ms: m.ann_rebuild_ms.to_string(),
            ann_rebuild_scanned_nodes: m.ann_rebuild_scanned_nodes.to_string(),
            derived_rebuild_ms: m.derived_rebuild_ms.to_string(),
            text_index_rebuild_ms: m.text_index_rebuild_ms.to_string(),
            text_postings_written: m.text_postings_written.to_string(),
            text_index_repairs: m.text_index_repairs.to_string(),
            text_lexical_queries: m.text_lexical_queries.to_string(),
            text_lexical_query_ms: m.text_lexical_query_ms.to_string(),
            text_candidates_scored: m.text_candidates_scored.to_string(),
            text_consistency_audits: m.text_consistency_audits.to_string(),
            text_consistency_audit_failures: m.text_consistency_audit_failures.to_string(),
            hybrid_query_ms: m.hybrid_query_ms.to_string(),
            hybrid_candidates_fused: m.hybrid_candidates_fused.to_string(),
            planner_hybrid_queries: m.planner_hybrid_queries.to_string(),
            planner_text_only_queries: m.planner_text_only_queries.to_string(),
            planner_vector_only_queries: m.planner_vector_only_queries.to_string(),
            records_exported: m.records_exported.to_string(),
            records_imported: m.records_imported.to_string(),
            import_errors: m.import_errors.to_string(),
            derived_prefix_scans: m.derived_prefix_scans.to_string(),
            derived_full_scan_fallbacks: m.derived_full_scan_fallbacks.to_string(),
            process_rss_bytes: m.process_rss_bytes.to_string(),
            process_virtual_bytes: m.process_virtual_bytes.to_string(),
            hnsw_nodes_count: m.hnsw_nodes_count.to_string(),
            hnsw_logical_bytes: m.hnsw_logical_bytes.to_string(),
            mmap_resident_bytes: m.mmap_resident_bytes.map(|v| v.to_string()),
            volatile_cache_entries: m.volatile_cache_entries.to_string(),
            volatile_cache_cap_bytes: m.volatile_cache_cap_bytes.to_string(),
            jemalloc_allocated_bytes: m.jemalloc_allocated_bytes.map(|v| v.to_string()),
            jemalloc_active_bytes: m.jemalloc_active_bytes.map(|v| v.to_string()),
            jemalloc_metadata_bytes: m.jemalloc_metadata_bytes.map(|v| v.to_string()),
            jemalloc_resident_bytes: m.jemalloc_resident_bytes.map(|v| v.to_string()),
            jemalloc_mapped_bytes: m.jemalloc_mapped_bytes.map(|v| v.to_string()),
            jemalloc_retained_bytes: m.jemalloc_retained_bytes.map(|v| v.to_string()),
            // WSM-12: pull from WASM-local counter (not core snapshot).
            nan_sanitization_count: NAN_SANITIZATION_COUNT.load(Ordering::Relaxed).to_string(),
            // WSM-11: pull from WASM-local counter (not core snapshot).
            metadata_drop_count: METADATA_DROP_COUNT.load(Ordering::Relaxed).to_string(),
        }
    }
}

/// The main VantaDB handle exposed to JavaScript via `wasm_bindgen`.
#[wasm_bindgen]
pub struct VantaDB {
    inner: VantaEmbedded,
    opfs: Option<OpfsStorage>,
    /// Whether this handle has a durable persistence backend attached (OPFS,
    /// IDB, or worker). `inner.capabilities().persistence` is hardcoded true
    /// in the core SDK (Fjall default), so the WASM layer overrides it with
    /// this faithful flag — fixes WSM-01 where OPFS failure silently fell
    /// back to in-memory while `capabilities().persistence` still claimed true.
    persistence: bool,
    op_gate: OpGate,
    #[cfg(feature = "opfs")]
    worker: Option<worker::OpfsWorkerProxy>,
    /// PERF-08 differential-persist cache (see `PersistCache`). Holds the last
    /// persiated snapshot so `save`/`save_idb` re-serialize only what changed.
    persist_cache: Mutex<PersistCache>,
    /// WSM-03: Tracks whether there are unsaved changes since the last persist.
    /// Set to `true` by mutation methods (`put`, `put_batch`, `delete`, etc.),
    /// reset to `false` after a successful `save`/`save_idb`.
    dirty: AtomicBool,
    /// WSM-03: Whether auto-save is enabled. When `true`, the JS glue will
    /// call `try_auto_save()` on `visibilitychange`/`pagehide` events.
    auto_save_enabled: AtomicBool,
}

/// PERF-08 differential-persist cache.
///
/// Keeps the last-persisted snapshot as `(namespace, key) -> (version,
/// serialized_json)`. On `save`/`save_idb` only records whose `version`
/// changed (or that were deleted) since the last persist are (re)serialized;
/// every other record reuses its previously-serialized JSON string. This turns
/// the old O(N) full-dataset `serde_json::to_vec` on every persist into
/// O(changes) serialization and skips the file write entirely when nothing
/// changed — eliminating the multi-second event-loop block on >100MB datasets.
///
/// `dirty`/`deleted` are fed by the mutation entry points (`put`, `delete`,
/// ...). `cache_invalid` is set by bulk operations whose changed keys are not
/// individually known (import/bulk/reindex/purge) and forces a one-time full
/// rebuild of the cache on the next persist.
struct PersistCache {
    /// (namespace, key) -> (record version at last persist, serialized JSON of that record)
    records: HashMap<(String, String), (u64, String)>,
    /// Keys touched since the last persist (need re-serialize / re-fetch).
    dirty: HashSet<(String, String)>,
    /// Keys deleted since the last persist.
    deleted: HashSet<(String, String)>,
    /// Cache must be rebuilt from a full `collect_all_deduped` (bulk op happened).
    cache_invalid: bool,
}

impl PersistCache {
    fn new() -> Self {
        Self {
            records: HashMap::new(),
            dirty: HashSet::new(),
            deleted: HashSet::new(),
            cache_invalid: false,
        }
    }
}

/// Durability gate: rejects new operations once `close()` has begun and keeps
/// `close()` waiting until every in-flight operation finishes. Mirrors
/// `vantadb-node/src/lib.rs` and `vantadb-python/src/lib.rs` — closes the
/// write-after-close race where an operation started before `close()` would
/// still write to the engine after close returned.
struct OpGate {
    state: Arc<(Mutex<OpState>, Condvar)>,
}

struct OpState {
    closing: bool,
    count: usize,
}

impl OpGate {
    fn new() -> Self {
        Self {
            state: Arc::new((
                Mutex::new(OpState {
                    closing: false,
                    count: 0,
                }),
                Condvar::new(),
            )),
        }
    }

    /// Register a new in-flight operation. Returns `None` if `close()` has
    /// started (new operations are rejected past the durability barrier).
    fn try_enter(&self) -> Option<OpGuard> {
        let (lock, _) = &*self.state;
        let mut state = lock.lock().unwrap_or_else(PoisonError::into_inner);
        if state.closing {
            return None;
        }
        state.count += 1;
        Some(OpGuard {
            state: self.state.clone(),
        })
    }

    /// Start closing and block until every in-flight operation drains.
    ///
    /// Sets `closing = true` (so new ops are rejected) then waits until
    /// `count == 0`. Blocks the calling thread; acceptable: this is the
    /// durability barrier and engine operations are bounded.
    fn drain(&self) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap_or_else(PoisonError::into_inner);
        state.closing = true;
        // wasm32-unknown-unknown: std Condvar::wait panics (single-threaded
        // no_threads shim) and could never make progress anyway — the only
        // thread that could drain `count` is this one, so a blocking wait
        // would deadlock the JS event loop. The barrier still rejects new
        // ops (closing=true); in-flight async ops finish on the event loop.
        #[cfg(not(target_arch = "wasm32"))]
        while state.count > 0 {
            state = cvar.wait(state).unwrap_or_else(PoisonError::into_inner);
        }
        #[cfg(target_arch = "wasm32")]
        let _ = cvar;
    }
}

/// RAII guard that decrements the in-flight count and wakes `close()` when
/// dropped (at the end of the owning method, after the engine call completes).
struct OpGuard {
    state: Arc<(Mutex<OpState>, Condvar)>,
}

impl Drop for OpGuard {
    fn drop(&mut self) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap_or_else(PoisonError::into_inner);
        state.count -= 1;
        cvar.notify_one();
    }
}

/// Enter the gate for an engine operation, or fail with a descriptive error
/// if the database is closing.
fn enter(gate: &OpGate) -> Result<OpGuard, JsValue> {
    gate.try_enter()
        .ok_or_else(|| JsValue::from_str("database is closing"))
}

/// Emit `console.warn(msg)` when a console exists (best-effort, never throws).
fn console_warn(msg: &str) {
    let global = js_sys::global();
    let console = js_sys::Reflect::get(&global, &"console".into()).ok();
    let warn = console
        .as_ref()
        .and_then(|c| js_sys::Reflect::get(c, &"warn".into()).ok());
    if let Some(w) = warn.and_then(|w| w.dyn_into::<js_sys::Function>().ok()) {
        let _ = w.call1(&JsValue::undefined(), &JsValue::from_str(msg));
    }
}

const MAX_RECORDS: usize = 1_000_000;

#[wasm_bindgen]
impl VantaDB {
    /// Create a new VantaDB instance from an optional WASM config object.
    #[wasm_bindgen(constructor)]
    pub fn new(config_val: Option<JsValue>) -> Result<VantaDB, JsValue> {
        init();
        let wasm_cfg = match config_val {
            Some(val) => from_js::<WasmConfig>(val)?,
            None => WasmConfig::default(),
        };
        let config = build_config(wasm_cfg);
        let inner = VantaEmbedded::open_with_config(config).map_err(to_js_err)?;
        Ok(VantaDB {
            inner,
            opfs: None,
            persistence: false,
            op_gate: OpGate::new(),
            persist_cache: Mutex::new(PersistCache::new()),
            #[cfg(feature = "opfs")]
            worker: None,
            dirty: AtomicBool::new(false),
            auto_save_enabled: AtomicBool::new(false),
        })
    }

    /// Open VantaDB at the given storage path.
    pub fn open(path: &str) -> Result<VantaDB, JsValue> {
        init();
        let wasm_cfg = WasmConfig {
            storage_path: path.to_string(),
            ..WasmConfig::default()
        };
        let config = build_config(wasm_cfg);
        let inner = VantaEmbedded::open_with_config(config).map_err(to_js_err)?;
        Ok(VantaDB {
            inner,
            opfs: None,
            persistence: false,
            op_gate: OpGate::new(),
            persist_cache: Mutex::new(PersistCache::new()),
            #[cfg(feature = "opfs")]
            worker: None,
            dirty: AtomicBool::new(false),
            auto_save_enabled: AtomicBool::new(false),
        })
    }

    /// Open VantaDB with OPFS-based persistent storage in the browser.
    ///
    /// WSM-01: this no longer swallows `OpfsStorage::open` errors (previous
    /// `ok` fallback removed). If OPFS is unavailable (e.g.
    /// `navigator.storage.getDirectory` rejects),
    /// the call now rejects with a descriptive `JsValue` so the caller never
    /// gets a silent in-memory DB under the illusion of persistence (capabilities
    /// would otherwise still claim `persistence:true`).
    pub async fn connect_persistent(path: &str) -> Result<VantaDB, JsValue> {
        init();
        let opfs = OpfsStorage::open(path).await.map_err(|e| {
            let detail = js_sys::Error::from(e)
                .message()
                .as_string()
                .unwrap_or_else(|| "unknown OPFS error".to_string());
            JsValue::from(js_sys::Error::new(&format!(
                "OPFS unavailable for '{path}': {detail} — use VantaDB.connect_idb for IndexedDB fallback"
            )))
        })?;
        let wasm_cfg = WasmConfig {
            storage_path: path.to_string(),
            ..WasmConfig::default()
        };
        let config = build_config(wasm_cfg);
        let inner = VantaEmbedded::open_with_config(config).map_err(to_js_err)?;
        let db = VantaDB {
            inner,
            opfs: Some(opfs),
            persistence: true,
            op_gate: OpGate::new(),
            persist_cache: Mutex::new(PersistCache::new()),
            #[cfg(feature = "opfs")]
            worker: None,
            dirty: AtomicBool::new(false),
            auto_save_enabled: AtomicBool::new(false),
        };
        db.load().await?;
        Ok(db)
    }

    /// Open VantaDB with IndexedDB-based persistent storage (fallback when OPFS is unavailable).
    pub async fn connect_idb(path: &str) -> Result<VantaDB, JsValue> {
        init();
        let wasm_cfg = WasmConfig {
            storage_path: path.to_string(),
            ..WasmConfig::default()
        };
        let config = build_config(wasm_cfg);
        let inner = VantaEmbedded::open_with_config(config).map_err(to_js_err)?;
        let db = VantaDB {
            inner,
            opfs: None,
            persistence: true,
            op_gate: OpGate::new(),
            persist_cache: Mutex::new(PersistCache::new()),
            #[cfg(feature = "opfs")]
            worker: None,
            dirty: AtomicBool::new(false),
            auto_save_enabled: AtomicBool::new(false),
        };
        db.load_idb().await?;
        Ok(db)
    }

    /// Open VantaDB with OPFS persistence via a dedicated Web Worker.
    ///
    /// **Optional capability:** this method only exists when the package is
    /// built with the `opfs` feature (`wasm-pack build --features opfs`).
    ///
    /// The `spawnOpfsWorker` helper is automatically registered on
    /// `globalThis` when `opfs_bridge.js` is loaded (glue side-effect,
    /// DuckDB-WASM pattern), so callers can just do:
    ///
    /// ```js
    /// import "vantadb-wasm/opfs_bridge.js";
    /// const db = await VantaDB.connect_worker("my-db");
    /// ```
    ///
    /// For backwards compatibility, manual injection still works:
    ///
    /// ```js
    /// import { spawnOpfsWorker } from "vantadb-wasm/src/opfs_bridge.js";
    /// globalThis.spawnOpfsWorker = spawnOpfsWorker;
    /// ```
    #[cfg(feature = "opfs")]
    pub async fn connect_worker(path: &str) -> Result<VantaDB, JsValue> {
        init();
        let worker_proxy = {
            let global = js_sys::global();
            let spawn_fn =
                js_sys::Reflect::get(&global, &"spawnOpfsWorker".into()).map_err(|_| {
                    JsValue::from_str("spawnOpfsWorker not available — import opfs_bridge.js")
                })?;
            let worker = spawn_fn
                .dyn_into::<js_sys::Function>()
                .map_err(|_| JsValue::from_str("spawnOpfsWorker is not a function"))?
                .call0(&global)?;
            let proxy = worker::OpfsWorkerProxy::new(worker);
            proxy.init(path).await?;
            proxy
        };
        let wasm_cfg = WasmConfig {
            storage_path: path.to_string(),
            ..WasmConfig::default()
        };
        let config = build_config(wasm_cfg);
        let inner = VantaEmbedded::open_with_config(config).map_err(to_js_err)?;
        let db = VantaDB {
            inner,
            opfs: None,
            persistence: true,
            op_gate: OpGate::new(),
            persist_cache: Mutex::new(PersistCache::new()),
            worker: Some(worker_proxy),
            dirty: AtomicBool::new(false),
            auto_save_enabled: AtomicBool::new(false),
        };
        // Load from worker-backed storage
        let data = db.worker_read("db_state.json").await?;
        if let Some(d) = data {
            let records: Vec<VantaMemoryRecord> = serde_json::from_slice(&d)
                .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
            db.populate_cache_from_records(&records);
            if !records.is_empty() {
                db.inner.import_records(records).map_err(to_js_err)?;
            }
        }
        // CORE-02: restore the graph store alongside the memory records.
        if let Some(graph) = db.worker_read("graph_state.json").await? {
            db.restore_graph_payload(&graph)?;
        }
        Ok(db)
    }

    /// Read a file from the worker-backed OPFS storage.
    ///
    /// **Optional capability:** only available when built with the `opfs`
    /// feature and the instance was opened via `connect_worker`.
    #[cfg(feature = "opfs")]
    pub async fn worker_read(&self, path: &str) -> Result<Option<Vec<u8>>, JsValue> {
        match &self.worker {
            Some(w) => w.read(path).await,
            None => Err(JsValue::from_str("worker not initialized")),
        }
    }

    /// Write a file through the worker-backed OPFS storage.
    ///
    /// **Optional capability:** only available when built with the `opfs`
    /// feature and the instance was opened via `connect_worker`.
    #[cfg(feature = "opfs")]
    pub async fn worker_write(&self, path: &str, data: Vec<u8>) -> Result<(), JsValue> {
        match &self.worker {
            Some(w) => w.write(path, &data).await,
            None => Err(JsValue::from_str("worker not initialized")),
        }
    }

    /// Delete a file through the worker-backed OPFS storage.
    ///
    /// **Optional capability:** only available when built with the `opfs`
    /// feature and the instance was opened via `connect_worker`.
    #[cfg(feature = "opfs")]
    pub async fn worker_delete(&self, path: &str) -> Result<(), JsValue> {
        match &self.worker {
            Some(w) => w.delete(path).await,
            None => Err(JsValue::from_str("worker not initialized")),
        }
    }

    /// Collect all in-memory records deduplicated by (namespace, key).
    ///
    /// Dedup uses `node_id` (a u128 `XxHash3_128` over `namespace\0key`, see
    /// `vantadb::sdk::serialization::memory_node_id`) instead of allocating two
    /// Strings per record — identical semantics, zero per-record allocation.
    fn collect_all_deduped(&self) -> Result<Vec<VantaMemoryRecord>, JsValue> {
        let _g = enter(&self.op_gate)?;
        let mut seen: HashSet<u128> = HashSet::new();
        let mut state: Vec<VantaMemoryRecord> = Vec::new();
        let namespaces: Vec<String> = self.inner.list_namespaces().map_err(to_js_err)?;
        for ns in &namespaces {
            let mut cursor: Option<usize> = None;
            loop {
                let opts = VantaMemoryListOptions {
                    #[allow(deprecated)]
                    filters: VantaMemoryMetadata::new(),
                    filter_ops: None,
                    limit: 10_000,
                    cursor,
                    exclude_superseded: false,
                };
                let page = self.inner.list(ns, opts).map_err(to_js_err)?;
                for record in page.records {
                    if seen.insert(record.node_id) {
                        if state.len() >= MAX_RECORDS {
                            return Err(JsValue::from_str("too many records to export"));
                        }
                        state.push(record);
                    }
                }
                cursor = page.next_cursor;
                if cursor.is_none() {
                    break;
                }
            }
        }
        Ok(state)
    }

    // ── PERF-08 differential-persist helpers ──────────────────────────────

    /// Mark a single record key as changed since the last persist.
    fn mark_dirty(&self, namespace: &str, key: &str) {
        let mut cache = self
            .persist_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        cache.dirty.insert((namespace.to_string(), key.to_string()));
        // WSM-03: Track that there are unsaved changes for auto-save
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Mark a single record key as deleted since the last persist.
    fn mark_deleted(&self, namespace: &str, key: &str) {
        let mut cache = self
            .persist_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let k = (namespace.to_string(), key.to_string());
        cache.dirty.remove(&k);
        cache.deleted.insert(k);
        // WSM-03: Track that there are unsaved changes for auto-save
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Mark the cache stale because an unknown set of keys changed (bulk ops).
    fn mark_cache_invalid(&self) {
        let mut cache = self
            .persist_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        cache.cache_invalid = true;
        // WSM-03: Track that there are unsaved changes for auto-save
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Seed the cache from a freshly-loaded snapshot so the next `save` is a
    /// no-op unless a mutation occurs (avoids a redundant full re-serialize).
    fn populate_cache_from_records(&self, records: &[VantaMemoryRecord]) {
        let mut cache = self
            .persist_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        cache.records.clear();
        cache.dirty.clear();
        cache.deleted.clear();
        cache.cache_invalid = false;
        for rec in records {
            let key = (rec.namespace.clone(), rec.key.clone());
            match serde_json::to_string(rec) {
                Ok(s) => {
                    cache.records.insert(key, (rec.version, s));
                }
                Err(_) => {
                    // Serialization failure is unrecoverable here; force a full
                    // rebuild on the next persist instead of silently dropping.
                    cache.cache_invalid = true;
                    break;
                }
            }
        }
    }

    /// Build the serialized `db_state.json` payload using the differential
    /// persist cache. Returns `None` when nothing changed since the last
    /// successful persist, so the caller can skip the (potentially large) file
    /// write entirely.
    ///
    /// Only records whose `version` changed (or that were deleted) since the
    /// last persist are (re)serialized; every other record reuses its cached
    /// JSON string. Output is a valid `Vec<VantaMemoryRecord>` JSON array,
    /// byte-for-byte loadable by `load`/`load_idb`.
    fn persist_payload(&self) -> Result<Option<Vec<u8>>, JsValue> {
        let mut cache = self
            .persist_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        if cache.dirty.is_empty() && cache.deleted.is_empty() && !cache.cache_invalid {
            return Ok(None);
        }

        if cache.cache_invalid {
            // Bulk op changed an unknown key set — rebuild cache from a full
            // collect once, then fall through to emit the full snapshot.
            cache.records.clear();
            cache.dirty.clear();
            cache.deleted.clear();
            let current = self.collect_all_deduped()?;
            for rec in current {
                let key = (rec.namespace.clone(), rec.key.clone());
                let s = serde_json::to_string(&rec)
                    .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
                cache.records.insert(key, (rec.version, s));
            }
            cache.cache_invalid = false;
        } else {
            // Apply deletions.
            let deleted: Vec<(String, String)> = cache.deleted.drain().collect();
            for k in deleted {
                cache.records.remove(&k);
            }
            // Re-serialize only the dirty records (fetch current version).
            let dirty: Vec<(String, String)> = cache.dirty.drain().collect();
            for (ns, key) in dirty {
                match self.inner.get(&ns, &key).map_err(to_js_err)? {
                    Some(rec) => {
                        let s = serde_json::to_string(&rec)
                            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
                        cache.records.insert((ns, key), (rec.version, s));
                    }
                    None => {
                        cache.records.remove(&(ns, key));
                    }
                }
            }
        }

        // Reconstruct a valid JSON array from cached serialized record strings.
        let mut out = String::with_capacity(cache.records.len().saturating_mul(64) + 2);
        out.push('[');
        let mut first = true;
        for (_k, (_v, s)) in cache.records.iter() {
            if !first {
                out.push(',');
            }
            out.push_str(s);
            first = false;
        }
        out.push(']');
        Ok(Some(out.into_bytes()))
    }

    /// Build the serialized `graph_state.json` payload (CORE-02): every live
    /// non-memory-record node with labels/edges resolved, so the graph store
    /// survives an OPFS/IDB reopen. Unlike `persist_payload` this is a full
    /// rewrite each save — ponytail: differential graph persist only if saves
    /// ever show up in a profile.
    fn graph_payload(&self) -> Result<Vec<u8>, JsValue> {
        let nodes = self.inner.collect_graph_nodes().map_err(to_js_err)?;
        serde_json::to_vec(&nodes).map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))
    }

    /// Restore graph nodes from a `graph_state.json` payload (CORE-02).
    fn restore_graph_payload(&self, data: &[u8]) -> Result<(), JsValue> {
        let nodes: Vec<VantaNodeRecord> = serde_json::from_slice(data)
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
        if !nodes.is_empty() {
            self.inner.restore_graph_nodes(nodes).map_err(to_js_err)?;
        }
        Ok(())
    }

    /// Persist in-memory records to OPFS storage using differential writes.
    ///
    /// Only records changed since the last successful `save` are serialized;
    /// if nothing changed the file write is skipped entirely (PERF-08).
    pub async fn save(&self) -> Result<(), JsValue> {
        let _g = enter(&self.op_gate)?;
        let opfs = match &self.opfs {
            Some(o) => o,
            None => return Ok(()),
        };
        if let Some(data) = self.persist_payload()? {
            if let Err(e) = opfs.write_file("db_state.json", &data).await {
                // Write failed: force a full rebuild+rewrite on the next
                // save instead of silently skipping (dirty was already
                // drained by persist_payload).
                self.mark_cache_invalid();
                return Err(e);
            }
        }
        // CORE-02: graph nodes live outside the memory-record snapshot —
        // always rewrite so deletions are reflected (small file, standalone
        // scale).
        let graph = self.graph_payload()?;
        opfs.write_file("graph_state.json", &graph).await?;
        // WSM-03: Clear dirty flag on successful persist
        self.dirty.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Persist in-memory records to IndexedDB storage using differential writes.
    ///
    /// Only records changed since the last successful `save_idb` are
    /// serialized; if nothing changed the file write is skipped (PERF-08).
    pub async fn save_idb(&self) -> Result<(), JsValue> {
        let _g = enter(&self.op_gate)?;
        if let Some(data) = self.persist_payload()? {
            if let Err(e) = IdbStorage::write_file("db_state.json", &data).await {
                self.mark_cache_invalid();
                return Err(e);
            }
        }
        // CORE-02: see save() — graph snapshot rides alongside db_state.json.
        let graph = self.graph_payload()?;
        IdbStorage::write_file("graph_state.json", &graph).await?;
        // WSM-03: Clear dirty flag on successful persist
        self.dirty.store(false, Ordering::Relaxed);
        Ok(())
    }

    // ── WSM-03 Auto-save methods ────────────────────────────────────────────

    /// Enable auto-save on visibilitychange/pagehide events.
    ///
    /// When enabled, the JavaScript glue code (opfs_bridge.js) will call
    /// `try_auto_save()` when the document becomes hidden or is about to unload.
    /// This is opt-in — call this method after creating a persistent connection
    /// (`connect_persistent`, `connect_idb`, or `connect_worker`) to activate it.
    ///
    /// # Example (JavaScript)
    /// ```js
    /// import { registerAutoSave } from "vantadb-wasm/src/opfs_bridge.js";
    /// const db = await VantaDB.connect_persistent("my-db");
    /// db.enable_auto_save();
    /// registerAutoSave(db);
    /// ```
    #[wasm_bindgen]
    pub fn enable_auto_save(&self) {
        self.auto_save_enabled.store(true, Ordering::Relaxed);
    }

    /// Disable auto-save.
    ///
    /// After calling this, the JS glue will no longer attempt auto-save on
    /// visibilitychange/pagehide events.
    #[wasm_bindgen]
    pub fn disable_auto_save(&self) {
        self.auto_save_enabled.store(false, Ordering::Relaxed);
    }

    /// Check if auto-save is currently enabled.
    #[wasm_bindgen]
    pub fn is_auto_save_enabled(&self) -> bool {
        self.auto_save_enabled.load(Ordering::Relaxed)
    }

    /// Attempt an auto-save if there are unsaved changes and auto-save is enabled.
    ///
    /// This method is intended to be called from JavaScript (via the
    /// `registerAutoSave` glue) on `visibilitychange` (with debounce) and
    /// `pagehide` events. It performs a differential persist (same as `save`/
    /// `save_idb`) but only if the `dirty` flag is set and auto-save is enabled.
    ///
    /// Returns `true` if a save was attempted, `false` if skipped (no changes or
    /// auto-save disabled).
    #[wasm_bindgen]
    pub async fn try_auto_save(&self) -> Result<bool, JsValue> {
        // Check if auto-save is enabled and there are unsaved changes
        if !self.auto_save_enabled.load(Ordering::Relaxed) {
            return Ok(false);
        }
        if !self.dirty.load(Ordering::Relaxed) {
            return Ok(false);
        }

        // Try OPFS first, then IDB
        if self.opfs.is_some() {
            self.save().await?;
        } else {
            self.save_idb().await?;
        }
        Ok(true)
    }

    /// Restore all records from IndexedDB storage into memory.
    pub async fn load_idb(&self) -> Result<(), JsValue> {
        let _g = enter(&self.op_gate)?;
        let data = match IdbStorage::read_file("db_state.json").await? {
            Some(d) => d,
            None => return Ok(()),
        };
        let records: Vec<VantaMemoryRecord> = serde_json::from_slice(&data)
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
        self.populate_cache_from_records(&records);
        if !records.is_empty() {
            self.inner.import_records(records).map_err(to_js_err)?;
        }
        // CORE-02: restore the graph store if a snapshot exists (older
        // snapshots without graph_state.json restore nothing here).
        if let Some(graph) = IdbStorage::read_file("graph_state.json").await? {
            self.restore_graph_payload(&graph)?;
        }
        Ok(())
    }

    /// Delete persisted state from IndexedDB.
    pub async fn delete_idb(&self) -> Result<(), JsValue> {
        IdbStorage::delete_file("db_state.json").await?;
        IdbStorage::delete_file("graph_state.json").await
    }

    /// Restore all records from OPFS storage into memory.
    pub async fn load(&self) -> Result<(), JsValue> {
        let _g = enter(&self.op_gate)?;
        let opfs = match &self.opfs {
            Some(o) => o,
            None => return Ok(()),
        };
        let data = match opfs.read_file("db_state.json").await? {
            Some(d) => d,
            None => return Ok(()),
        };
        let records: Vec<VantaMemoryRecord> = serde_json::from_slice(&data)
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
        self.populate_cache_from_records(&records);
        if !records.is_empty() {
            self.inner.import_records(records).map_err(to_js_err)?;
        }
        // CORE-02: restore the graph store if a snapshot exists (older
        // snapshots without graph_state.json restore nothing here).
        if let Some(graph) = opfs.read_file("graph_state.json").await? {
            self.restore_graph_payload(&graph)?;
        }
        Ok(())
    }

    /// Close the database and release underlying engine resources.
    /// After close, the VantaDB handle should not be used for further operations.
    /// This does NOT free the JS wrapper object — callers should drop references
    /// after close to allow WASM GC to reclaim the wrapper.
    pub fn close(&self) -> Result<(), JsValue> {
        // Durability barrier: reject new ops and wait for in-flight ones to
        // finish BEFORE the engine is closed (see OpGate docs).
        self.op_gate.drain();
        self.inner.close().map_err(to_js_err)
    }

    /// Return the capabilities object describing supported features.
    ///
    /// WSM-01: the core SDK always reports `persistence:true` (Fjall default),
    /// but the WASM persistence lives in OPFS/IDB/worker at this layer. This
    /// override makes `capabilities().persistence` faithful to the actual
    /// backend attached at construction time (`new`/`open` → false,
    /// `connect_persistent`/`connect_idb`/`connect_worker` → true).
    pub fn capabilities(&self) -> Result<JsValue, JsValue> {
        let mut caps = self.inner.capabilities();
        caps.persistence = self.persistence;
        to_js(&caps)
    }

    /// Insert or update a single memory record from a JS object.
    pub fn put(&self, input: JsValue) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let input: MemoryInput = from_js(input)?;
        if let Some(ref v) = input.vector {
            if v.len() > MAX_F32_VEC_LEN {
                return Err(to_js_err(VantaError::InvalidInput(format!(
                    "vector length {} exceeds max {}",
                    v.len(),
                    MAX_F32_VEC_LEN
                ))));
            }
        }
        let vanta_input = VantaMemoryInput {
            namespace: input.namespace.clone(),
            key: input.key.clone(),
            payload: input.payload,
            metadata: input.metadata,
            vector: input.vector,
            sparse_vector: input.sparse_vector,
            ttl_ms: input.ttl_ms,
        };
        let record = self.inner.put(vanta_input).map_err(to_js_err)?;
        self.mark_dirty(&input.namespace, &input.key);
        Ok(memory_record_to_js(record))
    }

    /// Insert or update multiple memory records from a JS array.
    pub fn put_batch(&self, inputs: JsValue) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let inputs: Vec<MemoryInput> = from_js(inputs)?;
        if inputs.len() > MAX_BATCH_SIZE {
            return Err(to_js_err(VantaError::InvalidInput(format!(
                "batch size {} exceeds max {}",
                inputs.len(),
                MAX_BATCH_SIZE
            ))));
        }
        for input in &inputs {
            if let Some(ref v) = input.vector {
                if v.len() > MAX_F32_VEC_LEN {
                    return Err(to_js_err(VantaError::InvalidInput(format!(
                        "vector length {} exceeds max {}",
                        v.len(),
                        MAX_F32_VEC_LEN
                    ))));
                }
            }
        }
        for input in &inputs {
            self.mark_dirty(&input.namespace, &input.key);
        }
        let vanta_inputs: Vec<VantaMemoryInput> = inputs
            .into_iter()
            .map(|i| VantaMemoryInput {
                namespace: i.namespace,
                key: i.key,
                payload: i.payload,
                metadata: i.metadata,
                vector: i.vector,
                sparse_vector: i.sparse_vector,
                ttl_ms: i.ttl_ms,
            })
            .collect();
        let records = self.inner.put_batch(vanta_inputs).map_err(to_js_err)?;
        let arr = js_sys::Array::new();
        for rec in records {
            arr.push(&memory_record_to_js(rec));
        }
        Ok(arr.into())
    }

    /// Retrieve a single record by namespace and key.
    pub fn get(&self, namespace: &str, key: &str) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let record: Option<VantaMemoryRecord> =
            self.inner.get(namespace, key).map_err(to_js_err)?;
        match record {
            Some(rec) => Ok(memory_record_to_js(rec)),
            None => Ok(JsValue::null()),
        }
    }

    /// Delete a single record by namespace and key. Returns whether a record was deleted.
    pub fn delete(&self, namespace: &str, key: &str) -> Result<bool, JsValue> {
        let _g = enter(&self.op_gate)?;
        let deleted = self.inner.delete(namespace, key).map_err(to_js_err)?;
        if deleted {
            self.mark_deleted(namespace, key);
        }
        Ok(deleted)
    }

    /// Return all namespaces as a JS array of strings.
    pub fn list_namespaces(&self) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let nss = self.inner.list_namespaces().map_err(to_js_err)?;
        to_js(&nss)
    }

    /// List records in a namespace with optional filters, limit, and cursor pagination.
    pub fn list(&self, namespace: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let opts: ListOptions = from_js(options)?;
        let vanta_opts = VantaMemoryListOptions {
            #[allow(deprecated)]
            filters: opts.filters,
            filter_ops: None,
            limit: opts.limit,
            cursor: opts.cursor,
            exclude_superseded: opts.exclude_superseded,
        };
        let page = self.inner.list(namespace, vanta_opts).map_err(to_js_err)?;
        let obj = js_sys::Object::new();
        let arr = js_sys::Array::new();
        for rec in page.records {
            arr.push(&memory_record_to_js(rec));
        }
        js_sys::Reflect::set(&obj, &"records".into(), &arr).ok();
        if let Some(cursor) = page.next_cursor {
            let _ = js_sys::Reflect::set(
                &obj,
                &"next_cursor".into(),
                &next_cursor_to_js(Some(cursor as u64)),
            );
        }
        Ok(obj.into())
    }

    /// Serialize a `VantaMemorySearchHit` into a JS object.
    /// Sanitizes NaN/Infinity in explanation scores to avoid JSON serialization errors.
    fn search_hit_to_js(hit: VantaMemorySearchHit) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"record".into(), &memory_record_to_js(hit.record)).ok();
        js_sys::Reflect::set(&obj, &"score".into(), &(hit.score as f64).into()).ok();
        if let Some(ref explanation) = hit.explanation {
            let mut sanitized = explanation.clone();
            if sanitized.score.is_nan() || sanitized.score.is_infinite() {
                sanitized.score = 0.0;
                record_nan_sanitization(1);
            }
            for term in &mut sanitized.bm25_terms {
                if term.contribution.is_nan() || term.contribution.is_infinite() {
                    term.contribution = 0.0;
                    record_nan_sanitization(1);
                }
            }
            if let Ok(expl_js) = serde_wasm_bindgen::to_value(&sanitized) {
                js_sys::Reflect::set(&obj, &"explanation".into(), &expl_js).ok();
            }
        }
        obj.into()
    }

    /// Search memory records by vector similarity with optional filters and text query.
    pub fn search(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let req: SearchRequest = from_js(request)?;
        if req.query_vector.len() > MAX_F32_VEC_LEN {
            return Err(to_js_err(VantaError::InvalidInput(format!(
                "query vector length {} exceeds max {}",
                req.query_vector.len(),
                MAX_F32_VEC_LEN
            ))));
        }
        let distance = match req.distance_metric.as_str() {
            "Euclidean" => vantadb::DistanceMetric::Euclidean,
            _ => vantadb::DistanceMetric::Cosine,
        };
        let vanta_req = VantaMemorySearchRequest {
            namespace: req.namespace,
            query_vector: req.query_vector,
            query_sparse: None,
            #[allow(deprecated)]
            filters: req.filters,
            text_query: req.text_query,
            top_k: req.top_k.min(MAX_K),
            distance_metric: distance,
            explain: req.explain,
            exclude_superseded: req.exclude_superseded,
            search_profile: None,
        };
        let hits = self.inner.search(vanta_req).map_err(to_js_err)?;
        let arr = js_sys::Array::new();
        for hit in hits {
            arr.push(&Self::search_hit_to_js(hit));
        }
        Ok(arr.into())
    }

    /// Search nodes by raw vector without namespace scoping.
    ///
    /// Returns one `{node_id, distance}` entry per result (u128 ids as decimal strings).
    /// The `distance` field is a **lower-is-better** raw L2 / cosine distance, mirroring
    /// `VantaSearchHit.distance` in the Rust core. See `docs/api/WASM_API.md` for the
    /// full score-vs-distance convention across the 3 transports (WSM-10).
    pub fn search_vector(&self, vector: Vec<f32>, top_k: usize) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        if vector.len() > MAX_F32_VEC_LEN {
            return Err(to_js_err(VantaError::InvalidInput(format!(
                "vector length {} exceeds max {}",
                vector.len(),
                MAX_F32_VEC_LEN
            ))));
        }
        let hits = self
            .inner
            .search_vector(&vector, top_k.min(MAX_K))
            .map_err(to_js_err)?;
        let arr = js_sys::Array::new();
        for hit in hits {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"node_id".into(), &hit.node_id.to_string().into()).ok();
            js_sys::Reflect::set(&obj, &"distance".into(), &(hit.distance as f64).into()).ok();
            arr.push(&obj);
        }
        Ok(arr.into())
    }

    /// Run a search with explanation metadata for debugging scoring.
    pub fn explain_memory_search(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let req: SearchRequest = from_js(request)?;
        if req.query_vector.len() > MAX_F32_VEC_LEN {
            return Err(to_js_err(VantaError::InvalidInput(format!(
                "query vector length {} exceeds max {}",
                req.query_vector.len(),
                MAX_F32_VEC_LEN
            ))));
        }
        let distance = match req.distance_metric.as_str() {
            "Euclidean" => vantadb::DistanceMetric::Euclidean,
            _ => vantadb::DistanceMetric::Cosine,
        };
        let vanta_req = VantaMemorySearchRequest {
            namespace: req.namespace,
            query_vector: req.query_vector,
            query_sparse: None,
            #[allow(deprecated)]
            filters: req.filters,
            text_query: req.text_query,
            top_k: req.top_k.min(MAX_K),
            distance_metric: distance,
            explain: true,
            exclude_superseded: req.exclude_superseded,
            search_profile: None,
        };
        let explanation = self
            .inner
            .explain_memory_search(vanta_req)
            .map_err(to_js_err)?;
        to_js(&explanation)
    }

    /// Export all records in a namespace to a JSON file at the given path.
    pub fn export_namespace(&self, path: &str, namespace: &str) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let report = self
            .inner
            .export_namespace(path, namespace, None)
            .map_err(to_js_err)?;
        to_js(&report)
    }

    /// Export records in a namespace matching an AND-combined metadata filter
    /// (list of `{field, op, value}` items) to a JSON file at the given path.
    pub fn export_namespace_filtered(
        &self,
        path: &str,
        namespace: &str,
        filter: JsValue,
    ) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let filter: Vec<VantaMemoryFilterItem> = from_js(filter)?;
        let report = self
            .inner
            .export_namespace(path, namespace, Some(filter))
            .map_err(to_js_err)?;
        to_js(&report)
    }

    /// Delete all records in a namespace matching an AND-combined metadata
    /// filter (list of `{field, op, value}` items). Returns the number of
    /// deleted records.
    ///
    /// The core rejects an empty filter to prevent accidental full-namespace
    /// deletion — that error propagates to the caller unchanged.
    pub fn delete_by_filter(&self, namespace: &str, filter: JsValue) -> Result<u64, JsValue> {
        let _g = enter(&self.op_gate)?;
        let filter: Vec<VantaMemoryFilterItem> = from_js(filter)?;
        let deleted = self
            .inner
            .delete_by_filter(namespace, filter)
            .map_err(to_js_err)?;
        self.mark_cache_invalid();
        Ok(deleted)
    }

    /// Count records in a namespace, optionally matching an AND-combined
    /// metadata filter (list of `{field, op, value}` items). Pass an empty
    /// filter array / `null` to count every record in the namespace.
    pub fn count(&self, namespace: &str, filter: JsValue) -> Result<u64, JsValue> {
        let _g = enter(&self.op_gate)?;
        let filter: Vec<VantaMemoryFilterItem> = from_js(filter)?;
        let filter_opt = if filter.is_empty() {
            None
        } else {
            Some(filter)
        };
        self.inner.count(namespace, filter_opt).map_err(to_js_err)
    }

    /// Mark an existing record as superseded by another existing record.
    ///
    /// See `src/sdk/api.rs::supersede` for full semantics. Returns
    /// `Err` if either key is missing, `old_key == new_key`, or the old
    /// record is already superseded.
    pub fn supersede(&self, namespace: &str, old_key: &str, new_key: &str) -> Result<(), JsValue> {
        let _g = enter(&self.op_gate)?;
        self.inner
            .supersede(namespace, old_key, new_key)
            .map_err(to_js_err)?;
        self.mark_cache_invalid();
        Ok(())
    }

    /// Search namespace-scoped memory records by vector similarity from an
    /// existing key, without supplying a query vector. The source record
    /// itself is excluded from the results.
    pub fn similar_to_key(
        &self,
        namespace: &str,
        key: &str,
        top_k: usize,
    ) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let hits = self
            .inner
            .similar_to_key(namespace, key, top_k.min(MAX_K))
            .map_err(to_js_err)?;
        let arr = js_sys::Array::new();
        for hit in hits {
            arr.push(&Self::search_hit_to_js(hit));
        }
        Ok(arr.into())
    }

    /// Search across multiple namespaces in a single call. Results from
    /// each namespace are merged and sorted by descending score, capped
    /// at `request.top_k` globally.
    ///
    /// Accepts the same `SearchRequest` shape as `search()`; the
    /// `namespace` field on the request is ignored — pass `namespaces`
    /// instead.
    pub fn search_multi(&self, namespaces: JsValue, request: JsValue) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let ns_vec: Vec<String> = from_js(namespaces)?;
        let req: SearchRequest = from_js(request)?;
        if req.query_vector.len() > MAX_F32_VEC_LEN {
            return Err(to_js_err(VantaError::InvalidInput(format!(
                "query vector length {} exceeds max {}",
                req.query_vector.len(),
                MAX_F32_VEC_LEN
            ))));
        }
        let distance = match req.distance_metric.as_str() {
            "Euclidean" => vantadb::DistanceMetric::Euclidean,
            _ => vantadb::DistanceMetric::Cosine,
        };
        let vanta_req = VantaMemorySearchRequest {
            namespace: String::new(),
            query_vector: req.query_vector,
            #[allow(deprecated)]
            filters: req.filters,
            query_sparse: None,
            text_query: req.text_query,
            top_k: req.top_k.min(MAX_K),
            distance_metric: distance,
            explain: req.explain,
            exclude_superseded: req.exclude_superseded,
            search_profile: None,
        };
        let ns_refs: Vec<&str> = ns_vec.iter().map(String::as_str).collect();
        let hits = self
            .inner
            .search_multi(&ns_refs, vanta_req)
            .map_err(to_js_err)?;
        let arr = js_sys::Array::new();
        for hit in hits {
            arr.push(&Self::search_hit_to_js(hit));
        }
        Ok(arr.into())
    }

    /// Export all records across all namespaces to the given path.
    pub fn export_all(&self, path: &str) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let report = self.inner.export_all(path).map_err(to_js_err)?;
        to_js(&report)
    }

    /// Import records from a JS array of memory record objects.
    pub fn import_records(&self, records: JsValue) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let records: Vec<VantaMemoryRecord> = from_js(records)?;
        if records.len() > MAX_BATCH_SIZE {
            return Err(to_js_err(VantaError::InvalidInput(format!(
                "record batch size {} exceeds max {}",
                records.len(),
                MAX_BATCH_SIZE
            ))));
        }
        let report = self.inner.import_records(records).map_err(to_js_err)?;
        self.mark_cache_invalid();
        to_js(&report)
    }

    /// Import records from a JSON file at the given path.
    pub fn import_file(&self, path: &str) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let report = self.inner.import_file(path).map_err(to_js_err)?;
        self.mark_cache_invalid();
        to_js(&report)
    }

    /// Bulk-import records from a binary .vdbdump file.
    /// Returns a report object with total_records, batches_committed, duration_ms.
    pub fn bulk_import(&self, path: &str) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let report = self.inner.bulk_import_file(path).map_err(to_js_err)?;
        self.mark_cache_invalid();
        to_js(&report)
    }

    /// Bulk-import records from binary bytes (.vdbdump format).
    /// Accepts a Uint8Array and returns a report object.
    pub fn bulk_import_bytes(&self, data: &[u8]) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let mut cursor = std::io::Cursor::new(data);
        let report = self
            .inner
            .bulk_import_stream(&mut cursor)
            .map_err(to_js_err)?;
        self.mark_cache_invalid();
        to_js(&report)
    }

    /// Rebuild the HNSW index and return a rebuild report.
    pub fn rebuild_index(&self) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let report = self.inner.rebuild_index().map_err(to_js_err)?;
        to_js(&report)
    }

    /// Paginated HNSW rebuild from text records.
    ///
    /// Iterates through memory records in batches (max 1000) using the
    /// cursor-based list() API to prevent OOM on large namespaces.
    pub fn reindex_hnsw_from_text(
        &self,
        namespace: &str,
        page_size: Option<usize>,
    ) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let report = self
            .inner
            .reindex_hnsw_from_text(namespace, page_size)
            .map_err(to_js_err)?;
        self.mark_cache_invalid();
        to_js(&report)
    }

    /// Compact the storage layout and return the number of freed bytes.
    pub fn compact_layout(&self) -> Result<u64, JsValue> {
        let _g = enter(&self.op_gate)?;
        self.inner.compact_layout().map_err(to_js_err)
    }

    /// Run a text index consistency audit for an optional namespace.
    pub fn audit_text_index(&self, namespace: Option<String>) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let report = self
            .inner
            .audit_text_index(namespace.as_deref())
            .map_err(to_js_err)?;
        to_js(&report)
    }

    /// Run a deep text index consistency audit for an optional namespace.
    pub fn audit_text_index_deep(&self, namespace: Option<String>) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let report = self
            .inner
            .audit_text_index_deep(namespace.as_deref())
            .map_err(to_js_err)?;
        to_js(&report)
    }

    /// Repair the text index and return a repair report.
    pub fn repair_text_index(&self) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let report = self.inner.repair_text_index().map_err(to_js_err)?;
        to_js(&report)
    }

    /// Flush engine-internal buffers.
    ///
    /// NOTE (H-05): this is NOT a durability guarantee in the browser — the
    /// engine backend here may be purely in-memory. Persisted state only
    /// becomes durable after an explicit `save()` / `save_idb()` call.
    /// Emits a console warning when no persistent backend is attached.
    pub fn flush(&self) -> Result<(), JsValue> {
        let _g = enter(&self.op_gate)?;
        if self.opfs.is_none() {
            console_warn(
                "VantaDB.flush(): engine buffers flushed, but this is not a durability \
                 guarantee in the browser — call save() / save_idb() to persist.",
            );
        }
        self.inner.flush().map_err(to_js_err)
    }

    /// Compact the write-ahead log.
    pub fn compact_wal(&self) -> Result<(), JsValue> {
        let _g = enter(&self.op_gate)?;
        self.inner.compact_wal().map_err(to_js_err)
    }

    /// Purge all expired records and return the number removed.
    pub fn purge_expired(&self) -> Result<u64, JsValue> {
        let _g = enter(&self.op_gate)?;
        let removed = self.inner.purge_expired().map_err(to_js_err)?;
        if removed > 0 {
            self.mark_cache_invalid();
        }
        Ok(removed)
    }

    /// Return operational metrics as a JS object with stringified large numbers.
    pub fn operational_metrics(&self) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let metrics = self.inner.operational_metrics();
        let js: JsOperationalMetrics = metrics.into();
        to_js(&js)
    }

    /// Execute a raw DSL query string and return the result.
    pub fn query(&self, query: &str) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let result = self.inner.query(query).map_err(to_js_err)?;
        to_js(&result)
    }

    /// Insert a graph node with optional content, vector, and fields.
    pub fn insert_node(
        &self,
        id: &str,
        content: Option<String>,
        vector: Option<Vec<f32>>,
        fields: JsValue,
    ) -> Result<(), JsValue> {
        if let Some(ref v) = vector {
            if v.len() > MAX_F32_VEC_LEN {
                return Err(to_js_err(VantaError::InvalidInput(format!(
                    "vector length {} exceeds max {}",
                    v.len(),
                    MAX_F32_VEC_LEN
                ))));
            }
        }
        let fields: VantaFields = if fields.is_undefined() || fields.is_null() {
            VantaFields::new()
        } else {
            from_js(fields)?
        };
        let input = VantaNodeInput {
            id: parse_node_id(id)?,
            content,
            vector,
            fields,
        };
        let _g = enter(&self.op_gate)?;
        self.inner.insert_node(input).map_err(to_js_err)
    }

    /// Retrieve a graph node by its numeric ID (decimal string).
    pub fn get_node(&self, id: &str) -> Result<JsValue, JsValue> {
        let _g = enter(&self.op_gate)?;
        let node: Option<VantaNodeRecord> =
            self.inner.get_node(parse_node_id(id)?).map_err(to_js_err)?;
        let js: Option<JsNodeRecord> = node.map(Into::into);
        to_js(&js)
    }

    /// Delete a graph node by ID with an associated reason string.
    pub fn delete_node(&self, id: &str, reason: &str) -> Result<(), JsValue> {
        let _g = enter(&self.op_gate)?;
        self.inner
            .delete_node(parse_node_id(id)?, reason)
            .map_err(to_js_err)
    }

    /// Add a directed edge between two graph nodes with an optional weight
    /// and creation timestamp (Unix ms).
    pub fn add_edge(
        &self,
        source_id: &str,
        target_id: &str,
        label: &str,
        weight: Option<f32>,
        created_at_ms: Option<u64>,
    ) -> Result<(), JsValue> {
        let _g = enter(&self.op_gate)?;
        self.inner
            .add_edge(
                parse_node_id(source_id)?,
                parse_node_id(target_id)?,
                label,
                weight,
                created_at_ms,
            )
            .map_err(to_js_err)
    }

    /// Remove all edges between two graph nodes with the given label
    /// (both directions).
    pub fn remove_edge(
        &self,
        source_id: &str,
        target_id: &str,
        label: &str,
    ) -> Result<(), JsValue> {
        let _g = enter(&self.op_gate)?;
        self.inner
            .remove_edge(parse_node_id(source_id)?, parse_node_id(target_id)?, label)
            .map_err(to_js_err)?;
        self.mark_cache_invalid();
        Ok(())
    }

    /// Perform a breadth-first traversal from the given root node IDs.
    pub fn graph_bfs(
        &self,
        roots: Vec<String>,
        max_depth: usize,
        direction: String,
    ) -> Result<JsValue, JsValue> {
        let dir = match direction.as_str() {
            "Forward" => TraversalDirection::Forward,
            "Reverse" => TraversalDirection::Reverse,
            "Both" => TraversalDirection::Both,
            _ => {
                return Err(JsValue::from_str(&format!(
                    "invalid direction '{direction}': expected 'Forward', 'Reverse', or 'Both'"
                )))
            }
        };
        let roots: Vec<u128> = roots
            .into_iter()
            .map(|r| parse_node_id(&r))
            .collect::<Result<Vec<u128>, JsValue>>()?;
        let _g = enter(&self.op_gate)?;
        let result = self
            .inner
            .graph_bfs(&roots, max_depth, dir)
            .map_err(to_js_err)?;
        to_js(&result)
    }

    /// Perform a depth-first traversal from the given root node IDs.
    pub fn graph_dfs(
        &self,
        roots: Vec<String>,
        max_depth: usize,
        direction: String,
    ) -> Result<JsValue, JsValue> {
        let dir = match direction.as_str() {
            "Forward" => TraversalDirection::Forward,
            "Reverse" => TraversalDirection::Reverse,
            "Both" => TraversalDirection::Both,
            _ => {
                return Err(JsValue::from_str(&format!(
                    "invalid direction '{direction}': expected 'Forward', 'Reverse', or 'Both'"
                )))
            }
        };
        let roots: Vec<u128> = roots
            .into_iter()
            .map(|r| parse_node_id(&r))
            .collect::<Result<Vec<u128>, JsValue>>()?;
        let _g = enter(&self.op_gate)?;
        let result = self
            .inner
            .graph_dfs(&roots, max_depth, dir)
            .map_err(to_js_err)?;
        to_js(&result)
    }

    /// Compute a topological sort order starting from the given root node IDs.
    pub fn graph_topological_sort(&self, roots: Vec<String>) -> Result<JsValue, JsValue> {
        let roots: Vec<u128> = roots
            .into_iter()
            .map(|r| parse_node_id(&r))
            .collect::<Result<Vec<u128>, JsValue>>()?;
        let _g = enter(&self.op_gate)?;
        let result = self
            .inner
            .graph_topological_sort(&roots)
            .map_err(to_js_err)?;
        to_js(&result)
    }

    /// Return whether the subgraph reachable from the given roots forms a DAG.
    pub fn graph_is_dag(&self, roots: Vec<String>) -> Result<bool, JsValue> {
        let roots: Vec<u128> = roots
            .into_iter()
            .map(|r| parse_node_id(&r))
            .collect::<Result<Vec<u128>, JsValue>>()?;
        let _g = enter(&self.op_gate)?;
        self.inner.graph_is_dag(&roots).map_err(to_js_err)
    }

    /// Breadth-first traversal with optional edge label/time filtering
    /// (GRAFO-01).
    ///
    /// `filter` is `{labels?: number[], time_range?: [number, number]}` —
    /// only edges whose label id is in `labels` (and, when given, created
    /// inside the inclusive `time_range` window) are followed. `null`/
    /// `undefined` disables both filters. Delegates to the core
    /// `graph_bfs_filtered`; returns visited node ids in BFS order.
    pub fn graph_filtered_traversal(
        &self,
        roots: Vec<String>,
        max_depth: usize,
        direction: String,
        filter: JsValue,
    ) -> Result<JsValue, JsValue> {
        let dir = match direction.as_str() {
            "Forward" => TraversalDirection::Forward,
            "Reverse" => TraversalDirection::Reverse,
            "Both" => TraversalDirection::Both,
            _ => {
                return Err(JsValue::from_str(&format!(
                    "invalid direction '{direction}': expected 'Forward', 'Reverse', or 'Both'"
                )))
            }
        };
        let roots: Vec<u128> = roots
            .into_iter()
            .map(|r| parse_node_id(&r))
            .collect::<Result<Vec<u128>, JsValue>>()?;
        let filter: Option<GraphTraversalFilter> = if filter.is_null() || filter.is_undefined() {
            None
        } else {
            Some(from_js(filter)?)
        };
        let (labels, time_range) = filter.map(|f| (f.labels, f.time_range)).unwrap_or_default();
        let _g = enter(&self.op_gate)?;
        let result = self
            .inner
            .graph_bfs_filtered(&roots, max_depth, dir, &labels, time_range)
            .map_err(to_js_err)?;
        to_js(&result)
    }

    /// Degree centrality (in/out counts) for the subgraph reachable from the
    /// given root node IDs (GRAFO-01). Returns an array of
    /// `{id, in_degree, out_degree}` entries (u128 ids as strings).
    pub fn graph_degree(&self, roots: Vec<String>) -> Result<JsValue, JsValue> {
        let roots: Vec<u128> = roots
            .into_iter()
            .map(|r| parse_node_id(&r))
            .collect::<Result<Vec<u128>, JsValue>>()?;
        let _g = enter(&self.op_gate)?;
        let degrees = self
            .inner
            .graph_degree_centrality(&roots)
            .map_err(to_js_err)?;
        let entries: Vec<GraphDegreeEntry> = degrees
            .into_iter()
            .map(|(id, (in_degree, out_degree))| GraphDegreeEntry {
                id: id.to_string(),
                in_degree,
                out_degree,
            })
            .collect();
        to_js(&entries)
    }

    /// Generate a text snippet with optional highlighting for a given query.
    pub fn generate_snippet(
        &self,
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> Option<String> {
        let _g = enter(&self.op_gate).ok()?;
        self.inner
            .generate_snippet(payload, text_query, with_highlighting)
    }
}

static TRACING_INIT: AtomicBool = AtomicBool::new(false);

fn init() {
    if !TRACING_INIT.swap(true, Ordering::Relaxed) {
        console_error_panic_hook::set_once();
        #[cfg(feature = "tracing-wasm")]
        tracing_wasm::set_as_global_default();
    }
}

/// Optional label/time filter for filtered graph traversal (GRAFO-01).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct GraphTraversalFilter {
    /// Only follow edges whose label id is in this set. Empty = no label filter.
    labels: Vec<u32>,
    /// Inclusive (from_ms, to_ms) window on edge creation time. Absent = no filter.
    time_range: Option<(u64, u64)>,
}

/// One degree-centrality entry (GRAFO-01). u128 ids don't serialize as JSON
/// object keys in serde_wasm_bindgen, so the map becomes an array of entries.
#[derive(Debug, Clone, Serialize)]
struct GraphDegreeEntry {
    id: String,
    in_degree: usize,
    out_degree: usize,
}

/// Map a `VantaError` to a JS error carrying the `{code, message}` shape
/// shared across the TS/node/Python bindings (MOD-20 Python parity). The
/// standard `error.message` and a `code` property are attached via
/// `Reflect::set`; both reflect the underlying variant so consumers can
/// classify errors without parsing the message string. Backward-compatible:
/// `wrapWasmError` in `vantadb-ts/src/errors.ts` keeps reading `e.message`
/// and `(e as WasmErrorLike).code` exactly as before.
///
/// ERR-TS-01: the code comes straight from the core's `VantaError::code()`
/// (single source of truth) — the old 30→8 lookup table was removed once
/// `code()` became `pub`, so WASM now emits the canonical `VANTADB_*` codes.
fn to_js_err(e: VantaError) -> JsValue {
    let message = e.to_string();
    let err = js_sys::Error::new(&message);
    // Attach a structured code so TS consumers can classify errors without
    // parsing messages. Reflect::set on a fresh Error cannot fail in practice;
    // on failure we degrade gracefully to message-only classification.
    let _ = js_sys::Reflect::set(&err, &"code".into(), &JsValue::from_str(e.code()));
    // Mirror `message` as an own property so the JS-side shape is the symmetric
    // `{code, message}` documented for cross-SDK taxonomy. The standard
    // `error.message` remains the canonical source of truth.
    let _ = js_sys::Reflect::set(&err, &"message".into(), &JsValue::from_str(&message));
    err.into()
}

// ── CORE-02: graph-store persistence roundtrip (wasm-bindgen layer) ─────
//
// Contract: insert edge → IQL FROM returns it, across the same save/restore
// boundary the standalone Studio uses. Runs under `wasm-pack test --node`
// (no OPFS/IDB needed — the payload helpers are exercised directly).

#[cfg(all(test, target_arch = "wasm32"))]
mod core02_graph_persist_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    /// Nodes created via IQL (typed), edges via the binding graph API.
    fn seed_graph(db: &VantaDB) {
        db.query(r#"INSERT NODE#1 TYPE Person {}"#)
            .expect("insert#1");
        db.query(r#"INSERT NODE#2 TYPE Person {}"#)
            .expect("insert#2");
        db.add_edge("1", "2", "knows", Some(0.75), None)
            .expect("add_edge");
    }

    /// Read result helper: `{"Read": [nodeRecord...]}`.
    fn read_nodes(result: JsValue) -> Vec<(String, Vec<VantaEdgeRecord>)> {
        let arr = js_sys::Reflect::get(&result, &"Read".into()).expect("Read variant");
        let arr = js_sys::Array::from(&arr);
        let mut out = Vec::new();
        for i in 0..arr.length() {
            let rec = arr.get(i);
            let id = js_sys::Reflect::get(&rec, &"id".into())
                .unwrap()
                .as_string()
                .unwrap();
            // Edges come through as plain JS objects; pull label/target back.
            let edges_js = js_sys::Reflect::get(&rec, &"edges".into()).unwrap();
            let edges_arr = js_sys::Array::from(&edges_js);
            let mut edges = Vec::new();
            for j in 0..edges_arr.length() {
                let e = edges_arr.get(j);
                edges.push(VantaEdgeRecord {
                    target: js_sys::Reflect::get(&e, &"target".into())
                        .unwrap()
                        .as_string()
                        .unwrap()
                        .parse()
                        .unwrap(),
                    label: js_sys::Reflect::get(&e, &"label".into())
                        .unwrap()
                        .as_string()
                        .unwrap(),
                    weight: js_sys::Reflect::get(&e, &"weight".into())
                        .unwrap()
                        .as_f64()
                        .unwrap() as f32,
                    reverse: js_sys::Reflect::get(&e, &"reverse".into())
                        .unwrap()
                        .as_bool()
                        .unwrap_or(false),
                    created_at_ms: 0,
                });
            }
            out.push((id, edges));
        }
        out
    }

    #[wasm_bindgen_test]
    fn graph_roundtrip_through_snapshot_payload() {
        let db = VantaDB::new(None).expect("db");
        seed_graph(&db);

        // In-session: IQL FROM sees the edge (MCP-29 scan path).
        let nodes = read_nodes(db.query("SELECT * FROM Person").expect("query"));
        assert_eq!(nodes.len(), 2, "both persons visible in-session");
        let (_, alice_edges) = nodes.iter().find(|(id, _)| id == "1").expect("alice");
        assert!(
            alice_edges
                .iter()
                .any(|e| e.label == "knows" && e.target == 2),
            "edge visible to IQL in-session"
        );

        // Persistence boundary: snapshot → fresh engine → restore.
        let payload = db.graph_payload().expect("graph_payload");
        let db2 = VantaDB::new(None).expect("db2");
        db2.restore_graph_payload(&payload).expect("restore");

        let nodes2 = read_nodes(db2.query("SELECT * FROM Person").expect("query2"));
        assert_eq!(nodes2.len(), 2, "graph store must survive the snapshot");
        let (_, restored_edges) = nodes2.iter().find(|(id, _)| id == "1").expect("alice2");
        let edge = restored_edges
            .iter()
            .find(|e| e.label == "knows")
            .expect("edge survives restore");
        assert_eq!(edge.target, 2);
        assert!((edge.weight - 0.75).abs() < f32::EPSILON);
        assert!(!edge.reverse);

        // Directional traversal intact after restore (reverse flag preserved).
        let bfs = db2
            .graph_bfs(vec!["1".to_string()], 1, "Forward".to_string())
            .expect("bfs");
        let visited = js_sys::Array::from(&bfs);
        assert!(
            visited.length() >= 2,
            "traversal reaches bob after restore, got {}",
            visited.length()
        );
    }
}

// ── H-08: cursor policy tests (string-u64) ─────────────────────────────
//
// Contract: cursors cross the JS boundary as decimal strings, never f64,
// so values beyond 2^53 keep their exact digits. Runs under
// `wasm-pack test --node` (pure JS-value plumbing, no OPFS needed).
#[cfg(all(test, target_arch = "wasm32"))]
mod cursor_policy_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn next_cursor_serializes_as_decimal_string() {
        // 2^53 + 1 is not representable as f64 — the string must carry it.
        let big: u64 = (1u64 << 53) + 1;
        let v = next_cursor_to_js(Some(big));
        assert!(v.is_string(), "next_cursor must be a JS string");
        assert_eq!(
            v.as_string().as_deref(),
            Some("9007199254740993"),
            "exact digits must survive the boundary"
        );
    }

    #[wasm_bindgen_test]
    fn next_cursor_none_is_null() {
        let v = next_cursor_to_js(None);
        assert!(v.is_null());
    }

    #[wasm_bindgen_test]
    fn list_options_accepts_decimal_string_cursor() {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"cursor".into(), &JsValue::from_str("12345")).unwrap();
        let opts: ListOptions = from_js(obj.into()).expect("string cursor parses");
        assert_eq!(opts.cursor, Some(12345));
    }

    #[wasm_bindgen_test]
    fn list_options_accepts_numeric_cursor_back_compat() {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"cursor".into(), &JsValue::from_f64(7.0)).unwrap();
        let opts: ListOptions = from_js(obj.into()).expect("legacy numeric cursor parses");
        assert_eq!(opts.cursor, Some(7));
    }

    #[wasm_bindgen_test]
    fn list_options_rejects_garbage_cursor() {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"cursor".into(), &JsValue::from_str("not-a-number")).unwrap();
        assert!(from_js::<ListOptions>(obj.into()).is_err());
    }

    #[wasm_bindgen_test]
    fn flush_smoke_without_persistent_backend() {
        // H-05: flush stays callable with no OPFS attached; it must warn
        // (console side-effect not asserted here) and still return Ok.
        let db = VantaDB::new(None).expect("db");
        db.flush().expect("flush without persistent backend");
    }
}

// ── WSM-01: OPFS silent-fallback elimination + capabilities.persistence fiel ──
//
// Contract: `rg -n "\.ok\(\)" vantadb-wasm/src/lib.rs` around
// `OpfsStorage::open` is 0 after fix; `connect_persistent` rejects when
// `navigator.storage.getDirectory` fails (no silent in-memory fallback);
// `capabilities().persistence` is faithful (new/open → false,
// connect_persistent/connect_idb/connect_worker → true).
#[cfg(all(test, target_arch = "wasm32"))]
mod wsm01_persistence_tests {
    use super::*;
    use js_sys::Reflect;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn connect_persistent_opfs_failure_propagates() {
        let global = js_sys::global();
        let navigator = match Reflect::get(&global, &"navigator".into()) {
            Ok(v) if !v.is_undefined() => v,
            _ => return, // no navigator → skip (not a browser)
        };
        let storage = match Reflect::get(&navigator, &"storage".into()) {
            Ok(v) if !v.is_undefined() => v,
            _ => return, // no storage → skip, OpfsStorage::open already fails
        };
        let orig = Reflect::get(&storage, &"getDirectory".into()).unwrap_or(JsValue::UNDEFINED);
        // Stub getDirectory to reject — simulates OPFS unavailable / permission
        // denied / private mode.
        let stub = js_sys::Function::new_no_args(
            "return Promise.reject(new TypeError('OPFS blocked for WSM-01 test'));",
        );
        Reflect::set(&storage, &"getDirectory".into(), &stub).expect("set stub");
        let result = VantaDB::connect_persistent("wsm01_fail_test").await;
        // Restore before asserting so later tests are not poisoned.
        if orig.is_undefined() {
            let _ = Reflect::set(&storage, &"getDirectory".into(), &JsValue::UNDEFINED);
        } else {
            let _ = Reflect::set(&storage, &"getDirectory".into(), &orig);
        }
        assert!(
            result.is_err(),
            "connect_persistent must reject when OPFS getDirectory fails (no silent in-memory fallback)"
        );
        let err = result.err().expect("err");
        let msg = js_sys::Error::from(err)
            .message()
            .as_string()
            .unwrap_or_default();
        assert!(
            msg.contains("OPFS unavailable")
                || msg.contains("OPFS blocked")
                || msg.contains("getDirectory"),
            "error must be descriptive about OPFS failure, got: {msg}"
        );
    }

    #[wasm_bindgen_test]
    fn capabilities_persistence_fidelity_in_memory() {
        let db = VantaDB::new(None).expect("new");
        let caps_js = db.capabilities().expect("capabilities");
        let caps: serde_json::Value =
            serde_wasm_bindgen::from_value(caps_js).expect("deserialize caps");
        assert_eq!(
            caps["persistence"],
            serde_json::Value::Bool(false),
            "VantaDB::new in-memory must report persistence:false, got {caps}"
        );
        let db2 = VantaDB::open("wsm01_open_test").expect("open");
        let caps2_js = db2.capabilities().expect("caps2");
        let caps2: serde_json::Value =
            serde_wasm_bindgen::from_value(caps2_js).expect("deserialize caps2");
        assert_eq!(
            caps2["persistence"],
            serde_json::Value::Bool(false),
            "VantaDB::open in-memory must report persistence:false, got {caps2}"
        );
    }

    #[wasm_bindgen_test]
    async fn capabilities_persistence_fidelity_idb() {
        if !IdbStorage::is_available() {
            return;
        }
        let db = VantaDB::connect_idb("wsm01_idb_caps")
            .await
            .expect("connect_idb");
        let caps_js = db.capabilities().expect("caps");
        let caps: serde_json::Value =
            serde_wasm_bindgen::from_value(caps_js).expect("deserialize caps");
        assert_eq!(
            caps["persistence"],
            serde_json::Value::Bool(true),
            "VantaDB::connect_idb must report persistence:true, got {caps}"
        );
        // Cleanup to avoid leaking state across tests.
        let _ = db.delete_idb().await;
    }

    #[wasm_bindgen_test]
    async fn connect_persistent_success_reports_persistence_true() {
        // Only runs when OPFS is actually available in this browser/Node env.
        if !OpfsStorage::is_available() {
            return;
        }
        // Probe OPFS availability without stubbing — try to open a scratch dir.
        let probe = OpfsStorage::open("wsm01_probe").await;
        let _ = probe; // we only care that we don't poison global state
                       // If probe succeeded, the real connect should also succeed and report true.
                       // Use a fresh path to avoid probe leftovers.
        let path = "wsm01_persist_true_probe";
        let db = match VantaDB::connect_persistent(path).await {
            Ok(d) => d,
            Err(_) => return, // OPFS present but failed for this path — skip
        };
        let caps_js = db.capabilities().expect("caps");
        let caps: serde_json::Value =
            serde_wasm_bindgen::from_value(caps_js).expect("deserialize caps");
        assert_eq!(
            caps["persistence"],
            serde_json::Value::Bool(true),
            "VantaDB::connect_persistent success must report persistence:true, got {caps}"
        );
        // Cleanup OPFS file if any was created (load wrote nothing, but delete dir entry)
        // Best-effort: delete the test dir files via raw OpfsStorage if possible.
        if let Ok(s) = OpfsStorage::open(path).await {
            let _ = s.delete_file("db_state.json").await;
            let _ = s.delete_file("graph_state.json").await;
        }
    }
}

/// Parse a graph node id from a JS string.
///
/// Node ids are u128 in the core SDK; JS Numbers lose precision above 2^53 and
/// cannot represent ids above 2^64, so the WASM API takes ids as decimal
/// strings (strings in, strings out — matches ERR-025 MCP and ERR-023 Python).
fn parse_node_id(id: &str) -> Result<u128, JsValue> {
    id.parse::<u128>().map_err(|_| {
        to_js_err(VantaError::InvalidInput(format!(
            "invalid node id '{id}': expected a decimal u128 string"
        )))
    })
}

fn memory_record_to_js(rec: VantaMemoryRecord) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"namespace".into(), &rec.namespace.into()).ok();
    js_sys::Reflect::set(&obj, &"key".into(), &rec.key.into()).ok();
    js_sys::Reflect::set(&obj, &"payload".into(), &rec.payload.into()).ok();
    js_sys::Reflect::set(
        &obj,
        &"created_at_ms".into(),
        &rec.created_at_ms.to_string().into(),
    )
    .ok();
    js_sys::Reflect::set(
        &obj,
        &"updated_at_ms".into(),
        &rec.updated_at_ms.to_string().into(),
    )
    .ok();
    js_sys::Reflect::set(&obj, &"version".into(), &rec.version.to_string().into()).ok();
    js_sys::Reflect::set(&obj, &"node_id".into(), &rec.node_id.to_string().into()).ok();
    if let Some(ref vector) = rec.vector {
        // PERF-08 / P2-7: zero-copy vector. Sanitize NaN/Inf → 0.0
        // (JSON/JS cannot represent NaN or Infinity as f32), then bulk-copy
        // into a Float32Array instead of letting serde_wasm_bindgen build a
        // JS number[] element-by-element (one Reflect call + alloc per f32).
        // This is the hot path hit on every search/list/put record.
        //
        // WSM-12: count replaced elements in a single fetch_add to keep the
        // hot path branch-light. NaN/Inf from upstream embeddings is the bug
        // signal we want to surface via `operational_metrics()`.
        let mut sanitized_hits: u64 = 0;
        let sanitized: Vec<f32> = vector
            .iter()
            .map(|x| {
                if x.is_nan() || x.is_infinite() {
                    sanitized_hits += 1;
                    0.0
                } else {
                    *x
                }
            })
            .collect();
        if sanitized_hits > 0 {
            record_nan_sanitization(sanitized_hits);
        }
        let arr = js_sys::Float32Array::new_with_length(sanitized.len() as u32);
        arr.copy_from(&sanitized);
        js_sys::Reflect::set(&obj, &"vector".into(), &arr).ok();
    }
    if let Some(expires_at) = rec.expires_at_ms {
        js_sys::Reflect::set(
            &obj,
            &"expires_at_ms".into(),
            &expires_at.to_string().into(),
        )
        .ok();
    }
    if let Ok(meta) = serde_wasm_bindgen::to_value(&rec.metadata) {
        js_sys::Reflect::set(&obj, &"metadata".into(), &meta).ok();
    } else {
        // WSM-11: silent metadata error (serialization failure) was a
        // data-loss path. Bump the counter so JS callers can detect it via
        // operational_metrics(). Keeping the `metadata` field absent preserves
        // backward compat with existing TS glue that expects optional metadata.
        record_metadata_drop(1);
    }
    JsValue::from(&obj)
}

fn from_js<T: serde::de::DeserializeOwned>(val: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(val).map_err(|e| js_sys::Error::new(&e.to_string()).into())
}

fn to_js<T: serde::Serialize>(val: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(val).map_err(|e| js_sys::Error::new(&e.to_string()).into())
}
// ponytail: P2-7 RESUELTO por PERF-08 — memory_record_to_js emite `vector` como
// Float32Array zero-copy (js_sys::Float32Array + copy_from) en vez de
// serde_wasm_bindgen::to_value(&Vec<f32>). Mismo shape; evita N allocs/Reflect
// por vector en el hot path de search/list/put.
// Input zero-copy (from_js) queda pendiente: requiere tocar el parseo de
// MemoryInput/VantaNodeInput (fuera del scope de PERF-08).

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    // Tests use #[wasm_bindgen_test] which supports both sync and async
    // functions. Async tests run inside the browser's microtask queue;
    // wasm-bindgen-test handles the executor. These tests require a
    // browser environment — run with `wasm-pack test --chrome`.

    fn create_db() -> VantaDB {
        VantaDB::new(None).expect("failed to create VantaDB")
    }

    /// Serialize a serde value into a JS value with the JSON-compatible
    /// serializer (plain objects). serde-wasm-bindgen 0.6's default serializer
    /// turns maps into ES2015 `Map` instances, which `from_value` cannot read
    /// as struct fields ("missing field ..." errors).
    fn json_to_js<T: serde::Serialize>(value: &T) -> JsValue {
        serde::Serialize::serialize(value, &serde_wasm_bindgen::Serializer::json_compatible())
            .expect("json to js value")
    }

    #[wasm_bindgen_test]
    fn test_put_and_get() {
        let db = create_db();
        let input = json_to_js(&serde_json::json!({
            "namespace": "test",
            "key": "hello",
            "payload": "world"
        }));
        db.put(input).unwrap();
        let got = db.get("test", "hello").unwrap();
        assert!(!got.is_null());
    }

    // ── Basic CRUD Operations ──

    #[wasm_bindgen_test]
    fn test_get_nonexistent() {
        let db = create_db();
        let got = db.get("nosuch", "nonexistent").unwrap();
        assert!(got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_delete_record() {
        let db = create_db();
        let input = json_to_js(&serde_json::json!({
            "namespace": "test",
            "key": "todelete",
            "payload": "bye"
        }));
        db.put(input).unwrap();
        let deleted = db.delete("test", "todelete").unwrap();
        assert!(deleted);
        let got = db.get("test", "todelete").unwrap();
        assert!(got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_delete_nonexistent() {
        let db = create_db();
        let deleted = db.delete("test", "ghost").unwrap();
        assert!(!deleted);
    }

    #[wasm_bindgen_test]
    fn test_delete_by_filter_counts() {
        let db = create_db();
        for i in 0..3 {
            let input = json_to_js(&serde_json::json!({
                "namespace": "filterdel",
                "key": format!("hot_{i}"),
                "payload": "x",
                "metadata": { "tier": { "String": "hot" } }
            }));
            db.put(input).unwrap();
        }
        let keep = json_to_js(&serde_json::json!({
            "namespace": "filterdel",
            "key": "keep",
            "payload": "x",
            "metadata": { "tier": { "String": "cold" } }
        }));
        db.put(keep).unwrap();

        let filter = json_to_js(&serde_json::json!([
            { "field": "tier", "op": "Eq", "value": { "String": "hot" } }
        ]));
        let deleted = db.delete_by_filter("filterdel", filter).unwrap();
        assert_eq!(deleted, 3);
        // Non-matching record survives.
        assert!(!db.get("filterdel", "keep").unwrap().is_null());
    }

    #[wasm_bindgen_test]
    fn test_delete_by_filter_empty_rejected() {
        let db = create_db();
        let empty = json_to_js(&serde_json::json!([]));
        let err = db.delete_by_filter("filterdel", empty).unwrap_err();
        let msg = err
            .as_string()
            .or_else(|| js_sys::Error::from(err).message().as_string())
            .unwrap_or_default();
        assert!(
            msg.contains("requires at least one filter item"),
            "unexpected error: {msg}"
        );
    }

    #[wasm_bindgen_test]
    fn test_empty_vector_put() {
        let db = create_db();
        let input = json_to_js(&serde_json::json!({
            "namespace": "test",
            "key": "empty_vec",
            "payload": "no vector",
            "vector": []
        }));
        let record = db.put(input).unwrap();
        assert!(!record.is_null());
        let got = db.get("test", "empty_vec").unwrap();
        assert!(!got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_put_and_get_with_vector() {
        let db = create_db();
        let input = json_to_js(&serde_json::json!({
            "namespace": "test",
            "key": "vec_key",
            "payload": "vector data",
            "vector": [0.1, 0.2, 0.3, 0.4]
        }));
        db.put(input).unwrap();
        let got = db.get("test", "vec_key").unwrap();
        assert!(!got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_large_metadata() {
        let db = create_db();
        let mut meta = serde_json::Map::new();
        for i in 0..100 {
            meta.insert(
                format!("key_{}", i),
                serde_json::json!({ "String": format!("value_{}", i) }),
            );
        }
        let input_val = serde_json::json!({
            "namespace": "test",
            "key": "large_meta",
            "payload": "big metadata payload",
            "metadata": meta
        });
        let input = json_to_js(&input_val);
        db.put(input).unwrap();
        let got = db.get("test", "large_meta").unwrap();
        assert!(!got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_put_batch_empty() {
        let db = create_db();
        let items: Vec<serde_json::Value> = vec![];
        let batch = json_to_js(&items);
        let records = db.put_batch(batch).unwrap();
        assert!(records.is_array());
    }

    #[wasm_bindgen_test]
    fn test_put_batch_multiple() {
        let db = create_db();
        let items: Vec<serde_json::Value> = (0..10)
            .map(|i| {
                serde_json::json!({
                    "namespace": "batch",
                    "key": format!("item_{}", i),
                    "payload": format!("batch item {}", i),
                    "vector": [i as f32 * 0.1, 0.2, 0.3, 0.4]
                })
            })
            .collect();
        let batch = json_to_js(&items);
        db.put_batch(batch).unwrap();
        for i in 0..10 {
            let got = db.get("batch", &format!("item_{}", i)).unwrap();
            assert!(!got.is_null());
        }
    }

    // ── Batch & Concurrent Operations ──

    #[wasm_bindgen_test]
    fn test_concurrent_put_get() {
        let db = create_db();
        for i in 0..20 {
            let input = json_to_js(&serde_json::json!({
                "namespace": "concurrent",
                "key": format!("key_{}", i),
                "payload": format!("data {}", i),
                "vector": [i as f32 * 0.05, 0.1, 0.2, 0.3]
            }));
            db.put(input).unwrap();
            let got = db.get("concurrent", &format!("key_{}", i)).unwrap();
            assert!(!got.is_null());
        }
    }

    // ── Capabilities & Maintenance ──

    #[wasm_bindgen_test]
    fn test_capabilities() {
        let db = create_db();
        let caps = db.capabilities().unwrap();
        assert!(!caps.is_null());
    }

    #[wasm_bindgen_test]
    fn test_list_namespaces() {
        let db = create_db();
        let nss = db.list_namespaces().unwrap();
        assert!(nss.is_array());
    }

    #[wasm_bindgen_test]
    fn test_search_without_results() {
        let db = create_db();
        let input = json_to_js(&serde_json::json!({
            "namespace": "test",
            "key": "only_text",
            "payload": "some text content for text-only search"
        }));
        db.put(input).unwrap();
        let req = json_to_js(&serde_json::json!({
            "namespace": "test",
            "query_vector": [0.1, 0.2, 0.3, 0.4],
            "top_k": 5
        }));
        let hits = db.search(req).unwrap();
        assert!(hits.is_array() || hits.is_null());
    }

    #[wasm_bindgen_test]
    fn test_flush_and_compact() {
        let db = create_db();
        db.flush().unwrap();
        db.compact_wal().unwrap();
        let freed = db.compact_layout().unwrap();
        assert_eq!(freed, 0);
    }

    // ── WSM-11: metadata drop counter ──

    #[test]
    fn test_metadata_drop_counter_accumulates() {
        let baseline = METADATA_DROP_COUNT.load(Ordering::Relaxed);
        record_metadata_drop(1);
        record_metadata_drop(1);
        record_metadata_drop(1);
        let after = METADATA_DROP_COUNT.load(Ordering::Relaxed);
        assert_eq!(after, baseline + 3);
        // Counter is never reset (except by WASM instance lifetime); verify
        // monotonic accumulation rather than the exact value.
        assert!(after >= baseline + 3);
    }

    #[test]
    fn test_record_metadata_drop_zero_is_noop() {
        let before = METADATA_DROP_COUNT.load(Ordering::Relaxed);
        record_metadata_drop(0);
        assert_eq!(METADATA_DROP_COUNT.load(Ordering::Relaxed), before);
    }

    // ── WSM-12: NaN/Inf sanitization counter ──

    #[test]
    fn test_nan_sanitization_counter_accumulates() {
        // The counter is process-wide; capture the baseline so the assertion
        // doesn't flake when other tests in the same process bump it.
        let baseline = NAN_SANITIZATION_COUNT.load(Ordering::Relaxed);
        record_nan_sanitization(1);
        record_nan_sanitization(4);
        let after = NAN_SANITIZATION_COUNT.load(Ordering::Relaxed);
        assert_eq!(after, baseline + 5);
        // Counter is never reset (except by WASM instance lifetime); verify
        // monotonic accumulation rather than the exact value.
        assert!(after >= baseline + 5);
    }

    #[test]
    fn test_record_nan_sanitization_zero_is_noop() {
        let before = NAN_SANITIZATION_COUNT.load(Ordering::Relaxed);
        record_nan_sanitization(0);
        assert_eq!(NAN_SANITIZATION_COUNT.load(Ordering::Relaxed), before);
    }

    // ── ERR-024: u128 node id round-trip ──

    #[test]
    fn test_parse_node_id_accepts_u128_strings() {
        // Node ids are u128 in the core SDK. 2^64 is one past the old u64
        // ceiling and cannot be represented by a JS Number (max safe 2^53).
        // The helper converts to u128 exactly, then core maps it (js_sys error
        // path is browser-only, exercised by the wasm_bindgen_test below).
        let big: u128 = "18446744073709551616".parse().unwrap(); // 2^64
        assert_eq!(parse_node_id("18446744073709551616").unwrap(), big);
        assert_eq!(parse_node_id("0").unwrap(), 0);
        // Reject non-numeric, negative, and overflow inputs (> u128::MAX)
        assert!("not-a-number".parse::<u128>().is_err());
        assert!("-1".parse::<u128>().is_err());
        assert!("340282366920938463463374607431768211456"
            .parse::<u128>()
            .is_err()); // 2^128
    }

    #[wasm_bindgen_test]
    fn test_insert_get_node_u128_roundtrip() {
        // ERR-024: nodes with ids > 2^64 were previously truncated to u64 and
        // inaccessible via WASM. String ids must round-trip exactly.
        let db = create_db();
        let big_id = "18446744073709551616"; // 2^64
        db.insert_node(
            big_id,
            Some("big id node".to_string()),
            None,
            JsValue::UNDEFINED,
        )
        .expect("insert node with u128 id");
        let got = db.get_node(big_id).expect("get node by string id");
        let val: serde_json::Value =
            serde_wasm_bindgen::from_value(got).expect("deserialize node record");
        assert_eq!(
            val["id"],
            serde_json::Value::String(big_id.to_string()),
            "node id must round-trip exactly (old u64 binding would clamp to 0)"
        );
    }

    #[wasm_bindgen_test]
    fn test_collect_all_deduped_no_duplicates() {
        // AUD-043: dedup by (namespace, key) must never emit duplicate pairs.
        // Re-putting the same key overwrites the store copy, so the invariant
        // to prove is: the deduped collection has exactly the unique records
        // and no (namespace, key) pair appears twice.
        let db = create_db();
        for (ns, key) in [("ns_a", "k1"), ("ns_a", "k2"), ("ns_b", "k1")] {
            let input = json_to_js(&serde_json::json!({
                "namespace": ns,
                "key": key,
                "payload": "p"
            }));
            db.put(input).unwrap();
        }
        // Overwrite an existing key — must not create a second record.
        let dup = json_to_js(&serde_json::json!({
            "namespace": "ns_a",
            "key": "k1",
            "payload": "overwritten"
        }));
        db.put(dup).unwrap();

        let records = db.collect_all_deduped().unwrap();
        let mut seen = HashSet::new();
        for rec in &records {
            assert!(
                seen.insert((rec.namespace.clone(), rec.key.clone())),
                "duplicate (namespace, key) in collect_all_deduped output: {} {}",
                rec.namespace,
                rec.key
            );
        }
        assert_eq!(
            records.len(),
            3,
            "expected 3 unique records, got {}",
            records.len()
        );
    }
}
