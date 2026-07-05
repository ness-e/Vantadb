//! Python bindings for the VantaDB vector-graph database via PyO3.
//!
//! This crate exposes the [`VantaDB`] class and a [`connect`] function
//! for in-process, zero-network-overhead access to VantaDB from Python.
#![warn(missing_docs)]

use pyo3::exceptions::{
    PyFileExistsError, PyFileNotFoundError, PyKeyError, PyOSError, PyPermissionError,
    PyRuntimeError, PyTimeoutError, PyTypeError, PyValueError,
};
use pyo3::prelude::*;
use pyo3::types::{
    PyAnyMethods, PyDict, PyDictMethods, PyList, PyListMethods, PyModuleMethods, PyTuple,
    PyTupleMethods,
};
use pyo3::Py;
use std::cell::RefCell;
use std::collections::HashMap;
use vantadb::config::VantaConfig;
use vantadb::metadata;
use vantadb::sdk::{
    VantaBm25TermContribution, VantaCapabilities, VantaEmbedded, VantaExportReport,
    VantaHybridFusionReport, VantaImportReport, VantaIndexRebuildReport, VantaMemoryInput,
    VantaMemoryListOptions, VantaMemoryRecord, VantaMemorySearchHit, VantaMemorySearchRequest,
    VantaNodeInput, VantaNodeRecord, VantaOperationalMetrics, VantaQueryResult,
    VantaRuntimeProfile, VantaSearchExplanation, VantaSearchExplanationHit, VantaStorageTier,
    VantaTextIndexAuditReport, VantaTextIndexRepairReport, VantaValue,
};
use vantadb::DistanceMetric;

thread_local! {
    static LRU_CACHE: RefCell<LruCache> = RefCell::new(LruCache::new(64));
}

struct LruCache {
    map: HashMap<String, std::collections::BTreeMap<String, VantaValue>>,
    order: Vec<String>,
    capacity: usize,
}

impl LruCache {
    fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            order: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Retrieve a cached metadata map by key.
    /// Moves the entry to the most-recently-used position on access.
    fn get(&mut self, key: &str) -> Option<std::collections::BTreeMap<String, VantaValue>> {
        if let Some(value) = self.map.get(key) {
            // Move to back (most recently used)
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
                self.order.push(key.to_string());
            }
            Some(value.clone())
        } else {
            None
        }
    }

    /// Insert or update a metadata cache entry.
    /// Evicts the least recently used entry when at capacity.
    /// Refreshes access order on update (CODE-038).
    fn put(&mut self, key: String, value: std::collections::BTreeMap<String, VantaValue>) {
        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            if let Some(lru_key) = self.order.first().cloned() {
                self.map.remove(&lru_key);
                self.order.remove(0);
            }
        }
        if !self.map.contains_key(&key) {
            self.order.push(key.clone());
        } else {
            // CODE-038: refresh order on update
            if let Some(pos) = self.order.iter().position(|k| *k == key) {
                self.order.remove(pos);
                self.order.push(key.clone());
            }
        }
        self.map.insert(key, value);
    }
}

