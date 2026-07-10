//! Migration command handlers — plan, check, and execute.

use console::Term;
use web_time::Instant;

use crate::cli_handlers::fmt::header_style;
use crate::cli_handlers::{
    confirm_action, create_spinner, print_error, print_info, print_success, print_warning,
};
use crate::error::Result;

#[tracing::instrument]
/// Print the planned migrations without executing them
pub fn cmd_migrate_plan(db_path: &str, verbose: bool) -> Result<()> {
    use crate::migration::MigrationEngine;

    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_error(&format!("Database directory not found: {}", db_path));
        return Err(crate::error::VantaError::CliError(format!(
            "Database path does not exist: {}",
            db_path
        )));
    }

    let engine = MigrationEngine::new(db_path);
    let plans = engine.plan_all()?;

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
pub fn cmd_migrate_check(db_path: &str, verbose: bool) -> Result<()> {
    use crate::migration::MigrationEngine;

    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_error(&format!("Database directory not found: {}", db_path));
        return Err(crate::error::VantaError::CliError(format!(
            "Database path does not exist: {}",
            db_path
        )));
    }

    let engine = MigrationEngine::new(db_path);
    let issues = engine.check_integrity()?;

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
