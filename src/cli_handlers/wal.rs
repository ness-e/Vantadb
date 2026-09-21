//! WAL command handlers — compact, vacuum, salvage.

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

#[tracing::instrument]
/// Salvage a truncated sharded WAL (FIND-109, explicit opt-in).
/// `--dry_run`: report only. Otherwise truncate shards to the coherent
/// prefix (tails quarantined to `<shard>.salvage[.N]`). Exit 0 on success;
/// already-coherent WALs report and exit 0 without mutating.
pub fn cmd_wal_salvage(db_path: &str, dry_run: bool) -> Result<()> {
    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔═══════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║           VantaDB WAL Salvage (opt-in)                   ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚═══════════════════════════════════════════════════════════╝")
    ));
    let _ = term.write_line("");

    let wal_base = std::path::Path::new(db_path).join("data").join("vanta.wal");
    let num_shards = crate::wal_sharded::detect_shard_count(&wal_base)
        .or_else(|| crate::wal_sharded::read_shard_meta(&wal_base))
        .unwrap_or(4)
        .max(1);

    let spinner = create_spinner("Inspecting WAL shards (read-only)...");
    let preview = crate::wal_sharded::salvage_preview(&wal_base, num_shards)?;
    spinner.finish_and_clear();

    let _ = term.write_line(&format!("│  Shards:            {:?}", preview.shard_counts));
    let _ = term.write_line(&format!(
        "│  Coherent prefix:   {:?}",
        preview.coherent_prefix
    ));
    let _ = term.write_line(&format!("│  Replayed (kept):   {}", preview.replayed));
    let _ = term.write_line(&format!("│  Discarded:         {}", preview.discarded));
    if preview.discarded_global_seqs.is_empty() {
        let _ = term.write_line("│  Discarded globals: none");
    } else {
        let _ = term.write_line(&format!(
            "│  Discarded globals: {:?}",
            preview.discarded_global_seqs
        ));
    }

    if preview.coherent {
        print_success("WAL already coherent — nothing to salvage");
        return Ok(());
    }
    if dry_run {
        print_success("Dry-run: coherent prefix above, no files mutated");
        return Ok(());
    }

    let spinner = create_spinner("Truncating shards to coherent prefix...");
    let done = crate::wal_sharded::salvage(&wal_base, num_shards)?;
    spinner.finish_and_clear();

    let _ = term.write_line(&format!("│  Before:            {:?}", done.before));
    let _ = term.write_line(&format!("│  Prefix applied:    {:?}", done.prefix));
    let _ = term.write_line(&format!(
        "│  After (kept):      {:?} (replayed {})",
        done.after.shard_counts, done.after.replayed
    ));
    for b in &done.backups {
        let _ = term.write_line(&format!("│  Quarantined tail:  {}", b.display()));
    }
    let _ = term.write_line(&format!(
        "{}",
        success_style().apply_to("│  ✓ WAL salvaged — coherent prefix restored           │")
    ));
    print_success("WAL salvage complete");
    Ok(())
}
