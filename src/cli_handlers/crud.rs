//! CRUD command handlers — put, get, list, delete.

use console::Term;

use crate::cli_handlers::{
    create_spinner, field_value_to_json, memory_node_id, open_database, open_embedded, print_error,
    print_info, print_json, print_success, print_warning, stdout_is_term, truncate_for_term,
    FIELD_CREATED_AT_MS, FIELD_KEY, FIELD_NAMESPACE, FIELD_PAYLOAD, FIELD_VERSION,
};
use crate::error::{ChainedError, Result};
use crate::node::{FieldValue, VectorRepresentations};

#[tracing::instrument]
/// Store a key-value record with optional vector embedding and metadata
pub fn cmd_put(
    db_path: &str,
    namespace: &str,
    key: &str,
    payload: &str,
    vector: Option<&str>,
    metadata: Option<&str>,
    verbose: bool,
    json_output: bool,
) -> Result<()> {
    let spinner = create_spinner("Opening database...");

    // API-07: writes go through the SDK (`Embedded::put`) instead of building
    // a `UnifiedNode` and calling `engine.insert` directly. The SDK keeps the
    // derived/text/sparse indexes current at write time (and validates
    // namespace/key/metadata), which is what lets the read path open
    // read-only (shared lock, no `ensure_indexes_current` reconciliation).
    let db = open_embedded(db_path, false)?;
    spinner.set_message("Preparing record...");

    // Parse optional vector
    let vector_data = if let Some(vec_str) = vector {
        let parsed: std::result::Result<Vec<f32>, _> = vec_str
            .split(',')
            .map(|s| s.trim().parse::<f32>().map_err(|e| e.to_string()))
            .collect();
        match parsed {
            Ok(v) => Some(v),
            Err(e) => {
                spinner.finish_and_clear();
                print_error(&format!("Invalid vector format: {}", e));
                return Err(crate::error::Error::Cli(ChainedError::msg(format!(
                    "Vector must be comma-separated f32 values: {}",
                    e
                ))));
            }
        }
    } else {
        None
    };

    // Optional metadata: JSON object -> user fields. The SDK's
    // `validate_metadata` rejects keys under the internal `__vanta_` prefix
    // (and NUL bytes), so the CLI cannot collide with internal fields or fake
    // system timestamps.
    let mut metadata_map = crate::sdk::MemoryMetadata::new();
    if let Some(meta_str) = metadata {
        let parsed: serde_json::Value = serde_json::from_str(meta_str).map_err(|e| {
            spinner.finish_and_clear();
            print_error(&format!("Invalid metadata JSON: {e}"));
            crate::error::Error::Cli(ChainedError::msg(format!(
                "Metadata must be a JSON object, e.g. '{{\"k\":\"v\"}}': {e}"
            )))
        })?;
        let obj = parsed.as_object().ok_or_else(|| {
            spinner.finish_and_clear();
            print_error("Metadata must be a JSON object at the root level");
            crate::error::Error::Cli(ChainedError::msg(
                "Metadata must be a JSON object at the root level, e.g. '{\"k\":\"v\"}'",
            ))
        })?;
        for (field, value) in obj {
            let vanta_value = json_to_vanta_value(value).map_err(|e| {
                spinner.finish_and_clear();
                print_error(&format!("Invalid metadata value for '{field}': {e}"));
                e
            })?;
            metadata_map.insert(field.clone(), vanta_value);
        }
    }

    spinner.set_message("Inserting record...");
    let record = db.put(crate::sdk::MemoryInput {
        namespace: namespace.to_string(),
        key: key.to_string(),
        payload: payload.to_string(),
        metadata: metadata_map,
        vector: vector_data,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })?;
    // ERR-050b: the write is WAL-buffered until flushed; a later read-only
    // open does not replay the WAL, so flush before returning.
    db.flush()?;
    spinner.finish_and_clear();

    if verbose && !json_output {
        print_info(&format!("Node ID: {}", record.node_id));
        if let Some(v) = vector {
            print_info(&format!("Vector dimensions: {}", v.split(',').count()));
        }
    }

    if json_output {
        print_json(&serde_json::json!({
            "status": "stored",
            "namespace": record.namespace,
            "key": record.key,
            "node_id": record.node_id.to_string(),
            "version": record.version,
            "payload_bytes": record.payload.len(),
            "has_vector": record.vector.is_some(),
        }))?;
    } else {
        print_success(&format!(
            "Record stored: {}:{} ({} bytes)",
            namespace,
            key,
            payload.len()
        ));
    }

    Ok(())
}

