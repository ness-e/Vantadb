//! Shared PyO3 helpers for VantaDB providers (openai, litellm, ollama).
//!
//! Included via `#[path = "../shared_py.rs"] mod common;` from each
//! provider's `python.rs`. This avoids creating a new crate while
//! eliminating the ~370 LOC duplication that caused drift bugs
//! (PROV-01 compile drift, precursor to PROV-04 contract drift).
//!
//! Visibility is `pub(super)` — items are accessible to the parent
//! module (`python` in each crate) but not exported.
//!
//! Surface decisions (PROV-05 canonical):
//! - `record_to_pydict` payload key: `"text"` (was drift openai="text"/litellm="payload")
//! - `record_to_pydict` extras: includes `node_id` (was litellm-only)
//! - search shape: full record + `"score"` (was ollama-minimal {id, text, score})
//!
//! PROV-04 (canonical contract unification) may revise these — tracked separately.

use pyo3::create_exception;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods};
use std::collections::{BTreeMap, HashMap};
use vantadb::error::VantaError as CoreError;
use vantadb::sdk::{VantaMemoryRecord, VantaMemorySearchRequest, VantaValue};

// ─── Typed Python exception hierarchy (MOD-20 parity, ERR-PY-01) ────────────
//
// Providers depend on `vantadb` only — NOT on `vantadb-python` — so they cannot
// import `convert.rs::map_vanta_error`. This mirrors the SDK's MOD-20 class
// names/semantics so provider clients can `except TimeoutError`, `except
// BusyError`, etc. instead of the old 4-bucket `KeyError`/`ValueError`/
// `RuntimeError` collapse that swallowed 6 fine-grained variants and leaked
// `Debug` formatting.
//
// ponytail: these are distinct type objects from the SDK's (per extension
// module); `except vantadb_py.NotFoundError` will NOT catch errors raised by
// vantadb_openai. Share via vantadb-python re-export (or a common crate) if a
// 3rd consumer appears — see docs/api/ERROR_HANDLING.md §5.
create_exception!(vantadb_py, VantaError, PyRuntimeError);
create_exception!(vantadb_py, NotFoundError, VantaError);
create_exception!(vantadb_py, ValidationError, VantaError);
create_exception!(vantadb_py, CorruptError, VantaError);
create_exception!(vantadb_py, StorageError, VantaError);
create_exception!(vantadb_py, ConflictError, VantaError);
create_exception!(vantadb_py, UnsupportedError, VantaError);
create_exception!(vantadb_py, ResourceLimitError, VantaError);
create_exception!(vantadb_py, BusyError, VantaError);
create_exception!(vantadb_py, NoVectorError, VantaError);
create_exception!(vantadb_py, TimeoutError, VantaError);

/// Attach the canonical `VANTADB_*` metadata (§5.1) to a mapped exception:
/// `code` (exact wire value), `retriable`, `hint`. `Python::attach` is
/// required because `err_to_py` runs inside `py.detach` closures (GIL
/// released); it is a cheap no-op when the GIL is already held. Exception
/// instances always have `__dict__`, so setattr failures are swallowed.
fn attach_err_meta(py_err: &PyErr, err: &CoreError) {
    Python::attach(|py| {
        let obj = py_err.value(py);
        let _ = obj.setattr("code", err.code());
        let _ = obj.setattr("retriable", err.is_retriable());
        let _ = obj.setattr("hint", err.recovery_hint());
    });
}

