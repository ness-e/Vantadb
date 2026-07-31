use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyList, PyModuleMethods};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use vantadb::config::VantaConfig;
use vantadb::error::VantaError;
use vantadb::sdk::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemorySearchRequest, VantaValue,
};

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

/// Ollama local embedding wrapper with VantaDB storage.
///
/// Generates embeddings via Ollama's local API and stores/searches them in VantaDB.
///
/// Usage::
///
///     from vantadb_ollama import VantaDBOllama
///     store = VantaDBOllama("/tmp/vantadb-ollama")
///     emb = store.embed(["hello world"])
///     store.store("hello world", emb[0])
///     results = store.search(emb[0], top_k=5)
#[pyclass(name = "VantaDBOllama")]
pub struct VantaDBOllama {
    engine: VantaEmbedded,
    client: Py<PyAny>,
    model: String,
    namespace: String,
    #[allow(dead_code)] // ponytail: passed to ollama.Client constructor, verified at client level
    timeout: Option<f64>,
}

#[pymethods]
impl VantaDBOllama {
    /// Creates a new VantaDB Ollama provider.
    ///
    /// Args:
    ///     db_path: Path to VantaDB storage directory.
    ///     base_url: Ollama server URL (default: "http://localhost:11434").
    ///     model: Ollama embedding model name (default: "nomic-embed-text").
    ///     namespace: Default namespace for all operations (default: "ollama_store").
    ///
    /// Returns:
    ///     A new VantaDBOllama instance.
    #[new]
    #[pyo3(signature = (db_path, base_url = "http://localhost:11434", model = "nomic-embed-text", namespace = "ollama_store", timeout = None))]
    fn new(
        py: Python,
        db_path: &str,
        base_url: &str,
        model: &str,
        namespace: &str,
        timeout: Option<f64>,
    ) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config).map_err(err_to_py)?;
        let ollama_mod = pyo3::types::PyModule::import(py, "ollama")
            .map_err(|e| PyRuntimeError::new_err(format!("ollama import error: {:?}", e)))?;
        let client_kwargs = PyDict::new(py);
        client_kwargs.set_item("host", base_url)?;
        if let Some(t) = timeout {
            client_kwargs.set_item("timeout", t)?;
        }
        let client = ollama_mod
            .getattr("Client")
            .and_then(|cls| cls.call((), Some(&client_kwargs)))
            .map_err(|e| PyRuntimeError::new_err(format!("Ollama client error: {:?}", e)))?;
        Ok(Self {
            engine,
            client: client.unbind(),
            model: model.to_string(),
            namespace: namespace.to_string(),
            timeout,
        })
    }

    /// Generate embeddings for a list of texts using Ollama.
    ///
    /// Args:
    ///     texts: List of strings to embed.
    ///
    /// Returns:
    ///     A list of embedding vectors, one per input text.
    fn embed(&self, py: Python, texts: Vec<String>) -> PyResult<Vec<Vec<f32>>> {
        let client = self.client.bind(py);
        let kwargs = PyDict::new(py);
        kwargs.set_item("model", &self.model)?;
        kwargs.set_item("input", texts)?;
        let response = client
            .getattr("embed")
            .and_then(|func| func.call((), Some(&kwargs)))
            .map_err(|e| PyRuntimeError::new_err(format!("Ollama embed error: {:?}", e)))?;

        response
            .get_item("embeddings")
            .and_then(|v| v.extract::<Vec<Vec<f32>>>())
            .map_err(|e| PyRuntimeError::new_err(format!("missing embeddings: {:?}", e)))
    }

    /// Search for similar records by vector similarity with optional filters.
    ///
    /// Args:
    ///     namespace: Namespace to search in.
    ///     query_embedding: The embedding vector to search with.
    ///     text_query: Optional text query for BM25 lexical search.
    ///     filters: Optional metadata filters (string key-value pairs).
    ///     distance_metric: Distance metric ("cosine" or "euclidean"/"l2").
    ///
    /// Returns:
    ///     A list of dicts with ``id``, ``text``, and ``score`` keys.
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
        };

        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust search
        let hits = py.detach(move || engine.search(request).map_err(err_to_py))?;

        let mut results = Vec::with_capacity(hits.len());
        for hit in hits {
            let d = PyDict::new(py);
            d.set_item("id", format!("{}:{}", hit.record.namespace, hit.record.key))?;
            d.set_item("text", &hit.record.payload)?;
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
        let key = format!("ollama_{ts}");
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

    /// Delete a record by its key, optionally in a specific namespace.
    ///
    /// Args:
    ///     key: The record key to delete.
    ///     namespace: Optional namespace override. Uses the default namespace if not provided.
    ///
    /// Returns:
    ///     True if the record was deleted, False if not found.
    #[pyo3(signature = (key, namespace = None))]
    fn delete(&self, py: Python, key: &str, namespace: Option<String>) -> PyResult<bool> {
        let ns = namespace.unwrap_or(self.namespace.clone());
        let engine = self.engine.clone();
        py.detach(move || engine.delete(&ns, key).map_err(err_to_py))
    }

    /// Retrieve a memory record by namespace and key.
    ///
    /// Args:
    ///     namespace: The namespace to read from.
    ///     key: The record key to retrieve.
    ///
    /// Returns:
    ///     A dict with ``namespace``, ``key``, ``text``, and timestamps, or None if not found.
    #[pyo3(signature = (namespace, key))]
    fn get(&self, py: Python, namespace: &str, key: &str) -> PyResult<Option<Py<PyAny>>> {
        let engine = self.engine.clone();
        let ns = namespace.to_string();
        let k = key.to_string();
        let result = py.detach(move || engine.get(&ns, &k).map_err(err_to_py))?;

        match result {
            Some(record) => {
                let d = PyDict::new(py);
                d.set_item("namespace", &record.namespace)?;
                d.set_item("key", &record.key)?;
                d.set_item("text", &record.payload)?;
                d.set_item("created_at_ms", record.created_at_ms)?;
                d.set_item("updated_at_ms", record.updated_at_ms)?;
                d.set_item("version", record.version)?;
                if let Some(ref vector) = record.vector {
                    d.set_item("vector", vector)?;
                }
                if let Some(expires) = record.expires_at_ms {
                    d.set_item("expires_at_ms", expires)?;
                }
                Ok(Some(d.unbind().into()))
            }
            None => Ok(None),
        }
    }

    /// List memory records in a namespace with cursor-based pagination.
    ///
    /// Args:
    ///     namespace: The namespace to list.
    ///     limit: Maximum number of results (default: 100).
    ///     cursor: Optional cursor string for pagination.
    ///
    /// Returns:
    ///     A dict with ``records`` (list of dicts) and optional ``cursor`` for next page.
    #[pyo3(signature = (namespace, limit = 100, cursor = None))]
    fn list(
        &self,
        py: Python,
        namespace: &str,
        limit: usize,
        cursor: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let options = VantaMemoryListOptions {
            #[allow(deprecated)]
            filters: vantadb::sdk::VantaMemoryMetadata::new(),
            filter_ops: None,
            limit,
            cursor,
        };

        let engine = self.engine.clone();
        let ns = namespace.to_string();
        let page = py.detach(move || engine.list(&ns, options).map_err(err_to_py))?;

        let d = PyDict::new(py);
        let records_list = PyList::empty(py);
        for record in page.records {
            let rd = PyDict::new(py);
            rd.set_item("namespace", &record.namespace)?;
            rd.set_item("key", &record.key)?;
            rd.set_item("text", &record.payload)?;
            let meta = PyDict::new(py);
            for (mk, mv) in &record.metadata {
                match mv {
                    vantadb::sdk::VantaValue::String(s) => meta.set_item(mk, s)?,
                    vantadb::sdk::VantaValue::Int(i) => meta.set_item(mk, i)?,
                    vantadb::sdk::VantaValue::Float(f) => meta.set_item(mk, f)?,
                    vantadb::sdk::VantaValue::Bool(b) => meta.set_item(mk, b)?,
                    other => meta.set_item(mk, format!("{:?}", other))?,
                };
            }
            rd.set_item("metadata", meta)?;
            records_list.append(rd)?;
        }
        d.set_item("records", records_list)?;
        if let Some(next_cursor) = page.next_cursor {
            d.set_item("cursor", next_cursor.to_string())?;
        }
        Ok(d.unbind().into())
    }

    /// List all namespaces that contain at least one memory record.
    fn list_namespaces(&self, py: Python) -> PyResult<Vec<String>> {
        let engine = self.engine.clone();
        py.detach(move || engine.list_namespaces().map_err(err_to_py))
    }
}

#[pymodule]
fn vantadb_ollama(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBOllama>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
