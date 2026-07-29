//! WAL command handlers — compact, vacuum.

use console::Term;
use web_time::Instant;

use crate::cli_handlers::fmt::{header_style, success_style};
use crate::cli_handlers::{create_spinner, open_embedded, print_error, print_success};
use crate::error::Result;

#[tracing::instrument]
/// Compact the WAL: flush all data, archive the current WAL file, start a fresh one.
pub fn cmd_wal_compact(db_path: &str) -> Result<()> {
    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔═══════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║           VantaDB WAL Compaction                         ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚═══════════════════════════════════════════════════════════╝")
    ));
    let _ = term.write_line("");

    let spinner = create_spinner("Opening database...");

    let db = open_embedded(db_path, false)?;
    spinner.finish_and_clear();
    print_success("Database opened");

    let compact_spinner = create_spinner("Compacting WAL...");
    db.compact_wal()?;
    compact_spinner.finish_and_clear();

    let _ = term.write_line(&format!(
        "{}",
        success_style().apply_to("│  ✓ WAL compacted successfully                        │")
    ));

    Ok(())
}

#[tracing::instrument]
/// Remove tombstoned nodes from HNSW and reclaim space.
pub fn cmd_wal_vacuum(db_path: &str) -> Result<()> {
    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔═══════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║           VantaDB WAL Vacuum                             ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚═══════════════════════════════════════════════════════════╝")
    ));
    let _ = term.write_line("");

    let spinner = create_spinner("Opening database...");

    let db = open_embedded(db_path, false)?;
    spinner.finish_and_clear();
    print_success("Database opened");

    let vacuum_spinner = create_spinner("Vacuuming...");
    let start = Instant::now();

    let report = db.vacuum()?;

    vacuum_spinner.finish_and_clear();

    let total_duration = start.elapsed();

    if report.success {
        print_success("Vacuum completed successfully");
    } else {
        print_error("Vacuum failed");
    }

    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╭─────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        success_style().apply_to("│  ✓ Vacuum completed                     │")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├─────────────────────────────────────────┤")
    ));
    let _ = term.write_line(&format!(
        "│  Total time:        {:<18} │",
        format!("{:?}", total_duration)
    ));
    let _ = term.write_line(&format!(
        "│  Scanned nodes:     {:<18} │",
        report.scanned_nodes
    ));
    let _ = term.write_line(&format!(
        "│  Removed nodes:     {:<18} │",
        report.removed_nodes
    ));
    let _ = term.write_line(&format!(
        "│  Reclaimed bytes:   {:<18} │",
        report.reclaimed_bytes
    ));
    let _ = term.write_line(&format!(
        "│  Duration:          {:<18} │",
        format!("{} ms", report.duration_ms)
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}
