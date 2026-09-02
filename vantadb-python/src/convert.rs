//! Conversion helpers between internal VantaDB types and Python objects.
#![allow(deprecated)]

use lru::LruCache;
use pyo3::exceptions::{PyImportError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyListMethods};
use std::cell::RefCell;
use std::num::NonZeroUsize;
use vantadb::graph::TraversalDirection;
use vantadb::sdk::{
    VantaBm25TermContribution, VantaCapabilities, VantaExportReport, VantaFilterOp,
    VantaHybridFusionReport, VantaImportReport, VantaIndexRebuildReport, VantaMemoryFilter,
    VantaMemoryFilterItem, VantaNodeRecord, VantaOperationalMetrics, VantaQueryResult,
    VantaRuntimeProfile, VantaSearchExplanation, VantaSearchExplanationHit, VantaStorageTier,
    VantaTextIndexAuditReport, VantaTextIndexRepairReport, VantaValue,
};

use crate::vector::VantaVector;

// ─── Typed Python exception hierarchy (MOD-20) ───────────────────────────────
//
// `VantaError` is the base for every VantaDB error raised by this binding. It
// inherits from `RuntimeError` so existing `except RuntimeError` / `except
// Exception` callers keep working (backward compat). Each core `VantaError`
// variant maps to a specific subclass below (see `map_vanta_error`).
//
// Single-inheritance only: CPython's built-in exceptions have fixed memory
// layouts that make multiple exception inheritance unsafe (docs.python.org/3/
// library/exceptions.html — "recommended to only subclass one exception type
// at a time"), and PyO3's `create_exception!` takes a single base.
use pyo3::create_exception;

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

thread_local! {
    static LRU_CACHE: RefCell<LruCache<String, std::collections::BTreeMap<String, VantaValue>>> =
        RefCell::new(LruCache::new(CACHE_CAPACITY));
}

/// Cache capacity for small-dict metadata reuse in `py_dict_to_metadata` (CODE-014).
/// 64 is a compile-time constant — the match cannot fail.
const CACHE_CAPACITY: NonZeroUsize = match NonZeroUsize::new(64) {
    Some(cap) => cap,
    // 64 is a compile-time constant; unreachable!() without args is const-compatible.
    None => unreachable!(),
};

pub(crate) fn py_any_to_value(value: &Bound<'_, PyAny>) -> PyResult<VantaValue> {
    if value.is_none() {
        return Ok(VantaValue::Null);
    }
    if let Ok(boolean) = value.extract::<bool>() {
        return Ok(VantaValue::Bool(boolean));
    }
    if let Ok(dt) = value.extract::<chrono::DateTime<chrono::Utc>>() {
        return Ok(VantaValue::DateTime(dt));
    }
    if let Ok(dt) = value.extract::<chrono::DateTime<chrono::FixedOffset>>() {
        return Ok(VantaValue::DateTime(dt.with_timezone(&chrono::Utc)));
    }
    if let Ok(py_list) = value.cast::<pyo3::types::PyList>() {
        if py_list.is_empty() {
            return Ok(VantaValue::ListString(Vec::new()));
        }
        let first = py_list.get_item(0)?;
        if first.is_none() {
            return Err(PyTypeError::new_err("List elements cannot be None."));
        }
        // Check i64 before bool since Python bools are a subclass of int.
        // This ensures e.g. [0, 1] is classified as ListInt, not ListBool.
        if first.extract::<i64>().is_ok() {
            let mut vec = Vec::with_capacity(py_list.len());
            for item in py_list.iter() {
                vec.push(item.extract::<i64>()?);
            }
            return Ok(VantaValue::ListInt(vec));
        }
        if first.extract::<bool>().is_ok() {
            let mut vec = Vec::with_capacity(py_list.len());
            for item in py_list.iter() {
                vec.push(item.extract::<bool>()?);
            }
            return Ok(VantaValue::ListBool(vec));
        }
        if first.extract::<chrono::DateTime<chrono::Utc>>().is_ok()
            || first
                .extract::<chrono::DateTime<chrono::FixedOffset>>()
                .is_ok()
        {
            let mut vec = Vec::with_capacity(py_list.len());
            for item in py_list.iter() {
                if let Ok(dt) = item.extract::<chrono::DateTime<chrono::Utc>>() {
                    vec.push(dt);
                } else if let Ok(dt) = item.extract::<chrono::DateTime<chrono::FixedOffset>>() {
                    vec.push(dt.with_timezone(&chrono::Utc));
                } else {
                    return Err(PyTypeError::new_err(
                        "List elements must be consistent datetime objects.",
                    ));
                }
            }
            return Ok(VantaValue::ListDateTime(vec));
        }
        if first.extract::<f64>().is_ok() {
            let mut vec = Vec::with_capacity(py_list.len());
            for item in py_list.iter() {
                let val: f64 = item.extract()?;
                if val.is_nan() {
                    return Err(PyTypeError::new_err("ListFloat elements cannot be NaN."));
                }
                if val.is_infinite() {
                    return Err(PyTypeError::new_err(
                        "ListFloat elements cannot be Infinity.",
                    ));
                }
                vec.push(val);
            }
            return Ok(VantaValue::ListFloat(vec));
        }
        if first.extract::<String>().is_ok() {
            let mut vec = Vec::with_capacity(py_list.len());
            for item in py_list.iter() {
                vec.push(item.extract::<String>()?);
            }
            return Ok(VantaValue::ListString(vec));
        }
        let first_type = first
            .get_type()
            .name()
            .ok()
            .map(|n| n.to_string())
            .unwrap_or("unknown".into());
        return Err(PyTypeError::new_err(format!(
            "Unsupported list element type '{first_type}' (inferred from first element). \
             All list elements must be the same type: bool, int, float, str, or datetime."
        )));
    }
    if let Ok(string) = value.extract::<String>() {
        return Ok(VantaValue::String(string));
    }
    if let Ok(integer) = value.extract::<i64>() {
        return Ok(VantaValue::Int(integer));
    }
    if let Ok(float) = value.extract::<f64>() {
        if float.is_nan() {
            return Err(PyTypeError::new_err("Float field value cannot be NaN."));
        }
        if float.is_infinite() {
            return Err(PyTypeError::new_err(
                "Float field value cannot be Infinity.",
            ));
        }
        return Ok(VantaValue::Float(float));
    }

    Err(PyTypeError::new_err(
        "Unsupported field value. Use str, int, float, bool, datetime, list, or None.",
    ))
}

