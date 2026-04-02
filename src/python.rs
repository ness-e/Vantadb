#![cfg(feature = "python_sdk")]
use pyo3::prelude::*;
use crate::storage::StorageEngine;
use crate::node::UnifiedNode;

#[pyclass]
pub struct ClientEngine {
    _storage: StorageEngine,
}

#[pymethods]
impl ClientEngine {
    #[new]
    pub fn new() -> Self {
        ClientEngine {
            _storage: StorageEngine::new()
        }
    }

    /// High level query mapping directly traversing the execution plan.
    pub fn execute(&self, query: &str) -> PyResult<Vec<String>> {
        // Scaffolding physical execution invocation from python scope
        let simulated_result = format!("Executed via PyEngine: {}", query);
        Ok(vec![simulated_result])
    }

    /// Exposes node insertion directly to python scripts skipping HTTP serialization
    pub fn insert_node(&self, id: u64, vec_data: Option<Vec<f32>>) -> PyResult<()> {
        let mut node = UnifiedNode::new(id);
        if let Some(v) = vec_data {
            node.set_vector(v);
        }
        self._storage.put(node).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }
}

/// The python module definition. 
/// Compiled utilizing `maturin develop --features python_sdk`.
#[pymodule]
fn iadbms_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ClientEngine>()?;
    Ok(())
}
