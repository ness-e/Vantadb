//! Data command handlers — export, import, query.

use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::cli_handlers::fmt::{error_style, header_style, success_style};
use crate::cli_handlers::{
    create_spinner, open_database, open_embedded, print_error, print_info, print_success,
    print_warning,
};
use crate::error::{ChainedError, Result};

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
    bar.enable_steady_tick(Duration::from_millis(100));

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
                    .map_err(|e| crate::error::VantaError::serialization(e))?;
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
        return Err(crate::error::VantaError::CliError(ChainedError::msg(
            format!("Input file not found: {}", input_path),
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
    use web_time::Instant;

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