/// Try to create a NumPy float32 array from `&[f32]` data using `numpy.array()`.
///
/// Returns `Ok(None)` if numpy is not installed, allowing the caller to fall
/// back to a plain Python list or `VantaVector` (PERF-31).
pub(crate) fn try_numpy_array(py: Python<'_>, data: &[f32]) -> PyResult<Option<Py<PyAny>>> {
    let numpy_mod = match PyModule::import(py, "numpy") {
        Ok(m) => m,
        Err(e) => {
            if e.is_instance_of::<PyImportError>(py) {
                return Ok(None);
            }
            return Err(e);
        }
    };
    let array_fn = numpy_mod.getattr("array")?;
    let vv = VantaVector::new(data.to_vec());
    let result = array_fn.call1((vv,))?;
    Ok(Some(result.unbind().into()))
}

/// Extract a `Vec<f32>` from a Python object using the buffer protocol
/// (NumPy, `array.array`, `memoryview`, `bytes`, `bytearray`) for zero-copy,
/// with fallback to Python list extraction.
///
/// Uses a thread-local buffer cache to reduce allocation churn in hot paths.
// ponytail: P2-9 PyO3 PyBuffer::as_slice() devuelve ReadOnlyCell<f32> con cell.get() atomic load.
// Para 768 dims ~768ns overhead — implementar zero-copy real (PtrExt::as_ptr + copy_nonoverlapping)
// solo si profiling muestra bottleneck en extracción de vectores.
pub(crate) fn extract_vector<'py>(obj: &Bound<'py, PyAny>, py: Python<'py>) -> PyResult<Vec<f32>> {
    // Attempt zero-copy via buffer protocol (requires Python 3.11+)
    if let Ok(buf) = pyo3::buffer::PyBuffer::<f32>::get(obj) {
        if buf.is_c_contiguous() {
            if let Some(slice) = buf.as_slice(py) {
                return Ok(slice.iter().map(|cell| cell.get()).collect());
            }
        }
        // Non-contiguous or as_slice failed: use to_vec as fallback
        if let Ok(v) = buf.to_vec(py) {
            return Ok(v);
        }
    }
    // Try f64 buffer (common in NumPy) and downcast to f32
    if let Ok(buf) = pyo3::buffer::PyBuffer::<f64>::get(obj) {
        if buf.is_c_contiguous() {
            if let Ok(v) = buf.to_vec(py) {
                let len = v.len();
                let mut result = Vec::with_capacity(len);
                for x in v {
                    let cast = x as f32;
                    // CODE-082: warn on significant precision loss
                    if (cast as f64 - x).abs() > 1e-7 {
                        tracing::debug!(
                            "Precision loss casting f64 {} to f32: delta={}",
                            x,
                            (cast as f64 - x).abs()
                        );
                    }
                    result.push(cast);
                }
                return Ok(result);
            }
        }
    }
    // Fallback: PyO3 native Vec<f32> extraction
    // Use a temporary Vec to avoid redundant capacity allocation in hot path
    let result: Vec<f32> = obj.extract().map_err(|e| {
        PyTypeError::new_err(format!(
            "Expected a list of floats or a NumPy array (buffer protocol). Got: {}",
            e
        ))
    })?;
    Ok(result)
}

