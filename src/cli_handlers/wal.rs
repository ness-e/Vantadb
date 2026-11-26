//! WAL command handlers — compact, vacuum, salvage, verify.

use console::Term;
use std::path::Path;
use web_time::Instant;

use crate::cli_handlers::fmt::{header_style, success_style};
use crate::cli_handlers::{create_spinner, open_embedded, print_error, print_json, print_success};
use crate::error::Result;
use crate::wal::{WalVerifyReport, WalVerifyStatus};

#[tracing::instrument]
/// Compact the WAL: flush all data, archive the current WAL file, start a fresh one.
pub fn cmd_wal_compact(db_path: &str, json_output: bool) -> Result<()> {
    let term = Term::stdout();
    if !json_output {
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╔═══════════════════════════════════════════════════════════╗")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to("║           VantaDB WAL Compaction                         ║")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╚═══════════════════════════════════════════════════════════╝")
        ));
        let _ = term.write_line("");
    }

    let spinner = create_spinner("Opening database...");

    let db = open_embedded(db_path, false)?;
    spinner.finish_and_clear();
    if !json_output {
        print_success("Database opened");
    }

    let compact_spinner = create_spinner("Compacting WAL...");
    db.compact_wal()?;
    compact_spinner.finish_and_clear();

    if json_output {
        return print_json(&serde_json::json!({
            "command": "wal_compact",
            "success": true,
            "path": db_path,
        }));
    }

    let _ = term.write_line(&format!(
        "{}",
        success_style().apply_to("│  ✓ WAL compacted successfully                        │")
    ));

    Ok(())
}

#[tracing::instrument]
/// Remove tombstoned nodes from HNSW and reclaim space.
pub fn cmd_wal_vacuum(db_path: &str, json_output: bool) -> Result<()> {
    let term = Term::stdout();
    if !json_output {
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╔═══════════════════════════════════════════════════════════╗")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to("║           VantaDB WAL Vacuum                             ║")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╚═══════════════════════════════════════════════════════════╝")
        ));
        let _ = term.write_line("");
    }

    let spinner = create_spinner("Opening database...");

    let db = open_embedded(db_path, false)?;
    spinner.finish_and_clear();
    if !json_output {
        print_success("Database opened");
    }

    let vacuum_spinner = create_spinner("Vacuuming...");
    let start = Instant::now();

    let report = db.vacuum()?;

    vacuum_spinner.finish_and_clear();

    let total_duration = start.elapsed();

    if json_output {
        return print_json(&serde_json::json!({
            "command": "wal_vacuum",
            "success": report.success,
            "scanned_nodes": report.scanned_nodes,
            "removed_nodes": report.removed_nodes,
            "reclaimed_bytes": report.reclaimed_bytes,
            "duration_ms": report.duration_ms,
            "total_ms": total_duration.as_millis() as u64,
        }));
    }

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
pub fn cmd_wal_salvage(db_path: &str, dry_run: bool, json_output: bool) -> Result<()> {
    let term = Term::stdout();
    if !json_output {
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╔═══════════════════════════════════════════════════════════╗")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to("║           VantaDB WAL Salvage (opt-in)                   ║")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╚═══════════════════════════════════════════════════════════╝")
        ));
        let _ = term.write_line("");
    }

    let wal_base = std::path::Path::new(db_path).join("data").join("vanta.wal");
    let num_shards = crate::wal_sharded::detect_shard_count(&wal_base)
        .or_else(|| crate::wal_sharded::read_shard_meta(&wal_base))
        .unwrap_or(4)
        .max(1);

    let spinner = create_spinner("Inspecting WAL shards (read-only)...");
    let preview = crate::wal_sharded::salvage_preview(&wal_base, num_shards)?;
    spinner.finish_and_clear();

    if json_output {
        let mut result = serde_json::json!({
            "command": "wal_salvage",
            "dry_run": dry_run,
            "coherent": preview.coherent,
            "shards": preview.shard_counts,
            "coherent_prefix": preview.coherent_prefix,
            "replayed": preview.replayed,
            "discarded": preview.discarded,
            "discarded_global_seqs": preview.discarded_global_seqs,
            "backups": [],
        });
        if !preview.coherent && !dry_run {
            let done = crate::wal_sharded::salvage(&wal_base, num_shards)?;
            result["after"] = serde_json::json!({
                "shards": done.after.shard_counts,
                "replayed": done.after.replayed,
            });
            result["backups"] = serde_json::Value::Array(
                done.backups
                    .iter()
                    .map(|b| serde_json::Value::String(b.display().to_string()))
                    .collect(),
            );
        }
        return print_json(&result);
    }

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

