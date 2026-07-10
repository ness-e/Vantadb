use std::error::Error as StdError;
use std::fmt;
use thiserror::Error;

/// A serialization error that preserves both context and the original error.
///
/// Unlike `SerializationError(Box<dyn Error>)`, this variant adds a human-readable
/// message while keeping the underlying error chainable via `.source()`.
#[derive(Debug)]
pub struct SerdeMsgError {
    msg: String,
    source: Box<dyn StdError + Send + Sync>,
}

impl SerdeMsgError {
    /// Wrap a serialization error with additional context.
    pub fn new(ctx: impl fmt::Display, source: impl StdError + Send + Sync + 'static) -> Self {
        Self {
            msg: ctx.to_string(),
            source: Box::new(source),
        }
    }
}

impl fmt::Display for SerdeMsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.msg)
    }
}

impl StdError for SerdeMsgError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.source)
    }
}

/// Core error type for all VantaDB operations
#[derive(Error, Debug)]
pub enum VantaError {
    /// A node with the given ID was not found.
    #[error("Node not found: {0}")]
    NodeNotFound(u128),

    /// A node with the given ID already exists.
    #[error("Duplicate node ID: {0}")]
    DuplicateNode(u128),

    /// Vector dimensions do not match the expected value.
    #[error("Vector dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch {
        /// Expected vector dimension.
        expected: usize,
        /// Actual vector dimension received.
        got: usize,
    },

    /// Write-ahead log operation failed.
    #[error("WAL error: {0}")]
    WalError(String),

    /// WAL version does not match the expected version.
    #[error("WAL version mismatch: expected {expected}, found {found}. Hint: {hint}")]
    WALVersionMismatch {
        /// Expected WAL version number.
        expected: u32,
        /// Actual WAL version found.
        found: u32,
        /// Human-readable hint for resolution.
        hint: String,
    },

    /// Serialization or deserialization failure with source chaining.
    #[error("Serialization error: {0}")]
    SerializationError(#[source] Box<dyn StdError + Send + Sync>),

    /// Wrapped I/O error from the standard library.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Binary format magic bytes or version mismatch.
    #[error("Incompatible binary format: expected magic {expected_magic:?}, version {expected_version}, found magic {found_magic:?}, version {found_version}. Hint: {hint}")]
    IncompatibleFormat {
        /// Expected magic bytes.
        expected_magic: [u8; 4],
        /// Expected format version.
        expected_version: u16,
        /// Actual magic bytes found.
        found_magic: [u8; 4],
        /// Actual format version found.
        found_version: u16,
        /// Human-readable hint for resolution.
        hint: String,
    },

    /// Engine has not been initialised.
    #[error("Engine not initialized")]
    NotInitialized,

    /// A resource limit (e.g. memory) was exceeded.
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    /// Two nodes have colliding IDs.
    #[error("Node ID collision: {0}")]
    NodeIdCollision(u128),

    /// A cycle was detected in a graph operation.
    #[error("Cycle detected in graph operation")]
    CycleDetected,

    /// Parsing of an IQL query string failed.
    #[error("IQL parse error at line {line}, col {col}: {msg}")]
    IqlParseError {
        /// Parse error message.
        msg: String,
        /// Line number where the error occurred.
        line: usize,
        /// Column number where the error occurred.
        col: usize,
    },

    /// A requested entity (namespace, node, etc.) was not found.
    #[error("{kind} not found: {id}")]
    NotFound {
        /// Entity kind (e.g. "namespace").
        kind: String,
        /// Entity identifier.
        id: String,
    },

    /// Input validation failed.
    #[error("Validation error on {field}: {reason}")]
    ValidationError {
        /// Field that failed validation.
        field: String,
        /// Validation failure reason.
        reason: String,
    },

    /// An operation exceeded its time budget.
    #[error("Operation {operation} timed out after {duration_ms}ms")]
    Timeout {
        /// Name of the operation that timed out.
        operation: String,
        /// Timeout duration in milliseconds.
        duration_ms: u64,
    },

    /// Execution attempted an unsupported operation.
    #[error("Unsupported operation: {operation} — {detail}")]
    UnsupportedOperation {
        /// The unsupported operation name.
        operation: String,
        /// Explanation of why it is unsupported.
        detail: String,
    },

    /// Execution conflict (e.g. concurrent modification).
    #[error("Execution conflict on {resource}: {detail}")]
    ExecutionConflict {
        /// The resource involved in the conflict.
        resource: String,
        /// Details about the conflict.
        detail: String,
    },

    /// Error during IQL processing.
    #[error("IQL error: {0}")]
    IqlError(String),

    /// Error in CLI command processing.
    #[error("CLI error: {0}")]
    CliError(String),

    /// Error during search execution.
    #[error("Search error: {0}")]
    SearchError(String),

    /// Unexpected runtime error.
    #[error("Runtime error: {0}")]
    RuntimeError(String),

    /// Error during database restore.
    #[error("Restore error: {0}")]
    RestoreError(String),

    /// Error during database backup.
    #[error("Backup error: {0}")]
    BackupError(String),

    /// Generic catch-all error.
    #[error("Generic error: {0}")]
    Generic(String),

    /// Error from the storage backend.
    #[error("Backend error: {0}")]
    BackendError(String),

    /// Invalid input provided.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Schema-related error.
    #[error("Schema error: {0}")]
    SchemaError(String),

    /// Database is busy and cannot accept the operation.
    #[error("Database busy: {0}")]
    DatabaseBusy(String),
}

