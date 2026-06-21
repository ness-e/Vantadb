//! CLI command handlers for VantaDB — extracted for testability.

use clap::CommandFactory;
use console::{Style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use web_time::Instant;

use crate::cli::{Cli, Shell};
use crate::config::VantaConfig;
use crate::error::Result;
use crate::storage::StorageEngine;
use crate::VantaEmbedded;

// ─── Styling Helpers ─────────────────────────────────────────

pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .expect("valid spinner template"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

fn success_style() -> Style {
    Style::new().green().bold()
}

fn error_style() -> Style {
    Style::new().red().bold()
}

fn info_style() -> Style {
    Style::new().cyan()
}

fn warning_style() -> Style {
    Style::new().yellow()
}

fn header_style() -> Style {
    Style::new().white().bold()
}

pub fn print_success(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", success_style().apply_to("✓"), msg));
}

pub fn print_error(msg: &str) {
    let term = Term::stderr();
    let _ = term.write_line(&format!("{} {}", error_style().apply_to("✗"), msg));
}

pub fn print_info(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", info_style().apply_to("ℹ"), msg));
}

pub fn print_warning(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", warning_style().apply_to("⚠"), msg));
}

// ─── Database Operations ─────────────────────────────────────

pub fn open_database(path: &str, read_only: bool) -> Result<StorageEngine> {
    let config = VantaConfig {
        read_only,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config))
}

pub fn open_embedded(path: &str, read_only: bool) -> Result<VantaEmbedded> {
    let config = VantaConfig {
        storage_path: path.to_string(),
        read_only,
        ..Default::default()
    };
    VantaEmbedded::open_with_config(config)
}

/// Compute a deterministic node ID from namespace and key using xxHash64
pub fn memory_node_id(namespace: &str, key: &str) -> u64 {
    use std::hash::Hasher;
    let mut hasher = twox_hash::XxHash64::default();
    hasher.write(namespace.as_bytes());
    hasher.write(b"\0");
    hasher.write(key.as_bytes());
    hasher.finish()
}

pub const FIELD_NAMESPACE: &str = "__vanta_namespace";
pub const FIELD_KEY: &str = "__vanta_key";
pub const FIELD_PAYLOAD: &str = "__vanta_payload";
pub const FIELD_CREATED_AT_MS: &str = "__vanta_created_at_ms";
pub const FIELD_UPDATED_AT_MS: &str = "__vanta_updated_at_ms";
pub const FIELD_VERSION: &str = "__vanta_version";

pub fn cmd_put(
    db_path: &str,
    namespace: &str,
    key: &str,
    payload: &str,
    vector: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let spinner = create_spinner("Opening database...");

    let engine = open_database(db_path, false)?;
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
                return Err(crate::error::VantaError::Execution(format!(
                    "Vector must be comma-separated f32 values: {}",
                    e
                )));
            }
        }
    } else {
        None
    };

    spinner.set_message("Inserting record...");

    // Build the node with memory record fields
    let node_id = memory_node_id(namespace, key);
    let mut node = crate::node::UnifiedNode::new(node_id);

    // Set memory fields
    node.relational.insert(
        FIELD_NAMESPACE.to_string(),
        crate::node::FieldValue::String(namespace.to_string()),
    );
    node.relational.insert(
        FIELD_KEY.to_string(),
        crate::node::FieldValue::String(key.to_string()),
    );
    node.relational.insert(
        FIELD_PAYLOAD.to_string(),
        crate::node::FieldValue::String(payload.to_string()),
    );

    let now_ms = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    node.relational.insert(
        FIELD_CREATED_AT_MS.to_string(),
        crate::node::FieldValue::Int(now_ms as i64),
    );
    node.relational.insert(
        FIELD_UPDATED_AT_MS.to_string(),
        crate::node::FieldValue::Int(now_ms as i64),
    );
    node.relational
        .insert(FIELD_VERSION.to_string(), crate::node::FieldValue::Int(1));

    if let Some(vec) = vector_data {
        node.vector = crate::node::VectorRepresentations::Full(vec);
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
    }

    node.flags.set(crate::node::NodeFlags::ACTIVE);

    engine.insert(&node)?;
    engine.flush()?;

    spinner.finish_and_clear();

    if verbose {
        print_info(&format!("Node ID: {}", node_id));
        if let Some(v) = vector {
            print_info(&format!("Vector dimensions: {}", v.split(',').count()));
        }
    }

    print_success(&format!(
        "Record stored: {}:{} ({} bytes)",
        namespace,
        key,
        payload.len()
    ));

    Ok(())
}

