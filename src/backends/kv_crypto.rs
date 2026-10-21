//! Encryption at rest for the backend key-value store (ADR-0057 W4).
//!
//! The backend KV is the one artefact whose **keys must stay in cleartext**:
//! the text index needs prefix scans per namespace and the namespace index
//! needs ordered lookups (ADR-0057 §5.3). The **values** are therefore
//! encrypted, one AEAD envelope per value:
//!
//! ```text
//! stored value = [12B nonce][ct ‖ 16B tag]
//! ```
//!
//! * The **AAD** binds the value to its public identifier
//!   `partition ‖ 0x00 ‖ key` (slot = 0), so a value moved to another key
//!   does not authenticate (ADR-0057 D4).
//! * The nonce is `epoch ‖ counter`; the epoch is persisted **before** the
//!   first encryption under it (ADR-0057 D3).
//! * The **mode marker** is the registry: the backend KV is encrypted iff the
//!   registry holds an entry for the `backend` artefact. The backend KV has
//!   no header of its own (ADR-0057 D1), so the registry — the mechanism that
//!   already exists — is the single source of truth; no second version file
//!   is invented. The envelope's format version travels in the AAD
//!   (authenticated), exactly like the WAL's v4 and the VantaFile's v3.
//!
//! Without a key the backend is **not** wrapped at all: the bytes on disk are
//! byte-identical to the layout before this change.

use crate::backend::{
    BackendCapabilities, BackendPartition, BackendWriteOp, Compactable, Scannable, Snapshotable,
    StorageBackend,
};
use crate::config::Config;
use crate::crypto::registry::{AadBuilder, Artifact, KeyRegistry, NonceAllocator};
use crate::crypto::{Cipher, CryptoError, Nonce};
use crate::error::{Error, Result};
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;

/// The artefact id used in the error messages of this module.
pub(crate) const ARTIFACT: &str = "backend";

/// The version of the encrypted-value envelope.
///
/// It travels in the AAD (authenticated), not in a sidecar: the backend KV
/// has no header of its own (ADR-0057 D1).
pub(crate) const BACKEND_KV_FORMAT_VERSION: u16 = 1;

/// Length of the AEAD nonce (ADR-0057 D3).
const NONCE_LEN: usize = 12;

/// Length of the AEAD tag.
const TAG_LEN: usize = 16;

/// The encryption context of the `backend` artefact: one DEK, one nonce
/// space, shared by every partition and every key of the database.
///
/// The context is resolved **once**, at the open of the backend, and handed
/// down to the operations: the hot path takes **no new lock** (the nonce
/// counter is an `AtomicU32`; the registry is locked only to roll the epoch,
/// which happens when the counter is exhausted — never in the hot path).
pub(crate) struct BackendKvCrypto {
    /// The DEK cipher of the artefact.
    cipher: Cipher,
    /// The nonce allocator of the current epoch. `None` in a read-only
    /// context: a read-only open never allocates an epoch, because that
    /// would write the registry.
    allocator: Option<arc_swap::ArcSwapAny<Arc<NonceAllocator>>>,
    /// The registry — locked only to roll the epoch (never in the hot path).
    registry: Option<Arc<Mutex<KeyRegistry>>>,
}

// The context holds no secret in a `Debug` view: the `Cipher` redacts its
// key material and the registry redacts the DEKs.
impl std::fmt::Debug for BackendKvCrypto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendKvCrypto")
            .field("writable", &self.allocator.is_some())
            .finish()
    }
}

impl BackendKvCrypto {
    /// Open the **write-side** context of the `backend` artefact in `data_dir`.
    ///
    /// Allocates a **fresh epoch** and persists it **before** the first
    /// encryption under it (ADR-0057 D3).
    ///
    /// # Errors
    ///
    /// [`Error::EncryptionKeyRejected`] when the master key does not unwrap
    /// the registry (wrong key **or** tampered registry) and
    /// [`Error::EncryptionCorrupt`] when the registry is malformed.
    pub(crate) fn open(data_dir: &Path, key_material: &[u8]) -> Result<Self> {
        let mut registry =
            KeyRegistry::open_or_create(data_dir, key_material).map_err(crypto_error)?;
        registry
            .ensure_dek(Artifact::BackendKv)
            .map_err(crypto_error)?;
        registry
            .begin_epoch(Artifact::BackendKv)
            .map_err(crypto_error)?;
        let allocator = registry
            .allocator(Artifact::BackendKv)
            .cloned()
            .ok_or_else(|| {
                Error::generic_error(
                    "encryption registry has no nonce allocator for the backend artefact",
                )
            })?;
        let dek =
            registry
                .dek(Artifact::BackendKv)
                .ok_or_else(|| Error::EncryptionKeyRequired {
                    artifact: ARTIFACT.to_string(),
                })?;
        Ok(Self {
            cipher: Cipher::new(&dek[..]),
            allocator: Some(arc_swap::ArcSwapAny::new(allocator)),
            registry: Some(Arc::new(Mutex::new(registry))),
        })
    }

