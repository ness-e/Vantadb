use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyModuleMethods};
use std::sync::RwLock;
use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};

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
        let engine = VantaEmbedded::open_with_config(config)
            .map_err(|e| PyRuntimeError::new_err(format!("VantaDB open error: {:?}", e)))?;
        Ok(Self {
            engine,
            collection: RwLock::new(collection.to_string()),
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

        let hits = self
            .engine
            .search(request)
            .map_err(|e| PyRuntimeError::new_err(format!("forward error: {:?}", e)))?;

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
        passage: &str,
        embedding: Vec<f32>,
        metadata: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<String> {
        let namespace = self.collection.read().unwrap().clone();
        let key = format!("passage_{}", passage.len());
        let mut input = VantaMemoryInput::new(&namespace, &key, passage);
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
            .map_err(|e| PyRuntimeError::new_err(format!("add_passage error: {:?}", e)))?;
        Ok(format!("{}:{}", record.namespace, record.key))
    }
}

#[pymodule]
fn vantadb_dspy(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBRM>()?;
    m.add("__version__", "0.1.5")?;
    Ok(())
}
