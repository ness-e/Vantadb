//! VantaDB CLI binary — thin entry point.
//! Handlers live in `vantadb::cli_handlers` for testability.

use anyhow::Context as _;
use clap::Parser;

use vantadb::cli::{Cli, Commands, ExportFormat};
use vantadb::cli_handlers;
use vantadb::config::LogFormat;
use vantadb::console;

#[cfg(all(feature = "jemalloc", not(target_os = "windows")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(all(
    feature = "custom-allocator",
    any(not(feature = "jemalloc"), target_os = "windows")
))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// ERR-CORE-02: bin → anyhow con `.context()` (cadena humana en stderr).
// La lib nunca usa anyhow; solo este bin. `?` sobre los handlers convierte
// `Error` (Error + Send + Sync + 'static) vía el `From` genérico de anyhow.
fn main() -> anyhow::Result<()> {
    run().context("vanta-cli: command failed")
}

fn run() -> anyhow::Result<()> {
    let args = Cli::parse();

    if args.verbose {
        console::init_logging(LogFormat::Full);
    }

    match args.command {
        Commands::Put {
            namespace,
            key,
            payload,
            vector,
            metadata,
        } => cli_handlers::cmd_put(
            &args.db,
            &namespace,
            &key,
            &payload,
            vector.as_deref(),
            metadata.as_deref(),
            args.verbose,
            args.json,
        )?,

        Commands::Get { namespace, key } => {
            cli_handlers::cmd_get(&args.db, &namespace, &key, args.verbose, args.json)?
        }

        Commands::List { namespace, limit } => {
            cli_handlers::cmd_list(&args.db, &namespace, limit, args.verbose, args.json)?
        }

        Commands::RebuildIndex => {
            cli_handlers::cmd_rebuild_index(&args.db, args.verbose, args.json)?
        }

        Commands::AuditIndex { namespace, deep } => {
            cli_handlers::cmd_audit_index(&args.db, namespace.as_deref(), args.json, deep)?
        }

        Commands::RepairTextIndex => cli_handlers::cmd_repair_text_index(&args.db, args.json)?,

        Commands::Export {
            namespace,
            out,
            format,
        } => match format {
            ExportFormat::Jsonl => {
                cli_handlers::cmd_export(&args.db, namespace.as_deref(), &out, args.json)?
            }
            ExportFormat::Md => {
                cli_handlers::cmd_export_md(&args.db, namespace.as_deref(), &out, args.json)?
            }
        },

        Commands::Import { input } => {
            cli_handlers::cmd_import(&args.db, &input, args.verbose, args.json)?
        }

        Commands::Query { query, limit } => {
            cli_handlers::cmd_query(&args.db, &query, limit, args.verbose, args.json)?
        }

        Commands::Search {
            namespace,
            query,
            query_flag,
            query_vector,
            limit,
        } => {
            // Clap requires one of the two (`required_unless_present`); keep a
            // defensive error so the binary never panics on the None case.
            let query = query.or(query_flag).ok_or_else(|| {
                anyhow::anyhow!("a query is required: pass it as <QUERY> or --query")
            })?;
            cli_handlers::cmd_search(
                &args.db,
                &namespace,
                &query,
                query_vector.as_deref(),
                limit,
                args.json,
            )?
        }

        Commands::Delete {
            namespace,
            key,
            attest,
            out,
        } => {
            if attest {
                cli_handlers::cmd_delete_certified(
                    &args.db,
                    &namespace,
                    &key,
                    out.as_deref(),
                    args.verbose,
                    args.json,
                )?
            } else {
                cli_handlers::cmd_delete(&args.db, &namespace, &key, args.verbose, args.json)?
            }
        }

        Commands::Certificate(cmd) => match cmd {
            vantadb::cli::CertificateCommand::Verify { file } => {
                let code = cli_handlers::cmd_certificate_verify(&args.db, &file, args.json)?;
                if code != 0 {
                    std::process::exit(code);
                }
            }
        },

        Commands::DeleteByFilter { namespace, filter } => cli_handlers::cmd_delete_by_filter(
            &args.db,
            &namespace,
            &filter,
            args.verbose,
            args.json,
        )?,

        Commands::Count { namespace, filter } => cli_handlers::cmd_count(
            &args.db,
            &namespace,
            filter.as_deref(),
            args.json,
            args.verbose,
        )?,

        Commands::SimilarToKey {
            namespace,
            key,
            limit,
        } => cli_handlers::cmd_similar_to_key(&args.db, &namespace, &key, limit, args.json)?,

        Commands::SearchMulti {
            namespaces,
            query,
            query_flag,
            query_vector,
            limit,
        } => cli_handlers::search::cmd_search_multi(
            &args.db,
            &namespaces,
            query.or(query_flag).as_deref(),
            query_vector.as_deref(),
            limit,
            args.json,
        )?,

        Commands::SearchAll {
            query,
            query_flag,
            query_vector,
            limit,
        } => cli_handlers::search::cmd_search_all(
            &args.db,
            query.or(query_flag).as_deref(),
            query_vector.as_deref(),
            limit,
            args.json,
        )?,

        Commands::Namespace(cmd) => match cmd {
            vantadb::cli::NamespaceCommand::List => {
                cli_handlers::cmd_namespace_list(&args.db, args.json)?
            }
            vantadb::cli::NamespaceCommand::Info { namespace } => {
                cli_handlers::cmd_namespace_info(&args.db, &namespace, args.json)?
            }
        },

        Commands::Migrate(cmd) => match cmd {
            vantadb::cli::MigrateCommand::Plan { target } => {
                cli_handlers::cmd_migrate_plan(&target, args.verbose, args.json)?
            }
            vantadb::cli::MigrateCommand::Run {
                target,
                format,
                dry_run,
                force,
            } => cli_handlers::cmd_migrate(
                &target,
                &format,
                dry_run,
                force,
                args.verbose,
                args.json,
            )?,
            vantadb::cli::MigrateCommand::Check { target } => {
                cli_handlers::cmd_migrate_check(&target, args.verbose, args.json)?
            }
        },

        Commands::Status => cli_handlers::cmd_status(&args.db, args.verbose, args.json)?,

        Commands::Backup { out } => {
            cli_handlers::cmd_backup(&args.db, out.as_deref(), args.verbose, args.json)?
        }

        Commands::Restore {
            input,
            force,
            rebuild,
            dry_run,
        } => cli_handlers::cmd_restore(
            &args.db,
            &input,
            cli_handlers::RestoreOptions {
                overwrite: if force {
                    cli_handlers::OverwritePolicy::Overwrite
                } else {
                    cli_handlers::OverwritePolicy::FailIfExists
                },
                rebuild: if rebuild {
                    cli_handlers::IndexRebuild::Yes
                } else {
                    cli_handlers::IndexRebuild::No
                },
                mode: if dry_run {
                    cli_handlers::RestoreMode::DryRun
                } else {
                    cli_handlers::RestoreMode::Apply
                },
                verbose: cli_handlers::Verbosity::from_flag(args.verbose),
            },
            args.json,
        )?,

        Commands::Doctor { fix, force } => {
            let mode = if force {
                cli_handlers::DoctorFix::Apply
            } else if fix {
                cli_handlers::DoctorFix::DryRun
            } else {
                cli_handlers::DoctorFix::Off
            };
            cli_handlers::cmd_doctor(
                &args.db,
                cli_handlers::DoctorOptions {
                    fix: mode,
                    verbose: cli_handlers::Verbosity::from_flag(args.verbose),
                },
                args.json,
            )?
        }

        Commands::Inspect { namespace, key } => {
            cli_handlers::cmd_inspect(&args.db, &namespace, &key, args.verbose, args.json)?
        }

        Commands::Stats => cli_handlers::cmd_stats(&args.db, args.json, args.verbose)?,

        Commands::Snapshot(cmd) => match cmd {
            vantadb::cli::SnapshotCommand::Create { name } => {
                cli_handlers::cmd_snapshot_create(&args.db, &name, args.verbose, args.json)?
            }
            vantadb::cli::SnapshotCommand::List => {
                cli_handlers::cmd_snapshot_list(&args.db, args.json)?
            }
        },

        Commands::Wal(cmd) => match cmd {
            vantadb::cli::WalCommand::Compact => {
                cli_handlers::cmd_wal_compact(&args.db, args.json)?
            }
            vantadb::cli::WalCommand::Vacuum => cli_handlers::cmd_wal_vacuum(&args.db, args.json)?,
            vantadb::cli::WalCommand::Salvage { dry_run } => {
                cli_handlers::cmd_wal_salvage(&args.db, dry_run, args.json)?
            }
        },

        Commands::Verify => {
            let code = cli_handlers::cmd_verify(&args.db, args.json)?;
            if code != 0 {
                std::process::exit(code);
            }
        }

        Commands::Completions { shell } => cli_handlers::cmd_completions(shell),

        Commands::McpCall {
            tool,
            args: tool_args,
            timeout_secs,
        } => {
            let code = cli_handlers::cmd_mcp_call(&args.db, &tool, &tool_args, timeout_secs)?;
            if code != 0 {
                std::process::exit(code);
            }
        }

        Commands::Server {
            http,
            mcp,
            port,
            host,
            require_auth,
            allow_insecure,
            dashboard_dir,
        } => cli_handlers::cmd_server(
            &args.db,
            http,
            mcp,
            port,
            host,
            require_auth,
            allow_insecure,
            dashboard_dir,
            args.memory_limit.as_deref(),
            args.verbose,
        )?,

        #[cfg(feature = "tui")]
        Commands::Tui => {
            let engine = std::sync::Arc::new(cli_handlers::open_database(&args.db, true)?);
            vantadb::tui::run_tui(engine)?
        }
    }

    Ok(())
}
