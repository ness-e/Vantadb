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
    /// Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or './db' if neither is set.
    #[arg(short, long, env = "VANTA_DB", default_value = "./db", global = true)]
    pub db: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Optional memory limit for the database engine, in bytes.
    /// Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`.
    /// Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable.
    #[arg(long, env = "VANTADB_MEMORY_LIMIT", global = true)]
    pub memory_limit: Option<String>,

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
        /// Output results as JSON
        #[arg(long)]
        json: bool,
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
        /// Input file path
        #[arg(long, name = "in")]
        input: String,
    },

    /// Execute a structured query (IQL/hybrid)
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
        /// Output directory for the backup (default: vantadb_backups/backup_<timestamp>)
        #[arg(long)]
        out: Option<String>,
    },

    /// Restore the database from a previously created backup directory
    Restore {
        /// Path to the backup directory
        #[arg(long)]
        input: String,
        /// Overwrite existing database directory if it exists
        #[arg(long)]
        force: bool,
        /// Rebuild indexes after restore
        #[arg(long)]
        rebuild: bool,
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
    Stats {
        /// Output statistics as JSON
        #[arg(long)]
        json: bool,
    },

    /// Launch the interactive TUI (requires `tui` feature)
    #[cfg(feature = "tui")]
    Tui,

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
        /// Text query for semantic/hybrid search
        #[arg(long)]
        query: String,
        /// Optional explicit vector query (comma-separated f32 values)
        #[arg(long)]
        query_vector: Option<String>,
        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Delete a record by namespace and key
    Delete {
        /// Namespace of the record
        #[arg(long)]
        namespace: String,
        /// Key of the record to delete
        #[arg(long)]
        key: String,
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

    /// Count records in a namespace, optionally filtered by metadata
    Count {
        /// Namespace to count records in
        #[arg(long)]
        namespace: String,
        /// Optional JSON filter (same format as delete-by-filter)
        #[arg(long)]
        filter: Option<String>,
        /// Output as raw number only
        #[arg(long)]
        json: bool,
    },

    /// Find records similar to a given key using vector similarity search
    SimilarToKey {
        /// Namespace of the reference record
        #[arg(long)]
        namespace: String,
        /// Key of the reference record
        #[arg(long)]
        key: String,
        /// Number of similar records to return
        #[arg(long, default_value = "10")]
        top_k: usize,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
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
        /// Text query for hybrid/lexical search
        #[arg(long)]
        query: Option<String>,
        /// Optional explicit vector query (comma-separated f32 values)
        #[arg(long)]
        query_vector: Option<String>,
        /// Maximum number of results across all namespaces
        #[arg(long, default_value = "10")]
        top_k: usize,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Search across ALL known namespaces and merge results by score
    SearchAll {
        /// Text query for hybrid/lexical search
        #[arg(long)]
        query: Option<String>,
        /// Optional explicit vector query (comma-separated f32 values)
        #[arg(long)]
        query_vector: Option<String>,
        /// Maximum number of results across all namespaces
        #[arg(long, default_value = "10")]
        top_k: usize,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
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
pub enum MigrateCommand {
    /// Plan migrations that would be performed
    Plan {
        /// Path to the database directory
        target: String,
    },
    /// Run migrations to bring formats up to date
    Run {
        /// Path to the database directory
        target: String,
        /// Specific format to migrate (vfile, index, wal, schema, all)
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
        /// Path to the database directory
        target: String,
    },
}

/// Subcommands for Write-Ahead Log management
#[derive(Subcommand, Debug, Clone)]
pub enum WalCommand {
    /// Compact the WAL: flush all data, archive the current WAL file, and start a fresh one
    Compact,
    /// Remove tombstoned nodes from HNSW and reclaim space
    Vacuum,
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
