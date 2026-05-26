#![cfg(feature = "python_sdk")]
use crate::node::{UnifiedNode, VectorRepresentations};
use crate::storage::StorageEngine;
use pyo3::prelude::*;
use pyo3::types::PyModuleMethods;

#[pyclass]
pub struct ClientEngine {
    _storage: StorageEngine,
}

impl Default for ClientEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl ClientEngine {
    #[new]
    pub fn new() -> Self {
        ClientEngine {
            _storage: StorageEngine::open("vantadb_data").expect("Failed to open StorageEngine"),
        }
    }

    /// High level query mapping directly traversing the execution plan.
    pub fn execute(&self, query: &str) -> PyResult<Vec<String>> {
        let executor = crate::executor::Executor::new(&self._storage);
        match executor.execute_hybrid(query) {
            Ok(crate::executor::ExecutionResult::Read(nodes)) => {
                let results = nodes
                    .into_iter()
                    .map(|n| format!("ID: {} | Relational: {:?}", n.id, n.relational))
                    .collect();
                Ok(results)
            }
            Ok(crate::executor::ExecutionResult::Write { message, .. }) => Ok(vec![message]),
            Ok(crate::executor::ExecutionResult::StaleContext(id)) => {
                Ok(vec![format!("STALE_CONTEXT: {}", id)])
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    /// Exposes node insertion directly to python scripts skipping HTTP serialization
    pub fn insert_node(&self, id: u64, vec_data: Option<Vec<f32>>) -> PyResult<()> {
        let mut node = UnifiedNode::new(id);
        if let Some(v) = vec_data {
            node.vector = VectorRepresentations::Full(v);
            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        }
        self._storage
            .insert(&node)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }
}

/// The python module definition.
/// Compiled utilizing `maturin develop --features python_sdk`.
#[pymodule]
fn vantadb(_py: Python<'_>, m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_class::<ClientEngine>()?;
    Ok(())
}