pub fn cmd_get(db_path: &str, namespace: &str, key: &str, verbose: bool) -> Result<()> {
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
    spinner.set_message("Searching record...");

    let node_id = memory_node_id(namespace, key);

    match engine.get(node_id)? {
        Some(node) => {
            spinner.finish_and_clear();

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
            if let Some(crate::node::FieldValue::String(payload)) =
                node.relational.get(FIELD_PAYLOAD)
            {
                let _ = term.write_line(&format!(
                    "{} {}",
                    info_style().apply_to("│  Payload:"),
                    payload
                ));
            }

            // Display vector info
            match &node.vector {
                crate::node::VectorRepresentations::Full(v) => {
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
            if let Some(crate::node::FieldValue::Int(created)) =
                node.relational.get(FIELD_CREATED_AT_MS)
            {
                let _ = term.write_line(&format!(
                    "{} {}",
                    info_style().apply_to("│  Created:"),
                    created
                ));
            }

            if let Some(crate::node::FieldValue::Int(version)) = node.relational.get(FIELD_VERSION)
            {
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
            Err(crate::error::VantaError::NodeNotFound(node_id))
        }
    }
}

pub fn cmd_list(db_path: &str, namespace: &str, limit: usize, verbose: bool) -> Result<()> {
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
    spinner.set_message("Scanning namespace...");

    let nodes = engine.scan_nodes()?;

    // Filter by namespace
    let filtered: Vec<_> = nodes
        .into_iter()
        .filter(|n| {
            n.relational
                .get(FIELD_NAMESPACE)
                .map(|v| matches!(v, crate::node::FieldValue::String(s) if s == namespace))
                .unwrap_or(false)
        })
        .take(limit)
        .collect();

    spinner.finish_and_clear();

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
                crate::node::FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "?".to_string());

        let payload = node
            .relational
            .get(FIELD_PAYLOAD)
            .and_then(|v| match v {
                crate::node::FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "".to_string());

        let preview = if payload.len() > 38 {
            format!("{}...", &payload[..35])
        } else {
            payload
        };

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

pub fn cmd_rebuild_index(db_path: &str, _verbose: bool) -> Result<()> {
    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔═══════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║           VantaDB Index Rebuild                           ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚═══════════════════════════════════════════════════════════╝")
    ));
    let _ = term.write_line("");

    let spinner = create_spinner("Opening database...");
    let start = Instant::now();

    let db = open_embedded(db_path, false)?;
    spinner.finish_and_clear();
    print_success("Database opened");

    let rebuild_spinner = create_spinner("Rebuilding all indexes...");
    let report = db.rebuild_index()?;
    rebuild_spinner.finish_and_clear();

    if report.success {
        print_success("All indexes rebuilt successfully");
    } else {
        print_error("Index rebuild failed");
    }

    let total_duration = start.elapsed();

    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╭─────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        success_style().apply_to("│  ✓ Index rebuild completed successfully │")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├─────────────────────────────────────────┤")
    ));
    let _ = term.write_line(&format!(
        "│  Total time:         {:<18} │",
        format!("{:?}", total_duration)
    ));
    let _ = term.write_line(&format!(
        "│  Scanned nodes:      {:<18} │",
        report.scanned_nodes
    ));
    let _ = term.write_line(&format!(
        "│  Indexed vectors:    {:<18} │",
        report.indexed_vectors
    ));
    let _ = term.write_line(&format!(
        "│  Skipped tombstones: {:<18} │",
        report.skipped_tombstones
    ));
    let _ = term.write_line(&format!(
        "│  Rebuild duration:   {:<18} │",
        format!("{} ms", report.duration_ms)
    ));
    let _ = term.write_line(&format!(
        "│  Derived rebuild:    {:<18} │",
        format!("{} ms", report.derived_rebuild_ms)
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}

pub fn cmd_audit_index(
    db_path: &str,
    namespace: Option<&str>,
    json_output: bool,
    deep: bool,
) -> Result<()> {
    let spinner = create_spinner("Opening database...");

    let db = open_embedded(db_path, true)?;
    spinner.set_message("Running audit...");

    let report = if deep {
        db.audit_text_index_deep(namespace)?
    } else {
        db.audit_text_index(namespace)?
    };

    spinner.finish_and_clear();

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|err| {
                crate::error::VantaError::Execution(format!("failed to encode audit report: {err}"))
            })?
        );
    } else {
        let term = Term::stdout();
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to("╭─────────────────────────────────────────╮")
        ));
        let _ = term.write_line("│  Index Status Check                     │");
        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to("├─────────────────────────────────────────┤")
        ));
        let _ = term.write_line(&format!("│  Status:           {:<18} │", report.status));
        let _ = term.write_line(&format!(
            "│  Passed:           {:<18} │",
            if report.passed { "Yes" } else { "No" }
        ));
        let _ = term.write_line(&format!(
            "│  Scanned nodes:    {:<18} │",
            report.records_scanned
        ));
        let _ = term.write_line(&format!(
            "│  Expected entries: {:<18} │",
            report.expected_entries
        ));
        let _ = term.write_line(&format!(
            "│  Actual entries:   {:<18} │",
            report.actual_entries
        ));
        let _ = term.write_line(&format!("│  Mismatches:       {:<18} │", report.mismatches));
        let _ = term.write_line(&format!(
            "│  Missing entries:  {:<18} │",
            report.missing_entries
        ));
        let _ = term.write_line(&format!(
            "│  Unexpected:       {:<18} │",
            report.unexpected_entries
        ));
        let _ = term.write_line(&format!(
            "│  Value mismatches: {:<18} │",
            report.value_mismatches
        ));
        let _ = term.write_line(&format!(
            "│  Unreadable:       {:<18} │",
            report.unreadable_entries
        ));
        let _ = term.write_line(&format!(
            "│  State status:     {:<18} │",
            report.state_status
        ));
        if let Some(ns) = namespace {
            let _ = term.write_line(&format!("│  Namespace filter: {:<18} │", ns));
        }
        let _ = term.write_line(&format!(
            "│  Deep audit:       {:<18} │",
            if deep { "Yes" } else { "No" }
        ));

        if report.deep_audit {
            let _ = term.write_line(&format!(
                "{}",
                header_style().apply_to("├─────────────────────────────────────────┤")
            ));
            let _ = term.write_line(&format!("│  TF errors:        {:<18} │", report.tf_errors));
            let _ = term.write_line(&format!(
                "│  Position errors:  {:<18} │",
                report.position_errors
            ));
            let _ = term.write_line(&format!("│  DF errors:        {:<18} │", report.df_errors));
            let _ = term.write_line(&format!(
                "│  Doc len errors:   {:<18} │",
                report.doc_len_errors
            ));
            let _ = term.write_line(&format!(
                "│  Logical corrupts: {:<18} │",
                report.logical_corruptions
            ));
        }

        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to("╰─────────────────────────────────────────╯")
        ));

        if !report.passed {
            print_warning(&format!(
                "Text index drift detected. Run: vanta-cli repair-text-index --db {} or vanta-cli rebuild-index --db {}",
                db_path, db_path
            ));
        }
    }

    if !report.passed {
        std::process::exit(3);
    }

    Ok(())
}

