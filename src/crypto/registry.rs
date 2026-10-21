//! Key registry, nonce allocator and AAD builder for at-rest encryption
//! (ADR-0057 §D2–§D4).
//!
//! # Key hierarchy
//!
//! ```text
//! master key ──HKDF-SHA256(salt=db_salt, info="vanta-kek-v1"‖key_version)──▶ KEK
//!                                                                            │
//!                                            wraps (AEAD) a random 32-byte DEK per artefact
//!                                                                            ▼
//!                                                            registry (on disk, in clear)
//! ```
//!
//! * The **master key** is never persisted. It comes from the environment, a
//!   `0600` file or a KMS shim (see [`crate::crypto::KeySource`]).
//! * The **KEK** is derived per database from the master and the database salt
//!   (`db_salt`, public, stored in the registry).
//! * Every artefact owns a random **DEK**, stored **wrapped** under the KEK.
//!   Rotating the master key re-wraps the DEKs; it never re-derives them, so
//!   no ciphertext is invalidated.
//!
//! # Registry file
//!
//! `<data_dir>/encryption.registry` — inside `data_dir` on purpose: the
//! snapshot machinery (`mirror_data_dir`) already copies that directory, and
//! the registry is needed **before** the backend can be opened (the backend's
//! own values are encrypted).
//!
//! It is written atomically: tmp file → `fsync` → `rename` → `fsync` of the
//! parent directory (the pattern of `storage::archive`).
//!
//! # Nonce scheme
//!
//! `nonce (12 bytes) = epoch (u64 BE) ‖ counter (u32 BE)`.
//!
//! The **epoch** is random, persisted in the registry **before** the first
//! encryption under it, and never reused. The **counter** is monotonic within
//! an epoch and lives in memory; when it would overflow, the caller allocates
//! a new epoch (the counter never wraps).

use crate::crypto::{Cipher, CryptoError, Zeroizing};
use crate::utils::fs::sync_parent_dir;
use parking_lot::Mutex;
use rand::RngCore;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, OnceLock, Weak};

/// File name of the registry inside the storage `data_dir`.
pub const REGISTRY_FILE_NAME: &str = "encryption.registry";

/// File name of the **previous** registry, left behind by a rotation.
///
/// The rotation re-wraps the DEKs (it never re-derives them), so the old
/// registry is still a valid description of the database as long as the
/// **old** master key exists: copying it back is a manual rollback
/// (ADR-0057 §7.2).
pub const REGISTRY_BACKUP_FILE_NAME: &str = "encryption.registry.bak";

/// Magic bytes at the head of the registry file.
const REGISTRY_MAGIC: &[u8; 8] = b"VANTAREG";
/// On-disk format version of the registry file.
const REGISTRY_FORMAT: u16 = 1;
/// AAD prefix of the wrapped DEK.
const DEK_AAD_PREFIX: &[u8] = b"vanta-dek-v1";
/// AAD prefix of the wrapped key-check sentinel.
const KEY_CHECK_AAD_PREFIX: &[u8] = b"vanta-keycheck-v1";
/// Length of the DEK id (a random label, so a re-wrap is detectable).
const DEK_ID_LEN: usize = 8;

/// A logical on-disk artefact that owns its own DEK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Artifact {
    /// The write-ahead log (`data/vanta.wal`, `data/vanta.shard{N}.wal`).
    Wal,
    /// The VantaFile segments (`data/vstore_L{0..3}.vanta`).
    VantaFile,
    /// The HNSW index image (`data/vector_index.bin`).
    HnswImage,
    /// The backend key-value store (Fjall / RocksDB values).
    BackendKv,
}

impl Artifact {
    /// Stable numeric id — travels in the AAD and in the registry file.
    pub const fn code(self) -> u16 {
        match self {
            Artifact::Wal => 1,
            Artifact::VantaFile => 2,
            Artifact::HnswImage => 3,
            Artifact::BackendKv => 4,
        }
    }

    /// Short human-readable name (error messages, logs).
    pub const fn as_str(self) -> &'static str {
        match self {
            Artifact::Wal => "wal",
            Artifact::VantaFile => "vfile",
            Artifact::HnswImage => "hnsw",
            Artifact::BackendKv => "backend",
        }
    }

    /// Parse a numeric id back (registry file).
    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            1 => Some(Artifact::Wal),
            2 => Some(Artifact::VantaFile),
            3 => Some(Artifact::HnswImage),
            4 => Some(Artifact::BackendKv),
            _ => None,
        }
    }
}

/// A 12-byte AEAD nonce: `epoch (u64 BE) ‖ counter (u32 BE)`.
///
/// The two halves are unique **by construction** under one DEK: the epoch is
/// random and never reused, the counter is monotonic within the epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nonce {
    bytes: [u8; 12],
}

impl Nonce {
    /// Build a nonce from its two halves.
    pub fn from_parts(epoch: u64, counter: u32) -> Self {
        let mut bytes = [0u8; 12];
        bytes[..8].copy_from_slice(&epoch.to_be_bytes());
        bytes[8..].copy_from_slice(&counter.to_be_bytes());
        Self { bytes }
    }

    /// Parse a nonce from its 12 wire bytes.
    pub const fn from_bytes(bytes: [u8; 12]) -> Self {
        Self { bytes }
    }

    /// The 12 nonce bytes.
    pub const fn as_bytes(&self) -> &[u8; 12] {
        &self.bytes
    }

    /// The epoch half.
    pub fn epoch(&self) -> u64 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.bytes[..8]);
        u64::from_be_bytes(buf)
    }

    /// The counter half.
    pub fn counter(&self) -> u32 {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.bytes[8..]);
        u32::from_be_bytes(buf)
    }
}

