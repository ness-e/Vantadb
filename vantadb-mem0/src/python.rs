use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyModuleMethods};
use std::collections::BTreeMap;
use std::sync::RwLock;
use vantadb::config::VantaConfig;
use vantadb::sdk::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemorySearchRequest,
};

/// Parse a Mem0-style filter dict into VantaDB metadata pairs.
fn py_dict_to_metadata(
    fields: Option<&Bound<'_, PyDict>>,
) -> PyResult<BTreeMap<String, vantadb::sdk::VantaValue>> {
    let mut metadata = BTreeMap::new();
    if let Some(extra) = fields {
        for (key, value) in extra.iter() {
            let k: String = key.extract()?;
            metadata.insert(k, py_any_to_vanta_value(&value)?);
        }
    }
    Ok(metadata)
}

fn py_any_to_vanta_value(value: &Bound<'_, PyAny>) -> PyResult<vantadb::sdk::VantaValue> {
    if value.is_none() {
        return Ok(vantadb::sdk::VantaValue::Null);
    }
    if let Ok(b) = value.extract::<bool>() {
        return Ok(vantadb::sdk::VantaValue::Bool(b));
    }
    if let Ok(s) = value.extract::<String>() {
        return Ok(vantadb::sdk::VantaValue::String(s));
    }
    if let Ok(i) = value.extract::<i64>() {
        return Ok(vantadb::sdk::VantaValue::Int(i));
    }
    if let Ok(f) = value.extract::<f64>() {
        return Ok(vantadb::sdk::VantaValue::Float(f));
    }
    if let Ok(list) = value.cast::<PyList>() {
        if list.is_empty() {
            return Ok(vantadb::sdk::VantaValue::ListString(vec![]));
        }
        let first = list.get_item(0)?;
        if first.extract::<String>().is_ok() {
            let v: Vec<String> = list.extract()?;
            return Ok(vantadb::sdk::VantaValue::ListString(v));
        }
        if first.extract::<i64>().is_ok() {
            let v: Vec<i64> = list.extract()?;
            return Ok(vantadb::sdk::VantaValue::ListInt(v));
        }
        if first.extract::<f64>().is_ok() {
            let v: Vec<f64> = list.extract()?;
            return Ok(vantadb::sdk::VantaValue::ListFloat(v));
        }
        if first.extract::<bool>().is_ok() {
            let v: Vec<bool> = list.extract()?;
            return Ok(vantadb::sdk::VantaValue::ListBool(v));
        }
        return Ok(vantadb::sdk::VantaValue::ListString(vec![]));
    }
    Ok(vantadb::sdk::VantaValue::Null)
}

fn vanta_distance_to_mem0_score(distance: f32, metric: &vantadb::DistanceMetric) -> f32 {
    match metric {
        vantadb::DistanceMetric::Cosine => (1.0 - distance).max(0.0),
        vantadb::DistanceMetric::Euclidean => 1.0 / (1.0 + distance),
    }
}

fn mem0_namespace_from_collection(name: &str) -> String {
    name.replace(
        |c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-',
        "_",
    )
}

/// VantaDB vector store backend for Mem0.
///
/// Implements the Mem0 ``VectorStoreBase`` protocol so that
/// ``Memory.from_config({"vector_store": {"provider": "vantadb"}})``
/// works out of the box.
///
/// Usage::
///
///     from mem0 import Memory
///
///     config = {
///         "vector_store": {
///             "provider": "vantadb",
///             "config": {
///                 "path": "/tmp/vantadb-mem0",
///                 "collection_name": "memories",
///             },
///         },
///     }
///     memory = Memory.from_config(config)
#[pyclass(name = "VantaDBStore")]
pub struct VantaDBStore {
    engine: VantaEmbedded,
    collection_name: RwLock<String>,
}

