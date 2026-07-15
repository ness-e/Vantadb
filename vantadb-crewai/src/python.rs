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

/// VantaDB memory backend for CrewAI.
///
/// Implements CrewAI's ``RAGMemory`` / ``LongTermMemory`` interface
/// with ``save``, ``search``, and ``clear`` operations.
///
/// Usage::
///
///     from vantadb_crewai import CrewAIMemory
///     memory = CrewAIMemory("/tmp/vantadb-crewai")
///     memory.save("context data", {"key": "value"}, [0.1, 0.2, ...])
///     results = memory.search([0.1, 0.2, ...], top_k=5)
#[pyclass(name = "CrewAIMemory")]
pub struct CrewAIMemory {
    engine: VantaEmbedded,
    namespace: RwLock<String>,
    counter: AtomicU64,
}

#[pymethods]
impl CrewAIMemory {
    #[new]
    #[pyo3(signature = (db_path, namespace = "crewai_memories"))]
    fn new(db_path: &str, namespace: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config).map_err(err_to_py)?;
        Ok(Self {
            engine,
            namespace: RwLock::new(namespace.to_string()),
            counter: AtomicU64::new(0),
        })
    }

    fn save(
        &self,
        py: Python,
        context: &str,
        metadata: &Bound<'_, PyDict>,
        embedding: Vec<f32>,
    ) -> PyResult<String> {
        let namespace = self.namespace.read().unwrap().clone();
        let meta_str = serde_json::to_string(&py_dict_to_string_map(metadata)).unwrap_or_default();
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        let key = format!("crew_{n}");
        let mut input = VantaMemoryInput::new(&namespace, &key, context);
        input.vector = Some(embedding);
        input.metadata.insert(
            "metadata_json".into(),
            vantadb::sdk::VantaValue::String(meta_str),
        );

        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust insert
        let record = py.detach(move || engine.put(input).map_err(err_to_py))?;
        Ok(format!("{}:{}", record.namespace, record.key))
    }

    #[pyo3(signature = (query_embedding, top_k = 5, threshold = 0.0))]
    fn search(
        &self,
        py: Python,
        query_embedding: Vec<f32>,
        top_k: i32,
        threshold: f32,
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
            let score = (1.0 - hit.score).max(0.0);
            if score < threshold {
                continue;
            }
            let d = PyDict::new(py);
            d.set_item("id", format!("{}:{}", hit.record.namespace, hit.record.key))?;
            d.set_item("context", &hit.record.payload)?;
            d.set_item("score", score)?;
            results.push(d.unbind().into());
        }
        Ok(results)
    }

    fn clear(&self, py: Python) -> PyResult<()> {
        let namespace = self.namespace.read().unwrap().clone();
        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust list
        let page = py.detach(move || {
            engine
                .list(&namespace, Default::default())
                .map_err(err_to_py)
        })?;
        let engine = self.engine.clone();
        for record in &page.records {
            let eng = engine.clone();
            let ns = record.namespace.clone();
            let key = record.key.clone();
            // GIL RELEASED — pure Rust delete
            py.detach(move || eng.delete(&ns, &key).map_err(err_to_py))?;
        }
        Ok(())
    }
}

fn py_dict_to_string_map(dict: &Bound<'_, PyDict>) -> std::collections::BTreeMap<String, String> {
    let mut map = std::collections::BTreeMap::new();
    for (k, v) in dict.iter() {
        if let Ok(key) = k.extract::<String>() {
            let val = v
                .extract::<String>()
                .ok()
                .or_else(|| v.extract::<bool>().ok().map(|b| b.to_string()))
                .or_else(|| v.extract::<i64>().ok().map(|i| i.to_string()))
                .or_else(|| v.extract::<f64>().ok().map(|f| f.to_string()));
            if let Some(val) = val {
                map.insert(key, val);
            }
        }
    }
    map
}

#[pymodule]
fn vantadb_crewai(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CrewAIMemory>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
