//! Python bindings for the VantaDB vector-graph database via PyO3.
//!
//! This crate exposes the [`VantaDB`] class and a [`connect`] function
//! for in-process, zero-network-overhead access to VantaDB from Python.
#![warn(missing_docs)]
#![allow(deprecated)]

use pyo3::buffer::PyBuffer;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods, PyModuleMethods, PyTuple, PyTupleMethods};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use vantadb::config::VantaConfig;
use vantadb::metadata;
use vantadb::sdk::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemorySearchRequest,
    VantaNodeInput, VantaValue,
};
use vantadb::DistanceMetric;

mod convert;
use convert::parse_direction;
mod types;
mod vector;

use types::{FlatBufferView, VantaPyListResult, VantaPyMemoryRecord, VantaPySearchHit};

use vector::{VantaVector, VantaVectorIter};

use crate::convert::{
    bulk_import_report_to_pydict, capabilities_to_pydict, check_lens, export_report_to_pydict,
    extract_vector, format_query_result, import_report_to_pydict, map_vanta_error, node_to_pydict,
    operational_metrics_to_pydict, py_any_to_value, py_dict_to_metadata, rebuild_report_to_pydict,
    runtime_profile_label, search_explanation_to_pydict, text_index_audit_report_to_pydict,
    text_index_repair_report_to_pydict,
};

/// Cap on `top_k`/`k` for all search entry points (ERR-022). Prevents absurd
/// values (e.g. `k = 10^9`) from reaching `HashSet::with_capacity(ef*3)`
/// style allocations in the engine, which abort the process (panic-alloc).
const MAX_K: usize = 1_000;

#[pyclass]
/// Python-accessible embedded VantaDB engine.
///
/// Create or open a database with ``VantaDB(db_path, ...)``:
///
/// Args:
///     db_path: Path to the database directory. Pass ``":memory:"`` (or an
///         empty string) to create an in-memory database that is discarded
///         when the connection closes.
///     memory_limit_bytes: Optional memory budget in bytes for the Rust engine.
///         Isolates the DB's memory from Python's heap. If None, uses hardware
///         detection or VANTADB_MEMORY_LIMIT env var.
///     read_only: If True, opens the DB in read-only mode. Safe for multi-process
///         access when another process holds the write lock.
///     backend: Storage backend to use — ``"memory"``, ``"rocksdb"``, or None
///         (None selects the default persistent backend, fjall). Unknown values
///         fall back to the default backend with a warning.
///
/// Returns:
///     VantaDB: A connected VantaDB database handle.
///
/// Raises:
///     ValueError: If the database file has an incompatible format or the
///         configuration is invalid.
///     OSError: If the database directory cannot be created or opened.
///     PermissionError: If the database directory is not accessible.
///     RuntimeError: For any other engine-level failure.
///
/// Example:
///     ```python
///     >>> from vantadb_py import VantaDB
///     >>> db = VantaDB(":memory:", backend="memory")  # in-memory engine
///     >>> db.put("agent/main", "task-1", "alpha")
///     >>> db.get_memory("agent/main", "task-1").payload
///     'alpha'
///     ```
pub struct VantaDB {
    engine: VantaEmbedded,
    op_gate: OpGate,
}

/// Durability gate: rejects new operations once `close()` has begun and keeps
/// `close()` waiting until every in-flight operation finishes. Mirrors
/// `vantadb-node/src/lib.rs` — closes the write-after-close race where a
/// thread whose engine call had not yet run (or is running GIL-released via
/// `py.detach`) would write after `close()` returned.
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
        while state.count > 0 {
            state = cvar.wait(state).unwrap_or_else(PoisonError::into_inner);
        }
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
fn enter(gate: &OpGate) -> PyResult<OpGuard> {
    gate.try_enter()
        .ok_or_else(|| PyRuntimeError::new_err("database is closing"))
}

#[pymethods]
impl VantaDB {
    /// Create or open a VantaDB database.
    ///
    /// Args:
    ///     db_path: Path to the database directory. Pass ``":memory:"`` (or an
    ///         empty string) to create an in-memory database that is discarded
    ///         when the connection closes.
    ///     memory_limit_bytes: Optional memory budget in bytes for the Rust engine.
    ///         Isolates the DB's memory from Python's heap. If None, uses hardware
    ///         detection or VANTADB_MEMORY_LIMIT env var.
    ///     read_only: If True, opens the DB in read-only mode. Safe for multi-process
    ///         access when another process holds the write lock.
    ///     backend: Storage backend to use — ``"memory"``, ``"rocksdb"``, or None
    ///         (None selects the default persistent backend, fjall). Unknown values
    ///         fall back to the default backend with a warning.
    ///
    /// Returns:
    ///     VantaDB: A connected VantaDB database handle.
    ///
    /// Raises:
    ///     ValueError: If the database file has an incompatible format or the
    ///         configuration is invalid.
    ///     OSError: If the database directory cannot be created or opened.
    ///     PermissionError: If the database directory is not accessible.
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")  # in-memory engine
    ///     >>> db.put("agent/main", "task-1", "alpha")
    ///     >>> db.get_memory("agent/main", "task-1").payload
    ///     'alpha'
    ///     ```
    #[new]
    #[pyo3(signature = (db_path, memory_limit_bytes=None, read_only=false, backend=None))]
    fn new(
        py: Python<'_>,
        db_path: &str,
        memory_limit_bytes: Option<u64>,
        read_only: bool,
        backend: Option<&str>,
    ) -> PyResult<Self> {
        let backend_kind = match backend {
            Some("rocksdb") => vantadb::BackendKind::RocksDb,
            Some("memory") => vantadb::BackendKind::InMemory,
            Some(other) => {
                tracing::warn!(
                    "Unknown backend \"{}\" — falling back to default (fjall). Known values: rocksdb, memory",
                    other
                );
                vantadb::BackendKind::Fjall
            }
            None => vantadb::BackendKind::Fjall,
        };
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            memory_limit: memory_limit_bytes,
            read_only,
            backend_kind,
            ..Default::default()
        };
        let engine = py
            .detach(move || VantaEmbedded::open_with_config(config))
            .map_err(map_vanta_error)?;

