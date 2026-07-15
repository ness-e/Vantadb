use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyModuleMethods};
use std::sync::atomic::{AtomicU64, Ordering};
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
    namespace: String,
    counter: AtomicU64,
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
        Ok(Self {
            engine,
            namespace: collection.to_string(),
            counter: AtomicU64::new(0),
        })
    }

    #[pyo3(signature = (texts, embeddings, metadatas = None, ids = None))]
    fn add(
        &self,
        py: Python,
        texts: Vec<String>,
        embeddings: Vec<Vec<f32>>,
        metadatas: Option<Vec<Option<Py<PyDict>>>>,
        ids: Option<Vec<Option<String>>>,
    ) -> PyResult<Vec<String>> {
        let namespace = &self.namespace;
        let mut out_ids = Vec::with_capacity(texts.len());

        let mut inputs: Vec<VantaMemoryInput> = Vec::with_capacity(texts.len());

        for i in 0..texts.len() {
            let key = match &ids {
                Some(ids) => ids.get(i).and_then(|o| o.clone()).unwrap_or_else(|| {
                    let n = self.counter.fetch_add(1, Ordering::Relaxed);
                    format!("node_{}_{}", namespace, n)
                }),
                None => {
                    let n = self.counter.fetch_add(1, Ordering::Relaxed);
                    format!("node_{}_{}", namespace, n)
                }
            };
            let mut input = VantaMemoryInput::new(namespace, &key, &texts[i]);
            input.vector = Some(embeddings[i].clone());

            if let Some(ref metas) = metadatas {
                if let Some(Some(meta)) = metas.get(i) {
                    for (k, v) in meta.bind(py).iter() {
                        if let Ok(key) = k.extract::<String>() {
                            let val = v
                                .extract::<String>()
                                .ok()
                                .map(vantadb::sdk::VantaValue::String)
                                .or_else(|| {
                                    v.extract::<bool>().ok().map(vantadb::sdk::VantaValue::Bool)
                                })
                                .or_else(|| {
                                    v.extract::<i64>().ok().map(vantadb::sdk::VantaValue::Int)
                                })
                                .or_else(|| {
                                    v.extract::<f64>().ok().map(vantadb::sdk::VantaValue::Float)
                                });
                            if let Some(val) = val {
                                input.metadata.insert(key, val);
                            }
                        }
                    }
                }
            }

            inputs.push(input);
        }

        let results: Vec<Result<String, String>> = py.detach(|| {
            let mut results = Vec::with_capacity(inputs.len());
            for input in inputs {
                match self.engine.put(input) {
                    Ok(record) => results.push(Ok(format!("{}:{}", record.namespace, record.key))),
                    Err(e) => results.push(Err(format!("add error: {:?}", e))),
                }
            }
            results
        });

        for r in results {
            match r {
                Ok(id) => out_ids.push(id),
                Err(e) => return Err(PyRuntimeError::new_err(e)),
            }
        }

        Ok(out_ids)
    }

    #[pyo3(signature = (embedding, top_k = 10))]
    fn query(&self, py: Python, embedding: Vec<f32>, top_k: i32) -> PyResult<Vec<Py<PyAny>>> {
        let request = VantaMemorySearchRequest {
            namespace: self.namespace.clone(),
            query_vector: embedding,
            filters: Default::default(),
            text_query: None,
            top_k: top_k as usize,
            distance_metric: vantadb::DistanceMetric::Cosine,
            explain: false,
        };

        use std::collections::BTreeMap;

        let hits: Vec<(
            String,
            String,
            f32,
            BTreeMap<String, vantadb::sdk::VantaValue>,
        )> = py
            .detach(|| match self.engine.search(request) {
                Ok(hits) => Ok(hits
                    .into_iter()
                    .map(|hit| {
                        (
                            format!("{}:{}", hit.record.namespace, hit.record.key),
                            hit.record.payload,
                            hit.score,
                            hit.record.metadata,
                        )
                    })
                    .collect()),
                Err(e) => Err(format!("query error: {:?}", e)),
            })
            .map_err(PyRuntimeError::new_err)?;

        Python::attach(|py| {
            let mut results = Vec::with_capacity(hits.len());
            for (id, text, score, metadata) in hits {
                let d = PyDict::new(py);
                d.set_item("id", &id)?;
                d.set_item("text", &text)?;
                d.set_item("score", score)?;
                if !metadata.is_empty() {
                    let meta = PyDict::new(py);
                    for (k, v) in &metadata {
                        match v {
                            vantadb::sdk::VantaValue::String(s) => meta.set_item(k, s)?,
                            vantadb::sdk::VantaValue::Int(n) => meta.set_item(k, n)?,
                            vantadb::sdk::VantaValue::Float(f) => meta.set_item(k, f)?,
                            vantadb::sdk::VantaValue::Bool(b) => meta.set_item(k, b)?,
                            _ => {}
                        }
                    }
                    d.set_item("metadata", meta)?;
                }
                results.push(d.unbind().into());
            }
            Ok(results)
        })
    }

    fn delete(&self, py: Python, ids: Vec<String>) -> PyResult<()> {
        py.detach(|| {
            for id in &ids {
                let parts: Vec<&str> = id.split(':').collect();
                if parts.len() == 2 {
                    self.engine
                        .delete(parts[0], parts[1])
                        .map_err(|e| format!("delete error: {:?}", e))?;
                }
            }
            Ok(())
        })
        .map_err(|e: String| PyRuntimeError::new_err(e))
    }
}

#[pymodule]
fn vantadb_llamaindex(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBVectorStore>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