    /// Open the **read-side** context of the `backend` artefact in `data_dir`.
    ///
    /// A read-only open must not write anything, so it neither allocates an
    /// epoch nor persists the registry.
    ///
    /// # Errors
    ///
    /// [`Error::EncryptionKeyRequired`] when the registry is missing (the DEK
    /// only lives there) and [`Error::EncryptionKeyRejected`] when the master
    /// key does not unwrap it.
    pub(crate) fn open_read(data_dir: &Path, key_material: &[u8]) -> Result<Self> {
        if !KeyRegistry::path_in(data_dir).exists() {
            return Err(Error::EncryptionKeyRequired {
                artifact: ARTIFACT.to_string(),
            });
        }
        let registry = KeyRegistry::open_or_create(data_dir, key_material).map_err(crypto_error)?;
        let dek =
            registry
                .dek(Artifact::BackendKv)
                .ok_or_else(|| Error::EncryptionKeyRequired {
                    artifact: ARTIFACT.to_string(),
                })?;
        Ok(Self {
            cipher: Cipher::new(&dek[..]),
            allocator: None,
            registry: None,
        })
    }

    /// Seal a value: `[12B nonce][ct ‖ 16B tag]`.
    ///
    /// The plaintext length is derivable from the length of the envelope, so
    /// the envelope needs no length prefix (ADR-0057 D1).
    ///
    /// # Errors
    ///
    /// [`Error::EncryptionKeyRequired`] when the context is read-only and
    /// [`Error::EncryptionCorrupt`] when the registry cannot be written.
    pub(crate) fn seal_value(
        &self,
        partition: BackendPartition,
        key: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>> {
        let nonce = self.next_nonce()?;
        let aad = AadBuilder::new(Artifact::BackendKv, BACKEND_KV_FORMAT_VERSION)
            .epoch(nonce.epoch())
            .slot(0)
            .plaintext_len(plaintext.len())
            .identifier(&identifier(partition, key))
            .build();
        // `ct` is the AEAD blob: `ciphertext ‖ 16-byte tag`.
        let ct = self
            .cipher
            .encrypt_with_nonce(nonce.as_bytes(), plaintext, &aad)
            .map_err(crypto_error)?;
        let mut envelope = Vec::with_capacity(NONCE_LEN + ct.len());
        envelope.extend_from_slice(nonce.as_bytes());
        envelope.extend_from_slice(&ct);
        Ok(envelope)
    }

    /// Open a value: the plaintext it was sealed from.
    ///
    /// # Errors
    ///
    /// [`Error::EncryptionCorrupt`] when the envelope is structurally broken
    /// (truncated) and [`Error::EncryptionKeyRejected`] when the AEAD tag
    /// does not authenticate (wrong key **or** tampered ciphertext — a single
    /// tag cannot tell them apart).
    pub(crate) fn open_value(
        &self,
        partition: BackendPartition,
        key: &[u8],
        envelope: &[u8],
    ) -> Result<Vec<u8>> {
        if envelope.len() < NONCE_LEN + TAG_LEN {
            return Err(corrupt(format!(
                "backend value is truncated ({} bytes)",
                envelope.len()
            )));
        }
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&envelope[..NONCE_LEN]);
        let ct = &envelope[NONCE_LEN..];
        let aad = AadBuilder::new(Artifact::BackendKv, BACKEND_KV_FORMAT_VERSION)
            .epoch(Nonce::from_bytes(nonce).epoch())
            .slot(0)
            .plaintext_len(ct.len() - TAG_LEN)
            .identifier(&identifier(partition, key))
            .build();
        self.cipher
            .decrypt_with_nonce(&nonce, ct, &aad)
            .map_err(|_| Error::EncryptionKeyRejected {
                artifact: ARTIFACT.to_string(),
            })
    }

