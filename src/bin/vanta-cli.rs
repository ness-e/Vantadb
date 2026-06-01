//! VantaDB CLI - Modern command-line interface for VantaDB
//!
//! This CLI provides a premium developer experience with:
//! - Structured commands via clap
//! - Progress spinners via indicatif
//! - Rich terminal output via console
//! - Shell completions via clap_complete

use clap::{CommandFactory, Parser};
use console::{Style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

use vantadb::cli::{Cli, Commands, Shell};
use vantadb::config::VantaConfig;
use vantadb::error::Result;
use vantadb::storage::StorageEngine;
use vantadb::VantaEmbedded;

// ─── Styling Helpers ─────────────────────────────────────────

fn create_spinner(message: &str) -> ProgressBar {
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

fn create_progress_bar(len: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("valid progress template")
            .progress_chars("█▓▒░"),
    );
    pb.set_message(message.to_string());
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

fn print_success(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", success_style().apply_to("✓"), msg));
}

fn print_error(msg: &str) {
    let term = Term::stderr();
    let _ = term.write_line(&format!("{} {}", error_style().apply_to("✗"), msg));
}

fn print_info(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", info_style().apply_to("ℹ"), msg));
}

fn print_warning(msg: &str) {
    let term = Term::stdout();
    let _ = term.write_line(&format!("{} {}", warning_style().apply_to("⚠"), msg));
}

// ─── Database Operations ─────────────────────────────────────

fn open_database(path: &str, read_only: bool) -> Result<StorageEngine> {
    let config = VantaConfig {
        read_only,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config))
}

fn open_embedded(path: &str, read_only: bool) -> Result<VantaEmbedded> {
    let config = VantaConfig {
        storage_path: path.to_string(),
        read_only,
        ..Default::default()
    };
    VantaEmbedded::open_with_config(config)
}

/// Compute a deterministic node ID from namespace and key using xxHash64
fn memory_node_id(namespace: &str, key: &str) -> u64 {
    use std::hash::Hasher;
    let mut hasher = twox_hash::XxHash64::default();
    hasher.write(namespace.as_bytes());
    hasher.write(b"\0");
    hasher.write(key.as_bytes());
    hasher.finish()
}

const FIELD_NAMESPACE: &str = "__vanta_namespace";
const FIELD_KEY: &str = "__vanta_key";
const FIELD_PAYLOAD: &str = "__vanta_payload";
const FIELD_CREATED_AT_MS: &str = "__vanta_created_at_ms";
const FIELD_UPDATED_AT_MS: &str = "__vanta_updated_at_ms";
const FIELD_VERSION: &str = "__vanta_version";

fn cmd_put(
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
                return Err(vantadb::error::VantaError::Execution(format!(
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
    let mut node = vantadb::node::UnifiedNode::new(node_id);

    // Set memory fields
    node.relational.insert(
        FIELD_NAMESPACE.to_string(),
        vantadb::node::FieldValue::String(namespace.to_string()),
    );
    node.relational.insert(
        FIELD_KEY.to_string(),
        vantadb::node::FieldValue::String(key.to_string()),
    );
    node.relational.insert(
        FIELD_PAYLOAD.to_string(),
        vantadb::node::FieldValue::String(payload.to_string()),
    );

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    node.relational.insert(
        FIELD_CREATED_AT_MS.to_string(),
        vantadb::node::FieldValue::Int(now_ms as i64),
    );
    node.relational.insert(
        FIELD_UPDATED_AT_MS.to_string(),
        vantadb::node::FieldValue::Int(now_ms as i64),
    );
    node.relational
        .insert(FIELD_VERSION.to_string(), vantadb::node::FieldValue::Int(1));

    if let Some(vec) = vector_data {
        node.vector = vantadb::node::VectorRepresentations::Full(vec);
        node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
    }

    node.flags.set(vantadb::node::NodeFlags::ACTIVE);

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

fn cmd_get(db_path: &str, namespace: &str, key: &str, verbose: bool) -> Result<()> {
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
            if let Some(vantadb::node::FieldValue::String(payload)) =
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
                vantadb::node::VectorRepresentations::Full(v) => {
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
            if let Some(vantadb::node::FieldValue::Int(created)) =
                node.relational.get(FIELD_CREATED_AT_MS)
            {
                let _ = term.write_line(&format!(
                    "{} {}",
                    info_style().apply_to("│  Created:"),
                    created
                ));
            }

            if let Some(vantadb::node::FieldValue::Int(version)) =
                node.relational.get(FIELD_VERSION)
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
            Err(vantadb::error::VantaError::NodeNotFound(node_id))
        }
    }
}

fn cmd_list(db_path: &str, namespace: &str, limit: usize, verbose: bool) -> Result<()> {
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
                .map(|v| matches!(v, vantadb::node::FieldValue::String(s) if s == namespace))
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
                vantadb::node::FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "?".to_string());

        let payload = node
            .relational
            .get(FIELD_PAYLOAD)
            .and_then(|v| match v {
                vantadb::node::FieldValue::String(s) => Some(s.clone()),
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

fn cmd_rebuild_index(db_path: &str, _verbose: bool) -> Result<()> {
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

fn cmd_audit_index(
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
                vantadb::error::VantaError::Execution(format!(
                    "failed to encode audit report: {err}"
                ))
            })?
        );
    } else {
        let term = Term::stdout();
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to("╭─────────────────────────────────────────╮")
        ));
        let _ = term.write_line(&"│  Index Status Check                     │".to_string());
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

fn cmd_repair_text_index(db_path: &str) -> Result<()> {
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

fn cmd_export(db_path: &str, namespace: Option<&str>, output_path: &str) -> Result<()> {
    let spinner = create_spinner("Opening database...");

    let engine = open_database(db_path, true)?;
    spinner.set_message("Scanning records...");

    let nodes = engine.scan_nodes()?;

    // Filter by namespace if specified
    let filtered: Vec<_> = if let Some(ns) = namespace {
        nodes
            .into_iter()
            .filter(|n| {
                n.relational
                    .get(FIELD_NAMESPACE)
                    .map(|v| matches!(v, vantadb::node::FieldValue::String(s) if s == ns))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        nodes
    };

    spinner.set_message("Exporting records...");

    // Simple JSON export format
    let mut output = String::from("[\n");
    for (i, node) in filtered.iter().enumerate() {
        if i > 0 {
            output.push_str(",\n");
        }
        let namespace = node
            .relational
            .get(FIELD_NAMESPACE)
            .and_then(|v| match v {
                vantadb::node::FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let key = node
            .relational
            .get(FIELD_KEY)
            .and_then(|v| match v {
                vantadb::node::FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let payload = node
            .relational
            .get(FIELD_PAYLOAD)
            .and_then(|v| match v {
                vantadb::node::FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_default();

        output.push_str(&format!(
            r#"  {{"namespace": "{}", "key": "{}", "payload": "{}", "node_id": {}}}"#,
            namespace.replace('"', "\\\""),
            key.replace('"', "\\\""),
            payload.replace('"', "\\\"").replace('\n', "\\n"),
            node.id
        ));
    }
    output.push_str("\n]");

    std::fs::write(output_path, output).map_err(vantadb::error::VantaError::IoError)?;

    spinner.finish_and_clear();

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
    let _ = term.write_line(&format!("│  Records exported:   {:<18} │", filtered.len()));
    let _ = term.write_line(&format!("│  Output file:        {:<18} │", output_path));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}

fn cmd_import(db_path: &str, input_path: &str, _verbose: bool) -> Result<()> {
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

    // Validate input file exists
    if !std::path::Path::new(input_path).exists() {
        print_error(&format!("Input file not found: {}", input_path));
        return Err(vantadb::error::VantaError::Execution(format!(
            "Input file not found: {}",
            input_path
        )));
    }

    let spinner = create_spinner("Opening database...");
    let engine = open_database(db_path, false)?;
    spinner.finish_and_clear();
    print_success("Database opened");

    // Read and parse JSON
    let count_spinner = create_spinner("Reading import file...");
    let content =
        std::fs::read_to_string(input_path).map_err(vantadb::error::VantaError::IoError)?;
    count_spinner.finish_and_clear();

    // Simple JSON parsing (expects array of objects with namespace, key, payload)
    // This is a simplified parser for the CLI export format
    let start = Instant::now();
    let mut inserted = 0u64;
    let mut errors = 0u64;

    // Very basic JSON parsing - look for patterns in the export format
    let pb = create_progress_bar(content.lines().count() as u64, "Importing...");

    for line in content.lines() {
        pb.inc(1);
        let line = line.trim();
        if line.starts_with('{') {
            // Try to extract namespace, key, payload from the line
            // This is a simplified parser
            if let (Some(ns), Some(key), Some(payload)) = (
                extract_json_field(line, "namespace"),
                extract_json_field(line, "key"),
                extract_json_field(line, "payload"),
            ) {
                let node_id = memory_node_id(&ns, &key);
                let mut node = vantadb::node::UnifiedNode::new(node_id);

                node.relational.insert(
                    FIELD_NAMESPACE.to_string(),
                    vantadb::node::FieldValue::String(ns),
                );
                node.relational.insert(
                    FIELD_KEY.to_string(),
                    vantadb::node::FieldValue::String(key),
                );
                node.relational.insert(
                    FIELD_PAYLOAD.to_string(),
                    vantadb::node::FieldValue::String(
                        payload.replace("\\n", "\n").replace("\\\"", "\""),
                    ),
                );

                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                node.relational.insert(
                    FIELD_CREATED_AT_MS.to_string(),
                    vantadb::node::FieldValue::Int(now_ms as i64),
                );
                node.flags.set(vantadb::node::NodeFlags::ACTIVE);

                match engine.insert(&node) {
                    Ok(_) => inserted += 1,
                    Err(_) => errors += 1,
                }
            } else {
                errors += 1;
            }
        }
    }

    pb.finish_and_clear();

    let flush_spinner = create_spinner("Flushing changes...");
    engine.flush()?;
    flush_spinner.finish_and_clear();

    let duration = start.elapsed();

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
    let _ = term.write_line(&format!("│  Inserted:           {:<18} │", inserted));

    if errors > 0 {
        let _ = term.write_line(&format!(
            "{}",
            error_style().apply_to(format!("│  Errors:             {:<18} │", errors))
        ));
    }

    let _ = term.write_line(&format!(
        "│  Duration:           {:<18} │",
        format!("{:?}", duration)
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}

/// Extract a string field value from a simple JSON object line
fn extract_json_field(line: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\": \"", field);
    if let Some(start) = line.find(&pattern) {
        let value_start = start + pattern.len();
        let rest = &line[value_start..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn cmd_query(db_path: &str, query: &str, limit: usize, verbose: bool) -> Result<()> {
    let spinner = create_spinner("Opening database...");

    let engine = open_database(db_path, true)?;
    spinner.set_message("Executing query...");

    let start = Instant::now();

    // Parse and execute query using the executor
    let executor = vantadb::executor::Executor::new(&engine);
    let result = executor.execute_hybrid(query)?;

    let duration = start.elapsed();
    spinner.finish_and_clear();

    let term = Term::stdout();
    let _ = term.write_line("");

    match result {
        vantadb::executor::ExecutionResult::Read(nodes) => {
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
                print_info(&"Query parsed successfully".to_string());
            }
        }
        vantadb::executor::ExecutionResult::Write {
            affected_nodes,
            message,
            node_id,
        } => {
            print_success(&format!("{} ({} nodes affected)", message, affected_nodes));
            if let Some(id) = node_id {
                print_info(&format!("Node ID: {}", id));
            }
        }
        vantadb::executor::ExecutionResult::StaleContext(node_id) => {
            print_warning(&format!("Stale context for node {}", node_id));
        }
    }

    Ok(())
}

fn cmd_status(db_path: &str, verbose: bool) -> Result<()> {
    let path = std::path::Path::new(db_path);
    let term = Term::stdout();

    if !path.exists() {
        let metrics = vantadb::metrics::operational_metrics_snapshot();
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

        // Database Info
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

        // Storage Stats
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to("║  💾 Storage Statistics                                    ║")
        ));
        let _ = term.write_line(&format!("║     HNSW Nodes:     {:<38} ║", "0 (Empty)"));
        let _ = term.write_line(&format!("║     Cache entries:  {:<38} ║", "0"));
        let _ = term.write_line(&format!("║     Logical size:   {:<38} ║", "0 MB"));

        // Performance Metrics
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
    let metrics = vantadb::metrics::operational_metrics_snapshot();

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

    // Database Info
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

    // Storage Stats
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

    // Performance Metrics
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

    // Import/Export Stats
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

fn cmd_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let shell: clap_complete::Shell = shell.into();
    clap_complete::generate(shell, &mut cmd, "vanta-cli", &mut std::io::stdout());
}

// ─── Main Entry Point ────────────────────────────────────────

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Initialize tracing for verbose mode
    if args.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    match args.command {
        Commands::Put {
            namespace,
            key,
            payload,
            vector,
        } => cmd_put(
            &args.db,
            &namespace,
            &key,
            &payload,
            vector.as_deref(),
            args.verbose,
        )?,

        Commands::Get { namespace, key } => cmd_get(&args.db, &namespace, &key, args.verbose)?,

        Commands::List { namespace, limit } => cmd_list(&args.db, &namespace, limit, args.verbose)?,

        Commands::RebuildIndex => cmd_rebuild_index(&args.db, args.verbose)?,

        Commands::AuditIndex {
            namespace,
            json,
            deep,
        } => cmd_audit_index(&args.db, namespace.as_deref(), json, deep)?,

        Commands::RepairTextIndex => cmd_repair_text_index(&args.db)?,

        Commands::Export { namespace, out } => cmd_export(&args.db, namespace.as_deref(), &out)?,

        Commands::Import { input } => cmd_import(&args.db, &input, args.verbose)?,

        Commands::Query { query, limit } => cmd_query(&args.db, &query, limit, args.verbose)?,

        Commands::Status => cmd_status(&args.db, args.verbose)?,

        Commands::Completions { shell } => cmd_completions(shell),
    }

    Ok(())
}
