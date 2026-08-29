//! Input validation helpers for MCP tool parameters.

use crate::config::McpConfig;
use crate::error::McpError;
use serde::Serialize;
use serde_json::{json, Value};
use tracing::error;

// ── Input validation helpers ───────────────────────────────────────────────

pub(crate) fn validate_identifier(
    value: &str,
    label: &str,
    max_len: usize,
) -> Result<(), McpError> {
    if value.is_empty() {
        return Err(McpError::invalid_params(format!(
            "'{}' must not be empty",
            label
        )));
    }
    if value.len() > max_len {
        return Err(McpError::invalid_params(format!(
            "'{}' exceeds maximum length of {} bytes",
            label, max_len
        )));
    }
    if value.contains('\0') {
        return Err(McpError::invalid_params(format!(
            "'{}' contains null byte",
            label
        )));
    }
    Ok(())
}

/// MCP-34: stricter validator for identifier-shaped values that become a
/// single filesystem path segment (snapshot names, label keys, etc.).
/// Rejects path separators, '.', and '..' so the value can only be a
/// single, clean directory name. Defense in depth: the core
/// (`StorageEngine::validate_snapshot_name`) also rejects these, but the
/// MCP layer must reject first so an attacker probing the surface never
/// gets the core's filesystem error path.
///
/// Use this INSTEAD of `validate_identifier` only for fields used as a
/// single path segment — namespaces can legitimately contain `/` (e.g.
/// `mmd/s1/history` → IQL table `mmd_s1_history`), so they should keep
/// using the permissive `validate_identifier`. File paths (e.g.
/// `bulk_import_file.path`) use [`validate_safe_path`], which allows `/`
/// but blocks `..` and null bytes.
pub(crate) fn validate_path_segment(
    value: &str,
    label: &str,
    max_len: usize,
) -> Result<(), McpError> {
    validate_identifier(value, label, max_len)?;
    if value.contains('/') || value.contains('\\') || value == "." || value == ".." {
        return Err(McpError::invalid_params(format!(
            "'{label}' must be a plain identifier (no path separators, '.', or '..')"
        )));
    }
    Ok(())
}

/// MCP-34: validator for filesystem paths the MCP layer passes through to
/// the host filesystem (e.g. `bulk_import_file.path`). Allows `/` (legitimate
/// path separator) but blocks `..` (parent-directory traversal), null bytes,
/// and Windows-style `\` separators (so cross-platform clients can't sneak
/// in a Windows-only escape on a Unix host). The file-IO layer enforces
/// existence — this guard only shapes the surface.
pub(crate) fn validate_safe_path(value: &str, label: &str, max_len: usize) -> Result<(), McpError> {
    validate_identifier(value, label, max_len)?;
    if value.contains('\0')
        || value.contains('\\')
        || value.split(['/', '\\']).any(|seg| seg == "..")
    {
        return Err(McpError::invalid_params(format!(
            "'{label}' must be a safe filesystem path (no '..', '\\', or null bytes)"
        )));
    }
    Ok(())
}

pub(crate) fn validate_payload(value: &str, max_len: usize) -> Result<(), McpError> {
    if value.len() > max_len {
        return Err(McpError::invalid_params(format!(
            "Payload exceeds maximum length of {} bytes",
            max_len
        )));
    }
    Ok(())
}

pub(crate) fn validate_vector(array: &[Value], max_dim: usize) -> Result<Vec<f32>, McpError> {
    if array.is_empty() {
        return Err(McpError::invalid_params("Vector must not be empty"));
    }
    if array.len() > max_dim {
        return Err(McpError::invalid_params(format!(
            "Vector dimension {} exceeds maximum {}",
            array.len(),
            max_dim
        )));
    }
    let mut v = Vec::with_capacity(array.len());
    for val in array {
        let f = val
            .as_f64()
            .ok_or_else(|| McpError::invalid_params("Vector elements must be numbers"))?;
        if !f.is_finite() {
            return Err(McpError::invalid_params(
                "Vector elements must be finite numbers",
            ));
        }
        v.push(f as f32);
    }
    Ok(v)
}

