use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyList, PyModule, PyModuleMethods};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use vantadb::config::VantaConfig;
use vantadb::error::VantaError;
use vantadb::sdk::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryRecord,
    VantaMemorySearchRequest, VantaValue,
};

fn record_to_pydict<'py>(py: Python<'py>, r: VantaMemoryRecord) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("namespace", &r.namespace)?;
    d.set_item("key", &r.key)?;
    d.set_item("payload", &r.payload)?;
    let meta = PyDict::new(py);
    for (mk, mv) in &r.metadata {
        match mv {
            vantadb::sdk::VantaValue::String(s) => meta.set_item(mk, s)?,
            vantadb::sdk::VantaValue::Int(i) => meta.set_item(mk, i)?,
            vantadb::sdk::VantaValue::Float(f) => meta.set_item(mk, f)?,
            vantadb::sdk::VantaValue::Bool(b) => meta.set_item(mk, b)?,
            other => meta.set_item(mk, format!("{:?}", other))?,
        };
    }
    d.set_item("metadata", meta)?;
    d.set_item("created_at_ms", r.created_at_ms)?;
    d.set_item("updated_at_ms", r.updated_at_ms)?;
    d.set_item("version", r.version)?;
    d.set_item("node_id", r.node_id)?;
    if let Some(ref v) = r.vector {
        d.set_item("vector", v.clone())?;
    }
    if let Some(exp) = r.expires_at_ms {
        d.set_item("expires_at_ms", exp)?;
    }
    Ok(d)
}

fn err_to_py(e: VantaError) -> PyErr {
    match e {
        VantaError::NotFound { .. } => pyo3::exceptions::PyKeyError::new_err(e.to_string()),
        VantaError::BackendError(_) => PyRuntimeError::new_err(e.to_string()),
        VantaError::InvalidInput(_)
        | VantaError::SchemaError(_)
        | VantaError::SerializationError(_) => {
            pyo3::exceptions::PyValueError::new_err(e.to_string())
        }
        _ => PyRuntimeError::new_err(format!("{:?}", e)),
    }
}

/// LiteLLM embedding gateway with VantaDB storage.
///
/// Uses LiteLLM as a unified embedding provider and stores/searches vectors in VantaDB.
///
/// Usage::
///
///     from vantadb_litellm import VantaDBLiteLLM
///     store = VantaDBLiteLLM("/tmp/vantadb-litellm")
///     emb = store.embed(["hello world"])
///     store.store("hello world", emb[0])
///     results = store.search(emb[0], top_k=5)
#[pyclass(name = "VantaDBLiteLLM")]
pub struct VantaDBLiteLLM {
    engine: VantaEmbedded,
    api_key: String,
    embed_fn: Option<Py<PyAny>>,
    model: String,
    namespace: String,
    #[allow(dead_code)]
    timeout: Option<f64>,
}

