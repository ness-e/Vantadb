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

/// OpenAI embedding wrapper with VantaDB storage.
///
/// Generates embeddings via OpenAI's API and stores/searches them in VantaDB.
///
/// Usage::
///
///     from vantadb_openai import VantaDBOpenAI
///     store = VantaDBOpenAI("/tmp/vantadb-openai", "sk-...")
///     emb = store.embed(["hello world"])
///     store.store("hello world", emb[0])
///     results = store.search(emb[0], top_k=5)
#[pyclass(name = "VantaDBOpenAI")]
pub struct VantaDBOpenAI {
    engine: VantaEmbedded,
    client: Py<PyAny>,
    model: String,
    namespace: String,
    #[allow(dead_code)]
    timeout: Option<f64>,
}

#[pymethods]
impl VantaDBOpenAI {
    /// Creates a new VantaDB OpenAI provider.
    ///
    /// Args:
    ///     db_path: Path to VantaDB storage directory.
    ///     api_key: OpenAI API key.
    ///     model: OpenAI embedding model name (default: "text-embedding-3-small").
    ///     namespace: Default namespace for all operations (default: "openai_store").
    ///
    /// Returns:
    ///     A new VantaDBOpenAI instance.
    #[new]
    #[pyo3(signature = (db_path, api_key, model = "text-embedding-3-small", namespace = "openai_store", timeout = None))]
    fn new(
        py: Python,
        db_path: &str,
        api_key: &str,
        model: &str,
        namespace: &str,
        timeout: Option<f64>,
    ) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config).map_err(err_to_py)?;
        let openai_mod = pyo3::types::PyModule::import(py, "openai")
            .map_err(|e| PyRuntimeError::new_err(format!("openai import error: {:?}", e)))?;
        let client_kwargs = PyDict::new(py);
        client_kwargs.set_item("api_key", api_key)?;
        if let Some(t) = timeout {
            client_kwargs.set_item("timeout", t)?;
        }
        let client = openai_mod
            .getattr("OpenAI")
            .and_then(|cls| cls.call((), Some(&client_kwargs)))
            .map_err(|e| PyRuntimeError::new_err(format!("OpenAI client error: {:?}", e)))?;
        Ok(Self {
            engine,
            client: client.unbind(),
            model: model.to_string(),
            namespace: namespace.to_string(),
            timeout,
        })
    }

    /// Generate embeddings for a list of texts using OpenAI.
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
            .getattr("embeddings")
            .and_then(|e| e.getattr("create"))
            .and_then(|func| func.call((), Some(&kwargs)))
            .map_err(|e| PyRuntimeError::new_err(format!("embed API error: {:?}", e)))?;

        let data = response
            .get_item("data")
            .map_err(|e| PyRuntimeError::new_err(format!("missing data: {:?}", e)))?;
        let data_list = data.cast::<PyList>()?;

        let mut result = Vec::with_capacity(data_list.len());
        for item in data_list.iter() {
            let v: Vec<f32> = item.get_item("embedding")?.extract()?;
            result.push(v);
        }
        Ok(result)
    }

    /// Search for similar records by vector similarity.
    ///
    /// Args:
    ///     query_embedding: The embedding vector to search with.
    ///     top_k: Number of top results to return.
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
        let key = format!("openai_{ts}");
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
    ///     namespace: Optional namespace override. Defaults to the instance namespace.
    ///
    /// Returns:
    ///     True if the record was deleted, False if not found.
    #[pyo3(signature = (key, namespace = None))]
    fn delete(&self, py: Python, key: &str, namespace: Option<String>) -> PyResult<bool> {
        let namespace = namespace.unwrap_or(self.namespace.clone());
        let engine = self.engine.clone();
        py.detach(move || engine.delete(&namespace, key).map_err(err_to_py))
    }

    /// Retrieve a single record by namespace and key.
    /// Returns a dict with full record fields, or `None` if not found.
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

    /// List records in a namespace with cursor-based pagination.
    /// Returns a dict with `records` (list of dicts with full record fields) and `next_cursor`.
    #[pyo3(signature = (namespace, limit = 100, cursor = None))]
    fn list(
        &self,
        py: Python,
        namespace: &str,
        limit: i32,
        cursor: Option<i32>,
    ) -> PyResult<Py<PyDict>> {
        let engine = self.engine.clone();
        let ns = namespace.to_string();
        let options = VantaMemoryListOptions {
            filters: Default::default(),
            limit: limit.max(1) as usize,
            cursor: cursor.map(|c| c.max(0) as usize),
        };
        let page = py.detach(move || engine.list(&ns, options).map_err(err_to_py))?;

        let d = PyDict::new(py);
        let records = PyList::empty(py);
        for record in &page.records {
            let rd = PyDict::new(py);
            rd.set_item("namespace", &record.namespace)?;
            rd.set_item("key", &record.key)?;
            rd.set_item("text", &record.payload)?;
            rd.set_item("created_at_ms", record.created_at_ms)?;
            rd.set_item("updated_at_ms", record.updated_at_ms)?;
            rd.set_item("version", record.version)?;
            if let Some(ref vector) = record.vector {
                rd.set_item("vector", vector)?;
            }
            if let Some(expires) = record.expires_at_ms {
                rd.set_item("expires_at_ms", expires)?;
            }
            records.append(rd)?;
        }
        d.set_item("records", records)?;
        d.set_item("next_cursor", page.next_cursor.map(|c| c as i32))?;
        Ok(d.unbind())
    }

    /// List all namespaces that contain at least one memory record.
    fn list_namespaces(&self, py: Python) -> PyResult<Vec<String>> {
        let engine = self.engine.clone();
        py.detach(move || engine.list_namespaces().map_err(err_to_py))
    }
}

#[pymodule]
fn vantadb_openai(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBOpenAI>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
