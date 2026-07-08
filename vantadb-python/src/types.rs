//! Python-accessible #[pyclass] types for the VantaDB Python SDK.
//! Avoids per-result PyDict allocations in hot paths.

use pyo3::buffer::ReadOnlyCell;
use pyo3::exceptions::PyStopIteration;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use vantadb::sdk::VantaMemoryRecord;

use crate::set_python_value;
use crate::try_numpy_array;
use crate::VantaVector;

/// A zero-copy view over a 2D PyBuffer (NumPy ndarray) of f32 data.
///
/// Avoids the intermediate Vec<f32> allocation that `to_vec()` creates.
/// The view borrows from the underlying Python buffer object — the caller
/// must ensure the buffer outlives the view.
pub struct FlatBufferView<'a> {
    data: &'a [ReadOnlyCell<f32>],
    ndims: usize,
}

impl<'a> FlatBufferView<'a> {
    /// Create a view from a PyBuffer f32 slice, nrows, and ndims.
    pub fn new(data: &'a [ReadOnlyCell<f32>], _nrows: usize, ndims: usize) -> Self {
        Self { data, ndims }
    }

    /// Read the i-th row into an owned Vec<f32> by copying each element.
    /// Skips the intermediate full-buffer Vec<f32> from `to_vec()`.
    pub fn row_to_vec(&self, index: usize) -> Vec<f32> {
        let start = index * self.ndims;
        self.data[start..start + self.ndims]
            .iter()
            .map(|c| c.get())
            .collect()
    }
}

/// A Python-accessible memory record with typed getter properties.
///
/// Wraps a `VantaMemoryRecord` and exposes fields as individual properties
/// instead of allocating a PyDict per record.
#[pyclass(name = "VantaMemoryRecord", skip_from_py_object)]
#[derive(Clone)]
pub struct VantaPyMemoryRecord {
    pub inner: VantaMemoryRecord,
}

impl VantaPyMemoryRecord {
    /// Create from an owned SDK record (Rust-only, not a Python constructor).
    pub fn new(inner: VantaMemoryRecord) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl VantaPyMemoryRecord {
    #[getter]
    fn namespace(&self) -> &str {
        &self.inner.namespace
    }

    #[getter]
    fn key(&self) -> &str {
        &self.inner.key
    }

    #[getter]
    fn payload(&self) -> &str {
        &self.inner.payload
    }

    #[getter]
    fn metadata(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (k, v) in &self.inner.metadata {
            set_python_value(py, &dict, k, v)?;
        }
        Ok(dict.unbind())
    }

    #[getter]
    fn vector(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        // PERF-31: try numpy array first; fall back to VantaVector (backward compat)
        match &self.inner.vector {
            Some(v) => match try_numpy_array(py, v)? {
                Some(arr) => Ok(Some(arr)),
                None => Ok(Some(
                    py.get_type::<VantaVector>().call1((v.clone(),))?.unbind(),
                )),
            },
            None => Ok(None),
        }
    }

    #[getter]
    fn created_at_ms(&self) -> u64 {
        self.inner.created_at_ms
    }

    #[getter]
    fn updated_at_ms(&self) -> u64 {
        self.inner.updated_at_ms
    }

    #[getter]
    fn version(&self) -> u64 {
        self.inner.version
    }

    #[getter]
    fn node_id(&self) -> u128 {
        self.inner.node_id
    }

    #[getter]
    fn expires_at_ms(&self) -> Option<u64> {
        self.inner.expires_at_ms
    }

    fn __getitem__<'py>(&self, py: Python<'py>, key: &str) -> PyResult<Bound<'py, PyAny>> {
        use pyo3::conversion::IntoPyObject;
        Ok(match key {
            "namespace" => self.namespace().into_pyobject(py)?.into_any(),
            "key" => self.key().into_pyobject(py)?.into_any(),
            "payload" => self.payload().into_pyobject(py)?.into_any(),
            "metadata" => self.metadata(py)?.into_bound(py).into_any(),
            "vector" => match self.vector(py)? {
                Some(v) => v.into_bound(py),
                None => py.None().into_bound(py),
            },
            "created_at_ms" => self.created_at_ms().into_pyobject(py)?.into_any(),
            "updated_at_ms" => self.updated_at_ms().into_pyobject(py)?.into_any(),
            "version" => self.version().into_pyobject(py)?.into_any(),
            "node_id" => self.node_id().into_pyobject(py)?.into_any(),
            "expires_at_ms" => self.expires_at_ms().into_pyobject(py)?.into_any(),
            _ => {
                return Err(pyo3::exceptions::PyKeyError::new_err(format!(
                    "VantaMemoryRecord has no field '{key}'"
                )))
            }
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "VantaMemoryRecord(namespace={}, key={}, dim={})",
            self.inner.namespace,
            self.inner.key,
            self.inner.vector.as_ref().map(|v| v.len()).unwrap_or(0),
        )
    }
}

/// A Python-accessible list result page.
///
/// Wraps a page of memory records with pagination info.
#[pyclass(name = "VantaListResult", skip_from_py_object)]
#[derive(Clone)]
pub struct VantaPyListResult {
    pub records: Vec<VantaPyMemoryRecord>,
    pub next_cursor: Option<usize>,
}

impl VantaPyListResult {
    pub fn new(records: Vec<VantaPyMemoryRecord>, next_cursor: Option<usize>) -> Self {
        Self {
            records,
            next_cursor,
        }
    }
}

#[pymethods]
impl VantaPyListResult {
    /// Return the list of records in this page.
    #[getter]
    fn records(&self) -> Vec<VantaPyMemoryRecord> {
        self.records.clone()
    }

    /// Return the number of records in this page.
    #[getter]
    fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Cursor for the next page, or None if this was the last page.
    #[getter]
    fn next_cursor(&self) -> Option<usize> {
        self.next_cursor
    }

    fn __len__(&self) -> usize {
        self.records.len()
    }

    fn __getitem__<'py>(
        &self,
        py: Python<'py>,
        key: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        use pyo3::conversion::IntoPyObject;
        if let Ok(idx) = key.extract::<usize>() {
            match self.records.get(idx) {
                Some(r) => Ok(r.clone().into_pyobject(py)?.into_any()),
                None => Err(pyo3::exceptions::PyIndexError::new_err(
                    "list index out of range",
                )),
            }
        } else if let Ok(s) = key.extract::<String>() {
            match s.as_str() {
                "records" => Ok(self.records().into_pyobject(py)?.into_any()),
                "next_cursor" => Ok(self.next_cursor().into_pyobject(py)?.into_any()),
                "total_count" => Ok(self.total_count().into_pyobject(py)?.into_any()),
                _ => Err(pyo3::exceptions::PyKeyError::new_err(format!(
                    "VantaListResult has no field '{s}'"
                ))),
            }
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "VantaListResult indices must be integers or strings",
            ))
        }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> VantaListResultIter {
        VantaListResultIter {
            inner: slf.records.clone(),
            index: 0,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "VantaListResult(count={}, next_cursor={:?})",
            self.records.len(),
            self.next_cursor
        )
    }
}

/// Iterator for `VantaListResult`.
#[pyclass(name = "VantaListResultIter")]
struct VantaListResultIter {
    inner: Vec<VantaPyMemoryRecord>,
    index: usize,
}

#[pymethods]
impl VantaListResultIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<VantaPyMemoryRecord>> {
        if self.index < self.inner.len() {
            let val = self.inner[self.index].clone();
            self.index += 1;
            Ok(Some(val))
        } else {
            Err(PyStopIteration::new_err("end of iteration"))
        }
    }
}