pub(crate) fn set_python_value(
    py: Python<'_>,
    dict: &Bound<'_, PyDict>,
    key: &str,
    value: &VantaValue,
) -> PyResult<()> {
    match value {
        VantaValue::String(value) => dict.set_item(key, value),
        VantaValue::Int(value) => dict.set_item(key, value),
        VantaValue::Float(value) => dict.set_item(key, value),
        VantaValue::Bool(value) => dict.set_item(key, value),
        VantaValue::DateTime(value) => dict.set_item(key, value),
        VantaValue::ListString(value) => dict.set_item(key, value),
        VantaValue::ListInt(value) => dict.set_item(key, value),
        VantaValue::ListFloat(value) => dict.set_item(key, value),
        VantaValue::ListBool(value) => dict.set_item(key, value),
        VantaValue::ListDateTime(value) => {
            let py_list = pyo3::types::PyList::new(py, value.iter())?;
            dict.set_item(key, py_list)
        }
        VantaValue::Null => dict.set_item(key, py.None()),
    }
}

pub(crate) fn runtime_profile_label(profile: VantaRuntimeProfile) -> &'static str {
    match profile {
        VantaRuntimeProfile::Enterprise => "ENTERPRISE",
        VantaRuntimeProfile::Performance => "PERFORMANCE",
        VantaRuntimeProfile::LowResource => "LOW_RESOURCE",
    }
}

pub(crate) fn tier_label(tier: VantaStorageTier) -> &'static str {
    match tier {
        VantaStorageTier::Hot => "Hot",
        VantaStorageTier::Cold => "Cold",
    }
}

/// Convert a stable SDK node into a Python dictionary for maximum interop
/// with the AI ecosystem (LangChain, LlamaIndex, etc.)
pub(crate) fn node_to_pydict(py: Python, node: &VantaNodeRecord) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("id", node.id)?;
    dict.set_item("confidence_score", node.confidence_score)?;
    dict.set_item("importance", node.importance)?;
    dict.set_item("hits", node.hits)?;
    dict.set_item("last_accessed", node.last_accessed)?;
    dict.set_item("epoch", node.epoch)?;
    dict.set_item("tier", tier_label(node.tier))?;
    dict.set_item("is_alive", node.is_alive)?;

    match &node.vector {
        Some(vector) => {
            dict.set_item("vector", vector.clone())?;
            dict.set_item("vector_dims", node.vector_dimensions)?;
        }
        None => {
            dict.set_item("vector", py.None())?;
            dict.set_item("vector_dims", node.vector_dimensions)?;
        }
    }

    let fields = PyDict::new(py);
    for (k, v) in &node.fields {
        set_python_value(py, &fields, k, v)?;
    }
    dict.set_item("fields", fields)?;

    let edges = PyList::empty(py);
    for e in &node.edges {
        let edge_tuple = (e.target, e.label.as_str(), e.weight);
        edges.append(edge_tuple)?;
    }
    dict.set_item("edges", edges)?;

    Ok(dict.unbind().into())
}

