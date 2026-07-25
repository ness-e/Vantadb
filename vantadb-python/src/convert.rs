//! Conversion helpers between internal VantaDB types and Python objects.
#![allow(deprecated)]

use pyo3::exceptions::{
    PyFileExistsError, PyFileNotFoundError, PyImportError, PyKeyError, PyOSError,
    PyPermissionError, PyRuntimeError, PyTimeoutError, PyTypeError, PyValueError,
};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyListMethods};
use std::cell::RefCell;
use std::collections::HashMap;
use vantadb::sdk::{
    VantaBm25TermContribution, VantaCapabilities, VantaExportReport, VantaHybridFusionReport,
    VantaImportReport, VantaIndexRebuildReport, VantaNodeRecord, VantaOperationalMetrics,
    VantaQueryResult, VantaRuntimeProfile, VantaSearchExplanation, VantaSearchExplanationHit,
    VantaStorageTier, VantaTextIndexAuditReport, VantaTextIndexRepairReport, VantaValue,
};

use crate::vector::VantaVector;

thread_local! {
    static LRU_CACHE: RefCell<LruCache> = RefCell::new(LruCache::new(64));
}

struct LruCache {
    map: HashMap<String, (std::collections::BTreeMap<String, VantaValue>, u64)>,
    capacity: usize,
    tick: u64,
}

impl LruCache {
    fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            capacity,
            tick: 0,
        }
    }

    /// Retrieve a cached metadata map by key.
    /// Moves the entry to the most-recently-used position on access in O(1).
    fn get(&mut self, key: &str) -> Option<std::collections::BTreeMap<String, VantaValue>> {
        if let Some((value, last_used)) = self.map.get_mut(key) {
            self.tick = self.tick.wrapping_add(1);
            *last_used = self.tick;
            Some(value.clone())
        } else {
            None
        }
    }

    /// Insert or update a metadata cache entry in O(1).
    /// Evicts the least recently used entry when at capacity.
    fn put(&mut self, key: String, value: std::collections::BTreeMap<String, VantaValue>) {
        self.tick = self.tick.wrapping_add(1);
        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            // Evict least recently used entry (minimum tick)
            if let Some(lru_key) = self
                .map
                .iter()
                .min_by_key(|(_, (_, last_used))| *last_used)
                .map(|(k, _)| k.clone())
            {
                self.map.remove(&lru_key);
            }
        }
        self.map.insert(key, (value, self.tick));
    }
}

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

pub(crate) fn rebuild_report_to_pydict(
    py: Python,
    report: &VantaIndexRebuildReport,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("scanned_nodes", report.scanned_nodes)?;
    dict.set_item("indexed_vectors", report.indexed_vectors)?;
    dict.set_item("skipped_tombstones", report.skipped_tombstones)?;
    dict.set_item("duration_ms", report.duration_ms)?;
    dict.set_item("derived_rebuild_ms", report.derived_rebuild_ms)?;
    dict.set_item("index_path", &report.index_path)?;
    dict.set_item("success", report.success)?;
    Ok(dict.unbind().into())
}

pub(crate) fn export_report_to_pydict(
    py: Python,
    report: &VantaExportReport,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("records_exported", report.records_exported)?;
    dict.set_item("namespaces", report.namespaces.clone())?;
    dict.set_item("path", &report.path)?;
    dict.set_item("duration_ms", report.duration_ms)?;
    Ok(dict.unbind().into())
}

pub(crate) fn import_report_to_pydict(
    py: Python,
    report: &VantaImportReport,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("inserted", report.inserted)?;
    dict.set_item("updated", report.updated)?;
    dict.set_item("skipped", report.skipped)?;
    dict.set_item("errors", report.errors)?;
    dict.set_item("duration_ms", report.duration_ms)?;
    Ok(dict.unbind().into())
}

pub(crate) fn text_index_repair_report_to_pydict(
    py: Python,
    report: &VantaTextIndexRepairReport,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("record_count", report.record_count)?;
    dict.set_item("posting_entries", report.posting_entries)?;
    dict.set_item("doc_stats_entries", report.doc_stats_entries)?;
    dict.set_item("term_stats_entries", report.term_stats_entries)?;
    dict.set_item("namespace_stats_entries", report.namespace_stats_entries)?;
    dict.set_item("duration_ms", report.duration_ms)?;
    dict.set_item("success", report.success)?;
    Ok(dict.unbind().into())
}