#[pymethods]
impl VantaDBLiteLLM {
    /// Creates a new VantaDB LiteLLM provider.
    ///
    /// Args:
    ///     db_path: Path to VantaDB storage directory.
    ///     api_key: Optional API key for the embedding provider.
    ///     model: Embedding model name (default: "text-embedding-3-small").
    ///     namespace: Default namespace for all operations (default: "litellm_store").
    ///
    /// Returns:
    ///     A new VantaDBLiteLLM instance.
    #[new]
    #[pyo3(signature = (db_path, api_key = None, model = "text-embedding-3-small", namespace = "litellm_store", timeout = None))]
    fn new(
        py: Python,
        db_path: &str,
        api_key: Option<&str>,
        model: &str,
        namespace: &str,
        timeout: Option<f64>,
    ) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config).map_err(err_to_py)?;
        let litellm = PyModule::import(py, "litellm")
            .map_err(|e| PyRuntimeError::new_err(format!("litellm import error: {:?}", e)))?;
        let embed_fn = Some(litellm.getattr("embedding")?.unbind());
        Ok(Self {
            engine,
            api_key: api_key.unwrap_or_default().to_string(),
            embed_fn,
            model: model.to_string(),
            namespace: namespace.to_string(),
            timeout,
        })
    }

    /// Generate embeddings for a list of texts using LiteLLM.
    ///
    /// Args:
    ///     texts: List of strings to embed.
    ///
    /// Returns:
    ///     A list of embedding vectors, one per input text.
    fn embed(&self, py: Python, texts: Vec<String>) -> PyResult<Vec<Vec<f32>>> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("model", &self.model)?;
        kwargs.set_item("input", texts)?;
        if !self.api_key.is_empty() {
            kwargs.set_item("api_key", &self.api_key)?;
        }
        let func = self.embed_fn.as_ref().unwrap().bind(py);
        let response = func
            .call((), Some(&kwargs))
            .map_err(|e| PyRuntimeError::new_err(format!("liteLLM embed error: {:?}", e)))?;

        let data = response
            .get_item("data")
            .map_err(|e| PyRuntimeError::new_err(format!("missing data: {:?}", e)))?;
        let data_list = data.cast::<PyList>()?;

        let mut result = Vec::with_capacity(data_list.len());
        for item in data_list.iter() {
            let d = item.cast::<PyDict>()?;
            let emb: Vec<f32> = d
                .get_item("embedding")
                .ok()
                .flatten()
                .and_then(|v| v.extract::<Vec<f32>>().ok())
                .ok_or_else(|| PyRuntimeError::new_err("missing embedding"))?;
            result.push(emb);
        }
        Ok(result)
    }

    /// Retrieve a memory record by namespace and key.
    #[pyo3(signature = (namespace, key))]
    fn get(&self, py: Python, namespace: &str, key: &str) -> PyResult<Option<Py<PyAny>>> {
        let namespace = namespace.to_string();
        let key = key.to_string();
        let engine = self.engine.clone();
        let record = py.detach(move || engine.get(&namespace, &key).map_err(err_to_py))?;
        record
            .map(|r| record_to_pydict(py, r).map(|d| d.unbind().into()))
            .transpose()
    }

    /// List memory records in a namespace with cursor-based pagination.
    #[pyo3(signature = (namespace, limit = 100, cursor = None))]
    fn list(
        &self,
        py: Python,
        namespace: &str,
        limit: usize,
        cursor: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let namespace = namespace.to_string();
        let engine = self.engine.clone();
        let page = py.detach(move || {
            engine
                .list(
                    &namespace,
                    VantaMemoryListOptions {
                        #[allow(deprecated)]
                        filters: vantadb::sdk::VantaMemoryMetadata::new(),
                        filter_ops: None,
                        limit,
                        cursor,
                        exclude_superseded: false,                    },
                )
                .map_err(err_to_py)
        })?;
        let records: Vec<Py<PyAny>> = page
            .records
            .into_iter()
            .map(|r| record_to_pydict(py, r).map(|d| d.unbind().into()))
            .collect::<PyResult<_>>()?;
        let result = PyDict::new(py);
        result.set_item("records", records)?;
        result.set_item("next_cursor", page.next_cursor)?;
        Ok(result.unbind().into())
    }

    /// Search for similar records by vector similarity with optional filters.
    ///
    /// Args:
    ///     namespace: Namespace to search in.
    ///     query_embedding: The embedding vector to search with.
    ///     text_query: Optional text query for BM25 lexical search.
    ///     filters: Optional metadata filters (string key-value pairs).
    ///     distance_metric: Distance metric ("cosine" or "euclidean"/"l2").
    ///     top_k: Number of results to return (default: 10).
    ///
    /// Returns:
    ///     A list of dicts with full record fields plus ``score``.
    #[pyo3(signature = (namespace, query_embedding, text_query = None, filters = None, distance_metric = None, top_k = 10))]
    fn search(
        &self,
        py: Python,
        namespace: &str,
        query_embedding: Vec<f32>,
        text_query: Option<String>,
        filters: Option<HashMap<String, String>>,
        distance_metric: Option<String>,
        top_k: usize,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let request = VantaMemorySearchRequest {
            namespace: namespace.to_string(),
            query_vector: query_embedding,
            filters: filters
                .unwrap_or_default()
                .into_iter()
                .map(|(k, v)| (k, VantaValue::String(v)))
                .collect(),
            text_query,
            top_k,
            distance_metric: match distance_metric.as_deref() {
                Some("euclidean" | "l2") => vantadb::DistanceMetric::Euclidean,
                _ => vantadb::DistanceMetric::Cosine,
            },
            explain: false,
            query_sparse: None,
        };

        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust search
        let hits = py.detach(move || engine.search(request).map_err(err_to_py))?;

        let mut results = Vec::with_capacity(hits.len());
        for hit in hits {
            let d = record_to_pydict(py, hit.record)?;
            d.set_item("score", hit.score)?;
            results.push(d.unbind().into());
        }
        Ok(results)
    }

    /// Store a text record with its embedding vector in VantaDB.
    ///
    /// Args:
    ///     text: The text content to store.
    ///     embedding: The embedding vector for this text.
    ///     metadata: Optional metadata dict (string keys, string/bool/int/float values).
    ///
    /// Returns:
    ///     The record ID as ``namespace:key``.
    fn store(
        &self,
        py: Python,
        text: &str,
        embedding: Vec<f32>,
        metadata: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<String> {
        let namespace = self.namespace.clone();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let key = format!("litellm_{ts}");
        let mut input = VantaMemoryInput::new(&namespace, &key, text);
        input.vector = Some(embedding);

        if let Some(meta) = metadata {
            for (k, v) in meta.iter() {
                if let Ok(key) = k.extract::<String>() {
                    let val = v
                        .extract::<String>()
                        .ok()
                        .map(vantadb::sdk::VantaValue::String)
                        .or_else(|| v.extract::<bool>().ok().map(vantadb::sdk::VantaValue::Bool))
                        .or_else(|| v.extract::<i64>().ok().map(vantadb::sdk::VantaValue::Int))
                        .or_else(|| v.extract::<f64>().ok().map(vantadb::sdk::VantaValue::Float));
                    if let Some(val) = val {
                        input.metadata.insert(key, val);
                    }
                }
            }
        }

        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust insert
        let record = py.detach(move || engine.put(input).map_err(err_to_py))?;
        Ok(format!("{}:{}", record.namespace, record.key))
    }

    /// Delete a record by its key, optionally specifying a namespace.
    ///
    /// Args:
    ///     key: The record key to delete.
    ///     namespace: Optional namespace override (defaults to the instance namespace).
    ///
    /// Returns:
    ///     True if the record was deleted, False if not found.
    #[pyo3(signature = (key, namespace = None))]
    fn delete(&self, py: Python, key: &str, namespace: Option<String>) -> PyResult<bool> {
        let namespace = namespace.unwrap_or(self.namespace.clone());
        let engine = self.engine.clone();
        py.detach(move || engine.delete(&namespace, key).map_err(err_to_py))
    }

    /// List all namespaces that contain at least one memory record.
    fn list_namespaces(&self, py: Python) -> PyResult<Vec<String>> {
        let engine = self.engine.clone();
        py.detach(move || engine.list_namespaces().map_err(err_to_py))
    }
}

#[pymodule]
fn vantadb_litellm(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBLiteLLM>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
