#![allow(non_local_definitions)]

use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList};
use vantadb::sdk::{
    VantaCapabilities, VantaEmbedded, VantaExportReport, VantaImportReport,
    VantaIndexRebuildReport, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryRecord,
    VantaMemorySearchHit, VantaMemorySearchRequest, VantaNodeInput, VantaNodeRecord,
    VantaOpenOptions, VantaQueryResult, VantaRuntimeProfile, VantaStorageTier, VantaValue,
};

fn py_any_to_value(value: &PyAny) -> PyResult<VantaValue> {
    if value.is_none() {
        return Ok(VantaValue::Null);
    }
    if let Ok(boolean) = value.extract::<bool>() {
        return Ok(VantaValue::Bool(boolean));
    }
    if let Ok(string) = value.extract::<String>() {
        return Ok(VantaValue::String(string));
    }
    if let Ok(integer) = value.extract::<i64>() {
        return Ok(VantaValue::Int(integer));
    }
    if let Ok(float) = value.extract::<f64>() {
        return Ok(VantaValue::Float(float));
    }

    Err(PyTypeError::new_err(
        "Unsupported field value. Use str, int, float, bool, or None.",
    ))
}

fn set_python_value(dict: &PyDict, key: &str, value: &VantaValue) -> PyResult<()> {
    match value {
        VantaValue::String(value) => dict.set_item(key, value),
        VantaValue::Int(value) => dict.set_item(key, value),
        VantaValue::Float(value) => dict.set_item(key, value),
        VantaValue::Bool(value) => dict.set_item(key, value),
        VantaValue::Null => dict.set_item(key, dict.py().None()),
    }
}

fn runtime_profile_label(profile: VantaRuntimeProfile) -> &'static str {
    match profile {
        VantaRuntimeProfile::Enterprise => "ENTERPRISE",
        VantaRuntimeProfile::Performance => "PERFORMANCE",
        VantaRuntimeProfile::LowResource => "LOW_RESOURCE",
    }
}

fn tier_label(tier: VantaStorageTier) -> &'static str {
    match tier {
        VantaStorageTier::Hot => "Hot",
        VantaStorageTier::Cold => "Cold",
    }
}

/// Convert a stable SDK node into a Python dictionary for maximum interop
/// with the AI ecosystem (LangChain, LlamaIndex, etc.)
fn node_to_pydict(py: Python, node: &VantaNodeRecord) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("id", node.id)?;
    dict.set_item("confidence_score", node.confidence_score)?;
    dict.set_item("importance", node.importance)?;
    dict.set_item("hits", node.hits)?;
    dict.set_item("last_accessed", node.last_accessed)?;
    dict.set_item("epoch", node.epoch)?;
    dict.set_item("tier", tier_label(node.tier))?;
    dict.set_item("is_alive", node.is_alive)?;

    match &node.vector {
        Some(vector) => {
            dict.set_item("vector", vector.clone())?;
            dict.set_item("vector_dims", node.vector_dimensions)?;
        }
        None => {
            dict.set_item("vector", py.None())?;
            dict.set_item("vector_dims", node.vector_dimensions)?;
        }
    }

    let fields = PyDict::new(py);
    for (k, v) in &node.fields {
        set_python_value(fields, k, v)?;
    }
    dict.set_item("fields", fields)?;

    let edges = PyList::empty(py);
    for e in &node.edges {
        let edge_tuple = (e.target, e.label.as_str(), e.weight);
        edges.append(edge_tuple)?;
    }
    dict.set_item("edges", edges)?;

    Ok(dict.into())
}

