use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyModuleMethods};
use std::sync::atomic::{AtomicU64, Ordering};
use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};

/// VantaDB vector store for LangChain.
///
/// Implements the LangChain ``VectorStore`` protocol with ``add_texts``,
/// ``similarity_search_by_vector``, ``delete``, and ``from_texts``.
///
/// Usage::
///
///     from vantadb_langchain import VantaDBVectorStore
///     store = VantaDBVectorStore("/tmp/vantadb-langchain")
///     ids = store.add_texts(["hello"], [[0.1, 0.2]])
///     docs = store.similarity_search_by_vector([0.1, 0.2], k=5)
#[pyclass(name = "VantaDBVectorStore")]
pub struct VantaDBVectorStore {
    engine: VantaEmbedded,
    namespace: String,
    counter: AtomicU64,
}

#[pymethods]
impl VantaDBVectorStore {
    #[new]
    #[pyo3(signature = (db_path, collection = "langchain_store"))]
    fn new(db_path: &str, collection: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config)
            .map_err(|e| PyRuntimeError::new_err(format!("VantaDB open error: {:?}", e)))?;
        Ok(Self {
            engine,
            namespace: collection.to_string(),
            counter: AtomicU64::new(0),
        })
    }

    #[pyo3(signature = (texts, embeddings, metadatas = None, ids = None))]
    fn add_texts(
        &self,
        py: Python,
        texts: Vec<String>,
        embeddings: Vec<Vec<f32>>,
        metadatas: Option<Vec<Option<Py<PyDict>>>>,
        ids: Option<Vec<Option<String>>>,
    ) -> PyResult<Vec<String>> {
        let namespace = &self.namespace;
        let mut out_ids = Vec::with_capacity(texts.len());

        for i in 0..texts.len() {
            let key = match &ids {
                Some(ids) => ids.get(i).and_then(|o| o.clone()).unwrap_or_else(|| {
                    let n = self.counter.fetch_add(1, Ordering::Relaxed);
                    format!("doc_{}_{}", namespace, n)
                }),
                None => {
                    let n = self.counter.fetch_add(1, Ordering::Relaxed);
                    format!("doc_{}_{}", namespace, n)
                }
            };
            let mut input = VantaMemoryInput::new(namespace, &key, &texts[i]);
            input.vector = Some(embeddings[i].clone());

            if let Some(ref metas) = metadatas {
                if let Some(Some(meta)) = metas.get(i) {
                    for (k, v) in meta.bind(py).iter() {
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
                .map_err(|e| PyRuntimeError::new_err(format!("add_texts error: {:?}", e)))?;
            out_ids.push(format!("{}:{}", record.namespace, record.key));
        }

        Ok(out_ids)
    }

    #[pyo3(signature = (embedding, k = 10))]
    fn similarity_search_by_vector(
        &self,
        py: Python,
        embedding: Vec<f32>,
        k: i32,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let request = VantaMemorySearchRequest {
            namespace: self.namespace.clone(),
            query_vector: embedding,
            filters: Default::default(),
            text_query: None,
            top_k: k as usize,
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
fn vantadb_langchain(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBVectorStore>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
