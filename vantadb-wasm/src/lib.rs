#![cfg_attr(target_arch = "wasm32", no_main)]
#![warn(missing_docs)]
//! WASM bindings for the VantaDB embedded vector database.
//!
//! This crate provides JavaScript-accessible types and functions via `wasm_bindgen`,
//! exposing VantaDB's core operations (put, get, delete, search, graph traversal, etc.)
//! to WebAssembly targets. It also includes an optional OPFS persistence layer and
//! a SIMD-accelerated cosine distance helper.

use core::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};
use vantadb::config::VantaConfig;
use vantadb::sdk::*;
use vantadb::BackendKind;
use vantadb::VantaError;
use wasm_bindgen::prelude::*;

mod opfs;
/// OPFS file handle abstraction wrapping a JS `FileSystemFileHandle`.
pub use opfs::OpfsFile;
/// OPFS-based storage for persisting VantaDB state in the browser.
pub use opfs::OpfsStorage;

mod idb;
/// IndexedDB-based storage for browsers without OPFS support.
pub use idb::IdbStorage;

#[cfg(feature = "opfs")]
pub mod worker;

const MAX_F32_VEC_LEN: usize = 10_000_000;
const MAX_BATCH_SIZE: usize = 100_000;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl_ms: Option<u64>,
}

/// Search request
#[derive(Serialize, Deserialize)]
struct SearchRequest {
    namespace: String,
    query_vector: Vec<f32>,
    #[serde(default)]
    filters: VantaMemoryMetadata,
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

#[derive(Serialize, Deserialize)]
struct ListOptions {
    #[serde(default)]
    filters: VantaMemoryMetadata,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    cursor: Option<usize>,
}

fn default_limit() -> usize {
    100
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
        }
    }
}

