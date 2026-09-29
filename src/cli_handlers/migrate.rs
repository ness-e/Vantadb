//! Migration command handlers — plan, check, and execute.

use console::Term;
use web_time::Instant;

use crate::cli_handlers::fmt::header_style;
use crate::cli_handlers::{
    confirm_action, create_spinner, print_error, print_info, print_json, print_success,
    print_warning,
};
use crate::error::{ChainedError, Result};

#[tracing::instrument]
/// Print the planned migrations without executing them
pub fn cmd_migrate_plan(db_path: &str, verbose: bool, json_output: bool) -> Result<()> {
    use crate::migration::MigrationEngine;

    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_error(&format!("Database directory not found: {}", db_path));
        return Err(crate::error::Error::Cli(ChainedError::msg(format!(
            "Database path does not exist: {}",
            db_path
        ))));
    }

    let engine = MigrationEngine::new(db_path);
    let plans = engine.plan_all()?;

    if json_output {
        let plans_json: Vec<serde_json::Value> = plans
            .iter()
            .map(|plan| {
                serde_json::json!({
                    "format": plan.format.name(),
                    "current_version": plan.current_version,
                    "target_version": plan.target_version,
                    "action": plan.action,
                })
            })
            .collect();
        return print_json(&serde_json::json!({ "plans": plans_json }));
    }

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔═══════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║           VantaDB Migration Plan                        ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚═══════════════════════════════════════════════════════════╝")
    ));
    let _ = term.write_line("");

    if plans.is_empty() {
        print_success("All formats are at their latest version");
    } else {
        for plan in &plans {
            let _ = term.write_line(&format!(
                "  [{}] v{} → v{}: {}",
                plan.format.name(),
                plan.current_version,
                plan.target_version,
                plan.action
            ));
        }
        if verbose {
            print_info(&format!("Total planned migrations: {}", plans.len()));
        }
    }

    Ok(())
}

