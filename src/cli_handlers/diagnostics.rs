//! Diagnostics command handlers — doctor, inspect, stats.

use console::Term;
use std::collections::HashSet;
use web_time::{SystemTime, UNIX_EPOCH};

use crate::cli_handlers::fmt::{header_style, info_style};
use crate::cli_handlers::{
    create_spinner, human_readable_size, memory_node_id, open_database, print_info, print_success,
    print_warning, FIELD_EXPIRES_AT_MS, FIELD_NAMESPACE, FIELD_PAYLOAD,
};
use crate::error::{ChainedError, Result, VantaError};
use crate::node::{FieldValue, NodeFlags, VectorRepresentations};

/// A safe, non-destructive repair that `doctor --fix` can apply.
#[derive(Debug, PartialEq, Eq)]
struct PendingRepair {
    /// Human-readable description (printed as `Would fix:` / `Fixed:`).
    description: String,
    /// Filesystem path to create (always a directory).
    path: std::path::PathBuf,
}

/// Compute the safe repairs pending for `db_path` without mutating anything.
///
/// Scope is deliberately minimal and additive only:
///
/// - missing database directory → create it
/// - missing `data/` subdirectory → create it
///
/// Everything else (lock files, WAL, schema, permissions, user data) is
/// report-only and never touched — see stop conditions in GOV-TK1.
fn pending_safe_repairs(db_path: &str) -> Vec<PendingRepair> {
    let base = std::path::Path::new(db_path);
    if !base.exists() {
        let data_dir = base.join("data");
        return vec![
            PendingRepair {
                description: format!("create missing database directory at '{db_path}'"),
                path: base.to_path_buf(),
            },
            PendingRepair {
                description: format!(
                    "create missing data subdirectory at '{}'",
                    data_dir.display()
                ),
                path: data_dir,
            },
        ];
    }
    let data_dir = base.join("data");
    if !data_dir.exists() {
        return vec![PendingRepair {
            description: format!(
                "create missing data subdirectory at '{}'",
                data_dir.display()
            ),
            path: data_dir,
        }];
    }
    Vec::new()
}

/// True when the open error means "empty/uninitialised", not corruption.
///
/// Matches `NotFound` (missing path/lock/data dir) and the missing-schema
/// message. Anything else (bad version, invalid header, busy, IO) is a real
/// problem and must still surface as an error.
fn is_empty_database_state(e: &VantaError) -> bool {
    match e {
        VantaError::NotFound { .. } => true,
        VantaError::SchemaError(msg) => msg.contains("no schema file"),
        _ => false,
    }
}

#[tracing::instrument]
/// Run comprehensive health diagnostics on the database
pub fn cmd_doctor(db_path: &str, fix: bool, force: bool, verbose: bool) -> Result<()> {
    if fix {
        let pending = pending_safe_repairs(db_path);
        if !force {
            // Dry-run by default: list, never mutate.
            if pending.is_empty() {
                print_success("doctor --fix: nothing to fix");
            } else {
                for repair in &pending {
                    print_warning(&format!("Would fix: {}", repair.description));
                }
                print_info("dry-run: re-run with `doctor --fix --force` to apply");
            }
        } else {
            // --force: apply additive-only repairs (create missing dirs).
            if pending.is_empty() {
                print_success("doctor --fix: nothing to fix");
            } else {
                for repair in &pending {
                    std::fs::create_dir_all(&repair.path).map_err(VantaError::IoError)?;
                    print_success(&format!("Fixed: {}", repair.description));
                }
            }
            // A freshly created (empty) database has no schema/lock yet —
            // opening it read-only would fail with NotFound/SchemaError.
            // That is the expected empty state, not an error: exit 0.
            if !pending.is_empty() {
                return Ok(());
            }
        }
        // ponytail: stale `.vanta.lock` / permissions / WAL / user data are
        // intentionally left alone — deleting them risks data loss. Report
        // only; manual review required (GOV-TK1 stop condition).
        if verbose && pending.is_empty() {
            print_info(
                "Left alone (manual review if unhealthy): .vanta.lock, WAL segments, user data",
            );
        }
    }
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database for diagnostics...");
    // In --fix mode an empty/uninitialised database (fresh dirs, no
    // schema/lock yet) is the expected state after repairs — exit 0 with a
    // warning instead of propagating NotFound/SchemaError. Genuine corruption
    // (incompatible version, invalid header) still errors for manual review.
    let engine = match open_database(db_path, true) {
        Ok(engine) => engine,
        Err(e) if fix && is_empty_database_state(&e) => {
            spinner.finish_and_clear();
            print_warning(&format!(
                "Database is empty/uninitialised ({e}); nothing further to diagnose."
            ));
            return Ok(());
        }
        Err(e) => {
            spinner.finish_and_clear();
            return Err(e);
        }
    };
    spinner.set_message("Running diagnostics...");

    let nodes = engine.scan_nodes()?;
    let total_nodes = nodes.len();

    let mut namespaces: Vec<String> = Vec::new();
    let mut total_vectors = 0usize;
    let mut total_expired = 0u64;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    for node in &nodes {
        let ns = node.relational.get(FIELD_NAMESPACE).and_then(|v| match v {
            FieldValue::String(s) => Some(s.clone()),
            _ => None,
        });
        if let Some(ns) = ns {
            if !namespaces.contains(&ns) {
                namespaces.push(ns);
            }
        }

        if node.flags.is_set(NodeFlags::HAS_VECTOR) {
            total_vectors += 1;
        }

        if let Some(FieldValue::Int(exp)) = node.relational.get(FIELD_EXPIRES_AT_MS) {
            if *exp < now_ms {
                total_expired += 1;
            }
        }
    }

    let stats = engine.get_memory_stats();

    spinner.finish_and_clear();

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔══════════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║                VantaDB Health Diagnostics                    ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╠══════════════════════════════════════════════════════════════╣")
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  Overview                                                 ║")
    ));
    let _ = term.write_line(&format!("║     Total nodes:     {:<38} ║", total_nodes));
    let _ = term.write_line(&format!(
        "║     Namespaces:      {:<38} ║",
        namespaces.len()
    ));
    let _ = term.write_line(&format!("║     Vectors stored:  {:<38} ║", total_vectors));
    let _ = term.write_line(&format!("║     Expired records: {:<38} ║", total_expired));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  Storage                                                 ║")
    ));
    let _ = term.write_line(&format!(
        "║     Node count:      {:<38} ║",
        stats.node_count
    ));
    let _ = term.write_line(&format!(
        "║     Logical size:    {:<38} ║",
        human_readable_size(stats.logical_bytes)
    ));
    let _ = term.write_line(&format!(
        "║     Cache entries:   {:<38} ║",
        stats.cache_entries
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚══════════════════════════════════════════════════════════════╝")
    ));

    if verbose {
        let _ = term.write_line("");
        print_info("Namespaces:");
        for ns in &namespaces {
            print_info(&format!("  {}", ns));
        }
    }

    if total_expired > 0 {
        print_warning(&format!(
            "Found {} expired record(s). Consider compacting the database.",
            total_expired
        ));
    }

    Ok(())
}