/// Format a stable SDK query result into a JSON-like string for Python consumption.
pub(crate) fn format_query_result(result: &VantaQueryResult) -> String {
    match result {
        VantaQueryResult::Read(nodes) => {
            let summaries: Vec<String> = nodes
                .iter()
                .map(|n| {
                    format!(
                        "{{id: {}, tier: {:?}, confidence: {:.2}, hits: {}}}",
                        n.id, n.tier, n.confidence_score, n.hits
                    )
                })
                .collect();
            format!("[{}]", summaries.join(", "))
        }
        VantaQueryResult::Write {
            affected_nodes,
            message,
            node_id,
        } => {
            format!(
                "{{affected: {}, message: \"{}\", node_id: {:?}}}",
                affected_nodes, message, node_id
            )
        }
        VantaQueryResult::StaleContext { node_id } => {
            format!(
                "{{stale_context: {}, action: \"rehydration_required\"}}",
                node_id
            )
        }
    }
}

/// Convert a `VantaQueryResult` into a structured Python dict (MOD-20),
/// mirroring the string form produced by `format_query_result` but as data so
/// callers can consume the result without parsing text.
///
/// `u128` node ids are returned as strings to avoid precision loss (same wire
/// convention as MCP/CLI).
pub(crate) fn query_result_to_pydict(py: Python, result: &VantaQueryResult) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    match result {
        VantaQueryResult::Read(nodes) => {
            dict.set_item("kind", "read")?;
            let node_list = PyList::empty(py);
            for n in nodes {
                let nd = PyDict::new(py);
                nd.set_item("id", n.id.to_string())?;
                nd.set_item("tier", format!("{:?}", n.tier))?;
                nd.set_item("confidence", n.confidence_score)?;
                nd.set_item("hits", n.hits)?;
                node_list.append(nd)?;
            }
            dict.set_item("nodes", node_list)?;
        }
        VantaQueryResult::Write {
            affected_nodes,
            message,
            node_id,
        } => {
            dict.set_item("kind", "write")?;
            dict.set_item("affected_nodes", affected_nodes)?;
            dict.set_item("message", message)?;
            dict.set_item("node_id", node_id.map(|id| id.to_string()))?;
        }
        VantaQueryResult::StaleContext { node_id } => {
            dict.set_item("kind", "stale_context")?;
            dict.set_item("node_id", node_id.to_string())?;
        }
    }
    Ok(dict.unbind().into())
}

pub(crate) fn capabilities_to_pydict(
    py: Python,
    capabilities: &VantaCapabilities,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item(
        "profile",
        runtime_profile_label(capabilities.runtime_profile),
    )?;
    dict.set_item("read_only", capabilities.read_only)?;
    dict.set_item("persistence", capabilities.persistence)?;
    dict.set_item("vector_search", capabilities.vector_search)?;
    dict.set_item("iql_queries", capabilities.iql_queries)?;
    Ok(dict.unbind().into())
}

pub(crate) fn bm25_term_to_pydict(
    py: Python,
    term: &VantaBm25TermContribution,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("token", &term.token)?;
    dict.set_item("tf", term.tf)?;
    dict.set_item("df", term.df)?;
    dict.set_item("doc_len", term.doc_len)?;
    dict.set_item("contribution", term.contribution)?;
    Ok(dict.unbind().into())
}

pub(crate) fn explanation_hit_to_pydict(
    py: Python,
    exp: &VantaSearchExplanationHit,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("identity", &exp.identity)?;
    dict.set_item("score", exp.score)?;
    dict.set_item("snippet", exp.snippet.clone())?;
    dict.set_item("matched_tokens", exp.matched_tokens.clone())?;
    dict.set_item("matched_phrases", exp.matched_phrases.clone())?;

    let bm25_terms = PyList::empty(py);
    for term in &exp.bm25_terms {
        bm25_terms.append(bm25_term_to_pydict(py, term)?)?;
    }
    dict.set_item("bm25_terms", bm25_terms)?;
    dict.set_item("rrf_text_rank", exp.rrf_text_rank)?;
    dict.set_item("rrf_vector_rank", exp.rrf_vector_rank)?;

    Ok(dict.unbind().into())
}

pub(crate) fn hybrid_fusion_report_to_pydict(
    py: Python,
    report: &VantaHybridFusionReport,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("text_candidates", report.text_candidates)?;
    dict.set_item("vector_candidates", report.vector_candidates)?;
    dict.set_item("fused_candidates", report.fused_candidates)?;
    dict.set_item("rrf_k", report.rrf_k)?;
    Ok(dict.unbind().into())
}

