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
use std::sync::{Arc, Condvar, Mutex, PoisonError};

use vantadb::config::VantaConfig;
use vantadb::graph::TraversalDirection;
use vantadb::index::IndexType;
use vantadb::node::DistanceMetric;
use vantadb::sdk::{
    VantaEmbedded, VantaMemoryFilterItem, VantaMemoryInput, VantaMemoryListOptions,
    VantaMemoryMetadata, VantaMemorySearchRequest, VantaNodeInput, VantaSearchExplanation,
};
// FFI guards: single source of truth from core (WSM-09).
use vantadb::{MAX_K, MAX_VEC_DIM};

/// Clamp `top_k`/`k` to [`MAX_K`], warning when the caller requested more than
/// the cap. Mirrors `vantadb-python::clamp_top_k` (ERR-022).
fn clamp_top_k(requested: usize) -> usize {
    if requested > MAX_K {
        eprintln!(
            "vantadb-node: top_k={requested} exceeds MAX_K={MAX_K}; clamping to {MAX_K} (ERR-022)"
        );
    }
    requested.min(MAX_K)
}

/// Native VantaDB handle exposed to Node.js. Thin wrapper over the SDK's
/// `VantaEmbedded`; all engine methods are async to avoid blocking the JS thread.
#[napi]
pub struct VantaDB {
    engine: VantaEmbedded,
    op_gate: OpGate,
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
    pub async fn connect(
        path: String,
        // Types overwrite: https://napi.rs/docs/concepts/types-overwrite — the
        // public TS contract is a typed options object while runtime conversion
        // stays on serde_json::Value with manual validation.
        #[napi(ts_arg_type = "ConnectOptions")] options: Option<Value>,
    ) -> napi::Result<VantaDB> {
        let config = build_config(&path, options.as_ref())?;
        let engine = spawn_blocking(move || VantaEmbedded::open_with_config(config)).await?;
        Ok(VantaDB {
            engine,
            op_gate: OpGate::new(),
        })
    }

    /// Flush the WAL and memory-mapped files to disk.
    #[napi]
    pub async fn flush(&self) -> napi::Result<()> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.flush()).await
    }

    /// Close the database handle. Pending writes are flushed first.
    ///
    /// Once closing starts, new operations are rejected (`database is
    /// closing`). This waits for every in-flight operation to finish before
    /// flushing, so a fire-and-forget `put` whose `spawn_blocking` had not yet
    /// run can never write after `close()` returns and be silently lost on
    /// process exit.
    #[napi]
    pub async fn close(&self) -> napi::Result<()> {
        self.op_gate.drain();
        let engine = self.engine.clone();
        spawn_blocking(move || engine.close()).await
    }

    /// Insert or update a persistent memory record.
    ///
    /// `record`: `{ namespace, key, payload, metadata?, vector?, ttl_ms? }`.
    /// Returns the created/updated record with system timestamps and version.
    #[napi(ts_return_type = "Promise<MemoryRecord>")]
    pub async fn put(
        &self,
        #[napi(ts_arg_type = "MemoryInput")] record: Value,
    ) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let input = parse_memory_input(&record)?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.put(input)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Insert or update multiple records. Returns the resulting records.
    #[napi(ts_return_type = "Promise<MemoryRecord[]>")]
    pub async fn put_batch(
        &self,
        #[napi(ts_arg_type = "MemoryInput[]")] records: Value,
    ) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
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
    #[napi(ts_return_type = "Promise<MemoryRecord | null>")]
    pub async fn get(&self, namespace: String, key: String) -> napi::Result<Option<Value>> {
        let _g = enter(&self.op_gate)?;
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
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.delete(&namespace, &key)).await
    }

    /// List records in a namespace with optional filters and cursor pagination.
    ///
    /// `options` (optional): `{ filters?: VantaMetadata, limit?: number, cursor?: number }`.
    /// Returns `{ records: MemoryRecord[], next_cursor?: number }`.
    #[napi(ts_return_type = "Promise<MemoryListResult>")]
    pub async fn list(
        &self,
        namespace: String,
        #[napi(ts_arg_type = "MemoryListOptions")] options: Option<Value>,
    ) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let opts = parse_list_options(options.as_ref())?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.list(&namespace, opts)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Return all namespaces that contain at least one memory record.
    #[napi]
    pub async fn list_namespaces(&self) -> napi::Result<Vec<String>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.list_namespaces()).await
    }

