use thiserror::Error;

/// Core error type for all VantaDB operations
#[derive(Error, Debug)]
pub enum VantaError {
    #[error("Node not found: {0}")]
    NodeNotFound(u64),

    #[error("Duplicate node ID: {0}")]
    DuplicateNode(u64),

    #[error("Vector dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    #[error("WAL error: {0}")]
    WalError(String),

    #[error("WAL version mismatch: expected {expected}, found {found}. Hint: {hint}")]
    WALVersionMismatch {
        expected: u32,
        found: u32,
        hint: String,
    },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Incompatible binary format: expected magic {expected_magic:?}, version {expected_version}, found magic {found_magic:?}, version {found_version}. Hint: {hint}")]
    IncompatibleFormat {
        expected_magic: [u8; 4],
        expected_version: u16,
        found_magic: [u8; 4],
        found_version: u16,
        hint: String,
    },

    #[error("Engine not initialized")]
    NotInitialized,

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    #[error("Node ID collision: {0}")]
    NodeIdCollision(u64),

    #[error("Cycle detected in graph operation")]
    CycleDetected,

    #[error("IQL parse error at line {line}, col {col}: {msg}")]
    IqlParseError {
        msg: String,
        line: usize,
        col: usize,
    },

    #[error("{kind} not found: {id}")]
    NotFound { kind: String, id: String },

    #[error("Validation error on {field}: {reason}")]
    ValidationError { field: String, reason: String },

    #[error("Operation {operation} timed out after {duration_ms}ms")]
    Timeout { operation: String, duration_ms: u64 },

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("IQL error: {0}")]
    IqlError(String),

    #[error("CLI error: {0}")]
    CliError(String),

    #[error("Search error: {0}")]
    SearchError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Restore error: {0}")]
    RestoreError(String),

    #[error("Backup error: {0}")]
    BackupError(String),

    #[error("Generic error: {0}")]
    Generic(String),

    #[error("Serialization error: {0}")]
    #[deprecated(
        note = "Use the non-deprecated SerializationError variant or a more specific variant"
    )]
    OldSerializationError(String),

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("Database busy: {0}")]
    DatabaseBusy(String),
}

/// Crate-wide Result alias
pub type Result<T> = std::result::Result<T, VantaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_node_not_found() {
        let e = VantaError::NodeNotFound(42);
        assert_eq!(e.to_string(), "Node not found: 42");
    }

    #[test]
    fn display_duplicate_node() {
        let e = VantaError::DuplicateNode(99);
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
        let e = VantaError::NodeIdCollision(42);
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
        let e = VantaError::NodeNotFound(7);
        let debug = format!("{:?}", e);
        assert!(
            debug.contains("NodeNotFound"),
            "Debug should contain variant name"
        );
        assert!(debug.contains("7"), "Debug should contain the value");
    }
}
