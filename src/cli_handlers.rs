//! CLI command handlers for VantaDB — extracted for testability.

use clap::CommandFactory;
use console::{Style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use web_time::{Instant, SystemTime, UNIX_EPOCH};

use crate::cli::{Cli, Shell};
use crate::config::VantaConfig;
use crate::error::Result;
pub use crate::sdk::{
    FIELD_CREATED_AT_MS, FIELD_EXPIRES_AT_MS, FIELD_KEY, FIELD_NAMESPACE, FIELD_PAYLOAD,
    FIELD_UPDATED_AT_MS, FIELD_VERSION,
};
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
                        VantaError::Execution(format!(
                            "Failed to start vantadb-server from {}: {e}",
                            current_exe.display()
                        ))
                    })?
                } else {
                    return Err(VantaError::Execution(format!(
                        "vantadb-server binary not found. \
                         Searched PATH for '{}' and CLI directory for '{}'. \
                         The MCP server requires the vantadb-server binary (compiled with the 'server' feature). \
                         Install it via 'cargo build --bin vantadb-server' or place it alongside this binary.",
                        exe_name,
                        current_exe.display()
                    )));
                }
            } else {
                return Err(VantaError::Execution(format!(
                    "vantadb-server binary '{}' not found in PATH. \
                     Current executable path could not be determined. \
                     Ensure vantadb-server is installed and available in PATH.",
                    exe_name
                )));
            }
        }
        Err(e) => {
            return Err(VantaError::Execution(format!(
                "Failed to spawn vantadb-server process (db_path={}): {e}",
                db_path
            )));
        }
    };

    let status = child.wait().map_err(|e| {
        VantaError::Execution(format!(
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
            return Err(VantaError::Execution(format!(
                "vantadb-server terminated by signal (db_path={})",
                db_path
            )));
        }
    }

    Ok(())
}

pub fn cmd_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let shell: clap_complete::Shell = shell.into();
    clap_complete::generate(shell, &mut cmd, "vanta-cli", &mut std::io::stdout());
}

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
                    crate::error::VantaError::Execution(format!(
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
                crate::error::VantaError::Execution(format!("JSON serialization error: {e}"))
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

pub fn cmd_search_similar(
    db_path: &str,
    namespace: &str,
    key: &str,
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

    let engine = open_database(db_path, true)?;
    let db = open_embedded(db_path, true)?;
    let spinner = create_spinner("Opening database...");
    spinner.set_message(format!("Searching similar to '{}'...", key));

    let node_id = memory_node_id(namespace, key);
    let source_node = engine.get(node_id)?.ok_or_else(|| {
        crate::error::VantaError::Execution(format!(
            "Source record not found: {}:{}",
            namespace, key
        ))
    })?;

    let query_vector = match &source_node.vector {
        crate::node::VectorRepresentations::Full(v) => v.clone(),
        _ => {
            return Err(crate::error::VantaError::Execution(format!(
                "Record '{}:{}' has no vector embedding",
                namespace, key
            )));
        }
    };

    let raw_hits = db.search_vector(&query_vector, limit + 1)?;
    let mut hits: Vec<(String, String, String, f64)> = Vec::new();

    for hit in &raw_hits {
        if hit.node_id == node_id {
            continue;
        }
        if let Ok(Some(node)) = engine.get(hit.node_id) {
            let hit_ns = node
                .relational
                .get(FIELD_NAMESPACE)
                .and_then(|v| match v {
                    crate::node::FieldValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let hit_key = node
                .relational
                .get(FIELD_KEY)
                .and_then(|v| match v {
                    crate::node::FieldValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let hit_payload = node
                .relational
                .get(FIELD_PAYLOAD)
                .and_then(|v| match v {
                    crate::node::FieldValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let score = 1.0 - (hit.distance as f64).clamp(0.0, 1.0);
            hits.push((hit_ns, hit_key, hit_payload, score));
        }
    }

    hits.truncate(limit);
    spinner.finish_and_clear();

    if json_output {
        let results: Vec<serde_json::Value> = hits
            .iter()
            .map(|(ns, k, payload, score)| {
                serde_json::json!({
                    "key": k,
                    "namespace": ns,
                    "payload": payload,
                    "score": score,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&results).map_err(|e| {
                crate::error::VantaError::Execution(format!("JSON serialization error: {e}"))
            })?
        );
        return Ok(());
    }

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╭──────────────────────────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(format!(
            "│  Similar to \"{}\" in \"{}\" ({} hits)                        │",
            key,
            namespace,
            hits.len()
        ))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├──────────────────────────────────────────────────────────────┤")
    ));

    if hits.is_empty() {
        let _ = term.write_line(&format!(
            "{}",
            warning_style()
                .apply_to("│  No results found                                          │")
        ));
    } else {
        for (i, (_, k, _, score)) in hits.iter().enumerate() {
            let line = format!("│  #{:<3} │ {:<30} │ {:>8.4} │", i + 1, k, score);
            let _ = term.write_line(&format!("{}", info_style().apply_to(&line)));
        }
    }

    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰──────────────────────────────────────────────────────────────╯")
    ));

    Ok(())
}

pub fn cmd_count(
    db_path: &str,
    namespace: &str,
    _filter: Option<&str>,
    _filter_op: &str,
    json_output: bool,
) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            println!("0");
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
    spinner.set_message("Counting records...");

    let nodes = engine.scan_nodes()?;
    let count = nodes
        .iter()
        .filter(|n| {
            let ns = n.relational.get(FIELD_NAMESPACE).and_then(|v| match v {
                crate::node::FieldValue::String(s) => Some(s.as_str()),
                _ => None,
            });
            ns == Some(namespace)
        })
        .count() as u64;

    spinner.finish_and_clear();

    if json_output {
        let result = serde_json::json!({
            "namespace": namespace,
            "count": count,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| {
                crate::error::VantaError::Execution(format!("JSON serialization error: {e}"))
            })?
        );
        return Ok(());
    }

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!("Record count in '{}': {}", namespace, count));

    Ok(())
}

pub fn cmd_delete_by_filter(
    db_path: &str,
    namespace: &str,
    filter_key: &str,
    filter_val: &str,
    verbose: bool,
) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let engine = open_database(db_path, false)?;
    spinner.set_message("Scanning for matching records...");

    let nodes = engine.scan_nodes()?;
    let mut deleted_count = 0u64;

    for node in nodes {
        let ns = node.relational.get(FIELD_NAMESPACE).and_then(|v| match v {
            crate::node::FieldValue::String(s) => Some(s.as_str()),
            _ => None,
        });

        if ns != Some(namespace) {
            continue;
        }

        let fv = node.relational.get(filter_key);
        let matches = match fv {
            Some(crate::node::FieldValue::String(s)) => s == filter_val,
            Some(crate::node::FieldValue::Int(i)) => i.to_string() == filter_val,
            Some(crate::node::FieldValue::Float(f)) => f.to_string() == filter_val,
            _ => false,
        };

        if matches {
            let key = node
                .relational
                .get(FIELD_KEY)
                .and_then(|v| match v {
                    crate::node::FieldValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            let node_id = memory_node_id(namespace, &key);
            engine.delete(node_id, "delete_by_filter")?;
            deleted_count += 1;
        }
    }

    spinner.finish_and_clear();

    if deleted_count > 0 {
        print_success(&format!(
            "Deleted {} record(s) matching {}={} in namespace '{}'",
            deleted_count, filter_key, filter_val, namespace
        ));
    } else {
        print_warning(&format!(
            "No records matching {}={} found in namespace '{}'",
            filter_key, filter_val, namespace
        ));
    }

    if verbose {
        print_info(&format!("Deleted count: {}", deleted_count));
    }

    Ok(())
}

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
        return Err(crate::error::VantaError::Execution(format!(
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
        crate::error::VantaError::Execution(format!("Failed to copy database to backup: {e}"))
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

pub fn cmd_restore(
    db_path: &str,
    input: &str,
    force: bool,
    rebuild: bool,
    verbose: bool,
) -> Result<()> {
    let src = std::path::Path::new(input);
    if !src.exists() {
        return Err(crate::error::VantaError::Execution(format!(
            "Backup directory does not exist at '{}'",
            input
        )));
    }

    let dst = std::path::Path::new(db_path);

    if dst.exists() && !force {
        return Err(crate::error::VantaError::Execution(
            "Destination database directory already exists. Use --force to overwrite.".to_string(),
        ));
    }

    let spinner = create_spinner("Restoring from backup...");

    if dst.exists() && force {
        std::fs::remove_dir_all(dst).map_err(|e| {
            crate::error::VantaError::Execution(format!(
                "Failed to remove existing database directory: {e}"
            ))
        })?;
    }

    std::fs::create_dir_all(dst).map_err(|e| {
        crate::error::VantaError::Execution(format!("Failed to create database directory: {e}"))
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
        crate::error::VantaError::Execution(format!("Failed to restore from backup: {e}"))
    })?;

    spinner.set_message("Verifying restored database...");

    if rebuild {
        spinner.set_message("Rebuilding indexes...");
        let db = open_embedded(db_path, false)?;
        db.rebuild_index().map_err(|e| {
            crate::error::VantaError::Execution(format!("Index rebuild after restore failed: {e}"))
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
                crate::error::VantaError::Execution(format!("JSON serialization error: {e}"))
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

pub fn cmd_repl(db_path: &str, command: Option<&str>) -> Result<()> {
    let db = open_embedded(db_path, true)?;

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║        VantaDB Interactive REPL           ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╠════════════════════════════════════════════╣")
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  Commands:                                 ║")
    ));
    let _ = term.write_line("║  put <ns> <key> <payload>                ║");
    let _ = term.write_line("║  get <ns> <key>                          ║");
    let _ = term.write_line("║  del <ns> <key>                          ║");
    let _ = term.write_line("║  search <ns> <query>                     ║");
    let _ = term.write_line("║  list <ns>                               ║");
    let _ = term.write_line("║  count <ns> [filter]                     ║");
    let _ = term.write_line("║  ns                                      ║");
    let _ = term.write_line("║  stats                                   ║");
    let _ = term.write_line("║  help                                    ║");
    let _ = term.write_line("║  exit / quit                             ║");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚════════════════════════════════════════════╝")
    ));

    if let Some(startup) = command {
        let _ = term.write_line("");
        print_info(&format!("Startup command: {}", startup));
        print_info("Startup commands not supported in single-command mode.");
    }

    loop {
        let _ = write!(std::io::stdout(), "vanta> ").ok();
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() || line.trim().is_empty() {
            continue;
        }

        let line = line.trim().to_string();

        match line.as_str() {
            "exit" | "quit" | ".exit" | "q" => {
                println!("Goodbye!");
                break;
            }
            "help" | "?" => {
                println!("Commands: put, get, del, search, list, count, ns, stats, help, exit");
            }
            _ => {
                let parts: Vec<&str> = line.splitn(4, ' ').collect();
                match parts[0] {
                    "put" => {
                        if parts.len() < 4 {
                            println!("Usage: put <namespace> <key> <payload>");
                            continue;
                        }
                        match db.put(crate::sdk::VantaMemoryInput {
                            namespace: parts[1].to_string(),
                            key: parts[2].to_string(),
                            payload: parts[3].to_string(),
                            vector: None,
                            metadata: crate::sdk::VantaMemoryMetadata::new(),
                            ttl_ms: None,
                        }) {
                            Ok(_) => println!("OK"),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    "get" => {
                        if parts.len() < 3 {
                            println!("Usage: get <namespace> <key>");
                            continue;
                        }
                        match db.get(parts[1], parts[2]) {
                            Ok(Some(rec)) => println!("{}", rec.payload),
                            Ok(None) => println!("(not found)"),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    "del" => {
                        if parts.len() < 3 {
                            println!("Usage: del <namespace> <key>");
                            continue;
                        }
                        match db.delete(parts[1], parts[2]) {
                            Ok(true) => println!("Deleted"),
                            Ok(false) => println!("(not found)"),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    "search" => {
                        if parts.len() < 3 {
                            println!("Usage: search <namespace> <query>");
                            continue;
                        }
                        let req = crate::sdk::VantaMemorySearchRequest {
                            namespace: parts[1].to_string(),
                            query_vector: vec![],
                            filters: crate::sdk::VantaMemoryMetadata::new(),
                            text_query: Some(parts[2].to_string()),
                            top_k: 10,
                            distance_metric: crate::node::DistanceMetric::Cosine,
                            explain: false,
                        };
                        match db.search(req) {
                            Ok(hits) => {
                                for hit in &hits {
                                    println!(
                                        "  [{}] {:>8.4}  {}",
                                        hit.record.key, hit.score, hit.record.payload
                                    );
                                }
                            }
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    "list" => {
                        if parts.len() < 2 {
                            println!("Usage: list <namespace>");
                            continue;
                        }
                        let opts = crate::sdk::VantaMemoryListOptions {
                            filters: crate::sdk::VantaMemoryMetadata::new(),
                            limit: 100,
                            cursor: None,
                        };
                        match db.list(parts[1], opts) {
                            Ok(page) => {
                                for rec in &page.records {
                                    println!("  {}  {}", rec.key, rec.payload);
                                }
                            }
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    "count" => {
                        let ns = parts.get(1).unwrap_or(&"default");
                        match db.list(
                            ns,
                            crate::sdk::VantaMemoryListOptions {
                                filters: crate::sdk::VantaMemoryMetadata::new(),
                                limit: 1000000,
                                cursor: None,
                            },
                        ) {
                            Ok(page) => println!("Count: {}", page.records.len()),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    "ns" => match db.list_namespaces() {
                        Ok(ns_list) => {
                            for ns in &ns_list {
                                println!("  {}", ns);
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    },
                    "stats" => {
                        let engine = open_database(db_path, false)?;
                        let s = engine.get_memory_stats();
                        println!("  Records on HNSW:  {}", s.node_count);
                        println!("  Cache entries:     {}", s.cache_entries);
                        println!(
                            "  Logical size:      {}",
                            human_readable_size(s.logical_bytes)
                        );
                        if let Some(rss) = s.physical_rss {
                            println!("  Physical RSS:      {}", human_readable_size(rss));
                        }
                    }
                    "" => {}
                    other => {
                        println!("Unknown command: '{}'. Type 'help'.", other);
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn cmd_tui(db_path: &str) -> Result<()> {
    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔══════════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║               VantaDB Live TUI Dashboard                     ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╠══════════════════════════════════════════════════════════════╣")
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  Refreshing every 2 seconds (Ctrl+C to exit)                ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚══════════════════════════════════════════════════════════════╝")
    ));
    let _ = term.write_line("");

    loop {
        let embed = open_embedded(db_path, true)?;
        let namespaces = embed.list_namespaces().unwrap_or_default();
        let engine = open_database(db_path, true)?;
        let stats = engine.get_memory_stats();
        let nodes = engine.scan_nodes().unwrap_or_default();

        let total_records = nodes.len();
        let vector_nodes = nodes
            .iter()
            .filter(|n| n.flags.is_set(crate::node::NodeFlags::HAS_VECTOR))
            .count();
        let total_payload: u64 = nodes
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

        let _ = term.write_line(&format!(
            "{}",
            info_style()
                .apply_to("┌──────────────────────────────────────────────────────────────┐")
        ));
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to(format!(
                "│  Records: {:<5}  Vectors: {:<5}  NS: {:<3}  Size: {:<10}      │",
                total_records,
                vector_nodes,
                namespaces.len(),
                human_readable_size(total_payload),
            ))
        ));
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to(format!(
                "│  HNSW: {:<5}  Cache: {:<5}  Logical: {:<10}               │",
                stats.node_count,
                stats.cache_entries,
                human_readable_size(stats.logical_bytes),
            ))
        ));
        let _ = term.write_line(&format!(
            "{}",
            info_style()
                .apply_to("└──────────────────────────────────────────────────────────────┘")
        ));
        let _ = term.write_line("");
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

fn human_readable_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
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