/// Builder of the AEAD associated data (ADR-0057 D4).
///
/// ```text
/// AAD = artifact_id (u16 BE) ‖ format_version (u16 BE) ‖ epoch (u64 BE)
///     ‖ slot (u64 BE) ‖ plaintext_len (u64 BE) [‖ identifier]
/// ```
///
/// The prefix has a fixed length, so the optional trailing identifier is
/// unambiguous. The AAD is **authenticated, not encrypted**: it holds no
/// secret, it binds a ciphertext to its position.
#[derive(Debug, Clone)]
pub struct AadBuilder {
    artifact: u16,
    format_version: u16,
    epoch: u64,
    slot: u64,
    plaintext_len: u64,
    identifier: Option<Vec<u8>>,
}

impl AadBuilder {
    /// Start an AAD for `artifact` at `format_version`.
    pub fn new(artifact: Artifact, format_version: u16) -> Self {
        Self {
            artifact: artifact.code(),
            format_version,
            epoch: 0,
            slot: 0,
            plaintext_len: 0,
            identifier: None,
        }
    }

    /// The nonce epoch this AAD belongs to.
    pub fn epoch(mut self, epoch: u64) -> Self {
        self.epoch = epoch;
        self
    }

    /// The slot (frame ordinal, node offset, …) the ciphertext belongs to.
    pub fn slot(mut self, slot: u64) -> Self {
        self.slot = slot;
        self
    }

    /// The length of the plaintext that is being encrypted.
    pub fn plaintext_len(mut self, len: usize) -> Self {
        self.plaintext_len = len as u64;
        self
    }

    /// A public identifier of the record (e.g. `partition‖0x00‖key`).
    pub fn identifier(mut self, identifier: &[u8]) -> Self {
        self.identifier = Some(identifier.to_vec());
        self
    }

    /// Build the canonical AAD bytes.
    pub fn build(&self) -> Vec<u8> {
        let identifier_len = self.identifier.as_ref().map_or(0, Vec::len);
        let mut out = Vec::with_capacity(28 + identifier_len);
        out.extend_from_slice(&self.artifact.to_be_bytes());
        out.extend_from_slice(&self.format_version.to_be_bytes());
        out.extend_from_slice(&self.epoch.to_be_bytes());
        out.extend_from_slice(&self.slot.to_be_bytes());
        out.extend_from_slice(&self.plaintext_len.to_be_bytes());
        if let Some(identifier) = &self.identifier {
            out.extend_from_slice(identifier);
        }
        out
    }
}

/// Per-artefact nonce allocator: a persisted epoch plus an in-memory counter.
///
/// The counter is an [`AtomicU32`], so the hot path is lock-free. When it
/// would overflow the allocator **fails** instead of wrapping: the caller
/// allocates a new epoch (see [`KeyRegistry::begin_epoch`]).
#[derive(Debug)]
pub struct NonceAllocator {
    artifact: Artifact,
    epoch: u64,
    counter: AtomicU32,
}

impl NonceAllocator {
    /// Create an allocator bound to `epoch`.
    pub fn new(artifact: Artifact, epoch: u64) -> Self {
        Self {
            artifact,
            epoch,
            counter: AtomicU32::new(0),
        }
    }

    /// The artefact this allocator serves.
    pub fn artifact(&self) -> Artifact {
        self.artifact
    }

    /// The current epoch.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// The next nonce of this epoch.
    ///
    /// # Errors
    ///
    /// [`CryptoError::NonceEpochExhausted`] when the counter would wrap. The
    /// counter is **not** advanced in that case, so no nonce is ever reused.
    pub fn next(&self) -> Result<Nonce, CryptoError> {
        let counter = self
            .counter
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |c| c.checked_add(1))
            .map_err(|_| CryptoError::NonceEpochExhausted)?;
        Ok(Nonce::from_parts(self.epoch, counter))
    }
}

#[cfg(test)]
impl NonceAllocator {
    /// Test-only: force the counter to `value`.
    ///
    /// The counter is private to this module, so a test living in another
    /// module (e.g. the WAL overflow test) cannot reach it. This is the only
    /// way to exercise the "counter exhausted → new epoch" path from the
    /// outside without adding a production knob.
    pub fn set_counter_for_test(&self, value: u32) {
        self.counter.store(value, Ordering::SeqCst);
    }
}

/// One artefact's entry in the registry.
#[derive(Debug)]
struct ArtifactEntry {
    /// The DEK itself (in memory, wiped on drop).
    dek: Zeroizing<[u8; 32]>,
    /// The DEK as stored on disk (wrapped under the KEK).
    wrapped: Vec<u8>,
    /// Random 8-byte label of the DEK, so a re-wrap is detectable.
    dek_id: [u8; DEK_ID_LEN],
    /// The current nonce epoch of this artefact.
    epoch: u64,
}

/// The mutable state of one database's registry.
///
/// It lives behind a lock and is **shared by every handle** of the same
/// database (ADR-0057 W6): the four artefacts open the registry
/// independently, and a `persist` from one of them must never drop the
/// entries of the others (FIND-514).
#[derive(Debug)]
struct RegistryState {
    /// The database salt (public; domain-separates this database's keys).
    db_salt: [u8; 32],
    /// The key version the DEKs are wrapped under.
    key_version: u8,
    /// The KEK — derived from the master and the database salt.
    kek: Cipher,
    /// Wrapped sentinel: proves the master key is the right one at open.
    key_check: Vec<u8>,
    /// The per-artefact entries.
    entries: BTreeMap<Artifact, ArtifactEntry>,
}

