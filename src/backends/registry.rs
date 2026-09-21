//! `BackendRegistry`: OCP factory for `StorageBackend` construction (C2S2).
//!
//! Adding a new backend is additive: call `register()` with the new
//! `BackendKind` + factory. `StorageEngine::init_storage` (`init.rs`) only
//! calls `create()` — it never grows a new `match` arm again.
//!
//! Factories for feature-gated backends (`fjall`, `rocksdb`) return the same
//! honest `Error::Validation{ backend_feature }` as the old inline `match`
//! when their Cargo feature is disabled. No semantic change.

use std::collections::HashMap;
use std::sync::Arc;

use crate::backend::{BackendKind, StorageBackend};
use crate::config::Config;
use crate::error::{Error, Result};

/// Build a backend instance for `path` with `config`.
pub(crate) type BackendFactory = fn(&str, &Config) -> Result<Arc<dyn StorageBackend>>;

/// Composition-root registry: `BackendKind` → factory. No `match` inside.
pub(crate) struct BackendRegistry {
    factories: HashMap<BackendKind, BackendFactory>,
}

fn in_memory_factory(_path: &str, _config: &Config) -> Result<Arc<dyn StorageBackend>> {
    Ok(Arc::new(super::in_memory::InMemoryBackend::new()))
}

fn fjall_factory(path: &str, config: &Config) -> Result<Arc<dyn StorageBackend>> {
    #[cfg(feature = "fjall")]
    {
        Ok(Arc::new(super::fjall_backend::FjallBackend::open(
            path, config,
        )?))
    }
    #[cfg(not(feature = "fjall"))]
    {
        let _ = (path, config);
        Err(Error::Validation {
            field: "backend_feature".into(),
            reason: "Fjall backend requires the 'fjall' feature".into(),
        })
    }
}

fn rocksdb_factory(path: &str, config: &Config) -> Result<Arc<dyn StorageBackend>> {
    #[cfg(feature = "rocksdb")]
    {
        Ok(Arc::new(super::rocksdb_backend::RocksDbBackend::open(
            path, config,
        )?))
    }
    #[cfg(not(feature = "rocksdb"))]
    {
        let _ = (path, config);
        Err(Error::Validation {
            field: "backend_feature".into(),
            reason: "RocksDB backend requires the 'rocksdb' feature".into(),
        })
    }
}

impl BackendRegistry {
    /// Default composition: the 3 known backends. Called once per open.
    pub(crate) fn default_registry() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };
        registry.register(BackendKind::RocksDb, rocksdb_factory as BackendFactory);
        registry.register(BackendKind::Fjall, fjall_factory as BackendFactory);
        registry.register(BackendKind::InMemory, in_memory_factory as BackendFactory);
        registry
    }

    /// Register (or override) a backend factory. This is the OCP extension
    /// point: new backends arrive via this call, never via an `init.rs` edit.
    pub(crate) fn register(&mut self, kind: BackendKind, factory: BackendFactory) {
        self.factories.insert(kind, factory);
    }

    /// Build the backend for `kind`. No `match`: pure map lookup.
    pub(crate) fn create(
        &self,
        kind: BackendKind,
        path: &str,
        config: &Config,
    ) -> Result<Arc<dyn StorageBackend>> {
        let factory = self.factories.get(&kind).ok_or_else(|| Error::Validation {
            field: "backend_kind".into(),
            reason: format!("no factory registered for backend '{}'", kind.as_str()),
        })?;
        factory(path, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendPartition;
    use std::sync::atomic::{AtomicBool, Ordering};

    static FAKE_USED: AtomicBool = AtomicBool::new(false);

    /// Fictitious backend: proves extension without touching `init.rs`.
    /// Wraps `InMemoryBackend`; the flag proves the registry routed here.
    struct FakeBackend {
        inner: super::super::in_memory::InMemoryBackend,
    }

    impl crate::backend::Scannable for FakeBackend {
        fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
            self.inner.scan(partition)
        }
        fn scan_prefix_iter<'a>(
            &'a self,
            partition: BackendPartition,
            prefix: &'a [u8],
        ) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + 'a>> {
            self.inner.scan_prefix_iter(partition, prefix)
        }
    }

    impl StorageBackend for FakeBackend {
        fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
            self.inner.put(partition, key, value)
        }
        fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
            self.inner.get(partition, key)
        }
        fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
            self.inner.delete(partition, key)
        }
        fn write_batch(&self, ops: Vec<crate::backend::BackendWriteOp>) -> Result<()> {
            self.inner.write_batch(ops)
        }
        fn capabilities(&self) -> crate::backend::BackendCapabilities {
            self.inner.capabilities()
        }
    }

    fn fake_factory(_path: &str, _config: &Config) -> Result<Arc<dyn StorageBackend>> {
        FAKE_USED.store(true, Ordering::SeqCst);
        Ok(Arc::new(FakeBackend {
            inner: super::super::in_memory::InMemoryBackend::new(),
        }))
    }

    #[test]
    fn registry_creates_in_memory_without_touching_init() {
        // OCP proof part 1: the default composition builds a working
        // backend through the registry alone (`init.rs` not involved).
        let registry = BackendRegistry::default_registry();
        let backend = registry
            .create(BackendKind::InMemory, "", &Config::default())
            .unwrap();
        backend.put(BackendPartition::Default, b"k", b"v").unwrap();
        assert_eq!(
            backend.get(BackendPartition::Default, b"k").unwrap(),
            Some(b"v".to_vec())
        );
        assert_eq!(backend.capabilities().kind, BackendKind::InMemory);
    }

    #[test]
    fn registry_extension_is_additive_no_init_edit() {
        // OCP proof part 2: a fictitious backend registers on a fresh
        // registry and `create()` routes to it — zero edits to `init.rs`.
        FAKE_USED.store(false, Ordering::SeqCst);
        let mut registry = BackendRegistry::default_registry();
        registry.register(BackendKind::InMemory, fake_factory);
        let backend = registry
            .create(BackendKind::InMemory, "", &Config::default())
            .unwrap();
        assert!(
            FAKE_USED.load(Ordering::SeqCst),
            "registry must route to the newly registered factory"
        );
        backend
            .put(BackendPartition::Default, b"fk", b"fv")
            .unwrap();
        assert_eq!(
            backend.get(BackendPartition::Default, b"fk").unwrap(),
            Some(b"fv".to_vec())
        );
    }

    #[test]
    fn registry_unregistered_kind_errors_honestly() {
        let registry = BackendRegistry {
            factories: HashMap::new(),
        };
        let err = match registry.create(BackendKind::Fjall, "", &Config::default()) {
            Ok(_) => panic!("unregistered kind must fail"),
            Err(e) => e,
        };
        assert!(
            matches!(err, Error::Validation { .. }),
            "unregistered kind must be an honest Validation error"
        );
    }
}