/// True when the error means the database directory exists but was never
/// initialised (no `.vanta.lock` / no schema yet). Read-only opens cannot
/// create those files, so read commands treat this like "empty".
pub(crate) fn is_uninitialized_db(e: &crate::error::Error) -> bool {
    matches!(
        e,
        crate::error::Error::NotFound { kind, .. }
            if kind == "database_path" || kind == "lock_file"
    )
}

#[tracing::instrument]
/// Retrieve and display a record by namespace and key
pub fn cmd_get(
    db_path: &str,
    namespace: &str,
    key: &str,
    verbose: bool,
    json_output: bool,
) -> Result<()> {
    use crate::cli_handlers::fmt::{header_style, info_style};

    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            return print_json(&serde_json::Value::Null);
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let engine = open_database(db_path, true)?;
    spinner.set_message("Searching record...");

    let node_id = memory_node_id(namespace, key);

    match engine.get(node_id)? {
        Some(node) => {
            spinner.finish_and_clear();

            if json_output {
                let mut fields = serde_json::Map::new();
                for (field_key, value) in node.relational.iter() {
                    fields.insert(field_key.clone(), field_value_to_json(value));
                }
                let vector_dimensions = match &node.vector {
                    VectorRepresentations::Full(v) => Some(v.len()),
                    _ => None,
                };
                print_json(&serde_json::json!({
                    "namespace": namespace,
                    "key": key,
                    "node_id": node_id.to_string(),
                    "payload": node.relational.get(FIELD_PAYLOAD).and_then(FieldValue::as_str),
                    "vector_dimensions": vector_dimensions,
                    "fields": fields,
                }))?;
                return Ok(());
            }

            let term = Term::stdout();
            let _ = term.write_line("");
            let _ = term.write_line(&format!(
                "{}",
                header_style().apply_to("╭─────────────────────────────────────────╮")
            ));
            let _ = term.write_line(&format!(
                "{}",
                header_style().apply_to(format!("│  Record: {}:{}", namespace, key))
            ));
            let _ = term.write_line(&format!(
                "{}",
                header_style().apply_to("├─────────────────────────────────────────┤")
            ));

            // Display payload
            if let Some(FieldValue::String(payload)) = node.relational.get(FIELD_PAYLOAD) {
                let _ = term.write_line(&format!(
                    "{} {}",
                    info_style().apply_to("│  Payload:"),
                    payload
                ));
            }

            // Display vector info
            match &node.vector {
                VectorRepresentations::Full(v) => {
                    let _ = term.write_line(&format!(
                        "{} {} dimensions",
                        info_style().apply_to("│  Vector:"),
                        v.len()
                    ));
                }
                _ => {
                    let _ =
                        term.write_line(&format!("{} None", info_style().apply_to("│  Vector:")));
                }
            }

            // Display metadata
            if let Some(FieldValue::Int(created)) = node.relational.get(FIELD_CREATED_AT_MS) {
                let _ = term.write_line(&format!(
                    "{} {}",
                    info_style().apply_to("│  Created:"),
                    created
                ));
            }

            if let Some(FieldValue::Int(version)) = node.relational.get(FIELD_VERSION) {
                let _ = term.write_line(&format!(
                    "{} {}",
                    info_style().apply_to("│  Version:"),
                    version
                ));
            }

            let _ = term.write_line(&format!(
                "{}",
                header_style().apply_to("╰─────────────────────────────────────────╯")
            ));

            if verbose {
                print_info(&format!("Node ID: {}", node_id));
                print_info(&format!("Tier: {:?}", node.tier));
                print_info(&format!("Hits: {}", node.hits));
            }

            Ok(())
        }
        None => {
            spinner.finish_and_clear();
            print_error(&format!("Record not found: {}:{}", namespace, key));
            Err(crate::error::Error::NodeNotFound(node_id))
        }
    }
}