/// The shared, per-process state of one database's registry.
#[derive(Debug)]
struct RegistryShared {
    state: Mutex<RegistryState>,
}

/// The process-wide table of open registries, keyed by the registry file.
///
/// One entry per storage root: every artefact that opens the registry of
/// the same database gets a handle to the **same** state, so there is one
/// in-memory registry and one `persist` path per process (ADR-0057 W6,
/// FIND-514). The entry disappears when the last handle goes away, so a
/// later open re-reads the file from disk.
static OPEN_REGISTRIES: OnceLock<Mutex<HashMap<PathBuf, Weak<RegistryShared>>>> = OnceLock::new();

/// The table of open registries.
fn open_registries() -> &'static Mutex<HashMap<PathBuf, Weak<RegistryShared>>> {
    OPEN_REGISTRIES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The per-artefact DEK registry of one database.
///
/// The registry owns the database salt, the wrapped DEKs and the current nonce
/// epoch per artefact. It is written atomically (tmp → rename → dir fsync) and
/// lives **inside** the storage `data_dir`.
///
/// A `KeyRegistry` is a **handle**: the state behind it is shared by every
/// handle of the same database (ADR-0057 W6), so the four artefacts of one
/// engine write the same file from the same state — no lost update, one
/// in-memory registry, one `persist` path.
pub struct KeyRegistry {
    path: PathBuf,
    /// The shared state — one instance per database per process.
    shared: Arc<RegistryShared>,
    /// The nonce allocators of this handle. Lock-free reads: the hot path
    /// must not take a lock (ADR-0057 D5).
    allocators: BTreeMap<Artifact, Arc<NonceAllocator>>,
}

