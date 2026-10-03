//! Python-accessible #[pyclass] types for the VantaDB Python SDK.
//! Avoids per-result PyDict allocations in hot paths.
//!
//! Casing (Gate P, API-01 foundation): Python-native attributes stay
//! `snake_case` (`rec.node_id`) — the native interior never renames for wire
//! polish. JSON payloads emitted elsewhere follow
//! `docs/api/BINDINGS_NAMESPACES.md` § Casing Contract and migrate in
//! W1/API-02. PyO3 class names stay `PascalCase` (`Record`, `SearchHit`).

use pyo3::buffer::ReadOnlyCell;
use pyo3::exceptions::{PyRuntimeError, PyStopIteration};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyBytes, PyDict, PyDictMethods, PyTuple};
use vantadb::sdk::MemoryRecord;

use crate::convert::{set_python_value, try_numpy_array};
use crate::vector::Vector;

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
    // ponytail: P2-10 row_to_vec() copia por fila (inherente — necesitás Vec<f32> individual por record).
    // No hay zero-copy alternativo para put_batch() con PyBuffer. No aplicar en search().
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
/// Wraps a `MemoryRecord` and exposes fields as individual properties
/// instead of allocating a PyDict per record.
#[pyclass(name = "Record", skip_from_py_object)]
#[derive(Clone)]
pub struct VantaPyMemoryRecord {
    pub inner: MemoryRecord,
}

impl VantaPyMemoryRecord {
    /// Create from an owned SDK record (Rust-only, not a Python constructor).
    pub fn new(inner: MemoryRecord) -> Self {
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
        // PERF-31: try numpy array first; fall back to Vector (backward compat)
        match &self.inner.vector {
            Some(v) => match try_numpy_array(py, v)? {
                Some(arr) => Ok(Some(arr)),
                None => Ok(Some(py.get_type::<Vector>().call1((v.clone(),))?.unbind())),
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

    #[getter]
    fn superseded_by(&self) -> Option<String> {
        self.inner.superseded_by.clone()
    }

    #[getter]
    fn superseded_at_ms(&self) -> Option<u64> {
        self.inner.superseded_at_ms
    }

    /// Provenance class (ADR-046): `"Asserted"` (direct writer claim) or
    /// `"Derived"` (computed by the engine from `derived_from` parents).
    #[getter]
    fn confidence_class(&self) -> &'static str {
        self.inner.confidence_class.as_wire_str()
    }

    /// Declared/computed confidence range in `[0, 1]` (not a calibrated
    /// probability — see `docs/api/scores.md`; default `1.0` for asserted).
    #[getter]
    fn confidence(&self) -> f32 {
        self.inner.confidence
    }

    /// Last successful re-validation timestamp (ms); `None` = never validated.
    #[getter]
    fn last_validated_at_ms(&self) -> Option<u64> {
        self.inner.last_validated_at_ms
    }

    /// Parent record keys for a `derived` record (empty for `asserted`).
    #[getter]
    fn derived_from(&self) -> Vec<String> {
        self.inner.derived_from.clone()
    }

    /// Start of the validity window (ADR-046 §D3, SCH-02); v1 records
    /// normalize to `created_at_ms`.
    #[getter]
    fn valid_at_ms(&self) -> u64 {
        self.inner.valid_at_ms
    }

    /// End of the validity window (exclusive); `None` = open-ended.
    #[getter]
    fn invalid_at_ms(&self) -> Option<u64> {
        self.inner.invalid_at_ms
    }

    /// Quarantine entry timestamp (SCH-05); `None` = active.
    #[getter]
    fn quarantined_at_ms(&self) -> Option<u64> {
        self.inner.quarantined_at_ms
    }

    /// Stable quarantine reason code (`explicit_write`, `unreviewed_import`,
    /// `derived_promotion`, `policy_match`); the code set may grow.
    #[getter]
    fn quarantine_reason(&self) -> Option<String> {
        self.inner.quarantine_reason.clone()
    }

    /// Principal that applied the quarantine (or `system:<op>`).
    #[getter]
    fn quarantined_by(&self) -> Option<String> {
        self.inner.quarantined_by.clone()
    }

    /// Review deadline (ms); `None` = no default deadline configured.
    #[getter]
    fn quarantine_review_due_ms(&self) -> Option<u64> {
        self.inner.quarantine_review_due_ms
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
            "superseded_by" => self.superseded_by().into_pyobject(py)?.into_any(),
            "superseded_at_ms" => self.superseded_at_ms().into_pyobject(py)?.into_any(),
            "confidence_class" => self.confidence_class().into_pyobject(py)?.into_any(),
            "confidence" => self.confidence().into_pyobject(py)?.into_any(),
            "last_validated_at_ms" => self.last_validated_at_ms().into_pyobject(py)?.into_any(),
            "derived_from" => self.derived_from().into_pyobject(py)?.into_any(),
            "valid_at_ms" => self.valid_at_ms().into_pyobject(py)?.into_any(),
            "invalid_at_ms" => self.invalid_at_ms().into_pyobject(py)?.into_any(),
            "quarantined_at_ms" => self.quarantined_at_ms().into_pyobject(py)?.into_any(),
            "quarantine_reason" => self.quarantine_reason().into_pyobject(py)?.into_any(),
            "quarantined_by" => self.quarantined_by().into_pyobject(py)?.into_any(),
            "quarantine_review_due_ms" => self
                .quarantine_review_due_ms()
                .into_pyobject(py)?
                .into_any(),
            _ => {
                return Err(pyo3::exceptions::PyKeyError::new_err(format!(
                    "Record has no field '{key}'"
                )))
            }
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "Record(namespace={}, key={}, dim={})",
            self.inner.namespace,
            self.inner.key,
            self.inner.vector.as_ref().map(|v| v.len()).unwrap_or(0),
        )
    }
}

/// A Python-accessible list result page.
///
/// Wraps a page of memory records with pagination info.
#[pyclass(name = "ListResult", skip_from_py_object)]
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
                    "ListResult has no field '{s}'"
                ))),
            }
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "ListResult indices must be integers or strings",
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
            "ListResult(count={}, next_cursor={:?})",
            self.records.len(),
            self.next_cursor
        )
    }
}