#[tracing::instrument]
/// List records in a namespace with an optional limit
pub fn cmd_list(
    db_path: &str,
    namespace: &str,
    limit: usize,
    verbose: bool,
    json_output: bool,
) -> Result<()> {
    use crate::cli_handlers::fmt::header_style;

    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            return print_json(&serde_json::Value::Array(Vec::new()));
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");

    let engine = open_database(db_path, true)?;
    spinner.set_message("Scanning namespace...");

    let nodes = engine.scan_nodes()?;

    // Filter by namespace
    let filtered: Vec<_> = nodes
        .into_iter()
        .filter(|n| {
            n.relational
                .get(FIELD_NAMESPACE)
                .map(|v| matches!(v, FieldValue::String(s) if s == namespace))
                .unwrap_or(false)
        })
        .take(limit)
        .collect();

    spinner.finish_and_clear();

    // `--json` output is always complete (full payload, no previews).
    if json_output {
        let records: Vec<serde_json::Value> = filtered
            .iter()
            .map(|node| {
                let mut fields = serde_json::Map::new();
                for (field_key, value) in node.relational.iter() {
                    fields.insert(field_key.clone(), field_value_to_json(value));
                }
                serde_json::json!({
                    "key": node.relational.get(FIELD_KEY).and_then(FieldValue::as_str),
                    "payload": node.relational.get(FIELD_PAYLOAD).and_then(FieldValue::as_str),
                    "node_id": node.id.to_string(),
                    "fields": fields,
                })
            })
            .collect();
        return print_json(&serde_json::Value::Array(records));
    }

    if filtered.is_empty() {
        print_warning(&format!("No records found in namespace '{}'", namespace));
        return Ok(());
    }

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(format!(
            "Records in '{}' (showing {})",
            namespace,
            filtered.len()
        ))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("┌────────────────────┬────────────────────────────────────────┐")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("│ Key                │ Payload Preview                        │")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├────────────────────┼────────────────────────────────────────┤")
    ));

    for node in &filtered {
        let key = node
            .relational
            .get(FIELD_KEY)
            .and_then(|v| match v {
                FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "?".to_string());

        let payload = node
            .relational
            .get(FIELD_PAYLOAD)
            .and_then(|v| match v {
                FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "".to_string());

        let preview = truncate_for_term(&payload, 35, stdout_is_term());

        let _ = term.write_line(&format!("│ {:<18} │ {:<38} │", key, preview));
    }

    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("└────────────────────┴────────────────────────────────────────┘")
    ));

    if verbose {
        print_info(&format!("Total nodes scanned: {}", filtered.len()));
    }

    Ok(())
}

#[tracing::instrument]
/// Delete a record by namespace and key
pub fn cmd_delete(
    db_path: &str,
    namespace: &str,
    key: &str,
    verbose: bool,
    json_output: bool,
) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            return print_json(&serde_json::json!({
                "deleted": false,
                "namespace": namespace,
                "key": key,
                "node_id": memory_node_id(namespace, key).to_string(),
            }));
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let db = open_embedded(db_path, false)?;
    spinner.set_message("Deleting record...");

    let deleted = db.delete(namespace, key)?;
    spinner.finish_and_clear();

    if json_output {
        print_json(&serde_json::json!({
            "deleted": deleted,
            "namespace": namespace,
            "key": key,
            "node_id": memory_node_id(namespace, key).to_string(),
        }))?;
        return Ok(());
    }

    if deleted {
        print_success(&format!("Record deleted: {}:{}", namespace, key));
        if verbose {
            let node_id = memory_node_id(namespace, key);
            print_info(&format!("Node ID: {}", node_id));
        }
    } else {
        print_warning(&format!("Record not found: {}:{}", namespace, key));
    }

    Ok(())
}

/// Delete a record and emit a purge certificate (VER-02).
///
/// The certificate inventories every purge surface (store, JSON-shredded
/// metadata, vector index/store, derived/text/sparse indexes, version history,
/// WAL tombstone) with per-surface evidence, an integrity hash and the VER-01
/// chain reference. `--json` emits `{deleted, certificate}`; otherwise the
/// human summary plus the pretty certificate JSON. With `out`, the raw pretty
/// certificate is written to that file by the CLI itself (UTF-8; preferred
/// over shell redirection on Windows, which can mangle non-ASCII).
#[tracing::instrument]
pub fn cmd_delete_certified(
    db_path: &str,
    namespace: &str,
    key: &str,
    out: Option<&str>,
    verbose: bool,
    json_output: bool,
) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            return print_json(&serde_json::json!({
                "deleted": false,
                "namespace": namespace,
                "key": key,
                "node_id": memory_node_id(namespace, key).to_string(),
                "certificate": serde_json::Value::Null,
            }));
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let db = open_embedded(db_path, false)?;
    spinner.set_message("Deleting record + scanning purge surfaces...");

    let certificate = db.delete_certified(namespace, key)?;
    // Persist the purge (WAL + mmap / HNSW state) before returning: the
    // certificate claims must survive a read-only reopen — `certificate
    // verify` opens read-only and does not replay the WAL.
    db.close()?;
    spinner.finish_and_clear();

    let deleted = certificate.status != "not_found";
    let pretty =
        serde_json::to_string_pretty(&certificate).map_err(crate::error::Error::serialization)?;

    if let Some(out_path) = out {
        std::fs::write(out_path, &pretty)?;
        if json_output {
            print_json(&serde_json::json!({
                "deleted": deleted,
                "namespace": namespace,
                "key": key,
                "node_id": &certificate.node_id,
                "certificate_file": out_path,
            }))?;
        } else {
            if deleted {
                print_success(&format!("Record deleted: {}:{}", namespace, key));
            } else {
                print_warning(&format!("Record not found: {}:{}", namespace, key));
            }
            print_success(&format!("Certificate written to {out_path}"));
        }
        if verbose {
            print_info(&format!("Certificate status: {}", certificate.status));
        }
        return Ok(());
    }

    if json_output {
        print_json(&serde_json::json!({
            "deleted": deleted,
            "namespace": namespace,
            "key": key,
            "node_id": &certificate.node_id,
            "certificate": &certificate,
        }))?;
        return Ok(());
    }

    if deleted {
        print_success(&format!("Record deleted: {}:{}", namespace, key));
    } else {
        print_warning(&format!("Record not found: {}:{}", namespace, key));
    }
    if verbose {
        print_info(&format!("Certificate status: {}", certificate.status));
    }
    println!("{pretty}");
    Ok(())
}