impl std::fmt::Debug for KeyRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.shared.state.lock();
        f.debug_struct("KeyRegistry")
            .field("path", &self.path)
            .field("key_version", &state.key_version)
            .field("artifacts", &state.entries.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl KeyRegistry {
    /// Path of the registry file inside `data_dir`.
    pub fn path_in(data_dir: &Path) -> PathBuf {
        data_dir.join(REGISTRY_FILE_NAME)
    }

    /// Path of the backup the rotation leaves behind.
    pub fn backup_path_in(data_dir: &Path) -> PathBuf {
        data_dir.join(REGISTRY_BACKUP_FILE_NAME)
    }

    /// Open the registry of `data_dir`, creating it when it does not exist.
    ///
    /// `key_material` is the material coming from the key source: **32 bytes**
    /// are used as the master key directly; a shorter value is a passphrase
    /// and is stretched **once** with PBKDF2-HMAC-SHA256 and the database salt
    /// (ADR-0057 C-4).
    ///
    /// The registry is **shared per database and per process** (ADR-0057
    /// W6): the four artefacts of an engine open it independently and every
    /// one of them gets a handle to the **same** in-memory state, so a
    /// `persist` from one artefact can never drop the entries of another
    /// (FIND-514). A wrong master key is rejected here — the key check does
    /// not unwrap — never silently accepted.
    ///
    /// # Errors
    ///
    /// [`CryptoError::RegistryInvalid`] when the file is malformed,
    /// [`CryptoError::RegistryIo`] on an I/O failure and
    /// [`CryptoError::DecryptionFailed`] when the master key does not unwrap
    /// the registry (wrong key **or** tampered registry — indistinguishable).
    pub fn open_or_create(data_dir: &Path, key_material: &[u8]) -> Result<Self, CryptoError> {
        let path = Self::path_in(data_dir);
        // The table lock is held across the read: two databases opened at the
        // same time in one process serialise here. The open is a startup
        // path (a handful of times per process), so the cost is irrelevant
        // next to the correctness it buys (one state per database).
        let mut table = open_registries().lock();
        if let Some(shared) = table.get(&path).and_then(Weak::upgrade) {
            // Another handle of this process already opened the registry:
            // reuse its state instead of reading a stale copy from disk.
            return Self::from_shared(path, shared, key_material);
        }
        let state = Self::read_state(&path, key_material)?;
        let shared = Arc::new(RegistryShared {
            state: Mutex::new(state),
        });
        table.insert(path.clone(), Arc::downgrade(&shared));
        let registry = Self::from_shared(path, shared, key_material)?;
        // A brand-new registry is persisted immediately: the key check must
        // exist before the first open, so a wrong key is detected then.
        if registry.is_fresh() {
            registry.persist()?;
        }
        Ok(registry)
    }

    /// Build a handle over an existing shared state, verifying the material.
    fn from_shared(
        path: PathBuf,
        shared: Arc<RegistryShared>,
        key_material: &[u8],
    ) -> Result<Self, CryptoError> {
        {
            let state = shared.state.lock();
            // The key check proves the master key is the right one: a wrong
            // key cannot unwrap it.
            let master = master_from_material(key_material, &state.db_salt)?;
            let kek = Cipher::new(&master[..]).derive_kek(&state.db_salt, state.key_version)?;
            unwrap_key_check(&kek, &state.key_check, state.key_version)?;
        }
        Ok(Self {
            path,
            shared,
            allocators: BTreeMap::new(),
        })
    }

    /// Read the registry state from disk (or build a fresh one).
    fn read_state(path: &Path, key_material: &[u8]) -> Result<RegistryState, CryptoError> {
        let raw = match std::fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                return Err(CryptoError::RegistryIo(format!(
                    "cannot read {}: {e}",
                    path.display()
                )))
            }
        };
        let file = match raw {
            Some(bytes) => Some(decode(&bytes)?),
            None => None,
        };
        let db_salt = match &file {
            Some(f) => f.db_salt,
            None => random_bytes::<32>(),
        };
        let key_version = file.as_ref().map_or(1, |f| f.key_version);
        // The master: a raw 32-byte key, or a passphrase stretched once with
        // the database salt.
        let master = master_from_material(key_material, &db_salt)?;
        let kek = Cipher::new(&master[..]).derive_kek(&db_salt, key_version)?;
        let mut state = RegistryState {
            db_salt,
            key_version,
            kek,
            key_check: Vec::new(),
            entries: BTreeMap::new(),
        };
        match file {
            Some(f) => {
                state.key_check = f.key_check;
                unwrap_key_check(&state.kek, &state.key_check, key_version)?;
                for entry in f.entries {
                    let artifact = Artifact::from_code(entry.artifact).ok_or_else(|| {
                        CryptoError::RegistryInvalid(format!(
                            "unknown artefact id {}",
                            entry.artifact
                        ))
                    })?;
                    let dek = unwrap_dek(&state.kek, &entry.wrapped, artifact, key_version)?;
                    state.entries.insert(
                        artifact,
                        ArtifactEntry {
                            dek,
                            wrapped: entry.wrapped,
                            dek_id: entry.dek_id,
                            epoch: entry.epoch,
                        },
                    );
                }
            }
            None => {
                // A fresh registry: create the sentinel that will detect a
                // wrong key on the next open.
                state.key_check = wrap_key_check(&state.kek, key_version)?;
            }
        }
        Ok(state)
    }

    /// Whether this registry was created by this open (no entries yet).
    fn is_fresh(&self) -> bool {
        self.shared.state.lock().entries.is_empty()
    }

    /// The database salt (public; domain-separates this database's keys).
    pub fn db_salt(&self) -> [u8; 32] {
        self.shared.state.lock().db_salt
    }

    /// The key version the DEKs are wrapped under.
    pub fn key_version(&self) -> u8 {
        self.shared.state.lock().key_version
    }

    /// The path of the registry file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The artefacts this registry holds a DEK for.
    ///
    /// This is the **mode record** of the database (ADR-0057 D1 rule 3): an
    /// artefact without an entry is plaintext, and it is never migrated
    /// implicitly.
    pub fn artifacts(&self) -> std::collections::BTreeSet<Artifact> {
        self.shared.state.lock().entries.keys().copied().collect()
    }

    /// The DEK of `artifact`, if it already exists.
    pub fn dek(&self, artifact: Artifact) -> Option<Zeroizing<[u8; 32]>> {
        self.shared
            .state
            .lock()
            .entries
            .get(&artifact)
            .map(|e| e.dek.clone())
    }

    /// The DEK of `artifact`, creating (and persisting) one when missing.
    ///
    /// # Errors
    ///
    /// [`CryptoError::RegistryIo`] when the registry cannot be written.
    pub fn ensure_dek(&mut self, artifact: Artifact) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
        let created = {
            let mut state = self.shared.state.lock();
            if state.entries.contains_key(&artifact) {
                false
            } else {
                let dek = random_bytes::<32>();
                let wrapped = wrap_dek(&state.kek, &dek, artifact, state.key_version)?;
                let mut dek_id = [0u8; DEK_ID_LEN];
                rand::rng().fill_bytes(&mut dek_id);
                state.entries.insert(
                    artifact,
                    ArtifactEntry {
                        dek: Zeroizing::new(dek),
                        wrapped,
                        dek_id,
                        epoch: 0,
                    },
                );
                true
            }
        };
        if created {
            self.persist()?;
        }
        self.dek(artifact).ok_or_else(|| {
            CryptoError::RegistryInvalid(format!("no DEK for {}", artifact.as_str()))
        })
    }

    /// Allocate a **fresh epoch** for `artifact` and persist it **before** the
    /// first encryption under it.
    ///
    /// The epoch is random (64 bits) and never reused: a rollback of the
    /// registry cannot replay an epoch, because a new one is allocated on
    /// every write epoch.
    ///
    /// # Errors
    ///
    /// [`CryptoError::RegistryIo`] when the registry cannot be written.
    pub fn begin_epoch(&mut self, artifact: Artifact) -> Result<u64, CryptoError> {
        let epoch = u64::from_be_bytes(random_bytes::<8>());
        {
            let mut state = self.shared.state.lock();
            if !state.entries.contains_key(&artifact) {
                let dek = random_bytes::<32>();
                let wrapped = wrap_dek(&state.kek, &dek, artifact, state.key_version)?;
                let mut dek_id = [0u8; DEK_ID_LEN];
                rand::rng().fill_bytes(&mut dek_id);
                state.entries.insert(
                    artifact,
                    ArtifactEntry {
                        dek: Zeroizing::new(dek),
                        wrapped,
                        dek_id,
                        epoch: 0,
                    },
                );
            }
            if let Some(entry) = state.entries.get_mut(&artifact) {
                entry.epoch = epoch;
            }
        }
        self.allocators
            .insert(artifact, Arc::new(NonceAllocator::new(artifact, epoch)));
        // The epoch must be durable BEFORE the first encryption under it.
        self.persist()?;
        Ok(epoch)
    }

    /// The nonce allocator of `artifact` (`None` until [`KeyRegistry::begin_epoch`]).
    pub fn allocator(&self, artifact: Artifact) -> Option<&Arc<NonceAllocator>> {
        self.allocators.get(&artifact)
    }

    /// The next nonce of `artifact` (lock-free: one atomic increment).
    ///
    /// # Errors
    ///
    /// [`CryptoError::NoEpoch`] when no epoch was allocated yet and
    /// [`CryptoError::NonceEpochExhausted`] when the counter is exhausted —
    /// in both cases the caller must [`KeyRegistry::begin_epoch`] again.
    pub fn next_nonce(&self, artifact: Artifact) -> Result<Nonce, CryptoError> {
        let allocator = self
            .allocators
            .get(&artifact)
            .ok_or_else(|| CryptoError::NoEpoch(artifact.as_str().to_string()))?;
        allocator.next()
    }

    /// **Rotate** the master key: re-wrap every DEK under a new KEK.
    ///
    /// The DEKs are **not** re-derived — only their wrapping changes — so no
    /// ciphertext already on disk is invalidated (`FIND-194`). The operation
    /// is O(number of artefacts) = 4, not O(data).
    ///
    /// The previous registry is left as `encryption.registry.bak`: copying it
    /// back is a manual rollback, possible while the **old** master key still
    /// exists (ADR-0057 §7.2).
    ///
    /// # Errors
    ///
    /// [`CryptoError::RegistryIo`] when the registry or its backup cannot be
    /// written, and [`CryptoError::RegistryInvalid`] when the key version is
    /// exhausted.
    pub fn rotate(&mut self, new_material: &[u8]) -> Result<u8, CryptoError> {
        // The backup first: a crash after this point leaves the old registry
        // recoverable.
        if self.path.exists() {
            let backup = Self::backup_path_in(self.path.parent().unwrap_or(Path::new(".")));
            std::fs::copy(&self.path, &backup).map_err(|e| {
                CryptoError::RegistryIo(format!(
                    "cannot write the registry backup {}: {e}",
                    backup.display()
                ))
            })?;
        }
        let new_version = {
            let mut state = self.shared.state.lock();
            let new_version = state.key_version.checked_add(1).ok_or_else(|| {
                CryptoError::RegistryInvalid("the key version is exhausted".into())
            })?;
            // The salt is public and stays: only the wrapping changes.
            let master = master_from_material(new_material, &state.db_salt)?;
            let new_kek = Cipher::new(&master[..]).derive_kek(&state.db_salt, new_version)?;
            let artifacts: Vec<Artifact> = state.entries.keys().copied().collect();
            for artifact in artifacts {
                let dek = state
                    .entries
                    .get(&artifact)
                    .map(|e| *e.dek)
                    .ok_or_else(|| {
                        CryptoError::RegistryInvalid(format!("no DEK for {}", artifact.as_str()))
                    })?;
                let wrapped = wrap_dek(&new_kek, &dek, artifact, new_version)?;
                let mut dek_id = [0u8; DEK_ID_LEN];
                rand::rng().fill_bytes(&mut dek_id);
                if let Some(entry) = state.entries.get_mut(&artifact) {
                    entry.wrapped = wrapped;
                    // A new label: a re-wrap is detectable.
                    entry.dek_id = dek_id;
                }
            }
            // The sentinel is re-wrapped too: it must prove the **new** key.
            state.key_check = wrap_key_check(&new_kek, new_version)?;
            state.key_version = new_version;
            state.kek = new_kek;
            new_version
        };
        self.persist()?;
        Ok(new_version)
    }

    /// Serialize the registry state.
    fn encode(state: &RegistryState) -> Vec<u8> {
        let mut out = Vec::with_capacity(256);
        out.extend_from_slice(REGISTRY_MAGIC);
        out.extend_from_slice(&REGISTRY_FORMAT.to_be_bytes());
        out.push(state.key_version);
        out.push(0); // reserved
        out.extend_from_slice(&state.db_salt);
        out.extend_from_slice(&(state.key_check.len() as u32).to_be_bytes());
        out.extend_from_slice(&state.key_check);
        out.extend_from_slice(&(state.entries.len() as u32).to_be_bytes());
        for (artifact, entry) in &state.entries {
            out.extend_from_slice(&artifact.code().to_be_bytes());
            out.extend_from_slice(&entry.dek_id);
            out.extend_from_slice(&(entry.wrapped.len() as u32).to_be_bytes());
            out.extend_from_slice(&entry.wrapped);
            out.extend_from_slice(&entry.epoch.to_be_bytes());
        }
        let crc = crc32c::crc32c(&out);
        out.extend_from_slice(&crc.to_be_bytes());
        out
    }

    /// Write the registry atomically: tmp file → fsync → rename → dir fsync.
    fn persist(&self) -> Result<(), CryptoError> {
        let bytes = Self::encode(&self.shared.state.lock());
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CryptoError::RegistryIo(format!("cannot create {}: {e}", parent.display()))
            })?;
        }
        let tmp_path = self.tmp_path();
        {
            let mut file = std::fs::File::create(&tmp_path).map_err(|e| {
                CryptoError::RegistryIo(format!("cannot create {}: {e}", tmp_path.display()))
            })?;
            use std::io::Write;
            file.write_all(&bytes).map_err(|e| {
                CryptoError::RegistryIo(format!("cannot write {}: {e}", tmp_path.display()))
            })?;
            file.sync_all().map_err(|e| {
                CryptoError::RegistryIo(format!("cannot sync {}: {e}", tmp_path.display()))
            })?;
        }
        std::fs::rename(&tmp_path, &self.path).map_err(|e| {
            CryptoError::RegistryIo(format!(
                "cannot rename {} to {}: {e}",
                tmp_path.display(),
                self.path.display()
            ))
        })?;
        // A rename is not durable until the parent directory is fsync'd.
        sync_parent_dir(&self.path).map_err(|e| {
            CryptoError::RegistryIo(format!("cannot sync the registry directory: {e}"))
        })?;
        Ok(())
    }

    /// Path of the temporary file used by the atomic write.
    fn tmp_path(&self) -> PathBuf {
        let name = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| REGISTRY_FILE_NAME.to_string());
        self.path.with_file_name(format!("{name}.tmp"))
    }
}