pub fn cmd_repair_text_index(db_path: &str) -> Result<()> {
    let spinner = create_spinner("Opening database...");

    let db = open_embedded(db_path, false)?;
    spinner.set_message("Repairing text index...");

    let report = db.repair_text_index()?;
    spinner.finish_and_clear();

    if report.success {
        println!(
            "repair success=true record_count={} posting_entries={} doc_stats_entries={} term_stats_entries={} namespace_stats_entries={} duration_ms={}",
            report.record_count,
            report.posting_entries,
            report.doc_stats_entries,
            report.term_stats_entries,
            report.namespace_stats_entries,
            report.duration_ms
        );
    } else {
        print_error("Repair failed");
    }

    Ok(())
}

pub fn cmd_export(db_path: &str, namespace: Option<&str>, output_path: &str) -> Result<()> {
    let spinner = create_spinner("Opening database...");
    let embedded = open_embedded(db_path, true)?;
    spinner.finish_and_clear();

    let report = if let Some(ns) = namespace {
        embedded.export_namespace(output_path, ns)?
    } else {
        embedded.export_all(output_path)?
    };

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╭─────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        success_style().apply_to("│  ✓ Export Completed Successfully        │")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├─────────────────────────────────────────┤")
    ));
    let _ = term.write_line(&format!(
        "│  Records exported:   {:<18} │",
        report.records_exported
    ));
    let _ = term.write_line(&format!("│  Output file:        {:<18} │", output_path));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}

