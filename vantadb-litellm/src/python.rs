use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyList, PyModuleMethods};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use vantadb::config::VantaConfig;
use vantadb::error::VantaError;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};

fn err_to_py(e: VantaError) -> PyErr {
    match e {
        VantaError::NotFound { .. } => pyo3::exceptions::PyKeyError::new_err(e.to_string()),
        VantaError::BackendError(_) => pyo3::exceptions::PyIOError::new_err(e.to_string()),
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
    namespace: RwLock<String>,
    counter: AtomicU64,
}

#[pymethods]
impl VantaDBLiteLLM {
    #[new]
    #[pyo3(signature = (db_path, api_key = None, namespace = "litellm_store"))]
    fn new(db_path: &str, api_key: Option<&str>, namespace: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config).map_err(err_to_py)?;
        Ok(Self {
            engine,
            api_key: api_key.unwrap_or_default().to_string(),
            namespace: RwLock::new(namespace.to_string()),
            counter: AtomicU64::new(0),
        })
    }

    fn embed(&self, py: Python, texts: Vec<String>) -> PyResult<Vec<Vec<f32>>> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("model", "text-embedding-3-small")?;
        kwargs.set_item("input", texts)?;
        if !self.api_key.is_empty() {
            kwargs.set_item("api_key", &self.api_key)?;
        }
        let response = pyo3::types::PyModule::import(py, "litellm")
            .and_then(|m| m.getattr("embed"))
            .and_then(|func| func.call((), Some(&kwargs)))
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
                .and_then(|v| v.extract::<Vec<f64>>().ok())
                .ok_or_else(|| PyRuntimeError::new_err("missing embedding"))?
                .into_iter()
                .map(|x| x as f32)
                .collect();
            result.push(emb);
        }
        Ok(result)
    }

    fn search(
        &self,
        py: Python,
        query_embedding: Vec<f32>,
        top_k: i32,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let namespace = self.namespace.read().unwrap().clone();
        let request = VantaMemorySearchRequest {
            namespace,
            query_vector: query_embedding,
            filters: Default::default(),
            text_query: None,
            top_k: top_k as usize,
            distance_metric: vantadb::DistanceMetric::Cosine,
            explain: false,
        };

        let hits: Vec<vantadb::sdk::VantaMemorySearchHit> = py
            .detach(|| self.engine.search(request))
            .map_err(|e: VantaError| PyRuntimeError::new_err(format!("search error: {:?}", e)))?;

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

    fn store(
        &self,
        text: &str,
        embedding: Vec<f32>,
        metadata: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<String> {
        let namespace = self.namespace.read().unwrap().clone();
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        let key = format!("litellm_{n}");
        let mut input = VantaMemoryInput::new(&namespace, &key, text);
        input.vector = Some(embedding);

        if let Some(meta) = metadata {
            for (k, v) in meta.iter() {
                if let (Ok(key), Ok(val)) = (k.extract::<String>(), v.extract::<String>()) {
                    input
                        .metadata
                        .insert(key, vantadb::sdk::VantaValue::String(val));
                }
            }
        }

        let record = self.engine.put(input).map_err(err_to_py)?;
        Ok(format!("{}:{}", record.namespace, record.key))
    }
}

#[pymodule]
fn vantadb_litellm(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBLiteLLM>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