/// Wrap a DEK under `kek`: `nonce(12) ‖ ciphertext ‖ tag(16)`.
fn wrap_dek(
    kek: &Cipher,
    dek: &[u8; 32],
    artifact: Artifact,
    key_version: u8,
) -> Result<Vec<u8>, CryptoError> {
    let mut nonce = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce);
    let aad = dek_aad(artifact, key_version);
    let sealed = kek.encrypt_with_nonce(&nonce, dek, &aad)?;
    let mut wrapped = Vec::with_capacity(nonce.len() + sealed.len());
    wrapped.extend_from_slice(&nonce);
    wrapped.extend_from_slice(&sealed);
    Ok(wrapped)
}

/// Wrap the key-check sentinel: `nonce(12) ‖ ciphertext ‖ tag(16)`.
fn wrap_key_check(kek: &Cipher, key_version: u8) -> Result<Vec<u8>, CryptoError> {
    let sentinel = random_bytes::<32>();
    let mut nonce = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce);
    let aad = key_check_aad(key_version);
    let sealed = kek.encrypt_with_nonce(&nonce, &sentinel, &aad)?;
    let mut wrapped = Vec::with_capacity(nonce.len() + sealed.len());
    wrapped.extend_from_slice(&nonce);
    wrapped.extend_from_slice(&sealed);
    Ok(wrapped)
}