pub(crate) fn text_index_audit_report_to_pydict(
    py: Python,
    report: &VantaTextIndexAuditReport,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("schema_version", report.schema_version)?;
    dict.set_item("tokenizer", &report.tokenizer)?;
    dict.set_item("tokenizer_version", report.tokenizer_version)?;
    dict.set_item("key_format", &report.key_format)?;
    dict.set_item("namespace_filter", report.namespace_filter.clone())?;
    dict.set_item("namespaces_audited", report.namespaces_audited.clone())?;
    dict.set_item("records_scanned", report.records_scanned)?;
    dict.set_item("expected_entries", report.expected_entries)?;
    dict.set_item("actual_entries", report.actual_entries)?;
    dict.set_item("missing_entries", report.missing_entries)?;
    dict.set_item("unexpected_entries", report.unexpected_entries)?;
    dict.set_item("value_mismatches", report.value_mismatches)?;
    dict.set_item("unreadable_entries", report.unreadable_entries)?;
    dict.set_item("mismatches", report.mismatches)?;
    dict.set_item("deep_audit", report.deep_audit)?;
    dict.set_item("position_errors", report.position_errors)?;
    dict.set_item("tf_errors", report.tf_errors)?;
    dict.set_item("df_errors", report.df_errors)?;
    dict.set_item("doc_len_errors", report.doc_len_errors)?;
    dict.set_item("logical_corruptions", report.logical_corruptions)?;
    dict.set_item("state_valid", report.state_valid)?;
    dict.set_item("state_status", &report.state_status)?;
    dict.set_item("duration_ms", report.duration_ms)?;
    dict.set_item("passed", report.passed)?;
    dict.set_item("status", &report.status)?;
    Ok(dict.unbind().into())
}

pub(crate) fn operational_metrics_to_pydict(
    py: Python,
    metrics: &VantaOperationalMetrics,
) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("startup_ms", metrics.startup_ms)?;
    dict.set_item("wal_replay_ms", metrics.wal_replay_ms)?;
    dict.set_item("wal_records_replayed", metrics.wal_records_replayed)?;
    dict.set_item("ann_rebuild_ms", metrics.ann_rebuild_ms)?;
    dict.set_item(
        "ann_rebuild_scanned_nodes",
        metrics.ann_rebuild_scanned_nodes,
    )?;
    dict.set_item("derived_rebuild_ms", metrics.derived_rebuild_ms)?;
    dict.set_item("text_index_rebuild_ms", metrics.text_index_rebuild_ms)?;
    dict.set_item("text_postings_written", metrics.text_postings_written)?;
    dict.set_item("text_index_repairs", metrics.text_index_repairs)?;
    dict.set_item("text_lexical_queries", metrics.text_lexical_queries)?;
    dict.set_item("text_lexical_query_ms", metrics.text_lexical_query_ms)?;
    dict.set_item("text_candidates_scored", metrics.text_candidates_scored)?;
    dict.set_item("text_consistency_audits", metrics.text_consistency_audits)?;
    dict.set_item(
        "text_consistency_audit_failures",
        metrics.text_consistency_audit_failures,
    )?;
    dict.set_item("hybrid_query_ms", metrics.hybrid_query_ms)?;
    dict.set_item("hybrid_candidates_fused", metrics.hybrid_candidates_fused)?;
    dict.set_item("planner_hybrid_queries", metrics.planner_hybrid_queries)?;
    dict.set_item(
        "planner_text_only_queries",
        metrics.planner_text_only_queries,
    )?;
    dict.set_item(
        "planner_vector_only_queries",
        metrics.planner_vector_only_queries,
    )?;
    dict.set_item("records_exported", metrics.records_exported)?;
    dict.set_item("records_imported", metrics.records_imported)?;
    dict.set_item("import_errors", metrics.import_errors)?;
    dict.set_item("derived_prefix_scans", metrics.derived_prefix_scans)?;
    dict.set_item(
        "derived_full_scan_fallbacks",
        metrics.derived_full_scan_fallbacks,
    )?;
    // Per-subsystem memory breakdown
    dict.set_item("process_rss_bytes", metrics.process_rss_bytes)?;
    dict.set_item("process_virtual_bytes", metrics.process_virtual_bytes)?;
    dict.set_item("hnsw_nodes_count", metrics.hnsw_nodes_count)?;
    dict.set_item("hnsw_logical_bytes", metrics.hnsw_logical_bytes)?;
    dict.set_item("mmap_resident_bytes", metrics.mmap_resident_bytes)?;
    dict.set_item("volatile_cache_entries", metrics.volatile_cache_entries)?;
    dict.set_item("volatile_cache_cap_bytes", metrics.volatile_cache_cap_bytes)?;
    dict.set_item("jemalloc_allocated_bytes", metrics.jemalloc_allocated_bytes)?;
    dict.set_item("jemalloc_active_bytes", metrics.jemalloc_active_bytes)?;
    dict.set_item("jemalloc_metadata_bytes", metrics.jemalloc_metadata_bytes)?;
    dict.set_item("jemalloc_resident_bytes", metrics.jemalloc_resident_bytes)?;
    dict.set_item("jemalloc_mapped_bytes", metrics.jemalloc_mapped_bytes)?;
    dict.set_item("jemalloc_retained_bytes", metrics.jemalloc_retained_bytes)?;
    Ok(dict.unbind().into())
}

