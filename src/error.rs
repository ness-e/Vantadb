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

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Database busy: {0}")]
    DatabaseBusy(String),
}

/// Crate-wide Result alias
pub type Result<T> = std::result::Result<T, VantaError>;
