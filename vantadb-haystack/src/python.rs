use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods, PyList, PyListMethods, PyModuleMethods};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use vantadb::config::VantaConfig;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaValue};

/// VantaDB DocumentStore for Haystack.
#[pyclass(name = "VantaDBDocumentStore")]
pub struct VantaDBDocumentStore {
    engine: VantaEmbedded,
    doc_counter: AtomicU64,
}

#[pymethods]
impl VantaDBDocumentStore {
    #[new]
    fn new(db_path: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config)
            .map_err(|e| PyRuntimeError::new_err(format!("VantaDB open error: {:?}", e)))?;
        Ok(Self {
            engine,
            doc_counter: AtomicU64::new(0),
        })
    }

    fn write_documents(&self, documents: &Bound<'_, PyList>) -> PyResult<Vec<String>> {
        let mut ids = Vec::with_capacity(documents.len());
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

            let mut input = VantaMemoryInput::new("haystack_docs", &doc_id, &content);
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

            self.engine
                .put(input)
                .map_err(|e| PyRuntimeError::new_err(format!("write_documents error: {:?}", e)))?;
            ids.push(doc_id);
        }
        Ok(ids)
    }

    #[pyo3(signature = (filters = None, top_k = 100))]
    fn filter_documents(
        &self,
        py: Python,
        filters: Option<&Bound<'_, PyDict>>,
        top_k: i32,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let page = self
            .engine
            .list(
                "haystack_docs",
                VantaMemoryListOptions {
                    filters: py_dict_to_vanta_metadata(filters),
                    limit: top_k.max(1) as usize,
                    cursor: None,
                },
            )
            .map_err(|e| PyRuntimeError::new_err(format!("filter_documents error: {:?}", e)))?;

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

    fn delete_documents(&self, document_ids: Vec<String>) -> PyResult<()> {
        for doc_id in &document_ids {
            self.engine
                .delete("haystack_docs", doc_id)
                .map_err(|e| PyRuntimeError::new_err(format!("delete_documents error: {:?}", e)))?;
        }
        Ok(())
    }

    fn count_documents(&self) -> PyResult<i64> {
        let page = self
            .engine
            .list("haystack_docs", Default::default())
            .map_err(|e| PyRuntimeError::new_err(format!("count_documents error: {:?}", e)))?;
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
    m.add("__version__", "0.1.5")?;
    Ok(())
}