/// Verify a stored purge certificate (VER-02) against the live database:
/// integrity hash + re-scan of the re-checkable surfaces (store, shred,
/// vector index, version history). Returns the CLI exit code
/// (0 = valid; 1 = edited/corrupted certificate or residues reappeared).
#[tracing::instrument]
pub fn cmd_certificate_verify(db_path: &str, file: &str, json_output: bool) -> Result<i32> {
    let content = std::fs::read_to_string(file)?;
    // Accept both a raw certificate and the `delete --attest --json` envelope
    // (`{"deleted":..,"certificate":{..}}`) so the CLI output round-trips.
    let certificate_json = match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(value) => match value.get("certificate") {
            Some(cert) if !cert.is_null() => cert.to_string(),
            _ => content.clone(),
        },
        // Not valid JSON: hand it to the SDK parser so the error is typed.
        Err(_) => content.clone(),
    };
    let db = open_embedded(db_path, true)?;
    match db.verify_purge_certificate(&certificate_json) {
        Ok(verification) => {
            if json_output {
                print_json(&serde_json::json!({
                    "command": "certificate_verify",
                    "ok": true,
                    "db": db_path,
                    "file": file,
                    "verification": &verification,
                }))?;
            } else {
                print_success(&format!(
                    "Certificate valid — status '{}', integrity ok, residues now: {}, surfaces re-checked: {}",
                    verification.status,
                    verification.residues_now,
                    verification.rechecked_surfaces.join(", ")
                ));
            }
            Ok(0)
        }
        Err(e) => {
            if json_output {
                print_json(&serde_json::json!({
                    "command": "certificate_verify",
                    "ok": false,
                    "db": db_path,
                    "file": file,
                    "error": e.to_string(),
                }))?;
            } else {
                print_warning(&format!("Certificate INVALID: {e}"));
            }
            Ok(1)
        }
    }
}

/// Parse a JSON filter string (MongoDB-like) into a `MemoryFilter`.
///
/// Accepts BOTH formats (AUD-048, unified semantics with the MCP channel):
/// - Operator objects: `{"field": {"$eq": "value"}, "score": {"$gte": 50}}`
/// - Flat values, interpreted as implicit `$eq` (same as the MCP flat form):
///   `{"field": "value", "score": 50}`
pub(crate) fn parse_filter_json(
    filter_str: &str,
) -> crate::error::Result<crate::sdk::MemoryFilter> {
    use crate::sdk::{FilterOp, MemoryFilterItem};

    let root: serde_json::Value = serde_json::from_str(filter_str)
        .map_err(|e| crate::error::Error::InvalidInput(format!("Invalid filter JSON: {e}")))?;

    let obj = root.as_object().ok_or_else(|| {
        crate::error::Error::InvalidInput("Filter must be a JSON object at the root level".into())
    })?;

    let mut items = Vec::new();
    for (field, spec) in obj {
        let Some(spec_obj) = spec.as_object() else {
            // AUD-048: flat value → implicit equality, matching the MCP
            // channel's published flat semantics `{"field": value}`.
            let value = json_to_vanta_value(spec)?;
            items.push(MemoryFilterItem {
                field: field.clone(),
                op: FilterOp::Eq,
                value,
            });
            continue;
        };

        for (op_str, val_json) in spec_obj {
            let op = match op_str.as_str() {
                "$eq" => FilterOp::Eq,
                "$neq" => FilterOp::Neq,
                "$gt" => FilterOp::Gt,
                "$gte" => FilterOp::Gte,
                "$lt" => FilterOp::Lt,
                "$lte" => FilterOp::Lte,
                other => {
                    return Err(crate::error::Error::InvalidInput(format!(
                    "Unknown filter operator '{other}'. Supported: $eq, $neq, $gt, $gte, $lt, $lte"
                )))
                }
            };
            let value = json_to_vanta_value(val_json)?;
            items.push(MemoryFilterItem {
                field: field.clone(),
                op,
                value,
            });
        }
    }
    Ok(items)
}

