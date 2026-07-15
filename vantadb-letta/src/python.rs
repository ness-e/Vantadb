use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyModuleMethods};
use std::sync::atomic::{AtomicU64, Ordering};
use vantadb::config::VantaConfig;
use vantadb::error::VantaError;
use vantadb::sdk::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryRecord,
    VantaMemorySearchRequest,
};

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

fn ns_for(user_id: &str, agent_id: &str) -> String {
    let safe = |s: &str| -> String {
        s.replace(
            |c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-',
            "_",
        )
    };
    format!("letta_{}_{}", safe(user_id), safe(agent_id))
}

/// VantaDB vector store backend for Letta/MemGPT.
///
/// Implements the Letta ``VectorStore`` protocol with methods for storing,
/// retrieving, listing, and deleting agent memories.
///
/// Usage::
///
///     from vantadb_letta import LettaStore
///     store = LettaStore("/tmp/vantadb-letta")
///     store.store_memory("user1", "agent1", "My memory content", [0.1, 0.2, ...])
///     results = store.retrieve_memory("user1", "agent1", [0.1, 0.2, ...], top_k=5)
#[pyclass(name = "LettaStore")]
pub struct LettaStore {
    engine: VantaEmbedded,
    counter: AtomicU64,
}

#[pymethods]
impl LettaStore {
    #[new]
    #[pyo3(signature = (path, _collection_name = "letta_memories"))]
    fn new(path: &str, _collection_name: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config).map_err(err_to_py)?;
        Ok(Self {
            engine,
            counter: AtomicU64::new(0),
        })
    }

    fn store_memory(
        &self,
        py: Python,
        user_id: &str,
        agent_id: &str,
        content: &str,
        embedding: Vec<f32>,
    ) -> PyResult<String> {
        let namespace = ns_for(user_id, agent_id);
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        let key = format!("mem_{n}");
        let mut input = VantaMemoryInput::new(&namespace, &key, content);
        input.vector = Some(embedding);

        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust insert
        let record = py.detach(move || engine.put(input).map_err(err_to_py))?;
        Ok(format!("{}:{}", record.namespace, record.key))
    }

    #[pyo3(signature = (user_id, agent_id, query_embedding, top_k = 5))]
    fn retrieve_memory(
        &self,
        py: Python,
        user_id: &str,
        agent_id: &str,
        query_embedding: Vec<f32>,
        top_k: i32,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let namespace = ns_for(user_id, agent_id);
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
            d.set_item("content", &hit.record.payload)?;
            let normalized: f32 = 1.0 - (hit.score / 2.0);
            d.set_item("score", normalized)?;
            results.push(d.unbind().into());
        }
        Ok(results)
    }

    fn list_memories(&self, py: Python, user_id: &str, agent_id: &str) -> PyResult<Vec<Py<PyAny>>> {
        let namespace = ns_for(user_id, agent_id);
        let engine = self.engine.clone();
        // GIL RELEASED — paginate through ALL pages (not just first 100)
        let all_records = py.detach(move || -> PyResult<Vec<VantaMemoryRecord>> {
            let mut all_records = Vec::new();
            let mut cursor = None;
            loop {
                let opts = VantaMemoryListOptions {
                    cursor,
                    ..Default::default()
                };
                let page = engine.list(&namespace, opts).map_err(err_to_py)?;
                all_records.extend(page.records);
                cursor = page.next_cursor;
                if cursor.is_none() {
                    break;
                }
            }
            Ok(all_records)
        })?;

        let mut results = Vec::with_capacity(all_records.len());
        for record in &all_records {
            let d = PyDict::new(py);
            d.set_item("id", format!("{}:{}", record.namespace, record.key))?;
            d.set_item("content", &record.payload)?;
            results.push(d.unbind().into());
        }
        Ok(results)
    }

    fn delete_memory(&self, py: Python, memory_id: &str) -> PyResult<()> {
        let parts: Vec<&str> = memory_id.split(':').collect();
        if parts.len() != 2 {
            return Err(PyRuntimeError::new_err(format!(
                "invalid memory_id: {memory_id}, expected namespace:key"
            )));
        }
        let engine = self.engine.clone();
        let ns = parts[0].to_string();
        let key = parts[1].to_string();
        // GIL RELEASED — pure Rust delete
        py.detach(move || engine.delete(&ns, &key).map_err(err_to_py))?;
        Ok(())
    }
}

#[pymodule]
fn vantadb_letta(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LettaStore>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
