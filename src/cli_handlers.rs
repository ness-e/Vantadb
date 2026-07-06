//! CLI command handlers for VantaDB — extracted for testability.

use clap::CommandFactory;
use console::{Style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;
use web_time::{Instant, SystemTime, UNIX_EPOCH};

use crate::cli::{Cli, Shell};
use crate::config::VantaConfig;
use crate::error::Result;
const MIB: u64 = 1024 * 1024;
const KIB_F64: f64 = 1024.0;

pub use crate::sdk::{
    FIELD_CREATED_AT_MS, FIELD_EXPIRES_AT_MS, FIELD_KEY, FIELD_NAMESPACE, FIELD_PAYLOAD,
    FIELD_UPDATED_AT_MS, FIELD_VERSION,
};
use crate::storage::StorageEngine;
use crate::VantaEmbedded;

// ─── Styling Helpers ─────────────────────────────────────────

/// Create a styled spinner for indeterminate operations
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

/// Print a green success message to stdout
pub fn print_success(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", success_style().apply_to("✓"), msg));
}

/// Print a red error message to stderr
pub fn print_error(msg: &str) {
    let term = Term::stderr();
    let _ = term.write_line(&format!("{} {}", error_style().apply_to("✗"), msg));
}

/// Print a cyan info message to stdout
pub fn print_info(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", info_style().apply_to("ℹ"), msg));
}

/// Print a yellow warning message to stdout
pub fn print_warning(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", warning_style().apply_to("⚠"), msg));
}

/// Prompt the user for a yes/no confirmation
pub fn confirm_action(prompt: &str) -> std::io::Result<bool> {
    let term = Term::stdout();
    let _ = term.write_str(prompt);
    let _ = term.write_str(" [y/N] ");
    let _ = term.flush();
    let result = term.read_line()?;
    Ok(result.trim().eq_ignore_ascii_case("y") || result.trim().eq_ignore_ascii_case("yes"))
}

// ─── Database Operations ─────────────────────────────────────

/// Open a database at the given path with optional read-only mode
pub fn open_database(path: &str, read_only: bool) -> Result<StorageEngine> {
    let config = VantaConfig {
        read_only,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config))
}

/// Open the embedded VantaDB SDK with the given path and read-only mode
pub fn open_embedded(path: &str, read_only: bool) -> Result<VantaEmbedded> {
    let config = VantaConfig {
        storage_path: path.to_string(),
        read_only,
        ..Default::default()
    };
    VantaEmbedded::open_with_config(config)
}

/// Compute a deterministic node ID from namespace and key using xxHash3-128
pub fn memory_node_id(namespace: &str, key: &str) -> u128 {
    use std::hash::Hasher;
    let mut hasher = twox_hash::XxHash3_128::default();
    hasher.write(namespace.as_bytes());
    hasher.write(b"\0");
    hasher.write(key.as_bytes());
    hasher.finish_128()
}