#[tracing::instrument]
/// Verify the WAL hash-chain integrity (VER-01, tamper-evident).
///
/// Read-only and offline: walks every on-disk shard of `<db>/data/vanta.wal`
/// without opening the engine (works on a closed/locked DB). Detects altered
/// records (even with a recomputed CRC), removed/inserted/reordered records
/// (chain link break) and frame damage, reporting the exact byte offset +
/// 1-based record index. Legacy (pre-chain v1/v2) files are reported
/// explicitly, never as tampering.
///
/// Exit code: 0 = verified / legacy / crash-tail only; 1 = tampered or corrupt.
/// `docs/api/` documents the chain's coverage and its limits (clean-boundary
/// truncation, consistent whole-file rewrite and whole-segment deletion are
/// undetectable without an external anchor — v1.0 follow-up).
pub fn cmd_verify(db_path: &str, json_output: bool) -> Result<i32> {
    let term = Term::stdout();
    if !json_output {
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╔═══════════════════════════════════════════════════════════╗")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to("║           VantaDB WAL Verify (tamper-evident)            ║")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╚═══════════════════════════════════════════════════════════╝")
        ));
        let _ = term.write_line("");
    }

    let wal_base = Path::new(db_path).join("data").join("vanta.wal");
    let num_shards = crate::wal_sharded::detect_shard_count(&wal_base)
        .or_else(|| crate::wal_sharded::read_shard_meta(&wal_base))
        .unwrap_or(4)
        .max(1);

    let spinner = create_spinner("Verifying WAL hash-chain (read-only)...");
    let (reports, incoherence) = crate::wal_sharded::verify_shards(&wal_base, num_shards)?;
    spinner.finish_and_clear();

    let ok = reports.iter().all(|r| r.status.is_ok()) && incoherence.is_none();
    let exit_code = if ok { 0 } else { 1 };

    if json_output {
        let result = serde_json::json!({
            "command": "verify",
            "ok": ok,
            "wal": wal_base.display().to_string(),
            "shards": reports.iter().map(status_json).collect::<Vec<_>>(),
            "shard_counts_coherent": incoherence.is_none(),
            "shard_coherence_error": incoherence,
        });
        print_json(&result)?;
        return Ok(exit_code);
    }

    if reports.is_empty() {
        let _ = term.write_line(&format!("│  No WAL files found at {}", wal_base.display()));
        print_success("Nothing to verify");
        return Ok(0);
    }

    for report in &reports {
        let _ = term.write_line(&format!(
            "│  File:    {} (format v{})",
            report.path.display(),
            report.format_version
        ));
        let _ = term.write_line(&format!("│  Records: {}", report.records));
        match &report.status {
            WalVerifyStatus::Verified => {
                let _ = term.write_line("│  Status:  ✓ chain verified (SHA-256)");
            }
            WalVerifyStatus::Legacy => {
                let _ = term.write_line(
                    "│  Status:  • legacy format (pre-chain) — CRC walk only, no chain to verify",
                );
            }
            WalVerifyStatus::Tampered {
                offset,
                record,
                reason,
            } => {
                let _ = term.write_line(&format!(
                    "│  Status:  ✗ TAMPERED at record {record} (offset {offset}): {reason}"
                ));
            }
            WalVerifyStatus::Corrupt {
                offset,
                record,
                reason,
            } => {
                let _ = term.write_line(&format!(
                    "│  Status:  ✗ CORRUPT at record {record} (offset {offset}): {reason}"
                ));
            }
            WalVerifyStatus::IncompleteTail { offset } => {
                let _ = term.write_line(&format!(
                    "│  Status:  △ incomplete tail at offset {offset} (unclean shutdown; recovery will quarantine)"
                ));
            }
        }
        let _ = term.write_line("");
    }
    if let Some(msg) = &incoherence {
        let _ = term.write_line(&format!("│  ✗ Shard layout incoherent: {msg}"));
        let _ = term.write_line("");
    }

    if ok {
        print_success("WAL hash-chain verified");
    } else {
        print_error("WAL integrity FAILED — positions above");
    }
    Ok(exit_code)
}

/// JSON projection of one per-file verification report.
fn status_json(report: &WalVerifyReport) -> serde_json::Value {
    let mut value = serde_json::json!({
        "path": report.path.display().to_string(),
        "format_version": report.format_version,
        "records": report.records,
        "status": report.status.as_str(),
    });
    match &report.status {
        WalVerifyStatus::Tampered {
            offset,
            record,
            reason,
        }
        | WalVerifyStatus::Corrupt {
            offset,
            record,
            reason,
        } => {
            value["offset"] = serde_json::json!(offset);
            value["record"] = serde_json::json!(record);
            value["reason"] = serde_json::json!(reason);
        }
        WalVerifyStatus::IncompleteTail { offset } => {
            value["offset"] = serde_json::json!(offset);
        }
        WalVerifyStatus::Verified | WalVerifyStatus::Legacy => {}
    }
    value
}
