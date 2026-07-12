use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyList, PyListMethods, PyModuleMethods};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use vantadb::config::VantaConfig;
use vantadb::error::VantaError;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaValue};

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

/// VantaDB DocumentStore for Haystack.
///
/// Implements the Haystack ``DocumentStore`` protocol with ``write_documents``,
/// ``filter_documents``, ``delete_documents``, and ``count_documents``.
///
/// Usage::
///
///     from vantadb_haystack import VantaDBDocumentStore
///     store = VantaDBDocumentStore("/tmp/vantadb-haystack")
///     ids = store.write_documents([{"id": "1", "content": "hello", "embedding": [0.1, 0.2]}])
#[pyclass(name = "VantaDBDocumentStore")]
pub struct VantaDBDocumentStore {
    engine: VantaEmbedded,
    namespace: RwLock<String>,
    doc_counter: AtomicU64,
}

#[pymethods]
impl VantaDBDocumentStore {
    #[new]
    #[pyo3(signature = (db_path, namespace = "haystack_docs"))]
    fn new(db_path: &str, namespace: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config).map_err(err_to_py)?;
        Ok(Self {
            engine,
            namespace: RwLock::new(namespace.to_string()),
            doc_counter: AtomicU64::new(0),
        })
    }

    fn write_documents(&self, py: Python, documents: &Bound<'_, PyList>) -> PyResult<Vec<String>> {
        let namespace = self.namespace.read().unwrap().clone();
        let mut ids = Vec::with_capacity(documents.len());
        let mut inputs = Vec::with_capacity(documents.len());
        for item in documents.iter() {
            let d = item.cast::<PyDict>()?;
            let doc_id: String = d
                .get_item("id")
                .ok()
                .flatten()
                .and_then(|v| v.extract::<String>().ok())
                .unwrap_or_else(|| {
                    let n = self.doc_counter.fetch_add(1, Ordering::SeqCst);
                    format!("doc_{n}")
                });
            let content: String = d
                .get_item("content")
                .ok()
                .flatten()
                .and_then(|v| v.extract::<String>().ok())
                .unwrap_or_default();
            let embedding: Option<Vec<f32>> = d
                .get_item("embedding")
                .ok()
                .flatten()
                .and_then(|v| v.extract::<Vec<f32>>().ok())
                .filter(|v| !v.is_empty());

            let mut input = VantaMemoryInput::new(&namespace, &doc_id, &content);
            input.vector = embedding;

            if let Ok(Some(meta)) = d.get_item("metadata") {
                if let Ok(mdict) = meta.cast::<PyDict>() {
                    for entry in mdict.iter() {
                        if let (Ok(key), Ok(val)) =
                            (entry.0.extract::<String>(), entry.1.extract::<String>())
                        {
                            input.metadata.insert(key, VantaValue::String(val));
                        }
                    }
                }
            }

            inputs.push((doc_id, input));
        }

        let engine = self.engine.clone();
        // GIL RELEASED — batch pure Rust inserts
        let results = py.detach(move || {
            let mut out = Vec::with_capacity(inputs.len());
            for (doc_id, input) in inputs {
                let _ = engine.put(input).map_err(err_to_py)?;
                out.push(doc_id);
            }
            Ok::<_, PyErr>(out)
        })?;
        Ok(results)
    }

    #[pyo3(signature = (filters = None, top_k = 100))]
    fn filter_documents(
        &self,
        py: Python,
        filters: Option<&Bound<'_, PyDict>>,
        top_k: i32,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let namespace = self.namespace.read().unwrap().clone();
        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust list
        let page = py.detach(move || {
            engine
                .list(
                    &namespace,
                    VantaMemoryListOptions {
                        filters: py_dict_to_vanta_metadata(filters),
                        limit: top_k.max(1) as usize,
                        cursor: None,
                    },
                )
                .map_err(err_to_py)
        })?;

        let mut results = Vec::with_capacity(page.records.len());
        for rec in &page.records {
            let d = PyDict::new(py);
            d.set_item("id", &rec.key)?;
            d.set_item("content", &rec.payload)?;
            d.set_item("embedding", rec.vector.clone())?;
            results.push(d.unbind().into());
        }
        Ok(results)
    }

    fn delete_documents(&self, py: Python, document_ids: Vec<String>) -> PyResult<()> {
        let namespace = self.namespace.read().unwrap().clone();
        let engine = self.engine.clone();
        // GIL RELEASED — batch pure Rust deletes
        py.detach(move || {
            for doc_id in &document_ids {
                engine.delete(&namespace, doc_id).map_err(err_to_py)?;
            }
            Ok(())
        })
    }

    fn count_documents(&self, py: Python) -> PyResult<i64> {
        let namespace = self.namespace.read().unwrap().clone();
        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust count
        let page = py.detach(move || {
            engine
                .list(&namespace, Default::default())
                .map_err(err_to_py)
        })?;
        Ok(page.records.len() as i64)
    }
}

fn py_dict_to_vanta_metadata(dict: Option<&Bound<'_, PyDict>>) -> BTreeMap<String, VantaValue> {
    let mut map = BTreeMap::new();
    if let Some(d) = dict {
        for (k, v) in d.iter() {
            if let Ok(key) = k.extract::<String>() {
                if let Ok(s) = v.extract::<String>() {
                    map.insert(key, VantaValue::String(s));
                } else if let Ok(i) = v.extract::<i64>() {
                    map.insert(key, VantaValue::Int(i));
                } else if let Ok(f) = v.extract::<f64>() {
                    map.insert(key, VantaValue::Float(f));
                } else if let Ok(b) = v.extract::<bool>() {
                    map.insert(key, VantaValue::Bool(b));
                }
            }
        }
    }
    map
}

#[pymodule]
fn vantadb_haystack(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBDocumentStore>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