pub fn cmd_import(db_path: &str, input_path: &str, _verbose: bool) -> Result<()> {
    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔═══════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║           VantaDB Memory Import                           ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚═══════════════════════════════════════════════════════════╝")
    ));
    let _ = term.write_line("");

    if !std::path::Path::new(input_path).exists() {
        print_error(&format!("Input file not found: {}", input_path));
        return Err(crate::error::VantaError::Execution(format!(
            "Input file not found: {}",
            input_path
        )));
    }

    let spinner = create_spinner("Opening database...");
    let embedded = open_embedded(db_path, false)?;
    spinner.finish_and_clear();
    print_success("Database opened");

    let report = embedded.import_file(input_path)?;
    embedded.flush()?;

    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╭─────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        success_style().apply_to("│  ✓ Import Completed                     │")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├─────────────────────────────────────────┤")
    ));
    let _ = term.write_line(&format!("│  Inserted:           {:<18} │", report.inserted));

    if report.updated > 0 {
        let _ = term.write_line(&format!("│  Updated:            {:<18} │", report.updated));
    }

    if report.errors > 0 {
        let _ = term.write_line(&format!(
            "{}",
            error_style().apply_to(format!("│  Errors:             {:<18} │", report.errors))
        ));
    }

    let _ = term.write_line(&format!(
        "│  Duration:           {:<18} │",
        format!("{:?}", std::time::Duration::from_millis(report.duration_ms))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}

pub fn cmd_query(db_path: &str, query: &str, limit: usize, verbose: bool) -> Result<()> {
    let spinner = create_spinner("Opening database...");

    let engine = open_database(db_path, true)?;
    spinner.set_message("Executing query...");

    let start = Instant::now();

    // Parse and execute query using the executor
    let executor = crate::executor::Executor::new(&engine);
    let result = executor.execute_hybrid(query)?;

    let duration = start.elapsed();
    spinner.finish_and_clear();

    let term = Term::stdout();
    let _ = term.write_line("");

    match result {
        crate::executor::ExecutionResult::Read(nodes) => {
            let display_nodes: Vec<_> = nodes.into_iter().take(limit).collect();

            if display_nodes.is_empty() {
                print_warning("Query returned no results");
                return Ok(());
            }

            let _ = term.write_line(&format!(
                "{}",
                header_style().apply_to(format!(
                    "Query Results ({} records, {:?})",
                    display_nodes.len(),
                    duration
                ))
            ));
            let _ = term.write_line(&format!(
                "{}",
                header_style()
                    .apply_to("┌──────────┬────────────────────────────────────────────────┐")
            ));
            let _ = term.write_line(&format!(
                "{}",
                header_style()
                    .apply_to("│ ID       │ Fields                                         │")
            ));
            let _ = term.write_line(&format!(
                "{}",
                header_style()
                    .apply_to("├──────────┼────────────────────────────────────────────────┤")
            ));

            for node in &display_nodes {
                let fields_preview: String = node
                    .relational
                    .iter()
                    .take(3)
                    .map(|(k, v)| format!("{}={:?}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");

                let preview = if fields_preview.len() > 46 {
                    format!("{}...", &fields_preview[..43])
                } else {
                    fields_preview
                };

                let _ = term.write_line(&format!("│ {:<8} │ {:<46} │", node.id, preview));
            }

            let _ = term.write_line(&format!(
                "{}",
                header_style()
                    .apply_to("└──────────┴────────────────────────────────────────────────┘")
            ));

            if verbose {
                print_info("Query parsed successfully");
            }
        }
        crate::executor::ExecutionResult::Write {
            affected_nodes,
            message,
            node_id,
        } => {
            print_success(&format!("{} ({} nodes affected)", message, affected_nodes));
            if let Some(id) = node_id {
                print_info(&format!("Node ID: {}", id));
            }
        }
        crate::executor::ExecutionResult::StaleContext(node_id) => {
            print_warning(&format!("Stale context for node {}", node_id));
        }
    }

    Ok(())
}

pub fn cmd_status(db_path: &str, verbose: bool) -> Result<()> {
    let path = std::path::Path::new(db_path);
    let term = Term::stdout();

    if !path.exists() {
        let metrics = crate::metrics::operational_metrics_snapshot();
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╔═══════════════════════════════════════════════════════════╗")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("║               VantaDB Status Dashboard                    ║")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╠═══════════════════════════════════════════════════════════╣")
        ));
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to("║  📁 Database Information                                  ║")
        ));
        let _ = term.write_line(&format!("║     Path:           {:<38} ║", db_path));
        let _ = term.write_line(&format!(
            "║     Backend:        {:<38} ║",
            "Uninitialized (directory not found)"
        ));
        let _ = term.write_line(&format!("║     Read-only:      {:<38} ║", "Yes (Fallback)"));
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to("║  💾 Storage Statistics                                    ║")
        ));
        let _ = term.write_line(&format!("║     HNSW Nodes:     {:<38} ║", "0 (Empty)"));
        let _ = term.write_line(&format!("║     Cache entries:  {:<38} ║", "0"));
        let _ = term.write_line(&format!("║     Logical size:   {:<38} ║", "0 MB"));
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to("║  ⚡ Performance Metrics                                   ║")
        ));
        let _ = term.write_line(&format!(
            "║     Startup time:   {:<38} ║",
            format!("{} ms", metrics.startup_ms)
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╚═══════════════════════════════════════════════════════════╝")
        ));
        print_warning(&format!(
            "Database is not initialized. Run a mutation (like `put`) to create it at '{}'",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");

    let engine = open_database(db_path, true)?;
    let stats = engine.get_memory_stats();
    let metrics = crate::metrics::operational_metrics_snapshot();

    spinner.finish_and_clear();

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔═══════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║               VantaDB Status Dashboard                    ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╠═══════════════════════════════════════════════════════════╣")
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  📁 Database Information                                  ║")
    ));
    let _ = term.write_line(&format!("║     Path:           {:<38} ║", db_path));
    let _ = term.write_line(&format!(
        "║     Backend:        {:<38} ║",
        format!("{:?}", engine.backend_kind())
    ));
    let _ = term.write_line(&format!(
        "║     Read-only:      {:<38} ║",
        if engine.read_only { "Yes" } else { "No" }
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  💾 Storage Statistics                                    ║")
    ));
    let _ = term.write_line(&format!("║     HNSW Nodes:     {:<38} ║", stats.node_count));
    let _ = term.write_line(&format!(
        "║     Cache entries:  {:<38} ║",
        stats.cache_entries
    ));
    let _ = term.write_line(&format!(
        "║     Logical size:   {:<38} ║",
        format!("{} MB", stats.logical_bytes / (1024 * 1024))
    ));
    if let Some(rss) = stats.physical_rss {
        let _ = term.write_line(&format!(
            "║     Physical RSS:   {:<38} ║",
            format!("{} MB", rss / (1024 * 1024))
        ));
    }
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  ⚡ Performance Metrics                                   ║")
    ));
    let _ = term.write_line(&format!(
        "║     Startup time:   {:<38} ║",
        format!("{} ms", metrics.startup_ms)
    ));
    let _ = term.write_line(&format!(
        "║     WAL replay:     {:<38} ║",
        format!(
            "{} ms ({} records)",
            metrics.wal_replay_ms, metrics.wal_records_replayed
        )
    ));
    let _ = term.write_line(&format!(
        "║     ANN rebuild:    {:<38} ║",
        format!("{} ms", metrics.ann_rebuild_ms)
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  📦 Data Operations                                       ║")
    ));
    let _ = term.write_line(&format!(
        "║     Records exported:{:<37} ║",
        metrics.records_exported
    ));
    let _ = term.write_line(&format!(
        "║     Records imported:{:<37} ║",
        metrics.records_imported
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚═══════════════════════════════════════════════════════════╝")
    ));

    if verbose {
        let _ = term.write_line("");
        print_info("Verbose mode: Extended metrics available via VantaOperationalMetrics API");
    }

    Ok(())
}

