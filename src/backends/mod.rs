//! Concrete `StorageBackend` implementations.

#[cfg(feature = "fjall")]
pub(crate) mod fjall_backend;
pub(crate) mod in_memory;
#[cfg(feature = "rocksdb")]
pub(crate) mod rocksdb_backend;
