use thiserror::Error;

/// Core error type for all IADBMS operations
#[derive(Error, Debug)]
pub enum IadbmsError {
    #[error("Node not found: {0}")]
    NodeNotFound(u64),

    #[error("Duplicate node ID: {0}")]
    DuplicateNode(u64),

    #[error("Vector dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    #[error("WAL error: {0}")]
    WalError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Engine not initialized")]
    NotInitialized,

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    #[error("Execution error: {0}")]
    Execution(String),
}

/// Crate-wide Result alias
pub type Result<T> = std::result::Result<T, IadbmsError>;
