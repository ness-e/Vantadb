//! Python bindings for the VantaDB vector-graph database via PyO3.
//!
//! This crate exposes the [`VantaDB`] class and a [`connect`] function
//! for in-process, zero-network-overhead access to VantaDB from Python.
#![warn(missing_docs)]
#![allow(deprecated)]

use pyo3::buffer::PyBuffer;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods, PyModuleMethods, PyTuple, PyTupleMethods};
use std::collections::HashMap;
use vantadb::config::VantaConfig;
use vantadb::metadata;
use vantadb::sdk::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemorySearchRequest,
    VantaNodeInput, VantaValue,
};
use vantadb::DistanceMetric;

mod convert;
mod types;
mod vector;

use types::{FlatBufferView, VantaPyListResult, VantaPyMemoryRecord, VantaPySearchHit};

use vector::{VantaVector, VantaVectorIter};

use crate::convert::{
    capabilities_to_pydict, check_lens, export_report_to_pydict, extract_vector,
    format_query_result, import_report_to_pydict, map_vanta_error, node_to_pydict,
    operational_metrics_to_pydict, py_any_to_value, py_dict_to_metadata, rebuild_report_to_pydict,
    runtime_profile_label, search_explanation_to_pydict, text_index_audit_report_to_pydict,
    text_index_repair_report_to_pydict,
};