/// Crate-wide Result alias
pub type Result<T> = std::result::Result<T, VantaError>;

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn display_node_not_found() {
        let e = VantaError::NodeNotFound(42u128);
        assert_eq!(e.to_string(), "Node not found: 42");
    }

    #[test]
    fn display_duplicate_node() {
        let e = VantaError::DuplicateNode(99u128);
        assert_eq!(e.to_string(), "Duplicate node ID: 99");
    }

    #[test]
    fn display_dimension_mismatch() {
        let e = VantaError::DimensionMismatch {
            expected: 128,
            got: 64,
        };
        assert_eq!(
            e.to_string(),
            "Vector dimension mismatch: expected 128, got 64"
        );
    }

    #[test]
    fn display_wal_error() {
        let e = VantaError::WalError("corrupt crc".into());
        assert_eq!(e.to_string(), "WAL error: corrupt crc");
    }

    #[test]
    fn display_incompatible_format() {
        let e = VantaError::IncompatibleFormat {
            expected_magic: *b"VWAL",
            expected_version: 2,
            found_magic: *b"VNDX",
            found_version: 1,
            hint: "wrong file type".into(),
        };
        let s = e.to_string();
        assert!(s.contains("expected magic"), "should mention expected");
        assert!(s.contains("found magic"), "should mention found");
        assert!(s.contains("wrong file type"), "should include hint");
        assert!(s.contains("2"), "should mention expected version");
        assert!(s.contains("1"), "should mention found version");
    }

    #[test]
    fn display_engine_not_initialized() {
        let e = VantaError::NotInitialized;
        assert_eq!(e.to_string(), "Engine not initialized");
    }

    #[test]
    fn display_resource_limit() {
        let e = VantaError::ResourceLimit("too many requests".into());
        assert_eq!(e.to_string(), "Resource limit exceeded: too many requests");
    }

    #[test]
    fn display_node_id_collision() {
        let e = VantaError::NodeIdCollision(42u128);
        assert_eq!(e.to_string(), "Node ID collision: 42");
    }

    #[test]
    fn display_cycle_detected() {
        let e = VantaError::CycleDetected;
        assert_eq!(e.to_string(), "Cycle detected in graph operation");
    }

    #[test]
    fn display_iql_parse_error() {
        let e = VantaError::IqlParseError {
            msg: "unexpected token".into(),
            line: 3,
            col: 15,
        };
        assert_eq!(
            e.to_string(),
            "IQL parse error at line 3, col 15: unexpected token"
        );
    }

    #[test]
    fn display_not_found() {
        let e = VantaError::NotFound {
            kind: "namespace".into(),
            id: "my-ns".into(),
        };
        assert_eq!(e.to_string(), "namespace not found: my-ns");
    }

    #[test]
    fn display_validation_error() {
        let e = VantaError::ValidationError {
            field: "name".into(),
            reason: "cannot be empty".into(),
        };
        assert_eq!(e.to_string(), "Validation error on name: cannot be empty");
    }

    #[test]
    fn display_timeout() {
        let e = VantaError::Timeout {
            operation: "search".into(),
            duration_ms: 5000,
        };
        assert_eq!(e.to_string(), "Operation search timed out after 5000ms");
    }

    #[test]
    fn display_database_busy() {
        let e = VantaError::DatabaseBusy("lock held".into());
        assert_eq!(e.to_string(), "Database busy: lock held");
    }

    #[test]
    fn io_error_conversion() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let e: VantaError = io.into();
        assert_eq!(e.to_string(), "IO error: file not found");
    }

    #[test]
    fn debug_format() {
        let e = VantaError::NodeNotFound(7u128);
        let debug = format!("{:?}", e);
        assert!(
            debug.contains("NodeNotFound"),
            "Debug should contain variant name"
        );
        assert!(debug.contains("7"), "Debug should contain the value");
    }

    #[test]
    fn serde_msg_error_display() {
        let inner = std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid utf-8");
        let e = SerdeMsgError::new("text index decode error", inner);
        assert_eq!(e.to_string(), "text index decode error");
    }

    #[test]
    fn serde_msg_error_source() {
        let inner = std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid utf-8");
        let e = SerdeMsgError::new("text index decode error", inner);
        let source = e.source().unwrap();
        assert_eq!(source.to_string(), "invalid utf-8");
    }

    #[test]
    fn serde_msg_error_into_vanta_error() {
        let inner = std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid utf-8");
        let e = VantaError::SerializationError(Box::new(SerdeMsgError::new(
            "text index decode error",
            inner,
        )));
        assert!(e.to_string().contains("text index decode error"));
        assert!(e.source().is_some());
        let source_msg = e.source().unwrap().to_string();
        assert_eq!(source_msg, "text index decode error");
    }

    #[test]
    fn serialization_error_source_plain() {
        let inner = postcard::Error::SerdeSerCustom;
        let e = VantaError::SerializationError(Box::new(inner));
        assert!(e.source().is_some());
    }
}