/// Validate an MCP `search_profile` object against the config bounds.
///
/// The wire format is the serde shape of `SearchProfileConfig` itself
/// (mode + optional rrf_k/candidate_k), so parsing delegates to the core's
/// `Deserialize` — single source of truth (MEM-01) and guaranteed shape
/// parity with the native API and the IQL `PROFILE` clause (D13/D19).
/// Explicit numeric bounds are enforced here: `candidate_k` is a per-channel
/// candidate budget that could inflate memory (same trust-boundary pattern as
/// MCP-04's dimension check); `rrf_k = 0` would produce a degenerate fusion.
pub(crate) fn validate_search_profile(
    obj: &serde_json::Map<String, Value>,
    config: &McpConfig,
) -> Result<vantadb::sdk::SearchProfileConfig, McpError> {
    let profile: vantadb::sdk::SearchProfileConfig =
        serde_json::from_value(Value::Object(obj.clone()))
            .map_err(|e| McpError::invalid_params(format!("search_profile: {}", e)))?;
    if let Some(k) = profile.rrf_k {
        if k < 1 || k > config.max_rrf_k {
            return Err(McpError::invalid_params(format!(
                "search_profile.rrf_k must be in 1..={} (got {})",
                config.max_rrf_k, k
            )));
        }
    }
    if let Some(k) = profile.candidate_k {
        if k < 1 || k > config.max_candidate_k {
            return Err(McpError::invalid_params(format!(
                "search_profile.candidate_k must be in 1..={} (got {})",
                config.max_candidate_k, k
            )));
        }
    }
    Ok(profile)
}

/// Convert a single JSON metadata value into a `VantaValue`.
///
/// Scalars map 1:1. `null` and homogeneous arrays are delegated to the core,
/// which represents them as `VantaValue::Null` / `VantaValue::List*` and
/// filters them by strict equality (`matches_memory_filters`). This mirrors
/// the Python binding's `py_any_to_value` contract (ERR-026: a filter must
/// never be silently dropped — either the core applies it or the MCP rejects
/// the request with an explicit error). JSON objects have no `VantaValue`
/// variant and are rejected.
pub(crate) fn json_value_to_vanta_value(
    key: &str,
    val: &Value,
) -> Result<vantadb::sdk::VantaValue, McpError> {
    if let Some(s) = val.as_str() {
        return Ok(vantadb::sdk::VantaValue::String(s.to_string()));
    }
    if let Some(b) = val.as_bool() {
        return Ok(vantadb::sdk::VantaValue::Bool(b));
    }
    if let Some(i) = val.as_i64() {
        return Ok(vantadb::sdk::VantaValue::Int(i));
    }
    if let Some(f) = val.as_f64() {
        return Ok(vantadb::sdk::VantaValue::Float(f));
    }
    if val.is_null() {
        return Ok(vantadb::sdk::VantaValue::Null);
    }
    if let Some(items) = val.as_array() {
        return json_array_to_vanta_value(key, items);
    }
    Err(McpError::invalid_params(format!(
        "metadata field '{key}' has unsupported type object — supported: string, boolean, integer, float, null, or an array of one of those"
    )))
}

/// Convert a homogeneous JSON array into the matching `VantaValue::List*`
/// variant, mirroring `py_any_to_value`: the first element fixes the element
/// type and every element must match. Empty arrays become an empty string
/// list. Mixed-type, nested, or null-containing arrays cannot be represented
/// by the core and are rejected explicitly.
pub(crate) fn json_array_to_vanta_value(
    key: &str,
    items: &[Value],
) -> Result<vantadb::sdk::VantaValue, McpError> {
    let Some(first) = items.first() else {
        return Ok(vantadb::sdk::VantaValue::ListString(Vec::new()));
    };

    if first.as_str().is_some() {
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            match item.as_str() {
                Some(v) => out.push(v.to_string()),
                None => return Err(mixed_array_error(key)),
            }
        }
        return Ok(vantadb::sdk::VantaValue::ListString(out));
    }
    if first.as_bool().is_some() {
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            match item.as_bool() {
                Some(v) => out.push(v),
                None => return Err(mixed_array_error(key)),
            }
        }
        return Ok(vantadb::sdk::VantaValue::ListBool(out));
    }
    // i64 before f64: integral arrays (e.g. [1, 2]) must stay ListInt, not
    // ListFloat, so equality against stored Int metadata keeps matching.
    if first.as_i64().is_some() {
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            match item.as_i64() {
                Some(v) => out.push(v),
                None => return Err(mixed_array_error(key)),
            }
        }
        return Ok(vantadb::sdk::VantaValue::ListInt(out));
    }
    if first.as_f64().is_some() {
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            match item.as_f64() {
                Some(v) => out.push(v),
                None => return Err(mixed_array_error(key)),
            }
        }
        return Ok(vantadb::sdk::VantaValue::ListFloat(out));
    }

    Err(mixed_array_error(key))
}

