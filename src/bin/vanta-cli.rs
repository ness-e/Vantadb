//! VantaDB CLI binary — thin entry point.
//! Handlers live in `vantadb::cli_handlers` for testability.

use clap::Parser;

use vantadb::cli::{Cli, Commands};
use vantadb::cli_handlers;
use vantadb::error::Result;

#[cfg(feature = "custom-allocator")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<()> {
    let args = Cli::parse();

    if args.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
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
        } => cli_handlers::cmd_audit_index(
            &args.db,
            namespace.as_deref(),
            json,
            deep,
        )?,

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
            limit,
        } => cli_handlers::cmd_search(&args.db, &namespace, &query, limit)?,

        Commands::Delete { namespace, key } => {
            cli_handlers::cmd_delete(&args.db, &namespace, &key, args.verbose)?
        }

        Commands::Namespace(cmd) => match cmd {
            vantadb::cli::NamespaceCommand::List => cli_handlers::cmd_namespace_list(&args.db)?,
            vantadb::cli::NamespaceCommand::Info { namespace } => {
                cli_handlers::cmd_namespace_info(&args.db, &namespace)?
            }
        },

        Commands::Status => cli_handlers::cmd_status(&args.db, args.verbose)?,

        Commands::Completions { shell } => cli_handlers::cmd_completions(shell),

        Commands::Server {
            http,
            mcp,
            port,
            host,
        } => cli_handlers::cmd_server(&args.db, http, mcp, port, host, args.verbose)?,
    }

    Ok(())
}