/// Verify that the master key unwraps the sentinel.
fn unwrap_key_check(kek: &Cipher, key_check: &[u8], key_version: u8) -> Result<(), CryptoError> {
    if key_check.len() < 12 + 16 {
        return Err(CryptoError::RegistryInvalid(
            "key check sentinel is truncated".into(),
        ));
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&key_check[..12]);
    let aad = key_check_aad(key_version);
    kek.decrypt_with_nonce(&nonce, &key_check[12..], &aad)
        .map(|_| ())
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// AAD of a wrapped DEK: `"vanta-dek-v1" ‖ artifact_id ‖ key_version`.
fn dek_aad(artifact: Artifact, key_version: u8) -> Vec<u8> {
    let mut aad = Vec::with_capacity(DEK_AAD_PREFIX.len() + 3);
    aad.extend_from_slice(DEK_AAD_PREFIX);
    aad.extend_from_slice(&artifact.code().to_be_bytes());
    aad.push(key_version);
    aad
}

/// AAD of the wrapped key-check sentinel.
fn key_check_aad(key_version: u8) -> Vec<u8> {
    let mut aad = Vec::with_capacity(KEY_CHECK_AAD_PREFIX.len() + 1);
    aad.extend_from_slice(KEY_CHECK_AAD_PREFIX);
    aad.push(key_version);
    aad
}

/// Unwrap a DEK under the KEK.
fn unwrap_dek(
    kek: &Cipher,
    wrapped: &[u8],
    artifact: Artifact,
    key_version: u8,
) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    if wrapped.len() < 12 + 16 {
        return Err(CryptoError::RegistryInvalid(format!(
            "wrapped DEK of {} is truncated",
            artifact.as_str()
        )));
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&wrapped[..12]);
    let aad = dek_aad(artifact, key_version);
    let dek = kek.decrypt_with_nonce(&nonce, &wrapped[12..], &aad)?;
    if dek.len() != 32 {
        return Err(CryptoError::RegistryInvalid(format!(
            "wrapped DEK of {} has an unexpected length",
            artifact.as_str()
        )));
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&dek);
    Ok(Zeroizing::new(bytes))
}

/// Stretch the key material into a 32-byte master key.
///
/// A 32-byte material **is** the master. A shorter one is a passphrase and is
/// stretched once with PBKDF2-HMAC-SHA256 and the database salt
/// (ADR-0057 C-4) — the recommended form remains a 32-byte random key.
fn master_from_material(
    material: &[u8],
    db_salt: &[u8; 32],
) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    if material.len() == 32 {
        let mut master = [0u8; 32];
        master.copy_from_slice(material);
        return Ok(Zeroizing::new(master));
    }
    let mut master = [0u8; 32];
    let iterations = std::num::NonZeroU32::new(crate::crypto::KDF_ITERATIONS)
        .ok_or_else(|| CryptoError::InvalidKey("PBKDF2 iteration count is zero".into()))?;
    ring::pbkdf2::derive(
        ring::pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        db_salt,
        material,
        &mut master,
    );
    Ok(Zeroizing::new(master))
}

/// Fill a buffer with cryptographically secure random bytes.
fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    rand::rng().fill_bytes(&mut buf);
    buf
}

/// The decoded content of a registry file.
struct RegistryFile {
    key_version: u8,
    db_salt: [u8; 32],
    key_check: Vec<u8>,
    entries: Vec<FileEntry>,
}

/// One decoded entry of a registry file.
struct FileEntry {
    artifact: u16,
    dek_id: [u8; DEK_ID_LEN],
    wrapped: Vec<u8>,
    epoch: u64,
}