pub(crate) fn mixed_array_error(key: &str) -> McpError {
    McpError::invalid_params(format!(
        "metadata field '{key}' has unsupported array — elements must all be the same type: string, boolean, integer, or float (no nested arrays, objects, or null)"
    ))
}

/// Parse a JSON sparse vector object into the core `SparseVector`.
///
/// The sparse vector format is an OBJECT mapping dimension id → weight
/// (e.g. `{"0": 0.5, "7": 1.25}`), matching `SparseVector(BTreeMap<u32, f32>)`
/// in `src/node/vector_data.rs`. Keys must parse as unsigned integers; values
/// must be finite numbers. Unlike dense `vector`, an empty object is valid
/// (a sparse vector with no entries).
pub(crate) fn parse_sparse_vector(
    obj: &serde_json::Map<String, Value>,
) -> Result<vantadb::SparseVector, McpError> {
    let mut sparse = vantadb::SparseVector::new();
    for (dim, val) in obj {
        let dim: u32 = dim.parse().map_err(|_| {
            McpError::invalid_params(format!(
                "sparse_vector key '{dim}' is not a valid dimension id (expected unsigned integer)"
            ))
        })?;
        let weight = val.as_f64().ok_or_else(|| {
            McpError::invalid_params(format!(
                "sparse_vector dimension '{dim}' must have a numeric weight"
            ))
        })?;
        if !weight.is_finite() {
            return Err(McpError::invalid_params(format!(
                "sparse_vector dimension '{dim}' must have a finite weight"
            )));
        }
        sparse.insert(dim, weight as f32);
    }
    Ok(sparse)
}

pub(crate) fn parse_metadata(
    obj: &serde_json::Map<String, Value>,
) -> Result<vantadb::sdk::VantaMemoryMetadata, McpError> {
    let mut meta = vantadb::sdk::VantaMemoryMetadata::new();
    for (key, val) in obj {
        meta.insert(key.clone(), json_value_to_vanta_value(key, val)?);
    }
    Ok(meta)
}

/// Parse a JSON `filters` object into an operator-based `VantaMemoryFilter`,
/// accepting BOTH published filter formats (AUD-048 — unified semantics with
/// the CLI channel):
///
/// - Flat values `{"field": value}` → implicit equality (`$eq`). This is the
///   MCP's long-published form and keeps working unchanged.
/// - Operator objects `{"field": {"$eq": v}}` / `{"field": {"$gt": v}}` etc.
///   → explicit operators, matching the CLI's `parse_filter_json`.
///
/// Any object value whose keys are not known operators is rejected explicitly
/// (never silently ignored), same error contract as the CLI channel.
pub(crate) fn parse_filter_ops(
    obj: &serde_json::Map<String, Value>,
) -> Result<vantadb::sdk::VantaMemoryFilter, McpError> {
    use vantadb::sdk::{VantaFilterOp, VantaMemoryFilterItem};

    let mut ops = Vec::with_capacity(obj.len());
    for (field, spec) in obj {
        let Some(spec_obj) = spec.as_object() else {
            // Flat value → implicit equality (MCP published flat form).
            ops.push(VantaMemoryFilterItem {
                field: field.clone(),
                op: VantaFilterOp::Eq,
                value: json_value_to_vanta_value(field, spec)?,
            });
            continue;
        };

        for (op_str, val_json) in spec_obj {
            let op = match op_str.as_str() {
                "$eq" => VantaFilterOp::Eq,
                "$neq" => VantaFilterOp::Neq,
                "$gt" => VantaFilterOp::Gt,
                "$gte" => VantaFilterOp::Gte,
                "$lt" => VantaFilterOp::Lt,
                "$lte" => VantaFilterOp::Lte,
                other => {
                    return Err(McpError::invalid_params(format!(
                        "Unknown filter operator '{other}' for field '{field}'. Supported: $eq, $neq, $gt, $gte, $lt, $lte"
                    )))
                }
            };
            let value = json_value_to_vanta_value(field, val_json)?;
            ops.push(VantaMemoryFilterItem {
                field: field.clone(),
                op,
                value,
            });
        }
    }
    Ok(ops)
}

