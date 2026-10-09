//! VantaDB CLI Arguments - Shareable definitions for CLI binary and build.rs
//!
//! Exposes the struct definitions and command enums required for parsing.

use clap::{Parser, Subcommand, ValueEnum};

/// VantaDB CLI - Embedded persistent memory and vector retrieval engine
#[derive(Parser, Debug)]
#[command(name = "vanta-cli")]
#[command(author = "VantaDB Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "CLI for interacting with VantaDB", long_about = None)]
pub struct Cli {
    /// Path to the database directory. Defaults to the value of the VANTADB_STORAGE_PATH environment variable, or './db' if neither is set.
    #[arg(
        short,
        long,
        env = "VANTADB_STORAGE_PATH",
        default_value = "./db",
        global = true
    )]
    pub db: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Optional memory limit for the database engine, in bytes.
    /// Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`.
    /// Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable.
    #[arg(long, env = "VANTADB_MEMORY_LIMIT", global = true)]
    pub memory_limit: Option<String>,

    /// Output complete, machine-readable JSON on stdout (never truncated).
    /// Errors still go to stderr and exit codes are unchanged. Applies to
    /// every subcommand that produces data output.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    /// The subcommand to execute
    pub command: Commands,
}

/// All supported CLI subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Save a key-value pair to persistent memory
    Put {
        /// Namespace for the key
        #[arg(long)]
        namespace: String,
        /// Key to store the value under
        #[arg(long)]
        key: String,
        /// Value to store (payload text)
        #[arg(long)]
        payload: String,
        /// Optional vector embedding (comma-separated f32 values)
        #[arg(long)]
        vector: Option<String>,
        /// Optional metadata as a JSON object, e.g. '{"k":"v","n":1}'
        #[arg(long)]
        metadata: Option<String>,
    },

    /// Retrieve a value from persistent memory
    Get {
        /// Namespace for the key
        #[arg(long)]
        namespace: String,
        /// Key to retrieve the value for
        #[arg(long)]
        key: String,
    },

    /// List keys and values in a namespace
    List {
        /// Namespace to list
        #[arg(long)]
        namespace: String,
        /// Maximum number of records to return
        #[arg(long, default_value = "100")]
        limit: usize,
    },

    /// Rebuild all database indexes (HNSW, text index, derived indexes)
    RebuildIndex,

    /// Validate text index integrity without repairing
    AuditIndex {
        /// Optional namespace to audit (audits all if not specified)
        #[arg(long)]
        namespace: Option<String>,
        /// Perform deep structural validation
        #[arg(long)]
        deep: bool,
    },

    /// Repair text index if inconsistencies are detected
    RepairTextIndex,

    /// Export records to a file (jsonl) or to a directory of Markdown files
    /// with JSON frontmatter (--format md, git-friendly, round-trips with
    /// `vanta-seed import-md`).
    Export {
        /// Optional namespace to export (exports all if not specified)
        #[arg(long)]
        namespace: Option<String>,
        /// Output path: file for `--format jsonl` (default), directory for
        /// `--format md`.
        #[arg(long)]
        out: String,
        /// Export format. `jsonl` writes one record per line to a file
        /// (default, backwards-compatible). `md` writes one file per record
        /// under `<out>/<namespace>/<key>.md` with JSON frontmatter; the
        /// directory is git-friendly and round-trips with
        /// `vanta-seed import-md`.
        #[arg(long, value_enum, default_value_t = ExportFormat::Jsonl)]
        format: ExportFormat,
    },

    /// Import records from a JSON file
    Import {
        /// Input file path (`--in`; `--input` kept as a hidden alias)
        #[arg(long = "in", alias = "input")]
        input: String,
    },

    /// Execute a structured query (IQL/hybrid)
    ///
    /// Read statements (`SELECT`, `FROM`/`MATCH`) open the database
    /// read-only; mutating statements (`INSERT`/`INSERT MESSAGE`, `UPDATE`,
    /// `DELETE`, `RELATE`) open it read-write.
    Query {
        /// Query string
        query: String,
        /// Maximum results to return
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// Display database health diagnostics and system status
    Status,

    /// Create a filesystem-level backup of the database directory
    Backup {
        /// Output directory for the backup (default: `vantadb_backups/backup_<timestamp>`)
        #[arg(long)]
        out: Option<String>,
    },

    /// Restore the database from a previously created backup directory
    Restore {
        /// Path to the backup directory (`--in`; `--input` kept as a hidden alias)
        #[arg(long = "in", alias = "input")]
        input: String,
        /// Overwrite existing database directory if it exists
        #[arg(long)]
        force: bool,
        /// Rebuild indexes after restore
        #[arg(long)]
        rebuild: bool,
        /// Validate the backup without restoring (dry-run).
        /// Lists what would be restored and exits 0 without touching the target.
        #[arg(long)]
        dry_run: bool,
    },

    /// Run comprehensive health diagnostics on the database
    Doctor {
        /// Apply safe repairs (create missing data directories).
        /// Without --force this only lists what would be fixed (dry-run).
        #[arg(long)]
        fix: bool,
        /// Actually apply the repairs listed by --fix (without it --fix is a dry-run).
        #[arg(long)]
        force: bool,
    },

    /// Inspect a single record showing all fields, vectors, and metadata
    Inspect {
        /// Namespace of the record
        #[arg(long)]
        namespace: String,
        /// Key of the record to inspect
        #[arg(long)]
        key: String,
    },

    /// Display detailed database statistics in human-readable or JSON format
    Stats,

    /// Launch the interactive TUI (requires `tui` feature)
    #[cfg(feature = "tui")]
    Tui,

    /// Verify the WAL hash-chain integrity (tamper-evident, VER-01): detects
    /// altered or removed records with their exact position. Read-only and
    /// offline — no engine open. Exit code ≠0 when integrity fails.
    Verify,

    /// Generate shell completion scripts
    Completions {
        /// Shell type for the completion script
        #[arg(long, value_enum)]
        shell: Shell,
    },

    /// Search records semantically across a namespace
    Search {
        /// Namespace to search within
        #[arg(long)]
        namespace: String,
        /// Text query (positional operand; `--query` remains as a hidden alias)
        #[arg(value_name = "QUERY", required_unless_present = "query_flag")]
        query: Option<String>,
        /// Hidden alias for the positional QUERY (kept for existing scripts)
        #[arg(long = "query", hide = true, conflicts_with = "query")]
        query_flag: Option<String>,
        /// Optional explicit vector query (comma-separated f32 values)
        #[arg(long)]
        query_vector: Option<String>,
        /// Maximum number of results
        #[arg(long, default_value = "10", alias = "top-k")]
        limit: usize,
    },

    /// Delete a record by namespace and key
    Delete {
        /// Namespace of the record
        #[arg(long)]
        namespace: String,
        /// Key of the record to delete
        #[arg(long)]
        key: String,
        /// Emit a purge certificate (VER-02): per-surface residue inventory +
        /// integrity hash + VER-01 WAL chain reference (JSON on stdout)
        #[arg(long, default_value_t = false)]
        attest: bool,
        /// With --attest: write the certificate to this file (UTF-8, written
        /// by the CLI itself — preferred over shell redirection on Windows)
        #[arg(long, requires = "attest")]
        out: Option<String>,
    },

    /// Delete all records in a namespace matching a JSON metadata filter
    DeleteByFilter {
        /// Namespace to operate on
        #[arg(long)]
        namespace: String,
        /// JSON filter in MongoDB-like format, e.g. '{"field": {"$op": value}}'
        /// Operators: $eq, $neq, $gt, $gte, $lt, $lte
        /// Example: '{"status": {"$eq": "inactive"}}'
        #[arg(long)]
        filter: String,
    },

    /// Verification of purge certificates (VER-02)
    #[command(subcommand)]
    Certificate(CertificateCommand),

    /// Count records in a namespace, optionally filtered by metadata
    Count {
        /// Namespace to count records in
        #[arg(long)]
        namespace: String,
        /// Optional JSON filter (same format as delete-by-filter)
        #[arg(long)]
        filter: Option<String>,
    },

    /// Find records similar to a given key using vector similarity search
    SimilarToKey {
        /// Namespace of the reference record
        #[arg(long)]
        namespace: String,
        /// Key of the reference record
        #[arg(long)]
        key: String,
        /// Number of similar records to return (`--top-k` kept as a hidden alias)
        #[arg(long, default_value = "10", alias = "top-k")]
        limit: usize,
    },

    /// Migrate a database to the latest storage schema version
    #[command(subcommand)]
    Migrate(MigrateCommand),

    /// Manage namespaces
    #[command(subcommand)]
    Namespace(NamespaceCommand),

    /// Manage instant filesystem snapshots
    #[command(subcommand)]
    Snapshot(SnapshotCommand),

    /// Manage the Write-Ahead Log (compact, vacuum)
    #[command(subcommand)]
    Wal(WalCommand),

    /// Search across multiple namespaces and merge results by score
    SearchMulti {
        /// Comma-separated list of namespaces to search (e.g. "ns1,ns2,ns3")
        #[arg(long)]
        namespaces: String,
        /// Text query (optional positional operand; `--query` remains as a hidden alias)
        #[arg(value_name = "QUERY")]
        query: Option<String>,
        /// Hidden alias for the positional QUERY (kept for existing scripts)
        #[arg(long = "query", hide = true, conflicts_with = "query")]
        query_flag: Option<String>,
        /// Optional explicit vector query (comma-separated f32 values)
        #[arg(long)]
        query_vector: Option<String>,
        /// Maximum number of results across all namespaces (`--top-k` hidden alias)
        #[arg(long, default_value = "10", alias = "top-k")]
        limit: usize,
    },

    /// Search across ALL known namespaces and merge results by score
    SearchAll {
        /// Text query (optional positional operand; `--query` remains as a hidden alias)
        #[arg(value_name = "QUERY")]
        query: Option<String>,
        /// Hidden alias for the positional QUERY (kept for existing scripts)
        #[arg(long = "query", hide = true, conflicts_with = "query")]
        query_flag: Option<String>,
        /// Optional explicit vector query (comma-separated f32 values)
        #[arg(long)]
        query_vector: Option<String>,
        /// Maximum number of results across all namespaces (`--top-k` hidden alias)
        #[arg(long, default_value = "10", alias = "top-k")]
        limit: usize,
    },

    /// Start the HTTP or MCP server wrapper
    Server {
        /// Start HTTP server wrapper (default)
        #[arg(long)]
        http: bool,

        /// Start MCP server wrapper over stdio
        #[arg(long)]
        mcp: bool,

        /// Port for the HTTP server
        #[arg(long, short, env = "VANTADB_PORT")]
        port: Option<u16>,

        /// Host for the HTTP server
        #[arg(long, env = "VANTADB_HOST")]
        host: Option<String>,

        /// Force authentication: refuse to start without an API key
        #[arg(long, env = "VANTADB_REQUIRE_AUTH")]
        require_auth: bool,

        /// Allow binding a non-loopback host without an API key (dev only).
        /// The server logs a prominent security warning and starts unauthenticated.
        #[arg(long)]
        allow_insecure: bool,

        /// Directory of static files to serve at /dashboard (Vanta Studio web
        /// console). When unset, /dashboard responds 404 with a hint.
        #[arg(long, env = "VANTADB_DASHBOARD_DIR")]
        dashboard_dir: Option<String>,
    },

    /// Call one MCP tool through a one-shot stdio server (no pwsh needed).
    ///
    /// Spawns `vantadb-server --mcp` (same binary resolution as
    /// `server --mcp`), sends `initialize` + `tools/call` as line-delimited
    /// JSON-RPC, and prints the tool `result` verbatim to stdout.
    /// Secrets are inherited from the session env, never CLI args.
    /// Exit codes: 0 ok · 1 infra (spawn/io/timeout/protocol) ·
    /// 2 tool-level error (MCP error or `isError` result) — exit 2 also
    /// covers invalid client-side input (`--args`/placeholder/stdin errors),
    /// which never reaches the tool.
    McpCall {
        /// Tool name, e.g. `memory_recall` or `thread_send`
        #[arg(long)]
        tool: String,
        /// Tool arguments as a JSON object string, e.g. `'{"query":"x"}'`.
        /// Hook templates may splice client event fields as
        /// `{{dotted.path}}` (e.g. `'{"query":"{{prompt}}"}'`), resolved
        /// against the hook-input JSON on stdin. Stdin is read only when
        /// the template has placeholders.
        #[arg(long, default_value = "{}")]
        args: String,
        /// Seconds to wait for the server response before failing
        #[arg(long, default_value = "60")]
        timeout_secs: u64,
    },
}