pub(crate) fn search_explanation_to_pydict(
    py: Python,
    exp: &VantaSearchExplanation,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("route", &exp.route)?;

    let hits = PyList::empty(py);
    for hit in &exp.hits {
        hits.append(explanation_hit_to_pydict(py, hit)?)?;
    }
    dict.set_item("hits", hits)?;

    match &exp.fusion_report {
        Some(report) => {
            dict.set_item("fusion_report", hybrid_fusion_report_to_pydict(py, report)?)?
        }
        None => dict.set_item("fusion_report", py.None())?,
    }

    Ok(dict.unbind().into())
}

macro_rules! pydict_set {
    ($py:expr, $( $key:expr => $val:expr ),* $(,)?) => {{
        let dict = PyDict::new($py);
        $(
            dict.set_item($key, $val)?;
        )*
        Ok::<Py<PyAny>, pyo3::PyErr>(dict.unbind().into())
    }};
}

pub(crate) fn rebuild_report_to_pydict(
    py: Python,
    report: &VantaIndexRebuildReport,
) -> PyResult<Py<PyAny>> {
    pydict_set!(py,
        "scanned_nodes" => report.scanned_nodes,
        "indexed_vectors" => report.indexed_vectors,
        "skipped_tombstones" => report.skipped_tombstones,
        "duration_ms" => report.duration_ms,
        "derived_rebuild_ms" => report.derived_rebuild_ms,
        "index_path" => &report.index_path,
        "success" => report.success,
    )
}

pub(crate) fn export_report_to_pydict(
    py: Python,
    report: &VantaExportReport,
) -> PyResult<Py<PyAny>> {
    pydict_set!(py,
        "records_exported" => report.records_exported,
        "namespaces" => report.namespaces.clone(),
        "path" => &report.path,
        "duration_ms" => report.duration_ms,
    )
}

pub(crate) fn import_report_to_pydict(
    py: Python,
    report: &VantaImportReport,
) -> PyResult<Py<PyAny>> {
    pydict_set!(py,
        "inserted" => report.inserted,
        "updated" => report.updated,
        "skipped" => report.skipped,
        "errors" => report.errors,
        "duration_ms" => report.duration_ms,
    )
}

pub(crate) fn bulk_import_report_to_pydict(
    py: Python,
    report: &vantadb::sdk::BulkImportReport,
) -> PyResult<Py<PyAny>> {
    pydict_set!(py,
        "total_records" => report.total_records,
        "batches_committed" => report.batches_committed,
        "duration_ms" => report.duration_ms,
    )
}

pub(crate) fn text_index_repair_report_to_pydict(
    py: Python,
    report: &VantaTextIndexRepairReport,
) -> PyResult<Py<PyAny>> {
    pydict_set!(py,
        "record_count" => report.record_count,
        "posting_entries" => report.posting_entries,
        "doc_stats_entries" => report.doc_stats_entries,
        "term_stats_entries" => report.term_stats_entries,
        "namespace_stats_entries" => report.namespace_stats_entries,
        "duration_ms" => report.duration_ms,
        "success" => report.success,
    )
}

pub(crate) fn text_index_audit_report_to_pydict(
    py: Python,
    report: &VantaTextIndexAuditReport,
) -> PyResult<Py<PyAny>> {
    pydict_set!(py,
        "schema_version" => report.schema_version,
        "tokenizer" => &report.tokenizer,
        "tokenizer_version" => report.tokenizer_version,
        "key_format" => &report.key_format,
        "namespace_filter" => report.namespace_filter.clone(),
        "namespaces_audited" => report.namespaces_audited.clone(),
        "records_scanned" => report.records_scanned,
        "expected_entries" => report.expected_entries,
        "actual_entries" => report.actual_entries,
        "missing_entries" => report.missing_entries,
        "unexpected_entries" => report.unexpected_entries,
        "value_mismatches" => report.value_mismatches,
        "unreadable_entries" => report.unreadable_entries,
        "mismatches" => report.mismatches,
        "deep_audit" => report.deep_audit,
        "position_errors" => report.position_errors,
        "tf_errors" => report.tf_errors,
        "df_errors" => report.df_errors,
        "doc_len_errors" => report.doc_len_errors,
        "logical_corruptions" => report.logical_corruptions,
        "state_valid" => report.state_valid,
        "state_status" => &report.state_status,
        "duration_ms" => report.duration_ms,
        "passed" => report.passed,
        "status" => &report.status,
    )
}