/// Convert a core `VantaError` into the MOD-20-parity hierarchy with
/// canonical metadata — the same variant→class table as the SDK's
/// `map_vanta_error` (vantadb-python/src/convert.rs). Messages go through
/// `Display` (never `Debug`); clients branch on `.code`, never on text.
pub(super) fn err_to_py(e: CoreError) -> PyErr {
    let py_err = match &e {
        CoreError::IoError(_) | CoreError::BackendError(_) => StorageError::new_err(e.to_string()),
        CoreError::NotFound { .. } | CoreError::NodeNotFound(_) => {
            NotFoundError::new_err(e.to_string())
        }
        CoreError::ValidationError { .. }
        | CoreError::DuplicateNode(_)
        | CoreError::DimensionMismatch { .. }
        | CoreError::SerializationError(_)
        | CoreError::InvalidInput(_)
        | CoreError::SchemaError(_)
        | CoreError::NodeIdCollision(_)
        | CoreError::IqlParseError { .. }
        | CoreError::IqlError(_) => ValidationError::new_err(e.to_string()),
        CoreError::IncompatibleFormat { .. }
        | CoreError::WALVersionMismatch { .. }
        | CoreError::WalError(_) => CorruptError::new_err(e.to_string()),
        CoreError::Timeout { .. } => TimeoutError::new_err(e.to_string()),
        CoreError::ResourceLimit(_) => ResourceLimitError::new_err(e.to_string()),
        CoreError::ExecutionConflict { .. } | CoreError::CycleDetected => {
            ConflictError::new_err(e.to_string())
        }
        CoreError::UnsupportedOperation { .. } => UnsupportedError::new_err(e.to_string()),
        CoreError::DatabaseBusy(_) | CoreError::NotInitialized => BusyError::new_err(e.to_string()),
        CoreError::NoVectorForKey(_) => NoVectorError::new_err(e.to_string()),
        // RuntimeError, Generic, CliError, SearchError, RestoreError, BackupError, …
        _ => VantaError::new_err(e.to_string()),
    };
    attach_err_meta(&py_err, &e);
    py_err
}

/// Register the MOD-20-parity exception classes on a provider's `#[pymodule]`
/// so clients can reference/catch them by name (e.g. `vantadb_openai.TimeoutError`).
pub(super) fn register_errors(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("VantaError", py.get_type::<VantaError>())?;
    m.add("NotFoundError", py.get_type::<NotFoundError>())?;
    m.add("ValidationError", py.get_type::<ValidationError>())?;
    m.add("CorruptError", py.get_type::<CorruptError>())?;
    m.add("StorageError", py.get_type::<StorageError>())?;
    m.add("ConflictError", py.get_type::<ConflictError>())?;
    m.add("UnsupportedError", py.get_type::<UnsupportedError>())?;
    m.add("ResourceLimitError", py.get_type::<ResourceLimitError>())?;
    m.add("BusyError", py.get_type::<BusyError>())?;
    m.add("NoVectorError", py.get_type::<NoVectorError>())?;
    m.add("TimeoutError", py.get_type::<TimeoutError>())?;
    Ok(())
}

/// Build a `PyDict` from a `VantaMemoryRecord` with the canonical surface:
///
/// Fields: `namespace`, `key`, `text`, `metadata`, `created_at_ms`,
/// `updated_at_ms`, `version`, `node_id`, optional `vector`, optional `expires_at_ms`.
pub(super) fn record_to_pydict(py: Python<'_>, r: VantaMemoryRecord) -> PyResult<Py<PyAny>> {
    let d = PyDict::new(py);
    d.set_item("namespace", &r.namespace)?;
    d.set_item("key", &r.key)?;
    d.set_item("text", &r.payload)?;
    d.set_item("metadata", vanta_values_to_pydict(py, &r.metadata)?)?;
    d.set_item("created_at_ms", r.created_at_ms)?;
    d.set_item("updated_at_ms", r.updated_at_ms)?;
    d.set_item("version", r.version)?;
    d.set_item("node_id", r.node_id)?;
    if let Some(ref v) = r.vector {
        d.set_item("vector", v.clone())?;
    }
    if let Some(exp) = r.expires_at_ms {
        d.set_item("expires_at_ms", exp)?;
    }
    Ok(d.unbind().into())
}

/// Convert a `VantaMemoryMetadata` (alias for `BTreeMap<String, VantaValue>`)
/// to a `PyDict` with native Python types. The match is exhaustive (ERR-PY-01
/// removed the `Debug` fallback): a new `VantaValue` variant is now a
/// compile error here instead of leaking `Debug` text into user metadata.
fn vanta_values_to_pydict(
    py: Python<'_>,
    meta: &BTreeMap<String, VantaValue>,
) -> PyResult<Py<PyDict>> {
    let d = PyDict::new(py);
    for (mk, mv) in meta {
        match mv {
            VantaValue::String(s) => d.set_item(mk, s)?,
            VantaValue::Int(i) => d.set_item(mk, i)?,
            VantaValue::Float(f) => d.set_item(mk, f)?,
            VantaValue::Bool(b) => d.set_item(mk, b)?,
            VantaValue::DateTime(dt) => d.set_item(mk, dt.to_rfc3339())?,
            VantaValue::ListString(v) => d.set_item(mk, v)?,
            VantaValue::ListInt(v) => d.set_item(mk, v)?,
            VantaValue::ListFloat(v) => d.set_item(mk, v)?,
            VantaValue::ListBool(v) => d.set_item(mk, v)?,
            VantaValue::ListDateTime(v) => {
                d.set_item(mk, v.iter().map(|dt| dt.to_rfc3339()).collect::<Vec<_>>())?
            }
            VantaValue::Null => d.set_item(mk, py.None())?,
        };
    }
    Ok(d.unbind())
}

