//! Concrete `StorageBackend` implementations.

#[cfg(feature = "fjall")]
pub(crate) mod fjall_backend;
pub(crate) mod in_memory;
#[cfg(feature = "encryption")]
pub(crate) mod kv_crypto;
pub(crate) mod registry;
#[cfg(feature = "rocksdb")]
pub(crate) mod rocksdb_backend;