pub(crate) fn operational_metrics_to_pydict(
    py: Python,
    metrics: &VantaOperationalMetrics,
) -> PyResult<Py<PyAny>> {
    pydict_set!(py,
        "startup_ms" => metrics.startup_ms,
        "wal_replay_ms" => metrics.wal_replay_ms,
        "wal_records_replayed" => metrics.wal_records_replayed,
        "ann_rebuild_ms" => metrics.ann_rebuild_ms,
        "ann_rebuild_scanned_nodes" => metrics.ann_rebuild_scanned_nodes,
        "derived_rebuild_ms" => metrics.derived_rebuild_ms,
        "text_index_rebuild_ms" => metrics.text_index_rebuild_ms,
        "text_postings_written" => metrics.text_postings_written,
        "text_index_repairs" => metrics.text_index_repairs,
        "text_lexical_queries" => metrics.text_lexical_queries,
        "text_lexical_query_ms" => metrics.text_lexical_query_ms,
        "text_candidates_scored" => metrics.text_candidates_scored,
        "text_consistency_audits" => metrics.text_consistency_audits,
        "text_consistency_audit_failures" => metrics.text_consistency_audit_failures,
        "hybrid_query_ms" => metrics.hybrid_query_ms,
        "hybrid_candidates_fused" => metrics.hybrid_candidates_fused,
        "planner_hybrid_queries" => metrics.planner_hybrid_queries,
        "planner_text_only_queries" => metrics.planner_text_only_queries,
        "planner_vector_only_queries" => metrics.planner_vector_only_queries,
        "records_exported" => metrics.records_exported,
        "records_imported" => metrics.records_imported,
        "import_errors" => metrics.import_errors,
        "derived_prefix_scans" => metrics.derived_prefix_scans,
        "derived_full_scan_fallbacks" => metrics.derived_full_scan_fallbacks,
        "process_rss_bytes" => metrics.process_rss_bytes,
        "process_virtual_bytes" => metrics.process_virtual_bytes,
        "hnsw_nodes_count" => metrics.hnsw_nodes_count,
        "hnsw_logical_bytes" => metrics.hnsw_logical_bytes,
        "mmap_resident_bytes" => metrics.mmap_resident_bytes,
        "volatile_cache_entries" => metrics.volatile_cache_entries,
        "volatile_cache_cap_bytes" => metrics.volatile_cache_cap_bytes,
        "jemalloc_allocated_bytes" => metrics.jemalloc_allocated_bytes,
        "jemalloc_active_bytes" => metrics.jemalloc_active_bytes,
        "jemalloc_metadata_bytes" => metrics.jemalloc_metadata_bytes,
        "jemalloc_resident_bytes" => metrics.jemalloc_resident_bytes,
        "jemalloc_mapped_bytes" => metrics.jemalloc_mapped_bytes,
        "jemalloc_retained_bytes" => metrics.jemalloc_retained_bytes,
    )
}

pub(crate) fn py_dict_to_metadata(
    fields: Option<&Bound<'_, PyDict>>,
) -> PyResult<std::collections::BTreeMap<String, VantaValue>> {
    let mut metadata = std::collections::BTreeMap::new();
    if let Some(extra) = fields {
        if extra.is_empty() {
            return Ok(metadata);
        }

        // Build value-aware cache key for small/common dicts (1..=4 entries)
        let mut use_cache = extra.len() <= 4;
        let cache_key = if use_cache {
            let mut entries: Vec<(String, String)> = Vec::with_capacity(extra.len());
            for (key, value) in extra.iter() {
                if let (Ok(k), Ok(repr)) = (key.extract::<String>(), value.repr()) {
                    if let Ok(v) = repr.to_str() {
                        entries.push((k, v.to_string()));
                        continue;
                    }
                }
                use_cache = false;
                break;
            }
            if use_cache {
                entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));
                let mut buf = String::with_capacity(64);
                for (k, v) in entries {
                    buf.push_str(&k);
                    buf.push('=');
                    buf.push_str(&v);
                    buf.push('\n');
                }
                buf
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Check cache first (CODE-014)
        if use_cache {
            let cached = LRU_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                cache.get(&cache_key).cloned()
            });
            if let Some(meta) = cached {
                return Ok(meta);
            }
        }

        for (key, value) in extra.iter() {
            let k: String = key.extract()?;
            metadata.insert(k, py_any_to_value(&value)?);
        }

        // Cache the result for small dicts
        if use_cache && metadata.len() <= 4 {
            LRU_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                // O(1) eviction: lru evicts the least-recently-used entry at capacity
                // (AUD-039) — the old hand-rolled cache scanned with min_by_key (O(n)).
                let _ = cache.put(cache_key, metadata.clone());
            });
        }
    }
    Ok(metadata)
}

