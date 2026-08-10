//! VantaDB CLI binary — thin entry point.
//! Handlers live in `vantadb::cli_handlers` for testability.

use clap::Parser;

use vantadb::cli::{Cli, Commands};
use vantadb::cli_handlers;
use vantadb::config::LogFormat;
use vantadb::console;
use vantadb::error::Result;

#[cfg(all(feature = "jemalloc", not(target_os = "windows")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(all(
    feature = "custom-allocator",
    any(not(feature = "jemalloc"), target_os = "windows")
))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<()> {
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
        } => cli_handlers::cmd_put(
            &args.db,
            &namespace,
            &key,
            &payload,
            vector.as_deref(),
            args.verbose,
        )?,

        Commands::Get { namespace, key } => {
            cli_handlers::cmd_get(&args.db, &namespace, &key, args.verbose)?
        }

        Commands::List { namespace, limit } => {
            cli_handlers::cmd_list(&args.db, &namespace, limit, args.verbose)?
        }

        Commands::RebuildIndex => cli_handlers::cmd_rebuild_index(&args.db, args.verbose)?,

        Commands::AuditIndex {
            namespace,
            json,
            deep,
        } => cli_handlers::cmd_audit_index(&args.db, namespace.as_deref(), json, deep)?,

        Commands::RepairTextIndex => cli_handlers::cmd_repair_text_index(&args.db)?,

        Commands::Export { namespace, out } => {
            cli_handlers::cmd_export(&args.db, namespace.as_deref(), &out)?
        }

        Commands::Import { input } => cli_handlers::cmd_import(&args.db, &input, args.verbose)?,

        Commands::Query { query, limit } => {
            cli_handlers::cmd_query(&args.db, &query, limit, args.verbose)?
        }

        Commands::Search {
            namespace,
            query,
            query_vector,
            limit,
            json,
        } => cli_handlers::cmd_search(
            &args.db,
            &namespace,
            &query,
            query_vector.as_deref(),
            limit,
            json,
        )?,

        Commands::Delete { namespace, key } => {
            cli_handlers::cmd_delete(&args.db, &namespace, &key, args.verbose)?
        }

        Commands::DeleteByFilter { namespace, filter } => {
            cli_handlers::cmd_delete_by_filter(&args.db, &namespace, &filter, args.verbose)?
        }

        Commands::Count {
            namespace,
            filter,
            json,
        } => cli_handlers::cmd_count(&args.db, &namespace, filter.as_deref(), json, args.verbose)?,

        Commands::SimilarToKey {
            namespace,
            key,
            top_k,
            json,
        } => cli_handlers::cmd_similar_to_key(&args.db, &namespace, &key, top_k, json)?,

        Commands::SearchMulti {
            namespaces,
            query,
            query_vector,
            top_k,
            json,
        } => cli_handlers::search::cmd_search_multi(
            &args.db,
            &namespaces,
            query.as_deref(),
            query_vector.as_deref(),
            top_k,
            json,
        )?,

        Commands::SearchAll {
            query,
            query_vector,
            top_k,
            json,
        } => cli_handlers::search::cmd_search_all(
            &args.db,
            query.as_deref(),
            query_vector.as_deref(),
            top_k,
            json,
        )?,

        Commands::Namespace(cmd) => match cmd {
            vantadb::cli::NamespaceCommand::List => cli_handlers::cmd_namespace_list(&args.db)?,
            vantadb::cli::NamespaceCommand::Info { namespace } => {
                cli_handlers::cmd_namespace_info(&args.db, &namespace)?
            }
        },

        Commands::Migrate(cmd) => match cmd {
            vantadb::cli::MigrateCommand::Plan { target } => {
                cli_handlers::cmd_migrate_plan(&target, args.verbose)?
            }
            vantadb::cli::MigrateCommand::Run {
                target,
                format,
                dry_run,
                force,
            } => cli_handlers::cmd_migrate(&target, &format, dry_run, force, args.verbose)?,
            vantadb::cli::MigrateCommand::Check { target } => {
                cli_handlers::cmd_migrate_check(&target, args.verbose)?
            }
        },

        Commands::Status => cli_handlers::cmd_status(&args.db, args.verbose)?,

        Commands::Backup { out } => {
            cli_handlers::cmd_backup(&args.db, out.as_deref(), args.verbose)?
        }

        Commands::Restore {
            input,
            force,
            rebuild,
        } => cli_handlers::cmd_restore(&args.db, &input, force, rebuild, args.verbose)?,

        Commands::Doctor => cli_handlers::cmd_doctor(&args.db, args.verbose)?,

        Commands::Inspect { namespace, key } => {
            cli_handlers::cmd_inspect(&args.db, &namespace, &key, args.verbose)?
        }

        Commands::Stats { json } => cli_handlers::cmd_stats(&args.db, json, args.verbose)?,

        Commands::Snapshot(cmd) => match cmd {
            vantadb::cli::SnapshotCommand::Create { name } => {
                cli_handlers::cmd_snapshot_create(&args.db, &name, args.verbose)?
            }
            vantadb::cli::SnapshotCommand::List => cli_handlers::cmd_snapshot_list(&args.db)?,
        },

        Commands::Wal(cmd) => match cmd {
            vantadb::cli::WalCommand::Compact => cli_handlers::cmd_wal_compact(&args.db)?,
            vantadb::cli::WalCommand::Vacuum => cli_handlers::cmd_wal_vacuum(&args.db)?,
        },

        Commands::Completions { shell } => cli_handlers::cmd_completions(shell),

        Commands::Server {
            http,
            mcp,
            port,
            host,
            require_auth,
        } => cli_handlers::cmd_server(
            &args.db,
            http,
            mcp,
            port,
            host,
            require_auth,
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
