use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyModuleMethods};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use vantadb::config::VantaConfig;
use vantadb::error::VantaError;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};

fn err_to_py(e: VantaError) -> PyErr {
    match e {
        VantaError::NotFound(..) => pyo3::exceptions::PyKeyError::new_err(e.to_string()),
        VantaError::Storage(..) => pyo3::exceptions::PyIOError::new_err(e.to_string()),
        VantaError::InvalidArgument(..)
        | VantaError::CollectionNotEmpty(..)
        | VantaError::Serialization(..) => pyo3::exceptions::PyValueError::new_err(e.to_string()),
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
    base_url: String,
    model: String,
    namespace: RwLock<String>,
    counter: AtomicU64,
}

#[pymethods]
impl VantaDBOllama {
    #[new]
    #[pyo3(signature = (db_path, base_url = "http://localhost:11434", model = "nomic-embed-text", namespace = "ollama_store"))]
    fn new(db_path: &str, base_url: &str, model: &str, namespace: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config).map_err(err_to_py)?;
        Ok(Self {
            engine,
            base_url: base_url.to_string(),
            model: model.to_string(),
            namespace: RwLock::new(namespace.to_string()),
            counter: AtomicU64::new(0),
        })
    }

    fn embed(&self, py: Python, texts: Vec<String>) -> PyResult<Vec<Vec<f32>>> {
        let ollama_mod = pyo3::types::PyModule::import(py, "ollama")
            .map_err(|e| PyRuntimeError::new_err(format!("ollama import error: {:?}", e)))?;
        let client_kwargs = PyDict::new(py);
        client_kwargs.set_item("host", &self.base_url)?;
        let client = ollama_mod
            .getattr("Client")
            .and_then(|cls| cls.call((), Some(&client_kwargs)))
            .map_err(|e| PyRuntimeError::new_err(format!("Ollama client error: {:?}", e)))?;

        let mut result = Vec::with_capacity(texts.len());
        for text in &texts {
            let kwargs = PyDict::new(py);
            kwargs.set_item("model", &self.model)?;
            kwargs.set_item("prompt", text)?;
            let response = client
                .getattr("embeddings")
                .and_then(|func| func.call((), Some(&kwargs)))
                .map_err(|e| PyRuntimeError::new_err(format!("Ollama embed error: {:?}", e)))?;

            let emb: Vec<f32> = response
                .get_item("embedding")
                .and_then(|v| v.extract::<Vec<f64>>())
                .map_err(|e| PyRuntimeError::new_err(format!("missing embedding: {:?}", e)))?
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
        let key = format!("ollama_{n}");
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

        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust insert
        let record = py.detach(move || engine.put(input).map_err(err_to_py))?;
        Ok(format!("{}:{}", record.namespace, record.key))
    }
}

#[pymodule]
fn vantadb_ollama(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBOllama>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