/// Format a stable SDK query result into a JSON-like string for Python consumption.
fn format_query_result(result: &VantaQueryResult) -> String {
    match result {
        VantaQueryResult::Read(nodes) => {
            let summaries: Vec<String> = nodes
                .iter()
                .map(|n| {
                    format!(
                        "{{id: {}, tier: {:?}, confidence: {:.2}, hits: {}}}",
                        n.id, n.tier, n.confidence_score, n.hits
                    )
                })
                .collect();
            format!("[{}]", summaries.join(", "))
        }
        VantaQueryResult::Write {
            affected_nodes,
            message,
            node_id,
        } => {
            format!(
                "{{affected: {}, message: \"{}\", node_id: {:?}}}",
                affected_nodes, message, node_id
            )
        }
        VantaQueryResult::StaleContext { node_id } => {
            format!(
                "{{stale_context: {}, action: \"rehydration_required\"}}",
                node_id
            )
        }
    }
}

fn capabilities_to_pydict(py: Python, capabilities: &VantaCapabilities) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item(
        "profile",
        runtime_profile_label(capabilities.runtime_profile),
    )?;
    dict.set_item("read_only", capabilities.read_only)?;
    dict.set_item("persistence", capabilities.persistence)?;
    dict.set_item("vector_search", capabilities.vector_search)?;
    dict.set_item("iql_queries", capabilities.iql_queries)?;
    Ok(dict.into())
}

fn memory_record_to_pydict(py: Python, record: &VantaMemoryRecord) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("namespace", &record.namespace)?;
    dict.set_item("key", &record.key)?;
    dict.set_item("payload", &record.payload)?;
    dict.set_item("created_at_ms", record.created_at_ms)?;
    dict.set_item("updated_at_ms", record.updated_at_ms)?;
    dict.set_item("version", record.version)?;
    dict.set_item("node_id", record.node_id)?;

    match &record.vector {
        Some(vector) => dict.set_item("vector", vector.clone())?,
        None => dict.set_item("vector", py.None())?,
    }

    let metadata = PyDict::new(py);
    for (key, value) in &record.metadata {
        set_python_value(metadata, key, value)?;
    }
    dict.set_item("metadata", metadata)?;

    Ok(dict.into())
}

fn memory_hit_to_pydict(py: Python, hit: &VantaMemorySearchHit) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("score", hit.score)?;
    dict.set_item("record", memory_record_to_pydict(py, &hit.record)?)?;
    Ok(dict.into())
}

fn rebuild_report_to_pydict(py: Python, report: &VantaIndexRebuildReport) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("scanned_nodes", report.scanned_nodes)?;
    dict.set_item("indexed_vectors", report.indexed_vectors)?;
    dict.set_item("skipped_tombstones", report.skipped_tombstones)?;
    dict.set_item("duration_ms", report.duration_ms)?;
    dict.set_item("index_path", &report.index_path)?;
    dict.set_item("success", report.success)?;
    Ok(dict.into())
}

fn export_report_to_pydict(py: Python, report: &VantaExportReport) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("records_exported", report.records_exported)?;
    dict.set_item("namespaces", report.namespaces.clone())?;
    dict.set_item("path", &report.path)?;
    dict.set_item("duration_ms", report.duration_ms)?;
    Ok(dict.into())
}

fn import_report_to_pydict(py: Python, report: &VantaImportReport) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("inserted", report.inserted)?;
    dict.set_item("updated", report.updated)?;
    dict.set_item("skipped", report.skipped)?;
    dict.set_item("errors", report.errors)?;
    dict.set_item("duration_ms", report.duration_ms)?;
    Ok(dict.into())
}

fn py_dict_to_metadata(
    fields: Option<&PyDict>,
) -> PyResult<std::collections::BTreeMap<String, VantaValue>> {
    let mut metadata = std::collections::BTreeMap::new();
    if let Some(extra) = fields {
        for (key, value) in extra.iter() {
            let k: String = key.extract()?;
            metadata.insert(k, py_any_to_value(value)?);
        }
    }
    Ok(metadata)
}