/// Build a core `VantaMemoryFilter` (operator filter_ops) from a Python dict,
/// following the canonical cross-SDK wire format used by CLI/MCP/TS:
///
/// - Flat value → implicit `$eq`: `{"field": "value"}`
/// - Operator object → one item per `$op` key: `{"field": {"$eq": v, "$gte": v2}}`
///
/// Supported operators: `$eq`, `$neq`, `$gt`, `$gte`, `$lt`, `$lte`.
/// An unknown operator raises `ValueError` (same error contract as the CLI/MCP
/// channels — never silently ignored). Values are converted with
/// `py_any_to_value` (str/int/float/bool/datetime/list/None).
pub(crate) fn py_dict_to_filter_ops(
    filters: Option<&Bound<'_, PyDict>>,
) -> PyResult<VantaMemoryFilter> {
    let mut ops: VantaMemoryFilter = Vec::new();
    let Some(dict) = filters else {
        return Ok(ops);
    };

    for (field, spec) in dict.iter() {
        let field: String = field.extract()?;
        // Flat value → implicit equality, matching the CLI/MCP flat form.
        if spec.cast::<PyDict>().is_err() {
            ops.push(VantaMemoryFilterItem {
                field: field.clone(),
                op: VantaFilterOp::Eq,
                value: py_any_to_value(&spec)?,
            });
            continue;
        }

        let op_dict = spec.cast::<PyDict>()?;
        for (op_str, val) in op_dict.iter() {
            let op: &str = op_str.extract()?;
            let op = match op {
                "$eq" => VantaFilterOp::Eq,
                "$neq" => VantaFilterOp::Neq,
                "$gt" => VantaFilterOp::Gt,
                "$gte" => VantaFilterOp::Gte,
                "$lt" => VantaFilterOp::Lt,
                "$lte" => VantaFilterOp::Lte,
                other => {
                    return Err(PyValueError::new_err(format!(
                        "Unknown filter operator '{other}' for field '{field}'. Supported: $eq, $neq, $gt, $gte, $lt, $lte"
                    )))
                }
            };
            ops.push(VantaMemoryFilterItem {
                field: field.clone(),
                op,
                value: py_any_to_value(&val)?,
            });
        }
    }
    Ok(ops)
}

/// Map a `VantaError` to the typed Python exception hierarchy (MOD-20).
///
/// Every core variant maps to a specific subclass of `VantaError`
/// (see the `create_exception!` block above). `VantaError` inherits from
/// `RuntimeError`, so `except RuntimeError` / `except Exception` callers keep
/// working; specific handlers use `NotFoundError`, `ValidationError`, etc.
///
/// Mapping (variant core → subclase Python):
/// - `NotFound` / `NodeNotFound` → `NotFoundError`
/// - `ValidationError`, `DuplicateNode`, `DimensionMismatch`,
///   `SerializationError`, `InvalidInput`, `SchemaError`, `NodeIdCollision`,
///   `IqlParseError`, `IqlError` → `ValidationError`
/// - `IncompatibleFormat`, `WALVersionMismatch`, `WalError` → `CorruptError`
/// - `IoError`, `BackendError` → `StorageError`
/// - `ExecutionConflict`, `CycleDetected` → `ConflictError`
/// - `UnsupportedOperation` → `UnsupportedError`
/// - `ResourceLimit` → `ResourceLimitError`
/// - `DatabaseBusy`, `NotInitialized` → `BusyError`
/// - `NoVectorForKey` → `NoVectorError`
/// - `Timeout` → `TimeoutError` (VantaDB's, not the builtin)
/// - remaining (`RuntimeError`, `Generic`, `CliError`, `SearchError`,
///   `RestoreError`, `BackupError`, …) → `VantaError` (base, catch-all)
pub(crate) fn map_vanta_error(err: vantadb::error::VantaError) -> PyErr {
    use vantadb::error::VantaError as CoreError;
    let py_err = match &err {
        CoreError::IoError(_) | CoreError::BackendError(_) => {
            StorageError::new_err(err.to_string())
        }
        CoreError::NotFound { .. } | CoreError::NodeNotFound(_) => {
            NotFoundError::new_err(err.to_string())
        }
        CoreError::ValidationError { .. }
        | CoreError::DuplicateNode(_)
        | CoreError::DimensionMismatch { .. }
        | CoreError::SerializationError(_)
        | CoreError::InvalidInput(_)
        | CoreError::SchemaError(_)
        | CoreError::NodeIdCollision(_)
        | CoreError::IqlParseError { .. }
        | CoreError::IqlError(_) => ValidationError::new_err(err.to_string()),
        CoreError::IncompatibleFormat { .. }
        | CoreError::WALVersionMismatch { .. }
        | CoreError::WalError(_) => CorruptError::new_err(err.to_string()),
        CoreError::Timeout { .. } => TimeoutError::new_err(err.to_string()),
        CoreError::ResourceLimit(_) => ResourceLimitError::new_err(err.to_string()),
        CoreError::ExecutionConflict { .. } | CoreError::CycleDetected => {
            ConflictError::new_err(err.to_string())
        }
        CoreError::UnsupportedOperation { .. } => UnsupportedError::new_err(err.to_string()),
        CoreError::DatabaseBusy(_) | CoreError::NotInitialized => {
            BusyError::new_err(err.to_string())
        }
        CoreError::NoVectorForKey(_) => NoVectorError::new_err(err.to_string()),
        // RuntimeError, Generic, CliError, SearchError, RestoreError, BackupError, …
        _ => VantaError::new_err(err.to_string()),
    };
    attach_err_meta(&py_err, &err);
    py_err
}