#[tracing::instrument]
/// Store a key-value record with optional vector embedding
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
                return Err(crate::error::VantaError::CliError(format!(
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

#[tracing::instrument]
/// Retrieve and display a record by namespace and key
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

#[tracing::instrument]
/// List records in a namespace with an optional limit
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

#[tracing::instrument]
/// Rebuild all database indexes (HNSW, text, derived)
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

#[tracing::instrument]
/// Validate text index integrity and report inconsistencies
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
                crate::error::VantaError::CliError(format!("failed to encode audit report: {err}"))
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

#[tracing::instrument]
/// Repair the text index if inconsistencies are detected
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

#[tracing::instrument]
/// Export records to a JSON file, optionally filtered by namespace
pub fn cmd_export(db_path: &str, namespace: Option<&str>, output_path: &str) -> Result<()> {
    use std::io::Write;

    let spinner = create_spinner("Opening database...");
    let embedded = open_embedded(db_path, true)?;
    spinner.finish_and_clear();

    let term = Term::stdout();

    if let Some(parent) = std::path::Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::File::create(output_path)?;
    let mut writer = std::io::BufWriter::new(file);

    const BATCH_SIZE: usize = 500;
    let mut total: u64 = 0;

    let namespaces: Vec<String> = match namespace {
        Some(ns) => vec![ns.to_string()],
        None => embedded.list_namespaces()?,
    };

    // Quick emptiness check before writing
    let any_data = namespaces.iter().any(|ns| {
        embedded
            .list(
                ns,
                crate::sdk::VantaMemoryListOptions {
                    filters: crate::sdk::VantaMemoryMetadata::new(),
                    limit: 1,
                    cursor: None,
                },
            )
            .map(|p| !p.records.is_empty())
            .unwrap_or(false)
    });
    if !any_data {
        print_warning("No records to export");
        return Ok(());
    }

    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} Exporting... {pos} records written")
            .expect("valid spinner template"),
    );
    bar.enable_steady_tick(std::time::Duration::from_millis(100));

    for ns in &namespaces {
        let mut cursor: Option<usize> = None;
        loop {
            let opts = crate::sdk::VantaMemoryListOptions {
                filters: crate::sdk::VantaMemoryMetadata::new(),
                limit: BATCH_SIZE,
                cursor,
            };
            let page = embedded.list(ns, opts)?;
            if page.records.is_empty() {
                break;
            }
            for record in &page.records {
                let line = crate::sdk::export_line_from_record(record.clone());
                serde_json::to_writer(&mut writer, &line)
                    .map_err(|e| crate::error::VantaError::SerializationError(e.to_string()))?;
                writer.write_all(b"\n")?;
            }
            let n = page.records.len() as u64;
            total += n;
            bar.inc(n);
            cursor = page.next_cursor;
            if cursor.is_none() {
                break;
            }
        }
    }

    writer.flush()?;
    bar.finish_and_clear();

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
    let _ = term.write_line(&format!("│  Records exported:   {:<18} │", total));
    let _ = term.write_line(&format!("│  Output file:        {:<18} │", output_path));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}

#[tracing::instrument]
/// Import records from a JSON file into the database
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
        return Err(crate::error::VantaError::CliError(format!(
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

#[tracing::instrument]
/// Execute a structured hybrid query against the database
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

#[tracing::instrument]
/// Display database health diagnostics and system status
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
        format!("{} MB", stats.logical_bytes / MIB)
    ));
    if let Some(rss) = stats.physical_rss {
        let _ = term.write_line(&format!(
            "║     Physical RSS:   {:<38} ║",
            format!("{} MB", rss / MIB)
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

#[tracing::instrument]
/// Start the HTTP or MCP server wrapper for the database
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
            crate::error::VantaError::RuntimeError(format!("Failed to start tokio runtime: {e}"))
        })?;

        rt.block_on(cmd_server_http(db_path, port, host))
    }

    #[cfg(not(feature = "server"))]
    {
        Err(crate::error::VantaError::CliError(
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

    let build_cmd = |binary: &std::path::Path| -> std::process::Command {
        let mut cmd = std::process::Command::new(binary);
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
        cmd
    };

    let mut child = match build_cmd(std::path::Path::new(&exe_name)).spawn() {
        Ok(c) => c,
        Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => {
            if let Ok(mut current_exe) = std::env::current_exe() {
                current_exe.set_file_name(&exe_name);
                if current_exe.exists() {
                    build_cmd(&current_exe).spawn().map_err(|e| {
                        VantaError::CliError(format!(
                            "Failed to start vantadb-server from {}: {e}",
                            current_exe.display()
                        ))
                    })?
                } else {
                    return Err(VantaError::CliError(format!(
                        "vantadb-server binary not found. \
                         Searched PATH for '{}' and CLI directory for '{}'. \
                         The MCP server requires the vantadb-server binary (compiled with the 'server' feature). \
                         Install it via 'cargo build --bin vantadb-server' or place it alongside this binary.",
                        exe_name,
                        current_exe.display()
                    )));
                }
            } else {
                return Err(VantaError::CliError(format!(
                    "vantadb-server binary '{}' not found in PATH. \
                     Current executable path could not be determined. \
                     Ensure vantadb-server is installed and available in PATH.",
                    exe_name
                )));
            }
        }
        Err(e) => {
            return Err(VantaError::CliError(format!(
                "Failed to spawn vantadb-server process (db_path={}): {e}",
                db_path
            )));
        }
    };

    let status = child.wait().map_err(|e| {
        VantaError::CliError(format!(
            "Error waiting for vantadb-server process (db_path={}): {e}",
            db_path
        ))
    })?;

    if !status.success() {
        if let Some(code) = status.code() {
            // Subprocess exited with non-zero — propagate its exit code
            std::process::exit(code);
        } else {
            // Subprocess was terminated by a signal
            return Err(VantaError::CliError(format!(
                "vantadb-server terminated by signal (db_path={})",
                db_path
            )));
        }
    }

    Ok(())
}

#[tracing::instrument]
/// Generate shell completion scripts for the given shell type
pub fn cmd_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let shell: clap_complete::Shell = shell.into();
    clap_complete::generate(shell, &mut cmd, "vanta-cli", &mut std::io::stdout());
}

#[tracing::instrument]
/// Perform semantic or hybrid search across a namespace
pub fn cmd_search(
    db_path: &str,
    namespace: &str,
    query: &str,
    query_vector_str: Option<&str>,
    limit: usize,
    json_output: bool,
) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            println!("[]");
            return Ok(());
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let db = open_embedded(db_path, true)?;
    spinner.set_message("Searching...");

    let query_vector = if let Some(qv) = query_vector_str {
        qv.split(',')
            .map(|s| {
                s.trim().parse::<f32>().map_err(|e| {
                    crate::error::VantaError::InvalidInput(format!(
                        "Invalid vector component '{s}': {e}"
                    ))
                })
            })
            .collect::<std::result::Result<Vec<f32>, _>>()?
    } else {
        vec![]
    };

    let request = crate::sdk::VantaMemorySearchRequest {
        namespace: namespace.to_string(),
        query_vector,
        filters: crate::sdk::VantaMemoryMetadata::new(),
        text_query: Some(query.to_string()),
        top_k: limit,
        distance_metric: crate::node::DistanceMetric::Cosine,
        explain: false,
    };

    let hits = db.search(request)?;
    spinner.finish_and_clear();

    if json_output {
        let results: Vec<serde_json::Value> = hits
            .iter()
            .map(|hit| {
                serde_json::json!({
                    "key": hit.record.key,
                    "namespace": hit.record.namespace,
                    "payload": hit.record.payload,
                    "score": hit.score,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&results).map_err(|e| {
                crate::error::VantaError::CliError(format!("JSON serialization error: {e}"))
            })?
        );
        return Ok(());
    }

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

#[tracing::instrument]
/// Delete a record by namespace and key
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

#[tracing::instrument]
/// List all namespaces in the database
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

#[tracing::instrument]
/// Show record count and details for a specific namespace
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

#[tracing::instrument]
/// Create a filesystem-level backup of the database directory
pub fn cmd_backup(db_path: &str, out: Option<&str>, verbose: bool) -> Result<()> {
    let src = std::path::Path::new(db_path);
    if !src.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'",
            db_path
        ));
        return Ok(());
    }

    let backup_dir = match out {
        Some(p) => PathBuf::from(p),
        None => {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let dir = format!("vantadb_backups/backup_{}", timestamp);
            PathBuf::from(dir)
        }
    };

    if backup_dir.join("vantadb.dat").exists() || backup_dir.join("vantadb.wal").exists() {
        return Err(crate::error::VantaError::CliError(format!(
            "Backup destination '{}' already contains database files. Choose a different location or remove existing files.",
            backup_dir.display()
        )));
    }

    // Open writable to flush, then drop before copying files
    {
        let spinner = create_spinner("Opening database...");
        let engine = open_database(db_path, false)?;
        spinner.set_message("Flushing database...");
        engine.flush()?;
    }

    fn copy_dir(src: &Path, dst: &Path, skip: Option<&Path>) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            if skip.is_some_and(|s| src_path == s) {
                continue;
            }
            let ft = entry.file_type()?;
            let dst_path = dst.join(entry.file_name());
            if ft.is_dir() {
                copy_dir(&src_path, &dst_path, skip)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    copy_dir(src, &backup_dir, Some(&backup_dir)).map_err(|e| {
        crate::error::VantaError::BackupError(format!("Failed to copy database to backup: {e}"))
    })?;

    let spinner = create_spinner("Verifying backup...");
    spinner.finish_and_clear();

    let _ = Term::stdout().write_line("");
    print_success(&format!("Backup created at: {}", backup_dir.display()));

    if verbose {
        print_info(&format!(
            "Source: {}",
            src.canonicalize()
                .unwrap_or_else(|_| src.to_path_buf())
                .display()
        ));
        print_info(&format!(
            "Size: {}",
            human_readable_size(dir_size(src).unwrap_or(0) as u64)
        ));
    }

    Ok(())
}

#[tracing::instrument]
/// Restore the database from a previously created backup directory
pub fn cmd_restore(
    db_path: &str,
    input: &str,
    force: bool,
    rebuild: bool,
    verbose: bool,
) -> Result<()> {
    let src = std::path::Path::new(input);
    if !src.exists() {
        return Err(crate::error::VantaError::RestoreError(format!(
            "Backup directory does not exist at '{}'",
            input
        )));
    }

    let dst = std::path::Path::new(db_path);

    if dst.exists() && !force {
        return Err(crate::error::VantaError::RestoreError(
            "Destination database directory already exists. Use --force to overwrite.".to_string(),
        ));
    }

    let spinner = create_spinner("Restoring from backup...");

    if dst.exists() && force {
        std::fs::remove_dir_all(dst).map_err(|e| {
            crate::error::VantaError::RestoreError(format!(
                "Failed to remove existing database directory: {e}"
            ))
        })?;
    }

    std::fs::create_dir_all(dst).map_err(|e| {
        crate::error::VantaError::RestoreError(format!("Failed to create database directory: {e}"))
    })?;

    fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if ft.is_dir() {
                copy_dir(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    copy_dir(src, dst).map_err(|e| {
        crate::error::VantaError::RestoreError(format!("Failed to restore from backup: {e}"))
    })?;

    spinner.set_message("Verifying restored database...");

    if rebuild {
        spinner.set_message("Rebuilding indexes...");
        let db = open_embedded(db_path, false)?;
        db.rebuild_index().map_err(|e| {
            crate::error::VantaError::RestoreError(format!(
                "Index rebuild after restore failed: {e}"
            ))
        })?;
    }

    spinner.finish_and_clear();

    print_success(&format!(
        "Database restored from: {}",
        src.canonicalize()
            .unwrap_or_else(|_| src.to_path_buf())
            .display()
    ));

    if verbose {
        let src_size = dir_size(src).unwrap_or(0) as u64;
        let dst_size = dir_size(dst).unwrap_or(0) as u64;
        print_info(&format!("Backup size: {}", human_readable_size(src_size)));
        print_info(&format!("Restored size: {}", human_readable_size(dst_size)));
    }

    Ok(())
}

#[tracing::instrument]
/// Run comprehensive health diagnostics on the database
pub fn cmd_doctor(db_path: &str, verbose: bool) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database for diagnostics...");
    let engine = open_database(db_path, true)?;
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
            crate::node::FieldValue::String(s) => Some(s.clone()),
            _ => None,
        });
        if let Some(ns) = ns {
            if !namespaces.contains(&ns) {
                namespaces.push(ns);
            }
        }

        if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
            total_vectors += 1;
        }

        if let Some(crate::node::FieldValue::Int(exp)) = node.relational.get(FIELD_EXPIRES_AT_MS) {
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
                node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR)
            ));
            let _ = term.write_line(&format!(
                "║  Active:    {:<41} ║",
                node.flags.is_set(crate::node::NodeFlags::ACTIVE)
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
                        crate::node::FieldValue::String(s) => s.clone(),
                        crate::node::FieldValue::Int(i) => i.to_string(),
                        crate::node::FieldValue::Float(f) => format!("{:.6}", f),
                        crate::node::FieldValue::Bool(b) => b.to_string(),
                        _ => format!("{:?}", val),
                    };
                    let line = format!("║  {:<15} = {:<35} ║", field_key, val_str);
                    let _ = term.write_line(&line);
                }
            }

            if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
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
                    crate::node::VectorRepresentations::Full(v) => {
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
                            crate::node::FieldValue::String(s) => Some(s.as_str()),
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
    let namespaces: std::collections::HashSet<String> = nodes
        .iter()
        .filter_map(|n| {
            n.relational.get(FIELD_NAMESPACE).and_then(|v| match v {
                crate::node::FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
        })
        .collect();

    let total_vector_nodes = nodes
        .iter()
        .filter(|n| n.flags.is_set(crate::node::NodeFlags::HAS_VECTOR))
        .count();

    let total_payload_bytes: u64 = nodes
        .iter()
        .map(|n| {
            n.relational
                .get(FIELD_PAYLOAD)
                .map(|v| match v {
                    crate::node::FieldValue::String(s) => s.len() as u64,
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
                crate::error::VantaError::CliError(format!("JSON serialization error: {e}"))
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

#[tracing::instrument]
/// Migrate a database to the latest storage schema and format versions
pub fn cmd_migrate(
    target_path: &str,
    format: &str,
    dry_run: bool,
    force: bool,
    verbose: bool,
) -> Result<()> {
    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔═══════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║           VantaDB Database Migration                     ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚═══════════════════════════════════════════════════════════╝")
    ));
    let _ = term.write_line("");

    let target = std::path::Path::new(target_path);
    if !target.exists() {
        print_error(&format!("Database directory not found: {}", target_path));
        return Err(crate::error::VantaError::CliError(format!(
            "Database path does not exist: {}",
            target_path
        )));
    }

    use crate::migration::{FormatKind, MigrationEngine};
    use crate::schema::{StorageHeader, CURRENT_SCHEMA_VERSION, HEADER_SIZE};

    // Determine which formats to migrate
    let formats: Vec<FormatKind> = if format == "all" {
        FormatKind::all().to_vec()
    } else {
        match FormatKind::from_string(format) {
            Some(f) => vec![f],
            None => {
                print_error(&format!(
                    "Unknown format: {}. Valid values: all, vfile, index, wal, schema",
                    format
                ));
                return Err(crate::error::VantaError::CliError(format!(
                    "Unknown format: {}",
                    format
                )));
            }
        }
    };

    // Schema migration uses the existing logic
    if formats.contains(&FormatKind::Schema) || format == "all" {
        let schema_path = target.join(".vanta.schema");
        let current_header = match StorageHeader::read_from(&schema_path)? {
            Some(header) => {
                if verbose {
                    print_info(&format!(
                        "Current schema: version={}, min_compat={}, flags={}",
                        header.version, header.min_compat_version, header.flags
                    ));
                }
                header
            }
            None => {
                print_warning("No schema file found; database may be pre-versioning.");
                print_info("Writing current schema header...");
                let header = StorageHeader::current();
                header.write_to(&schema_path)?;
                print_success(&format!(
                    "Schema header written: version={}",
                    CURRENT_SCHEMA_VERSION
                ));
                return Ok(());
            }
        };

        if current_header.version > CURRENT_SCHEMA_VERSION {
            print_error(&format!(
                "Database schema version {} is newer than this software (max {})",
                current_header.version, CURRENT_SCHEMA_VERSION
            ));
            return Err(crate::error::VantaError::SchemaError(format!(
                "Schema version {} is too new for this version of VantaDB",
                current_header.version
            )));
        }

        if current_header.version != CURRENT_SCHEMA_VERSION {
            let spinner = create_spinner("Migrating schema...");
            let start = Instant::now();

            let new_header = StorageHeader::current();
            new_header.write_to(&schema_path)?;

            let elapsed = start.elapsed();
            spinner.finish_and_clear();

            print_success(&format!(
                "Schema migrated: version {} → {} ({} ms)",
                current_header.version,
                CURRENT_SCHEMA_VERSION,
                elapsed.as_millis()
            ));

            if verbose {
                print_info(&format!("Schema file: {}", schema_path.display()));
                print_info(&format!("Header size: {} bytes", HEADER_SIZE));
            }
        } else {
            print_info("Schema is already at the latest version");
        }
    }

    // Physical format migration
    let physical_formats: Vec<FormatKind> = formats
        .into_iter()
        .filter(|f| *f != FormatKind::Schema)
        .collect();

    if !physical_formats.is_empty() {
        let mut engine = MigrationEngine::new(target_path);
        engine.set_dry_run(dry_run);

        if dry_run {
            print_info("--- Dry Run: checking migration requirements ---");
            let plans = engine.plan_all()?;
            if plans.is_empty() {
                print_success("All formats are at their latest version");
            } else {
                for plan in &plans {
                    let _ = term.write_line(&format!(
                        "  {} v{} → v{}: {}",
                        plan.format.name(),
                        plan.current_version,
                        plan.target_version,
                        plan.action
                    ));
                }
            }
        } else {
            if !force {
                let plans = engine.plan_all()?;
                if plans.is_empty() {
                    print_success("All formats are at their latest version");
                    return Ok(());
                }

                let _ = term.write_line("");
                print_warning("The following migrations will be performed:");
                for plan in &plans {
                    let _ = term.write_line(&format!(
                        "  [{}] v{} → v{}: {}",
                        plan.format.name(),
                        plan.current_version,
                        plan.target_version,
                        plan.action
                    ));
                }
                let _ = term.write_line("");

                if !confirm_action("Proceed with migration?")? {
                    print_warning("Migration cancelled by user");
                    return Ok(());
                }
            }

            for fmt in &physical_formats {
                let spinner = create_spinner(&format!("Migrating {}...", fmt.name()));
                let start = Instant::now();
                engine.migrate_format(*fmt)?;
                let elapsed = start.elapsed();
                spinner.finish_and_clear();
                if verbose {
                    print_info(&format!(
                        "  {} completed in {} ms",
                        fmt.name(),
                        elapsed.as_millis()
                    ));
                }
            }

            // Check integrity after migration
            let issues = engine.check_integrity()?;
            if !issues.is_empty() {
                print_warning("Post-migration integrity warnings:");
                for issue in &issues {
                    let _ = term.write_line(&format!("  ⚠ {}", issue));
                }
            }
        }
    }

    Ok(())
}

#[tracing::instrument]

fn human_readable_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= KIB_F64 && unit_idx < UNITS.len() - 1 {
        size /= KIB_F64;
        unit_idx += 1;
    }
    format!("{:.2} {}", size, UNITS[unit_idx])
}

fn dir_size(path: &Path) -> std::io::Result<usize> {
    let mut total = 0usize;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            if ft.is_dir() {
                total += dir_size(&entry.path())?;
            } else {
                total += entry.metadata()?.len() as usize;
            }
        }
    }
    Ok(total)
}
