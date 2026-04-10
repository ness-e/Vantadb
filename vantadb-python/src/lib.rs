use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::sync::Arc;
use vantadb::executor::{ExecutionResult, Executor};
use vantadb::node::{FieldValue, UnifiedNode, VectorRepresentations};
use vantadb::storage::{EngineConfig, StorageEngine};

/// Convert a UnifiedNode into a Python dictionary for maximum interop
/// with the AI ecosystem (LangChain, LlamaIndex, etc.)
fn node_to_pydict(py: Python, node: &UnifiedNode) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("id", node.id)?;
    dict.set_item("trust_score", node.trust_score)?;
    dict.set_item("importance", node.importance)?;
    dict.set_item("hits", node.hits)?;
    dict.set_item("last_accessed", node.last_accessed)?;
    dict.set_item("epoch", node.epoch)?;
    dict.set_item("tier", format!("{:?}", node.tier))?;
    dict.set_item("is_alive", node.is_alive())?;

    // Vector as Python list (zero-copy would need numpy, but this is safe)
    match &node.vector {
        VectorRepresentations::Full(v) => {
            dict.set_item("vector", v.clone())?;
            dict.set_item("vector_dims", v.len())?;
        }
        VectorRepresentations::None => {
            dict.set_item("vector", py.None())?;
            dict.set_item("vector_dims", 0u32)?;
        }
        _ => {
            dict.set_item("vector", py.None())?;
            dict.set_item("vector_dims", node.vector.dimensions())?;
        }
    }

    // Relational fields → nested dict
    let fields = PyDict::new(py);
    for (k, v) in &node.relational {
        match v {
            FieldValue::String(s) => fields.set_item(k, s)?,
            FieldValue::Int(i) => fields.set_item(k, i)?,
            FieldValue::Float(f) => fields.set_item(k, f)?,
            FieldValue::Bool(b) => fields.set_item(k, b)?,
            FieldValue::Null => fields.set_item(k, py.None())?,
        }
    }
    dict.set_item("fields", fields)?;

    // Edges → list of (target, label, weight) tuples
    let edges = PyList::empty(py);
    for e in &node.edges {
        let edge_tuple = (e.target, e.label.as_str(), e.weight);
        edges.append(edge_tuple)?;
    }
    dict.set_item("edges", edges)?;

    Ok(dict.into())
}

