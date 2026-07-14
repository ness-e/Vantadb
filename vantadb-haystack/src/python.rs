use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyListMethods, PyModuleMethods};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use vantadb::config::VantaConfig;
use vantadb::error::VantaError;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaValue};

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
        let mut inputs = Vec::with_capacity(documents.len());
        for item in documents.iter() {
            let (doc_id, content, embedding, meta) = if let Ok(d) = item.cast::<PyDict>() {
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
                let meta: BTreeMap<String, VantaValue> =
                    if let Some(m) = d.get_item("metadata").ok().flatten() {
                        if let Ok(dict) = m.cast::<PyDict>() {
                            py_dict_to_vanta_metadata(dict)
                        } else {
                            BTreeMap::new()
                        }
                    } else {
                        BTreeMap::new()
                    };
                (doc_id, content, embedding, meta)
            } else {
                let doc_id: String = item
                    .getattr("id")
                    .and_then(|v| v.extract::<String>())
                    .unwrap_or_else(|_| {
                        let n = self.doc_counter.fetch_add(1, Ordering::SeqCst);
                        format!("doc_{n}")
                    });
                let content: String = item
                    .getattr("content")
                    .and_then(|v| v.extract::<String>())
                    .unwrap_or_default();
                let embedding: Option<Vec<f32>> = item
                    .getattr("embedding")
                    .ok()
                    .and_then(|v| v.extract::<Vec<f32>>().ok())
                    .filter(|v| !v.is_empty());
                let meta: BTreeMap<String, VantaValue> = if let Some(m) = item.getattr("meta").ok()
                {
                    if let Ok(dict) = m.cast::<PyDict>() {
                        py_dict_to_vanta_metadata(dict)
                    } else {
                        BTreeMap::new()
                    }
                } else {
                    BTreeMap::new()
                };
                (doc_id, content, embedding, meta)
            };

            let mut input = VantaMemoryInput::new(&namespace, &doc_id, &content);
            input.vector = embedding;
            input.metadata = meta;
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
        let filters = filters
            .map(|d| py_dict_to_vanta_metadata(d))
            .unwrap_or_default();
        let engine = self.engine.clone();
        // GIL RELEASED — pure Rust list
        let page = py.detach(move || {
            engine
                .list(
                    &namespace,
                    VantaMemoryListOptions {
                        filters,
                        limit: top_k.max(1) as usize,
                        cursor: None,
                    },
                )
                .map_err(err_to_py)
        })?;

        let document_cls = py.import("haystack.dataclasses")?.getattr("Document")?;

        let mut results = Vec::with_capacity(page.records.len());
        for rec in &page.records {
            let meta = PyDict::new(py);
            for (k, v) in &rec.metadata {
                match v {
                    VantaValue::String(s) => meta.set_item(k.as_str(), s.as_str())?,
                    VantaValue::Int(i) => meta.set_item(k.as_str(), *i)?,
                    VantaValue::Float(f) => meta.set_item(k.as_str(), *f)?,
                    VantaValue::Bool(b) => meta.set_item(k.as_str(), *b)?,
                    VantaValue::DateTime(dt) => meta.set_item(k.as_str(), dt.to_rfc3339())?,
                    VantaValue::Null => meta.set_item(k.as_str(), py.None())?,
                    _ => meta.set_item(k.as_str(), format!("{:?}", v))?,
                };
            }
            let doc = document_cls.call1((&rec.key, &rec.payload))?;
            if let Some(vec) = &rec.vector {
                doc.setattr("embedding", vec)?;
            }
            doc.setattr("meta", meta)?;
            results.push(doc.unbind().into());
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

fn py_dict_to_vanta_metadata(dict: &Bound<'_, PyDict>) -> BTreeMap<String, VantaValue> {
    let mut map = BTreeMap::new();
    for (k, v) in dict.iter() {
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
    map
}

#[pymodule]
fn vantadb_haystack(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBDocumentStore>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