#[tracing::instrument]
/// Check storage integrity and report any issues found
pub fn cmd_migrate_check(db_path: &str, verbose: bool, json_output: bool) -> Result<()> {
    use crate::migration::MigrationEngine;

    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_error(&format!("Database directory not found: {}", db_path));
        return Err(crate::error::Error::Cli(ChainedError::msg(format!(
            "Database path does not exist: {}",
            db_path
        ))));
    }

    let engine = MigrationEngine::new(db_path);
    let issues = engine.check_integrity()?;

    if json_output {
        return print_json(&serde_json::json!({
            "issues": issues,
            "issue_count": issues.len(),
        }));
    }

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔═══════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║           VantaDB Integrity Check                       ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚═══════════════════════════════════════════════════════════╝")
    ));
    let _ = term.write_line("");

    if issues.is_empty() {
        print_success("No integrity issues found");
    } else {
        for issue in &issues {
            let _ = term.write_line(&format!("  ⚠ {}", issue));
        }
        if verbose {
            print_info(&format!("Total issues found: {}", issues.len()));
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
    json_output: bool,
) -> Result<()> {
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
            header_style().apply_to("║           VantaDB Database Migration                     ║")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╚═══════════════════════════════════════════════════════════╝")
        ));
        let _ = term.write_line("");
    }

    let target = std::path::Path::new(target_path);
    if !target.exists() {
        print_error(&format!("Database directory not found: {}", target_path));
        return Err(crate::error::Error::Cli(ChainedError::msg(format!(
            "Database path does not exist: {}",
            target_path
        ))));
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
                    "Unknown format: {}. Valid values: all, vfile, index, wal, records, schema",
                    format
                ));
                return Err(crate::error::Error::Cli(ChainedError::msg(format!(
                    "Unknown format: {}",
                    format
                ))));
            }
        }
    };

    let mut schema_json = serde_json::Value::Null;
    let mut records_json = serde_json::Value::Null;
    let mut plans_json: Vec<serde_json::Value> = Vec::new();
    let mut issues_json: Vec<String> = Vec::new();

    let mut engine = MigrationEngine::new(target_path);
    engine.set_dry_run(dry_run);
    // Whether this run already ensured the records backfill — the schema bump
    // below must never run while the backfill is pending (ADR-046).
    let mut records_processed = false;

    // ADR-046 §Migration: expand → backfill → bump. The records backfill runs
    // BEFORE the schema header bump — a crash mid-backfill leaves header v1 +
    // partially backfilled nodes (both binaries keep reading; re-run completes).
    if formats.contains(&FormatKind::Records) {
        records_processed = true;
        if engine.records_backfill_pending()? {
            if !dry_run {
                if json_output && !force {
                    return Err(crate::error::Error::Cli(ChainedError::msg(
                        "migrate run --json requires --force (JSON mode never prompts interactively)",
                    )));
                }
                if !force {
                    let _ = term.write_line("");
                    print_warning(
                        "The v1 → v2 record backfill will rewrite memory-record nodes and snapshots.",
                    );
                    if !confirm_action("Proceed with record backfill?")? {
                        print_warning("Migration cancelled by user");
                        return Ok(());
                    }
                }
            }
            let report = engine.migrate_records()?;
            records_json = serde_json::json!({
                "status": if dry_run { "dry-run" } else { "migrated" },
                "scanned": report.scanned,
                "backfilled": report.backfilled,
                "already_v2": report.already_v2,
                "snapshots_migrated": report.snapshots_migrated,
                "duration_ms": report.duration_ms,
            });
            if !json_output {
                if dry_run {
                    print_info(&format!(
                        "[dry-run] Records backfill: {} of {} memory records need v2 fields; {} snapshots need re-encode",
                        report.backfilled, report.scanned, report.snapshots_migrated
                    ));
                } else {
                    print_success(&format!(
                        "Records backfilled: {} nodes ({} already v2), {} snapshots re-encoded ({} ms)",
                        report.backfilled,
                        report.already_v2,
                        report.snapshots_migrated,
                        report.duration_ms
                    ));
                }
            }
        } else {
            records_json = serde_json::json!({ "status": "current" });
            if !json_output {
                print_info("Memory records are already at the latest schema version");
            }
        }
    }

    // Schema migration uses the existing logic
    if formats.contains(&FormatKind::Schema) || format == "all" {
        let schema_path = target.join(".vanta.schema");
        let current_header = match StorageHeader::read_from(&schema_path)? {
            Some(header) => {
                if verbose && !json_output {
                    print_info(&format!(
                        "Current schema: version={}, min_compat={}, flags={}",
                        header.version, header.min_compat_version, header.flags
                    ));
                }
                header
            }
            None => {
                // No header yet (pre-versioning DB). The bump-is-the-marker
                // rule cannot apply to a header-less DB (any engine open would
                // create the current header anyway) — but `dry_run` still must
                // not modify files (cli.rs "Preview changes without modifying").
                if !dry_run {
                    let header = StorageHeader::current();
                    header.write_to(&schema_path)?;
                }
                if json_output {
                    return print_json(&serde_json::json!({
                        "target": target_path,
                        "format": format,
                        "dry_run": dry_run,
                        "schema": {
                            "status": if dry_run { "dry-run" } else { "written" },
                            "version": CURRENT_SCHEMA_VERSION,
                        },
                        "records": records_json,
                        "plans": plans_json,
                        "integrity_issues": issues_json,
                    }));
                }
                print_warning("No schema file found; database may be pre-versioning.");
                if dry_run {
                    print_info(&format!(
                        "[dry-run] Would write schema header version={}",
                        CURRENT_SCHEMA_VERSION
                    ));
                } else {
                    print_info("Writing current schema header...");
                    print_success(&format!(
                        "Schema header written: version={}",
                        CURRENT_SCHEMA_VERSION
                    ));
                }
                return Ok(());
            }
        };

        if current_header.version > CURRENT_SCHEMA_VERSION {
            print_error(&format!(
                "Database schema version {} is newer than this software (max {})",
                current_header.version, CURRENT_SCHEMA_VERSION
            ));
            return Err(crate::error::Error::Schema(format!(
                "Schema version {} is too new for this version of VantaDB",
                current_header.version
            )));
        }

        if current_header.version != CURRENT_SCHEMA_VERSION {
            if dry_run {
                // cli.rs contract: dry-run previews without modifying files.
                schema_json = serde_json::json!({
                    "status": "dry-run",
                    "from": current_header.version,
                    "to": CURRENT_SCHEMA_VERSION,
                });
                if !json_output {
                    print_info(&format!(
                        "[dry-run] Schema would migrate: version {} → {}",
                        current_header.version, CURRENT_SCHEMA_VERSION
                    ));
                    if !records_processed {
                        print_info(
                            "[dry-run] Records backfill is pending — run `--format records` (or `--format all`) before the schema bump",
                        );
                    }
                }
            } else {
                // ADR-046 §Migration (expand → backfill → bump): the header
                // bump is the migration-complete marker. Never write it while
                // the records backfill hasn't run — `--format schema` alone
                // would otherwise disable the backfill forever (the pending
                // predicate is `header.version < CURRENT`). If `records` was
                // not part of this run, ensure it now (idempotent).
                if !records_processed {
                    let report = engine.migrate_records()?;
                    records_json = serde_json::json!({
                        "status": "ensured-before-schema-bump",
                        "scanned": report.scanned,
                        "backfilled": report.backfilled,
                        "already_v2": report.already_v2,
                        "snapshots_migrated": report.snapshots_migrated,
                        "duration_ms": report.duration_ms,
                    });
                    if !json_output {
                        print_info(&format!(
                            "Records backfill ensured before the schema bump: {} nodes ({} already v2), {} snapshots",
                            report.backfilled, report.already_v2, report.snapshots_migrated
                        ));
                    }
                }

                let spinner = create_spinner("Migrating schema...");
                let start = Instant::now();

                let new_header = StorageHeader::current();
                new_header.write_to(&schema_path)?;

                let elapsed = start.elapsed();
                spinner.finish_and_clear();

                schema_json = serde_json::json!({
                    "status": "migrated",
                    "from": current_header.version,
                    "to": CURRENT_SCHEMA_VERSION,
                });

                if !json_output {
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
                }
            }
        } else {
            schema_json = serde_json::json!({
                "status": "current",
                "version": CURRENT_SCHEMA_VERSION,
            });
            if !json_output {
                print_info("Schema is already at the latest version");
            }
        }
    }

    // Physical format migration (Records ran before the schema bump above;
    // Schema is also handled there).
    let physical_formats: Vec<FormatKind> = formats
        .into_iter()
        .filter(|f| *f != FormatKind::Schema && *f != FormatKind::Records)
        .collect();

    if !physical_formats.is_empty() {
        if dry_run {
            if !json_output {
                print_info("--- Dry Run: checking migration requirements ---");
            }
            let plans = engine.plan_all()?;
            plans_json = plans
                .iter()
                .map(|plan| {
                    serde_json::json!({
                        "format": plan.format.name(),
                        "current_version": plan.current_version,
                        "target_version": plan.target_version,
                        "action": plan.action,
                    })
                })
                .collect();
            if !json_output {
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
            }
        } else {
            if !force {
                // JSON mode is non-interactive: mutating migrations require
                // an explicit `--force` so `--json` consumers never hang on a
                // prompt.
                if json_output {
                    return Err(crate::error::Error::Cli(ChainedError::msg(
                        "migrate run --json requires --force (JSON mode never prompts interactively)",
                    )));
                }
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
            } else if json_output {
                plans_json = engine
                    .plan_all()?
                    .iter()
                    .map(|plan| {
                        serde_json::json!({
                            "format": plan.format.name(),
                            "current_version": plan.current_version,
                            "target_version": plan.target_version,
                            "action": plan.action,
                        })
                    })
                    .collect();
            }

            for fmt in &physical_formats {
                let spinner = create_spinner(&format!("Migrating {}...", fmt.name()));
                let start = Instant::now();
                engine.migrate_format(*fmt)?;
                let elapsed = start.elapsed();
                spinner.finish_and_clear();
                if verbose && !json_output {
                    print_info(&format!(
                        "  {} completed in {} ms",
                        fmt.name(),
                        elapsed.as_millis()
                    ));
                }
            }

            // Check integrity after migration
            let issues = engine.check_integrity()?;
            if !issues.is_empty() && !json_output {
                print_warning("Post-migration integrity warnings:");
                for issue in &issues {
                    let _ = term.write_line(&format!("  ⚠ {}", issue));
                }
            }
            issues_json = issues;
        }
    }

    if json_output {
        return print_json(&serde_json::json!({
            "target": target_path,
            "format": format,
            "dry_run": dry_run,
            "records": records_json,
            "schema": schema_json,
            "plans": plans_json,
            "integrity_issues": issues_json,
        }));
    }

    Ok(())
}