/// Extract metadata from a Python `dict`. Supports `str`/`bool`/`int`/`float`
/// values. Returns `(parsed_map, dropped_keys)` so callers can emit a warning
/// when unsupported value types are encountered (matches existing UX).
pub(super) fn extract_metadata(
    meta: Option<&Bound<'_, PyDict>>,
) -> PyResult<(HashMap<String, VantaValue>, Vec<String>)> {
    let mut parsed: HashMap<String, VantaValue> = HashMap::new();
    let mut dropped_keys: Vec<String> = Vec::new();
    if let Some(meta) = meta {
        for (k, v) in meta.iter() {
            let key = match k.extract::<String>() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let val = v
                .extract::<String>()
                .ok()
                .map(VantaValue::String)
                .or_else(|| v.extract::<bool>().ok().map(VantaValue::Bool))
                .or_else(|| v.extract::<i64>().ok().map(VantaValue::Int))
                .or_else(|| v.extract::<f64>().ok().map(VantaValue::Float));
            match val {
                Some(val) => {
                    parsed.insert(key, val);
                }
                None => dropped_keys.push(key),
            }
        }
    }
    Ok((parsed, dropped_keys))
}

/// Parse a user-supplied distance metric string. Returns `Err(message)` for
/// unknown values; callers wrap in `PyValueError` to match PROV-07 contract.
pub(super) fn parse_distance_metric(s: Option<&str>) -> Result<vantadb::DistanceMetric, String> {
    match s {
        None | Some("cosine") => Ok(vantadb::DistanceMetric::Cosine),
        Some("euclidean") | Some("l2") => Ok(vantadb::DistanceMetric::Euclidean),
        Some(other) => Err(format!(
            "invalid distance_metric '{other}': expected \"cosine\", \"euclidean\" or \"l2\""
        )),
    }
}

/// Build a `VantaMemorySearchRequest` from individual arguments.
/// Filters are coerced to `VantaValue::String` (matches existing semantics).
#[allow(clippy::too_many_arguments)]
pub(super) fn build_search_request(
    namespace: &str,
    query_embedding: Vec<f32>,
    text_query: Option<String>,
    filters: Option<HashMap<String, String>>,
    metric: vantadb::DistanceMetric,
    top_k: usize,
) -> VantaMemorySearchRequest {
    VantaMemorySearchRequest {
        namespace: namespace.to_string(),
        query_vector: query_embedding,
        filters: filters
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, VantaValue::String(v)))
            .collect(),
        text_query,
        top_k,
        distance_metric: metric,
        explain: false,
        query_sparse: None,
        exclude_superseded: false,
        search_profile: None,
    }
}

#[cfg(test)]
mod err_py01_contract_tests {
    /// ERR-PY-01 sanity check (mirrors the PROV-07 `include_str!` pattern):
    /// `err_to_py` must route through the MOD-20 mirror + canonical code,
    /// and a Debug bucket may never be reintroduced in this file. The
    /// debug-format needle is kept escaped so it does not match itself nor
    /// the mechanical `grep` contract (which only sees the raw form).
    #[test]
    fn err_to_py_uses_mod20_hierarchy_and_canonical_code() {
        let src = include_str!("shared_py.rs");
        assert!(
            src.contains("create_exception!(vantadb_py, NotFoundError"),
            "MOD-20 parity classes must exist in shared_py.rs"
        );
        assert!(
            src.contains("obj.setattr(\"code\", err.code())"),
            "code attribute must be attached from VantaError::code()"
        );
        assert!(
            src.contains("obj.setattr(\"retriable\", err.is_retriable())"),
            "retriable attribute must be attached from is_retriable()"
        );
        assert!(
            !src.contains("format!(\"{:?}\""),
            "Debug formatting must never leak into Python-boundary messages"
        );
    }
}
