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
    namespace: RwLock<String>,
    counter: AtomicU64,
}

#[pymethods]
impl VantaDBOpenAI {
    #[new]
    #[pyo3(signature = (db_path, api_key, model = "text-embedding-3-small", namespace = "openai_store"))]
    fn new(
        py: Python,
        db_path: &str,
        api_key: &str,
        model: &str,
        namespace: &str,
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
        let client = openai_mod
            .getattr("OpenAI")
            .and_then(|cls| cls.call((), Some(&client_kwargs)))
            .map_err(|e| PyRuntimeError::new_err(format!("OpenAI client error: {:?}", e)))?;
        Ok(Self {
            engine,
            client: client.unbind(),
            model: model.to_string(),
            namespace: RwLock::new(namespace.to_string()),
            counter: AtomicU64::new(0),
        })
    }

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

    fn store(
        &self,
        py: Python,
        text: &str,
        embedding: Vec<f32>,
        metadata: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<String> {
        let namespace = self.namespace.read().unwrap().clone();
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        let key = format!("openai_{n}");
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
}

#[pymodule]
fn vantadb_openai(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBOpenAI>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
