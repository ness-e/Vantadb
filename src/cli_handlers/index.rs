//! Index command handlers — rebuild, audit, repair.

use console::Term;
use web_time::Instant;

use crate::cli_handlers::fmt::{header_style, success_style};
use crate::cli_handlers::{
    create_spinner, open_embedded, print_error, print_success, print_warning,
};
use crate::error::{ChainedError, Result};

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
                crate::error::VantaError::CliError(ChainedError::msg(format!(
                    "failed to encode audit report: {err}"
                )))
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
