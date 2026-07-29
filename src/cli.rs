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

    /// Export records to a JSON file
    Export {
        /// Optional namespace to export (exports all if not specified)
        #[arg(long)]
        namespace: Option<String>,
        /// Output file path
        #[arg(long)]
        out: String,
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
    Doctor,

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