/// Hybrid memory search with optional vector / text / filter inputs.
///
/// Returns hits ordered by **relevance** (highest score first): each hit is
/// `{ record: MemoryRecord, score: number, explanation?: object }`.
///
/// # Score semantics (WSM-10 / CODE-091 cross-binding convention)
///
/// The `score` field is a **relevance score** — it is *higher-is-better*, not
/// a raw distance. The exact value depends on the input mix and `distance_metric`:
///
/// | Input                 | `distance_metric` | `score` formula / range                         |
/// |-----------------------|-------------------|--------------------------------------------------|
/// | `query_vector` only   | `"Cosine"`        | `cosine_similarity` ∈ `[-1.0, 1.0]`             |
/// | `query_vector` only   | `"Euclidean"`     | `-distance²` then sqrt → `(-∞, 0.0]`            |
/// | `text_query` only     | (n/a)             | BM25 (positive, no fixed upper bound)           |
/// | `query_vector` + `text_query` | any       | RRF-fused (higher = more relevant across channels) |
///
/// This is the same convention as the Rust core (`VantaMemorySearchHit.score`,
/// pinned by `src/sdk/serialization/vector_types.rs::tests`) and the Python SDK.
/// It is **different** from the TypeScript wrapper `vantadb-ts`, which renames
/// the field to `distance` and inverts the semantics (lower = more similar) —
/// see `docs/api/TS_SDK.md` → "Distance vs Score (CODE-091)" for the full
/// cross-binding table.
#[napi(ts_return_type = "Promise<MemorySearchHit[]>")]
pub async fn search(
        &self,
        #[napi(ts_arg_type = "SearchRequest")] request: Value,
    ) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let request = parse_search_request(&request)?;
        let engine = self.engine.clone();
        let out: Vec<vantadb::sdk::VantaMemorySearchHit> =
            spawn_blocking(move || engine.search(request)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Return stable runtime capabilities.
    #[napi(ts_return_type = "Capabilities")]
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

    /// Insert or update a graph node directly.
    ///
    /// `input`: `{ id, content?, vector?, fields? }` — `id` is a decimal string
    /// (or number; strings are the safe form for ids above 2^53), `fields` is an
    /// object of tagged values (e.g. `{ "name": { "String": "Ada" } }`).
    #[napi]
    pub async fn insert_node(
        &self,
        #[napi(ts_arg_type = "GraphNodeInput")] input: Value,
    ) -> napi::Result<()> {
        let _g = enter(&self.op_gate)?;
        let input = parse_node_input(&input)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.insert_node(input)).await
    }

    /// Retrieve a graph node by id. Returns `null` if absent.
    #[napi(ts_return_type = "Promise<GraphNodeRecord | null>")]
    pub async fn get_node(&self, id: String) -> napi::Result<Option<Value>> {
        let _g = enter(&self.op_gate)?;
        let id = parse_node_id(&id)?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.get_node(id)).await?;
        match out {
            Some(node) => Ok(Some(serde_json::to_value(&node).map_err(serde_map_err)?)),
            None => Ok(None),
        }
    }

    /// Delete a graph node by id. The `reason` string is recorded for auditing.
    #[napi]
    pub async fn delete_node(&self, id: String, reason: String) -> napi::Result<()> {
        let _g = enter(&self.op_gate)?;
        let id = parse_node_id(&id)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.delete_node(id, &reason)).await
    }

    /// Add a directed edge between two graph nodes.
    ///
    /// Automatically creates the reverse edge on the target node, enabling
    /// bidirectional traversal queries. `weight` defaults to 1.0 and
    /// `created_at_ms` to the current time.
    #[napi]
    pub async fn add_edge(
        &self,
        source_id: String,
        target_id: String,
        label: String,
        weight: Option<f64>,
        created_at_ms: Option<f64>,
    ) -> napi::Result<()> {
        let _g = enter(&self.op_gate)?;
        let source_id = parse_node_id(&source_id)?;
        let target_id = parse_node_id(&target_id)?;
        let created_at_ms = opt_f64_to_u64(created_at_ms, "created_at_ms")?;
        let engine = self.engine.clone();
        spawn_blocking(move || {
            engine.add_edge(
                source_id,
                target_id,
                &label,
                weight.map(|w| w as f32),
                created_at_ms,
            )
        })
        .await
    }

    /// Remove all edges between two nodes with the given label (both directions).
    #[napi]
    pub async fn remove_edge(
        &self,
        source_id: String,
        target_id: String,
        label: String,
    ) -> napi::Result<()> {
        let _g = enter(&self.op_gate)?;
        let source_id = parse_node_id(&source_id)?;
        let target_id = parse_node_id(&target_id)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.remove_edge(source_id, target_id, &label)).await
    }

    /// Breadth-first traversal from the given root node ids up to `max_depth`.
    ///
    /// `direction` is `"Forward"`, `"Reverse"`, or `"Both"`. Returns visited
    /// node ids as decimal strings (u128 ids exceed JS safe integers).
    #[napi]
    pub async fn graph_bfs(
        &self,
        roots: Vec<String>,
        max_depth: u32,
        direction: String,
    ) -> napi::Result<Vec<String>> {
        let _g = enter(&self.op_gate)?;
        let roots = parse_node_ids(&roots)?;
        let dir = parse_direction(&direction)?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.graph_bfs(&roots, max_depth as usize, dir)).await?;
        Ok(out.iter().map(|id| id.to_string()).collect())
    }

    /// Depth-first traversal from the given root node ids up to `max_depth`.
    ///
    /// `direction` is `"Forward"`, `"Reverse"`, or `"Both"`. Returns visited
    /// node ids as decimal strings.
    #[napi]
    pub async fn graph_dfs(
        &self,
        roots: Vec<String>,
        max_depth: u32,
        direction: String,
    ) -> napi::Result<Vec<String>> {
        let _g = enter(&self.op_gate)?;
        let roots = parse_node_ids(&roots)?;
        let dir = parse_direction(&direction)?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.graph_dfs(&roots, max_depth as usize, dir)).await?;
        Ok(out.iter().map(|id| id.to_string()).collect())
    }

    /// Topological sort of the subgraph reachable from the given root ids.
    ///
    /// Errors if the subgraph contains a cycle. Returns node ids as decimal strings.
    #[napi]
    pub async fn graph_topological_sort(&self, roots: Vec<String>) -> napi::Result<Vec<String>> {
        let _g = enter(&self.op_gate)?;
        let roots = parse_node_ids(&roots)?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.graph_topological_sort(&roots)).await?;
        Ok(out.iter().map(|id| id.to_string()).collect())
    }

    /// Return whether the subgraph reachable from the given roots forms a DAG.
    #[napi]
    pub async fn graph_is_dag(&self, roots: Vec<String>) -> napi::Result<bool> {
        let _g = enter(&self.op_gate)?;
        let roots = parse_node_ids(&roots)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.graph_is_dag(&roots)).await
    }

    /// Breadth-first traversal with optional edge label/time filtering.
    ///
    /// `filter` is `{ labels?: number[], time_range?: [number, number] }` —
    /// only edges whose label id is in `labels` (and, when given, created
    /// inside the inclusive `time_range` window) are followed. `null`/`undefined`
    /// disables both filters. Returns visited node ids as decimal strings.
    #[napi]
    pub async fn graph_filtered_traversal(
        &self,
        roots: Vec<String>,
        max_depth: u32,
        direction: String,
        #[napi(ts_arg_type = "GraphFilterOptions")] filter: Option<Value>,
    ) -> napi::Result<Vec<String>> {
        let _g = enter(&self.op_gate)?;
        let roots = parse_node_ids(&roots)?;
        let dir = parse_direction(&direction)?;
        let (labels, time_range) = parse_graph_filter(filter.as_ref())?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || {
            engine.graph_bfs_filtered(&roots, max_depth as usize, dir, &labels, time_range)
        })
        .await?;
        Ok(out.iter().map(|id| id.to_string()).collect())
    }

    /// Degree centrality (in/out counts) for the subgraph reachable from the roots.
    ///
    /// Returns an array of `{ id, in_degree, out_degree }` entries (u128 ids as
    /// decimal strings).
    #[napi(ts_return_type = "Promise<DegreeEntry[]>")]
    pub async fn graph_degree(&self, roots: Vec<String>) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let roots = parse_node_ids(&roots)?;
        let engine = self.engine.clone();
        let degrees = spawn_blocking(move || engine.graph_degree_centrality(&roots)).await?;
        let entries = degrees
            .into_iter()
            .map(|(id, (in_degree, out_degree))| {
                json!({ "id": id.to_string(), "in_degree": in_degree, "out_degree": out_degree })
            })
            .collect::<Vec<_>>();
        Ok(Value::Array(entries))
    }

    /// Explain the search plan for a memory search request without executing it.
    ///
    /// Same request shape as `search()`. Returns
    /// `{ route, hits, fusion_report }` with a per-hit scoring breakdown.
    #[napi(ts_return_type = "Promise<SearchExplanation>")]
    pub async fn explain_search(
        &self,
        #[napi(ts_arg_type = "SearchRequest")] request: Value,
    ) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let request = parse_search_request(&request)?;
        let engine = self.engine.clone();
        let out: VantaSearchExplanation =
            spawn_blocking(move || engine.explain_memory_search(request)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    // ── Lifecycle (BND-10) ───────────────────────────────────────────────────

    /// List every retained version of a memory record, ascending (v1..vN).
    /// Empty if the key does not exist or has no history.
    #[napi(ts_return_type = "Promise<MemoryRecord[]>")]
    pub async fn versions(&self, namespace: String, key: String) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.versions(&namespace, &key)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Retrieve a specific historical version of a memory record.
    /// Returns `null` if the version does not exist.
    #[napi(ts_return_type = "Promise<MemoryRecord | null>")]
    pub async fn get_version(
        &self,
        namespace: String,
        key: String,
        version: f64,
    ) -> napi::Result<Option<Value>> {
        let _g = enter(&self.op_gate)?;
        let v = opt_f64_to_u64(Some(version), "version")?
            .ok_or_else(|| Error::from_reason("`version` must be a positive integer"))?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.get_version(&namespace, &key, v)).await?;
        match out {
            Some(record) => Ok(Some(serde_json::to_value(&record).map_err(serde_map_err)?)),
            None => Ok(None),
        }
    }

    /// Mark `old_key` as superseded by `new_key` (ADR-028). Both keys must
    /// exist and differ; `old_key` must not already be superseded.
    #[napi]
    pub async fn supersede(
        &self,
        namespace: String,
        old_key: String,
        new_key: String,
    ) -> napi::Result<()> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.supersede(&namespace, &old_key, &new_key)).await
    }

    /// Purge tombstoned nodes from the HNSW index. Returns counts and timing.
    #[napi(ts_return_type = "Promise<VacuumReport>")]
    pub async fn vacuum(&self) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        // MOD-10: VacuumReport does not derive Serialize; build the JSON object
        // explicitly (same field set as `src/storage/engine/mod.rs:VacuumReport`).
        let report = spawn_blocking(move || engine.vacuum()).await?;
        Ok(json!({
            "scanned_nodes": report.scanned_nodes,
            "removed_nodes": report.removed_nodes,
            "reclaimed_bytes": report.reclaimed_bytes,
            "duration_ms": report.duration_ms,
            "success": report.success,
        }))
    }

    /// Rebuild the HNSW vector index, derived indexes, text index, and scalar
    /// index from scratch. Returns scan/index/timing counts.
    #[napi(ts_return_type = "Promise<RebuildReport>")]
    pub async fn rebuild_index(&self) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.rebuild_index()).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Compact the vector store file (BFS grouping from the HNSW entry point).
    /// Returns estimated bytes reclaimed.
    #[napi]
    pub async fn compact_layout(&self) -> napi::Result<u64> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.compact_layout()).await
    }

    // ── Maintenance (BND-10) ─────────────────────────────────────────────────

    /// Compact the WAL: flush, archive the current WAL file, and start fresh.
    #[napi]
    pub async fn compact_wal(&self) -> napi::Result<()> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.compact_wal()).await
    }

    /// Scan every memory record and physically delete those whose TTL has
    /// expired. Returns the number of records purged.
    #[napi]
    pub async fn purge_expired(&self) -> napi::Result<u64> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.purge_expired()).await
    }

    // ── Search-advanced (BND-10) ─────────────────────────────────────────────

    /// Delete every record in `namespace` whose metadata matches the given
    /// filter. Filter must contain at least one item (mirrors the SDK guard).
    #[napi]
    pub async fn delete_by_filter(
        &self,
        namespace: String,
        #[napi(ts_arg_type = "FilterItem[]")] filter: Value,
    ) -> napi::Result<u64> {
        let _g = enter(&self.op_gate)?;
        let items = parse_filter_items(&filter)?;
        let engine = self.engine.clone();
        spawn_blocking(move || engine.delete_by_filter(&namespace, items)).await
    }

    /// Count records in a namespace, optionally filtered by metadata.
    /// Pass `null`/`undefined` to count every record.
    #[napi]
    pub async fn count(
        &self,
        namespace: String,
        #[napi(ts_arg_type = "FilterItem[] | null")] filter: Option<Value>,
    ) -> napi::Result<u64> {
        let _g = enter(&self.op_gate)?;
        let items = match filter {
            None | Some(Value::Null) => None,
            Some(v) => Some(parse_filter_items(&v)?),
        };
        let engine = self.engine.clone();
        spawn_blocking(move || engine.count(&namespace, items)).await
    }

    /// Find records similar to the vector of an existing record (identified
    /// by `namespace` + `key`). The source record is filtered out of the
    /// results.
    #[napi(ts_return_type = "Promise<MemorySearchHit[]>")]
    pub async fn similar_to_key(
        &self,
        namespace: String,
        key: String,
        top_k: f64,
    ) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let k = clamp_top_k(
            opt_f64_to_u64(Some(top_k), "top_k")?
                .ok_or_else(|| Error::from_reason("`top_k` must be a positive integer"))?
                as usize,
        );
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.similar_to_key(&namespace, &key, k)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Same as `search()` with an explicit dense-vector index backend override.
    /// `method` accepts `"Hnsw" | "Ivf" | "Flat" | "DiskAnn" | "Scann"` (or
    /// `null`/`undefined` for automatic routing).
    #[napi(ts_return_type = "Promise<MemorySearchHit[]>")]
    pub async fn search_with_method(
        &self,
        #[napi(ts_arg_type = "SearchRequest")] request: Value,
        method: Option<String>,
    ) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let request = parse_search_request(&request)?;
        let method_idx = match method.as_deref() {
            None => None,
            Some(s) => Some(parse_index_method(s)?),
        };
        let engine = self.engine.clone();
        let out = spawn_blocking(move || engine.search_with_method(request, method_idx)).await?;
        serde_json::to_value(&out).map_err(serde_map_err)
    }

    /// Search across multiple namespaces in a single call. The `namespace`
    /// field on `request` is ignored; every namespace in `namespaces` is
    /// searched independently and results are merged by descending score.
    #[napi(ts_return_type = "Promise<MemorySearchHit[]>")]
    pub async fn search_multi(
        &self,
        #[napi(ts_arg_type = "string[]")] namespaces: Vec<String>,
        #[napi(ts_arg_type = "SearchRequest")] request: Value,
    ) -> napi::Result<Value> {
        let _g = enter(&self.op_gate)?;
        let request = parse_search_request(&request)?;
        let engine = self.engine.clone();
        let owned: Vec<String> = namespaces;
        let out = spawn_blocking(move || {
            let refs: Vec<&str> = owned.iter().map(String::as_str).collect();
            engine.search_multi(&refs, request)
        })
        .await?;
        serde_json::to_value(&out).map_err(serde_map_err)
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

/// Durability gate: rejects new operations once `close()` has begun and keeps
/// `close()` waiting until every in-flight operation finishes. Closes the
/// race where an async op whose `spawn_blocking` had not yet run would write
/// after `close()` returned — silently lost on process exit.
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
    /// `count == 0`. Blocks the calling thread; the `MutexGuard` is dropped on
    /// return so it never crosses an `.await` (a raw `MutexGuard` is not
    /// `Send`, and this future must be `Send` to run on the napi Tokio
    /// runtime). Acceptable to be blocking: this is the durability barrier.
    fn drain(&self) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap_or_else(PoisonError::into_inner);
        state.closing = true;
        while state.count > 0 {
            state = cvar.wait(state).unwrap_or_else(PoisonError::into_inner);
        }
        // `state` (the MutexGuard) is dropped here, before any await.
    }
}