#[pyclass]
/// Python-accessible embedded VantaDB engine.
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

        Ok(VantaDB { engine })
    }

    /// Insert a node with content and an optional embedding vector.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during node insert.
    ///
    /// Args:
    ///     id: Unique node identifier (u64).
    ///     content: Text content stored as a relational field.
    ///     vector: Embedding vector (list of floats). Pass empty list for no vector.
    ///     fields: Optional dict of additional relational fields.
    #[pyo3(signature = (id, content, vector, fields=None))]
    fn insert(
        &self,
        py: Python,
        id: u128,
        content: &str,
        vector: &Bound<'_, PyAny>,
        fields: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
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
    ///
    /// Returns a list of ``VantaMemoryRecord`` objects, up to ~5x faster
    /// than sequential ``put()`` for large batches.
    #[deprecated(
        note = "use keyword arguments (keys=..., vectors=..., payloads=..., metadatas=..., namespace=..., ttls=...) instead"
    )]
    #[allow(deprecated)]
    #[pyo3(signature = (entries, keys=None, vectors=None, payloads=None, metadatas=None, namespace=None, ttls=None))]
    fn put_batch(
        &self,
        py: Python,
        entries: Option<&Bound<'_, PyAny>>,
        keys: Option<Vec<String>>,
        vectors: Option<Vec<Vec<f32>>>,
        payloads: Option<Vec<String>>,
        metadatas: Option<Vec<Option<HashMap<String, String>>>>,
        namespace: Option<String>,
        ttls: Option<Vec<Option<u64>>>,
    ) -> PyResult<Vec<VantaPyMemoryRecord>> {
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

        let ns = namespace.unwrap_or_else(|| "default".to_string());
        let mut inputs = Vec::with_capacity(n);
        for i in 0..n {
            let payload = match &payloads {
                Some(p) => p[i].clone(),
                None => String::new(),
            };
            let mut input = VantaMemoryInput::new(ns.clone(), keys[i].clone(), payload);

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
    fn get_memory(
        &self,
        py: Python,
        namespace: &str,
        key: &str,
    ) -> PyResult<Option<VantaPyMemoryRecord>> {
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
    fn delete_memory(&self, py: Python, namespace: &str, key: &str) -> PyResult<bool> {
        let engine = self.engine.clone();
        let namespace = namespace.to_string();
        let key = key.to_string();
        py.detach(move || engine.delete(&namespace, &key).map_err(map_vanta_error))
    }

    /// List namespace-scoped persistent memory records.
    #[pyo3(signature = (namespace, filters=None, limit=100, cursor=None))]
    fn list_memory(
        &self,
        py: Python,
        namespace: &str,
        filters: Option<&Bound<'_, PyDict>>,
        limit: usize,
        cursor: Option<usize>,
    ) -> PyResult<VantaPyListResult> {
        let namespace = namespace.to_string();
        let filters_meta = py_dict_to_metadata(filters)?;
        let engine = self.engine.clone();
        let page = py.detach(move || {
            engine
                .list(
                    &namespace,
                    VantaMemoryListOptions {
                        filters: filters_meta,
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
            filters: py_dict_to_metadata(filters)?,
            text_query,
            top_k,
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
    fn rebuild_index(&self, py: Python) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let report = py.detach(move || engine.rebuild_index().map_err(map_vanta_error))?;
        rebuild_report_to_pydict(py, &report)
    }

    /// Export one namespace as JSONL.
    fn export_namespace(&self, py: Python, path: &str, namespace: &str) -> PyResult<Py<PyAny>> {
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
    fn export_all(&self, py: Python, path: &str) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let path = path.to_string();
        let report = py.detach(move || engine.export_all(&path).map_err(map_vanta_error))?;
        export_report_to_pydict(py, &report)
    }

    /// Import records from a VantaDB memory JSONL export.
    fn import_file(&self, py: Python, path: &str) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let path = path.to_string();
        let report = py.detach(move || engine.import_file(&path).map_err(map_vanta_error))?;
        import_report_to_pydict(py, &report)
    }

    /// Run a read-only structural audit of the derived text index.
    #[pyo3(signature = (namespace=None, deep=false))]
    fn audit_text_index(
        &self,
        py: Python,
        namespace: Option<&str>,
        deep: bool,
    ) -> PyResult<Py<PyAny>> {
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
        let engine = self.engine.clone();
        let report = py.detach(move || engine.repair_text_index().map_err(map_vanta_error))?;
        text_index_repair_report_to_pydict(py, &report)
    }

    /// Return operational metrics for startup, replay, rebuild, export, and import.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during metrics snapshot.
    fn operational_metrics(&self, py: Python) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let metrics = py.detach(move || engine.operational_metrics());
        operational_metrics_to_pydict(py, &metrics)
    }

    /// Retrieve a node by ID. Returns a dict or None.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during database retrieval.
    fn get(&self, py: Python, id: u128) -> PyResult<Option<Py<PyAny>>> {
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
    fn delete(&self, py: Python, id: u64, reason: &str) -> PyResult<()> {
        let engine = self.engine.clone();
        let reason_str = reason.to_string();
        py.detach(move || {
            engine
                .delete_node(id.into(), &reason_str)
                .map_err(map_vanta_error)
        })
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
    ) -> PyResult<Vec<(u64, f32)>> {
        // PERF-24: vector extraction (needs GIL) — done before detach
        let v = extract_vector(vector, py)?;
        let engine = self.engine.clone();
        // GIL RELEASED: only pure Rust — distance computation + graph traversal
        py.detach(move || {
            engine
                .search_vector(&v, top_k)
                .map(|hits| {
                    // Pure Rust tuple mapping — no Python objects created
                    hits.into_iter()
                        .map(|hit| (hit.node_id as u64, hit.distance))
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
    ) -> PyResult<Vec<Vec<(u64, f32)>>> {
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
                        .search_vector(&vector, top_k)
                        .map(|hits| {
                            hits.into_iter()
                                .map(|hit| (hit.node_id as u64, hit.distance))
                                .collect()
                        })
                        .map_err(map_vanta_error)
                })
                .collect::<Result<Vec<Vec<(u64, f32)>>, _>>()
        })
    }

    /// Execute an IQL or LISP query string. Returns a formatted result string.
    ///
    /// GIL Policy: RELEASED during Tokio execution — allows other Python
    /// threads to run while VantaDB processes the query.
    fn query(&self, py: Python, iql_query: &str) -> PyResult<String> {
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
        let engine = self.engine.clone();
        py.detach(move || engine.flush().map_err(map_vanta_error))
    }

    /// Compact the WAL: flush, archive ``vanta.wal`` as
    /// ``vanta.wal.<timestamp>``, and start a fresh WAL.
    #[pyo3(signature = ())]
    fn compact_wal(&self, py: Python) -> PyResult<()> {
        let engine = self.engine.clone();
        py.detach(move || engine.compact_wal().map_err(map_vanta_error))
    }

    /// Scan all memory records and physically delete expired ones.
    /// Returns the number of records purged.
    #[pyo3(signature = ())]
    fn purge_expired(&self, py: Python) -> PyResult<u64> {
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
    #[pyo3(signature = (source_id, target_id, label, weight=None))]
    fn add_edge(
        &self,
        py: Python,
        source_id: u128,
        target_id: u128,
        label: &str,
        weight: Option<f32>,
    ) -> PyResult<()> {
        let engine = self.engine.clone();
        let label_str = label.to_string();
        py.detach(move || {
            engine
                .add_edge(source_id, target_id, &label_str, weight)
                .map_err(map_vanta_error)
        })
    }

    /// Flush and close the embedded engine handle.
    fn close(&self, py: Python) -> PyResult<()> {
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
    #[pyo3(signature = (roots, max_depth=999999))]
    fn graph_bfs(&self, py: Python, roots: Vec<u128>, max_depth: usize) -> PyResult<Vec<u128>> {
        let engine = self.engine.clone();
        py.detach(move || engine.graph_bfs(&roots, max_depth).map_err(map_vanta_error))
    }

    /// Depth-First-Search starting from a designated set of root IDs,
    /// up to a maximum depth, returning the discovered distinct Node IDs.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during graph traversal.
    #[pyo3(signature = (roots, max_depth=999999))]
    fn graph_dfs(&self, py: Python, roots: Vec<u128>, max_depth: usize) -> PyResult<Vec<u128>> {
        let engine = self.engine.clone();
        py.detach(move || engine.graph_dfs(&roots, max_depth).map_err(map_vanta_error))
    }

    /// Performs a topological sort on the subgraph reachable from the given roots.
    /// Returns an error if a cycle is detected.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during topological sort.
    fn graph_topological_sort(&self, py: Python, roots: Vec<u128>) -> PyResult<Vec<u128>> {
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
        let engine = self.engine.clone();
        py.detach(move || engine.graph_is_dag(&roots).map_err(map_vanta_error))
    }

    /// Compact the storage layout: reorders nodes in BFS order to improve
    /// locality and free unused pages. Returns the number of nodes compacted.
    fn compact_layout(&self, py: Python) -> PyResult<u64> {
        let engine = self.engine.clone();
        py.detach(move || engine.compact_layout().map_err(map_vanta_error))
    }

    /// List all namespaces currently registered in the database.
    fn list_namespaces(&self, py: Python) -> PyResult<Vec<String>> {
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
            filters: py_dict_to_metadata(filters)?,
            text_query,
            top_k,
            distance_metric: metric,
            explain: true,
        };

        let engine = self.engine.clone();
        let explanation = py.detach(move || {
            engine
                .explain_memory_search(request)
                .map_err(map_vanta_error)
        })?;

        search_explanation_to_pydict(py, &explanation)
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
    Ok(VantaDB { engine })
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
