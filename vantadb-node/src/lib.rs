//! Native Node.js bindings for VantaDB via napi-rs.
//!
//! Additional backend to the WASM browser build (`vantadb-wasm`): the native
//! `.node` module gives Node.js real filesystem persistence (fjall/WAL/fsync),
//! which WASM cannot provide. The exposed API is isomorphic with the WASM
//! wrapper (`vantadb-ts/src/vantadb.ts`) for the methods covered here.
//!
//! Design notes (mirrors `vantadb-python/src/lib.rs`):
//! - `VantaEmbedded` is `Clone`; every engine operation runs on a blocking
//!   thread via `tokio::task::spawn_blocking` with a cloned handle so the
//!   Node.js main thread is never blocked.
//! - I/O boundary is `serde_json::Value`: inputs are parsed manually (the SDK
//!   input structs have no `#[serde(default)]`), outputs are serialized with
//!   the existing `Serialize` derives.
//! - All `VantaError`s map to `napi::Error::from_reason` with the Display text.

use napi::Error;
use napi_derive::napi;
use serde_json::{json, Map, Value};

/// Cap on vector dimension accepted from Node to bound CPU/memory on the FFI
/// trust boundary. Mirrors the guard the WASM backend applies
/// (`vantadb-wasm/src/lib.rs` `MAX_F32_VEC_LEN`).
const MAX_VEC_DIM: usize = 10_000;
use vantadb::config::VantaConfig;
use vantadb::node::DistanceMetric;
use vantadb::sdk::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryMetadata,
    VantaMemorySearchRequest,
};

/// Native VantaDB handle exposed to Node.js. Thin wrapper over the SDK's
/// `VantaEmbedded`; all engine methods are async to avoid blocking the JS thread.
#[napi]
pub struct VantaDB {
    engine: VantaEmbedded,
}

#[napi]
impl VantaDB {
    /// Open (or create) a VantaDB database at `path`.
    ///
    /// `path` may be a filesystem directory (persistent, fjall backend) or
    /// `":memory:"` for a non-persistent in-memory database.
    ///
    /// `options` (optional): `{ read_only?: boolean, memory_limit?: number }`.
    #[napi(factory)]
    pub async fn connect(path: String, options: Option<Value>) -> napi::Result<VantaDB> {
        let config = build_config(&path, options.as_ref())?;
        let engine = spawn_blocking(move || VantaEmbedded::open_with_config(config)).await?;
        Ok(VantaDB { engine })
    }

    /// Flush the WAL and memory-mapped files to disk.
    #[napi]
    pub async fn flush(&self) -> napi::Result<()> {
        let engine = self.engine.clone();
        spawn_blocking(move || engine.flush()).await
    }

    /// Close the database handle. Pending writes are flushed first.
    #[napi]
    pub fn close(&self) -> napi::Result<()> {
        self.engine.close().map_err(map_err)
    }