    /// The next nonce of the artefact.
    ///
    /// The counter **never wraps**: when it is exhausted a new epoch is
    /// allocated (and persisted) instead (ADR-0057 D3).
    fn next_nonce(&self) -> Result<Nonce> {
        let Some(allocator) = self.allocator.as_ref() else {
            return Err(Error::generic_error(
                "the backend encryption context is read-only: it cannot seal a value",
            ));
        };
        let current = allocator.load();
        match current.next() {
            Ok(nonce) => Ok(nonce),
            Err(CryptoError::NonceEpochExhausted) => {
                drop(current);
                self.roll_epoch(allocator)?;
                let fresh = allocator.load();
                fresh.next().map_err(crypto_error)
            }
            Err(err) => Err(crypto_error(err)),
        }
    }

    /// Roll the nonce epoch: a fresh random epoch, persisted **before** the
    /// first encryption under it.
    fn roll_epoch(&self, allocator: &arc_swap::ArcSwapAny<Arc<NonceAllocator>>) -> Result<()> {
        let Some(registry) = self.registry.as_ref() else {
            return Err(Error::generic_error(
                "the backend encryption context is read-only: it cannot roll the nonce epoch",
            ));
        };
        let mut registry = registry.lock();
        // Another writer may have rolled the epoch already: adopt its
        // allocator instead of burning a second epoch — the artefact keeps a
        // single, shared nonce sequence.
        if let Some(current) = registry.allocator(Artifact::BackendKv) {
            let mine = allocator.load();
            if !Arc::ptr_eq(current, &*mine) {
                allocator.store(current.clone());
                return Ok(());
            }
        }
        registry
            .begin_epoch(Artifact::BackendKv)
            .map_err(crypto_error)?;
        let fresh = registry
            .allocator(Artifact::BackendKv)
            .cloned()
            .ok_or_else(|| {
                Error::generic_error(
                    "encryption registry has no nonce allocator for the backend artefact",
                )
            })?;
        allocator.store(fresh);
        Ok(())
    }
}

/// The public identifier of a value: `partition ‖ 0x00 ‖ key`.
///
/// It is **not** a secret — the key names stay in cleartext (ADR-0057 §5.3).
/// It is bound into the AAD so a value cannot be moved to another key.
fn identifier(partition: BackendPartition, key: &[u8]) -> Vec<u8> {
    let name = partition.partition_name().as_bytes();
    let mut id = Vec::with_capacity(name.len() + 1 + key.len());
    id.extend_from_slice(name);
    id.push(0x00);
    id.extend_from_slice(key);
    id
}

/// A backend whose **values** are AEAD envelopes.
///
/// The decorator is the **only** place where the envelope is applied, so no
/// write path can forget it — and it works the same for every backend
/// (Fjall, RocksDB), because it sits on the common trait.
pub(crate) struct EncryptedBackend {
    /// The backend that actually stores the envelopes.
    inner: Arc<dyn StorageBackend>,
    /// The encryption context of the `backend` artefact.
    crypto: Arc<BackendKvCrypto>,
}

impl EncryptedBackend {
    /// Build the decorator over `inner`.
    pub(crate) fn new(inner: Arc<dyn StorageBackend>, crypto: Arc<BackendKvCrypto>) -> Self {
        Self { inner, crypto }
    }
}

impl StorageBackend for EncryptedBackend {
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
        let sealed = self.crypto.seal_value(partition, key, value)?;
        self.inner.put(partition, key, &sealed)
    }

    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
        match self.inner.get(partition, key)? {
            None => Ok(None),
            Some(raw) => Ok(Some(self.crypto.open_value(partition, key, &raw)?)),
        }
    }

    fn get_many(
        &self,
        partition: BackendPartition,
        keys: &[&[u8]],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.inner
            .get_many(partition, keys)?
            .into_iter()
            .map(|(k, raw)| {
                let plain = self.crypto.open_value(partition, &k, &raw)?;
                Ok((k, plain))
            })
            .collect()
    }

    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
        self.inner.delete(partition, key)
    }

    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        // The envelope is the value: the batch stays all-or-nothing in the
        // inner backend (ADR-0057 D5).
        let mut sealed = Vec::with_capacity(ops.len());
        for op in ops {
            sealed.push(match op {
                BackendWriteOp::Put {
                    partition,
                    key,
                    value,
                } => {
                    let sealed_value = self.crypto.seal_value(partition, &key, &value)?;
                    BackendWriteOp::Put {
                        partition,
                        key,
                        value: sealed_value,
                    }
                }
                BackendWriteOp::Delete { partition, key } => {
                    BackendWriteOp::Delete { partition, key }
                }
            });
        }
        self.inner.write_batch(sealed)
    }

    fn flush(&self) -> Result<()> {
        self.inner.flush()
    }

    fn capabilities(&self) -> BackendCapabilities {
        self.inner.capabilities()
    }

    fn as_snapshotable(&self) -> Option<&dyn Snapshotable> {
        self.inner.as_snapshotable()
    }

    fn as_compactable(&self) -> Option<&dyn Compactable> {
        self.inner.as_compactable()
    }
}

