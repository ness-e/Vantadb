//! Python bindings for VantaVector and VantaVectorIter.
#![warn(missing_docs)]
#![allow(deprecated)]

use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

/// A zero-copy vector wrapper that exposes f32 data to NumPy via
/// `__array_interface__` without copying, while remaining sequence-iterable
/// for pure-Python consumers.
#[pyclass(name = "VantaVector")]
pub(crate) struct VantaVector {
    data: Box<[f32]>,
}

#[pymethods]
impl VantaVector {
    #[new]
    pub(crate) fn new(data: Vec<f32>) -> Self {
        VantaVector {
            data: data.into_boxed_slice(),
        }
    }

    fn __len__(&self) -> usize {
        self.data.len()
    }

    fn __getitem__(&self, idx: isize) -> PyResult<f32> {
        let len = self.data.len() as isize;
        let idx = if idx < 0 { len + idx } else { idx };
        if idx < 0 || idx >= len {
            return Err(PyIndexError::new_err("vector index out of range"));
        }
        Ok(self.data[idx as usize])
    }

    fn __iter__(slf: PyRef<'_, Self>) -> VantaVectorIter {
        VantaVectorIter {
            data: slf.data.to_vec(),
            index: 0,
        }
    }

    fn __repr__(&self) -> String {
        if self.data.len() <= 6 {
            format!("VantaVector({:?})", &self.data[..])
        } else {
            format!(
                "VantaVector([{:.4}, ..., {:.4}], dim={})",
                self.data[0],
                self.data[self.data.len() - 1],
                self.data.len()
            )
        }
    }

    /// NumPy ``__array_interface__`` protocol — exposes the internal f32 buffer
    /// directly so ``np.asarray(vector_obj)`` creates a zero-copy view.
    /// The backing `Box<[f32]>` is never reallocated (unlike `Vec`),
    /// preventing UB from a Rust realloc while NumPy references the buffer.
    #[getter(__array_interface__)]
    fn get_array_interface(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        let shape = PyTuple::new(py, [self.data.len()])?;
        dict.set_item("shape", shape)?;
        dict.set_item("typestr", "<f4")?;
        let data = (self.data.as_ptr() as usize, true);
        dict.set_item("data", data)?;
        dict.set_item("version", 3)?;
        Ok(dict.unbind().into())
    }

    /// Support ``__getstate__`` / ``__setstate__`` for pickle compatibility.
    fn __getstate__(&self) -> Vec<f32> {
        self.data.to_vec()
    }

    fn __setstate__(&mut self, state: Vec<f32>) {
        self.data = state.into_boxed_slice();
    }
}

/// Iterator for ``VantaVector`` that lets Python iterate over the elements
/// without first converting the whole vector to a list.
#[pyclass(name = "VantaVectorIter")]
pub(crate) struct VantaVectorIter {
    data: Vec<f32>,
    index: usize,
}

#[pymethods]
impl VantaVectorIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<f32>> {
        if self.index < self.data.len() {
            let val = self.data[self.index];
            self.index += 1;
            Ok(Some(val))
        } else {
            Err(pyo3::exceptions::PyStopIteration::new_err(
                "end of iteration",
            ))
        }
    }
}
