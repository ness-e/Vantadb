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

/// VantaDB retrieval module for DSPy.
///
/// Implements the DSPy ``RetrieverModel`` protocol so it can be used
/// as a ``dspy.ColBERTv2``-style retriever.
///
/// Usage::
///
///     from vantadb_dspy import VantaDBRM
///     rm = VantaDBRM("/tmp/vantadb-dspy", "my_collection")
///     rm.add_passage("Paris is the capital of France", [0.1, 0.2, ...])
///     results = rm.forward([0.1, 0.2, ...], k=5)
#[pyclass(name = "VantaDBRM")]
pub struct VantaDBRM {
    engine: VantaEmbedded,
    collection: RwLock<String>,
    counter: AtomicU64,
}

#[pymethods]
impl VantaDBRM {
    #[new]
    #[pyo3(signature = (db_path, collection = "dspy_passages"))]
    fn new(db_path: &str, collection: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config).map_err(err_to_py)?;
        Ok(Self {
            engine,
            collection: RwLock::new(collection.to_string()),
            counter: AtomicU64::new(0),
        })
    }

    #[pyo3(signature = (query_embedding, k = 10))]
    fn forward(&self, py: Python, query_embedding: Vec<f32>, k: i32) -> PyResult<Vec<Py<PyAny>>> {
        let namespace = self.collection.read().unwrap().clone();
        let request = VantaMemorySearchRequest {
            namespace,
            query_vector: query_embedding,
            filters: Default::default(),
            text_query: None,
            top_k: k as usize,
            distance_metric: vantadb::DistanceMetric::Cosine,
            explain: false,
        };

        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust search
        let hits = py.detach(move || engine.search(request).map_err(err_to_py))?;

        let mut passages = Vec::with_capacity(hits.len());
        for hit in hits {
            let d = PyDict::new(py);
            d.set_item("passage", &hit.record.payload)?;
            d.set_item("score", hit.score)?;
            d.set_item("id", format!("{}:{}", hit.record.namespace, hit.record.key))?;
            passages.push(d.unbind().into());
        }
        Ok(passages)
    }

    #[pyo3(signature = (passage, embedding, metadata = None))]
    fn add_passage(
        &self,
        py: Python,
        passage: &str,
        embedding: Vec<f32>,
        metadata: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<String> {
        let namespace = self.collection.read().unwrap().clone();
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        let key = format!("passage_{n}");
        let mut input = VantaMemoryInput::new(&namespace, &key, passage);
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
fn vantadb_dspy(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBRM>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