        Ok(VantaDB {
            engine,
            op_gate: OpGate::new(),
        })
    }

    /// Insert a node with content and an optional embedding vector.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during node insert.
    ///
    /// Args:
    ///     id: Unique node identifier (u128).
    ///     content: Text content stored as a relational field.
    ///     vector: Embedding vector (list of floats). Pass empty list for no vector.
    ///     fields: Optional dict of additional relational fields.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     TypeError: If ``vector`` is not a list of floats or ``fields`` contains
    ///         unsupported value types.
    ///     ValueError: If the node ID already exists, the vector dimension is
    ///         inconsistent, or another validation error occurs.
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> db.insert(1, "first node", [0.1, 0.2, 0.3], {"kind": "note"})
    ///     >>> db.get(1)["fields"]["kind"]
    ///     'note'
    ///     ```
    #[pyo3(signature = (id, content, vector, fields=None))]
    fn insert(
        &self,
        py: Python,
        id: u128,
        content: &str,
        vector: &Bound<'_, PyAny>,
        fields: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let _g = enter(&self.op_gate)?;
        let mut input = VantaNodeInput::new(id);
        input.content = Some(content.to_string());
        let v = extract_vector(vector, py)?;
        input.vector = (!v.is_empty()).then_some(v);

        if let Some(extra) = fields {
            for (key, value) in extra.iter() {
                let k: String = key.extract()?;
                input.fields.insert(k, py_any_to_value(&value)?);
            }
        }

        let engine = self.engine.clone();
        // PERF-24: GIL RELEASED — pure Rust node insert + WAL write
        py.detach(move || engine.insert_node(input).map_err(map_vanta_error))?;

        Ok(())
    }

    /// Insert or update multiple namespace-scoped records in parallel (batched).
    ///
    /// Supports two calling conventions:
    ///
    /// 1. **Positional (tuple list)** — backward-compatible:
    ///    ```
    ///    db.put_batch([(namespace, key, payload, metadata, vector, ttl), ...])
    ///    ```
    ///    Each entry is a tuple of up to 6 elements.
    ///
    /// 2. **Keyword** — typed per-column arrays:
    ///    ```
    ///    db.put_batch(keys=["k1", "k2"], vectors=[[0.1]*384, [0.2]*384],
    ///                 payloads=["p1", "p2"], metadatas=[{"f": "v"}, None],
    ///                 namespace="ns", ttls=[None, 1000])
    ///    ```
    ///    A batch is single-namespace by default: every record goes to
    ///    ``namespace`` (or ``"default"`` when omitted). To route records of one
    ///    batch into different namespaces, pass the parallel per-record column
    ///    ``namespaces`` (length must equal ``keys``); it overrides
    ///    ``namespace`` for each record:
    ///    ```
    ///    db.put_batch(keys=["k1", "k2"], vectors=[[0.1]*384, [0.2]*384],
    ///                 namespaces=["ns1", "ns2"])
    ///    ```
    ///
    /// Returns a list of ``VantaMemoryRecord`` objects, up to ~5x faster
    /// than sequential ``put()`` for large batches.
    #[deprecated(
        note = "use keyword arguments (keys=..., vectors=..., payloads=..., metadatas=..., namespace=..., namespaces=..., ttls=...) instead"
    )]
    #[allow(deprecated)]
    #[pyo3(signature = (entries, keys=None, vectors=None, payloads=None, metadatas=None, namespace=None, namespaces=None, ttls=None))]
    fn put_batch(
        &self,
        py: Python,
        entries: Option<&Bound<'_, PyAny>>,
        keys: Option<Vec<String>>,
        vectors: Option<Vec<Vec<f32>>>,
        payloads: Option<Vec<String>>,
        metadatas: Option<Vec<Option<HashMap<String, String>>>>,
        namespace: Option<String>,
        namespaces: Option<Vec<String>>,
        ttls: Option<Vec<Option<u64>>>,
    ) -> PyResult<Vec<VantaPyMemoryRecord>> {
        let _g = enter(&self.op_gate)?;
        // Backward compat: old tuple-based list-of-entries API
        if let Some(entries_list) = entries {
            let _ = py.import("warnings")?.call_method1("warn", ("put_batch() with positional tuples is deprecated; use keyword arguments (keys=..., vectors=..., ...) instead",))?;
            let mut inputs = Vec::with_capacity(entries_list.len().unwrap_or(0));
            for entry in entries_list.try_iter()? {
                let entry = entry?.cast::<PyTuple>()?.clone();
                if entry.len() < 3 {
                    return Err(PyValueError::new_err(
                        "each entry must be a tuple of at least (namespace, key, payload)",
                    ));
                }
                let namespace: String = entry.get_item(0)?.extract()?;
                let key: String = entry.get_item(1)?.extract()?;
                let payload: String = entry.get_item(2)?.extract()?;
                let dict = if entry.len() > 3 && !entry.get_item(3)?.is_none() {
                    let item = entry.get_item(3)?;
                    Some(item.cast::<PyDict>()?.clone())
                } else {
                    None
                };
                let vector_obj: Option<Bound<'_, PyAny>> =
                    if entry.len() > 4 && !entry.get_item(4)?.is_none() {
                        Some(entry.get_item(4)?)
                    } else {
                        None
                    };
                let ttl_ms: Option<u64> = if entry.len() > 5 {
                    let item = entry.get_item(5)?;
                    if item.is_none() {
                        None
                    } else {
                        Some(item.extract()?)
                    }
                } else {
                    None
                };

                let mut input = VantaMemoryInput::new(namespace, key, payload);
                input.metadata = py_dict_to_metadata(dict.as_ref())?;
                input.ttl_ms = ttl_ms;
                input.vector = match &vector_obj {
                    Some(v) => {
                        let vec = extract_vector(v, py)?;
                        (!vec.is_empty()).then_some(vec)
                    }
                    None => None,
                };
                inputs.push(input);
            }

            let engine = self.engine.clone();
            let records = py.detach(move || engine.put_batch(inputs).map_err(map_vanta_error))?;
            return Ok(records.into_iter().map(VantaPyMemoryRecord::new).collect());
        }

        // New keyword-based API
        let keys = keys.ok_or_else(|| {
            PyTypeError::new_err(
                "either positional 'entries' or keyword 'keys' + 'vectors' is required",
            )
        })?;
        let vectors = vectors.ok_or_else(|| {
            PyTypeError::new_err(
                "either positional 'entries' or keyword 'keys' + 'vectors' is required",
            )
        })?;
        let n = keys.len();
        if vectors.len() != n {
            return Err(PyValueError::new_err(format!(
                "keys.len() ({}) must equal vectors.len() ({})",
                n,
                vectors.len()
            )));
        }
        if let Some(ref p) = payloads {
            if p.len() != n {
                return Err(PyValueError::new_err(format!(
                    "payloads.len() ({}) must equal keys.len() ({})",
                    p.len(),
                    n
                )));
            }
        }
        if let Some(ref m) = metadatas {
            if m.len() != n {
                return Err(PyValueError::new_err(format!(
                    "metadatas.len() ({}) must equal keys.len() ({})",
                    m.len(),
                    n
                )));
            }
        }
        if let Some(ref t) = ttls {
            if t.len() != n {
                return Err(PyValueError::new_err(format!(
                    "ttls.len() ({}) must equal keys.len() ({})",
                    t.len(),
                    n
                )));
            }
        }
        if let Some(ref nss) = namespaces {
            if nss.len() != n {
                return Err(PyValueError::new_err(format!(
                    "namespaces.len() ({}) must equal keys.len() ({})",
                    nss.len(),
                    n
                )));
            }
        }

        let ns = namespace.unwrap_or_else(|| "default".to_string());
        let mut inputs = Vec::with_capacity(n);
        for i in 0..n {
            let payload = match &payloads {
                Some(p) => p[i].clone(),
                None => String::new(),
            };
            // ERR-030: per-record namespace routing. A batch is single-namespace
            // by default (`namespace`, falling back to "default"), but each
            // record's intended namespace is honored when the parallel
            // per-record `namespaces` column is supplied.
            let ns_i = match &namespaces {
                Some(nss) => nss[i].clone(),
                None => ns.clone(),
            };
            let mut input = VantaMemoryInput::new(ns_i, keys[i].clone(), payload);

            if let Some(all_meta) = &metadatas {
                if let Some(meta_dict) = &all_meta[i] {
                    let mut btree = std::collections::BTreeMap::new();
                    for (k, v) in meta_dict.iter() {
                        btree.insert(k.clone(), VantaValue::String(v.clone()));
                    }
                    input.metadata = btree;
                }
            }

            if let Some(ttl_list) = &ttls {
                input.ttl_ms = ttl_list[i];
            }

            input.vector = Some(vectors[i].clone());
            inputs.push(input);
        }

        let engine = self.engine.clone();
        let records = py.detach(move || engine.put_batch(inputs).map_err(map_vanta_error))?;
        Ok(records.into_iter().map(VantaPyMemoryRecord::new).collect())
    }

    /// Insert or update multiple namespace-scoped records using a 2D NumPy array
    /// for zero-copy, per-row-avoidant vector input.
    ///
    /// Accepts `vectors` as a 2D PyBuffer (e.g. a NumPy float32 array of shape
    /// ``[N, D]``). ``keys`` is required; other fields are optional parallel Vecs.
    /// All Vecs must have length N matching ``vectors.shape[0]``.
    ///
    /// Returns a list of record dicts in the same order as inputs.
    // PyO3 keyword argument binding requires matching function parameters in Rust.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (vectors, keys, payloads = None, metadatas = None, namespaces = None, ttls = None))]
    fn put_batch_raw(
        &self,
        py: Python,
        vectors: &Bound<'_, PyAny>,
        keys: Vec<String>,
        payloads: Option<Vec<String>>,
        metadatas: Option<Vec<Option<Py<PyAny>>>>,
        namespaces: Option<Vec<String>>,
        ttls: Option<Vec<Option<u64>>>,
    ) -> PyResult<Vec<VantaPyMemoryRecord>> {
        let _g = enter(&self.op_gate)?;
        /// Build VantaMemoryInput vector from per-row parameters and a vector getter.
        fn build_inputs(
            nrows: usize,
            _ndims: usize,
            keys: &[String],
            payloads: &Option<Vec<String>>,
            metadatas: &Option<Vec<Option<Py<PyAny>>>>,
            namespaces: &Option<Vec<String>>,
            ttls: &Option<Vec<Option<u64>>>,
            py: Python,
            get_vector: &dyn Fn(usize) -> Vec<f32>,
        ) -> PyResult<Vec<VantaMemoryInput>> {
            let mut inputs = Vec::with_capacity(nrows);
            for i in 0..nrows {
                let namespace = match namespaces {
                    Some(ns) => ns[i].clone(),
                    None => "default".to_string(),
                };
                let key = keys[i].clone();
                let payload = match payloads {
                    Some(p) => p[i].clone(),
                    None => String::new(),
                };

                let mut input = VantaMemoryInput::new(namespace, key, payload);

                if let Some(all_meta) = metadatas {
                    if let Some(meta_obj) = &all_meta[i] {
                        let any_bound = meta_obj.bind(py);
                        let dict: &Bound<'_, PyDict> = any_bound.cast::<PyDict>()?;
                        input.metadata = py_dict_to_metadata(Some(dict))?;
                    }
                }

                if let Some(ttl_list) = ttls {
                    input.ttl_ms = ttl_list[i];
                }

                input.vector = Some(get_vector(i));
                inputs.push(input);
            }
            Ok(inputs)
        }

        // PERF-15: zero-copy f32 buffer path — avoids intermediate Vec<f32> allocation
        if let Ok(buf) = PyBuffer::<f32>::get(vectors) {
            let shape: &[usize] = buf.shape();
            if shape.len() != 2 {
                return Err(PyValueError::new_err(
                    "vectors must be a 2D array (shape [N, D])",
                ));
            }
            let nrows = shape[0];
            let ndims = shape[1];

            if nrows != keys.len() {
                return Err(PyValueError::new_err(format!(
                    "vectors.shape[0] ({}) must equal keys.len() ({})",
                    nrows,
                    keys.len()
                )));
            }
            check_lens(&payloads, &metadatas, &namespaces, &ttls, nrows)?;

            if buf.is_c_contiguous() {
                if let Some(slice) = buf.as_slice(py) {
                    let view = FlatBufferView::new(slice, nrows, ndims);
                    let inputs = build_inputs(
                        nrows,
                        ndims,
                        &keys,
                        &payloads,
                        &metadatas,
                        &namespaces,
                        &ttls,
                        py,
                        &|i| view.row_to_vec(i),
                    )?;
                    let engine = self.engine.clone();
                    let records =
                        py.detach(move || engine.put_batch(inputs).map_err(map_vanta_error))?;
                    return Ok(records.into_iter().map(VantaPyMemoryRecord::new).collect());
                }
            }

            // Fallback: f32 buffer not contiguous or as_slice failed
            let flat = buf.to_vec(py)?;
            let inputs = build_inputs(
                nrows,
                ndims,
                &keys,
                &payloads,
                &metadatas,
                &namespaces,
                &ttls,
                py,
                &|i| {
                    let start = i * ndims;
                    flat[start..start + ndims].to_vec()
                },
            )?;
            let engine = self.engine.clone();
            let records = py.detach(move || engine.put_batch(inputs).map_err(map_vanta_error))?;
            return Ok(records.into_iter().map(VantaPyMemoryRecord::new).collect());
        }

        // Fallback: f64 buffer — downcast to f32
        if let Ok(buf) = PyBuffer::<f64>::get(vectors) {
            let shape: &[usize] = buf.shape();
            if shape.len() != 2 {
                return Err(PyValueError::new_err(
                    "vectors must be a 2D array (shape [N, D])",
                ));
            }
            let nrows = shape[0];
            let ndims = shape[1];

            if nrows != keys.len() {
                return Err(PyValueError::new_err(format!(
                    "vectors.shape[0] ({}) must equal keys.len() ({})",
                    nrows,
                    keys.len()
                )));
            }
            check_lens(&payloads, &metadatas, &namespaces, &ttls, nrows)?;

            let flat_f64 = buf.to_vec(py)?;
            let flat: Vec<f32> = flat_f64.into_iter().map(|x| x as f32).collect();
            let inputs = build_inputs(
                nrows,
                ndims,
                &keys,
                &payloads,
                &metadatas,
                &namespaces,
                &ttls,
                py,
                &|i| {
                    let start = i * ndims;
                    flat[start..start + ndims].to_vec()
                },
            )?;
            let engine = self.engine.clone();
            let records = py.detach(move || engine.put_batch(inputs).map_err(map_vanta_error))?;
            return Ok(records.into_iter().map(VantaPyMemoryRecord::new).collect());
        }

        Err(PyTypeError::new_err(
            "Expected a 2D NumPy array (buffer protocol)",
        ))
    }

    /// Put or update a namespace-scoped persistent memory record.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during storage write
    /// and index update.
    ///
    /// Args:
    ///     namespace: Namespace that groups related records.
    ///     key: Unique record key within the namespace. Reusing an existing key
    ///         updates the stored record.
    ///     payload: Text payload stored with the record.
    ///     metadata: Optional dict of scalar values (str, int, float, bool,
    ///         datetime, list, or None) used for filtering.
    ///     vector: Optional embedding vector (list of floats or NumPy array).
    ///     ttl_ms: Optional time-to-live in milliseconds; the record expires
    ///         after this duration.
    ///
    /// Returns:
    ///     VantaMemoryRecord: The stored record, exposing ``namespace``, ``key``,
    ///     ``payload``, ``metadata``, ``vector``, ``created_at_ms``,
    ///     ``updated_at_ms``, ``version``, ``node_id``, and ``expires_at_ms``.
    ///
    /// Raises:
    ///     TypeError: If ``vector`` is not a list of floats or ``metadata``
    ///         contains unsupported value types.
    ///     ValueError: If the vector dimension is inconsistent with the index or
    ///         validation fails.
    ///     OSError: If the write cannot be persisted (WAL or storage I/O failure).
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> record = db.put("agent/main", "task-1", "organize the backlog",
    ///     ...                  metadata={"category": "task"}, vector=[1.0, 0.0, 0.0])
    ///     >>> record.key
    ///     'task-1'
    ///     >>> record["metadata"]["category"]
    ///     'task'
    ///     ```

    // PyO3 keyword argument binding requires matching function parameters in Rust.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (namespace, key, payload, metadata=None, vector=None, ttl_ms=None))]
    fn put(
        &self,
        py: Python,
        namespace: &str,
        key: &str,
        payload: &str,
        metadata: Option<&Bound<'_, PyDict>>,
        vector: Option<&Bound<'_, PyAny>>,
        ttl_ms: Option<u64>,
    ) -> PyResult<VantaPyMemoryRecord> {
        let _g = enter(&self.op_gate)?;
        let mut input = VantaMemoryInput::new(namespace, key, payload);
        input.metadata = py_dict_to_metadata(metadata)?;
        input.ttl_ms = ttl_ms;
        input.vector = match vector {
            Some(v) => {
                let vec = extract_vector(v, py)?;
                (!vec.is_empty()).then_some(vec)
            }
            None => None,
        };

        let engine = self.engine.clone();
        // PERF-24: GIL RELEASED — pure Rust storage write + index update
        let record = py.detach(move || engine.put(input).map_err(map_vanta_error))?;
        Ok(VantaPyMemoryRecord::new(record))
    }

    /// Retrieve a namespace-scoped persistent memory record.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during storage read.
    ///
    /// Args:
    ///     namespace: Namespace the record belongs to.
    ///     key: Record key within the namespace.
    ///
    /// Returns:
    ///     VantaMemoryRecord or None: The stored record, or None if no record
    ///     exists for the given namespace and key.
    ///
    /// Raises:
    ///     OSError: If the storage cannot be read.
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> db.put("agent/main", "task-1", "organize the backlog")
    ///     >>> record = db.get_memory("agent/main", "task-1")
    ///     >>> record.payload
    ///     'organize the backlog'
    ///     >>> db.get_memory("agent/main", "missing") is None
    ///     True
    ///     ```
    fn get_memory(
        &self,
        py: Python,
        namespace: &str,
        key: &str,
    ) -> PyResult<Option<VantaPyMemoryRecord>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let n = namespace.to_string();
        let k = key.to_string();
        let record = py.detach(move || engine.get(&n, &k).map_err(map_vanta_error))?;
        match record {
            Some(record) => Ok(Some(VantaPyMemoryRecord::new(record))),
            None => Ok(None),
        }
    }

    /// Delete a namespace-scoped persistent memory record.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during storage delete.
    ///
    /// Args:
    ///     namespace: Namespace the record belongs to.
    ///     key: Record key within the namespace.
    ///
    /// Returns:
    ///     bool: True if a record was deleted, False if no matching record existed.
    ///
    /// Raises:
    ///     OSError: If the storage cannot be written.
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> db.put("agent/main", "temp", "delete me")
    ///     >>> db.delete_memory("agent/main", "temp")
    ///     True
    ///     >>> db.get_memory("agent/main", "temp") is None
    ///     True
    ///     ```
    fn delete_memory(&self, py: Python, namespace: &str, key: &str) -> PyResult<bool> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let namespace = namespace.to_string();
        let key = key.to_string();
        py.detach(move || engine.delete(&namespace, &key).map_err(map_vanta_error))
    }

    /// List namespace-scoped persistent memory records.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during storage scan.
    ///
    /// Args:
    ///     namespace: Namespace to list records from.
    ///     filters: Optional dict of metadata field values to filter on.
    ///     limit: Maximum number of records to return (default 100).
    ///     cursor: Optional pagination cursor returned as ``next_cursor`` from a
    ///         previous page.
    ///
    /// Returns:
    ///     VantaListResult: A page of records with ``records``, ``total_count``,
    ///     and ``next_cursor`` properties. Iterate or index into the result to
    ///     access ``VantaMemoryRecord`` items.
    ///
    /// Raises:
    ///     TypeError: If ``filters`` contains unsupported value types.
    ///     ValueError: If a filter value is invalid.
    ///     OSError: If the storage cannot be read.
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> db.put("agent/main", "task-1", "alpha", metadata={"category": "task"})
    ///     >>> db.put("agent/main", "task-2", "beta", metadata={"category": "task"})
    ///     >>> page = db.list_memory("agent/main", filters={"category": "task"})
    ///     >>> len(page)
    ///     2
    ///     >>> page[0].key
    ///     'task-1'
    ///     ```
    #[pyo3(signature = (namespace, filters=None, limit=100, cursor=None))]
    fn list_memory(
        &self,
        py: Python,
        namespace: &str,
        filters: Option<&Bound<'_, PyDict>>,
        limit: usize,
        cursor: Option<usize>,
    ) -> PyResult<VantaPyListResult> {
        let _g = enter(&self.op_gate)?;
        let namespace = namespace.to_string();
        let filters_meta = py_dict_to_metadata(filters)?;
        let engine = self.engine.clone();
        let page = py.detach(move || {
            engine
                .list(
                    &namespace,
                    VantaMemoryListOptions {
                        #[allow(deprecated)]
                        filters: filters_meta,
                        filter_ops: None,
                        limit,
                        cursor,
                    },
                )
                .map_err(map_vanta_error)
        })?;

        let records: Vec<VantaPyMemoryRecord> = page
            .records
            .into_iter()
            .map(VantaPyMemoryRecord::new)
            .collect();

        Ok(VantaPyListResult::new(records, page.next_cursor))
    }

    /// Search namespace-scoped persistent memory records by vector + filters.
    ///
    /// Combines ANN vector search with optional metadata filters and an optional
    /// lexical text query (hybrid search).
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during distance
    /// computation and HNSW traversal.
    ///
    /// Args:
    ///     namespace: Namespace to search within.
    ///     query_vector: Query embedding vector (list of floats or NumPy array).
    ///     filters: Optional dict of metadata field values to filter on.
    ///     text_query: Optional full-text query to combine with the vector search.
    ///     top_k: Maximum number of hits to return (default 10).
    ///     distance_metric: Distance metric — ``"cosine"`` (default) or
    ///         ``"euclidean"``. Unknown values fall back to cosine with a warning.
    ///     explain: If True, include search explanation data on each hit
    ///         (default False).
    ///
    /// Returns:
    ///     list[VantaSearchHit]: Search hits ordered by relevance, each exposing
    ///     ``key``, ``payload``, ``metadata``, ``vector``, ``score``, and
    ///     ``node_id`` properties.
    ///
    /// Raises:
    ///     TypeError: If ``query_vector`` is not a list of floats or ``filters``
    ///         contains unsupported value types.
    ///     ValueError: If the vector dimension is inconsistent with the index or
    ///         validation fails.
    ///     OSError: If the storage cannot be read.
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> db.put("agent/main", "task-1", "organize the backlog",
    ///     ...          vector=[1.0, 0.0, 0.0])
    ///     >>> hits = db.search_memory("agent/main", [0.9, 0.1, 0.0], top_k=5)
    ///     >>> hits[0].key
    ///     'task-1'
    ///     >>> hits[0].score > 0.0
    ///     True
    ///     ```
    // PyO3 keyword argument binding requires matching function parameters in Rust.
    #[pyo3(signature = (namespace, query_vector, filters=None, text_query=None, top_k=10, distance_metric=None, explain=false))]
    #[allow(clippy::too_many_arguments)]
    fn search_memory(
        &self,
        py: Python,
        namespace: &str,
        query_vector: &Bound<'_, PyAny>,
        filters: Option<&Bound<'_, PyDict>>,
        text_query: Option<String>,
        top_k: usize,
        distance_metric: Option<&str>,
        explain: bool,
    ) -> PyResult<Vec<VantaPySearchHit>> {
        let _g = enter(&self.op_gate)?;
        let metric = match distance_metric {
            Some("euclidean") => DistanceMetric::Euclidean,
            Some(other) => {
                tracing::warn!(
                    "Unknown distance_metric \"{}\" — falling back to default (cosine). Known values: cosine, euclidean",
                    other
                );
                DistanceMetric::Cosine
            }
            None => DistanceMetric::Cosine,
        };

        let request = VantaMemorySearchRequest {
            namespace: namespace.to_string(),
            query_vector: extract_vector(query_vector, py)?,
            query_sparse: None,
            filters: py_dict_to_metadata(filters)?,
            text_query,
            top_k: top_k.min(MAX_K),
            distance_metric: metric,
            explain,
        };

        let engine = self.engine.clone();
        // PERF-24: GIL RELEASED — pure Rust distance computation + HNSW traversal
        let hits = py.detach(move || engine.search(request).map_err(map_vanta_error))?;

        // Pure Rust struct wrapping — no Python objects created
        hits.into_iter()
            .map(|hit| {
                Ok(VantaPySearchHit {
                    inner: hit.record,
                    score: hit.score,
                })
            })
            .collect()
    }

    /// Rebuild ANN and derived memory indexes from canonical storage.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during index rebuild.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     dict: Rebuild report with keys ``scanned_nodes``, ``indexed_vectors``,
    ///     ``skipped_tombstones``, ``duration_ms``, ``derived_rebuild_ms``,
    ///     ``index_path``, and ``success``.
    ///
    /// Raises:
    ///     OSError: If the index cannot be written to disk.
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> db.put("agent/main", "task-1", "alpha", vector=[1.0, 0.0, 0.0])
    ///     >>> report = db.rebuild_index()
    ///     >>> report["indexed_vectors"]
    ///     1
    ///     >>> report["success"]
    ///     True
    ///     ```
    fn rebuild_index(&self, py: Python) -> PyResult<Py<PyAny>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let report = py.detach(move || engine.rebuild_index().map_err(map_vanta_error))?;
        rebuild_report_to_pydict(py, &report)
    }

    /// Rebuild the HNSW vector index by paginating through text records.
    ///
    /// Paginates through memory records using cursor-based `list()` in
    /// batches of `page_size` (default 1000, max 1000) to prevent OOM
    /// on namespaces with 100K+ records.
    #[pyo3(signature = (namespace, page_size=1000))]
    fn reindex_hnsw_from_text(
        &self,
        py: Python,
        namespace: &str,
        page_size: usize,
    ) -> PyResult<Py<PyAny>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let namespace = namespace.to_string();
        let report = py.detach(move || {
            engine
                .reindex_hnsw_from_text(&namespace, Some(page_size))
                .map_err(map_vanta_error)
        })?;
        rebuild_report_to_pydict(py, &report)
    }

    /// Export one namespace as JSONL.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during export.
    ///
    /// Args:
    ///     path: Destination file path for the JSONL export.
    ///     namespace: Namespace to export.
    ///
    /// Returns:
    ///     dict: Export report with keys ``records_exported``, ``namespaces``,
    ///     ``path``, and ``duration_ms``.
    ///
    /// Raises:
    ///     FileNotFoundError: If the target directory does not exist.
    ///     PermissionError: If the target path is not writable.
    ///     OSError: For other file I/O failures.
    ///     ValueError: If the namespace does not exist or is invalid.
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> import tempfile
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> db.put("agent/main", "task-1", "alpha")
    ///     >>> with tempfile.TemporaryDirectory() as tmp:
    ///     ...     report = db.export_namespace(f"{tmp}/export.jsonl", "agent/main")
    ///     ...     report["records_exported"]
    ///     1
    ///     ```
    fn export_namespace(&self, py: Python, path: &str, namespace: &str) -> PyResult<Py<PyAny>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let path = path.to_string();
        let namespace = namespace.to_string();
        let report = py.detach(move || {
            engine
                .export_namespace(&path, &namespace)
                .map_err(map_vanta_error)
        })?;
        export_report_to_pydict(py, &report)
    }

    /// Export all namespaces as JSONL.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during export.
    ///
    /// Args:
    ///     path: Destination file path for the JSONL export.
    ///
    /// Returns:
    ///     dict: Export report with keys ``records_exported``, ``namespaces``,
    ///     ``path``, and ``duration_ms``.
    ///
    /// Raises:
    ///     FileNotFoundError: If the target directory does not exist.
    ///     PermissionError: If the target path is not writable.
    ///     OSError: For other file I/O failures.
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> import tempfile
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> db.put("agent/main", "task-1", "alpha")
    ///     >>> with tempfile.TemporaryDirectory() as tmp:
    ///     ...     report = db.export_all(f"{tmp}/export.jsonl")
    ///     ...     report["records_exported"]
    ///     1
    ///     ```
    fn export_all(&self, py: Python, path: &str) -> PyResult<Py<PyAny>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let path = path.to_string();
        let report = py.detach(move || engine.export_all(&path).map_err(map_vanta_error))?;
        export_report_to_pydict(py, &report)
    }

    /// Import records from a VantaDB memory JSONL export.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during import.
    ///
    /// Args:
    ///     path: Path to a JSONL file previously produced by ``export_namespace()``
    ///         or ``export_all()``.
    ///
    /// Returns:
    ///     dict: Import report with keys ``inserted``, ``updated``, ``skipped``,
    ///     ``errors``, and ``duration_ms``.
    ///
    /// Raises:
    ///     FileNotFoundError: If the import file does not exist.
    ///     ValueError: If the file is not a valid VantaDB JSONL export.
    ///     PermissionError: If the file cannot be read.
    ///     OSError: For other file I/O failures.
    ///     RuntimeError: For any other engine-level failure.
    ///
    /// Example:
    ///     ```python
    ///     >>> import tempfile
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> db.put("agent/main", "task-1", "alpha")
    ///     >>> with tempfile.TemporaryDirectory() as tmp:
    ///     ...     export_path = f"{tmp}/export.jsonl"
    ///     ...     db.export_namespace(export_path, "agent/main")
    ///     ...     target = VantaDB(":memory:", backend="memory")
    ///     ...     report = target.import_file(export_path)
    ///     ...     report["inserted"]
    ///     1
    ///     ...     target.get_memory("agent/main", "task-1").payload
    ///     'alpha'
    ///     ```
    fn import_file(&self, py: Python, path: &str) -> PyResult<Py<PyAny>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let path = path.to_string();
        let report = py.detach(move || engine.import_file(&path).map_err(map_vanta_error))?;
        import_report_to_pydict(py, &report)
    }

    /// Bulk-import records from a binary .vdbdump file.
    /// Returns a dict with total_records, batches_committed, duration_ms.
    fn bulk_import(&self, py: Python, path: &str) -> PyResult<Py<PyAny>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let path = path.to_string();
        let report = py.detach(move || engine.bulk_import_file(&path).map_err(map_vanta_error))?;
        bulk_import_report_to_pydict(py, &report)
    }

    /// Bulk-import records from binary bytes (.vdbdump format).
    /// Returns a dict with total_records, batches_committed, duration_ms.
    fn bulk_import_bytes(&self, py: Python, data: &[u8]) -> PyResult<Py<PyAny>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let data = data.to_vec();
        let report = py.detach(move || {
            let mut cursor = std::io::Cursor::new(&data[..]);
            engine
                .bulk_import_stream(&mut cursor)
                .map_err(map_vanta_error)
        })?;
        bulk_import_report_to_pydict(py, &report)
    }

    /// Run a read-only structural audit of the derived text index.
    #[pyo3(signature = (namespace=None, deep=false))]
    fn audit_text_index(
        &self,
        py: Python,
        namespace: Option<&str>,
        deep: bool,
    ) -> PyResult<Py<PyAny>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let namespace = namespace.map(|s| s.to_string());
        let report = py
            .detach(move || {
                let ns_ref = namespace.as_deref();
                if deep {
                    engine.audit_text_index_deep(ns_ref)
                } else {
                    engine.audit_text_index(ns_ref)
                }
            })
            .map_err(map_vanta_error)?;
        text_index_audit_report_to_pydict(py, &report)
    }

    /// Rebuild the text index from canonical storage as a repair primitive.
    fn repair_text_index(&self, py: Python) -> PyResult<Py<PyAny>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let report = py.detach(move || engine.repair_text_index().map_err(map_vanta_error))?;
        text_index_repair_report_to_pydict(py, &report)
    }

    /// Return operational metrics for startup, replay, rebuild, export, and import.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during metrics snapshot.
    ///
    /// Args:
    ///     None
    ///
    /// Returns:
    ///     dict: Operational counters and memory telemetry, including
    ///     ``startup_ms``, ``wal_replay_ms``, ``ann_rebuild_ms``,
    ///     ``records_exported``, ``records_imported``, ``process_rss_bytes``,
    ///     ``hnsw_nodes_count``, and jemalloc allocation counters.
    ///
    /// Raises:
    ///     RuntimeError: For any engine-level failure while snapshotting metrics.
    ///
    /// Example:
    ///     ```python
    ///     >>> from vantadb_py import VantaDB
    ///     >>> db = VantaDB(":memory:", backend="memory")
    ///     >>> db.put("agent/main", "task-1", "alpha")
    ///     >>> metrics = db.operational_metrics()
    ///     >>> metrics["startup_ms"] >= 0
    ///     True
    ///     ```
    fn operational_metrics(&self, py: Python) -> PyResult<Py<PyAny>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let metrics = py.detach(move || engine.operational_metrics());
        operational_metrics_to_pydict(py, &metrics)
    }

    /// Retrieve a node by ID. Returns a dict or None.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during database retrieval.
    fn get(&self, py: Python, id: u128) -> PyResult<Option<Py<PyAny>>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let node = py.detach(move || engine.get_node(id).map_err(map_vanta_error))?;
        match node {
            Some(node) => Ok(Some(node_to_pydict(py, &node)?)),
            None => Ok(None),
        }
    }

    /// Delete a node by ID with an auditable reason (tombstone).
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during node deletion.
    #[pyo3(signature = (id, reason="manual deletion"))]
    fn delete(&self, py: Python, id: u128, reason: &str) -> PyResult<()> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let reason_str = reason.to_string();
        py.detach(move || engine.delete_node(id, &reason_str).map_err(map_vanta_error))
    }

    /// K-NN vector search. Returns a list of (node_id, distance) tuples.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during HNSW traversal.
    ///
    /// Args:
    ///     vector: Query embedding vector.
    ///     top_k: Number of nearest neighbors to return.
    #[pyo3(signature = (vector, top_k=10))]
    fn search(
        &self,
        py: Python,
        vector: &Bound<'_, PyAny>,
        top_k: usize,
    ) -> PyResult<Vec<(u128, f32)>> {
        let _g = enter(&self.op_gate)?;
        // PERF-24: vector extraction (needs GIL) — done before detach
        let v = extract_vector(vector, py)?;
        let engine = self.engine.clone();
        // GIL RELEASED: only pure Rust — distance computation + graph traversal
        py.detach(move || {
            engine
                .search_vector(&v, top_k.min(MAX_K))
                .map(|hits| {
                    // Pure Rust tuple mapping — no Python objects created
                    // node_id is u128 in core; keep full precision (ERR-023)
                    hits.into_iter()
                        .map(|hit| (hit.node_id, hit.distance))
                        .collect()
                })
                .map_err(map_vanta_error)
        })
    }

    /// K-NN vector search for a batch of vectors.
    ///
    /// GIL Policy: RELEASED eager, runs search in parallel using Rayon.
    ///
    /// Args:
    ///     vectors: List of query embedding vectors.
    ///     top_k: Number of nearest neighbors to return per vector.
    #[pyo3(signature = (vectors, top_k=10))]
    fn search_batch(
        &self,
        py: Python,
        vectors: Vec<Bound<'_, PyAny>>,
        top_k: usize,
    ) -> PyResult<Vec<Vec<(u128, f32)>>> {
        let _g = enter(&self.op_gate)?;
        // PERF-24: vector extraction (needs GIL) — done before detach
        let parsed: PyResult<Vec<Vec<f32>>> =
            vectors.iter().map(|v| extract_vector(v, py)).collect();
        let parsed = parsed?;
        let engine = self.engine.clone();
        // GIL RELEASED: pure Rust — parallel graph traversal, no Python objects
        py.detach(move || {
            use rayon::prelude::*;
            parsed
                .into_par_iter()
                .map(|vector| {
                    engine
                        .search_vector(&vector, top_k.min(MAX_K))
                        .map(|hits| {
                            hits.into_iter()
                                .map(|hit| (hit.node_id, hit.distance))
                                .collect()
                        })
                        .map_err(map_vanta_error)
                })
                .collect::<Result<Vec<Vec<(u128, f32)>>, _>>()
        })
    }

    /// Hybrid memory search for a batch of full search requests.
    ///
    /// Each element is a [`SearchRequest`][1] dataclass or an equivalent
    /// ``dict`` with the same keys as ``search_memory``: ``namespace``,
    /// ``query_vector``, ``filters``, ``text_query``, ``top_k``,
    /// ``distance_metric``, ``explain``.
    ///
    /// [1]: https://vantadb.github.io (see `vantadb_py.SearchRequest`)
    ///
    /// GIL Policy: RELEASED eager, runs searches in parallel using Rayon.
    /// Fail-fast: ``try_for_each`` aborts at the first failing request and the
    /// first error is raised to Python.
    ///
    /// Args:
    ///     requests: List of `SearchRequest` dataclass instances (or dicts).
    ///     top_k: Fallback `top_k` for requests that omit it (default 10).
    ///
    /// Returns:
    ///     list[list[VantaSearchHit]]: One hit list per request, in input order.
    ///
    /// Raises:
    ///     ValueError: If a request fails engine validation (raised eagerly on
    ///         the first failing request).
    ///     RuntimeError: For internal failures or engine errors.
    #[pyo3(signature = (requests, top_k=10))]
    fn search_batch_requests(
        &self,
        py: Python,
        requests: Vec<Bound<'_, PyAny>>,
        top_k: usize,
    ) -> PyResult<Vec<Vec<VantaPySearchHit>>> {
        let _g = enter(&self.op_gate)?;
        // PERF-24: parse all requests (needs GIL) before detach
        let parsed: PyResult<Vec<VantaMemorySearchRequest>> = requests
            .iter()
            .map(|obj| self.parse_search_request(obj, py, top_k))
            .collect();
        let parsed = parsed?;
        let count = parsed.len();
        let engine = self.engine.clone();
        let results: std::sync::Mutex<Vec<Vec<VantaPySearchHit>>> =
            std::sync::Mutex::new((0..count).map(|_| Vec::new()).collect());
        let results_ref = &results;
        // GIL RELEASED: pure Rust — parallel hybrid search, no Python objects
        py.detach(move || -> PyResult<()> {
            use rayon::prelude::*;
            parsed
                .into_par_iter()
                .enumerate()
                .try_for_each(|(index, request)| {
                    let hits = engine.search(request).map_err(map_vanta_error)?;
                    results_ref.lock().map_err(|_| {
                        PyRuntimeError::new_err("search_batch_requests: result mutex poisoned")
                    })?[index] = hits
                        .into_iter()
                        .map(|hit| VantaPySearchHit {
                            inner: hit.record,
                            score: hit.score,
                        })
                        .collect();
                    Ok(())
                })
        })?;
        results
            .into_inner()
            .map_err(|_| PyRuntimeError::new_err("search_batch_requests: result mutex poisoned"))
    }

    /// Execute an IQL or LISP query string. Returns a formatted result string.
    ///
    /// GIL Policy: RELEASED during Tokio execution — allows other Python
    /// threads to run while VantaDB processes the query.
    fn query(&self, py: Python, iql_query: &str) -> PyResult<String> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let query_str = iql_query.to_string();

        py.detach(move || {
            engine
                .query(&query_str)
                .map(|result| format_query_result(&result))
                .map_err(map_vanta_error)
        })
    }

    /// Flush WAL and HNSW index to disk for durability.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during disk sync.
    fn flush(&self, py: Python) -> PyResult<()> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || engine.flush().map_err(map_vanta_error))
    }

    /// Compact the WAL: flush, archive ``vanta.wal`` as
    /// ``vanta.wal.<timestamp>``, and start a fresh WAL.
    #[pyo3(signature = ())]
    fn compact_wal(&self, py: Python) -> PyResult<()> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || engine.compact_wal().map_err(map_vanta_error))
    }

    /// Scan all memory records and physically delete expired ones.
    /// Returns the number of records purged.
    #[pyo3(signature = ())]
    fn purge_expired(&self, py: Python) -> PyResult<u64> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || engine.purge_expired().map_err(map_vanta_error))
    }

    /// Introspect the stable runtime capabilities exposed by the SDK boundary.
    fn capabilities(&self, py: Python) -> PyResult<Py<PyAny>> {
        let capabilities = self.engine.capabilities();
        capabilities_to_pydict(py, &capabilities)
    }

    /// Return capabilities and system memory telemetry.
    fn hardware_profile(&self, py: Python) -> PyResult<Py<PyAny>> {
        let caps_obj = self.capabilities(py)?;
        let metrics_obj = self.operational_metrics(py)?;

        let caps_dict = caps_obj.bind(py).cast::<PyDict>()?;
        let metrics_dict = metrics_obj.bind(py).cast::<PyDict>()?;

        // CODE-004: create a NEW dict instead of shallow-cloning caps_dict
        let merged_dict = PyDict::new(py);
        for entry in caps_dict.iter() {
            let (key, value) = entry;
            merged_dict.set_item(key, value)?;
        }

        let memory_keys = [
            "process_rss_bytes",
            "process_virtual_bytes",
            "hnsw_nodes_count",
            "hnsw_logical_bytes",
            "mmap_resident_bytes",
            "volatile_cache_entries",
            "volatile_cache_cap_bytes",
        ];

        for &key in &memory_keys {
            if let Some(val) = metrics_dict.get_item(key)? {
                merged_dict.set_item(key, val)?;
            }
        }

        Ok(merged_dict.unbind().into())
    }

    /// Add a labeled edge between two nodes.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during edge insertion.
    ///
    /// Args:
    ///     source_id: Source node ID.
    ///     target_id: Target node ID.
    ///     label: Edge label (e.g., "belongs_to", "similar_to").
    ///     weight: Optional edge weight (default 1.0).
    ///     created_at_ms: Optional creation timestamp (Unix ms). Defaults to now.
    #[pyo3(signature = (source_id, target_id, label, weight=None, created_at_ms=None))]
    fn add_edge(
        &self,
        py: Python,
        source_id: u128,
        target_id: u128,
        label: &str,
        weight: Option<f32>,
        created_at_ms: Option<u64>,
    ) -> PyResult<()> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let label_str = label.to_string();
        py.detach(move || {
            engine
                .add_edge(source_id, target_id, &label_str, weight, created_at_ms)
                .map_err(map_vanta_error)
        })
    }

    /// Flush and close the embedded engine handle.
    fn close(&self, py: Python) -> PyResult<()> {
        // Durability barrier: reject new ops and wait for in-flight ones to
        // finish BEFORE the engine is closed (see OpGate docs).
        self.op_gate.drain();
        let engine = self.engine.clone();
        py.detach(move || engine.close().map_err(map_vanta_error))
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

    /// Breadth-First-Search starting from a designated set of root IDs,
    /// up to a maximum depth, returning the discovered distinct Node IDs.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during graph traversal.
    #[pyo3(signature = (roots, max_depth=999999, direction="Forward"))]
    fn graph_bfs(
        &self,
        py: Python,
        roots: Vec<u128>,
        max_depth: usize,
        direction: &str,
    ) -> PyResult<Vec<u128>> {
        let dir = parse_direction(direction)?;
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || {
            engine
                .graph_bfs(&roots, max_depth, dir)
                .map_err(map_vanta_error)
        })
    }

    /// Depth-First-Search starting from a designated set of root IDs,
    /// up to a maximum depth, returning the discovered distinct Node IDs.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during graph traversal.
    #[pyo3(signature = (roots, max_depth=999999, direction="Forward"))]
    fn graph_dfs(
        &self,
        py: Python,
        roots: Vec<u128>,
        max_depth: usize,
        direction: &str,
    ) -> PyResult<Vec<u128>> {
        let dir = parse_direction(direction)?;
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || {
            engine
                .graph_dfs(&roots, max_depth, dir)
                .map_err(map_vanta_error)
        })
    }

    /// Performs a topological sort on the subgraph reachable from the given roots.
    /// Returns an error if a cycle is detected.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during topological sort.
    fn graph_topological_sort(&self, py: Python, roots: Vec<u128>) -> PyResult<Vec<u128>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || {
            engine
                .graph_topological_sort(&roots)
                .map_err(map_vanta_error)
        })
    }

    /// Checks if the subgraph reachable from the given roots is a Directed Acyclic Graph (DAG).
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during cycle detection.
    fn graph_is_dag(&self, py: Python, roots: Vec<u128>) -> PyResult<bool> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || engine.graph_is_dag(&roots).map_err(map_vanta_error))
    }

    /// Compute PageRank for the subgraph reachable from the given roots.
    ///
    /// Args:
    ///     roots: Starting node IDs for edge discovery.
    ///     max_iterations: Maximum iterations (default: 100).
    ///     damping: PageRank damping factor (default: 0.85).
    ///     tolerance: Convergence threshold (default: 1e-6).
    ///
    /// Returns:
    ///     A dict mapping node_id → rank.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during PageRank computation.
    #[pyo3(signature = (roots, max_iterations=100, damping=0.85, tolerance=1e-6))]
    fn graph_page_rank(
        &self,
        py: Python,
        roots: Vec<u128>,
        max_iterations: usize,
        damping: f64,
        tolerance: f64,
    ) -> PyResult<HashMap<u128, f64>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || {
            engine
                .graph_page_rank(&roots, max_iterations, damping, tolerance)
                .map_err(map_vanta_error)
        })
    }

    /// Compute degree centrality (in/out degree counts) for the subgraph
    /// reachable from the given roots.
    ///
    /// Args:
    ///     roots: Starting node IDs for edge discovery.
    ///
    /// Returns:
    ///     A dict mapping node_id → (in_degree, out_degree).
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during centrality computation.
    fn graph_degree_centrality(
        &self,
        py: Python,
        roots: Vec<u128>,
    ) -> PyResult<HashMap<u128, (usize, usize)>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || {
            engine
                .graph_degree_centrality(&roots)
                .map_err(map_vanta_error)
        })
    }

    /// Compact the storage layout: reorders nodes in BFS order to improve
    /// locality and free unused pages. Returns the number of nodes compacted.
    fn compact_layout(&self, py: Python) -> PyResult<u64> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || engine.compact_layout().map_err(map_vanta_error))
    }

    /// Recover shadow-archived nodes that belonged to a summary node.
    ///
    /// Scans TombstoneStorage for nodes with a `belonged_to` edge targeting
    /// `summary_id`, re-activates them, and inserts them back into the active store.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during tombstone scan.
    ///
    /// Args:
    ///     summary_id: The summary node ID as a decimal string (u128).
    ///
    /// Returns:
    ///     A list of recovered node dictionaries.
    #[pyo3(signature = (summary_id))]
    fn recover_archived_nodes(&self, py: Python, summary_id: &str) -> PyResult<Vec<Py<PyAny>>> {
        let sid: u128 = summary_id.parse().map_err(|_| {
            map_vanta_error(vantadb::VantaError::InvalidInput(format!(
                "Invalid summary_id: {summary_id}"
            )))
        })?;
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let nodes =
            py.detach(move || engine.recover_archived_nodes(sid).map_err(map_vanta_error))?;
        nodes.into_iter().map(|n| node_to_pydict(py, &n)).collect()
    }

    /// List all namespaces currently registered in the database.
    fn list_namespaces(&self, py: Python) -> PyResult<Vec<String>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        py.detach(move || engine.list_namespaces().map_err(map_vanta_error))
    }

    /// Generate a text snippet from a payload, highlighting matched query terms.
    #[pyo3(signature = (payload, text_query, with_highlighting=false))]
    fn generate_snippet(
        &self,
        py: Python,
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> PyResult<Option<String>> {
        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let payload = payload.to_string();
        let text_query = text_query.to_string();
        py.detach(move || {
            let result = engine.generate_snippet(&payload, &text_query, with_highlighting);
            Ok(result)
        })
    }

    /// Explain how a memory search arrives at its results — returns a detailed
    /// breakdown of the search route, fusion, and per-hit explanation.
    // PyO3 keyword argument binding requires matching function parameters in Rust.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (namespace, query_vector, filters=None, text_query=None, top_k=10, distance_metric=None))]
    fn explain_memory_search(
        &self,
        py: Python,
        namespace: &str,
        query_vector: &Bound<'_, PyAny>,
        filters: Option<&Bound<'_, PyDict>>,
        text_query: Option<String>,
        top_k: usize,
        distance_metric: Option<&str>,
    ) -> PyResult<Py<PyAny>> {
        let metric = match distance_metric {
            Some("euclidean") => DistanceMetric::Euclidean,
            Some(other) => {
                tracing::warn!(
                    "Unknown distance_metric \"{}\" — falling back to default (cosine). Known values: cosine, euclidean",
                    other
                );
                DistanceMetric::Cosine
            }
            None => DistanceMetric::Cosine,
        };

        let request = VantaMemorySearchRequest {
            namespace: namespace.to_string(),
            query_vector: extract_vector(query_vector, py)?,
            query_sparse: None,
            filters: py_dict_to_metadata(filters)?,
            text_query,
            top_k: top_k.min(MAX_K),
            distance_metric: metric,
            explain: true,
        };

        let _g = enter(&self.op_gate)?;
        let engine = self.engine.clone();
        let explanation = py.detach(move || {
            engine
                .explain_memory_search(request)
                .map_err(map_vanta_error)
        })?;

        search_explanation_to_pydict(py, &explanation)
    }
}