pub fn cmd_server(
    db_path: &str,
    http: bool,
    mcp: bool,
    port: Option<u16>,
    host: Option<String>,
    _verbose: bool,
) -> Result<()> {
    let mcp_mode = mcp && !http;

    if mcp_mode {
        return cmd_server_mcp(db_path, port, host);
    }

    #[cfg(feature = "server")]
    {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            crate::error::VantaError::Execution(format!("Failed to start tokio runtime: {e}"))
        })?;

        rt.block_on(cmd_server_http(db_path, port, host))
    }

    #[cfg(not(feature = "server"))]
    {
        Err(crate::error::VantaError::Execution(
            "HTTP server requires the 'server' feature. Rebuild with: cargo build --features server"
                .to_string(),
        ))
    }
}

#[cfg(feature = "server")]
async fn cmd_server_http(db_path: &str, port: Option<u16>, host: Option<String>) -> Result<()> {
    let config = VantaConfig {
        storage_path: db_path.to_string(),
        port: port.unwrap_or(8080),
        host: host.unwrap_or_else(|| "127.0.0.1".to_string()),
        ..Default::default()
    };

    crate::cli_server::run(config).await
}

fn cmd_server_mcp(db_path: &str, port: Option<u16>, host: Option<String>) -> Result<()> {
    use crate::error::VantaError;

    let binary_name = "vantadb-server";
    let exe_name = if cfg!(windows) {
        format!("{}.exe", binary_name)
    } else {
        binary_name.to_string()
    };

    let mut cmd = std::process::Command::new(&exe_name);

    cmd.env("VANTA_DB", db_path);
    if let Some(p) = port {
        cmd.env("VANTADB_PORT", p.to_string());
    }
    if let Some(ref h) = host {
        cmd.env("VANTADB_HOST", h);
    }

    cmd.arg("--mcp");

    cmd.stdin(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => {
            if let Ok(mut current_exe) = std::env::current_exe() {
                current_exe.set_file_name(&exe_name);
                if current_exe.exists() {
                    let mut fallback_cmd = std::process::Command::new(&current_exe);
                    fallback_cmd.env("VANTA_DB", db_path);
                    if let Some(p) = port {
                        fallback_cmd.env("VANTADB_PORT", p.to_string());
                    }
                    if let Some(ref h) = host {
                        fallback_cmd.env("VANTADB_HOST", h);
                    }
                    fallback_cmd.arg("--mcp");
                    fallback_cmd.stdin(std::process::Stdio::inherit());
                    fallback_cmd.stdout(std::process::Stdio::inherit());
                    fallback_cmd.stderr(std::process::Stdio::inherit());

                    match fallback_cmd.spawn() {
                        Ok(c) => c,
                        Err(e) => {
                            return Err(VantaError::Execution(format!(
                                "Failed to start vantadb-server from current directory: {e}"
                            )));
                        }
                    }
                } else {
                    return Err(VantaError::Execution(format!(
                        "vantadb-server binary not found in PATH or in the same directory as the CLI (expected: {})",
                        current_exe.display()
                    )));
                }
            } else {
                return Err(VantaError::Execution(
                    "vantadb-server binary not found in PATH and failed to retrieve current executable path".to_string()
                ));
            }
        }
        Err(e) => {
            return Err(VantaError::Execution(format!(
                "Failed to start vantadb-server process: {e}"
            )));
        }
    };

    let status = child.wait().map_err(|e| {
        VantaError::Execution(format!("Error waiting for vantadb-server process: {e}"))
    })?;

    if !status.success() {
        if let Some(code) = status.code() {
            std::process::exit(code);
        } else {
            std::process::exit(1);
        }
    }

    Ok(())
}