/// Iterator for `VantaListResult`.
#[pyclass(name = "ListResultIter")]
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

/// A Python-accessible search hit returned by `search`.
///
/// Wraps a `MemoryRecord` plus the relevance score as typed getters,
/// avoiding per-hit PyDict allocation in the hot path.
#[pyclass(name = "SearchHit")]
pub(crate) struct VantaPySearchHit {
    pub(crate) inner: MemoryRecord,
    pub(crate) score: f32,
}

#[pymethods]
impl VantaPySearchHit {
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
        // PERF-31: try numpy array first; fall back to Vector (backward compat)
        match &self.inner.vector {
            Some(v) => match try_numpy_array(py, v)? {
                Some(arr) => Ok(Some(arr)),
                None => Ok(Some(py.get_type::<Vector>().call1((v.clone(),))?.unbind())),
            },
            None => Ok(None),
        }
    }

    #[getter]
    fn score(&self) -> f32 {
        self.score
    }

    #[getter]
    fn id(&self) -> u128 {
        self.inner.node_id
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

    #[getter]
    fn superseded_by(&self) -> Option<String> {
        self.inner.superseded_by.clone()
    }

    #[getter]
    fn superseded_at_ms(&self) -> Option<u64> {
        self.inner.superseded_at_ms
    }

    /// Provenance class (ADR-046): `"Asserted"` (direct writer claim) or
    /// `"Derived"` (computed by the engine from `derived_from` parents).
    #[getter]
    fn confidence_class(&self) -> &'static str {
        self.inner.confidence_class.as_wire_str()
    }

    /// Declared/computed confidence range in `[0, 1]` (not a calibrated
    /// probability — see `docs/api/scores.md`; default `1.0` for asserted).
    #[getter]
    fn confidence(&self) -> f32 {
        self.inner.confidence
    }

    /// Last successful re-validation timestamp (ms); `None` = never validated.
    #[getter]
    fn last_validated_at_ms(&self) -> Option<u64> {
        self.inner.last_validated_at_ms
    }

    /// Parent record keys for a `derived` record (empty for `asserted`).
    #[getter]
    fn derived_from(&self) -> Vec<String> {
        self.inner.derived_from.clone()
    }

    /// Start of the validity window (ADR-046 §D3, SCH-02).
    #[getter]
    fn valid_at_ms(&self) -> u64 {
        self.inner.valid_at_ms
    }

    /// End of the validity window (exclusive); `None` = open-ended.
    #[getter]
    fn invalid_at_ms(&self) -> Option<u64> {
        self.inner.invalid_at_ms
    }

    /// Quarantine entry timestamp (SCH-05); `None` = active.
    #[getter]
    fn quarantined_at_ms(&self) -> Option<u64> {
        self.inner.quarantined_at_ms
    }

    /// Stable quarantine reason code; the code set may grow.
    #[getter]
    fn quarantine_reason(&self) -> Option<String> {
        self.inner.quarantine_reason.clone()
    }

    /// Principal that applied the quarantine (or `system:<op>`).
    #[getter]
    fn quarantined_by(&self) -> Option<String> {
        self.inner.quarantined_by.clone()
    }

    /// Review deadline (ms); `None` = no default deadline configured.
    #[getter]
    fn quarantine_review_due_ms(&self) -> Option<u64> {
        self.inner.quarantine_review_due_ms
    }

    fn __repr__(&self) -> String {
        format!(
            "SearchHit(namespace={}, key={}, score={:.4}, dim={})",
            self.inner.namespace,
            self.inner.key,
            self.score,
            self.inner.vector.as_ref().map(|v| v.len()).unwrap_or(0),
        )
    }

    /// NumPy ``__array_interface__`` protocol — hands NumPy an *owned* copy of
    /// the vector (as `bytes`) so the resulting ndarray never aliases this
    /// pyclass's memory.
    ///
    /// SEC-01 (UAF): this previously exposed the raw `Vec<f32>` pointer as
    /// `(ptr, True)`. NumPy built a zero-copy view over that memory, so when the
    /// wrapper was dropped (or the vector was replaced) the ndarray was left
    /// pointing at freed memory and read garbage. Passing a buffer-protocol
    /// object (`bytes`) as `data` makes NumPy copy the buffer into the ndarray's
    /// own allocation — the ndarray then survives any drop/mutation of this
    /// pyclass. Same fix as AUDIT-01 in `vector.rs`.
    #[getter(__array_interface__)]
    fn get_search_hit_array_interface(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match &self.inner.vector {
            Some(v) => {
                let dict = PyDict::new(py);
                let shape = PyTuple::new(py, [v.len()])?;
                dict.set_item("shape", shape)?;
                dict.set_item("typestr", "<f4")?;
                // Owned little-endian f32 bytes (host-order is irrelevant;
                // to_le_bytes always emits "<f4" layout). NumPy copies this
                // buffer, so the array never aliases self.inner.vector.
                let le_bytes: Vec<u8> = v.iter().flat_map(|f| f.to_le_bytes()).collect();
                dict.set_item("data", PyBytes::new(py, &le_bytes))?;
                dict.set_item("version", 3)?;
                Ok(dict.unbind().into())
            }
            None => Err(PyRuntimeError::new_err("SearchHit has no vector")),
        }
    }
}
