use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyModuleMethods};
use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};

/// Ollama local embedding wrapper with VantaDB storage.
#[pyclass(name = "VantaDBOllama")]
pub struct VantaDBOllama {
    engine: VantaEmbedded,
    base_url: String,
    model: String,
}

#[pymethods]
impl VantaDBOllama {
    #[new]
    #[pyo3(signature = (db_path, base_url = "http://localhost:11434", model = "nomic-embed-text"))]
    fn new(db_path: &str, base_url: &str, model: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config)
            .map_err(|e| PyRuntimeError::new_err(format!("VantaDB open error: {:?}", e)))?;
        Ok(Self {
            engine,
            base_url: base_url.to_string(),
            model: model.to_string(),
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
        let request = VantaMemorySearchRequest {
            namespace: "ollama_store".into(),
            query_vector: query_embedding,
            filters: Default::default(),
            text_query: None,
            top_k: top_k as usize,
            distance_metric: vantadb::DistanceMetric::Cosine,
            explain: false,
        };

        let hits = self
            .engine
            .search(request)
            .map_err(|e| PyRuntimeError::new_err(format!("search error: {:?}", e)))?;

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
        let key = format!("ollama_{}", text.len());
        let mut input = VantaMemoryInput::new("ollama_store", &key, text);
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

        let record = self
            .engine
            .put(input)
            .map_err(|e| PyRuntimeError::new_err(format!("store error: {:?}", e)))?;
        Ok(format!("{}:{}", record.namespace, record.key))
    }
}

#[pymodule]
fn vantadb_ollama(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBOllama>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