/// ERR-PY-01: attach the canonical error metadata (spec
/// `docs/api/ERROR_HANDLING.md` §5.1) to a freshly-mapped exception instance:
///
/// - `code` — exact `VANTADB_*` wire value from `VantaError::code()` (cross-binding contract)
/// - `retriable` — mirrors `is_retriable()`
/// - `hint` — recovery hint from `recovery_hint()`, `None` when absent
///
/// `create_exception!` types cannot carry `#[pymethods]` (static C types), so
/// `.to_dict()` is exposed as the module-level `error_to_dict()` helper in
/// `vantadb_py/__init__.py` instead (plain dict, same §5.2 shape).
///
/// `Python::attach` is required: call sites run both with the GIL held
/// (pyfunctions) and released (`py.detach` closures) — re-acquiring is a
/// cheap no-op when the GIL is already ours. Attribute set on an exception
/// instance cannot fail in practice (it has `__dict__`; values are
/// str/bool/None), so setattr errors are deliberately swallowed.
fn attach_err_meta(py_err: &PyErr, err: &vantadb::error::VantaError) {
    Python::attach(|py| {
        let obj = py_err.value(py);
        let _ = obj.setattr("code", err.code());
        let _ = obj.setattr("retriable", err.is_retriable());
        let _ = obj.setattr("hint", err.recovery_hint());
    });
}

/// Parse a `TraversalDirection` from its Python string name.
pub(crate) fn parse_direction(s: &str) -> PyResult<TraversalDirection> {
    match s {
        "Forward" => Ok(TraversalDirection::Forward),
        "Reverse" => Ok(TraversalDirection::Reverse),
        "Both" => Ok(TraversalDirection::Both),
        _ => Err(PyValueError::new_err(format!(
            "invalid direction '{s}': expected 'Forward', 'Reverse', or 'Both'"
        ))),
    }
}

/// Validate that optional parallel Vecs all match nrows length.
pub(crate) fn check_lens(
    payloads: &Option<Vec<String>>,
    metadatas: &Option<Vec<Option<Py<PyAny>>>>,
    namespaces: &Option<Vec<String>>,
    ttls: &Option<Vec<Option<u64>>>,
    nrows: usize,
) -> PyResult<()> {
    let check = |name: &str, len: usize| -> PyResult<()> {
        if len != nrows {
            Err(PyValueError::new_err(format!(
                "{}.len() ({}) must equal keys.len() ({})",
                name, len, nrows
            )))
        } else {
            Ok(())
        }
    };
    if let Some(ref p) = payloads {
        check("payloads", p.len())?;
    }
    if let Some(ref m) = metadatas {
        check("metadatas", m.len())?;
    }
    if let Some(ref n) = namespaces {
        check("namespaces", n.len())?;
    }
    if let Some(ref t) = ttls {
        check("ttls", t.len())?;
    }
    Ok(())
}