/// RAII guard that decrements the in-flight count and wakes `close()` when
/// dropped (at the end of the owning async method, after the op completes).
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
fn enter(gate: &OpGate) -> napi::Result<OpGuard> {
    gate.try_enter()
        .ok_or_else(|| Error::from_reason("database is closing"))
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
        sparse_vector: None,
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
        exclude_superseded: false,
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
        query_sparse: None,
        filters: get_metadata(obj, "filters")?,
        text_query: get_opt_str(obj, "text_query")?,
        top_k: clamp_top_k(
            obj.get("top_k")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize,
        ),
        distance_metric: match obj.get("distance_metric") {
            Some(Value::String(s)) if s == "Euclidean" || s == "euclidean" => {
                DistanceMetric::Euclidean
            }
            _ => DistanceMetric::Cosine,
        },
        explain: obj.get("explain").and_then(Value::as_bool).unwrap_or(false),
        exclude_superseded: false,
        search_profile: None,
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
            // Integer form: reject negatives explicitly instead of silently
            // mapping them to "no TTL" (mirrors `opt_f64_to_u64` below).
            if n.is_i64() || n.is_u64() {
                n.as_u64().map(Some).ok_or_else(|| {
                    Error::from_reason(format!("`{key}` must be a non-negative integer"))
                })
            } else {
                // Float form (e.g. `5.0`): accept exact non-negative integers.
                match n.as_f64() {
                    Some(f) if f.is_finite() && f >= 0.0 && f.fract() == 0.0 => Ok(Some(f as u64)),
                    _ => Err(Error::from_reason(format!(
                        "`{key}` must be a non-negative integer"
                    ))),
                }
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

/// Convert an optional JS number to a u64, rejecting non-integers and negatives
/// (napi does not accept `u64`/`Option<u64>` params, so unix-ms values arrive
/// as `f64` and are validated here).
fn opt_f64_to_u64(value: Option<f64>, key: &str) -> napi::Result<Option<u64>> {
    value
        .map(|v| {
            if v.is_finite() && v >= 0.0 && v.fract() == 0.0 {
                Ok(v as u64)
            } else {
                Err(Error::from_reason(format!(
                    "`{key}` must be a non-negative integer"
                )))
            }
        })
        .transpose()
}

// ── graph helpers ────────────────────────────────────────────────────────────

/// Parse a graph node id from a decimal string.
///
/// Node ids are u128 in the core SDK; JS Numbers lose precision above 2^53, so
/// the Node API takes ids as decimal strings (strings in, strings out — same
/// convention as the WASM binding `parse_node_id` and ERR-025/ERR-023).
fn parse_node_id(id: &str) -> napi::Result<u128> {
    id.trim().parse::<u128>().map_err(|_| {
        Error::from_reason(format!(
            "invalid node id '{id}': expected a decimal u128 string"
        ))
    })
}

fn parse_node_ids(ids: &[String]) -> napi::Result<Vec<u128>> {
    ids.iter().map(|id| parse_node_id(id)).collect()
}

/// Parse a traversal direction string into the core enum.
fn parse_direction(direction: &str) -> napi::Result<TraversalDirection> {
    match direction {
        "Forward" => Ok(TraversalDirection::Forward),
        "Reverse" => Ok(TraversalDirection::Reverse),
        "Both" => Ok(TraversalDirection::Both),
        _ => Err(Error::from_reason(format!(
            "invalid direction '{direction}': expected 'Forward', 'Reverse', or 'Both'"
        ))),
    }
}

/// Parse a `VantaNodeInput` from a JSON object `{ id, content?, vector?, fields? }`.
///
/// `id` may be a decimal string or a number. `fields` uses the tagged
/// `VantaValue` representation (e.g. `{ "name": { "String": "Ada" } }`).
fn parse_node_input(value: &Value) -> napi::Result<VantaNodeInput> {
    let obj = value
        .as_object()
        .ok_or_else(|| Error::from_reason("node input must be an object"))?;
    let id = match obj.get("id") {
        Some(Value::String(s)) => parse_node_id(s)?,
        Some(Value::Number(n)) => n
            .as_u64()
            .ok_or_else(|| Error::from_reason("`id` must be a non-negative integer"))?
            as u128,
        Some(_) => return Err(Error::from_reason("`id` must be a string or number")),
        None => return Err(Error::from_reason("missing required field `id`")),
    };
    Ok(VantaNodeInput {
        id,
        content: get_opt_str(obj, "content")?,
        vector: get_opt_f32_vec(obj, "vector")?,
        fields: match obj.get("fields") {
            None | Some(Value::Null) => Default::default(),
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| Error::from_reason(format!("invalid `fields`: {e}")))?,
        },
    })
}

/// Parsed graph traversal filter: label ids plus optional inclusive time window.
type GraphFilter = (Vec<u32>, Option<(u64, u64)>);

/// Parse the optional `{ labels?, time_range? }` filter of
/// `graph_filtered_traversal`. `labels` is an array of label ids; `time_range`
/// is an inclusive `[from_ms, to_ms]` window on edge creation time.
fn parse_graph_filter(filter: Option<&Value>) -> napi::Result<GraphFilter> {
    let Some(filter) = filter else {
        return Ok((Vec::new(), None));
    };
    let obj = filter
        .as_object()
        .ok_or_else(|| Error::from_reason("filter must be an object"))?;
    let labels = match obj.get("labels") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|v| {
                v.as_u64()
                    .and_then(|n| u32::try_from(n).ok())
                    .ok_or_else(|| {
                        Error::from_reason("`labels` must be an array of non-negative integers")
                    })
            })
            .collect::<napi::Result<Vec<u32>>>()?,
        Some(_) => return Err(Error::from_reason("`labels` must be an array")),
    };
    let time_range = match obj.get("time_range") {
        None | Some(Value::Null) => None,
        Some(Value::Array(items)) if items.len() == 2 => {
            let from = items[0].as_u64().ok_or_else(|| {
                Error::from_reason("`time_range` must be [from_ms, to_ms] of integers")
            })?;
            let to = items[1].as_u64().ok_or_else(|| {
                Error::from_reason("`time_range` must be [from_ms, to_ms] of integers")
            })?;
            Some((from, to))
        }
        Some(_) => {
            return Err(Error::from_reason(
                "`time_range` must be [from_ms, to_ms] of integers",
            ))
        }
    };
    Ok((labels, time_range))
}