/// The encrypted backend serves the scan role like every backend: the keys
/// come back in cleartext (they always were) and the values are decrypted.
impl Scannable for EncryptedBackend {
    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.inner
            .scan(partition)?
            .into_iter()
            .map(|(k, raw)| {
                let plain = self.crypto.open_value(partition, &k, &raw)?;
                Ok((k, plain))
            })
            .collect()
    }

    fn scan_prefix_iter<'a>(
        &'a self,
        partition: BackendPartition,
        prefix: &'a [u8],
    ) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> + 'a>> {
        let crypto = self.crypto.clone();
        let iter = self.inner.scan_prefix_iter(partition, prefix)?;
        Ok(Box::new(iter.map(move |item| {
            let (k, raw) = item?;
            let plain = crypto.open_value(partition, &k, &raw)?;
            Ok((k, plain))
        })))
    }
}

/// Resolve the encryption context of the backend KV from the configuration.
///
/// Returns `None` when the backend KV must stay **plaintext**:
///
/// * no key resolved — the bytes on disk are then byte-identical to the
///   layout before this change; or
/// * the database already holds data and the registry has no `backend`
///   entry: an existing plaintext database is **never** migrated implicitly
///   (ADR-0057 D7).
///
/// `is_empty` tells a brand-new database (which is encrypted from its first
/// byte) from an existing plaintext one.
pub(crate) fn backend_kv_crypto_from_config(
    data_dir: &Path,
    config: &Config,
    is_empty: bool,
) -> Result<Option<Arc<BackendKvCrypto>>> {
    let env_key = config
        .encryption_key
        .clone()
        .or_else(|| std::env::var("VANTADB_ENCRYPTION_KEY").ok());
    let material = match crate::crypto::resolve_master_key_material(
        env_key.as_deref(),
        config.encryption_key_file.as_deref(),
        config.encryption_key_cmd.as_deref(),
    ) {
        Ok(material) => material,
        Err(CryptoError::KeyNotSet) => return Ok(None),
        Err(err) => return Err(crypto_error(err)),
    };

    if !KeyRegistry::path_in(data_dir).exists() {
        // No artefact of this database was ever encrypted.
        if !is_empty {
            tracing::warn!(
                "encryption key configured but this database already holds data: \
                 the backend key-value store stays plaintext (no implicit migration)"
            );
            return Ok(None);
        }
        if config.read_only {
            // A read-only open must not write anything — and an empty
            // database has nothing to encrypt.
            return Ok(None);
        }
        // A brand-new database: the backend KV is encrypted from its first byte.
        return Ok(Some(Arc::new(BackendKvCrypto::open(
            data_dir,
            &material[..],
        )?)));
    }

    let registry = KeyRegistry::open_or_create(data_dir, &material[..]).map_err(crypto_error)?;
    if registry.dek(Artifact::BackendKv).is_none() {
        // The registry exists (another artefact is encrypted) but the backend
        // KV was never encrypted: it stays plaintext — no mixed mode
        // (ADR-0057 D1 rule 3).
        return Ok(None);
    }
    if config.read_only {
        return Ok(Some(Arc::new(BackendKvCrypto::open_read(
            data_dir,
            &material[..],
        )?)));
    }
    Ok(Some(Arc::new(BackendKvCrypto::open(
        data_dir,
        &material[..],
    )?)))
}

/// Wrap `backend` with the encrypted-value decorator when the configuration
/// resolves a key.
///
/// `storage_path` is the storage root; the registry lives in its `data`
/// subdirectory (the same place the WAL and the VantaFile use).
///
/// Without a key the backend is returned **unchanged**: the bytes on disk are
/// then byte-identical to the layout before this change.
pub(crate) fn wrap_backend(
    backend: Arc<dyn StorageBackend>,
    storage_path: &str,
    config: &Config,
    is_empty: bool,
) -> Result<Arc<dyn StorageBackend>> {
    let data_dir = Path::new(storage_path).join("data");
    match backend_kv_crypto_from_config(&data_dir, config, is_empty)? {
        Some(crypto) => Ok(Arc::new(EncryptedBackend::new(backend, crypto))),
        None => Ok(backend),
    }
}