pub fn cmd_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let shell: clap_complete::Shell = shell.into();
    clap_complete::generate(shell, &mut cmd, "vanta-cli", &mut std::io::stdout());
}

pub fn cmd_search(db_path: &str, namespace: &str, query: &str, limit: usize) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let db = open_embedded(db_path, true)?;
    spinner.set_message("Searching...");

    let request = crate::sdk::VantaMemorySearchRequest {
        namespace: namespace.to_string(),
        query_vector: vec![],
        filters: crate::sdk::VantaMemoryMetadata::new(),
        text_query: Some(query.to_string()),
        top_k: limit,
        distance_metric: crate::node::DistanceMetric::Cosine,
        explain: false,
    };

    let hits = db.search(request)?;
    spinner.finish_and_clear();

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style()
            .apply_to("╭──────────────────────────────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(format!(
            "│  Search results for \"{}\" in namespace \"{}\" ({}{}) │",
            query,
            namespace,
            hits.len(),
            if hits.len() < limit && !hits.is_empty() {
                " max"
            } else {
                ""
            }
        ))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style()
            .apply_to("├──────────────────────────────────────────────────────────────────┤")
    ));

    if hits.is_empty() {
        let _ = term.write_line(&format!(
            "{}",
            warning_style().apply_to("│  No results found                                   │")
        ));
    } else {
        for (i, hit) in hits.iter().enumerate() {
            let _ = term.write_line(&format!(
                "{}",
                info_style().apply_to(format!(
                    "│  #{:<3} │ Score: {:<8} │ {}:{}",
                    i + 1,
                    format!("{:.6}", hit.score),
                    hit.record.namespace,
                    hit.record.key
                ))
            ));
            let _ = term.write_line(&format!(
                "{}",
                info_style().apply_to(format!(
                    "│       │ Payload:  {}",
                    &hit.record.payload[..hit.record.payload.len().min(80)]
                ))
            ));
            if i < hits.len() - 1 {
                let _ = term.write_line(&format!(
                    "{}",
                    info_style().apply_to("│       │           │")
                ));
            }
        }
    }

    let _ = term.write_line(&format!(
        "{}",
        header_style()
            .apply_to("╰──────────────────────────────────────────────────────────────────╯")
    ));

    Ok(())
}