    /// Insert or update a persistent memory record.
    ///
    /// `record`: `{ namespace, key, payload, metadata?, vector?, ttl_ms? }`.
    /// Returns the created/updated record with system timestamps and version.
    #[napi]
    pub async fn put(&self, record: Value) -> napi::Result<Value> {
        let input = parse_memory_input(&record)?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.put(input)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Insert or update multiple records. Returns the resulting records.
    #[napi]
    pub async fn put_batch(&self, records: Value) -> napi::Result<Value> {
        let arr = records
            .as_array()
            .ok_or_else(|| Error::from_reason("records must be an array"))?;
        let inputs = arr
            .iter()
            .map(parse_memory_input)
            .collect::<napi::Result<Vec<VantaMemoryInput>>>()?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.put_batch(inputs)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Retrieve a memory record by namespace and key. Returns `null` if absent.
    #[napi]
    pub async fn get(&self, namespace: String, key: String) -> napi::Result<Option<Value>> {
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.get(&namespace, &key)).await?;
        match out {
            Some(record) => Ok(Some(serde_json::to_value(&record).map_err(serde_map_err)?)),
            None => Ok(None),
        }
    }

    /// Delete a memory record by namespace and key. Returns `true` if a record
    /// was actually deleted, `false` if it did not exist.
    #[napi]
    pub async fn delete(&self, namespace: String, key: String) -> napi::Result<bool> {
        let engine = self.engine.clone();
        spawn_blocking(move || engine.delete(&namespace, &key)).await
    }

    /// List records in a namespace with optional filters and cursor pagination.
    ///
    /// `options` (optional): `{ filters?: VantaMetadata, limit?: number, cursor?: number }`.
    /// Returns `{ records: MemoryRecord[], next_cursor?: number }`.
    #[napi]
    pub async fn list(&self, namespace: String, options: Option<Value>) -> napi::Result<Value> {
        let opts = parse_list_options(options.as_ref())?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.list(&namespace, opts)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Return all namespaces that contain at least one memory record.
    #[napi]
    pub async fn list_namespaces(&self) -> napi::Result<Vec<String>> {
        let engine = self.engine.clone();
        spawn_blocking(move || engine.list_namespaces()).await
    }

    /// Search memory records by vector similarity (with optional filters and
    /// text query). Returns hits ordered by relevance: each hit is
    /// `{ record: MemoryRecord, score: number, explanation?: object }`.
    #[napi]
    pub async fn search(&self, request: Value) -> napi::Result<Value> {
        let request = parse_search_request(&request)?;
        let engine = self.engine.clone();
        let out: Vec<vantadb::sdk::VantaMemorySearchHit> =
            spawn_blocking(move || engine.search(request)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Return stable runtime capabilities.
    #[napi]
    pub fn capabilities(&self) -> Value {
        let caps = self.engine.capabilities();
        json!({
            "runtime_profile": runtime_profile_label(caps.runtime_profile),
            "persistence": caps.persistence,
            "vector_search": caps.vector_search,
            "iql_queries": caps.iql_queries,
            "read_only": caps.read_only,
        })
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn runtime_profile_label(profile: vantadb::sdk::VantaRuntimeProfile) -> &'static str {
    match profile {
        vantadb::sdk::VantaRuntimeProfile::Enterprise => "Enterprise",
        vantadb::sdk::VantaRuntimeProfile::Performance => "Performance",
        vantadb::sdk::VantaRuntimeProfile::LowResource => "LowResource",
    }
}

fn map_err(e: vantadb::error::VantaError) -> napi::Error {
    Error::from_reason(e.to_string())
}

fn serde_map_err(e: serde_json::Error) -> napi::Error {
    Error::from_reason(format!("serialization error: {e}"))
}

/// Run a blocking engine operation on the tokio blocking threadpool.
async fn spawn_blocking<F, T>(f: F) -> napi::Result<T>
where
    F: FnOnce() -> vantadb::error::Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| Error::from_reason(format!("blocking task failed: {e}")))?
        .map_err(map_err)
}

fn build_config(path: &str, options: Option<&Value>) -> napi::Result<VantaConfig> {
    let mut config = VantaConfig {
        storage_path: path.to_string(),
        ..Default::default()
    };
    if path.is_empty() || path == ":memory:" {
        // The SDK's `connect()` treats empty/`:memory:` as an in-memory engine;
        // `open_with_config` alone would try to open a real directory with that
        // name, so mirror the backend selection here.
        config.backend_kind = vantadb::storage::BackendKind::InMemory;
    }
    if let Some(opts) = options {
        let obj = opts
            .as_object()
            .ok_or_else(|| Error::from_reason("options must be an object"))?;
        if let Some(read_only) = obj.get("read_only") {
            config.read_only = read_only
                .as_bool()
                .ok_or_else(|| Error::from_reason("read_only must be a boolean"))?;
        }
        if let Some(limit) = obj.get("memory_limit") {
            let value = limit
                .as_u64()
                .ok_or_else(|| Error::from_reason("memory_limit must be a number"))?;
            config.memory_limit = (value > 0).then_some(value);
        }
    }
    Ok(config)
}

fn parse_memory_input(value: &Value) -> napi::Result<VantaMemoryInput> {
    let obj = value
        .as_object()
        .ok_or_else(|| Error::from_reason("record must be an object"))?;
    Ok(VantaMemoryInput {
        namespace: get_str(obj, "namespace")?,
        key: get_str(obj, "key")?,
        payload: get_str(obj, "payload")?,
        metadata: get_metadata(obj, "metadata")?,
        vector: get_opt_f32_vec(obj, "vector")?,
        ttl_ms: get_opt_u64(obj, "ttl_ms")?,
    })
}

fn parse_list_options(value: Option<&Value>) -> napi::Result<VantaMemoryListOptions> {
    let mut filters = VantaMemoryMetadata::new();
    let mut limit = 100usize;
    let mut cursor = None;
    if let Some(opts) = value {
        let obj = opts
            .as_object()
            .ok_or_else(|| Error::from_reason("options must be an object"))?;
        if let Some(f) = obj.get("filters") {
            filters = serde_json::from_value(f.clone())
                .map_err(|e| Error::from_reason(format!("invalid filters: {e}")))?;
        }
        if let Some(l) = obj.get("limit") {
            limit = l
                .as_u64()
                .ok_or_else(|| Error::from_reason("limit must be a number"))?
                as usize;
        }
        if let Some(c) = obj.get("cursor") {
            cursor = Some(
                c.as_u64()
                    .ok_or_else(|| Error::from_reason("cursor must be a number"))?
                    as usize,
            );
        }
    }
    Ok(VantaMemoryListOptions {
        #[allow(deprecated)]
        filters,
        filter_ops: None,
        limit,
        cursor,
    })
}

fn parse_search_request(value: &Value) -> napi::Result<VantaMemorySearchRequest> {
    let obj = value
        .as_object()
        .ok_or_else(|| Error::from_reason("request must be an object"))?;
    let query_vector = get_f32_vec(obj, "query_vector")?;
    Ok(VantaMemorySearchRequest {
        namespace: get_str(obj, "namespace")?,
        query_vector,
        filters: get_metadata(obj, "filters")?,
        text_query: get_opt_str(obj, "text_query")?,
        top_k: obj
            .get("top_k")
            .and_then(Value::as_u64)
            .unwrap_or(10)
            .min(10_000) as usize,
        distance_metric: match obj.get("distance_metric") {
            Some(Value::String(s)) if s == "Euclidean" || s == "euclidean" => {
                DistanceMetric::Euclidean
            }
            _ => DistanceMetric::Cosine,
        },
        explain: obj.get("explain").and_then(Value::as_bool).unwrap_or(false),
    })
}

fn get_str(obj: &Map<String, Value>, key: &str) -> napi::Result<String> {
    match obj.get(key) {
        Some(Value::String(s)) => Ok(s.clone()),
        Some(_) => Err(Error::from_reason(format!("`{key}` must be a string"))),
        None => Err(Error::from_reason(format!(
            "missing required field `{key}`"
        ))),
    }
}

fn get_opt_str(obj: &Map<String, Value>, key: &str) -> napi::Result<Option<String>> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(Error::from_reason(format!("`{key}` must be a string"))),
    }
}

fn get_opt_u64(obj: &Map<String, Value>, key: &str) -> napi::Result<Option<u64>> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => {
            if n.is_i64() || n.is_u64() {
                Ok(n.as_u64())
            } else if n.as_f64().is_some_and(|f| f.is_finite() && f.fract() == 0.0 && f >= 0.0) {
                Ok(n.as_u64())
            } else {
                Err(Error::from_reason(format!(
                    "`{key}` must be a non-negative integer"
                )))
            }
        }
        Some(_) => Err(Error::from_reason(format!("`{key}` must be a number"))),
    }
}