#[tracing::instrument]
/// Inspect a single record showing all fields, vectors, and metadata
pub fn cmd_inspect(db_path: &str, namespace: &str, key: &str, verbose: bool) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let engine = open_database(db_path, true)?;
    spinner.set_message("Inspecting record...");

    let node_id = memory_node_id(namespace, key);

    match engine.get(node_id)? {
        Some(node) => {
            spinner.finish_and_clear();

            let term = Term::stdout();
            let _ = term.write_line("");
            let _ = term.write_line(&format!(
                "{}",
                header_style()
                    .apply_to("╔══════════════════════════════════════════════════════════╗")
            ));
            let _ = term.write_line(&format!(
                "{}",
                header_style()
                    .apply_to("║                 Record Inspection                         ║")
            ));
            let _ = term.write_line(&format!(
                "{}",
                header_style()
                    .apply_to("╠══════════════════════════════════════════════════════════╣")
            ));
            let _ = term.write_line(&format!("║  Namespace: {:<42} ║", namespace));
            let _ = term.write_line(&format!("║  Key:       {:<42} ║", key));
            let _ = term.write_line(&format!("║  Node ID:   {:<42} ║", node_id));
            let _ = term.write_line(&format!(
                "║  Has vector: {:<41} ║",
                node.flags.is_set(NodeFlags::HAS_VECTOR)
            ));
            let _ = term.write_line(&format!(
                "║  Active:    {:<41} ║",
                node.flags.is_set(NodeFlags::ACTIVE)
            ));
            let _ = term.write_line(&format!(
                "{}",
                header_style()
                    .apply_to("╠══════════════════════════════════════════════════════════╣")
            ));
            let _ = term.write_line(&format!(
                "{}",
                info_style()
                    .apply_to("║  Fields (relational metadata)                           ║")
            ));

            let mut rel_keys: Vec<&String> = node.relational.keys().collect();
            rel_keys.sort();
            for field_key in rel_keys {
                if let Some(val) = node.relational.get(field_key) {
                    let val_str = match val {
                        FieldValue::String(s) => s.clone(),
                        FieldValue::Int(i) => i.to_string(),
                        FieldValue::Float(f) => format!("{:.6}", f),
                        FieldValue::Bool(b) => b.to_string(),
                        _ => format!("{:?}", val),
                    };
                    let line = format!("║  {:<15} = {:<35} ║", field_key, val_str);
                    let _ = term.write_line(&line);
                }
            }

            if node.flags.is_set(NodeFlags::HAS_VECTOR) {
                let _ = term.write_line(&format!(
                    "{}",
                    header_style()
                        .apply_to("╠══════════════════════════════════════════════════════════╣")
                ));
                let _ = term.write_line(&format!(
                    "{}",
                    info_style()
                        .apply_to("║  Vector Data                                              ║")
                ));
                match &node.vector {
                    VectorRepresentations::Full(v) => {
                        let dims = v.len();
                        let preview: String = if dims > 6 {
                            format!(
                                "[{}, {}, {}, {}, {}, ...{} more]",
                                v[0],
                                v[1],
                                v[2],
                                v[3],
                                v[4],
                                dims - 5
                            )
                        } else {
                            format!("{:?}", v)
                        };
                        let _ = term.write_line(&format!("║  Dimensions: {:<39} ║", dims));
                        let truncated = if preview.len() > 38 {
                            format!("{}...", &preview[..35])
                        } else {
                            preview
                        };
                        let _ = term.write_line(&format!("║  Values:     {:<39} ║", truncated));
                    }
                    _ => {
                        let _ = term.write_line(
                            "║  (compressed)                                          ║",
                        );
                    }
                }
            }

            let _ = term.write_line(&format!(
                "{}",
                header_style()
                    .apply_to("╚══════════════════════════════════════════════════════════╝")
            ));

            if verbose {
                let _ = term.write_line("");
                print_info(&format!(
                    "Payload: {}",
                    node.relational
                        .get(FIELD_PAYLOAD)
                        .and_then(|v| match v {
                            FieldValue::String(s) => Some(s.as_str()),
                            _ => None,
                        })
                        .unwrap_or("(none)")
                ));
            }
        }
        None => {
            spinner.finish_and_clear();
            print_warning(&format!(
                "Record not found: {}:{} (node_id: {})",
                namespace, key, node_id
            ));
        }
    }

    Ok(())
}