pub fn cmd_delete(db_path: &str, namespace: &str, key: &str, verbose: bool) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
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

pub fn cmd_namespace_list(db_path: &str) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let db = open_embedded(db_path, true)?;
    spinner.set_message("Listing namespaces...");
    let namespaces = db.list_namespaces()?;
    spinner.finish_and_clear();

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╭─────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(format!(
            "│  Namespaces ({})                          │",
            namespaces.len()
        ))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├─────────────────────────────────────────┤")
    ));

    if namespaces.is_empty() {
        let _ = term.write_line(&format!(
            "{}",
            warning_style().apply_to("│  No namespaces found                     │")
        ));
    } else {
        for ns in &namespaces {
            let _ = term.write_line(&format!(
                "{}",
                info_style().apply_to(format!("│  • {}", ns))
            ));
        }
    }

    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}

pub fn cmd_namespace_info(db_path: &str, namespace: &str) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let db = open_embedded(db_path, true)?;
    spinner.set_message("Scanning namespace...");

    let options = crate::sdk::VantaMemoryListOptions {
        filters: crate::sdk::VantaMemoryMetadata::new(),
        limit: usize::MAX,
        cursor: None,
    };
    let page = db.list(namespace, options)?;
    spinner.finish_and_clear();

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╭─────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(format!("│  Namespace: {}", namespace))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├─────────────────────────────────────────┤")
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to(format!("│  Records: {}", page.records.len()))
    ));

    if page.records.is_empty() {
        let _ = term.write_line(&format!(
            "{}",
            warning_style().apply_to("│  (empty)                                   │")
        ));
    } else {
        let total_payload: usize = page.records.iter().map(|r| r.payload.len()).sum();
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to(format!("│  Total payload: {} bytes", total_payload))
        ));
    }

    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}
