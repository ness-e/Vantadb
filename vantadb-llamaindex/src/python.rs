use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyModuleMethods};
use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};

/// VantaDB vector store for LlamaIndex.
///
/// Implements the LlamaIndex ``VectorStore`` protocol with ``add``,
/// ``query``, and ``delete`` operations.
///
/// Usage::
///
///     from vantadb_llamaindex import VantaDBVectorStore
///     store = VantaDBVectorStore("/tmp/vantadb-llamaindex")
///     store.add(["hello"], [[0.1, 0.2]])
///     nodes = store.query([0.1, 0.2], similarity_top_k=5)
#[pyclass(name = "VantaDBVectorStore")]
pub struct VantaDBVectorStore {
    engine: VantaEmbedded,
}

#[pymethods]
impl VantaDBVectorStore {
    #[new]
    #[pyo3(signature = (db_path, collection = "llamaindex_store"))]
    fn new(db_path: &str, collection: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config)
            .map_err(|e| PyRuntimeError::new_err(format!("VantaDB open error: {:?}", e)))?;
        Ok(Self { engine })
    }

    #[pyo3(signature = (texts, embeddings, metadatas = None, ids = None))]
    fn add(
        &self,
        texts: Vec<String>,
        embeddings: Vec<Vec<f32>>,
        metadatas: Option<Vec<Option<&Bound<'_, PyDict>>>>,
        ids: Option<Vec<Option<String>>>,
    ) -> PyResult<Vec<String>> {
        let namespace = "llamaindex_store";
        let mut out_ids = Vec::with_capacity(texts.len());

        for i in 0..texts.len() {
            let key = match &ids {
                Some(ids) => ids
                    .get(i)
                    .and_then(|o| o.clone())
                    .unwrap_or_else(|| format!("node_{}_{}", namespace, i)),
                None => format!("node_{}_{}", namespace, i),
            };
            let mut input = VantaMemoryInput::new(namespace, &key, &texts[i]);
            input.vector = Some(embeddings[i].clone());

            if let Some(metas) = metadatas {
                if let Some(Some(meta)) = metas.get(i) {
                    for (k, v) in meta.iter() {
                        if let (Ok(key), Ok(val)) = (k.extract::<String>(), v.extract::<String>()) {
                            input
                                .metadata
                                .insert(key, vantadb::sdk::VantaValue::String(val));
                        }
                    }
                }
            }

            let record = self
                .engine
                .put(input)
                .map_err(|e| PyRuntimeError::new_err(format!("add error: {:?}", e)))?;
            out_ids.push(format!("{}:{}", record.namespace, record.key));
        }

        Ok(out_ids)
    }

    #[pyo3(signature = (embedding, top_k = 10))]
    fn query(&self, py: Python, embedding: Vec<f32>, top_k: i32) -> PyResult<Vec<Py<PyAny>>> {
        let request = VantaMemorySearchRequest {
            namespace: "llamaindex_store".into(),
            query_vector: embedding,
            filters: Default::default(),
            text_query: None,
            top_k: top_k as usize,
            distance_metric: vantadb::DistanceMetric::Cosine,
            explain: false,
        };

        let hits = self
            .engine
            .search(request)
            .map_err(|e| PyRuntimeError::new_err(format!("query error: {:?}", e)))?;

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

    fn delete(&self, ids: Vec<String>) -> PyResult<()> {
        for id in &ids {
            let parts: Vec<&str> = id.split(':').collect();
            if parts.len() == 2 {
                self.engine
                    .delete(parts[0], parts[1])
                    .map_err(|e| PyRuntimeError::new_err(format!("delete error: {:?}", e)))?;
            }
        }
        Ok(())
    }
}

#[pymodule]
fn vantadb_llamaindex(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBVectorStore>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