/// Subcommands for namespace management
#[derive(Subcommand, Debug, Clone)]
pub enum NamespaceCommand {
    /// List all namespaces
    List,
    /// Show record count and details for a namespace
    Info {
        /// Namespace to inspect
        namespace: String,
    },
}

/// Subcommands for filesystem snapshots
#[derive(Subcommand, Debug, Clone)]
pub enum SnapshotCommand {
    /// Create an instant filesystem snapshot by hard-linking all data files
    Create {
        /// Name for the snapshot
        name: String,
    },
    /// List all existing snapshots
    List,
}

/// Subcommands for database migration
#[derive(Subcommand, Debug, Clone)]
#[command(after_help = "\
Examples:
  vanta-cli migrate check ./my-db         # positional target
  vanta-cli migrate check --db ./my-db    # global --db fallback
  vanta-cli migrate run ./my-db --dry-run # preview without modifying")]
pub enum MigrateCommand {
    /// Plan migrations that would be performed
    Plan {
        /// Path to the database directory (defaults to the global `--db`)
        target: Option<String>,
    },
    /// Run migrations to bring formats up to date
    Run {
        /// Path to the database directory (defaults to the global `--db`)
        target: Option<String>,
        /// Specific format to migrate (all, vfile, index, wal, records, schema)
        #[arg(long, default_value = "all")]
        format: String,
        /// Preview changes without modifying files
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Skip confirmation prompts
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Check storage integrity for all formats
    Check {
        /// Path to the database directory (defaults to the global `--db`)
        target: Option<String>,
    },
}

/// Subcommands for Write-Ahead Log management
#[derive(Subcommand, Debug, Clone)]
pub enum WalCommand {
    /// Compact the WAL: flush all data, archive the current WAL file, and start a fresh one
    Compact,
    /// Remove tombstoned nodes from HNSW and reclaim space
    Vacuum,
    /// Salvage a truncated sharded WAL (FIND-109, opt-in): replay the coherent
    /// prefix and report explicit discards. The ERR-011 guard still aborts
    /// normal opens; this command is the explicit repair path.
    Salvage {
        /// Preview only: report what would be kept/discarded without mutating.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
}

/// Subcommands for purge certificates (VER-02)
#[derive(Subcommand, Debug, Clone)]
pub enum CertificateCommand {
    /// Verify a stored purge certificate: integrity hash + live re-scan of
    /// the re-checkable surfaces (store, shred, vector index, versions).
    /// Exit code ≠0 when the certificate is invalid or residues reappeared.
    Verify {
        /// Path to the certificate JSON file (emitted by `delete --attest`)
        #[arg(long)]
        file: String,
    },
}

/// Shell type for shell completion scripts
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum Shell {
    /// Bash shell completions
    Bash,
    /// Zsh shell completions
    Zsh,
    /// Fish shell completions
    Fish,
    /// PowerShell shell completions
    #[value(name = "powershell", alias = "power-shell")]
    PowerShell,
}

/// Output format for the `export` subcommand.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ExportFormat {
    /// JSONL on stdout/file (default; backwards compatible).
    #[default]
    Jsonl,
    /// One Markdown file per record under `<out>/<namespace>/<key>.md` with
    /// JSON frontmatter metadata; round-trips with `vanta-seed import-md`.
    Md,
}

impl From<Shell> for clap_complete::Shell {
    fn from(shell: Shell) -> Self {
        match shell {
            Shell::Bash => clap_complete::Shell::Bash,
            Shell::Zsh => clap_complete::Shell::Zsh,
            Shell::Fish => clap_complete::Shell::Fish,
            Shell::PowerShell => clap_complete::Shell::PowerShell,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: `migrate run --format` help must list every accepted format.
    /// The canonical list is `FormatKind` (plus `all`), same as the runtime
    /// error in `cli_handlers/migrate.rs` — `records` was missing (SCH-08 FIND).
    #[test]
    fn migrate_run_help_lists_every_format() {
        let err = Cli::try_parse_from(["vanta-cli", "migrate", "run", "--help"])
            .expect_err("--help must exit with a DisplayHelp error");
        let help = err.to_string();
        for format in crate::migration::FormatKind::all() {
            assert!(
                help.contains(format.name()),
                "`migrate run --help` must list format `{}`:\n{help}",
                format.name()
            );
        }
        assert!(
            help.contains("all"),
            "`migrate run --help` must list `all`:\n{help}"
        );
    }

    /// FIND-237: `migrate check --db <db>` must parse — the global `--db` is
    /// the fallback when the positional TARGET is omitted.
    #[test]
    fn migrate_check_accepts_global_db_without_positional() {
        let cli = Cli::try_parse_from(["vanta-cli", "migrate", "check", "--db", "./some-db"])
            .expect("`migrate check --db` must parse (global --db fallback)");
        assert_eq!(cli.db, "./some-db");
        match cli.command {
            Commands::Migrate(MigrateCommand::Check { target }) => {
                assert_eq!(target, None, "no positional target was given");
            }
            other => panic!("expected `migrate check`, got {other:?}"),
        }
    }

    /// FIND-237: the positional TARGET keeps winning when both forms are present.
    #[test]
    fn migrate_check_positional_target_wins_over_global_db() {
        let cli = Cli::try_parse_from([
            "vanta-cli",
            "migrate",
            "check",
            "./positional-db",
            "--db",
            "./global-db",
        ])
        .expect("both target forms together must parse");
        match cli.command {
            Commands::Migrate(MigrateCommand::Check { target }) => {
                assert_eq!(target.as_deref(), Some("./positional-db"));
            }
            other => panic!("expected `migrate check`, got {other:?}"),
        }
    }

    /// FIND-237: plan/run follow the same convention (no positional required).
    #[test]
    fn migrate_plan_and_run_accept_global_db_without_positional() {
        let plan = Cli::try_parse_from(["vanta-cli", "migrate", "plan", "--db", "./db"])
            .expect("`migrate plan --db` must parse");
        assert!(matches!(
            plan.command,
            Commands::Migrate(MigrateCommand::Plan { target: None })
        ));

        let run = Cli::try_parse_from(["vanta-cli", "migrate", "run", "--db", "./db"])
            .expect("`migrate run --db` must parse");
        assert!(matches!(
            run.command,
            Commands::Migrate(MigrateCommand::Run { target: None, .. })
        ));
    }

    /// FIND-237: `migrate --help` shows a usage example with the --db fallback.
    #[test]
    fn migrate_help_shows_db_fallback_example() {
        let err = Cli::try_parse_from(["vanta-cli", "migrate", "--help"])
            .expect_err("--help must exit with a DisplayHelp error");
        let help = err.to_string();
        assert!(
            help.contains("migrate check --db"),
            "`migrate --help` must show the --db fallback example:\n{help}"
        );
    }
}