#[tracing::instrument]
/// Display detailed database statistics in human-readable or JSON format
pub fn cmd_stats(db_path: &str, json_output: bool, verbose: bool) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            println!("null");
            return Ok(());
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let engine = open_database(db_path, true)?;
    spinner.set_message("Collecting statistics...");

    let stats = engine.get_memory_stats();
    let nodes = engine.scan_nodes()?;
    let namespaces: HashSet<String> = nodes
        .iter()
        .filter_map(|n| {
            n.relational.get(FIELD_NAMESPACE).and_then(|v| match v {
                FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
        })
        .collect();

    let total_vector_nodes = nodes
        .iter()
        .filter(|n| n.flags.is_set(NodeFlags::HAS_VECTOR))
        .count();

    let total_payload_bytes: u64 = nodes
        .iter()
        .map(|n| {
            n.relational
                .get(FIELD_PAYLOAD)
                .map(|v| match v {
                    FieldValue::String(s) => s.len() as u64,
                    _ => 0u64,
                })
                .unwrap_or(0u64)
        })
        .sum();

    spinner.finish_and_clear();

    if json_output {
        let result = serde_json::json!({
            "node_count": stats.node_count,
            "cache_entries": stats.cache_entries,
            "logical_bytes": stats.logical_bytes,
            "namespaces": namespaces.iter().cloned().collect::<Vec<_>>(),
            "namespace_count": namespaces.len(),
            "total_records": nodes.len(),
            "total_vector_nodes": total_vector_nodes,
            "total_payload_bytes": total_payload_bytes,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| {
                crate::error::VantaError::CliError(ChainedError::msg(format!(
                    "JSON serialization error: {e}"
                )))
            })?
        );
        return Ok(());
    }

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔══════════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style()
            .apply_to("║                 VantaDB Database Statistics                   ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╠══════════════════════════════════════════════════════════════╣")
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  Overview                                                 ║")
    ));
    let _ = term.write_line(&format!("║     Total records:   {:<38} ║", nodes.len()));
    let _ = term.write_line(&format!(
        "║     Vector records:  {:<38} ║",
        total_vector_nodes
    ));
    let _ = term.write_line(&format!(
        "║     Total payload:   {:<38} ║",
        human_readable_size(total_payload_bytes)
    ));
    let _ = term.write_line(&format!(
        "║     Namespaces:      {:<38} ║",
        namespaces.len()
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  Storage                                                 ║")
    ));
    let _ = term.write_line(&format!(
        "║     HNSW nodes:      {:<38} ║",
        stats.node_count
    ));
    let _ = term.write_line(&format!(
        "║     Cache entries:   {:<38} ║",
        stats.cache_entries
    ));
    let _ = term.write_line(&format!(
        "║     Logical size:    {:<38} ║",
        human_readable_size(stats.logical_bytes)
    ));
    if let Some(rss) = stats.physical_rss {
        let _ = term.write_line(&format!(
            "║     Physical RSS:    {:<38} ║",
            human_readable_size(rss)
        ));
    }
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚══════════════════════════════════════════════════════════════╝")
    ));

    if verbose {
        let _ = term.write_line("");
        print_info("Namespaces:");
        let mut sorted_ns: Vec<&String> = namespaces.iter().collect();
        sorted_ns.sort();
        for ns in sorted_ns {
            print_info(&format!("  {}", ns));
        }
    }

    Ok(())
}