fn get_f32_vec(obj: &Map<String, Value>, key: &str) -> napi::Result<Vec<f32>> {
    match obj.get(key) {
        Some(Value::Array(items)) => {
            if items.len() > MAX_VEC_DIM {
                return Err(Error::from_reason(format!(
                    "`{key}` exceeds max vector dimension {MAX_VEC_DIM}"
                )));
            }
            items
                .iter()
                .map(|v| {
                    v.as_f64()
                        .filter(|f| f.is_finite())
                        .map(|f| f as f32)
                        .ok_or_else(|| Error::from_reason(format!("`{key}` must be a number[]")))
                })
                .collect()
        }
        Some(_) => Err(Error::from_reason(format!("`{key}` must be a number[]"))),
        None => Err(Error::from_reason(format!(
            "missing required field `{key}`"
        ))),
    }
}

fn get_opt_f32_vec(obj: &Map<String, Value>, key: &str) -> napi::Result<Option<Vec<f32>>> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => {
            if items.len() > MAX_VEC_DIM {
                return Err(Error::from_reason(format!(
                    "`{key}` exceeds max vector dimension {MAX_VEC_DIM}"
                )));
            }
            items
                .iter()
                .map(|v| {
                    v.as_f64()
                        .filter(|f| f.is_finite())
                        .map(|f| f as f32)
                        .ok_or_else(|| Error::from_reason(format!("`{key}` must be a number[]")))
                })
                .collect::<napi::Result<Vec<f32>>>()
                .map(Some)
        }
        Some(_) => Err(Error::from_reason(format!("`{key}` must be a number[]"))),
    }
}

fn get_metadata(obj: &Map<String, Value>, key: &str) -> napi::Result<VantaMemoryMetadata> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(VantaMemoryMetadata::new()),
        Some(v) => serde_json::from_value(v.clone())
            .map_err(|e| Error::from_reason(format!("invalid `{key}`: {e}"))),
    }
}
