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
    pub command: Commands,
}

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

    /// Generate shell completion scripts
    Completions {
        /// Shell type for the completion script
        #[arg(long, value_enum)]
        shell: Shell,
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
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
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