/// Decode a registry file.
fn decode(bytes: &[u8]) -> Result<RegistryFile, CryptoError> {
    if bytes.len() < 4 {
        return Err(CryptoError::RegistryInvalid("file is too short".into()));
    }
    let (body, crc) = bytes.split_at(bytes.len() - 4);
    let expected = u32::from_be_bytes([crc[0], crc[1], crc[2], crc[3]]);
    if crc32c::crc32c(body) != expected {
        return Err(CryptoError::RegistryInvalid("CRC mismatch".into()));
    }
    let mut reader = Reader { data: body, pos: 0 };
    let magic = reader.take(8)?;
    if magic != REGISTRY_MAGIC {
        return Err(CryptoError::RegistryInvalid("bad magic".into()));
    }
    let format = reader.u16()?;
    if format != REGISTRY_FORMAT {
        return Err(CryptoError::RegistryInvalid(format!(
            "unsupported registry format {format}"
        )));
    }
    let key_version = reader.u8()?;
    let _reserved = reader.u8()?;
    let mut db_salt = [0u8; 32];
    db_salt.copy_from_slice(reader.take(32)?);
    let key_check_len = reader.u32()? as usize;
    let key_check = reader.take(key_check_len)?.to_vec();
    let entry_count = reader.u32()? as usize;
    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let artifact = reader.u16()?;
        let mut dek_id = [0u8; DEK_ID_LEN];
        dek_id.copy_from_slice(reader.take(DEK_ID_LEN)?);
        let wrapped_len = reader.u32()? as usize;
        let wrapped = reader.take(wrapped_len)?.to_vec();
        let epoch = reader.u64()?;
        entries.push(FileEntry {
            artifact,
            dek_id,
            wrapped,
            epoch,
        });
    }
    Ok(RegistryFile {
        key_version,
        db_salt,
        key_check,
        entries,
    })
}