// ── filter & index-method helpers (BND-10) ──────────────────────────────────

/// Parse a JSON array of `VantaMemoryFilterItem` objects.
///
/// Wire shape: `[{ field: string, op: "Eq"|"Neq"|"Gt"|"Lt"|"Gte"|"Lte",
/// value: VantaValue }]`. The array must contain at least one item —
/// `delete_by_filter` rejects empty filters at the SDK layer, so we mirror
/// the guard here for an early, descriptive error instead of a generic
/// "empty filter" string from deep inside the engine.
fn parse_filter_items(value: &Value) -> napi::Result<Vec<VantaMemoryFilterItem>> {
    let arr = value
        .as_array()
        .ok_or_else(|| Error::from_reason("filter must be an array of filter items"))?;
    if arr.is_empty() {
        return Err(Error::from_reason(
            "filter must contain at least one item (use count() with no filter to count all records)",
        ));
    }
    let mut out = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::from_reason(format!("filter[{i}] must be an object")))?;
        let field = obj
            .get("field")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::from_reason(format!("filter[{i}].field must be a string")))?
            .to_string();
        let op_str = obj
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::from_reason(format!("filter[{i}].op must be a string")))?;
        let op = match op_str {
            "Eq" => vantadb::sdk::VantaFilterOp::Eq,
            "Neq" => vantadb::sdk::VantaFilterOp::Neq,
            "Gt" => vantadb::sdk::VantaFilterOp::Gt,
            "Lt" => vantadb::sdk::VantaFilterOp::Lt,
            "Gte" => vantadb::sdk::VantaFilterOp::Gte,
            "Lte" => vantadb::sdk::VantaFilterOp::Lte,
            other => {
                return Err(Error::from_reason(format!(
                    "filter[{i}].op '{other}' is not one of Eq|Neq|Gt|Lt|Gte|Lte"
                )));
            }
        };
        let value_json = obj
            .get("value")
            .ok_or_else(|| Error::from_reason(format!("filter[{i}].value is required")))?;
        let vanta_value: vantadb::sdk::VantaValue = serde_json::from_value(value_json.clone())
            .map_err(|e| Error::from_reason(format!("filter[{i}].value: {e}")))?;
        out.push(VantaMemoryFilterItem {
            field,
            op,
            value: vanta_value,
        });
    }
    Ok(out)
}