/// VantaDB — The vector-graph database that thinks.
/// In-process Python binding via PyO3. Zero network overhead.
///
/// Usage:
///     import vantadb_py as vanta
///     db = vanta.VantaDB("./my_brain", memory_limit_bytes=256 * 1024 * 1024)
///     db.insert(1, "Hello world", [0.1] * 384)
///     node = db.get(1)
///     results = db.search([0.1] * 384, top_k=5)
///     db.flush()
#[pyclass]
pub struct VantaDB {
    engine: VantaEmbedded,
}

#[pymethods]
impl VantaDB {
    /// Create or open a VantaDB database.
    ///
    /// Args:
    ///     db_path: Path to the database directory.
    ///     memory_limit_bytes: Optional memory budget in bytes for the Rust engine.
    ///         Isolates the DB's memory from Python's heap. If None, uses hardware
    ///         detection or VANTADB_MEMORY_LIMIT env var.
    ///     read_only: If True, opens the DB in read-only mode. Safe for multi-process
    ///         access when another process holds the write lock.
    #[new]
    #[pyo3(signature = (db_path, memory_limit_bytes=None, read_only=false))]
    fn new(db_path: &str, memory_limit_bytes: Option<u64>, read_only: bool) -> PyResult<Self> {
        let options = VantaOpenOptions {
            memory_limit_bytes,
            read_only,
        };
        let engine = VantaEmbedded::open_with_options(db_path, options).map_err(|e| {
            PyRuntimeError::new_err(format!("VantaDB initialization error: {:?}", e))
        })?;

        Ok(VantaDB { engine })
    }

    /// Insert a node with content and an optional embedding vector.
    ///
    /// GIL Policy: HELD (operation is µs-fast, releasing GIL costs more).
    ///
    /// Args:
    ///     id: Unique node identifier (u64).
    ///     content: Text content stored as a relational field.
    ///     vector: Embedding vector (list of floats). Pass empty list for no vector.
    ///     fields: Optional dict of additional relational fields.
    #[pyo3(signature = (id, content, vector, fields=None))]
    fn insert(
        &self,
        id: u64,
        content: &str,
        vector: Vec<f32>,
        fields: Option<&PyDict>,
    ) -> PyResult<()> {
        let mut input = VantaNodeInput::new(id);
        input.content = Some(content.to_string());
        input.vector = (!vector.is_empty()).then_some(vector);

        if let Some(extra) = fields {
            for (key, value) in extra.iter() {
                let k: String = key.extract()?;
                input.fields.insert(k, py_any_to_value(value)?);
            }
        }

        self.engine
            .insert_node(input)
            .map_err(|e| PyRuntimeError::new_err(format!("Insert error: {:?}", e)))?;

        Ok(())
    }

    /// Put or update a namespace-scoped persistent memory record.
    #[pyo3(signature = (namespace, key, payload, metadata=None, vector=None))]
    fn put(
        &self,
        py: Python,
        namespace: &str,
        key: &str,
        payload: &str,
        metadata: Option<&PyDict>,
        vector: Option<Vec<f32>>,
    ) -> PyResult<PyObject> {
        let mut input = VantaMemoryInput::new(namespace, key, payload);
        input.metadata = py_dict_to_metadata(metadata)?;
        input.vector = vector.filter(|v| !v.is_empty());

        let record = self
            .engine
            .put(input)
            .map_err(|e| PyRuntimeError::new_err(format!("Put error: {:?}", e)))?;
        memory_record_to_pydict(py, &record)
    }