/// Parse a graph node id from a JSON-RPC value.
///
/// Node ids are u128: JSON numbers lose precision above 2^53 and cannot
/// represent ids above 2^64, so MCP clients MUST pass ids as decimal strings.
/// A numeric value is still accepted for backward compatibility, mirroring
/// `vantadb::sdk::u128_serde`.
pub(crate) fn parse_node_id(val: &Value) -> Option<u128> {
    if let Some(s) = val.as_str() {
        return s.parse().ok();
    }
    val.as_u64().map(u128::from)
}

/// Serialize value to JSON string; on error produces a JSON-error string rather
/// than silently returning "".
pub(crate) fn escape_iql_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            c if c.is_control() => {
                out.push_str(&format!("\\x{:02x}", c as u8));
            }
            c => out.push(c),
        }
    }
    out
}

pub(crate) fn serialize_content(value: &impl Serialize) -> String {
    serde_json::to_string(value).unwrap_or_else(|e| {
        error!(%e, "serialize_content: serialization failed");
        r#"{"error":"Serialization failed"}"#.to_string()
    })
}

pub(crate) fn text_content(text: String) -> Value {
    json!({"content": [{"type": "text", "text": text}]})
}

/// MCP 2025-06-18 structured output: returns both `content` (text) and
/// `structuredContent` (machine-parsable JSON) per spec. The text payload
/// is the JSON serialization; structuredContent is the parsed Value itself.
pub(crate) fn structured_text_content(structured: &Value) -> Value {
    let text = serde_json::to_string(structured).unwrap_or_else(|e| {
        error!(%e, "structured_text_content: serialization failed");
        r#"{"error":"Serialization failed"}"#.to_string()
    });
    json!({"content": [{"type": "text", "text": text}], "structuredContent": structured.clone()})
}

/// Serialize `value` and return a structured response (text + structuredContent).
pub(crate) fn text_content_structured(value: &impl Serialize) -> Value {
    let structured = serde_json::to_value(value).unwrap_or_else(|e| {
        error!(%e, "text_content_structured: to_value failed");
        json!({"error": "Serialization failed"})
    });
    structured_text_content(&structured)
}

/// MCP-39: emit a search-style response that keeps the `content[0].text`
/// payload as the raw hits array (preserves back-compat for clients/tests
/// parsing `text` as `Vec<Value>`) while the new `byte_count` and
/// `truncated` metadata lives under `structuredContent` only. The
/// `budget_value` helper trims trailing hits if the array alone exceeds
/// `byte_budget`.
pub(crate) fn text_content_hits_with_budget<T: Serialize>(hits: &T, byte_budget: usize) -> Value {
    // Serialize the raw hits into the text payload (preserves the array
    // shape clients and tests expect).
    let text_hits = serde_json::to_value(hits).unwrap_or_else(|e| {
        error!(%e, "text_content_hits_with_budget: to_value failed");
        json!({"error": "Serialization failed"})
    });

    // Apply byte budget: if the hits array alone fits, no truncation. If not,
    // pop trailing entries (top-level array, since hits is a JSON array).
    let (budgeted_hits, truncated, byte_count) = budget_value(&text_hits, byte_budget);

    // text payload: the (possibly truncated) raw hits array.
    let text = serde_json::to_string(&budgeted_hits).unwrap_or_else(|e| {
        error!(%e, "text_content_hits_with_budget: to_string failed");
        r#"{"error":"Serialization failed"}"#.to_string()
    });

    // structuredContent: the new envelope (machine-readable) carrying
    // hits + budget metadata.
    let structured = match budgeted_hits {
        Value::Array(arr) => json!({
            "hits": arr,
            "byte_count": byte_count,
            "truncated": truncated,
        }),
        other => json!({
            "hits": other,
            "byte_count": byte_count,
            "truncated": truncated,
        }),
    };

    json!({
        "content": [{"type": "text", "text": text}],
        "structuredContent": structured,
    })
}

