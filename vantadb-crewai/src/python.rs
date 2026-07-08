use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyModuleMethods};
use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};

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
}

#[pymethods]
impl CrewAIMemory {
    #[new]
    fn new(db_path: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config)
            .map_err(|e| PyRuntimeError::new_err(format!("VantaDB open error: {:?}", e)))?;
        Ok(Self { engine })
    }

    fn save(
        &self,
        context: &str,
        metadata: &Bound<'_, PyDict>,
        embedding: Vec<f32>,
    ) -> PyResult<String> {
        let meta_str = serde_json::to_string(&py_dict_to_string_map(metadata)).unwrap_or_default();
        let key = format!("crew_{}", context.len());
        let mut input = VantaMemoryInput::new("crewai_memories", &key, context);
        input.vector = Some(embedding);
        input.metadata.insert(
            "metadata_json".into(),
            vantadb::sdk::VantaValue::String(meta_str),
        );

        let record = self
            .engine
            .put(input)
            .map_err(|e| PyRuntimeError::new_err(format!("save error: {:?}", e)))?;
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
        let request = VantaMemorySearchRequest {
            namespace: "crewai_memories".into(),
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

    fn clear(&self) -> PyResult<()> {
        let page = self
            .engine
            .list("crewai_memories", Default::default())
            .map_err(|e| PyRuntimeError::new_err(format!("list for clear error: {:?}", e)))?;
        for record in &page.records {
            self.engine
                .delete(&record.namespace, &record.key)
                .map_err(|e| PyRuntimeError::new_err(format!("clear delete error: {:?}", e)))?;
        }
        Ok(())
    }
}

fn py_dict_to_string_map(dict: &Bound<'_, PyDict>) -> std::collections::BTreeMap<String, String> {
    let mut map = std::collections::BTreeMap::new();
    for (k, v) in dict.iter() {
        if let (Ok(key), Ok(val)) = (k.extract::<String>(), v.extract::<String>()) {
            map.insert(key, val);
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