    /// Retrieve a namespace-scoped persistent memory record.
    fn get_memory(&self, py: Python, namespace: &str, key: &str) -> PyResult<Option<PyObject>> {
        match self.engine.get(namespace, key) {
            Ok(Some(record)) => Ok(Some(memory_record_to_pydict(py, &record)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(PyRuntimeError::new_err(format!(
                "Get memory error: {:?}",
                e
            ))),
        }
    }

    /// Delete a namespace-scoped persistent memory record.
    fn delete_memory(&self, namespace: &str, key: &str) -> PyResult<bool> {
        self.engine
            .delete(namespace, key)
            .map_err(|e| PyRuntimeError::new_err(format!("Delete memory error: {:?}", e)))
    }

    /// List namespace-scoped persistent memory records.
    #[pyo3(signature = (namespace, filters=None, limit=100, cursor=None))]
    fn list_memory(
        &self,
        py: Python,
        namespace: &str,
        filters: Option<&PyDict>,
        limit: usize,
        cursor: Option<usize>,
    ) -> PyResult<PyObject> {
        let page = self
            .engine
            .list(
                namespace,
                VantaMemoryListOptions {
                    filters: py_dict_to_metadata(filters)?,
                    limit,
                    cursor,
                },
            )
            .map_err(|e| PyRuntimeError::new_err(format!("List memory error: {:?}", e)))?;

        let dict = PyDict::new(py);
        let records = PyList::empty(py);
        for record in &page.records {
            records.append(memory_record_to_pydict(py, record)?)?;
        }
        dict.set_item("records", records)?;
        dict.set_item("next_cursor", page.next_cursor)?;
        Ok(dict.into())
    }

    /// Search namespace-scoped persistent memory records by vector + filters.
    #[pyo3(signature = (namespace, query_vector, filters=None, text_query=None, top_k=10))]
    fn search_memory(
        &self,
        py: Python,
        namespace: &str,
        query_vector: Vec<f32>,
        filters: Option<&PyDict>,
        text_query: Option<String>,
        top_k: usize,
    ) -> PyResult<Vec<PyObject>> {
        let request = VantaMemorySearchRequest {
            namespace: namespace.to_string(),
            query_vector,
            filters: py_dict_to_metadata(filters)?,
            text_query,
            top_k,
        };

        let hits = self
            .engine
            .search(request)
            .map_err(|e| PyRuntimeError::new_err(format!("Search memory error: {:?}", e)))?;

        hits.iter()
            .map(|hit| memory_hit_to_pydict(py, hit))
            .collect()
    }

    /// Rebuild ANN and derived memory indexes from canonical storage.
    fn rebuild_index(&self, py: Python) -> PyResult<PyObject> {
        let report = self
            .engine
            .rebuild_index()
            .map_err(|e| PyRuntimeError::new_err(format!("Rebuild index error: {:?}", e)))?;
        rebuild_report_to_pydict(py, &report)
    }

    /// Export one namespace as JSONL.
    fn export_namespace(&self, py: Python, path: &str, namespace: &str) -> PyResult<PyObject> {
        let report = self
            .engine
            .export_namespace(path, namespace)
            .map_err(|e| PyRuntimeError::new_err(format!("Export namespace error: {:?}", e)))?;
        export_report_to_pydict(py, &report)
    }

    /// Export all namespaces as JSONL.
    fn export_all(&self, py: Python, path: &str) -> PyResult<PyObject> {
        let report = self
            .engine
            .export_all(path)
            .map_err(|e| PyRuntimeError::new_err(format!("Export all error: {:?}", e)))?;
        export_report_to_pydict(py, &report)
    }

    /// Import records from a VantaDB memory JSONL export.
    fn import_file(&self, py: Python, path: &str) -> PyResult<PyObject> {
        let report = self
            .engine
            .import_file(path)
            .map_err(|e| PyRuntimeError::new_err(format!("Import file error: {:?}", e)))?;
        import_report_to_pydict(py, &report)
    }

    /// Retrieve a node by ID. Returns a dict or None.
    ///
    /// GIL Policy: HELD (µs read from L1 cache or pinned RocksDB).
    fn get(&self, py: Python, id: u64) -> PyResult<Option<PyObject>> {
        match self.engine.get_node(id) {
            Ok(Some(node)) => Ok(Some(node_to_pydict(py, &node)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(PyRuntimeError::new_err(format!("Get error: {:?}", e))),
        }
    }

    /// Delete a node by ID with an auditable reason (tombstone).
    ///
    /// GIL Policy: HELD (atomic batch write).
    #[pyo3(signature = (id, reason="manual deletion"))]
    fn delete(&self, id: u64, reason: &str) -> PyResult<()> {
        self.engine
            .delete_node(id, reason)
            .map_err(|e| PyRuntimeError::new_err(format!("Delete error: {:?}", e)))
    }

    /// K-NN vector search. Returns a list of (node_id, distance) tuples.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during HNSW traversal.
    ///
    /// Args:
    ///     vector: Query embedding vector.
    ///     top_k: Number of nearest neighbors to return.
    #[pyo3(signature = (vector, top_k=10))]
    fn search(&self, py: Python, vector: Vec<f32>, top_k: usize) -> PyResult<Vec<(u64, f32)>> {
        let engine = self.engine.clone();
        py.allow_threads(move || {
            engine
                .search_vector(&vector, top_k)
                .map(|hits| {
                    hits.into_iter()
                        .map(|hit| (hit.node_id, hit.distance))
                        .collect()
                })
                .map_err(|e| PyRuntimeError::new_err(format!("Search error: {:?}", e)))
        })
    }

    /// Execute an IQL or LISP query string. Returns a formatted result string.
    ///
    /// GIL Policy: RELEASED during Tokio execution — allows other Python
    /// threads to run while VantaDB processes the query.
    fn query(&self, py: Python, iql_query: &str) -> PyResult<String> {
        let engine = self.engine.clone();
        let query_str = iql_query.to_string();

        py.allow_threads(move || {
            engine
                .query(&query_str)
                .map(|result| format_query_result(&result))
                .map_err(|e| PyRuntimeError::new_err(format!("Query error: {:?}", e)))
        })
    }

    /// Flush WAL and HNSW index to disk for durability.
    ///
    /// GIL Policy: HELD (fast sync).
    fn flush(&self) -> PyResult<()> {
        self.engine
            .flush()
            .map_err(|e| PyRuntimeError::new_err(format!("Flush error: {:?}", e)))
    }

    /// Introspect the stable runtime capabilities exposed by the SDK boundary.
    fn capabilities(&self, py: Python) -> PyResult<PyObject> {
        let capabilities = self.engine.capabilities();
        capabilities_to_pydict(py, &capabilities)
    }

    /// Backward-compatible alias for the stable capabilities surface.
    fn hardware_profile(&self, py: Python) -> PyResult<PyObject> {
        self.capabilities(py)
    }

    /// Add a labeled edge between two nodes.
    ///
    /// Args:
    ///     source_id: Source node ID.
    ///     target_id: Target node ID.
    ///     label: Edge label (e.g., "belongs_to", "similar_to").
    ///     weight: Optional edge weight (default 1.0).
    #[pyo3(signature = (source_id, target_id, label, weight=None))]
    fn add_edge(
        &self,
        source_id: u64,
        target_id: u64,
        label: &str,
        weight: Option<f32>,
    ) -> PyResult<()> {
        self.engine
            .add_edge(source_id, target_id, label, weight)
            .map_err(|e| PyRuntimeError::new_err(format!("Insert edge error: {:?}", e)))
    }

    /// Flush and close the embedded engine handle.
    fn close(&self) -> PyResult<()> {
        self.engine
            .close()
            .map_err(|e| PyRuntimeError::new_err(format!("Close error: {:?}", e)))
    }

    /// String representation showing the stable runtime profile.
    fn __repr__(&self) -> String {
        let caps = self.engine.capabilities();
        format!(
            "VantaDB(profile={}, read_only={}, vector_search={}, persistence={})",
            runtime_profile_label(caps.runtime_profile),
            caps.read_only,
            caps.vector_search,
            caps.persistence
        )
    }
}

/// The Python module for VantaDB.
/// Usage: `import vantadb_py`
#[pymodule]
fn vantadb_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<VantaDB>()?;
    m.add("__version__", "0.1.0")?;
    Ok(())
}