/// Human-readable name of a JSON value's type, for actionable error messages
/// that distinguish an absent field from a present-but-wrong-typed one.
pub(crate) fn json_value_type_name(val: &Value) -> &'static str {
    match val {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub(crate) fn error_content(msg: impl Into<String>) -> Value {
    json!({"isError": true, "content": [{"type": "text", "text": msg.into()}]})
}

/// MCP-39: budget a JSON value to fit within `byte_budget` bytes.
///
/// Returns the (possibly truncated) JSON value, a `truncated` flag, and the
/// final byte size. Truncation pops trailing elements from the *first* array
/// at the value's top level (the conventional shape for list/search tool
/// responses — `{records: [...]}`, `{hits: [...]}`). When no top-level array
/// is present, the value is returned intact and `truncated=false` even if it
/// exceeds the budget (the caller is expected to wrap a sized envelope).
///
/// This is the single chokepoint used by `memory_list`, `search_multi`, and
/// future list-shaped tools, so the truncation policy stays consistent across
/// the tool surface (pre-mortem: shapes distintos → helper genérico).
pub(crate) fn budget_value<T: Serialize>(value: &T, byte_budget: usize) -> (Value, bool, usize) {
    let serialized: Value = match serde_json::to_value(value) {
        Ok(v) => v,
        Err(_) => {
            // Fallback: hand the caller the original serialized form as a
            // string so the envelope still carries some signal rather than
            // dropping the response entirely.
            let raw = serialize_content(value);
            let size = raw.len();
            return (Value::String(raw), false, size);
        }
    };

    // Compute the candidate size. We always include the top-level object's
    // outer braces and key names in the budget check; the truncation only
    // touches array contents.
    let mut current = serialized.clone();
    let size_of = |v: &Value| -> usize { serde_json::to_string(v).map(|s| s.len()).unwrap_or(0) };
    let mut total_size = size_of(&current);
    if total_size <= byte_budget {
        return (current, false, total_size);
    }

    // Truncation path: pop trailing array elements until we fit. Two
    // shapes are supported:
    //  - top-level object with an array-valued key (e.g. `{records: [...]}`)
    //  - top-level array (e.g. raw `Vec<VantaSearchHit>`)
    // In both cases the *first* array is the truncation target.
    let array_key: Option<String> = if let Value::Object(map) = &current {
        map.iter()
            .find_map(|(k, v)| if v.is_array() { Some(k.clone()) } else { None })
    } else {
        None
    };
    if current.is_array() || array_key.is_some() {
        loop {
            total_size = size_of(&current);
            if total_size <= byte_budget {
                break;
            }
            let popped = if current.is_array() {
                if let Value::Array(items) = &mut current {
                    if !items.is_empty() {
                        items.pop();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else if let Some(key) = array_key.as_ref() {
                if let Some(map) = current.as_object_mut() {
                    match map.get_mut(key) {
                        Some(Value::Array(items)) if !items.is_empty() => {
                            items.pop();
                            true
                        }
                        Some(Value::Array(items)) if items.is_empty() => {
                            // Empty array → drop the key so the response
                            // shape stays consistent (no `{records: []}` if
                            // we truncated everything).
                            map.remove(key);
                            true
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            } else {
                false
            };
            if !popped {
                break;
            }
        }
        return (current, true, total_size);
    }

    // No top-level array to truncate; report size honestly. The caller
    // decides whether to escalate (e.g. via a tool-specific `next_cursor`).
    (current, false, total_size)
}

/// Stream a namespace's records page-by-page, invoking `f` on each record.
/// Never materializes the full namespace: at most `config.max_list_limit`
/// records are in memory at once (ERR-021: large namespaces OOM'd the server
/// when stats/list/delete collected the whole set into a Vec per call).
/// Returns the total number of records visited.
pub(crate) fn for_each_record(
    embedded: &vantadb::VantaEmbedded,
    namespace: &str,
    config: &McpConfig,
    mut f: impl FnMut(&vantadb::sdk::VantaMemoryRecord),
) -> Result<usize, String> {
    let mut count = 0usize;
    let mut cursor: Option<usize> = None;
    loop {
        let options = vantadb::sdk::VantaMemoryListOptions {
            limit: config.max_list_limit,
            cursor,
            #[allow(deprecated)]
            filters: vantadb::sdk::VantaMemoryMetadata::new(),
            filter_ops: None,
            exclude_superseded: false,
        };
        match embedded.list(namespace, options) {
            Ok(page) => {
                if page.records.is_empty() {
                    break;
                }
                for record in &page.records {
                    f(record);
                }
                count += page.records.len();
                match page.next_cursor {
                    Some(next) => cursor = Some(next),
                    None => break,
                }
            }
            Err(e) => return Err(format!("{}", e)),
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `validate_search_profile` delegates parsing to `SearchProfileConfig`'s
    /// Deserialize (single source of truth) and enforces explicit bounds.
    #[test]
    fn validate_search_profile_parses_and_bounds() {
        use serde_json::json;
        let config = McpConfig::default();

        let ok = json!({"mode": "keyword", "rrf_k": 30, "candidate_k": 64});
        let profile = validate_search_profile(ok.as_object().unwrap(), &config)
            .expect("valid profile should parse");
        assert_eq!(profile.mode, vantadb::sdk::SearchProfileMode::Keyword);
        assert_eq!(profile.rrf_k, Some(30));
        assert_eq!(profile.candidate_k, Some(64));

        // Empty object → serde defaults (mode Hybrid, None rrf/candidate).
        let empty = json!({});
        let profile = validate_search_profile(empty.as_object().unwrap(), &config)
            .expect("empty profile should default to hybrid");
        assert_eq!(profile.mode, vantadb::sdk::SearchProfileMode::Hybrid);
        assert_eq!(profile.rrf_k, None);

        // Unknown mode → serde enum error with a search_profile prefix.
        let bad_mode = json!({"mode": "bogus"});
        let err = validate_search_profile(bad_mode.as_object().unwrap(), &config).unwrap_err();
        assert!(
            err.message.contains("search_profile"),
            "mode error should name search_profile, got: {}",
            err.message
        );

        // rrf_k = 0 is degenerate (RRF would divide by zero-ish) → rejected.
        let bad_k = json!({"mode": "hybrid", "rrf_k": 0});
        let err = validate_search_profile(bad_k.as_object().unwrap(), &config).unwrap_err();
        assert!(
            err.message.contains("rrf_k"),
            "rrf_k error should name the field, got: {}",
            err.message
        );

        // candidate_k over the budget is a memory-inflation risk → rejected.
        let big_candidate = json!({"mode": "hybrid", "candidate_k": config.max_candidate_k + 1});
        let err = validate_search_profile(big_candidate.as_object().unwrap(), &config).unwrap_err();
        assert!(
            err.message.contains("candidate_k"),
            "candidate_k error should name the field, got: {}",
            err.message
        );
    }

    /// `parse_metadata` used to silently drop non-scalar metadata values
    /// (array/object/null), which turned a filter into no filter and returned
    /// a superset of results. The core CAN filter lists and null
    /// (`VantaValue::List*` / `Null` with strict equality), so the MCP must
    /// delegate them — mirroring the Python binding's `py_any_to_value`.
    /// Only JSON objects (no `VantaValue` variant) and mixed-type arrays are
    /// rejected explicitly.
    #[test]
    fn parse_metadata_delegates_lists_and_null_rejects_objects() {
        use serde_json::json;

        // Lists and null are delegable to the core.
        let delegable = json!({
            "tags": ["a", "b"],
            "counts": [1, 2],
            "ratios": [1.5, 2.5],
            "flags": [true, false],
            "empty": [],
            "flag": null,
            "s": "x", "b": true, "i": 42, "f": 1.5
        });
        let meta = parse_metadata(delegable.as_object().unwrap())
            .expect("lists, null, and scalars must be accepted");
        assert_eq!(
            meta.get("tags"),
            Some(&vantadb::sdk::VantaValue::ListString(vec![
                "a".to_string(),
                "b".to_string()
            ]))
        );
        assert_eq!(
            meta.get("counts"),
            Some(&vantadb::sdk::VantaValue::ListInt(vec![1, 2]))
        );
        assert_eq!(
            meta.get("ratios"),
            Some(&vantadb::sdk::VantaValue::ListFloat(vec![1.5, 2.5]))
        );
        assert_eq!(
            meta.get("flags"),
            Some(&vantadb::sdk::VantaValue::ListBool(vec![true, false]))
        );
        assert_eq!(
            meta.get("empty"),
            Some(&vantadb::sdk::VantaValue::ListString(Vec::new()))
        );
        assert_eq!(meta.get("flag"), Some(&vantadb::sdk::VantaValue::Null));
        assert_eq!(meta.len(), 10);

        // Objects cannot be represented by VantaValue → explicit error.
        let object_value = json!({"nested": {"a": 1}});
        let object = object_value.as_object().unwrap();
        let object_err = parse_metadata(object).unwrap_err();
        assert!(
            object_err.message.contains("nested"),
            "object metadata must error naming the key, got: {}",
            object_err.message
        );

        // Mixed-type arrays cannot be represented either.
        let mixed_value = json!({"tags": ["a", 1]});
        let mixed = mixed_value.as_object().unwrap();
        let mixed_err = parse_metadata(mixed).unwrap_err();
        assert!(
            mixed_err.message.contains("tags"),
            "mixed array metadata must error naming the key, got: {}",
            mixed_err.message
        );

        // Scalars are still accepted (regression guard).
        let scalars_value = json!({"s": "x", "b": true, "i": 42, "f": 1.5});
        let scalars = scalars_value.as_object().unwrap();
        let scalars_meta = parse_metadata(scalars).expect("scalar metadata is supported");
        assert_eq!(scalars_meta.len(), 4);
    }

    /// MCP-39: `budget_value` returns the value intact when it already fits
    /// inside the byte budget, and reports `truncated=false` with the real
    /// serialized size.
    #[test]
    fn budget_value_fits_without_truncation() {
        use serde_json::json;
        let value = json!({"records": [{"k": "a"}, {"k": "b"}]});
        let (out, truncated, size) = budget_value(&value, 1024);
        assert!(!truncated, "small payload should not be truncated");
        assert_eq!(out, value);
        assert!(size > 0 && size <= 1024);
    }

    /// MCP-39: when the payload exceeds the budget, `budget_value` pops
    /// trailing array elements until it fits and reports `truncated=true`.
    /// After truncation the envelope key is dropped if the array is empty
    /// (so consumers never see `{records: []}` after a hard-truncation).
    #[test]
    fn budget_value_truncates_trailing_array() {
        use serde_json::json;
        // Build a payload where each item is ~1KB so 5 items > 2KB budget.
        let big_item = "x".repeat(1024);
        let value = json!({
            "records": [
                {"k": big_item.clone()},
                {"k": big_item.clone()},
                {"k": big_item.clone()},
                {"k": big_item.clone()},
                {"k": big_item.clone()},
            ]
        });
        let (out, truncated, size) = budget_value(&value, 2 * 1024);
        assert!(truncated, "payload > budget must be flagged truncated");
        assert!(
            size <= 2 * 1024,
            "truncated payload must fit, got {size} bytes"
        );
        // The remaining array should be smaller than the original 5.
        let arr_len = out
            .get("records")
            .and_then(Value::as_array)
            .map(|a| a.len())
            .unwrap_or(0);
        assert!(arr_len < 5, "truncation should reduce array, got {arr_len}");
    }

    /// MCP-39: when the value has no top-level array, the helper returns it
    /// intact (no synthetic truncation policy). Callers wrapping a list-shaped
    /// response are responsible for shaping the value with an array key.
    #[test]
    fn budget_value_no_array_passthrough() {
        use serde_json::json;
        let value = json!({"count": 1, "namespaces": ["a", "b", "c"]});
        // The helper looks at the *first* top-level array, which here is
        // "namespaces". Use a tiny budget so it truncates "namespaces".
        let (out, truncated, _) = budget_value(&value, 20);
        assert!(
            truncated,
            "tiny budget should force truncation of first array"
        );
        let arr_len = out
            .get("namespaces")
            .and_then(Value::as_array)
            .map(|a| a.len())
            .unwrap_or(0);
        assert!(arr_len < 3, "should pop some entries, got {arr_len}");
    }

    /// MCP-39: `text_content_hits_with_budget` keeps the `content[0].text`
    /// payload as a raw hits array (back-compat) while the budget metadata
    /// rides on `structuredContent` only.
    #[test]
    fn text_content_hits_with_budget_keeps_text_as_array() {
        let hits = serde_json::json!([
            {"record": {"key": "a"}, "score": 0.9},
            {"record": {"key": "b"}, "score": 0.5},
        ]);
        let envelope = text_content_hits_with_budget(&hits, 10 * 1024);
        let text = envelope["content"][0]["text"].as_str().unwrap();
        // Text payload must parse as a JSON array (preserved shape).
        let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(
            parsed.is_array(),
            "text must stay a JSON array, got {parsed}"
        );
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        // structuredContent carries the new envelope.
        assert_eq!(
            envelope["structuredContent"]["hits"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(envelope["structuredContent"]["truncated"], false);
        assert!(
            envelope["structuredContent"]["byte_count"]
                .as_u64()
                .unwrap()
                > 0
        );
    }

    /// MCP-39: when the hits array exceeds the byte budget, the helper
    /// trims trailing entries and flips `truncated=true`. The text payload
    /// reflects the trimmed array.
    #[test]
    fn text_content_hits_with_budget_trims_when_oversize() {
        let big = "x".repeat(512);
        let hits = serde_json::json!([
            {"record": {"key": "a", "payload": big.clone()}},
            {"record": {"key": "b", "payload": big.clone()}},
            {"record": {"key": "c", "payload": big.clone()}},
        ]);
        let envelope = text_content_hits_with_budget(&hits, 600);
        assert_eq!(envelope["structuredContent"]["truncated"], true);
        let text = envelope["content"][0]["text"].as_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(parsed.is_array());
        assert!(
            parsed.as_array().unwrap().len() < 3,
            "expected some entries to be popped, got {}",
            parsed.as_array().unwrap().len()
        );
    }

    /// MCP-34: trust-boundary guards — `validate_path_segment` rejects any
    /// path separators (it's a single segment), while `validate_safe_path`
    /// allows '/' (real paths have it) but blocks '..' and '\'. Use the
    /// right one for the right field; the wrong call either locks out
    /// legitimate inputs (snapshots with '/') or lets a traversal through
    /// (paths with '..').
    #[test]
    fn validate_path_segment_rejects_all_path_separators() {
        let max_len = 512;
        for bad in [
            "/",                 // absolute root
            "\\",                // windows separator
            "..",                // parent dir
            ".",                 // current dir
            "snap/../etc",       // hidden traversal
            "snap\\..\\windows", // hidden traversal (windows flavor)
        ] {
            let err = validate_path_segment(bad, "name", max_len)
                .expect_err("path-segment input must be rejected");
            assert!(
                err.message.contains("plain identifier"),
                "error must name the path-segment rule, got: {} (input {bad:?})",
                err.message
            );
        }
        // Sanity: a clean path segment still passes.
        validate_path_segment("snap-2026-08-29", "name", max_len)
            .expect("plain identifier should pass");
        // Sanity: namespaces (which legitimately contain '/') keep using the
        // permissive validate_identifier — this regression guard documents
        // the split so the surface doesn't collapse them later.
        validate_identifier("mmd/s1/history", "namespace", 256)
            .expect("namespaces may contain '/'");
    }

    #[test]
    fn validate_safe_path_allows_slash_blocks_traversal() {
        let max_len = 4096;
        // Legitimate paths pass.
        for ok in [
            "./local/file.vdbdump",
            "abs/path/to/file.vdbdump",
            "C:/Users/x/file.bin", // windows-style with / also fine
            "simple.bin",
            "a/b/c/d/e.vdbdump",
        ] {
            validate_safe_path(ok, "path", max_len)
                .unwrap_or_else(|e| panic!("{ok:?} should pass, got error: {}", e.message));
        }
        // '..' segments, '\', and null bytes are rejected.
        for bad in [
            "../escape",
            "a/../../escape",
            "a\\b\\..",
            "has\\backslash",
            "with\0null",
            "/abs/../../etc",
        ] {
            let err =
                validate_safe_path(bad, "path", max_len).expect_err("unsafe path must be rejected");
            // null-byte rejection is delegated to validate_identifier and
            // surfaces its own message; everything else names the safe-path
            // rule. Both are MCP-34 trust-boundary errors.
            assert!(
                err.message.contains("safe filesystem path") || err.message.contains("null byte"),
                "error must name a trust-boundary rule, got: {} (input {bad:?})",
                err.message
            );
        }
    }
}