/// Map a crypto failure onto the crate error surface (ADR-0057 D6).
fn crypto_error(err: CryptoError) -> Error {
    match err {
        CryptoError::DecryptionFailed => Error::EncryptionKeyRejected {
            artifact: ARTIFACT.to_string(),
        },
        CryptoError::RegistryInvalid(detail) => Error::EncryptionCorrupt {
            artifact: ARTIFACT.to_string(),
            detail,
        },
        other => Error::generic_error(format!("Backend encryption error: {other}")),
    }
}

/// A structural failure of an encrypted artefact.
fn corrupt(detail: String) -> Error {
    Error::EncryptionCorrupt {
        artifact: ARTIFACT.to_string(),
        detail,
    }
}

// ─── Tests (ADR-0057 W4) ─────────────────────────────────────

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::backends::fjall_backend::FjallBackend;
    use tempfile::tempdir;

    /// A 32-byte master key.
    fn material(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    /// A backend under test: the decorated one (the engine's view) and the
    /// raw one (the stored envelopes).
    struct Fixture {
        /// What the engine sees: values are decrypted.
        backend: Arc<dyn StorageBackend>,
        /// What is on disk: values are the envelopes.
        raw: Arc<dyn StorageBackend>,
        /// The encryption context (to build envelopes by hand).
        crypto: Arc<BackendKvCrypto>,
        /// The directory holding the registry.
        data_dir: std::path::PathBuf,
        _dir: tempfile::TempDir,
    }

    /// Build a fixture over a Fjall store.
    fn fjall_fixture(seed: u8) -> Fixture {
        let dir = tempdir().unwrap();
        let path = dir.path().join("db");
        std::fs::create_dir_all(&path).unwrap();
        let data_dir = path.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let config = Config::default();
        let inner = FjallBackend::open(path.to_str().unwrap(), &config).unwrap();
        let raw: Arc<dyn StorageBackend> = Arc::new(inner);
        let crypto = Arc::new(BackendKvCrypto::open(&data_dir, &material(seed)).unwrap());
        let backend: Arc<dyn StorageBackend> =
            Arc::new(EncryptedBackend::new(raw.clone(), crypto.clone()));
        Fixture {
            backend,
            raw,
            crypto,
            data_dir,
            _dir: dir,
        }
    }

    /// Build a fixture over a RocksDB store.
    #[cfg(feature = "rocksdb")]
    fn rocksdb_fixture(seed: u8) -> Fixture {
        use crate::backends::rocksdb_backend::RocksDbBackend;
        let dir = tempdir().unwrap();
        let path = dir.path().join("db");
        std::fs::create_dir_all(&path).unwrap();
        let data_dir = path.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let config = Config::default();
        let inner = RocksDbBackend::open(path.to_str().unwrap(), &config).unwrap();
        let raw: Arc<dyn StorageBackend> = Arc::new(inner);
        let crypto = Arc::new(BackendKvCrypto::open(&data_dir, &material(seed)).unwrap());
        let backend: Arc<dyn StorageBackend> =
            Arc::new(EncryptedBackend::new(raw.clone(), crypto.clone()));
        Fixture {
            backend,
            raw,
            crypto,
            data_dir,
            _dir: dir,
        }
    }

    /// ADR-0057 W4: a value written to an encrypted backend round-trips, and
    /// what is stored on disk is the envelope — not the payload.
    #[test]
    fn backend_value_roundtrip_encrypt_decrypt() {
        fn roundtrip(f: &Fixture) {
            f.backend
                .put(BackendPartition::Default, b"node-1", b"the payload")
                .unwrap();
            assert_eq!(
                f.backend
                    .get(BackendPartition::Default, b"node-1")
                    .unwrap()
                    .as_deref(),
                Some(&b"the payload"[..]),
                "the value must round-trip through the envelope"
            );

            // The stored bytes are the envelope: [12B nonce][ct ‖ 16B tag].
            let raw = f.raw.scan(BackendPartition::Default).unwrap();
            assert_eq!(raw.len(), 1);
            assert_eq!(raw[0].0, b"node-1");
            assert_ne!(raw[0].1, b"the payload");
            assert_eq!(raw[0].1.len(), NONCE_LEN + b"the payload".len() + TAG_LEN);
        }
        roundtrip(&fjall_fixture(0x11));
        #[cfg(feature = "rocksdb")]
        roundtrip(&rocksdb_fixture(0x11));
    }

    /// ADR-0057 W4: a batch stays all-or-nothing and its values are
    /// envelopes on disk.
    #[test]
    fn backend_batch_stays_atomic_with_encrypted_values() {
        let f = fjall_fixture(0x22);
        f.backend
            .put(BackendPartition::Default, b"doomed", b"old value")
            .unwrap();

        let ops = vec![
            BackendWriteOp::Put {
                partition: BackendPartition::Default,
                key: b"kept".to_vec(),
                value: b"value one".to_vec(),
            },
            BackendWriteOp::Put {
                partition: BackendPartition::TextIndex,
                key: b"posting".to_vec(),
                value: b"value two".to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::Default,
                key: b"doomed".to_vec(),
            },
        ];
        f.backend.write_batch(ops).unwrap();

        // Every operation of the batch landed…
        assert_eq!(
            f.backend
                .get(BackendPartition::Default, b"kept")
                .unwrap()
                .as_deref(),
            Some(&b"value one"[..])
        );
        assert_eq!(
            f.backend
                .get(BackendPartition::TextIndex, b"posting")
                .unwrap()
                .as_deref(),
            Some(&b"value two"[..])
        );
        assert!(f
            .backend
            .get(BackendPartition::Default, b"doomed")
            .unwrap()
            .is_none());

        // …and every stored value is an envelope.
        for partition in [BackendPartition::Default, BackendPartition::TextIndex] {
            for (_, stored) in f.raw.scan(partition).unwrap() {
                assert_eq!(
                    stored.len(),
                    NONCE_LEN + stored.len() - NONCE_LEN - TAG_LEN + TAG_LEN,
                    "the stored value is an envelope"
                );
                assert!(stored.len() >= NONCE_LEN + TAG_LEN);
            }
        }
    }

    /// ADR-0057 W4: a wrong key is a hard error — never an empty result.
    #[test]
    fn backend_wrong_key_is_a_hard_error() {
        let f = fjall_fixture(0x33);
        f.backend
            .put(BackendPartition::Default, b"k", b"v")
            .unwrap();

        // The wrong key cannot even unwrap the registry.
        let err = BackendKvCrypto::open(&f.data_dir, &material(0x99)).unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "a wrong key must be a hard error, got: {err}"
        );

        // And a value sealed under another key does not authenticate either:
        // a context built from a *different* master key holds a different DEK.
        let other_dir = tempdir().unwrap();
        let other_data = other_dir.path().join("data");
        std::fs::create_dir_all(&other_data).unwrap();
        let other = Arc::new(BackendKvCrypto::open(&other_data, &material(0x44)).unwrap());
        let envelope = f.raw.scan(BackendPartition::Default).unwrap()[0].1.clone();
        let err = other
            .open_value(BackendPartition::Default, b"k", &envelope)
            .unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "a value sealed under another key must not authenticate, got: {err}"
        );
    }

    /// ADR-0057 W4: a corrupted envelope is detected as a hard error, not as
    /// an empty value.
    #[test]
    fn backend_corrupted_value_is_detected() {
        let f = fjall_fixture(0x44);
        let mut envelope = f
            .crypto
            .seal_value(BackendPartition::Default, b"k", b"payload")
            .unwrap();
        // Flip one bit inside the ciphertext.
        envelope[NONCE_LEN] ^= 0x01;
        f.raw
            .put(BackendPartition::Default, b"k", &envelope)
            .unwrap();

        let err = f.backend.get(BackendPartition::Default, b"k").unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "a corrupted value must be a hard error, got: {err}"
        );

        // A truncated envelope is a structural failure.
        f.raw
            .put(BackendPartition::Default, b"short", &envelope[..4])
            .unwrap();
        let err = f
            .backend
            .get(BackendPartition::Default, b"short")
            .unwrap_err();
        assert!(
            matches!(err, Error::EncryptionCorrupt { .. }),
            "a truncated value must be a structural error, got: {err}"
        );
    }

    /// ADR-0057 W4: a plaintext backend is read unchanged by a build that has
    /// the `encryption` feature — and without a key the stored bytes are
    /// **byte-identical** to the layout before this change.
    #[test]
    fn backend_plaintext_backend_still_reads_on_an_encrypted_build() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("db");
        std::fs::create_dir_all(&path).unwrap();
        let config = Config::default();

        // A database written without a key: the value is stored verbatim.
        let plain = FjallBackend::open(path.to_str().unwrap(), &config).unwrap();
        plain
            .put(BackendPartition::Default, b"k", b"plaintext value")
            .unwrap();
        let stored = plain.scan(BackendPartition::Default).unwrap();
        assert_eq!(
            stored[0].1,
            b"plaintext value".to_vec(),
            "without a key the value is stored byte-identically"
        );
        drop(plain);

        // Reopening it with a key does not migrate it: the value still reads.
        let mut keyed = config.clone();
        keyed.encryption_key = Some("ab".repeat(32));
        let backend = crate::backends::registry::BackendRegistry::default_registry()
            .create(
                crate::backend::BackendKind::Fjall,
                path.to_str().unwrap(),
                &keyed,
            )
            .unwrap();
        assert_eq!(
            backend
                .get(BackendPartition::Default, b"k")
                .unwrap()
                .as_deref(),
            Some(&b"plaintext value"[..]),
            "a plaintext backend must be read unchanged"
        );
    }

    /// ADR-0057 W4: the text index postings (surface #5) are ciphertext at
    /// rest too — they live in a keyspace of the backend KV.
    #[test]
    fn backend_text_index_postings_are_ciphertext_at_rest() {
        let f = fjall_fixture(0x55);
        let ops = crate::text_index::posting_put_ops("ns", "key", "hello world", 42).unwrap();
        assert!(!ops.is_empty(), "the payload produces postings");
        f.backend.write_batch(ops).unwrap();

        let raw = f.raw.scan(BackendPartition::TextIndex).unwrap();
        let plain = f.backend.scan(BackendPartition::TextIndex).unwrap();
        assert_eq!(raw.len(), plain.len());
        assert!(!raw.is_empty());
        for ((_, stored), (_, decrypted)) in raw.iter().zip(plain.iter()) {
            assert_ne!(stored, decrypted, "the posting must be encrypted at rest");
            assert_eq!(
                stored.len(),
                decrypted.len() + NONCE_LEN + TAG_LEN,
                "the envelope is the nonce plus the tag around the plaintext"
            );
        }
    }

    /// ADR-0057 W4: the keys stay in cleartext, so the ordered prefix scans
    /// the text index and the namespace index rely on still work.
    #[test]
    fn backend_namespace_prefix_scan_still_works() {
        let f = fjall_fixture(0x66);
        let key_a = b"alpha\x00token\x00k1".to_vec();
        let key_b = b"alpha\x00token\x00k2".to_vec();
        let key_c = b"beta\x00token\x00k1".to_vec();
        f.backend
            .put(BackendPartition::TextIndex, &key_a, b"value one")
            .unwrap();
        f.backend
            .put(BackendPartition::TextIndex, &key_b, b"value two")
            .unwrap();
        f.backend
            .put(BackendPartition::TextIndex, &key_c, b"value three")
            .unwrap();

        // The prefix scan finds the two "alpha" entries…
        let scanned: Vec<(Vec<u8>, Vec<u8>)> = f
            .backend
            .scan_prefix_iter(BackendPartition::TextIndex, b"alpha\x00")
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert_eq!(scanned.len(), 2);
        assert_eq!(scanned[0].0, key_a);
        assert_eq!(
            scanned[0].1,
            b"value one".to_vec(),
            "the value is decrypted"
        );
        assert_eq!(scanned[1].1, b"value two".to_vec());

        // …and the keys are still in cleartext on disk.
        let raw = f.raw.scan(BackendPartition::TextIndex).unwrap();
        assert!(raw.iter().any(|(k, _)| k == &key_c));
    }

    /// ADR-0057 W4: `internal_metadata` (the `checkpoint_seq`) is encrypted
    /// too and round-trips.
    #[test]
    fn backend_checkpoint_seq_roundtrips() {
        let f = fjall_fixture(0x77);
        let seq: u64 = 4242;
        f.backend
            .put(
                BackendPartition::InternalMetadata,
                b"checkpoint_seq",
                &seq.to_le_bytes(),
            )
            .unwrap();

        let read = f
            .backend
            .get(BackendPartition::InternalMetadata, b"checkpoint_seq")
            .unwrap()
            .expect("checkpoint_seq");
        assert_eq!(u64::from_le_bytes(read[..].try_into().unwrap()), seq);

        // And it is ciphertext at rest.
        let raw = f.raw.scan(BackendPartition::InternalMetadata).unwrap();
        assert_eq!(raw.len(), 1);
        assert_eq!(raw[0].1.len(), 8 + NONCE_LEN + TAG_LEN);
    }

    /// ADR-0057 W4 / D4: the AAD binds a value to its key and its partition
    /// — a value moved to another key does not authenticate.
    #[test]
    fn backend_value_moved_to_another_key_does_not_authenticate() {
        let f = fjall_fixture(0x88);
        f.backend
            .put(BackendPartition::Default, b"key-a", b"payload")
            .unwrap();
        let envelope = f.raw.scan(BackendPartition::Default).unwrap()[0].1.clone();

        // Move the envelope, byte for byte, to another key.
        f.raw
            .put(BackendPartition::Default, b"key-b", &envelope)
            .unwrap();
        let err = f
            .backend
            .get(BackendPartition::Default, b"key-b")
            .unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "a value moved to another key must not authenticate, got: {err}"
        );

        // The same envelope in another partition does not authenticate either.
        f.raw
            .put(BackendPartition::TextIndex, b"key-a", &envelope)
            .unwrap();
        let err = f
            .backend
            .get(BackendPartition::TextIndex, b"key-a")
            .unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "a value moved to another partition must not authenticate, got: {err}"
        );
    }

    /// ADR-0057 W4: the factory wiring — an engine opened with a key writes
    /// **encrypted** backend values, and the plaintext never reaches the
    /// backend's files.
    #[cfg(feature = "fjall")]
    #[test]
    fn backend_engine_roundtrip_with_key_encrypts_the_backend_kv() {
        use crate::config::Config;
        use crate::node::{FieldValue, NodeFlags, UnifiedNode, VectorRepresentations};
        use crate::storage::StorageEngine;

        /// A payload that is unique to this test: it must never appear in
        /// cleartext in the backend's files.
        const CANARY: &str = "vantadb-w4-backend-kv-canary-8f3c1d";

        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let config = Config {
            storage_path: path.to_string(),
            backend_kind: crate::backend::BackendKind::Fjall,
            encryption_key: Some("ab".repeat(32)),
            ..Config::default()
        };
        let engine = StorageEngine::open_with_config(path, Some(config)).expect("open");

        let id: u128 = 0x1234_5678_9abc_def0;
        let mut node = UnifiedNode::new(id);
        node.vector = VectorRepresentations::Full(vec![0.25, 0.5, 0.75]);
        node.flags.set(NodeFlags::HAS_VECTOR);
        node.relational.insert(
            "w4_canary".to_string(),
            FieldValue::String(CANARY.to_string()),
        );
        engine.insert(&node).expect("insert");

        // The node reads back through the encrypted backend.
        let read = engine.get(id).expect("get").expect("node present");
        assert_eq!(
            read.vector.as_f32_slice().expect("a FULL vector"),
            &[0.25, 0.5, 0.75]
        );

        // The registry holds the `backend` DEK: the backend KV is encrypted.
        let registry = KeyRegistry::open_or_create(&dir.path().join("data"), &material(0xab))
            .expect("registry");
        assert!(
            registry.dek(Artifact::BackendKv).is_some(),
            "the registry must hold the backend DEK"
        );

        // The payload never reaches the backend's files in cleartext. The
        // backend's own files are the ones at the storage root (the journal
        // and the SSTs); `data/` holds the WAL, the segments and the registry.
        //
        // The engine is dropped first: it holds the journal open (and locked
        // on Windows), so the bytes can only be read once it is closed.
        drop(engine);
        for entry in std::fs::read_dir(dir.path()).expect("the storage root") {
            let path = entry.expect("entry").path();
            if !path.is_file() {
                continue;
            }
            // The lock file is process-local: it holds no payload.
            if path.file_name().is_some_and(|n| n == ".vanta.lock") {
                continue;
            }
            let bytes = std::fs::read(&path).expect("read");
            assert!(
                !bytes.windows(CANARY.len()).any(|w| w == CANARY.as_bytes()),
                "the payload must not appear in cleartext in {}",
                path.display()
            );
        }
    }

    /// ADR-0057 W4 / D3: the nonce counter never wraps — when it is
    /// exhausted a **new epoch** is allocated (and persisted) instead.
    #[test]
    fn backend_nonce_epoch_overflow_allocates_a_new_epoch() {
        let f = fjall_fixture(0x99);
        {
            let guard = f.crypto.allocator.as_ref().unwrap().load();
            guard.set_counter_for_test(u32::MAX - 2);
        }
        // Two nonces fit in the old epoch…
        let a = f
            .crypto
            .seal_value(BackendPartition::Default, b"a", b"x")
            .unwrap();
        let b = f
            .crypto
            .seal_value(BackendPartition::Default, b"b", b"x")
            .unwrap();
        assert_eq!(&a[..8], &b[..8], "both use the same epoch");
        assert_ne!(&a[8..12], &b[8..12], "the counter advances");
        // …the third exhausts the counter: a new epoch is rolled.
        let c = f
            .crypto
            .seal_value(BackendPartition::Default, b"c", b"x")
            .unwrap();
        assert_ne!(
            &c[..8],
            &a[..8],
            "the counter must never wrap: a new epoch is allocated"
        );
        // The epoch is persisted before its first use.
        assert!(
            KeyRegistry::path_in(&f.data_dir).exists(),
            "the registry must exist on disk"
        );
    }
}