/// The main VantaDB handle exposed to JavaScript via `wasm_bindgen`.
#[wasm_bindgen]
pub struct VantaDB {
    inner: VantaEmbedded,
    opfs: Option<OpfsStorage>,
    #[cfg(feature = "opfs")]
    worker: Option<worker::OpfsWorkerProxy>,
}

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
            #[cfg(feature = "opfs")]
            worker: None,
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
            #[cfg(feature = "opfs")]
            worker: None,
        })
    }

    /// Open VantaDB with OPFS-based persistent storage in the browser.
    pub async fn connect_persistent(path: &str) -> Result<VantaDB, JsValue> {
        init();
        let opfs = OpfsStorage::open(path).await.ok();
        let wasm_cfg = WasmConfig {
            storage_path: path.to_string(),
            ..WasmConfig::default()
        };
        let config = build_config(wasm_cfg);
        let inner = VantaEmbedded::open_with_config(config).map_err(to_js_err)?;
        let db = VantaDB {
            inner,
            opfs,
            #[cfg(feature = "opfs")]
            worker: None,
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
            #[cfg(feature = "opfs")]
            worker: None,
        };
        db.load_idb().await?;
        Ok(db)
    }

    /// Open VantaDB with OPFS persistence via a dedicated Web Worker.
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
            worker: Some(worker_proxy),
        };
        // Load from worker-backed storage
        let data = db.worker_read("db_state.json").await?;
        if let Some(d) = data {
            let records: Vec<VantaMemoryRecord> = serde_json::from_slice(&d)
                .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
            if !records.is_empty() {
                db.inner.import_records(records).map_err(to_js_err)?;
            }
        }
        Ok(db)
    }

    /// Read a file from the worker-backed OPFS storage.
    #[cfg(feature = "opfs")]
    pub async fn worker_read(&self, path: &str) -> Result<Option<Vec<u8>>, JsValue> {
        match &self.worker {
            Some(w) => w.read(path).await,
            None => Err(JsValue::from_str("worker not initialized")),
        }
    }

    /// Write a file through the worker-backed OPFS storage.
    #[cfg(feature = "opfs")]
    pub async fn worker_write(&self, path: &str, data: Vec<u8>) -> Result<(), JsValue> {
        match &self.worker {
            Some(w) => w.write(path, &data).await,
            None => Err(JsValue::from_str("worker not initialized")),
        }
    }

    /// Delete a file through the worker-backed OPFS storage.
    #[cfg(feature = "opfs")]
    pub async fn worker_delete(&self, path: &str) -> Result<(), JsValue> {
        match &self.worker {
            Some(w) => w.delete(path).await,
            None => Err(JsValue::from_str("worker not initialized")),
        }
    }

    /// Collect all in-memory records deduplicated by (namespace, key).
    fn collect_all_deduped(&self) -> Result<Vec<VantaMemoryRecord>, JsValue> {
        let mut seen: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        let mut state: Vec<VantaMemoryRecord> = Vec::new();
        let namespaces: Vec<String> = self.inner.list_namespaces().map_err(to_js_err)?;
        for ns in &namespaces {
            let mut cursor: Option<usize> = None;
            loop {
                let opts = VantaMemoryListOptions {
                    filters: VantaMemoryMetadata::new(),
                    limit: 10_000,
                    cursor,
                };
                let page = self.inner.list(ns, opts).map_err(to_js_err)?;
                for record in page.records {
                    if seen.insert((record.namespace.clone(), record.key.clone())) {
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

    /// Persist all in-memory records to OPFS storage.
    pub async fn save(&self) -> Result<(), JsValue> {
        let opfs = match &self.opfs {
            Some(o) => o,
            None => return Ok(()),
        };
        let state = self.collect_all_deduped()?;
        let data = serde_json::to_vec(&state)
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
        opfs.write_file("db_state.json", &data).await
    }

    /// Persist all in-memory records to IndexedDB storage.
    pub async fn save_idb(&self) -> Result<(), JsValue> {
        let state = self.collect_all_deduped()?;
        let data = serde_json::to_vec(&state)
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
        IdbStorage::write_file("db_state.json", &data).await
    }

    /// Restore all records from IndexedDB storage into memory.
    pub async fn load_idb(&self) -> Result<(), JsValue> {
        let data = match IdbStorage::read_file("db_state.json").await? {
            Some(d) => d,
            None => return Ok(()),
        };
        let records: Vec<VantaMemoryRecord> = serde_json::from_slice(&data)
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;
        if !records.is_empty() {
            self.inner.import_records(records).map_err(to_js_err)?;
        }
        Ok(())
    }

    /// Delete persisted state from IndexedDB.
    pub async fn delete_idb(&self) -> Result<(), JsValue> {
        IdbStorage::delete_file("db_state.json").await
    }

    /// Restore all records from OPFS storage into memory.
    pub async fn load(&self) -> Result<(), JsValue> {
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
        if !records.is_empty() {
            self.inner.import_records(records).map_err(to_js_err)?;
        }
        Ok(())
    }

    /// Close the database and release underlying engine resources.
    /// After close, the VantaDB handle should not be used for further operations.
    /// This does NOT free the JS wrapper object — callers should drop references
    /// after close to allow WASM GC to reclaim the wrapper.
    pub fn close(&self) -> Result<(), JsValue> {
        self.inner.close().map_err(to_js_err)
    }

    /// Return the capabilities object describing supported features.
    pub fn capabilities(&self) -> Result<JsValue, JsValue> {
        let caps = self.inner.capabilities();
        to_js(&caps)
    }

    /// Insert or update a single memory record from a JS object.
    pub fn put(&self, input: JsValue) -> Result<JsValue, JsValue> {
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
            namespace: input.namespace,
            key: input.key,
            payload: input.payload,
            metadata: input.metadata,
            vector: input.vector,
            ttl_ms: input.ttl_ms,
        };
        let record = self.inner.put(vanta_input).map_err(to_js_err)?;
        Ok(memory_record_to_js(record))
    }

    /// Insert or update multiple memory records from a JS array.
    pub fn put_batch(&self, inputs: JsValue) -> Result<JsValue, JsValue> {
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
        let vanta_inputs: Vec<VantaMemoryInput> = inputs
            .into_iter()
            .map(|i| VantaMemoryInput {
                namespace: i.namespace,
                key: i.key,
                payload: i.payload,
                metadata: i.metadata,
                vector: i.vector,
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
        let record: Option<VantaMemoryRecord> =
            self.inner.get(namespace, key).map_err(to_js_err)?;
        match record {
            Some(rec) => Ok(memory_record_to_js(rec)),
            None => Ok(JsValue::null()),
        }
    }

    /// Delete a single record by namespace and key. Returns whether a record was deleted.
    pub fn delete(&self, namespace: &str, key: &str) -> Result<bool, JsValue> {
        self.inner.delete(namespace, key).map_err(to_js_err)
    }

    /// Return all namespaces as a JS array of strings.
    pub fn list_namespaces(&self) -> Result<JsValue, JsValue> {
        let nss = self.inner.list_namespaces().map_err(to_js_err)?;
        to_js(&nss)
    }

    /// List records in a namespace with optional filters, limit, and cursor pagination.
    pub fn list(&self, namespace: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let opts: ListOptions = from_js(options)?;
        let vanta_opts = VantaMemoryListOptions {
            filters: opts.filters,
            limit: opts.limit,
            cursor: opts.cursor,
        };
        let page = self.inner.list(namespace, vanta_opts).map_err(to_js_err)?;
        let obj = js_sys::Object::new();
        let arr = js_sys::Array::new();
        for rec in page.records {
            arr.push(&memory_record_to_js(rec));
        }
        js_sys::Reflect::set(&obj, &"records".into(), &arr).ok();
        if let Some(cursor) = page.next_cursor {
            js_sys::Reflect::set(&obj, &"next_cursor".into(), &(cursor as f64).into()).ok();
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
            }
            for term in &mut sanitized.bm25_terms {
                if term.contribution.is_nan() || term.contribution.is_infinite() {
                    term.contribution = 0.0;
                }
            }
            let expl_js: JsValue =
                serde_wasm_bindgen::to_value(&sanitized).expect("search explanation serialization");
            js_sys::Reflect::set(&obj, &"explanation".into(), &expl_js).ok();
        }
        obj.into()
    }

    /// Search memory records by vector similarity with optional filters and text query.
    pub fn search(&self, request: JsValue) -> Result<JsValue, JsValue> {
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
            filters: req.filters,
            text_query: req.text_query,
            top_k: req.top_k,
            distance_metric: distance,
            explain: req.explain,
        };
        let hits = self.inner.search(vanta_req).map_err(to_js_err)?;
        let arr = js_sys::Array::new();
        for hit in hits {
            arr.push(&Self::search_hit_to_js(hit));
        }
        Ok(arr.into())
    }

    /// Search nodes by raw vector without namespace scoping.
    pub fn search_vector(&self, vector: Vec<f32>, top_k: usize) -> Result<JsValue, JsValue> {
        if vector.len() > MAX_F32_VEC_LEN {
            return Err(to_js_err(VantaError::InvalidInput(format!(
                "vector length {} exceeds max {}",
                vector.len(),
                MAX_F32_VEC_LEN
            ))));
        }
        let hits = self
            .inner
            .search_vector(&vector, top_k)
            .map_err(to_js_err)?;
        let arr = js_sys::Array::new();
        for hit in hits {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"node_id".into(), &hit.node_id.to_string().into()).ok();
            js_sys::Reflect::set(&obj, &"score".into(), &(hit.distance as f64).into()).ok();
            arr.push(&obj);
        }
        Ok(arr.into())
    }

    /// Run a search with explanation metadata for debugging scoring.
    pub fn explain_memory_search(&self, request: JsValue) -> Result<JsValue, JsValue> {
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
            filters: req.filters,
            text_query: req.text_query,
            top_k: req.top_k,
            distance_metric: distance,
            explain: true,
        };
        let explanation = self
            .inner
            .explain_memory_search(vanta_req)
            .map_err(to_js_err)?;
        to_js(&explanation)
    }

    /// Export all records in a namespace to a JSON file at the given path.
    pub fn export_namespace(&self, path: &str, namespace: &str) -> Result<JsValue, JsValue> {
        let report = self
            .inner
            .export_namespace(path, namespace)
            .map_err(to_js_err)?;
        to_js(&report)
    }

    /// Export all records across all namespaces to the given path.
    pub fn export_all(&self, path: &str) -> Result<JsValue, JsValue> {
        let report = self.inner.export_all(path).map_err(to_js_err)?;
        to_js(&report)
    }

    /// Import records from a JS array of memory record objects.
    pub fn import_records(&self, records: JsValue) -> Result<JsValue, JsValue> {
        let records: Vec<VantaMemoryRecord> = from_js(records)?;
        if records.len() > MAX_BATCH_SIZE {
            return Err(to_js_err(VantaError::InvalidInput(format!(
                "record batch size {} exceeds max {}",
                records.len(),
                MAX_BATCH_SIZE
            ))));
        }
        let report = self.inner.import_records(records).map_err(to_js_err)?;
        to_js(&report)
    }

    /// Import records from a JSON file at the given path.
    pub fn import_file(&self, path: &str) -> Result<JsValue, JsValue> {
        let report = self.inner.import_file(path).map_err(to_js_err)?;
        to_js(&report)
    }

    /// Rebuild the HNSW index and return a rebuild report.
    pub fn rebuild_index(&self) -> Result<JsValue, JsValue> {
        let report = self.inner.rebuild_index().map_err(to_js_err)?;
        to_js(&report)
    }

    /// Compact the storage layout and return the number of freed bytes.
    pub fn compact_layout(&self) -> Result<u64, JsValue> {
        self.inner.compact_layout().map_err(to_js_err)
    }

    /// Run a text index consistency audit for an optional namespace.
    pub fn audit_text_index(&self, namespace: Option<String>) -> Result<JsValue, JsValue> {
        let report = self
            .inner
            .audit_text_index(namespace.as_deref())
            .map_err(to_js_err)?;
        to_js(&report)
    }

    /// Run a deep text index consistency audit for an optional namespace.
    pub fn audit_text_index_deep(&self, namespace: Option<String>) -> Result<JsValue, JsValue> {
        let report = self
            .inner
            .audit_text_index_deep(namespace.as_deref())
            .map_err(to_js_err)?;
        to_js(&report)
    }

    /// Repair the text index and return a repair report.
    pub fn repair_text_index(&self) -> Result<JsValue, JsValue> {
        let report = self.inner.repair_text_index().map_err(to_js_err)?;
        to_js(&report)
    }

    /// Flush all pending writes to disk.
    pub fn flush(&self) -> Result<(), JsValue> {
        self.inner.flush().map_err(to_js_err)
    }

    /// Compact the write-ahead log.
    pub fn compact_wal(&self) -> Result<(), JsValue> {
        self.inner.compact_wal().map_err(to_js_err)
    }

    /// Purge all expired records and return the number removed.
    pub fn purge_expired(&self) -> Result<u64, JsValue> {
        self.inner.purge_expired().map_err(to_js_err)
    }

    /// Return operational metrics as a JS object with stringified large numbers.
    pub fn operational_metrics(&self) -> Result<JsValue, JsValue> {
        let metrics = self.inner.operational_metrics();
        let js: JsOperationalMetrics = metrics.into();
        to_js(&js)
    }

    /// Execute a raw DSL query string and return the result.
    pub fn query(&self, query: &str) -> Result<JsValue, JsValue> {
        let result = self.inner.query(query).map_err(to_js_err)?;
        to_js(&result)
    }

    /// Insert a graph node with optional content, vector, and fields.
    pub fn insert_node(
        &self,
        id: u64,
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
            id: id.into(),
            content,
            vector,
            fields,
        };
        self.inner.insert_node(input).map_err(to_js_err)
    }

    /// Retrieve a graph node by its numeric ID.
    pub fn get_node(&self, id: u64) -> Result<JsValue, JsValue> {
        let node: Option<VantaNodeRecord> = self.inner.get_node(id.into()).map_err(to_js_err)?;
        let js: Option<JsNodeRecord> = node.map(Into::into);
        to_js(&js)
    }

    /// Delete a graph node by ID with an associated reason string.
    pub fn delete_node(&self, id: u64, reason: &str) -> Result<(), JsValue> {
        self.inner.delete_node(id.into(), reason).map_err(to_js_err)
    }

    /// Add a directed edge between two graph nodes with an optional weight.
    pub fn add_edge(
        &self,
        source_id: u64,
        target_id: u64,
        label: &str,
        weight: Option<f32>,
    ) -> Result<(), JsValue> {
        self.inner
            .add_edge(source_id.into(), target_id.into(), label, weight)
            .map_err(to_js_err)
    }

    /// Perform a breadth-first traversal from the given root node IDs.
    pub fn graph_bfs(&self, roots: Vec<u64>, max_depth: usize) -> Result<JsValue, JsValue> {
        let roots: Vec<u128> = roots.into_iter().map(|r| r.into()).collect();
        let result = self.inner.graph_bfs(&roots, max_depth).map_err(to_js_err)?;
        to_js(&result)
    }

    /// Perform a depth-first traversal from the given root node IDs.
    pub fn graph_dfs(&self, roots: Vec<u64>, max_depth: usize) -> Result<JsValue, JsValue> {
        let roots: Vec<u128> = roots.into_iter().map(|r| r.into()).collect();
        let result = self.inner.graph_dfs(&roots, max_depth).map_err(to_js_err)?;
        to_js(&result)
    }

    /// Compute a topological sort order starting from the given root node IDs.
    pub fn graph_topological_sort(&self, roots: Vec<u64>) -> Result<JsValue, JsValue> {
        let roots: Vec<u128> = roots.into_iter().map(|r| r.into()).collect();
        let result = self
            .inner
            .graph_topological_sort(&roots)
            .map_err(to_js_err)?;
        to_js(&result)
    }

    /// Return whether the subgraph reachable from the given roots forms a DAG.
    pub fn graph_is_dag(&self, roots: Vec<u64>) -> Result<bool, JsValue> {
        let roots: Vec<u128> = roots.into_iter().map(|r| r.into()).collect();
        self.inner.graph_is_dag(&roots).map_err(to_js_err)
    }

    /// Generate a text snippet with optional highlighting for a given query.
    pub fn generate_snippet(
        &self,
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> Option<String> {
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

fn to_js_err(e: VantaError) -> JsValue {
    js_sys::Error::new(&e.to_string()).into()
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
        // Sanitize NaN/Inf → 0.0: JSON/JS cannot represent NaN or Infinity as
        // f32 values, and serde_wasm_bindgen would throw on serialization.
        let sanitized: Vec<f32> = vector
            .iter()
            .map(|x| {
                if x.is_nan() || x.is_infinite() {
                    0.0
                } else {
                    *x
                }
            })
            .collect();
        let v: JsValue =
            serde_wasm_bindgen::to_value(&sanitized).expect("vector Vec<f32> serialization");
        js_sys::Reflect::set(&obj, &"vector".into(), &v).ok();
    }
    if let Some(expires_at) = rec.expires_at_ms {
        js_sys::Reflect::set(
            &obj,
            &"expires_at_ms".into(),
            &expires_at.to_string().into(),
        )
        .ok();
    }
    let meta: JsValue =
        serde_wasm_bindgen::to_value(&rec.metadata).expect("metadata serialization");
    js_sys::Reflect::set(&obj, &"metadata".into(), &meta).ok();
    JsValue::from(&obj)
}

fn from_js<T: serde::de::DeserializeOwned>(val: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(val).map_err(|e| js_sys::Error::new(&e.to_string()).into())
}

fn to_js<T: serde::Serialize>(val: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(val).map_err(|e| js_sys::Error::new(&e.to_string()).into())
}
// ponytail: P2-7 zero-copy path para Float32Array — ~2-5µs overhead por vector de 384/768 dims via serde.
// Implementar cuando profiling muestre que es bottleneck (vs HNSW search ~100µs-1ms).
// Output: memory_record_to_js → Float32Array::from(&sanitized[..]) en vez de serde_wasm_bindgen::to_value
// Input: from_js → extraer Float32Array directamente en vez de serde_wasm_bindgen::from_value

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

    #[wasm_bindgen_test]
    fn test_put_and_get() {
        let db = create_db();
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "key": "hello",
            "payload": "world"
        }))
        .unwrap();
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
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "key": "todelete",
            "payload": "bye"
        }))
        .unwrap();
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
    fn test_empty_vector_put() {
        let db = create_db();
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "key": "empty_vec",
            "payload": "no vector",
            "vector": []
        }))
        .unwrap();
        let record = db.put(input).unwrap();
        assert!(!record.is_null());
        let got = db.get("test", "empty_vec").unwrap();
        assert!(!got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_put_and_get_with_vector() {
        let db = create_db();
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "key": "vec_key",
            "payload": "vector data",
            "vector": [0.1, 0.2, 0.3, 0.4]
        }))
        .unwrap();
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
                serde_json::Value::String(format!("value_{}", i)),
            );
        }
        let input_val = serde_json::json!({
            "namespace": "test",
            "key": "large_meta",
            "payload": "big metadata payload",
            "metadata": meta
        });
        let input = serde_wasm_bindgen::to_value(&input_val).unwrap();
        db.put(input).unwrap();
        let got = db.get("test", "large_meta").unwrap();
        assert!(!got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_put_batch_empty() {
        let db = create_db();
        let items: Vec<serde_json::Value> = vec![];
        let batch = serde_wasm_bindgen::to_value(&items).unwrap();
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
        let batch = serde_wasm_bindgen::to_value(&items).unwrap();
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
            let input = serde_wasm_bindgen::to_value(&serde_json::json!({
                "namespace": "concurrent",
                "key": format!("key_{}", i),
                "payload": format!("data {}", i),
                "vector": [i as f32 * 0.05, 0.1, 0.2, 0.3]
            }))
            .unwrap();
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
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "key": "only_text",
            "payload": "some text content for text-only search"
        }))
        .unwrap();
        db.put(input).unwrap();
        let req = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "query_vector": [0.1, 0.2, 0.3, 0.4],
            "top_k": 5
        }))
        .unwrap();
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
}