/// Parse the optional dense-vector index backend override for
/// `search_with_method`. Accepts the canonical PascalCase names from
/// `vantadb::index::IndexType`.
fn parse_index_method(method: &str) -> napi::Result<IndexType> {
    match method {
        "Hnsw" => Ok(IndexType::Hnsw),
        "Ivf" => Ok(IndexType::Ivf),
        "Flat" => Ok(IndexType::Flat),
        "DiskAnn" => Ok(IndexType::DiskAnn),
        "Scann" => Ok(IndexType::Scann),
        other => Err(Error::from_reason(format!(
            "invalid method '{other}': expected one of Hnsw|Ivf|Flat|DiskAnn|Scann"
        ))),
    }
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Mirrors the JSON shape the TypeScript side emits for `VantaValue`. We
    /// don't load the `.node` binary in these tests — they exercise the pure
    /// serde_json -> VantaMemoryFilterItem conversion path that napi-rs would
    /// otherwise route through `Python::with_gil`. Keeping them Rust-native
    /// means `cargo test -p vantadb-node` reports ≥1 PASS without needing the
    /// Node runtime to be installed.
    #[test]
    fn parse_filter_items_accepts_eq_filter() {
        let raw = json!([{
            "field": "env",
            "op": "Eq",
            "value": { "String": "prod" },
        }]);
        let out = parse_filter_items(&raw).expect("filter parses");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].field, "env");
        assert!(matches!(out[0].op, vantadb::sdk::VantaFilterOp::Eq));
        assert!(matches!(
            out[0].value,
            vantadb::sdk::VantaValue::String(ref s) if s == "prod"
        ));
    }

    #[test]
    fn parse_filter_items_rejects_empty_array() {
        let raw = json!([]);
        let err = parse_filter_items(&raw).unwrap_err();
        assert!(
            err.reason.contains("at least one item"),
            "expected empty-filter guard, got: {}",
            err.reason
        );
    }

    #[test]
    fn parse_index_method_maps_all_backends() {
        assert!(matches!(parse_index_method("Hnsw"), Ok(IndexType::Hnsw)));
        assert!(matches!(parse_index_method("Ivf"), Ok(IndexType::Ivf)));
        assert!(matches!(parse_index_method("Flat"), Ok(IndexType::Flat)));
        assert!(matches!(
            parse_index_method("DiskAnn"),
            Ok(IndexType::DiskAnn)
        ));
        assert!(matches!(parse_index_method("Scann"), Ok(IndexType::Scann)));
        assert!(parse_index_method("bogus").is_err());
    }
}