fn py_any_to_value(value: &Bound<'_, PyAny>) -> PyResult<VantaValue> {
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
        if first.extract::<i64>().is_ok() {
            let mut vec = Vec::with_capacity(py_list.len());
            for item in py_list.iter() {
                vec.push(item.extract::<i64>()?);
            }
            return Ok(VantaValue::ListInt(vec));
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
        let first_type = first.get_type().name().unwrap_or("unknown");
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

/// Extract a `Vec<f32>` from a Python object using the buffer protocol
/// (NumPy, `array.array`, `memoryview`, `bytes`, `bytearray`) for zero-copy,
/// with fallback to Python list extraction.
///
/// Uses a thread-local buffer cache to reduce allocation churn in hot paths.
fn extract_vector<'py>(obj: &Bound<'py, PyAny>, py: Python<'py>) -> PyResult<Vec<f32>> {
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

fn set_python_value(
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

fn runtime_profile_label(profile: VantaRuntimeProfile) -> &'static str {
    match profile {
        VantaRuntimeProfile::Enterprise => "ENTERPRISE",
        VantaRuntimeProfile::Performance => "PERFORMANCE",
        VantaRuntimeProfile::LowResource => "LOW_RESOURCE",
    }
}

fn tier_label(tier: VantaStorageTier) -> &'static str {
    match tier {
        VantaStorageTier::Hot => "Hot",
        VantaStorageTier::Cold => "Cold",
    }
}

/// Convert a stable SDK node into a Python dictionary for maximum interop
/// with the AI ecosystem (LangChain, LlamaIndex, etc.)
fn node_to_pydict(py: Python, node: &VantaNodeRecord) -> PyResult<Py<PyAny>> {
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
fn format_query_result(result: &VantaQueryResult) -> String {
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

fn capabilities_to_pydict(py: Python, capabilities: &VantaCapabilities) -> PyResult<Py<PyAny>> {
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

fn memory_record_to_pydict(py: Python, record: &VantaMemoryRecord) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("namespace", &record.namespace)?;
    dict.set_item("key", &record.key)?;
    dict.set_item("payload", &record.payload)?;
    dict.set_item("created_at_ms", record.created_at_ms)?;
    dict.set_item("updated_at_ms", record.updated_at_ms)?;
    dict.set_item("version", record.version)?;
    dict.set_item("node_id", record.node_id)?;

    match &record.vector {
        Some(vector) => dict.set_item("vector", vector.clone())?,
        None => dict.set_item("vector", py.None())?,
    }

    match record.expires_at_ms {
        Some(ts) => dict.set_item("expires_at_ms", ts)?,
        None => dict.set_item("expires_at_ms", py.None())?,
    }

    let metadata = PyDict::new(py);
    for (key, value) in &record.metadata {
        set_python_value(py, &metadata, key, value)?;
    }
    dict.set_item("metadata", metadata)?;

    Ok(dict.unbind().into())
}

fn bm25_term_to_pydict(py: Python, term: &VantaBm25TermContribution) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("token", &term.token)?;
    dict.set_item("tf", term.tf)?;
    dict.set_item("df", term.df)?;
    dict.set_item("doc_len", term.doc_len)?;
    dict.set_item("contribution", term.contribution)?;
    Ok(dict.unbind().into())
}

fn explanation_hit_to_pydict(py: Python, exp: &VantaSearchExplanationHit) -> PyResult<Py<PyAny>> {
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

fn hybrid_fusion_report_to_pydict(
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

fn search_explanation_to_pydict(py: Python, exp: &VantaSearchExplanation) -> PyResult<Py<PyAny>> {
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

fn memory_hit_to_pydict(py: Python, hit: &VantaMemorySearchHit) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("score", hit.score)?;
    dict.set_item("record", memory_record_to_pydict(py, &hit.record)?)?;
    match &hit.explanation {
        Some(exp) => dict.set_item("explanation", explanation_hit_to_pydict(py, exp)?)?,
        None => dict.set_item("explanation", py.None())?,
    }
    Ok(dict.unbind().into())
}

fn rebuild_report_to_pydict(py: Python, report: &VantaIndexRebuildReport) -> PyResult<Py<PyAny>> {
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

fn export_report_to_pydict(py: Python, report: &VantaExportReport) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("records_exported", report.records_exported)?;
    dict.set_item("namespaces", report.namespaces.clone())?;
    dict.set_item("path", &report.path)?;
    dict.set_item("duration_ms", report.duration_ms)?;
    Ok(dict.unbind().into())
}

fn import_report_to_pydict(py: Python, report: &VantaImportReport) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("inserted", report.inserted)?;
    dict.set_item("updated", report.updated)?;
    dict.set_item("skipped", report.skipped)?;
    dict.set_item("errors", report.errors)?;
    dict.set_item("duration_ms", report.duration_ms)?;
    Ok(dict.unbind().into())
}

fn text_index_repair_report_to_pydict(
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

fn text_index_audit_report_to_pydict(
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

fn operational_metrics_to_pydict(
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

fn py_dict_to_metadata(
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

/// VantaDB — The vector-graph database that thinks.
/// In-process Python binding via PyO3. Zero network overhead.
///
/// Usage:
///     import vantadb_py as vanta
///     db = vanta.VantaDB("./my_brain", memory_limit_bytes=256 * 1024 * 1024)
///     db.insert(1, "Hello world", [0.1] * 384)
///     node = db.get(1)
///     results = db.search([0.1] * 384, top_k=5)
///     db.flush()
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
fn map_vanta_error(err: vantadb::error::VantaError) -> PyErr {
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

#[pyclass]
/// Python-accessible embedded VantaDB engine.
pub struct VantaDB {
    engine: VantaEmbedded,
}

#[pymethods]
impl VantaDB {
    /// Create or open a VantaDB database.
    ///
    /// Args:
    ///     db_path: Path to the database directory.
    ///     memory_limit_bytes: Optional memory budget in bytes for the Rust engine.
    ///         Isolates the DB's memory from Python's heap. If None, uses hardware
    ///         detection or VANTADB_MEMORY_LIMIT env var.
    ///     read_only: If True, opens the DB in read-only mode. Safe for multi-process
    ///         access when another process holds the write lock.
    #[new]
    #[pyo3(signature = (db_path, memory_limit_bytes=None, read_only=false, backend=None))]
    fn new(
        py: Python<'_>,
        db_path: &str,
        memory_limit_bytes: Option<u64>,
        read_only: bool,
        backend: Option<&str>,
    ) -> PyResult<Self> {
        let backend_kind = match backend {
            Some("rocksdb") => vantadb::BackendKind::RocksDb,
            Some("memory") => vantadb::BackendKind::InMemory,
            Some(other) => {
                tracing::warn!(
                    "Unknown backend \"{}\" — falling back to default (fjall). Known values: rocksdb, memory",
                    other
                );
                vantadb::BackendKind::Fjall
            }
            None => vantadb::BackendKind::Fjall,
        };
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            memory_limit: memory_limit_bytes,
            read_only,
            backend_kind,
            ..Default::default()
        };
        let engine = py
            .detach(move || VantaEmbedded::open_with_config(config))
            .map_err(map_vanta_error)?;

        Ok(VantaDB { engine })
    }

    /// Insert a node with content and an optional embedding vector.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during node insert.
    ///
    /// Args:
    ///     id: Unique node identifier (u64).
    ///     content: Text content stored as a relational field.
    ///     vector: Embedding vector (list of floats). Pass empty list for no vector.
    ///     fields: Optional dict of additional relational fields.
    #[pyo3(signature = (id, content, vector, fields=None))]
    fn insert(
        &self,
        py: Python,
        id: u64,
        content: &str,
        vector: &Bound<'_, PyAny>,
        fields: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let mut input = VantaNodeInput::new(id);
        input.content = Some(content.to_string());
        let v = extract_vector(vector, py)?;
        input.vector = (!v.is_empty()).then_some(v);

        if let Some(extra) = fields {
            for (key, value) in extra.iter() {
                let k: String = key.extract()?;
                input.fields.insert(k, py_any_to_value(&value)?);
            }
        }

        let engine = self.engine.clone();
        py.detach(move || engine.insert_node(input).map_err(map_vanta_error))?;

        Ok(())
    }

    /// Insert or update multiple namespace-scoped records in parallel (batched).
    ///
    /// Each entry is a tuple: `(namespace, key, payload, metadata_dict_or_None, vector_or_None, ttl_ms_or_None)`.
    /// Only `namespace`, `key`, `payload` are required; the remaining are keyword-style optional positional args.
    ///
    /// Returns a list of record dicts in the same order as inputs.
    /// Up to ~5x faster than sequential `put()` calls for large batches.
    #[pyo3(signature = (entries))]
    fn put_batch(&self, py: Python, entries: &Bound<'_, PyAny>) -> PyResult<Vec<Py<PyAny>>> {
        let mut inputs = Vec::with_capacity(entries.len().unwrap_or(0));
        for entry in entries.try_iter()? {
            let entry = entry?.cast::<PyTuple>()?.clone();
            if entry.len() < 3 {
                return Err(PyValueError::new_err(
                    "each entry must be a tuple of at least (namespace, key, payload)",
                ));
            }
            let namespace: String = entry.get_item(0)?.extract()?;
            let key: String = entry.get_item(1)?.extract()?;
            let payload: String = entry.get_item(2)?.extract()?;
            let dict = if entry.len() > 3 && !entry.get_item(3)?.is_none() {
                let item = entry.get_item(3)?;
                Some(item.cast::<PyDict>()?.clone())
            } else {
                None
            };
            let vector_obj: Option<Bound<'_, PyAny>> =
                if entry.len() > 4 && !entry.get_item(4)?.is_none() {
                    Some(entry.get_item(4)?)
                } else {
                    None
                };
            let ttl_ms: Option<u64> = if entry.len() > 5 {
                let item = entry.get_item(5)?;
                if item.is_none() {
                    None
                } else {
                    Some(item.extract()?)
                }
            } else {
                None
            };

            let mut input = VantaMemoryInput::new(namespace, key, payload);
            input.metadata = py_dict_to_metadata(dict.as_ref())?;
            input.ttl_ms = ttl_ms;
            input.vector = match &vector_obj {
                Some(v) => {
                    let vec = extract_vector(v, py)?;
                    (!vec.is_empty()).then_some(vec)
                }
                None => None,
            };
            inputs.push(input);
        }

        let engine = self.engine.clone();
        let records = py.detach(move || engine.put_batch(inputs).map_err(map_vanta_error))?;

        records
            .iter()
            .map(|record| memory_record_to_pydict(py, record))
            .collect()
    }

    /// Put or update a namespace-scoped persistent memory record.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (namespace, key, payload, metadata=None, vector=None, ttl_ms=None))]
    fn put(
        &self,
        py: Python,
        namespace: &str,
        key: &str,
        payload: &str,
        metadata: Option<&Bound<'_, PyDict>>,
        vector: Option<&Bound<'_, PyAny>>,
        ttl_ms: Option<u64>,
    ) -> PyResult<Py<PyAny>> {
        let mut input = VantaMemoryInput::new(namespace, key, payload);
        input.metadata = py_dict_to_metadata(metadata)?;
        input.ttl_ms = ttl_ms;
        input.vector = match vector {
            Some(v) => {
                let vec = extract_vector(v, py)?;
                (!vec.is_empty()).then_some(vec)
            }
            None => None,
        };

        let engine = self.engine.clone();
        let record = py.detach(move || engine.put(input).map_err(map_vanta_error))?;
        memory_record_to_pydict(py, &record)
    }

    /// Retrieve a namespace-scoped persistent memory record.
    fn get_memory(&self, py: Python, namespace: &str, key: &str) -> PyResult<Option<Py<PyAny>>> {
        let engine = self.engine.clone();
        let n = namespace.to_string();
        let k = key.to_string();
        let record = py.detach(move || engine.get(&n, &k).map_err(map_vanta_error))?;
        match record {
            Some(record) => Ok(Some(memory_record_to_pydict(py, &record)?)),
            None => Ok(None),
        }
    }

    /// Delete a namespace-scoped persistent memory record.
    fn delete_memory(&self, py: Python, namespace: &str, key: &str) -> PyResult<bool> {
        let engine = self.engine.clone();
        let namespace = namespace.to_string();
        let key = key.to_string();
        py.detach(move || engine.delete(&namespace, &key).map_err(map_vanta_error))
    }

    /// List namespace-scoped persistent memory records.
    #[pyo3(signature = (namespace, filters=None, limit=100, cursor=None))]
    fn list_memory(
        &self,
        py: Python,
        namespace: &str,
        filters: Option<&Bound<'_, PyDict>>,
        limit: usize,
        cursor: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let namespace = namespace.to_string();
        let filters_meta = py_dict_to_metadata(filters)?;
        let engine = self.engine.clone();
        let page = py.detach(move || {
            engine
                .list(
                    &namespace,
                    VantaMemoryListOptions {
                        filters: filters_meta,
                        limit,
                        cursor,
                    },
                )
                .map_err(map_vanta_error)
        })?;

        let dict = PyDict::new(py);
        let records = PyList::empty(py);
        for record in &page.records {
            records.append(memory_record_to_pydict(py, record)?)?;
        }
        dict.set_item("records", records)?;
        dict.set_item("next_cursor", page.next_cursor)?;
        Ok(dict.unbind().into())
    }

    /// Search namespace-scoped persistent memory records by vector + filters.
    #[pyo3(signature = (namespace, query_vector, filters=None, text_query=None, top_k=10, distance_metric=None, explain=false))]
    #[allow(clippy::too_many_arguments)]
    fn search_memory(
        &self,
        py: Python,
        namespace: &str,
        query_vector: &Bound<'_, PyAny>,
        filters: Option<&Bound<'_, PyDict>>,
        text_query: Option<String>,
        top_k: usize,
        distance_metric: Option<&str>,
        explain: bool,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let metric = match distance_metric {
            Some("euclidean") => DistanceMetric::Euclidean,
            Some(other) => {
                tracing::warn!(
                    "Unknown distance_metric \"{}\" — falling back to default (cosine). Known values: cosine, euclidean",
                    other
                );
                DistanceMetric::Cosine
            }
            None => DistanceMetric::Cosine,
        };

        let request = VantaMemorySearchRequest {
            namespace: namespace.to_string(),
            query_vector: extract_vector(query_vector, py)?,
            filters: py_dict_to_metadata(filters)?,
            text_query,
            top_k,
            distance_metric: metric,
            explain,
        };

        let engine = self.engine.clone();
        let hits = py.detach(move || engine.search(request).map_err(map_vanta_error))?;

        hits.iter()
            .map(|hit| memory_hit_to_pydict(py, hit))
            .collect()
    }

    /// Rebuild ANN and derived memory indexes from canonical storage.
    fn rebuild_index(&self, py: Python) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let report = py.detach(move || engine.rebuild_index().map_err(map_vanta_error))?;
        rebuild_report_to_pydict(py, &report)
    }

    /// Export one namespace as JSONL.
    fn export_namespace(&self, py: Python, path: &str, namespace: &str) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let path = path.to_string();
        let namespace = namespace.to_string();
        let report = py.detach(move || {
            engine
                .export_namespace(&path, &namespace)
                .map_err(map_vanta_error)
        })?;
        export_report_to_pydict(py, &report)
    }

    /// Export all namespaces as JSONL.
    fn export_all(&self, py: Python, path: &str) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let path = path.to_string();
        let report = py.detach(move || engine.export_all(&path).map_err(map_vanta_error))?;
        export_report_to_pydict(py, &report)
    }

    /// Import records from a VantaDB memory JSONL export.
    fn import_file(&self, py: Python, path: &str) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let path = path.to_string();
        let report = py.detach(move || engine.import_file(&path).map_err(map_vanta_error))?;
        import_report_to_pydict(py, &report)
    }

    /// Run a read-only structural audit of the derived text index.
    #[pyo3(signature = (namespace=None, deep=false))]
    fn audit_text_index(
        &self,
        py: Python,
        namespace: Option<&str>,
        deep: bool,
    ) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let namespace = namespace.map(|s| s.to_string());
        let report = py
            .detach(move || {
                let ns_ref = namespace.as_deref();
                if deep {
                    engine.audit_text_index_deep(ns_ref)
                } else {
                    engine.audit_text_index(ns_ref)
                }
            })
            .map_err(map_vanta_error)?;
        text_index_audit_report_to_pydict(py, &report)
    }

    /// Rebuild the text index from canonical storage as a repair primitive.
    fn repair_text_index(&self, py: Python) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let report = py.detach(move || engine.repair_text_index().map_err(map_vanta_error))?;
        text_index_repair_report_to_pydict(py, &report)
    }

    /// Return operational metrics for startup, replay, rebuild, export, and import.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during metrics snapshot.
    fn operational_metrics(&self, py: Python) -> PyResult<Py<PyAny>> {
        let engine = self.engine.clone();
        let metrics = py.detach(move || engine.operational_metrics());
        operational_metrics_to_pydict(py, &metrics)
    }

    /// Retrieve a node by ID. Returns a dict or None.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during database retrieval.
    fn get(&self, py: Python, id: u64) -> PyResult<Option<Py<PyAny>>> {
        let engine = self.engine.clone();
        let node = py.detach(move || engine.get_node(id).map_err(map_vanta_error))?;
        match node {
            Some(node) => Ok(Some(node_to_pydict(py, &node)?)),
            None => Ok(None),
        }
    }

    /// Delete a node by ID with an auditable reason (tombstone).
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during node deletion.
    #[pyo3(signature = (id, reason="manual deletion"))]
    fn delete(&self, py: Python, id: u64, reason: &str) -> PyResult<()> {
        let engine = self.engine.clone();
        let reason_str = reason.to_string();
        py.detach(move || engine.delete_node(id, &reason_str).map_err(map_vanta_error))
    }

    /// K-NN vector search. Returns a list of (node_id, distance) tuples.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during HNSW traversal.
    ///
    /// Args:
    ///     vector: Query embedding vector.
    ///     top_k: Number of nearest neighbors to return.
    #[pyo3(signature = (vector, top_k=10))]
    fn search(
        &self,
        py: Python,
        vector: &Bound<'_, PyAny>,
        top_k: usize,
    ) -> PyResult<Vec<(u64, f32)>> {
        let v = extract_vector(vector, py)?;
        let engine = self.engine.clone();
        py.detach(move || {
            engine
                .search_vector(&v, top_k)
                .map(|hits| {
                    hits.into_iter()
                        .map(|hit| (hit.node_id, hit.distance))
                        .collect()
                })
                .map_err(map_vanta_error)
        })
    }

    /// K-NN vector search for a batch of vectors.
    ///
    /// GIL Policy: RELEASED eager, runs search in parallel using Rayon.
    ///
    /// Args:
    ///     vectors: List of query embedding vectors.
    ///     top_k: Number of nearest neighbors to return per vector.
    #[pyo3(signature = (vectors, top_k=10))]
    fn search_batch(
        &self,
        py: Python,
        vectors: Vec<Bound<'_, PyAny>>,
        top_k: usize,
    ) -> PyResult<Vec<Vec<(u64, f32)>>> {
        let parsed: PyResult<Vec<Vec<f32>>> =
            vectors.iter().map(|v| extract_vector(v, py)).collect();
        let parsed = parsed?;
        let engine = self.engine.clone();
        py.detach(move || {
            use rayon::prelude::*;
            parsed
                .into_par_iter()
                .map(|vector| {
                    engine
                        .search_vector(&vector, top_k)
                        .map(|hits| {
                            hits.into_iter()
                                .map(|hit| (hit.node_id, hit.distance))
                                .collect()
                        })
                        .map_err(map_vanta_error)
                })
                .collect::<Result<Vec<Vec<(u64, f32)>>, _>>()
        })
    }

    /// Execute an IQL or LISP query string. Returns a formatted result string.
    ///
    /// GIL Policy: RELEASED during Tokio execution — allows other Python
    /// threads to run while VantaDB processes the query.
    fn query(&self, py: Python, iql_query: &str) -> PyResult<String> {
        let engine = self.engine.clone();
        let query_str = iql_query.to_string();

        py.detach(move || {
            engine
                .query(&query_str)
                .map(|result| format_query_result(&result))
                .map_err(map_vanta_error)
        })
    }

    /// Flush WAL and HNSW index to disk for durability.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during disk sync.
    fn flush(&self, py: Python) -> PyResult<()> {
        let engine = self.engine.clone();
        py.detach(move || engine.flush().map_err(map_vanta_error))
    }

    /// Compact the WAL: flush, archive ``vanta.wal`` as
    /// ``vanta.wal.<timestamp>``, and start a fresh WAL.
    #[pyo3(signature = ())]
    fn compact_wal(&self, py: Python) -> PyResult<()> {
        let engine = self.engine.clone();
        py.detach(move || engine.compact_wal().map_err(map_vanta_error))
    }

    /// Scan all memory records and physically delete expired ones.
    /// Returns the number of records purged.
    #[pyo3(signature = ())]
    fn purge_expired(&self, py: Python) -> PyResult<u64> {
        let engine = self.engine.clone();
        py.detach(move || engine.purge_expired().map_err(map_vanta_error))
    }

    /// Introspect the stable runtime capabilities exposed by the SDK boundary.
    fn capabilities(&self, py: Python) -> PyResult<Py<PyAny>> {
        let capabilities = self.engine.capabilities();
        capabilities_to_pydict(py, &capabilities)
    }

    /// Return capabilities and system memory telemetry.
    fn hardware_profile(&self, py: Python) -> PyResult<Py<PyAny>> {
        let caps_obj = self.capabilities(py)?;
        let metrics_obj = self.operational_metrics(py)?;

        let caps_dict = caps_obj.bind(py).cast::<PyDict>()?;
        let metrics_dict = metrics_obj.bind(py).cast::<PyDict>()?;

        // CODE-004: create a NEW dict instead of shallow-cloning caps_dict
        let merged_dict = PyDict::new(py);
        for entry in caps_dict.iter() {
            let (key, value) = entry;
            merged_dict.set_item(key, value)?;
        }

        let memory_keys = [
            "process_rss_bytes",
            "process_virtual_bytes",
            "hnsw_nodes_count",
            "hnsw_logical_bytes",
            "mmap_resident_bytes",
            "volatile_cache_entries",
            "volatile_cache_cap_bytes",
        ];

        for &key in &memory_keys {
            if let Some(val) = metrics_dict.get_item(key)? {
                merged_dict.set_item(key, val)?;
            }
        }

        Ok(merged_dict.unbind().into())
    }

    /// Add a labeled edge between two nodes.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during edge insertion.
    ///
    /// Args:
    ///     source_id: Source node ID.
    ///     target_id: Target node ID.
    ///     label: Edge label (e.g., "belongs_to", "similar_to").
    ///     weight: Optional edge weight (default 1.0).
    #[pyo3(signature = (source_id, target_id, label, weight=None))]
    fn add_edge(
        &self,
        py: Python,
        source_id: u64,
        target_id: u64,
        label: &str,
        weight: Option<f32>,
    ) -> PyResult<()> {
        let engine = self.engine.clone();
        let label_str = label.to_string();
        py.detach(move || {
            engine
                .add_edge(source_id, target_id, &label_str, weight)
                .map_err(map_vanta_error)
        })
    }

    /// Flush and close the embedded engine handle.
    fn close(&self, py: Python) -> PyResult<()> {
        let engine = self.engine.clone();
        py.detach(move || engine.close().map_err(map_vanta_error))
    }

    /// String representation showing the stable runtime profile.
    fn __repr__(&self) -> String {
        let caps = self.engine.capabilities();
        format!(
            "VantaDB(profile={}, read_only={}, vector_search={}, persistence={})",
            runtime_profile_label(caps.runtime_profile),
            caps.read_only,
            caps.vector_search,
            caps.persistence
        )
    }

    /// Breadth-First-Search starting from a designated set of root IDs,
    /// up to a maximum depth, returning the discovered distinct Node IDs.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during graph traversal.
    #[pyo3(signature = (roots, max_depth=999999))]
    fn graph_bfs(&self, py: Python, roots: Vec<u64>, max_depth: usize) -> PyResult<Vec<u64>> {
        let engine = self.engine.clone();
        py.detach(move || engine.graph_bfs(&roots, max_depth).map_err(map_vanta_error))
    }

    /// Depth-First-Search starting from a designated set of root IDs,
    /// up to a maximum depth, returning the discovered distinct Node IDs.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during graph traversal.
    #[pyo3(signature = (roots, max_depth=999999))]
    fn graph_dfs(&self, py: Python, roots: Vec<u64>, max_depth: usize) -> PyResult<Vec<u64>> {
        let engine = self.engine.clone();
        py.detach(move || engine.graph_dfs(&roots, max_depth).map_err(map_vanta_error))
    }

    /// Performs a topological sort on the subgraph reachable from the given roots.
    /// Returns an error if a cycle is detected.
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during topological sort.
    fn graph_topological_sort(&self, py: Python, roots: Vec<u64>) -> PyResult<Vec<u64>> {
        let engine = self.engine.clone();
        py.detach(move || {
            engine
                .graph_topological_sort(&roots)
                .map_err(map_vanta_error)
        })
    }

    /// Checks if the subgraph reachable from the given roots is a Directed Acyclic Graph (DAG).
    ///
    /// GIL Policy: RELEASED — allows Python threads to run during cycle detection.
    fn graph_is_dag(&self, py: Python, roots: Vec<u64>) -> PyResult<bool> {
        let engine = self.engine.clone();
        py.detach(move || engine.graph_is_dag(&roots).map_err(map_vanta_error))
    }

    /// Compact the storage layout: reorders nodes in BFS order to improve
    /// locality and free unused pages. Returns the number of nodes compacted.
    fn compact_layout(&self, py: Python) -> PyResult<u64> {
        let engine = self.engine.clone();
        py.detach(move || engine.compact_layout().map_err(map_vanta_error))
    }

    /// List all namespaces currently registered in the database.
    fn list_namespaces(&self, py: Python) -> PyResult<Vec<String>> {
        let engine = self.engine.clone();
        py.detach(move || engine.list_namespaces().map_err(map_vanta_error))
    }

    /// Generate a text snippet from a payload, highlighting matched query terms.
    #[pyo3(signature = (payload, text_query, with_highlighting=false))]
    fn generate_snippet(
        &self,
        py: Python,
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> PyResult<Option<String>> {
        let engine = self.engine.clone();
        let payload = payload.to_string();
        let text_query = text_query.to_string();
        py.detach(move || {
            let result = engine.generate_snippet(&payload, &text_query, with_highlighting);
            Ok(result)
        })
    }

    /// Explain how a memory search arrives at its results — returns a detailed
    /// breakdown of the search route, fusion, and per-hit explanation.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (namespace, query_vector, filters=None, text_query=None, top_k=10, distance_metric=None))]
    fn explain_memory_search(
        &self,
        py: Python,
        namespace: &str,
        query_vector: &Bound<'_, PyAny>,
        filters: Option<&Bound<'_, PyDict>>,
        text_query: Option<String>,
        top_k: usize,
        distance_metric: Option<&str>,
    ) -> PyResult<Py<PyAny>> {
        let metric = match distance_metric {
            Some("euclidean") => DistanceMetric::Euclidean,
            Some(other) => {
                tracing::warn!(
                    "Unknown distance_metric \"{}\" — falling back to default (cosine). Known values: cosine, euclidean",
                    other
                );
                DistanceMetric::Cosine
            }
            None => DistanceMetric::Cosine,
        };

        let request = VantaMemorySearchRequest {
            namespace: namespace.to_string(),
            query_vector: extract_vector(query_vector, py)?,
            filters: py_dict_to_metadata(filters)?,
            text_query,
            top_k,
            distance_metric: metric,
            explain: true,
        };

        let engine = self.engine.clone();
        let explanation = py.detach(move || {
            engine
                .explain_memory_search(request)
                .map_err(map_vanta_error)
        })?;

        search_explanation_to_pydict(py, &explanation)
    }
}

/// Connect to a VantaDB database.
///
/// Args:
///     path: Filesystem path, empty string, or ":memory:" for in-memory.
///     memory_limit: Optional memory budget in bytes.
///         Sets an upper bound on heap usage; when exceeded, VantaDB triggers a
///         controlled flush and architection of cold data to stay within budget.
#[pyfunction]
#[pyo3(signature = (path, memory_limit=None))]
fn connect(path: &str, memory_limit: Option<u64>) -> PyResult<VantaDB> {
    use vantadb::config::VantaConfig;
    use vantadb::sdk::VantaEmbedded;
    let config = VantaConfig {
        storage_path: if path.is_empty() || path == ":memory:" {
            ":memory:".to_string()
        } else {
            path.to_string()
        },
        memory_limit,
        ..Default::default()
    };
    let engine = VantaEmbedded::open_with_config(config).map_err(map_vanta_error)?;
    Ok(VantaDB { engine })
}

/// The Python module for VantaDB.
/// Usage: `import vantadb_py`
#[pymodule]
fn vantadb_py(_py: Python, m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_class::<VantaDB>()?;
    m.add_function(wrap_pyfunction!(connect, m)?)?;
    m.add("__version__", metadata::reported_version().into_owned())?;
    Ok(())
}