impl VantaDB {
    /// Read a field from a batch search request element — either a ``dict``
    /// (mapping keys) or a ``SearchRequest`` dataclass (attribute access).
    fn request_field<'py>(
        obj: &Bound<'py, PyAny>,
        key: &str,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        if let Ok(dict) = obj.cast::<PyDict>() {
            match dict.get_item(key)? {
                Some(value) if value.is_none() => Ok(None),
                Some(value) => Ok(Some(value)),
                None => Ok(None),
            }
        } else if let Ok(value) = obj.getattr(key) {
            if value.is_none() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        } else {
            Ok(None)
        }
    }

    /// Convert a Python batch-search request element (dict or `SearchRequest`
    /// dataclass) into a [`VantaMemorySearchRequest`]. Field access needs the
    /// GIL, so this runs before `py.detach` (PERF-24 pattern).
    fn parse_search_request(
        &self,
        obj: &Bound<'_, PyAny>,
        py: Python<'_>,
        default_top_k: usize,
    ) -> PyResult<VantaMemorySearchRequest> {
        let namespace: String = Self::request_field(obj, "namespace")?
            .ok_or_else(|| {
                PyValueError::new_err("search request missing required field 'namespace'")
            })?
            .extract()?;

        let query_vector = match Self::request_field(obj, "query_vector")? {
            Some(v) => extract_vector(&v, py)?,
            None => Vec::new(),
        };

        let filters = match Self::request_field(obj, "filters")? {
            Some(f) => py_dict_to_metadata(f.cast::<PyDict>().ok())?,
            None => Default::default(),
        };

        let text_query: Option<String> = match Self::request_field(obj, "text_query")? {
            Some(v) => Some(v.extract()?),
            None => None,
        };

        let top_k: usize = match Self::request_field(obj, "top_k")? {
            Some(v) => v.extract::<usize>()?.min(MAX_K),
            None => default_top_k.min(MAX_K),
        };

        let distance_metric = match Self::request_field(obj, "distance_metric")? {
            Some(v) => Some(v.extract::<String>()?),
            None => None,
        };
        let distance_metric = match distance_metric.as_deref() {
            Some("euclidean") => DistanceMetric::Euclidean,
            Some(other) => {
                tracing::warn!(
                    "Unknown distance_metric \"{}\" — falling back to default (cosine). Known values: cosine, euclidean",
                    other
                );
                DistanceMetric::Cosine
            }
            None => DistanceMetric::Cosine,
        };

        let explain: bool = match Self::request_field(obj, "explain")? {
            Some(v) => v.extract()?,
            None => false,
        };

        Ok(VantaMemorySearchRequest {
            namespace,
            query_vector,
            query_sparse: None,
            filters,
            text_query,
            top_k,
            distance_metric,
            explain,
        })
    }
}