pub(crate) fn py_dict_to_metadata(
    fields: Option<&Bound<'_, PyDict>>,
) -> PyResult<std::collections::BTreeMap<String, VantaValue>> {
    let mut metadata = std::collections::BTreeMap::new();
    if let Some(extra) = fields {
        // Build value-aware cache key for small/common dicts
        let mut use_cache = extra.len() <= 4;
        let cache_key = if use_cache {
            let mut buf = String::with_capacity(64);
            let mut entries: Vec<(String, String)> = Vec::with_capacity(extra.len());
            for (key, value) in extra.iter() {
                let k: Result<String, _> = key.extract();
                let v = value.repr().map(|r| r.to_string());
                match (k, v) {
                    (Ok(k), Ok(v)) => entries.push((k, v)),
                    _ => {
                        use_cache = false;
                        break;
                    }
                }
            }
            if use_cache {
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                for (k, v) in &entries {
                    buf.push_str(k);
                    buf.push('=');
                    buf.push_str(v);
                    buf.push('\n');
                }
            }
            buf
        } else {
            String::new()
        };

        // Check cache first (CODE-014)
        if use_cache {
            let cached = LRU_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                cache.get(&cache_key)
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
                cache.put(cache_key, metadata.clone());
            });
        }
    }
    Ok(metadata)
}

/// Map a VantaError to the appropriate Python exception type for ergonomic
/// error handling on the Python side.
///
/// Mapping:
/// - `IoError(NotFound)` → `FileNotFoundError`
/// - `IoError(PermissionDenied)` → `PermissionError`
/// - `IoError(AlreadyExists)` → `FileExistsError`
/// - `IoError` (other) → `OSError`
/// - `NotFound` / `NodeNotFound` → `KeyError`
/// - `ValidationError`, `DuplicateNode`, `DimensionMismatch`, `SerializationError`,
///   `InvalidInput`, `SchemaError`, `IncompatibleFormat`,
///   `NodeIdCollision`, `IqlParseError`, `IqlError` → `ValueError`
/// - `Timeout` → `TimeoutError`
/// - All other variants → `RuntimeError` (catch-all)
pub(crate) fn map_vanta_error(err: vantadb::error::VantaError) -> PyErr {
    use vantadb::error::VantaError;
    match &err {
        VantaError::IoError(e) => match e.kind() {
            std::io::ErrorKind::NotFound => PyFileNotFoundError::new_err(err.to_string()),
            std::io::ErrorKind::PermissionDenied => PyPermissionError::new_err(err.to_string()),
            std::io::ErrorKind::AlreadyExists => PyFileExistsError::new_err(err.to_string()),
            _ => PyOSError::new_err(err.to_string()),
        },
        VantaError::NotFound { .. } | VantaError::NodeNotFound(_) => {
            PyKeyError::new_err(err.to_string())
        }
        VantaError::ValidationError { .. }
        | VantaError::DuplicateNode(_)
        | VantaError::DimensionMismatch { .. }
        | VantaError::SerializationError(_)
        | VantaError::InvalidInput(_)
        | VantaError::SchemaError(_)
        | VantaError::IncompatibleFormat { .. }
        | VantaError::NodeIdCollision(_)
        | VantaError::IqlParseError { .. }
        | VantaError::IqlError(_) => PyValueError::new_err(err.to_string()),
        VantaError::Timeout { .. } => PyTimeoutError::new_err(err.to_string()),
        _ => PyRuntimeError::new_err(err.to_string()),
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