#[pymethods]
impl VantaDBStore {
    #[new]
    #[pyo3(signature = (path, collection_name = "memories"))]
    fn new(path: &str, collection_name: &str) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: path.to_string(),
            ..Default::default()
        };
        let engine = VantaEmbedded::open_with_config(config)
            .map_err(|e| PyRuntimeError::new_err(format!("VantaDB open error: {:?}", e)))?;
        Ok(Self {
            engine,
            collection_name: RwLock::new(collection_name.to_string()),
        })
    }

    // ── Mem0 VectorStoreBase interface ──────────────────────────────────

    fn create_col(
        &self,
        _py: Python,
        name: &str,
        _vector_size: i32,
        _distance: &str,
    ) -> PyResult<()> {
        *self.collection_name.write().unwrap() = name.to_string();
        Ok(())
    }

    fn insert(
        &self,
        py: Python,
        vectors: Vec<Vec<f32>>,
        payloads: Option<Vec<Option<Py<PyAny>>>>,
        ids: Option<Vec<String>>,
    ) -> PyResult<Vec<String>> {
        let n = vectors.len();
        let namespace = mem0_namespace_from_collection(&self.collection_name.read().unwrap());
        let mut out_ids = Vec::with_capacity(n);

        for i in 0..n {
            let key = match &ids {
                Some(ids) if i < ids.len() => ids[i].clone(),
                _ => format!("mem_{i}"),
            };
            let payload = match &payloads {
                Some(list) if i < list.len() => match &list[i] {
                    Some(obj) => {
                        let py_ref = obj.bind(py);
                        if let Ok(s) = py_ref.extract::<String>() {
                            s
                        } else {
                            String::new()
                        }
                    }
                    None => String::new(),
                },
                _ => String::new(),
            };

            let mut input = VantaMemoryInput::new(&namespace, &key, &payload);
            input.vector = Some(vectors[i].clone());

            let record = self
                .engine
                .put(input)
                .map_err(|e| PyRuntimeError::new_err(format!("Insert error: {:?}", e)))?;
            out_ids.push(format!("{}:{}", record.namespace, record.key));
        }

        Ok(out_ids)
    }

    #[pyo3(signature = (query, vectors, top_k = 5, filters = None))]
    fn search(
        &self,
        py: Python,
        query: &str,
        vectors: Vec<Vec<f32>>,
        top_k: i32,
        filters: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let _ = query;
        let namespace = mem0_namespace_from_collection(&self.collection_name.read().unwrap());
        let metadata_filters = py_dict_to_metadata(filters)?;

        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        let request = VantaMemorySearchRequest {
            namespace,
            query_vector: vectors[0].clone(),
            filters: metadata_filters,
            text_query: None,
            top_k: top_k as usize,
            distance_metric: vantadb::DistanceMetric::Cosine,
            explain: false,
        };

        let hits = self
            .engine
            .search(request)
            .map_err(|e| PyRuntimeError::new_err(format!("Search error: {:?}", e)))?;

        let mut results = Vec::with_capacity(hits.len());
        for hit in hits {
            let d = PyDict::new(py);
            d.set_item("id", format!("{}:{}", hit.record.namespace, hit.record.key))?;
            d.set_item(
                "score",
                vanta_distance_to_mem0_score(hit.score, &vantadb::DistanceMetric::Cosine),
            )?;
            d.set_item("payload", &hit.record.payload)?;
            results.push(d.unbind().into());
        }

        Ok(results)
    }

    #[pyo3(signature = (vector_id))]
    fn delete(&self, _py: Python, vector_id: &str) -> PyResult<()> {
        let namespace = mem0_namespace_from_collection(&self.collection_name.read().unwrap());
        let key = vector_id.to_string();
        self.engine
            .delete(&namespace, &key)
            .map_err(|e| PyRuntimeError::new_err(format!("Delete error: {:?}", e)))?;
        Ok(())
    }

    #[pyo3(signature = (vector_id, vector = None, payload = None))]
    fn update(
        &self,
        _py: Python,
        vector_id: &str,
        vector: Option<Vec<Vec<f32>>>,
        payload: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let namespace = mem0_namespace_from_collection(&self.collection_name.read().unwrap());
        let key = vector_id.to_string();

        let existing = self
            .engine
            .get(&namespace, &key)
            .map_err(|e| PyRuntimeError::new_err(format!("Get before update error: {:?}", e)))?;

        let mut input = match existing {
            Some(record) => VantaMemoryInput::new(&record.namespace, &record.key, &record.payload),
            None => VantaMemoryInput::new(&namespace, &key, ""),
        };

        if let Some(vec_list) = vector {
            if !vec_list.is_empty() {
                input.vector = Some(vec_list[0].clone());
            }
        }

        if let Some(p) = payload {
            if let Ok(s) = p.extract::<String>() {
                input.payload = s;
            }
        }

        self.engine
            .put(input)
            .map_err(|e| PyRuntimeError::new_err(format!("Update error: {:?}", e)))?;

        Ok(())
    }

    #[pyo3(signature = (vector_id))]
    fn get(&self, py: Python, vector_id: &str) -> PyResult<Option<Py<PyAny>>> {
        let namespace = mem0_namespace_from_collection(&self.collection_name.read().unwrap());
        let key = vector_id.to_string();
        let record = self
            .engine
            .get(&namespace, &key)
            .map_err(|e| PyRuntimeError::new_err(format!("Get error: {:?}", e)))?;

        match record {
            Some(record) => {
                let d = PyDict::new(py);
                d.set_item("id", format!("{}:{}", record.namespace, record.key))?;
                d.set_item("payload", &record.payload)?;
                d.set_item("vector", record.vector.clone())?;
                Ok(Some(d.unbind().into()))
            }
            None => Ok(None),
        }
    }

    fn list_cols(&self, _py: Python) -> PyResult<Vec<String>> {
        self.engine
            .list_namespaces()
            .map_err(|e| PyRuntimeError::new_err(format!("List namespaces error: {:?}", e)))
    }

    fn delete_col(&self, _py: Python) -> PyResult<()> {
        let namespace = mem0_namespace_from_collection(&self.collection_name.read().unwrap());
        let page = self
            .engine
            .list(&namespace, VantaMemoryListOptions::default())
            .map_err(|e| PyRuntimeError::new_err(format!("List for delete_col error: {:?}", e)))?;

        for record in &page.records {
            self.engine
                .delete(&record.namespace, &record.key)
                .map_err(|e| {
                    PyRuntimeError::new_err(format!("Delete during delete_col error: {:?}", e))
                })?;
        }

        Ok(())
    }

    fn col_info(&self, py: Python) -> PyResult<Py<PyAny>> {
        let d = PyDict::new(py);
        d.set_item(
            "collection_name",
            self.collection_name.read().unwrap().clone(),
        )?;
        Ok(d.unbind().into())
    }

    #[pyo3(signature = (filters = None, top_k = None))]
    fn list(
        &self,
        py: Python,
        filters: Option<&Bound<'_, PyDict>>,
        top_k: Option<i32>,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let namespace = mem0_namespace_from_collection(&self.collection_name.read().unwrap());
        let limit = top_k.unwrap_or(100) as usize;

        let page = self
            .engine
            .list(
                &namespace,
                VantaMemoryListOptions {
                    filters: py_dict_to_metadata(filters)?,
                    limit,
                    cursor: None,
                },
            )
            .map_err(|e| PyRuntimeError::new_err(format!("List error: {:?}", e)))?;

        let mut results = Vec::with_capacity(page.records.len());
        for record in &page.records {
            let d = PyDict::new(py);
            d.set_item("id", format!("{}:{}", record.namespace, record.key))?;
            d.set_item("payload", &record.payload)?;
            d.set_item("vector", record.vector.clone())?;
            results.push(d.unbind().into());
        }

        Ok(results)
    }

    fn reset(&self, py: Python) -> PyResult<()> {
        self.delete_col(py)
    }
}

/// VantaDB Mem0 vector store backend.
///
/// Register in Mem0 as ``"provider": "vantadb"``.
#[pymodule]
fn vantadb_mem0(_py: Python, m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_class::<VantaDBStore>()?;
    m.add("__version__", "0.1.5")?;
    Ok(())
}