/// Connect to a VantaDB database.
///
/// Args:
///     path: Filesystem path, empty string, or ":memory:" for in-memory.
///     memory_limit: Optional memory budget in bytes.
///         Sets an upper bound on heap usage; when exceeded, VantaDB triggers a
///         controlled flush and architection of cold data to stay within budget.
#[pyfunction]
#[pyo3(signature = (path, memory_limit=None))]
fn connect(path: &str, memory_limit: Option<u64>) -> PyResult<VantaDB> {
    use vantadb::config::VantaConfig;
    use vantadb::sdk::VantaEmbedded;
    let config = VantaConfig {
        storage_path: if path.is_empty() || path == ":memory:" {
            ":memory:".to_string()
        } else {
            path.to_string()
        },
        memory_limit,
        ..Default::default()
    };
    let engine = VantaEmbedded::open_with_config(config).map_err(map_vanta_error)?;
    Ok(VantaDB {
        engine,
        op_gate: OpGate::new(),
    })
}

/// The Python module for VantaDB.
/// Usage: `import vantadb_py`
#[pymodule]
fn vantadb_py(_py: Python, m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_class::<VantaDB>()?;
    m.add_class::<VantaVector>()?;
    m.add_class::<VantaVectorIter>()?;
    m.add_class::<VantaPySearchHit>()?;
    m.add_class::<VantaPyMemoryRecord>()?;
    m.add_class::<VantaPyListResult>()?;
    m.add_function(wrap_pyfunction!(connect, m)?)?;
    m.add("__version__", metadata::reported_version().into_owned())?;
    Ok(())
}