fn json_to_vanta_value(v: &serde_json::Value) -> crate::error::Result<crate::sdk::Value> {
    use crate::sdk::Value;
    match v {
        serde_json::Value::String(s) => Ok(Value::String(s.clone())),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(crate::error::Error::InvalidInput(format!(
                    "Cannot convert number {n} to Value"
                )))
            }
        }
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        other => Err(crate::error::Error::InvalidInput(format!(
            "Unsupported JSON value type: {other}. Use string, number, or bool."
        ))),
    }
}

#[tracing::instrument]
/// Delete all records in a namespace matching a JSON metadata filter
pub fn cmd_delete_by_filter(
    db_path: &str,
    namespace: &str,
    filter_str: &str,
    verbose: bool,
    json_output: bool,
) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            return print_json(&serde_json::json!({
                "deleted": 0,
                "namespace": namespace,
                "filter": filter_str,
            }));
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let filter = parse_filter_json(filter_str).map_err(|e| {
        print_error(&format!("Filter parse error: {e}"));
        e
    })?;

    let spinner = create_spinner("Opening database...");
    let db = open_embedded(db_path, false)?;
    spinner.set_message("Deleting matching records...");

    let deleted = db.delete_by_filter(namespace, filter)?;
    spinner.finish_and_clear();

    if json_output {
        print_json(&serde_json::json!({
            "deleted": deleted,
            "namespace": namespace,
            "filter": filter_str,
        }))?;
        return Ok(());
    }

    print_success(&format!(
        "Deleted {} record{} from namespace '{}'",
        deleted,
        if deleted == 1 { "" } else { "s" },
        namespace
    ));

    if verbose {
        print_info(&format!("Namespace: {namespace}"));
        print_info(&format!("Records deleted: {deleted}"));
    }

    Ok(())
}

#[tracing::instrument]
/// Count records in a namespace, optionally filtered by metadata
pub fn cmd_count(
    db_path: &str,
    namespace: &str,
    filter_str: Option<&str>,
    json_output: bool,
    verbose: bool,
) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        // API-07: a missing database is an error for `count` — exit≠0 so
        // scripts can tell "no database" apart from a real count of 0.
        print_error(&format!(
            "Database directory does not exist at '{}'",
            db_path
        ));
        return Err(crate::error::Error::Cli(ChainedError::msg(format!(
            "Database directory does not exist at '{}'",
            db_path
        ))));
    }

    let filter = if let Some(fs) = filter_str {
        Some(parse_filter_json(fs).map_err(|e| {
            print_error(&format!("Filter parse error: {e}"));
            e
        })?)
    } else {
        None
    };

    let spinner = create_spinner("Opening database...");
    // API-07: count is a read → read-only open (shared lock, no index
    // reconciliation). Writes keep the derived/text/sparse indexes current via
    // the SDK (see cmd_put), so no read-write open is needed here. A database
    // written by an older CLI needs one `rebuild-index` after upgrading
    // (documented in docs/user/operations/CONFIGURATION.md).
    let db = match open_embedded(db_path, true) {
        Ok(db) => db,
        Err(e) => {
            spinner.finish_and_clear();
            print_error(&format!("Cannot open database at '{}': {e}", db_path));
            return Err(e);
        }
    };
    spinner.set_message("Counting records...");

    let count = db.count(namespace, filter)?;
    spinner.finish_and_clear();

    if json_output {
        print_json(&serde_json::json!({
            "namespace": namespace,
            "count": count,
            "filter": filter_str,
        }))?;
        return Ok(());
    }

    let term = console::Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "Namespace '{}': {} record{}",
        namespace,
        count,
        if count == 1 { "" } else { "s" }
    ));

    if verbose && filter_str.is_some() {
        print_info(&format!("Filter applied: {}", filter_str.unwrap_or("")));
    }

    Ok(())
}
