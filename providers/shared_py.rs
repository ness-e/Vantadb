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

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyDictMethods};
use std::collections::{BTreeMap, HashMap};
use vantadb::error::VantaError;
use vantadb::sdk::{VantaMemoryRecord, VantaMemorySearchRequest, VantaValue};

/// Convert a VantaError to a Python exception with consistent semantics:
///
/// - `NotFound` → `KeyError`
/// - `BackendError` → `RuntimeError`
/// - `InvalidInput` / `SchemaError` / `SerializationError` → `ValueError`
/// - everything else → `RuntimeError` (with Debug formatting)
pub(super) fn err_to_py(e: VantaError) -> PyErr {
    match e {
        VantaError::NotFound { .. } => pyo3::exceptions::PyKeyError::new_err(e.to_string()),
        VantaError::BackendError(_) => PyRuntimeError::new_err(e.to_string()),
        VantaError::InvalidInput(_)
        | VantaError::SchemaError(_)
        | VantaError::SerializationError(_) => {
            pyo3::exceptions::PyValueError::new_err(e.to_string())
        }
        _ => PyRuntimeError::new_err(format!("{:?}", e)),
    }
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
/// to a `PyDict`. Unknown `VantaValue` variants fall back to `Debug`
/// formatting (preserves existing drift-safety).
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
            other => d.set_item(mk, format!("{:?}", other))?,
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