/// Format an ExecutionResult into a JSON-like string for Python consumption.
fn format_execution_result(result: &ExecutionResult) -> String {
    match result {
        ExecutionResult::Read(nodes) => {
            let summaries: Vec<String> = nodes
                .iter()
                .map(|n| {
                    format!(
                        "{{id: {}, type: {:?}, trust: {:.2}, hits: {}}}",
                        n.id, n.tier, n.trust_score, n.hits
                    )
                })
                .collect();
            format!("[{}]", summaries.join(", "))
        }
        ExecutionResult::Write {
            affected_nodes,
            message,
            node_id,
        } => {
            format!(
                "{{affected: {}, message: \"{}\", node_id: {:?}}}",
                affected_nodes, message, node_id
            )
        }
        ExecutionResult::StaleContext(id) => {
            format!(
                "{{stale_context: {}, action: \"rehydration_required\"}}",
                id
            )
        }
    }
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
    engine: Arc<StorageEngine>,
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
        let config = EngineConfig {
            memory_limit: memory_limit_bytes,
            force_mmap: false, // Let HardwareScout decide
            read_only,
        };

        let engine = StorageEngine::open_with_config(db_path, Some(config)).map_err(|e| {
            PyRuntimeError::new_err(format!("VantaDB initialization error: {:?}", e))
        })?;

        Ok(VantaDB {
            engine: Arc::new(engine),
        })
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
        let mut node = UnifiedNode::new(id);
        node.relational.insert(
            "content".to_string(),
            FieldValue::String(content.to_string()),
        );

        if !vector.is_empty() {
            node.vector = VectorRepresentations::Full(vector);
            node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
        }

        // Inject additional fields from Python dict
        if let Some(extra) = fields {
            for (key, value) in extra.iter() {
                let k: String = key.extract()?;
                if let Ok(b) = value.extract::<bool>() {
                    node.relational.insert(k, FieldValue::Bool(b));
                } else if let Ok(s) = value.extract::<String>() {
                    node.relational.insert(k, FieldValue::String(s));
                } else if let Ok(i) = value.extract::<i64>() {
                    node.relational.insert(k, FieldValue::Int(i));
                } else if let Ok(f) = value.extract::<f64>() {
                    node.relational.insert(k, FieldValue::Float(f));
                }
            }
        }

        self.engine
            .insert(&node)
            .map_err(|e| PyRuntimeError::new_err(format!("Insert error: {:?}", e)))?;

        Ok(())
    }

    /// Retrieve a node by ID. Returns a dict or None.
    ///
    /// GIL Policy: HELD (µs read from L1 cache or pinned RocksDB).
    fn get(&self, py: Python, id: u64) -> PyResult<Option<PyObject>> {
        match self.engine.get(id) {
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
            .delete(id, reason)
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
            let index = engine
                .hnsw
                .read()
                .map_err(|e| PyRuntimeError::new_err(format!("HNSW lock error: {:?}", e)))?;
            let results = index.search_nearest(&vector, None, None, 0, top_k);
            Ok(results)
        })
    }

    /// Execute an IQL or LISP query string. Returns a formatted result string.
    ///
    /// GIL Policy: RELEASED during Tokio execution — allows other Python
    /// threads to run while VantaDB processes the query.
    fn query(&self, py: Python, iql_query: &str) -> PyResult<String> {
        let engine = self.engine.clone();
        let query_str = iql_query.to_string();

        // Create Executor borrowing from Arc — the Arc keeps the engine alive.
        // We block_on inside allow_threads to release the GIL.
        py.allow_threads(move || {
            let executor = Executor::new(&*engine);
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| PyRuntimeError::new_err(format!("Runtime error: {:?}", e)))?;
            rt.block_on(async {
                match executor.execute_hybrid(&query_str).await {
                    Ok(result) => Ok(format_execution_result(&result)),
                    Err(e) => Err(PyRuntimeError::new_err(format!("Query error: {:?}", e))),
                }
            })
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

    /// Introspect the active hardware profile. Returns a dict with:
    /// - profile: "SURVIVAL" | "PERFORMANCE" | "ENTERPRISE"
    /// - instructions: "AVX-512" | "AVX2" | "NEON" | "SCALAR FALLBACK"
    /// - logical_cores: int
    /// - total_memory: int (bytes)
    /// - vitality_score: int
    fn hardware_profile(&self, py: Python) -> PyResult<PyObject> {
        let caps = vantadb::hardware::HardwareCapabilities::global();
        let dict = PyDict::new(py);
        dict.set_item("profile", format!("{:?}", caps.profile))?;
        dict.set_item("instructions", format!("{:?}", caps.instructions))?;
        dict.set_item("logical_cores", caps.logical_cores)?;
        dict.set_item("total_memory", caps.total_memory)?;
        dict.set_item("vitality_score", caps.vitality_score)?;
        Ok(dict.into())
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
        let mut node = self
            .engine
            .get(source_id)
            .map_err(|e| PyRuntimeError::new_err(format!("Get source error: {:?}", e)))?
            .ok_or_else(|| {
                PyRuntimeError::new_err(format!("Source node {} not found", source_id))
            })?;

        match weight {
            Some(w) => node.add_weighted_edge(target_id, label, w),
            None => node.add_edge(target_id, label),
        }

        self.engine
            .insert(&node)
            .map_err(|e| PyRuntimeError::new_err(format!("Insert edge error: {:?}", e)))
    }

    /// String representation showing hardware profile.
    fn __repr__(&self) -> String {
        let caps = vantadb::hardware::HardwareCapabilities::global();
        format!(
            "VantaDB(profile={:?}, instructions={:?}, cores={}, memory={}MB)",
            caps.profile,
            caps.instructions,
            caps.logical_cores,
            caps.total_memory / 1024 / 1024
        )
    }
}

/// The Python module for VantaDB.
/// Usage: `import vantadb_py`
#[pymodule]
fn vantadb_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<VantaDB>()?;
    m.add("__version__", "0.5.0")?;
    Ok(())
}