/// A bounds-checked cursor over the registry bytes.
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], CryptoError> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| CryptoError::RegistryInvalid("length overflow while decoding".into()))?;
        if end > self.data.len() {
            return Err(CryptoError::RegistryInvalid("truncated registry".into()));
        }
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, CryptoError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, CryptoError> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Result<u32, CryptoError> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u64(&mut self) -> Result<u64, CryptoError> {
        let b = self.take(8)?;
        Ok(u64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 32-byte master key.
    fn material(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    #[test]
    fn registry_wrap_unwrap_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let key = material(0x11);
        let mut registry = KeyRegistry::open_or_create(dir.path(), &key).unwrap();
        let dek = registry.ensure_dek(Artifact::Wal).unwrap();
        assert_eq!(dek.len(), 32);
        assert_ne!(&dek[..], &[0u8; 32][..], "the DEK must be random");

        // Reopening with the same key returns the same DEK.
        let reopened = KeyRegistry::open_or_create(dir.path(), &key).unwrap();
        assert_eq!(reopened.dek(Artifact::Wal).unwrap()[..], dek[..]);

        // Reopening with a wrong key fails: the key check does not unwrap.
        let err = KeyRegistry::open_or_create(dir.path(), &material(0x22)).unwrap_err();
        assert!(matches!(err, CryptoError::DecryptionFailed), "got: {err}");
    }

    #[test]
    fn registry_epoch_is_persisted_before_first_use() {
        let dir = tempfile::tempdir().unwrap();
        let key = material(0x33);
        let mut registry = KeyRegistry::open_or_create(dir.path(), &key).unwrap();
        let epoch = registry.begin_epoch(Artifact::Wal).unwrap();

        // The epoch is on disk before the first encryption under it.
        let bytes = std::fs::read(KeyRegistry::path_in(dir.path())).unwrap();
        let file = decode(&bytes).unwrap();
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].artifact, Artifact::Wal.code());
        assert_eq!(file.entries[0].epoch, epoch);
    }

    #[test]
    fn registry_keeps_only_the_current_epoch_per_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let key = material(0x44);
        let mut registry = KeyRegistry::open_or_create(dir.path(), &key).unwrap();
        let first = registry.begin_epoch(Artifact::Wal).unwrap();
        let second = registry.begin_epoch(Artifact::Wal).unwrap();
        assert_ne!(first, second, "every epoch is a fresh random value");

        let bytes = std::fs::read(KeyRegistry::path_in(dir.path())).unwrap();
        let file = decode(&bytes).unwrap();
        // One entry per artefact, holding the **current** epoch only — the
        // registry does not grow with the number of epochs.
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].epoch, second);
    }

    #[test]
    fn registry_write_is_atomic_tmp_rename() {
        let dir = tempfile::tempdir().unwrap();
        let key = material(0x55);
        let mut registry = KeyRegistry::open_or_create(dir.path(), &key).unwrap();
        registry.ensure_dek(Artifact::Wal).unwrap();

        let path = KeyRegistry::path_in(dir.path());
        let tmp = path.with_file_name(format!("{REGISTRY_FILE_NAME}.tmp"));
        // The tmp file does not survive the write: the rename consumed it.
        assert!(
            !tmp.exists(),
            "the temporary file must not survive the write"
        );
        assert!(path.exists());
        // The target holds a complete, parsable registry.
        let file = decode(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(file.entries.len(), 1);

        // A stale tmp file is overwritten, not appended to.
        std::fs::write(&tmp, b"garbage").unwrap();
        registry.begin_epoch(Artifact::Wal).unwrap();
        assert!(!tmp.exists(), "the stale temporary file must be consumed");
        let file = decode(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(file.entries.len(), 1);
    }

    #[test]
    fn nonce_is_unique_across_1000_encryptions_under_one_dek() {
        let dir = tempfile::tempdir().unwrap();
        let key = material(0x66);
        let mut registry = KeyRegistry::open_or_create(dir.path(), &key).unwrap();
        let epoch = registry.begin_epoch(Artifact::Wal).unwrap();
        let dek = registry.ensure_dek(Artifact::Wal).unwrap()[..].to_vec();
        let cipher = Cipher::new(&dek);

        let mut seen = std::collections::HashSet::new();
        for slot in 0..1000u64 {
            let nonce = registry.next_nonce(Artifact::Wal).unwrap();
            assert!(
                seen.insert(*nonce.as_bytes()),
                "the nonce repeated at slot {slot}"
            );
            let aad = AadBuilder::new(Artifact::Wal, 4)
                .epoch(epoch)
                .slot(slot)
                .plaintext_len(8)
                .build();
            let sealed = cipher
                .encrypt_with_nonce(nonce.as_bytes(), b"payload!", &aad)
                .unwrap();
            assert_eq!(
                cipher
                    .decrypt_with_nonce(nonce.as_bytes(), &sealed, &aad)
                    .unwrap(),
                b"payload!"
            );
        }
        assert_eq!(seen.len(), 1000);
    }

    #[test]
    fn nonce_does_not_repeat_after_a_simulated_restart_new_epoch() {
        let dir = tempfile::tempdir().unwrap();
        let key = material(0x77);
        let mut registry = KeyRegistry::open_or_create(dir.path(), &key).unwrap();
        let epoch_before = registry.begin_epoch(Artifact::Wal).unwrap();
        let before: Vec<Nonce> = (0..16)
            .map(|_| registry.next_nonce(Artifact::Wal).unwrap())
            .collect();

        // "Restart": the registry is reopened from disk — the in-memory
        // allocator is gone and a **new** epoch is allocated.
        let mut registry = KeyRegistry::open_or_create(dir.path(), &key).unwrap();
        let epoch_after = registry.begin_epoch(Artifact::Wal).unwrap();
        assert_ne!(
            epoch_before, epoch_after,
            "a restart must allocate a fresh random epoch"
        );

        let after: Vec<Nonce> = (0..16)
            .map(|_| registry.next_nonce(Artifact::Wal).unwrap())
            .collect();
        for nonce in &after {
            assert!(
                !before.contains(nonce),
                "a nonce repeated after the restart"
            );
        }
    }

    #[test]
    fn nonce_counter_overflow_allocates_a_new_epoch_instead_of_wrapping() {
        let dir = tempfile::tempdir().unwrap();
        let key = material(0x88);
        let mut registry = KeyRegistry::open_or_create(dir.path(), &key).unwrap();
        let epoch = registry.begin_epoch(Artifact::Wal).unwrap();
        let allocator = registry.allocator(Artifact::Wal).unwrap().clone();

        // Drive the counter to the edge.
        allocator.counter.store(u32::MAX - 1, Ordering::SeqCst);
        let last = allocator.next().unwrap();
        assert_eq!(last.counter(), u32::MAX - 1);

        // The next call must NOT wrap: it fails and the counter stays put.
        let err = allocator.next().unwrap_err();
        assert!(
            matches!(err, CryptoError::NonceEpochExhausted),
            "got: {err}"
        );
        assert_eq!(allocator.counter.load(Ordering::SeqCst), u32::MAX);

        // A new epoch is allocated instead — the counter restarts at 0.
        let new_epoch = registry.begin_epoch(Artifact::Wal).unwrap();
        assert_ne!(new_epoch, epoch);
        let fresh = registry.next_nonce(Artifact::Wal).unwrap();
        assert_eq!(fresh.epoch(), new_epoch);
        assert_eq!(fresh.counter(), 0);
        assert_ne!(fresh.as_bytes(), last.as_bytes());
    }

    #[test]
    fn aad_builder_is_deterministic_and_binds_slot_and_length() {
        let base = AadBuilder::new(Artifact::Wal, 4)
            .epoch(9)
            .slot(3)
            .plaintext_len(64);
        let same = AadBuilder::new(Artifact::Wal, 4)
            .epoch(9)
            .slot(3)
            .plaintext_len(64);
        assert_eq!(base.build(), same.build());

        // The slot and the plaintext length are bound.
        assert_ne!(base.build(), base.clone().slot(4).build());
        assert_ne!(base.build(), base.clone().plaintext_len(65).build());
        // So are the artefact, the format version and the epoch.
        assert_ne!(
            base.build(),
            AadBuilder::new(Artifact::VantaFile, 4)
                .epoch(9)
                .slot(3)
                .plaintext_len(64)
                .build()
        );
        assert_ne!(
            base.build(),
            AadBuilder::new(Artifact::Wal, 3)
                .epoch(9)
                .slot(3)
                .plaintext_len(64)
                .build()
        );
        assert_ne!(base.build(), base.clone().epoch(10).build());

        // The identifier is appended and bound too.
        let with_id = base.clone().identifier(b"partition\x00key");
        assert_ne!(base.build(), with_id.build());
        assert!(with_id.build().ends_with(b"partition\x00key"));
    }

    #[test]
    fn aad_mismatch_fails_authentication() {
        let cipher = Cipher::new(&[0x99u8; 32]);
        let nonce = [1u8; 12];
        let aad = AadBuilder::new(Artifact::Wal, 4)
            .epoch(1)
            .slot(7)
            .plaintext_len(5)
            .build();
        let sealed = cipher.encrypt_with_nonce(&nonce, b"hello", &aad).unwrap();

        // The same AAD decrypts.
        assert_eq!(
            cipher.decrypt_with_nonce(&nonce, &sealed, &aad).unwrap(),
            b"hello"
        );

        // A different AAD does not authenticate — and no plaintext leaks.
        let other = AadBuilder::new(Artifact::Wal, 4)
            .epoch(1)
            .slot(8)
            .plaintext_len(5)
            .build();
        let err = cipher
            .decrypt_with_nonce(&nonce, &sealed, &other)
            .unwrap_err();
        assert!(matches!(err, CryptoError::DecryptionFailed), "got: {err}");

        // A different nonce does not authenticate either.
        let err = cipher
            .decrypt_with_nonce(&[2u8; 12], &sealed, &aad)
            .unwrap_err();
        assert!(matches!(err, CryptoError::DecryptionFailed), "got: {err}");
    }
}
