use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tracing::warn;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::node::UnifiedNode;

/// Current WAL format version.
///
/// v1: original `Insert/Update/Delete/Checkpoint/Begin/Commit/Abort` (single-phase commit).
/// v2: adds `Prepare { txn_id, op_count }` for two-phase commit (RES-01, ACID Phase 4a).
///     - v2 binary reads v1 WAL fine (no `Prepare` present in old records, range-based compat).
///     - v1 binary reading v2 WAL: unknown postcard tag fails deserialization; scan-forward
///       skips affected records. Downgrade still requires dump/restore (same hint as today).
/// v3: adds the tamper-evident hash-chain (VER-01). Every record frame carries
///     `prev_hash ‖ record_hash` (32B each, SHA-256), where
///     `record_hash = SHA-256(prev_hash ‖ len_le ‖ payload ‖ crc_le)` and the first
///     record of a segment chains from [`CHAIN_GENESIS`].
///     - v3 readers read v1/v2 files with their legacy framing (range-based compat).
///     - v1/v2 readers reject v3 files as "newer format"; downgrade requires
///       dump/restore (unchanged hint). Existing v1/v2 files keep legacy framing
///       until rotation/compaction creates a fresh (v3) segment — no in-place rewrite.
pub const WAL_FORMAT_VERSION: u16 = 3;

/// Format version of the **encrypted** WAL (ADR-0057 W2).
///
/// The encrypted format wraps **every frame** in its own AEAD envelope:
///
/// ```text
/// [u32 ct_len][u32 crc32c(ct)][12B nonce][ct ‖ 16B tag][32B prev_hash][32B record_hash]
/// ```
///
/// * `ct_len` is the length of the AEAD blob (`ciphertext ‖ tag`), `crc` is the
///   CRC32C of that blob and the hash chain is computed over the **ciphertext**,
///   so `vanta-cli verify` keeps working **without the key**.
/// * The marker lives in the **existing cleartext header**
///   (`WalHeader.base.format_version`), so a reader decides by the bytes on
///   disk, never by configuration (ADR-0057 D1).
/// * The version bump happens **only in the encrypted format**: a v3 file
///   keeps its exact byte layout, and a WAL written without a key is
///   byte-identical to the one written before this change.
pub const WAL_ENCRYPTED_FORMAT_VERSION: u16 = 4;

/// The highest WAL format version this build can read.
///
/// v4 (the encrypted format) is only readable with the `encryption` feature:
/// a build without it refuses a v4 file with an explicit message.
pub const WAL_MAX_READABLE_VERSION: u16 = if cfg!(feature = "encryption") {
    WAL_ENCRYPTED_FORMAT_VERSION
} else {
    WAL_FORMAT_VERSION
};

/// Format version that introduced the chained frame layout. Files with
/// `format_version >= WAL_CHAIN_VERSION` carry `prev_hash ‖ record_hash` on
/// every frame; older files use the legacy `[len][payload][crc]` layout.
pub(crate) const WAL_CHAIN_VERSION: u16 = 3;

/// Length of one SHA-256 chain digest (`prev_hash` / `record_hash`).
pub(crate) const CHAIN_HASH_SIZE: usize = 32;

/// Extra bytes appended to every chained frame: `prev_hash ‖ record_hash`.
pub(crate) const CHAIN_FRAME_EXTRA: usize = 2 * CHAIN_HASH_SIZE;

/// Genesis link: the first record of a segment chains from this all-zero digest.
pub(crate) const CHAIN_GENESIS: [u8; CHAIN_HASH_SIZE] = [0u8; CHAIN_HASH_SIZE];

/// Tracks the postcard wire format version used for WAL record serialization.
/// Increment this when upgrading postcard to a potentially incompatible version.
/// Stored in Header.schema_version for forward-compatibility detection.
pub const WAL_POSTCARD_VERSION: u16 = 1;

const KIB: usize = 1024;
use crc32c::crc32c; // ← Import specific function to avoid namespace conflict

/// CRC32C (Castagnoli) using hardware-accelerated crate for performance
/// Falls back to pure Rust implementation if hardware acceleration unavailable
#[inline]
pub fn compute_crc32c(data: &[u8]) -> u32 {
    if data.is_empty() {
        // crc32c-0.5.0's `split` does from_raw_parts::<u64> with UB-check
        // preconditions; an empty slice (dangling, unaligned ptr) panics with
        // STATUS_STACK_BUFFER_OVERRUN instead of returning the CRC of empty
        // input (0x00000000). Guard here so backup manifest collection never
        // crashes on zero-length WAL shard files.
        return 0x0000_0000;
    }
    crc32c::crc32c(data)
}

// ─── Hash-chain primitives (VER-01) ────────────────────────

/// True when `format_version` uses the chained frame layout.
#[inline]
pub(crate) fn is_chained(format_version: u16) -> bool {
    format_version >= WAL_CHAIN_VERSION
}

/// True when `format_version` uses the **encrypted** (v4) frame layout.
#[inline]
pub(crate) fn is_encrypted(format_version: u16) -> bool {
    format_version >= WAL_ENCRYPTED_FORMAT_VERSION
}

/// Per-record framing overhead for a given format version (0 pre-chain,
/// [`CHAIN_FRAME_EXTRA`] chained).
#[inline]
pub(crate) fn frame_extra_for(format_version: u16) -> usize {
    if is_chained(format_version) {
        CHAIN_FRAME_EXTRA
    } else {
        0
    }
}

/// `record_hash = SHA-256(prev_hash ‖ len_le ‖ payload ‖ crc_le)`.
///
/// The hash commits to the previous link (removing, inserting or reordering a
/// frame breaks the successor's `prev_hash`) and to the full frame bytes
/// (altering the payload or recomputing its CRC breaks the `record_hash`).
/// Incremental: only the new frame is hashed — no full-file re-hash.
pub(crate) fn chain_hash(
    prev: &[u8; CHAIN_HASH_SIZE],
    len_le: &[u8; 4],
    payload: &[u8],
    crc_le: &[u8; 4],
) -> [u8; CHAIN_HASH_SIZE] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(prev);
    hasher.update(len_le);
    hasher.update(payload);
    hasher.update(crc_le);
    hasher.finalize().into()
}

// ─── WAL Record ────────────────────────────────────────────

/// WAL record types (postcard-serialized)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WalRecord {
    /// Insert a new node.
    Insert(UnifiedNode),
    /// Update an existing node.
    Update {
        /// Node ID to update.
        id: u128,
        /// New node data.
        node: UnifiedNode,
    },
    /// Delete a node by ID.
    Delete {
        /// Node ID to delete.
        id: u128,
    },
    /// Checkpoint with optional index checksum for integrity validation
    /// `index_checksum` is computed over serialized index state; None for backward compat
    /// `timestamp` allows ordering checkpoints for recovery decisions
    Checkpoint {
        /// Number of nodes at checkpoint time.
        node_count: u64,
        /// Optional CRC32C checksum of index state.
        index_checksum: Option<u32>,
        /// Optional timestamp in milliseconds.
        timestamp: Option<u64>,
    },
    /// Begin a transaction with the given ID.
    Begin(u64),
    /// Phase-1 marker for two-phase commit (WAL v2, RES-01 / ACID Phase 4a):
    /// all ops between `Begin` and this `Prepare` are durable on disk, but the
    /// transaction is not yet committed — apply to stores may still fail and emit
    /// a follow-up `Abort`. `op_count` is the number of ops between `Begin` and
    /// `Prepare` (integrity cross-check during replay).
    Prepare {
        /// Transaction being prepared.
        txn_id: u64,
        /// Number of ops buffered between Begin and Prepare.
        op_count: u32,
    },
    /// Commit a transaction with the given ID.
    Commit(u64),
    /// Abort a transaction with the given ID.
    Abort(u64),
}

// ─── WAL Header ────────────────────────────────────────────

/// WAL file header with magic bytes, version, schema version, and CRC integrity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalHeader {
    /// 16-byte Header (magic = `b"VWAL"`, version = 1, schema = 0, timestamp).
    pub base: crate::binary_header::Header,
    /// 4-byte CRC32C of the base header bytes.
    pub crc: u32,
}

impl WalHeader {
    /// Total header size in bytes (16 base + 4 CRC).
    pub const SIZE: usize = 20;

    /// Create a new WAL header.
    /// `format_version`: WAL format version (currently 1).
    /// Stores `WAL_POSTCARD_VERSION` in `schema_version` for forward-compatibility detection.
    pub fn new(format_version: u32) -> Self {
        let base = crate::binary_header::Header::new(
            *b"VWAL",
            format_version as u16,
            WAL_POSTCARD_VERSION,
        );
        let mut header = Self { base, crc: 0 };
        header.crc = header.compute_crc();
        header
    }

    /// Returns the postcard version recorded in this header's schema_version field.
    /// schema_version == 0 means legacy (pre-versioning), treated as v1.
    pub fn postcard_version(&self) -> u16 {
        if self.base.schema_version == 0 {
            1
        } else {
            self.base.schema_version
        }
    }

    /// Compute CRC32C of the base header bytes.
    pub fn compute_crc(&self) -> u32 {
        let bytes = self.base.serialize();
        crc32c(&bytes)
    }

    /// Serialize the header into a 20-byte array.
    pub fn serialize(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..16].copy_from_slice(&self.base.serialize());
        bytes[16..20].copy_from_slice(&self.crc.to_le_bytes());
        bytes
    }

    /// Deserialize a header from bytes, validating magic, CRC, and version.
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::SIZE {
            return Err(Error::wal_error(format!(
                "Invalid WAL header size: expected {}, got {}",
                Self::SIZE,
                bytes.len()
            )));
        }

        let base = crate::binary_header::Header::deserialize(&bytes[0..16])?;

        // The magic first: a file that is not a WAL at all is reported as
        // such, whatever its version field happens to say.
        if base.magic != *b"VWAL" {
            return Err(Error::IncompatibleFormat {
                expected_magic: *b"VWAL",
                expected_version: WAL_MAX_READABLE_VERSION,
                found_magic: base.magic,
                found_version: base.format_version,
                hint: format!(
                    "WAL format: expected magic {:?}, found {:?}",
                    *b"VWAL", base.magic
                ),
            });
        }

        // A v4 WAL is encrypted: this build can only read it with the
        // `encryption` feature compiled in. Say so explicitly — an encrypted
        // database must never be reported as "corrupt" (ADR-0057 D7).
        if is_encrypted(base.format_version) && !cfg!(feature = "encryption") {
            return Err(Error::wal_error(
                "WAL file is encrypted (format v4 or newer) but this build has no 'encryption' feature",
            ));
        }

        // Range-based compatibility check: accepts any format_version ≤ the
        // highest version this build understands, with matching magic.
        // Future-format files (version > current) are rejected; older files
        // with matching magic are accepted for forward-compatible reads.
        // The encrypted format (v4) is readable only with the feature.
        base.validate_compat(*b"VWAL", WAL_MAX_READABLE_VERSION, "WAL format")?;

        // Version 0 WAL was never a valid format — reject explicitly
        if base.format_version < 1 {
            return Err(Error::WALVersionMismatch {
                expected: WAL_FORMAT_VERSION as u32,
                found: base.format_version as u32,
                hint: "Delete WAL dir or run dump/restore before upgrading.".to_string(),
            });
        }

        let crc = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let header = Self { base, crc };

        let computed_crc = header.compute_crc();
        if computed_crc != crc {
            return Err(Error::wal_error(format!(
                "WAL header CRC mismatch: stored={:#x}, computed={:#x}",
                crc, computed_crc
            )));
        }

        let recorded_pc = header.postcard_version();
        if recorded_pc != WAL_POSTCARD_VERSION {
            warn!(
                "WAL was written with postcard v{}, current is v{}. \
                 Records may fail to deserialize if the wire format changed.",
                recorded_pc, WAL_POSTCARD_VERSION
            );
        }

        Ok(header)
    }
}

// ─── Encryption at rest (ADR-0057 W2) ──────────────────────
//
// The WAL is the first artefact wired to the at-rest envelope (ADR-0057
// §D1–§D5):
//
// * **Per frame**, not per file: every frame carries its own AEAD envelope
//   (`[ct_len][crc][nonce][ct‖tag][prev][record_hash]`).
// * The **CRC32C covers the ciphertext** and the **hash chain is computed over
//   the ciphertext** → `vanta-cli verify` keeps working **without the key**
//   (the AEAD tag, not the CRC, is what detects tampering).
// * The **AAD** binds the ciphertext to its position — `artifact=WAL,
//   format_version=4, epoch, slot (the frame ordinal within the file),
//   plaintext_len` (ADR-0057 D4) — so a frame moved to another ordinal fails
//   to authenticate.
// * The **nonce** is `epoch ‖ counter` (ADR-0057 D3): the epoch is random and
//   persisted **before** the first encryption under it; the counter is a
//   lock-free `AtomicU32` that never wraps (an exhausted counter allocates a
//   new epoch).
// * **No new lock in the hot path**: the nonce comes from one atomic
//   increment; the registry is touched only when the epoch is rolled (open /
//   counter exhaustion), never per append.
//
// The **slot** (the frame ordinal within its file) is the reason the
// encryption happens while the writer is held exclusively: the ordinal is the
// number of frames already written to *that* file, so it is only known under
// the writer's own lock. ADR-0057 D5 asks for "encrypt before the shard
// mutex"; that is not reachable with an ordinal-bound AAD, and the deviation
// is recorded in the wave report (the GCM cost of one small frame is
// negligible next to its `write`).

#[cfg(feature = "encryption")]
use crate::crypto::registry::NonceAllocator;
#[cfg(feature = "encryption")]
use crate::crypto::{AadBuilder, Artifact, Cipher, CryptoError, KeyRegistry, Nonce};
#[cfg(feature = "encryption")]
use parking_lot::Mutex;
#[cfg(feature = "encryption")]
use std::sync::Arc;

/// The encryption slot of a WAL writer.
///
/// With the `encryption` feature it holds the real [`WalCrypto`]; without it
/// the slot is uninhabited and every WAL is plaintext (byte-identical to the
/// pre-encryption layout).
#[cfg(feature = "encryption")]
pub(crate) type WalCryptoSlot = WalCrypto;
#[cfg(not(feature = "encryption"))]
pub(crate) type WalCryptoSlot = ();

/// The decryption key slot of a WAL reader.
///
/// With the `encryption` feature it holds the real [`WalKey`]; without it the
/// slot is uninhabited and an encrypted WAL cannot be opened at all.
#[cfg(feature = "encryption")]
type WalKeySlot = WalKey;
#[cfg(not(feature = "encryption"))]
type WalKeySlot = ();

/// The **read side** of an encrypted WAL: the DEK of the `wal` artefact.
///
/// Without the `encryption` feature the type is empty — an encrypted WAL
/// cannot even be opened, so the key is never needed.
#[cfg(feature = "encryption")]
#[derive(Clone, Debug)]
pub struct WalKey {
    cipher: Cipher,
}

/// The **read side** of an encrypted WAL (stub without the `encryption`
/// feature).
#[cfg(not(feature = "encryption"))]
#[derive(Debug, Clone, Copy)]
pub struct WalKey;

#[cfg(feature = "encryption")]
impl WalKey {
    /// Open the DEK of the WAL artefact living in `data_dir` (read-only).
    ///
    /// The registry holds the wrapped DEKs, so it must already exist: a
    /// missing registry means the DEK is gone and the database cannot be
    /// read (ADR-0057 D2 — refuse, never "corrupt").
    pub fn open(data_dir: &Path, key_material: &[u8]) -> Result<Self> {
        if !KeyRegistry::path_in(data_dir).exists() {
            return Err(Error::EncryptionKeyRequired {
                artifact: Artifact::Wal.as_str().to_string(),
            });
        }
        let registry = KeyRegistry::open_or_create(data_dir, key_material).map_err(crypto_error)?;
        let dek = registry
            .dek(Artifact::Wal)
            .ok_or_else(|| Error::EncryptionKeyRequired {
                artifact: Artifact::Wal.as_str().to_string(),
            })?;
        Ok(Self {
            cipher: Cipher::new(&dek[..]),
        })
    }

    /// The DEK cipher of the artefact.
    pub fn cipher(&self) -> &Cipher {
        &self.cipher
    }
}

/// The **write side** of an encrypted WAL: the DEK, the nonce allocator and
/// the registry handle needed to roll the epoch.
#[cfg(feature = "encryption")]
#[derive(Clone, Debug)]
pub struct WalCrypto {
    /// The read-side key (the DEK of the `wal` artefact).
    key: WalKey,
    /// The nonce allocator of the artefact — **shared by every shard**: one DEK
    /// means one nonce space, and a per-shard counter would reuse nonces.
    allocator: Arc<NonceAllocator>,
    /// The registry — locked only to roll the epoch (counter exhaustion).
    registry: Arc<Mutex<KeyRegistry>>,
}

#[cfg(feature = "encryption")]
impl WalCrypto {
    /// Open (or create) the write context of the WAL artefact in `data_dir`.
    ///
    /// Allocates a **fresh epoch** and persists it **before** the first
    /// encryption under it (ADR-0057 D3).
    pub fn open(data_dir: &Path, key_material: &[u8]) -> Result<Self> {
        let mut registry =
            KeyRegistry::open_or_create(data_dir, key_material).map_err(crypto_error)?;
        registry.ensure_dek(Artifact::Wal).map_err(crypto_error)?;
        registry.begin_epoch(Artifact::Wal).map_err(crypto_error)?;
        let allocator = registry.allocator(Artifact::Wal).cloned().ok_or_else(|| {
            Error::wal_error("encryption registry has no nonce allocator for the wal artefact")
        })?;
        let dek = registry
            .dek(Artifact::Wal)
            .ok_or_else(|| Error::EncryptionKeyRequired {
                artifact: Artifact::Wal.as_str().to_string(),
            })?;
        Ok(Self {
            key: WalKey {
                cipher: Cipher::new(&dek[..]),
            },
            allocator,
            registry: Arc::new(Mutex::new(registry)),
        })
    }

    /// The read-side key of this context.
    pub fn key(&self) -> WalKey {
        self.key.clone()
    }

    /// Build one encrypted frame and return it with the new chain head.
    ///
    /// Layout: `[u32 ct_len][u32 crc32c(ct)][12B nonce][ct ‖ 16B tag]
    /// [32B prev_hash][32B record_hash]`.
    fn frame(
        &mut self,
        payload: &[u8],
        slot: u64,
        prev: &[u8; CHAIN_HASH_SIZE],
    ) -> Result<(Vec<u8>, [u8; CHAIN_HASH_SIZE])> {
        let nonce = self.next_nonce()?;
        let aad = AadBuilder::new(Artifact::Wal, WAL_ENCRYPTED_FORMAT_VERSION)
            .epoch(nonce.epoch())
            .slot(slot)
            .plaintext_len(payload.len())
            .build();
        // `ct` is the AEAD blob: `ciphertext ‖ 16-byte tag`.
        let ct = self
            .key
            .cipher
            .encrypt_with_nonce(nonce.as_bytes(), payload, &aad)
            .map_err(crypto_error)?;
        let ct_len = ct.len() as u32;
        let crc = compute_crc32c(&ct);
        let record_hash = chain_hash_v4(
            prev,
            &ct_len.to_le_bytes(),
            &crc.to_le_bytes(),
            nonce.as_bytes(),
            &ct,
        );
        let mut frame = Vec::with_capacity(4 + 4 + 12 + ct.len() + CHAIN_FRAME_EXTRA);
        frame.extend_from_slice(&ct_len.to_le_bytes());
        frame.extend_from_slice(&crc.to_le_bytes());
        frame.extend_from_slice(nonce.as_bytes());
        frame.extend_from_slice(&ct);
        frame.extend_from_slice(prev);
        frame.extend_from_slice(&record_hash);
        Ok((frame, record_hash))
    }

    /// The next nonce of the artefact.
    ///
    /// The counter **never wraps**: when it is exhausted a new epoch is
    /// allocated (and persisted) instead (ADR-0057 D3).
    fn next_nonce(&mut self) -> Result<Nonce> {
        match self.allocator.next() {
            Ok(nonce) => Ok(nonce),
            Err(CryptoError::NonceEpochExhausted) => {
                self.roll_epoch()?;
                self.allocator.next().map_err(crypto_error)
            }
            Err(err) => Err(crypto_error(err)),
        }
    }

    /// Roll the nonce epoch: a fresh random epoch, persisted **before** the
    /// first encryption under it.
    ///
    /// When another writer already rolled the epoch (the allocator is shared
    /// by every shard) its allocator is adopted instead of burning a second
    /// epoch — the artefact keeps a single, shared nonce sequence.
    fn roll_epoch(&mut self) -> Result<()> {
        let registry_arc = self.registry.clone();
        let mut registry = registry_arc.lock();
        if let Some(current) = registry.allocator(Artifact::Wal) {
            if !Arc::ptr_eq(current, &self.allocator) {
                self.allocator = current.clone();
                return Ok(());
            }
        }
        registry.begin_epoch(Artifact::Wal).map_err(crypto_error)?;
        self.allocator = registry.allocator(Artifact::Wal).cloned().ok_or_else(|| {
            Error::wal_error("encryption registry has no nonce allocator for the wal artefact")
        })?;
        Ok(())
    }
}

/// Build the WAL encryption context from the configuration, if a key resolves.
///
/// Precedence: environment → key file → key command (ADR-0057 D2). Without a
/// key the WAL stays **plaintext and byte-identical** to the layout before
/// this change.
#[cfg(feature = "encryption")]
pub fn wal_crypto_from_config(
    data_dir: &Path,
    config: &crate::config::Config,
) -> Result<Option<WalCrypto>> {
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
    Ok(Some(WalCrypto::open(data_dir, &material[..])?))
}

/// Map a crypto failure onto the crate error surface (ADR-0057 D6).
#[cfg(feature = "encryption")]
fn crypto_error(err: CryptoError) -> Error {
    match err {
        CryptoError::DecryptionFailed => Error::EncryptionKeyRejected {
            artifact: Artifact::Wal.as_str().to_string(),
        },
        CryptoError::RegistryInvalid(detail) => Error::EncryptionCorrupt {
            artifact: Artifact::Wal.as_str().to_string(),
            detail,
        },
        other => Error::wal_error(format!("WAL encryption error: {other}")),
    }
}

/// `record_hash = SHA-256(prev_hash ‖ ct_len_le ‖ crc_le ‖ nonce ‖ ct)` (v4).
///
/// The hash commits to the **ciphertext** (and to the nonce), so the chain
/// stays verifiable by an operator who has no key (ADR-0057 D5).
#[cfg(feature = "encryption")]
fn chain_hash_v4(
    prev: &[u8; CHAIN_HASH_SIZE],
    ct_len_le: &[u8],
    crc_le: &[u8],
    nonce: &[u8; 12],
    ct: &[u8],
) -> [u8; CHAIN_HASH_SIZE] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(prev);
    hasher.update(ct_len_le);
    hasher.update(crc_le);
    hasher.update(nonce);
    hasher.update(ct);
    hasher.finalize().into()
}

/// The format version written to a **new** segment.
///
/// A writer holding an encryption context writes the encrypted format (v4);
/// every other writer keeps the plaintext format (v3), whose bytes are
/// unchanged by this feature.
fn segment_version_for(encrypted: bool) -> u16 {
    if encrypted {
        WAL_ENCRYPTED_FORMAT_VERSION
    } else {
        WAL_FORMAT_VERSION
    }
}

// ─── WAL Writer ────────────────────────────────────────────
//
// DURABILITY DAG — FIND-34 (2026-08-27): This module is a DAG, not a cycle.
// ```text
//   open ──► open_with_buffer ──► recover_valid_records ──┐
//                              └─► quarantine_corrupt_tail ─► quarantine_backup_path
//   helpers (leaves): check_record_at, scan_forward_valid, try_scan_forward
// ```
// No back-edge: `recover_valid_records` / `quarantine_corrupt_tail` never call
// `open` or `open_with_buffer`. CodeGraph reported a 4-node "cycle"
// (`open↔open_with_buffer↔recover↔quarantine`) via Leiden co-localisation
// (all 4 fns in `wal.rs:202-575`), not an SCC via CALLS edges. Verified via
// `rg -n "recover_valid_records|quarantine_corrupt_tail" src/wal.rs` (1 def each,
// callers only in `open_with_buffer`). Recovery is crash-safe: `quarantine`
// fails soft (warn + truncate anyway) so `open_with_buffer` never depends on
// backup success.
//
// ponytail: doc justifica falso positivo sin refactor; extraer helper si SCC real emerge.

/// Append-only WAL writer with CRC32C integrity checks and structured header.
///
/// File format: \[WalHeader(20 bytes)\]\[Record1\]\[Record2\]...
/// Record format: \[len:u32\]\[payload:postcard\]\[crc:u32\]
/// Append-only WAL writer with CRC32C integrity checks and structured header.
pub struct WalWriter {
    writer: BufWriter<File>,
    path: PathBuf,
    bytes_written: u64,
    record_count: u64,
    /// Whether to sync to disk on every write or periodically.
    pub sync_mode: crate::config::SyncMode,
    /// Number of records written since the last sync.
    records_since_sync: u64,
    /// If `Some(N)`, auto-sync after N records when sync_mode is Periodic.
    /// Ignored when sync_mode is Never (no auto-sync; use `sync()` explicitly).
    flush_threshold: Option<usize>,
    /// Maximum segment size in bytes before auto-rotation (default: 256MB).
    max_segment_size: u64,
    /// On-disk format version of the opened file — selects the frame layout
    /// (`WAL_CHAIN_VERSION`+ = chained, older = legacy).
    format_version: u16,
    /// Running chain head: the `record_hash` of the last appended record, or
    /// [`CHAIN_GENESIS`] when the file has no records yet (VER-01).
    chain_head: [u8; CHAIN_HASH_SIZE],
    /// The encryption context — `None` on a plaintext WAL (v3). With the
    /// `encryption` feature the slot holds the real context; without it the
    /// slot is uninhabited and every WAL is plaintext.
    crypto: Option<WalCryptoSlot>,
}

impl WalWriter {
    /// Open or create WAL file, writing or validating WalHeader.
    pub fn open(path: impl AsRef<Path>, sync_mode: crate::config::SyncMode) -> Result<Self> {
        Self::open_with_buffer(path, sync_mode, 64 * KIB, None)
    }

    /// Open or create WAL file with configurable buffer size and flush threshold.
    ///
    /// * `buffer_size` — capacity of the inner `BufWriter` (default: 64 KB).
    /// * `flush_threshold` — if `Some(N)`, auto-sync after N records.
    pub fn open_with_buffer(
        path: impl AsRef<Path>,
        sync_mode: crate::config::SyncMode,
        buffer_size: usize,
        flush_threshold: Option<usize>,
    ) -> Result<Self> {
        Self::open_impl(path, sync_mode, buffer_size, flush_threshold, None)
    }

    /// Open or create an **encrypted** WAL (format v4, ADR-0057 W2).
    ///
    /// The context carries the DEK of the `wal` artefact and the nonce
    /// allocator shared by every shard of the database.
    #[cfg(feature = "encryption")]
    pub fn open_encrypted(
        path: impl AsRef<Path>,
        sync_mode: crate::config::SyncMode,
        crypto: WalCrypto,
    ) -> Result<Self> {
        Self::open_encrypted_with_buffer(path, sync_mode, 64 * KIB, None, crypto)
    }

    /// Open or create an **encrypted** WAL with a configurable buffer size and
    /// flush threshold.
    #[cfg(feature = "encryption")]
    pub fn open_encrypted_with_buffer(
        path: impl AsRef<Path>,
        sync_mode: crate::config::SyncMode,
        buffer_size: usize,
        flush_threshold: Option<usize>,
        crypto: WalCrypto,
    ) -> Result<Self> {
        Self::open_impl(path, sync_mode, buffer_size, flush_threshold, Some(crypto))
    }

    /// The format version written to a **new** segment.
    ///
    /// A writer holding an encryption context writes the encrypted format
    /// (v4); every other writer keeps the plaintext format (v3), whose bytes
    /// are unchanged by this feature.
    fn segment_version(&self) -> u16 {
        segment_version_for(self.crypto.is_some())
    }

    fn open_impl(
        path: impl AsRef<Path>,
        sync_mode: crate::config::SyncMode,
        buffer_size: usize,
        flush_threshold: Option<usize>,
        crypto: Option<WalCryptoSlot>,
    ) -> Result<Self> {
        // The encryption context of this writer. A plaintext WAL keeps
        // `None`; an encrypted one keeps the context for the whole life of
        // the file.
        #[cfg(feature = "encryption")]
        let mut crypto = crypto;
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false) // ← Explicit for Clippy: preserve existing WAL data for recovery
            .open(&path)?;

        let file_len = file.metadata()?.len();
        let (bytes_written, record_count, format_version, chain_head) = if file_len == 0 {
            // A new segment is encrypted (v4) when a key is configured and
            // plaintext (v3) otherwise — the plaintext bytes never change.
            let version = self::segment_version_for(crypto.is_some());
            let header = WalHeader::new(version as u32);
            file.write_all(&header.serialize())?;
            file.flush()?;
            // POSIX: fsync on the file does NOT make its directory entry durable
            // ("Calling fsync() does not necessarily ensure that the entry in the
            // directory containing the file has also reached disk" — man fsync(2)).
            // Without this, a freshly created WAL whose appends were already
            // sync_data'd can vanish on power loss, losing acknowledged writes.
            // Same pattern as rotate() below and archive.rs (AUDREP-35).
            crate::utils::fs::sync_parent_dir(&path)?;
            (WalHeader::SIZE as u64, 0u64, version, CHAIN_GENESIS)
        } else {
            // Read the existing header
            let mut header_bytes = [0u8; WalHeader::SIZE];
            file.seek(SeekFrom::Start(0))?;
            file.read_exact(&mut header_bytes)?;
            let header = WalHeader::deserialize(&header_bytes)?;
            let version = header.base.format_version;

            // An encrypted WAL needs the key: without one the open fails with
            // an explicit error, never with "corrupt" (ADR-0057 D7).
            #[cfg(feature = "encryption")]
            if is_encrypted(version) && crypto.is_none() {
                return Err(Error::EncryptionKeyRequired {
                    artifact: Artifact::Wal.as_str().to_string(),
                });
            }
            // A plaintext WAL stays plaintext even when a key is configured:
            // no mixed mode, no implicit migration (ADR-0057 D1/D7).
            #[cfg(feature = "encryption")]
            if !is_encrypted(version) && crypto.is_some() {
                warn!(
                    path = %path.display(),
                    "encryption key configured but this WAL is plaintext; \
                     continuing in plaintext (no implicit migration)"
                );
                crypto = None;
            }

            #[cfg(feature = "encryption")]
            let (valid_end, count) = if is_encrypted(version) {
                let key = crypto.as_ref().map(|c| &c.key).ok_or_else(|| {
                    Error::EncryptionKeyRequired {
                        artifact: Artifact::Wal.as_str().to_string(),
                    }
                })?;
                recover_valid_records_v4(&path, file_len, key)?
            } else {
                recover_valid_records(&path, file_len, frame_extra_for(version))?
            };
            #[cfg(not(feature = "encryption"))]
            let (valid_end, count) =
                recover_valid_records(&path, file_len, frame_extra_for(version))?;

            if file_len > valid_end {
                warn!(
                    path = %path.display(),
                    expected_len = file_len,
                    valid_len = valid_end,
                    "Truncating corrupt or incomplete records at the end of WAL"
                );
                // Quarantine the corrupt tail BEFORE truncating it away
                // permanently, so the bytes stay recoverable for forensics.
                quarantine_corrupt_tail(&path, valid_end, file_len);
                file.set_len(valid_end)?;
            }

            // Recover the chain head so appends continue the chain across
            // reopens (VER-01). `record_hash` is the trailing 32 bytes of the
            // last valid chained frame; a header-only file starts at genesis.
            let mut chain_head = CHAIN_GENESIS;
            if is_chained(version) && count > 0 {
                file.seek(SeekFrom::Start(valid_end - CHAIN_HASH_SIZE as u64))?;
                file.read_exact(&mut chain_head)?;
            }

            file.seek(SeekFrom::Start(valid_end))?;
            (valid_end, count as u64, version, chain_head)
        };

        let buffer_size = buffer_size.clamp(KIB, 32 * 1024 * KIB);

        Ok(Self {
            writer: BufWriter::with_capacity(buffer_size, file),
            path,
            bytes_written,
            record_count,
            sync_mode,
            records_since_sync: 0,
            flush_threshold,
            max_segment_size: 256 * 1024 * 1024,
            format_version,
            chain_head,
            crypto,
        })
    }

    /// Append a single record to the WAL
    pub fn append(&mut self, record: &WalRecord) -> Result<()> {
        #[cfg(feature = "failpoints")]
        fail::fail_point!("wal_append_fail", |_| {
            Err(Error::Io(std::io::Error::other(
                "Simulated WAL append catastrophic I/O failure",
            )))
        });

        let payload = postcard::to_allocvec(record).map_err(Error::serialization)?;

        // Encrypted frame (v4): the AEAD envelope replaces the plaintext
        // payload and the CRC covers the **ciphertext**.
        #[cfg(feature = "encryption")]
        if let Some(crypto) = self.crypto.as_mut() {
            // The slot is the frame's ordinal within this file — the number of
            // frames already written to it (ADR-0057 D4).
            let slot = self.record_count;
            let prev = self.chain_head;
            let (frame, record_hash) = crypto.frame(&payload, slot, &prev)?;
            self.writer.write_all(&frame)?;
            self.chain_head = record_hash;
            self.bytes_written += frame.len() as u64;
            self.record_count += 1;
            self.records_since_sync += 1;
            self.maybe_sync()?;
            self.try_auto_rotate()?;
            return Ok(());
        }

        let len = payload.len() as u32;
        let crc = crc32c(&payload);

        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(&payload)?;
        self.writer.write_all(&crc.to_le_bytes())?;
        if is_chained(self.format_version) {
            // VER-01: prev_hash ‖ record_hash — the running chain head is the
            // previous record's `record_hash` (genesis for the first record).
            let prev = self.chain_head;
            let record_hash = chain_hash(&prev, &len.to_le_bytes(), &payload, &crc.to_le_bytes());
            self.writer.write_all(&prev)?;
            self.writer.write_all(&record_hash)?;
            self.chain_head = record_hash;
        }

        self.bytes_written +=
            4 + payload.len() as u64 + 4 + frame_extra_for(self.format_version) as u64;
        self.record_count += 1;
        self.records_since_sync += 1;

        self.maybe_sync()?;
        self.try_auto_rotate()?;
        Ok(())
    }

    /// Append multiple records in a single write call to reduce I/O overhead.
    pub fn batch_append(&mut self, records: &[WalRecord]) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        #[cfg(feature = "failpoints")]
        fail::fail_point!("wal_append_fail", |_| {
            Err(Error::Io(std::io::Error::other(
                "Simulated WAL append catastrophic I/O failure",
            )))
        });

        // Encrypted frames (v4): the whole group is sealed before the single
        // write, so the critical section only grows with the memcpy.
        #[cfg(feature = "encryption")]
        if self.crypto.is_some() {
            return self.batch_append_encrypted(records);
        }

        let estimated = records.len() * 128;
        let mut buf = Vec::with_capacity(estimated);
        // Reusable serialization buffer: one allocation for the whole batch
        // instead of one `to_allocvec` per record. `clear()` keeps the capacity,
        // so the payload Vec never reallocates across records of similar size.
        // On-disk framing is unchanged ([len u32 LE][payload][crc u32 LE]).
        let mut payload = Vec::with_capacity(128);

        for record in records {
            payload.clear();
            postcard::to_io(record, &mut payload).map_err(Error::serialization)?;
            let len = payload.len() as u32;
            let crc = crc32c(&payload);
            buf.extend_from_slice(&len.to_le_bytes());
            buf.extend_from_slice(&payload);
            buf.extend_from_slice(&crc.to_le_bytes());
            if is_chained(self.format_version) {
                // VER-01: same chain rule as `append` (byte-identical framing).
                let prev = self.chain_head;
                let record_hash =
                    chain_hash(&prev, &len.to_le_bytes(), &payload, &crc.to_le_bytes());
                buf.extend_from_slice(&prev);
                buf.extend_from_slice(&record_hash);
                self.chain_head = record_hash;
            }
        }

        self.writer.write_all(&buf)?;
        self.bytes_written += buf.len() as u64;
        self.record_count += records.len() as u64;
        self.records_since_sync += records.len() as u64;

        self.maybe_sync()?;
        self.try_auto_rotate()?;
        Ok(())
    }

    /// Append a batch of records as **encrypted** frames (v4).
    ///
    /// Every frame is sealed before the single write, so the group costs one
    /// `write_all` — the same shape as the plaintext path.
    #[cfg(feature = "encryption")]
    fn batch_append_encrypted(&mut self, records: &[WalRecord]) -> Result<()> {
        let mut buf = Vec::with_capacity(records.len() * 160);
        let mut payload = Vec::with_capacity(128);
        let mut slot = self.record_count;
        let mut prev = self.chain_head;

        {
            let crypto = self.crypto.as_mut().ok_or_else(|| {
                Error::wal_error("WAL writer lost its encryption context mid-batch")
            })?;
            for record in records {
                payload.clear();
                postcard::to_io(record, &mut payload).map_err(Error::serialization)?;
                let (frame, record_hash) = crypto.frame(&payload, slot, &prev)?;
                buf.extend_from_slice(&frame);
                prev = record_hash;
                slot += 1;
            }
        }

        self.writer.write_all(&buf)?;
        self.chain_head = prev;
        self.bytes_written += buf.len() as u64;
        self.record_count += records.len() as u64;
        self.records_since_sync += records.len() as u64;

        self.maybe_sync()?;
        self.try_auto_rotate()?;
        Ok(())
    }

    /// Conditionally sync based on sync mode and flush threshold.
    /// Default threshold for `Periodic` when none is configured: 1 (sync every write)
    /// to avoid losing more than one record on crash.
    const DEFAULT_PERIODIC_THRESHOLD: u64 = 1;

    fn maybe_sync(&mut self) -> Result<()> {
        match self.sync_mode {
            crate::config::SyncMode::Always => self.sync()?,
            // `Never` disables automatic syncing entirely: durability is left
            // to the OS page cache. Callers can still force it via `sync()`.
            crate::config::SyncMode::Never => {}
            crate::config::SyncMode::Periodic => {
                let threshold = self
                    .flush_threshold
                    .map(|t| t as u64)
                    .unwrap_or(Self::DEFAULT_PERIODIC_THRESHOLD);
                if self.records_since_sync >= threshold {
                    self.sync()?;
                }
            }
        }
        Ok(())
    }

    /// Flush buffer and fsync to disk
    pub fn sync(&mut self) -> Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;
        self.records_since_sync = 0;
        Ok(())
    }

    /// The encryption context of this writer — `None` on a plaintext WAL.
    #[cfg(feature = "encryption")]
    pub(crate) fn crypto(&self) -> Option<&WalCrypto> {
        self.crypto.as_ref()
    }

    /// Return the total bytes written to the WAL (including headers).
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
    /// Return the number of records appended so far.
    pub fn record_count(&self) -> u64 {
        self.record_count
    }
    /// Return the path of the WAL file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Rotate the WAL: flush, close, archive as `vanta.wal.<timestamp>`,
    /// then create a fresh empty WAL at the original path.
    ///
    /// Returns a new `WalWriter` with `record_count = 0` and `bytes_written = 0`.
    pub fn rotate(mut self, sync_mode: crate::config::SyncMode) -> Result<Self> {
        self.sync()?;
        let old_path = self.path.clone();
        // The encryption context survives the rotation: the fresh segment is
        // encrypted too (the format is decided by the context, not by luck).
        let crypto = self.crypto.take();
        drop(self);

        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let archive_name = format!(
            "{}.{}",
            old_path.file_name().unwrap_or_default().to_string_lossy(),
            now
        );
        let archive_path = old_path.with_file_name(&archive_name);

        if archive_path.exists() {
            std::fs::remove_file(&archive_path)?;
        }
        std::fs::rename(&old_path, &archive_path)?;
        crate::utils::fs::sync_parent_dir(&archive_path)?;

        Self::open_impl(&old_path, sync_mode, 64 * KIB, None, crypto)
    }

    /// If bytes_written exceeds max_segment_size, flush + archive current
    /// segment and start a fresh WAL at the same path.
    ///
    /// Returns `Ok(true)` if rotation occurred, `Ok(false)` otherwise.
    fn try_auto_rotate(&mut self) -> Result<bool> {
        if self.bytes_written < self.max_segment_size {
            return Ok(false);
        }

        // Flush and sync current content
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;

        let old_path = self.path.clone();
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let archive_name = format!(
            "{}.{}",
            old_path.file_name().unwrap_or_default().to_string_lossy(),
            now
        );
        let archive_path = old_path.with_file_name(&archive_name);

        // Remove existing archive if present, then rename current WAL
        if archive_path.exists() {
            std::fs::remove_file(&archive_path)?;
        }
        std::fs::rename(&old_path, &archive_path)?;
        crate::utils::fs::sync_parent_dir(&archive_path)?;

        // Open fresh WAL file at original path and write a new header
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&old_path)?;

        let header = WalHeader::new(self.segment_version() as u32);
        file.write_all(&header.serialize())?;
        // Fresh segment at the original path is a new directory entry — make it
        // durable before any of its appends are (see open_with_buffer).
        crate::utils::fs::sync_parent_dir(&old_path)?;

        // Reset writer and counters with the same buffer capacity
        let capacity = self.writer.capacity();
        self.writer = BufWriter::with_capacity(capacity, file);
        self.bytes_written = WalHeader::SIZE as u64;
        self.record_count = 0;
        self.records_since_sync = 0;
        // Fresh segment → fresh chain (VER-01): chained framing from genesis.
        self.format_version = self.segment_version();
        self.chain_head = CHAIN_GENESIS;

        Ok(true)
    }
}

// ─── FIND-343: physical erasure of the deleted payloads ──────────────────

/// The node a WAL record writes, if any.
///
/// Only the node-scoped records carry a payload: `Insert` and `Update` hold a
/// whole [`UnifiedNode`] (the memory payload travels on it), `Delete` is a
/// bare id. The transaction records carry no node.
pub(crate) fn record_node_id(record: &WalRecord) -> Option<u128> {
    match record {
        WalRecord::Insert(node) => Some(node.id),
        WalRecord::Update { id, .. } => Some(*id),
        WalRecord::Delete { id } => Some(*id),
        _ => None,
    }
}

/// The outcome of one redaction pass over a WAL segment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SegmentRedact {
    /// Records inspected.
    pub scanned: u64,
    /// Records replaced by a payload-free tombstone.
    pub redacted: u64,
    /// Segment size before the pass.
    pub bytes_before: u64,
    /// Segment size after the pass.
    pub bytes_after: u64,
    /// Why the segment was left untouched (empty when it was rewritten).
    pub skipped: Option<&'static str>,
}

/// The aggregate outcome of a redaction pass over every WAL segment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct WalRedactReport {
    /// Segments inspected.
    pub segments: u64,
    /// Records inspected.
    pub scanned: u64,
    /// Records replaced by a payload-free tombstone.
    pub redacted: u64,
    /// Bytes before the pass.
    pub bytes_before: u64,
    /// Bytes after the pass.
    pub bytes_after: u64,
    /// Why the pass was skipped (empty when it ran).
    pub skipped: Option<&'static str>,
}

/// The scratch name used while a segment is rewritten.
///
/// The name carries **no** `.wal.` infix: the WAL shipper discovers the
/// rotated segments with `name.contains(".wal.") && !name.ends_with(".wal")`
/// and must never pick up a half-written scratch file. A rotated segment is
/// named `<shard>.wal.<millis>`, hence the **infix** is cut, not just the
/// extension.
pub(crate) fn scratch_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "wal".to_string());
    let stem = match name.find(".wal") {
        Some(idx) => &name[..idx],
        None => &name[..],
    };
    let dir = path.parent().unwrap_or(Path::new("."));
    dir.join(format!("{stem}.redact-{}.tmp", std::process::id()))
}

/// Read the on-disk format version of a WAL segment (0 when unreadable).
fn segment_version(path: &Path) -> u16 {
    let Ok(mut file) = File::open(path) else {
        return 0;
    };
    let mut head = [0u8; WalHeader::SIZE];
    if file.read_exact(&mut head).is_err() {
        return 0;
    }
    WalHeader::deserialize(&head)
        .map(|h| h.base.format_version)
        .unwrap_or(0)
}

/// Write one record as a WAL frame, updating the VER-01 chain head.
///
/// The frame layout follows the segment's own version: the chained tail
/// (`prev_hash ‖ record_hash`) is appended from [`WAL_CHAIN_VERSION`] on, the
/// legacy layout is byte-compatible with it.
fn write_frame(
    out: &mut impl Write,
    record: &WalRecord,
    chained: bool,
    prev: &mut [u8; CHAIN_HASH_SIZE],
) -> Result<()> {
    let payload = postcard::to_allocvec(record).map_err(Error::serialization)?;
    let len = payload.len() as u32;
    let crc = crc32c(&payload);
    out.write_all(&len.to_le_bytes())?;
    out.write_all(&payload)?;
    out.write_all(&crc.to_le_bytes())?;
    if chained {
        let record_hash = chain_hash(prev, &len.to_le_bytes(), &payload, &crc.to_le_bytes());
        out.write_all(prev)?;
        out.write_all(&record_hash)?;
        *prev = record_hash;
    }
    Ok(())
}

/// Rewrite a **plaintext** WAL segment into `tmp`, replacing the records of
/// the erased nodes with payload-free tombstones.
///
/// * The record **count** is preserved (a tombstone replaces the record), so
///   the `checkpoint_seq` semantics — "the first N records are already
///   applied" — stay exact: the recovery skips the same number of records
///   before and after the pass.
/// * The replayed **state** is identical: a node whose last record is a
///   `Delete` ends up absent with or without the redaction.
/// * The VER-01 chain is recomputed over the new frames.
/// * The segment is left untouched when there is nothing to redact, when it is
///   **encrypted** (v4 — the payload is already ciphertext) or when its tail
///   is corrupt (the quarantine path owns that case).
///
/// The rewrite is **streaming** (one record in flight): a 256 MiB segment
/// never has to fit in RAM.
pub(crate) fn write_redacted_segment(
    src: &Path,
    tmp: &Path,
    erased: &dyn Fn(&WalRecord) -> bool,
) -> Result<SegmentRedact> {
    let bytes_before = std::fs::metadata(src).map(|m| m.len()).unwrap_or(0);
    let version = segment_version(src);
    if version == 0 {
        return Ok(SegmentRedact {
            bytes_before,
            skipped: Some("unreadable header"),
            ..Default::default()
        });
    }
    if is_encrypted(version) {
        return Ok(SegmentRedact {
            bytes_before,
            skipped: Some("encrypted segment (v4): the payload is already ciphertext"),
            ..Default::default()
        });
    }

    // Pass 1: the tail must be clean. A corrupt tail is the quarantine path's
    // business — never drop it here.
    let mut scanned = 0u64;
    let mut reader = WalReader::open(src)?;
    while reader.next_record()?.is_some() {
        scanned += 1;
    }
    let consumed = reader.consumed()?;
    drop(reader);
    let file_len = std::fs::metadata(src).map(|m| m.len()).unwrap_or(0);
    if consumed != file_len {
        return Ok(SegmentRedact {
            scanned,
            redacted: 0,
            bytes_before,
            bytes_after: bytes_before,
            skipped: Some("corrupt tail: left to the quarantine path"),
        });
    }

    // Pass 2: stream the survivors into the scratch file.
    let mut reader = WalReader::open(src)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(tmp)?;
    let mut redacted = 0u64;
    let mut prev = CHAIN_GENESIS;
    {
        let mut out = BufWriter::with_capacity(256 * KIB, &mut file);
        out.write_all(&WalHeader::new(version as u32).serialize())?;
        while let Some(record) = reader.next_record()? {
            let record = if erased(&record) && !matches!(record, WalRecord::Delete { .. }) {
                match record_node_id(&record) {
                    Some(id) => {
                        redacted += 1;
                        WalRecord::Delete { id }
                    }
                    None => record,
                }
            } else {
                record
            };
            write_frame(&mut out, &record, is_chained(version), &mut prev)?;
        }
        let flushed = out.flush();
        if let Err(err) = flushed {
            let _ = std::fs::remove_file(tmp);
            return Err(Error::Io(err));
        }
    }
    if let Err(err) = file.sync_all() {
        let _ = std::fs::remove_file(tmp);
        return Err(Error::Io(err));
    }
    let bytes_after = file.metadata().map(|m| m.len()).unwrap_or(bytes_before);
    drop(file);

    if redacted == 0 {
        let _ = std::fs::remove_file(tmp);
        return Ok(SegmentRedact {
            scanned,
            redacted: 0,
            bytes_before,
            bytes_after: bytes_before,
            skipped: Some("nothing to redact"),
        });
    }

    Ok(SegmentRedact {
        scanned,
        redacted,
        bytes_before,
        bytes_after,
        skipped: None,
    })
}

/// Swap `tmp` over `path` atomically: the rename is the commit point and the
/// parent directory is fsync'd so the swap survives a power loss.
pub(crate) fn swap_segment(tmp: &Path, path: &Path) -> Result<()> {
    std::fs::rename(tmp, path).map_err(Error::Io)?;
    crate::utils::fs::sync_parent_dir(path).map_err(Error::Io)?;
    Ok(())
}

impl WalWriter {
    /// Physically erase the payloads of the **deleted** nodes from this
    /// segment (FIND-343).
    ///
    /// The erased records are replaced by payload-free tombstones, so the
    /// record count — and therefore the `checkpoint_seq` semantics — are
    /// untouched, and the replayed state is identical.
    ///
    /// The pass is **opportunistic**: an encrypted (v4) segment, a corrupt
    /// tail or a segment with nothing to redact are left alone.
    pub(crate) fn redact_erased(
        &mut self,
        erased: &dyn Fn(&WalRecord) -> bool,
    ) -> Result<SegmentRedact> {
        if is_encrypted(self.format_version) {
            return Ok(SegmentRedact {
                bytes_before: self.bytes_written,
                skipped: Some("encrypted segment (v4): the payload is already ciphertext"),
                ..Default::default()
            });
        }
        // The pass rewrites the whole segment: the pending appends must be on
        // disk before the records are read.
        self.sync()?;
        let path = self.path.clone();
        let capacity = self.writer.capacity();
        let tmp = scratch_path(&path);
        let outcome = write_redacted_segment(&path, &tmp, erased)?;
        if outcome.redacted == 0 {
            let _ = std::fs::remove_file(&tmp);
            return Ok(outcome);
        }
        // The rename needs the destination closed (Windows refuses to replace
        // an open file): detach the handle, swap, then reopen. The
        // placeholder is a scratch file that is removed right away — it only
        // exists so the writer owns *some* handle while the swap happens.
        let placeholder = scratch_path(&path).with_extension("placeholder");
        let detached = std::mem::replace(
            &mut self.writer,
            BufWriter::with_capacity(
                1,
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&placeholder)?,
            ),
        );
        drop(detached);
        let _ = std::fs::remove_file(&placeholder);
        if let Err(err) = swap_segment(&tmp, &path) {
            let _ = std::fs::remove_file(&tmp);
            return Err(err);
        }
        // Reopen the segment: the reader re-derives the record count, the
        // byte count and the chain head from the new file.
        let reopened =
            Self::open_impl(&path, self.sync_mode, capacity, self.flush_threshold, None)?;
        *self = reopened;
        Ok(outcome)
    }
}

// ─── Scan-Forward Helpers ─────────────────────────────────

/// Check whether a valid WAL record exists at `pos` in the given reader.
/// Validates bounds, CRC32C, and postcard deserialization. Returns `false` on any I/O error.
///
/// `extra` is the per-frame chain overhead for the file's format version
/// ([`frame_extra_for`]) — chain bytes are not validated here (recovery keeps
/// its CRC+deser semantics; `verify_wal_file` is the chain authority).
fn check_record_at<R: Read + Seek>(reader: &mut R, pos: u64, file_len: u64, extra: usize) -> bool {
    if pos + 8 + extra as u64 > file_len {
        return false;
    }
    if reader.seek(SeekFrom::Start(pos)).is_err() {
        return false;
    }
    let mut len_buf = [0u8; 4];
    if reader.read_exact(&mut len_buf).is_err() {
        return false;
    }
    let len = u32::from_le_bytes(len_buf) as u64;
    if len == 0 || len > 10_000_000 || pos + 4 + len + 4 + extra as u64 > file_len {
        return false;
    }
    let mut record_bytes = vec![0u8; len as usize + 4];
    if reader.read_exact(&mut record_bytes).is_err() {
        return false;
    }
    let payload = &record_bytes[0..len as usize];
    let crc_bytes: [u8; 4] = match record_bytes[len as usize..len as usize + 4].try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let stored_crc = u32::from_le_bytes(crc_bytes);
    let computed_crc = crc32c(payload);
    stored_crc == computed_crc && postcard::from_bytes::<WalRecord>(payload).is_ok()
}

/// Scan forward byte-by-byte from `start_pos` to find the next valid WAL record.
fn scan_forward_valid<R: Read + Seek>(
    reader: &mut R,
    file_len: u64,
    start_pos: u64,
    extra: usize,
) -> Option<u64> {
    let mut scan_pos = start_pos + 1;
    while scan_pos + 8 + extra as u64 <= file_len {
        if check_record_at(reader, scan_pos, file_len, extra) {
            return Some(scan_pos);
        }
        scan_pos += 1;
    }
    None
}

/// Try to scan forward past corruption. Logs a warning and returns the found position,
/// or `None` if no valid record remains in the file.
fn try_scan_forward<R: Read + Seek>(
    reader: &mut R,
    file_len: u64,
    current_pos: u64,
    extra: usize,
) -> Option<u64> {
    let found = scan_forward_valid(reader, file_len, current_pos, extra)?;
    warn!(
        corrupt_bytes_skipped = found - current_pos,
        recovered_offset = found,
        "Scan-forward bypassed corrupt bytes and recovered next transaction."
    );
    Some(found)
}

/// Scan an existing WAL file to find the end of valid records and count them.
/// Handles mid-file corruption via Scan-Forward recovery. Returns `(valid_bytes_end, record_count)`.
///
/// `extra` is the per-frame chain overhead for the file's format version.
fn recover_valid_records(path: &Path, file_len: u64, extra: usize) -> Result<(u64, usize)> {
    let mut file_handle = File::open(path)?;
    let mut valid_bytes_limit = WalHeader::SIZE as u64;
    let mut record_count = 0usize;
    let mut current_offset = WalHeader::SIZE as u64;

    loop {
        if current_offset >= file_len {
            break;
        }
        if file_handle.seek(SeekFrom::Start(current_offset)).is_err() {
            break;
        }
        let mut len_buf = [0u8; 4];
        if file_handle.read_exact(&mut len_buf).is_err() {
            break;
        }
        let len = u32::from_le_bytes(len_buf) as u64;

        let is_valid = check_record_at(&mut file_handle, current_offset, file_len, extra);

        if is_valid {
            record_count += 1;
            current_offset += 4 + len + 4 + extra as u64;
            valid_bytes_limit = current_offset;
        } else {
            warn!(
                path = %path.display(),
                offset = current_offset,
                "Corrupt record detected in WAL. Entering Scan-Forward mode to locate next valid transaction..."
            );

            if let Some(found) = try_scan_forward(&mut file_handle, file_len, current_offset, extra)
            {
                current_offset = found;
            } else {
                break;
            }
        }
    }

    Ok((valid_bytes_limit, record_count))
}

// ─── Encrypted frame (v4, ADR-0057 W2) ────────────────────

/// The outcome of inspecting one encrypted frame.
#[cfg(feature = "encryption")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncryptedFrameStatus {
    /// The frame is intact and its AEAD tag authenticated.
    Valid,
    /// The frame is damaged (CRC / length / bounds) — a truncatable tail.
    Damaged,
    /// The AEAD tag did not authenticate: wrong key **or** tampering. A hard
    /// error: the WAL is refused, never truncated.
    Rejected,
    /// The file ended before the frame was complete (a crash tail).
    Incomplete,
}

/// Fixed bytes of a v4 frame before the ciphertext blob:
/// `ct_len (4) ‖ crc (4) ‖ nonce (12)`.
#[cfg(feature = "encryption")]
const V4_FRAME_PREFIX: u64 = 4 + 4 + 12;

/// The AEAD tag is part of the stored blob, so the blob is never shorter.
#[cfg(feature = "encryption")]
const V4_MIN_BLOB: u64 = 16;

/// Read and validate one encrypted frame (v4) at `pos`.
///
/// Returns the frame status, its **total on-disk length** (0 when the length
/// could not be determined) and the decrypted plaintext (empty unless the
/// frame is [`EncryptedFrameStatus::Valid`]).
///
/// The three integrity layers of ADR-0057 D5 are kept apart:
///
/// * the **CRC32C over the ciphertext** separates *corrupt* (bit rot) from
///   *wrong key* — a CRC failure is a truncatable tail;
/// * the **AEAD tag** authenticates the plaintext — a tag failure is a hard
///   error (wrong key or tampering), never a truncatable tail;
/// * the **hash chain** is over the ciphertext and is validated by
///   `verify_wal_file`, not by the replay path (VER-01 design).
#[cfg(feature = "encryption")]
fn read_encrypted_frame<R: Read + Seek>(
    reader: &mut R,
    pos: u64,
    file_len: u64,
    key: &WalKey,
    slot: u64,
) -> Result<(EncryptedFrameStatus, u64, Vec<u8>)> {
    if reader.seek(SeekFrom::Start(pos)).is_err() {
        return Ok((EncryptedFrameStatus::Damaged, 0, Vec::new()));
    }
    let mut prefix = [0u8; V4_FRAME_PREFIX as usize];
    if reader.read_exact(&mut prefix).is_err() {
        return Ok((EncryptedFrameStatus::Incomplete, 0, Vec::new()));
    }
    let ct_len = u32::from_le_bytes([prefix[0], prefix[1], prefix[2], prefix[3]]) as u64;
    let crc = u32::from_le_bytes([prefix[4], prefix[5], prefix[6], prefix[7]]);
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes.copy_from_slice(&prefix[8..20]);

    if !(V4_MIN_BLOB..=10_000_000).contains(&ct_len) {
        return Ok((EncryptedFrameStatus::Damaged, 0, Vec::new()));
    }
    let frame_len = V4_FRAME_PREFIX + ct_len + CHAIN_FRAME_EXTRA as u64;
    if pos + frame_len > file_len {
        return Ok((EncryptedFrameStatus::Incomplete, frame_len, Vec::new()));
    }

    let mut ct = vec![0u8; ct_len as usize];
    if reader.read_exact(&mut ct).is_err() {
        return Ok((EncryptedFrameStatus::Incomplete, frame_len, Vec::new()));
    }
    // The chain tail is read and skipped: the chain is validated by
    // `verify_wal_file`, never by the replay path (VER-01 design).
    let mut chain = [0u8; CHAIN_FRAME_EXTRA];
    if reader.read_exact(&mut chain).is_err() {
        return Ok((EncryptedFrameStatus::Incomplete, frame_len, Vec::new()));
    }

    // The CRC covers the ciphertext (ADR-0057 D5).
    if compute_crc32c(&ct) != crc {
        return Ok((EncryptedFrameStatus::Damaged, frame_len, Vec::new()));
    }

    let nonce = Nonce::from_bytes(nonce_bytes);
    let aad = AadBuilder::new(Artifact::Wal, WAL_ENCRYPTED_FORMAT_VERSION)
        .epoch(nonce.epoch())
        .slot(slot)
        .plaintext_len((ct_len - V4_MIN_BLOB) as usize)
        .build();
    let plaintext = match key.cipher.decrypt_with_nonce(nonce.as_bytes(), &ct, &aad) {
        Ok(plaintext) => plaintext,
        Err(CryptoError::DecryptionFailed) => {
            return Ok((EncryptedFrameStatus::Rejected, frame_len, Vec::new()));
        }
        Err(err) => return Err(crypto_error(err)),
    };

    if postcard::from_bytes::<WalRecord>(&plaintext).is_err() {
        return Ok((EncryptedFrameStatus::Damaged, frame_len, Vec::new()));
    }
    Ok((EncryptedFrameStatus::Valid, frame_len, plaintext))
}

/// Scan an encrypted WAL (v4) to find the end of its valid frames.
///
/// Mirrors [`recover_valid_records`] with the encrypted frame layout:
///
/// * a frame whose **CRC fails** is a corrupt tail → truncatable (the caller
///   quarantines and truncates it), exactly like the plaintext path;
/// * a frame whose **CRC passes but whose AEAD tag fails** is a **hard
///   error** (wrong key or tampering): the WAL is refused, never truncated —
///   truncating would silently drop the records that follow;
/// * a valid frame is counted and the walk continues.
///
/// Unlike the plaintext path there is **no scan-forward**: the AAD binds each
/// frame to its ordinal, so a frame that follows a gap could not be
/// authenticated anyway.
#[cfg(feature = "encryption")]
fn recover_valid_records_v4(path: &Path, file_len: u64, key: &WalKey) -> Result<(u64, usize)> {
    let mut file = File::open(path)?;
    let mut offset = WalHeader::SIZE as u64;
    let mut count = 0usize;

    while offset < file_len {
        let (status, frame_len, _) =
            read_encrypted_frame(&mut file, offset, file_len, key, count as u64)?;
        match status {
            EncryptedFrameStatus::Valid => {
                count += 1;
                offset += frame_len;
            }
            EncryptedFrameStatus::Damaged | EncryptedFrameStatus::Incomplete => break,
            EncryptedFrameStatus::Rejected => {
                return Err(Error::EncryptionKeyRejected {
                    artifact: Artifact::Wal.as_str().to_string(),
                });
            }
        }
    }

    Ok((offset, count))
}

/// Copy the corrupt trailing bytes `[valid_end, file_len)` of the WAL at
/// `path` to a quarantine backup file before they are truncated away, so the
/// bytes stay recoverable. Fails soft: recovery must never depend on backup
/// succeeding, so a failure is logged and ignored.
fn quarantine_corrupt_tail(path: &Path, valid_end: u64, file_len: u64) {
    if valid_end >= file_len {
        return;
    }
    let backup = quarantine_backup_path(path);
    let result = (|| -> std::io::Result<()> {
        let mut src = File::open(path)?;
        src.seek(SeekFrom::Start(valid_end))?;
        let mut tail = vec![0u8; (file_len - valid_end) as usize];
        src.read_exact(&mut tail)?;
        std::fs::write(&backup, tail)
    })();
    match result {
        Ok(()) => warn!(
            backup = %backup.display(),
            bytes = file_len - valid_end,
            "Quarantined corrupt WAL tail before truncation"
        ),
        Err(e) => warn!(
            error = %e,
            backup = %backup.display(),
            "Failed to quarantine corrupt WAL tail; truncating anyway"
        ),
    }
}

/// Prefer `<path>.corrupt`; if it already exists, fall back to
/// `<path>.corrupt.<N>` so earlier corruption evidence is never overwritten.
fn quarantine_backup_path(path: &Path) -> PathBuf {
    let plain = PathBuf::from(format!("{}.corrupt", path.display()));
    if plain.exists() {
        for n in 1..1000u32 {
            let candidate = PathBuf::from(format!("{}.corrupt.{}", path.display(), n));
            if !candidate.exists() {
                return candidate;
            }
        }
    }
    plain
}

// ─── WAL Chain Verification (VER-01) ───────────────────────
//
// Read-only, offline, strict walk (NO scan-forward healing — healing is a
// recovery behavior, not a verification one). Distinguishes chain evidence of
// manipulation (`Tampered`) from frame damage (`Corrupt`) and from a crash
// tail (`IncompleteTail`), and reports legacy (pre-chain) files explicitly.

/// Outcome of verifying one WAL file's integrity.
#[cfg(any(feature = "cli", test))]
#[derive(Debug, Clone)]
pub(crate) enum WalVerifyStatus {
    /// Every frame's CRC, chain link and record hash validated.
    Verified,
    /// Pre-chain format (v1/v2): CRC walk only — no chain to verify.
    Legacy,
    /// Chain evidence of manipulation (record altered/removed/inserted/reordered).
    Tampered {
        /// Byte offset of the first offending frame.
        offset: u64,
        /// 1-based index of the offending record.
        record: u64,
        /// Human-readable cause.
        reason: String,
    },
    /// Frame-level damage (CRC/deserialization/framing) — not a chain claim;
    /// may be bit-rot or a crash, not necessarily manipulation.
    Corrupt {
        /// Byte offset of the first offending frame.
        offset: u64,
        /// 1-based index of the offending record.
        record: u64,
        /// Human-readable cause.
        reason: String,
    },
    /// Trailing bytes form an incomplete frame (e.g. crash mid-append).
    /// Not treated as tamper: quarantine/recovery owns this case.
    IncompleteTail {
        /// Byte offset where the incomplete frame starts.
        offset: u64,
    },
}

#[cfg(any(feature = "cli", test))]
impl WalVerifyStatus {
    /// True when the file is not evidence of integrity failure (verified,
    /// legacy or crash-tail).
    pub(crate) fn is_ok(&self) -> bool {
        matches!(
            self,
            Self::Verified | Self::Legacy | Self::IncompleteTail { .. }
        )
    }

    /// Stable machine-readable label.
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Legacy => "legacy",
            Self::Tampered { .. } => "tampered",
            Self::Corrupt { .. } => "corrupt",
            Self::IncompleteTail { .. } => "incomplete_tail",
        }
    }
}

/// Per-file WAL verification report.
#[cfg(any(feature = "cli", test))]
#[derive(Debug, Clone)]
pub(crate) struct WalVerifyReport {
    /// Path of the verified file.
    pub path: PathBuf,
    /// On-disk format version from the header (0 when the header is invalid).
    pub format_version: u16,
    /// Number of records that validated completely.
    pub records: u64,
    /// Verification outcome.
    pub status: WalVerifyStatus,
}

/// Verify a WAL file's integrity offline (read-only, no engine, no mutation).
///
/// For chained files (v3+) this validates the full hash-chain: each frame's
/// `record_hash = SHA-256(prev_hash ‖ len ‖ payload ‖ crc)` and its link to the
/// previous `record_hash`. Removing/inserting/reordering a frame breaks the
/// successor's link; altering a frame (even with a recomputed CRC) breaks its
/// own hash. Legacy files (v1/v2) get a CRC walk reported as [`WalVerifyStatus::Legacy`]
/// — their missing chain is never called manipulation.
///
/// Documented limits: a clean-boundary suffix truncation, a fully consistent
/// whole-file rewrite, and whole-segment deletion are indistinguishable without
/// an external anchor (signature/manifest) — a v1.0 follow-up.
#[cfg(any(feature = "cli", test))]
pub(crate) fn verify_wal_file(path: &Path) -> Result<WalVerifyReport> {
    let mut file = File::open(path)?;
    let file_len = file.metadata()?.len();

    let report = |format_version: u16, records: u64, status: WalVerifyStatus| WalVerifyReport {
        path: path.to_path_buf(),
        format_version,
        records,
        status,
    };

    if file_len < WalHeader::SIZE as u64 {
        return Ok(report(
            0,
            0,
            WalVerifyStatus::Corrupt {
                offset: 0,
                record: 0,
                reason: "file too small for a WAL header".to_string(),
            },
        ));
    }

    let mut header_bytes = [0u8; WalHeader::SIZE];
    file.read_exact(&mut header_bytes)?;
    let header = match WalHeader::deserialize(&header_bytes) {
        Ok(h) => h,
        Err(e) => {
            return Ok(report(
                0,
                0,
                WalVerifyStatus::Corrupt {
                    offset: 0,
                    record: 0,
                    reason: format!("invalid header: {e}"),
                },
            ));
        }
    };
    let version = header.base.format_version;

    // An encrypted WAL is verified **without the key**: the CRC32C and the
    // hash chain both cover the ciphertext, so the operator can still check
    // the integrity of a WAL they cannot decrypt (ADR-0057 D5).
    #[cfg(feature = "encryption")]
    if is_encrypted(version) {
        return verify_wal_file_v4(path, &mut file, file_len);
    }

    let chained = is_chained(version);
    let extra = frame_extra_for(version);

    let mut prev = CHAIN_GENESIS;
    let mut record: u64 = 0;
    let mut offset: u64 = WalHeader::SIZE as u64;

    let status = loop {
        if offset == file_len {
            break if chained {
                WalVerifyStatus::Verified
            } else {
                WalVerifyStatus::Legacy
            };
        }
        // Smallest possible frame is [len][crc] + chain tail.
        if file_len - offset < 8 + extra as u64 {
            break WalVerifyStatus::IncompleteTail { offset };
        }

        let mut len_buf = [0u8; 4];
        if file.read_exact(&mut len_buf).is_err() {
            break WalVerifyStatus::IncompleteTail { offset };
        }
        let len = u32::from_le_bytes(len_buf) as u64;
        if len == 0 || len > 10_000_000 {
            break WalVerifyStatus::Corrupt {
                offset,
                record: record + 1,
                reason: format!("invalid record length prefix: {len}"),
            };
        }
        let frame_len = 4 + len + 4 + extra as u64;
        if offset + frame_len > file_len {
            break WalVerifyStatus::IncompleteTail { offset };
        }

        let mut payload = vec![0u8; len as usize];
        let mut crc_buf = [0u8; 4];
        if file.read_exact(&mut payload).is_err() || file.read_exact(&mut crc_buf).is_err() {
            break WalVerifyStatus::IncompleteTail { offset };
        }
        let stored_crc = u32::from_le_bytes(crc_buf);
        let computed_crc = crc32c(&payload);
        if stored_crc != computed_crc {
            break WalVerifyStatus::Corrupt {
                offset,
                record: record + 1,
                reason: format!(
                    "CRC32C mismatch (stored={stored_crc:#x}, computed={computed_crc:#x})"
                ),
            };
        }
        if postcard::from_bytes::<WalRecord>(&payload).is_err() {
            break WalVerifyStatus::Corrupt {
                offset,
                record: record + 1,
                reason: "payload does not deserialize as a WAL record".to_string(),
            };
        }

        if chained {
            let mut chain_buf = [0u8; CHAIN_FRAME_EXTRA];
            if file.read_exact(&mut chain_buf).is_err() {
                break WalVerifyStatus::IncompleteTail { offset };
            }
            let mut stored_prev = [0u8; CHAIN_HASH_SIZE];
            stored_prev.copy_from_slice(&chain_buf[..CHAIN_HASH_SIZE]);
            let mut stored_hash = [0u8; CHAIN_HASH_SIZE];
            stored_hash.copy_from_slice(&chain_buf[CHAIN_HASH_SIZE..]);

            if stored_prev != prev {
                break WalVerifyStatus::Tampered {
                    offset,
                    record: record + 1,
                    reason: "chain link broken (record removed, inserted or reordered)".to_string(),
                };
            }
            let computed = chain_hash(&prev, &len_buf, &payload, &crc_buf);
            if stored_hash != computed {
                break WalVerifyStatus::Tampered {
                    offset,
                    record: record + 1,
                    reason: "record content altered (frame hash mismatch)".to_string(),
                };
            }
            prev = stored_hash;
        }

        record += 1;
        offset += frame_len;
    };

    Ok(report(version, record, status))
}

/// Verify an **encrypted** WAL (v4) offline — **without the key**.
///
/// The CRC32C and the `prev_hash ‖ record_hash` chain both cover the
/// **ciphertext**, so an operator without the key still gets the tamper
/// evidence (reordering, splicing, truncation, bit rot). The AEAD tag — the
/// layer that proves the *plaintext* — is checked by the replay path, which
/// does have the key.
#[cfg(all(any(feature = "cli", test), feature = "encryption"))]
fn verify_wal_file_v4(path: &Path, file: &mut File, file_len: u64) -> Result<WalVerifyReport> {
    let report = |records: u64, status: WalVerifyStatus| WalVerifyReport {
        path: path.to_path_buf(),
        format_version: WAL_ENCRYPTED_FORMAT_VERSION,
        records,
        status,
    };

    let mut prev = CHAIN_GENESIS;
    let mut record: u64 = 0;
    let mut offset: u64 = WalHeader::SIZE as u64;

    let status = loop {
        if offset == file_len {
            break WalVerifyStatus::Verified;
        }
        // Smallest possible frame: [ct_len][crc][nonce] + tag + chain tail.
        if file_len - offset < V4_FRAME_PREFIX + V4_MIN_BLOB + CHAIN_FRAME_EXTRA as u64 {
            break WalVerifyStatus::IncompleteTail { offset };
        }

        let mut prefix = [0u8; V4_FRAME_PREFIX as usize];
        if file.read_exact(&mut prefix).is_err() {
            break WalVerifyStatus::IncompleteTail { offset };
        }
        let ct_len = u32::from_le_bytes([prefix[0], prefix[1], prefix[2], prefix[3]]) as u64;
        let crc = u32::from_le_bytes([prefix[4], prefix[5], prefix[6], prefix[7]]);
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&prefix[8..20]);

        if !(V4_MIN_BLOB..=10_000_000).contains(&ct_len) {
            break WalVerifyStatus::Corrupt {
                offset,
                record: record + 1,
                reason: format!("invalid ciphertext length prefix: {ct_len}"),
            };
        }
        let frame_len = V4_FRAME_PREFIX + ct_len + CHAIN_FRAME_EXTRA as u64;
        if offset + frame_len > file_len {
            break WalVerifyStatus::IncompleteTail { offset };
        }

        let mut ct = vec![0u8; ct_len as usize];
        if file.read_exact(&mut ct).is_err() {
            break WalVerifyStatus::IncompleteTail { offset };
        }
        let mut chain_buf = [0u8; CHAIN_FRAME_EXTRA];
        if file.read_exact(&mut chain_buf).is_err() {
            break WalVerifyStatus::IncompleteTail { offset };
        }
        let mut stored_prev = [0u8; CHAIN_HASH_SIZE];
        stored_prev.copy_from_slice(&chain_buf[..CHAIN_HASH_SIZE]);
        let mut stored_hash = [0u8; CHAIN_HASH_SIZE];
        stored_hash.copy_from_slice(&chain_buf[CHAIN_HASH_SIZE..]);

        // The CRC covers the ciphertext: it detects bit rot without the key.
        let computed_crc = compute_crc32c(&ct);
        if computed_crc != crc {
            break WalVerifyStatus::Corrupt {
                offset,
                record: record + 1,
                reason: format!("CRC32C mismatch (stored={crc:#x}, computed={computed_crc:#x})"),
            };
        }
        // The chain is over the ciphertext: it detects reordering, splicing
        // and truncation — still without the key.
        if stored_prev != prev {
            break WalVerifyStatus::Tampered {
                offset,
                record: record + 1,
                reason: "chain link broken (record removed, inserted or reordered)".to_string(),
            };
        }
        let computed = chain_hash_v4(&prev, &prefix[0..4], &prefix[4..8], &nonce, &ct);
        if stored_hash != computed {
            break WalVerifyStatus::Tampered {
                offset,
                record: record + 1,
                reason: "record content altered (frame hash mismatch)".to_string(),
            };
        }

        prev = stored_hash;
        record += 1;
        offset += frame_len;
    };

    Ok(report(record, status))
}

// ─── WAL Reader ────────────────────────────────────────────

/// Sequential WAL reader for crash recovery.
pub struct WalReader {
    reader: BufReader<File>,
    records_read: u64,
    /// Per-frame chain overhead for this file's format version
    /// ([`frame_extra_for`]): 0 for legacy v1/v2, 64 for chained v3+.
    frame_extra: usize,
    /// True when the file uses the encrypted (v4) frame layout.
    encrypted: bool,
    /// The decryption key — `Some` only for an encrypted (v4) WAL.
    #[cfg(feature = "encryption")]
    key: Option<WalKey>,
    /// The ordinal of the next frame — the AAD slot (v4 only).
    #[cfg(feature = "encryption")]
    frame_slot: u64,
}

impl WalReader {
    /// Open a WAL file and validate its header.
    ///
    /// An **encrypted** (v4) WAL needs its key: use [`WalReader::open_with_key`].
    /// Opening one without a key fails with
    /// [`Error::EncryptionKeyRequired`] — never with a corruption error.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_impl(path, None)
    }

    /// Open an **encrypted** WAL file (v4) with its decryption key.
    #[cfg(feature = "encryption")]
    pub fn open_with_key(path: impl AsRef<Path>, key: WalKey) -> Result<Self> {
        Self::open_impl(path, Some(key))
    }

    #[cfg_attr(not(feature = "encryption"), allow(unused_variables))]
    fn open_impl(path: impl AsRef<Path>, key: Option<WalKeySlot>) -> Result<Self> {
        let mut file = File::open(path)?;
        let file_len = file.metadata()?.len();

        if file_len < WalHeader::SIZE as u64 {
            return Err(Error::wal_error(
                "WAL file is truncated or too small for header",
            ));
        }

        // Read and validate the header
        let mut header_bytes = [0u8; WalHeader::SIZE];
        file.read_exact(&mut header_bytes)?;
        let header = WalHeader::deserialize(&header_bytes)?;
        let encrypted = is_encrypted(header.base.format_version);

        // An encrypted WAL needs the key: refuse explicitly, never "corrupt".
        #[cfg(feature = "encryption")]
        if encrypted && key.is_none() {
            return Err(Error::EncryptionKeyRequired {
                artifact: Artifact::Wal.as_str().to_string(),
            });
        }

        Ok(Self {
            reader: BufReader::with_capacity(64 * KIB, file),
            records_read: 0,
            frame_extra: frame_extra_for(header.base.format_version),
            encrypted,
            #[cfg(feature = "encryption")]
            key,
            #[cfg(feature = "encryption")]
            frame_slot: 0,
        })
    }

    /// How many bytes of the file the reader has consumed.
    ///
    /// A reader that stops **before** the end of the file hit a corrupt or
    /// incomplete tail: the redaction pass uses this to leave such a segment
    /// alone (the quarantine path owns it).
    pub(crate) fn consumed(&mut self) -> Result<u64> {
        Ok(self.reader.stream_position()?)
    }

    /// Read next record with Scan-Forward Auto-healing. Returns None at EOF.
    pub fn next_record(&mut self) -> Result<Option<WalRecord>> {
        if self.encrypted {
            return self.next_record_encrypted();
        }
        let file_len = self.reader.get_ref().metadata()?.len();

        loop {
            let current_pos = self.reader.stream_position()?;
            if current_pos >= file_len {
                return Ok(None);
            }

            // Attempt to read the length prefix
            let mut len_buf = [0u8; 4];
            if let Err(e) = self.reader.read_exact(&mut len_buf) {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return Ok(None);
                }
                return Err(e.into());
            }
            let len = u32::from_le_bytes(len_buf) as u64;

            let mut is_valid = false;
            let mut payload = Vec::new();
            if len > 0
                && len <= 10_000_000
                && current_pos + 4 + len + 4 + self.frame_extra as u64 <= file_len
            {
                payload = vec![0u8; len as usize];
                if self.reader.read_exact(&mut payload).is_ok() {
                    let mut crc_buf = [0u8; 4];
                    if self.reader.read_exact(&mut crc_buf).is_ok() {
                        let stored_crc = u32::from_le_bytes(crc_buf);
                        let computed_crc = crc32c(&payload);
                        let is_crc_valid = stored_crc == computed_crc;
                        let deserialize_res = postcard::from_bytes::<WalRecord>(&payload);
                        let is_deser_ok = deserialize_res.is_ok();

                        if is_crc_valid && is_deser_ok {
                            // Consume the chained frame tail (prev_hash ‖
                            // record_hash). The chain itself is validated by
                            // `verify_wal_file`, never here: replay keeps its
                            // CRC + deser fail-soft semantics (VER-01 design).
                            if self.frame_extra > 0 {
                                let mut chain_buf = [0u8; CHAIN_FRAME_EXTRA];
                                is_valid = self.reader.read_exact(&mut chain_buf).is_ok();
                            } else {
                                is_valid = true;
                            }
                        } else {
                            let prefix_len = std::cmp::min(16, payload.len());
                            tracing::warn!(
                                "WAL record at current_pos={} is invalid. len={}, is_crc_valid={} (stored={:#x}, computed={:#x}), is_deser_ok={}, deser_err={:?}, payload_prefix={:?}",
                                current_pos, len, is_crc_valid, stored_crc, computed_crc, is_deser_ok, deserialize_res.err(), &payload[0..prefix_len]);
                        }
                    } else {
                        tracing::warn!("WAL: Failed to read CRC buf at pos {}", current_pos);
                    }
                } else {
                    tracing::warn!(
                        "WAL: Failed to read payload of len {} at pos {}",
                        len,
                        current_pos
                    );
                }
            } else {
                tracing::warn!(
                    "WAL: Bounds check failed for record at pos {}: len={}, file_len={}",
                    current_pos,
                    len,
                    file_len
                );
            }

            if is_valid {
                let record: WalRecord =
                    postcard::from_bytes(&payload).map_err(Error::serialization)?;
                self.records_read += 1;
                return Ok(Some(record));
            } else {
                // Entering Scan-Forward mode to skip corruption and find the next consistent block
                warn!(
                    offset = current_pos,
                    "WalReader detected corrupt record. Scanning forward to recover next valid transaction..."
                );

                if let Some(found) =
                    try_scan_forward(&mut self.reader, file_len, current_pos, self.frame_extra)
                {
                    self.reader.seek(SeekFrom::Start(found))?;
                } else {
                    return Ok(None);
                }
            }
        }
    }

    /// Read the next record of an **encrypted** WAL (v4).
    ///
    /// The frame is decrypted with the AAD rebuilt from the frame's own
    /// nonce (epoch) and its ordinal (slot), so a frame that was moved to
    /// another position fails to authenticate.
    ///
    /// There is **no scan-forward** on this path: the AAD binds every frame
    /// to its ordinal, so a frame that follows a gap could not be
    /// authenticated. A damaged frame ends the replay — the recovery path
    /// owns the corrupt tail (quarantine + truncate).
    #[cfg(feature = "encryption")]
    fn next_record_encrypted(&mut self) -> Result<Option<WalRecord>> {
        let file_len = self.reader.get_ref().metadata()?.len();
        let key = self
            .key
            .as_ref()
            .ok_or_else(|| Error::EncryptionKeyRequired {
                artifact: Artifact::Wal.as_str().to_string(),
            })?;

        let current_pos = self.reader.stream_position()?;
        if current_pos >= file_len {
            return Ok(None);
        }
        let (status, _frame_len, plaintext) = read_encrypted_frame(
            &mut self.reader,
            current_pos,
            file_len,
            key,
            self.frame_slot,
        )?;
        match status {
            EncryptedFrameStatus::Valid => {
                let record =
                    postcard::from_bytes::<WalRecord>(&plaintext).map_err(Error::serialization)?;
                self.frame_slot += 1;
                self.records_read += 1;
                Ok(Some(record))
            }
            EncryptedFrameStatus::Rejected => Err(Error::EncryptionKeyRejected {
                artifact: Artifact::Wal.as_str().to_string(),
            }),
            EncryptedFrameStatus::Damaged | EncryptedFrameStatus::Incomplete => Ok(None),
        }
    }

    /// Read the next record of an encrypted WAL — unreachable without the
    /// `encryption` feature (an encrypted WAL is refused at the header), kept
    /// so the dispatch stays total.
    #[cfg(not(feature = "encryption"))]
    fn next_record_encrypted(&mut self) -> Result<Option<WalRecord>> {
        Err(Error::wal_error(
            "WAL file is encrypted (format v4) but this build has no 'encryption' feature",
        ))
    }

    /// Replay all records through a handler function
    pub fn replay_all<F>(&mut self, mut handler: F) -> Result<u64>
    where
        F: FnMut(WalRecord) -> Result<()>,
    {
        let mut count = 0u64;
        while let Some(record) = self.next_record()? {
            handler(record)?;
            count += 1;
        }
        Ok(count)
    }
    /// Byte offset of the next unread byte (end of the last record returned
    /// by `next_record`). FIND-109: lets salvage truncate exactly after the
    /// K-th *validated* record, even with scan-forward gaps in the file.
    pub fn pos(&mut self) -> Result<u64> {
        use std::io::Seek;
        Ok(self.reader.stream_position()?)
    }
}

// ─── Checkpoint Helpers ───────────────────────────────────

impl WalRecord {
    /// Create a checkpoint record with optional index state for checksum computation
    pub fn create_checkpoint(node_count: u64, index_state: Option<&[u8]>) -> Self {
        let index_checksum = index_state.map(compute_crc32c);
        let timestamp = Some(
            web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );

        WalRecord::Checkpoint {
            node_count,
            index_checksum,
            timestamp,
        }
    }

    /// Validate checkpoint integrity if checksum is present
    pub fn validate_checkpoint(&self, index_state: &[u8]) -> Result<()> {
        if let WalRecord::Checkpoint {
            index_checksum: Some(expected),
            ..
        } = self
        {
            let computed = compute_crc32c(index_state);
            if computed != *expected {
                return Err(Error::wal_error(format!(
                    "Checkpoint index checksum mismatch: expected={:#x}, computed={:#x}",
                    expected, computed
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::node::UnifiedNode;

    #[test]
    fn test_wal_roundtrip() {
        let dir = std::env::temp_dir().join(format!("vanta_test_wal_rt_{}", rand::random::<u32>()));
        let _ = std::fs::remove_file(&dir);

        {
            let mut w = WalWriter::open(&dir, crate::config::SyncMode::Periodic).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(2))).unwrap();
            w.append(&WalRecord::Delete { id: 1 }).unwrap();
            w.append(&WalRecord::create_checkpoint(2, None)).unwrap();
            w.sync().unwrap();
            assert_eq!(w.record_count(), 4);
        }

        {
            let mut r = WalReader::open(&dir).unwrap();
            let mut records = Vec::new();
            r.replay_all(|rec| {
                records.push(rec);
                Ok(())
            })
            .unwrap();
            assert_eq!(records.len(), 4);
            // Verify checkpoint was read correctly
            if let WalRecord::Checkpoint { node_count, .. } = &records[3] {
                assert_eq!(*node_count, 2);
            } else {
                panic!("Expected Checkpoint at index 3");
            }
        }

        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn test_compute_crc32c_deterministic() {
        let data = b"vanta wal test";
        assert_eq!(compute_crc32c(data), compute_crc32c(data));
        assert_ne!(compute_crc32c(data), compute_crc32c(b"vanta wal tesx"));
    }

    /// FIND-63: `SyncMode::Never` must never auto-sync (OS page cache only).
    /// `records_since_sync` is only reset by an explicit `sync()` call.
    #[test]
    fn test_sync_mode_never_skips_auto_sync() {
        use crate::config::SyncMode;

        let dir_never =
            std::env::temp_dir().join(format!("vanta_test_wal_never_{}", rand::random::<u32>()));
        let dir_periodic = std::env::temp_dir().join(format!(
            "vanta_test_wal_never_ctrl_{}",
            rand::random::<u32>()
        ));
        let _ = std::fs::remove_file(&dir_never);
        let _ = std::fs::remove_file(&dir_periodic);

        // Control: Periodic with threshold 1 auto-syncs (counter reset to 0).
        {
            let mut w =
                WalWriter::open_with_buffer(&dir_periodic, SyncMode::Periodic, 4096, Some(1))
                    .unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            assert_eq!(
                w.records_since_sync, 0,
                "Periodic must auto-sync at threshold"
            );
        }

        // Never: no auto-sync even at threshold 1; explicit sync() still works.
        {
            let mut w =
                WalWriter::open_with_buffer(&dir_never, SyncMode::Never, 4096, Some(1)).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(2))).unwrap();
            assert_eq!(
                w.records_since_sync, 2,
                "Never must never auto-sync (relies on OS page cache)"
            );
            w.sync().unwrap();
            assert_eq!(
                w.records_since_sync, 0,
                "explicit sync() still resets the counter"
            );
        }

        let _ = std::fs::remove_file(&dir_never);
        let _ = std::fs::remove_file(&dir_periodic);
    }

    /// WAL v2 (RES-01 / ACID Phase 4a): `Prepare { txn_id, op_count }` round-trips
    /// through `WalWriter` → reopen → `WalReader::replay_all`.
    #[test]
    fn test_wal_v2_prepare_roundtrip_unit() {
        let dir =
            std::env::temp_dir().join(format!("vanta_test_wal_prepare_{}", rand::random::<u32>()));
        let _ = std::fs::remove_file(&dir);

        {
            let mut w = WalWriter::open(&dir, crate::config::SyncMode::Periodic).unwrap();
            w.append(&WalRecord::Begin(7)).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(10))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(11))).unwrap();
            w.append(&WalRecord::Prepare {
                txn_id: 7,
                op_count: 2,
            })
            .unwrap();
            w.sync().unwrap();
            assert_eq!(w.record_count(), 4);
        }

        let mut r = WalReader::open(&dir).unwrap();
        let mut records = Vec::new();
        r.replay_all(|rec| {
            records.push(rec);
            Ok(())
        })
        .unwrap();
        assert_eq!(
            records.len(),
            4,
            "Begin + 2x Insert + Prepare survived close+reopen"
        );
        match &records[3] {
            WalRecord::Prepare { txn_id, op_count } => {
                assert_eq!(*txn_id, 7);
                assert_eq!(*op_count, 2);
            }
            other => panic!("expected Prepare, got {other:?}"),
        }

        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn test_batch_append_byte_format_matches_append() {
        let dir = std::env::temp_dir().join(format!(
            "vanta_test_wal_batch_fmt_{}",
            rand::random::<u32>()
        ));
        let _ = std::fs::remove_file(&dir);

        let records = vec![
            WalRecord::Insert(UnifiedNode::new(1)),
            WalRecord::Insert(UnifiedNode::with_vector(2, vec![1.5; 32])),
            WalRecord::Delete { id: 3 },
            WalRecord::create_checkpoint(3, None),
        ];

        // Reference: N sequential `append` calls (existing single-record path)
        {
            let mut w = WalWriter::open(&dir, crate::config::SyncMode::Periodic).unwrap();
            for rec in &records {
                w.append(rec).unwrap();
            }
            w.sync().unwrap();
        }
        let sequential = std::fs::read(&dir).unwrap();
        let _ = std::fs::remove_file(&dir);

        // batch_append must produce byte-identical record bytes (the header
        // carries a wall-clock timestamp, so only the record region is compared)
        {
            let mut w = WalWriter::open(&dir, crate::config::SyncMode::Periodic).unwrap();
            w.batch_append(&records).unwrap();
            w.sync().unwrap();
        }
        let batched = std::fs::read(&dir).unwrap();

        assert_eq!(
            &sequential[WalHeader::SIZE..],
            &batched[WalHeader::SIZE..],
            "batch_append must emit byte-identical WAL framing"
        );

        // And the batched file must replay cleanly
        let mut r = WalReader::open(&dir).unwrap();
        let mut replayed = Vec::new();
        r.replay_all(|rec| {
            replayed.push(rec);
            Ok(())
        })
        .unwrap();
        assert_eq!(replayed.len(), records.len());

        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn test_checkpoint_validation() {
        let index_state = b"serialized_index_bytes";
        let checkpoint = WalRecord::create_checkpoint(42, Some(index_state));

        // Valid checkpoint should pass
        assert!(checkpoint.validate_checkpoint(index_state).is_ok());

        // Corrupted state should fail
        let corrupted = b"corrupted_index";
        assert!(checkpoint.validate_checkpoint(corrupted).is_err());

        // Checkpoint without checksum should always pass validation
        let checkpoint_no_crc = WalRecord::Checkpoint {
            node_count: 42,
            index_checksum: None,
            timestamp: None,
        };
        assert!(checkpoint_no_crc.validate_checkpoint(b"any_state").is_ok());
    }

    #[test]
    fn test_wal_version_mismatch() {
        let dir =
            std::env::temp_dir().join(format!("vanta_test_wal_mismatch_{}", rand::random::<u32>()));
        let _ = std::fs::remove_file(&dir);

        {
            // Write a WAL without a valid signature (version 0 or generic file)
            let mut file = File::create(&dir).unwrap();
            file.write_all(b"NOT_A_VALID_MAGIC_BYTES_123456").unwrap();
        }

        {
            // Opening the WAL must yield an IncompatibleFormat error
            let r = WalReader::open(&dir);
            assert!(r.is_err());
            match r.err().unwrap() {
                Error::IncompatibleFormat {
                    expected_magic,
                    expected_version,
                    ..
                } => {
                    assert_eq!(expected_magic, *b"VWAL");
                    // WAL v2 (RES-01): expected_version tracks the current
                    // format version. The test file in this fixture uses v0,
                    // so the reader should report its CURRENT version as the
                    // expected version when rejecting.
                    assert_eq!(expected_version, WAL_MAX_READABLE_VERSION);
                }
                other => panic!("Expected IncompatibleFormat, got {:?}", other),
            }
        }

        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn test_wal_auto_healing_and_recovery() {
        let dir =
            std::env::temp_dir().join(format!("vanta_test_wal_healing_{}", rand::random::<u32>()));
        let _ = std::fs::remove_file(&dir);

        // 1. Write 3 valid records + checkpoint
        {
            let mut w = WalWriter::open(&dir, crate::config::SyncMode::Periodic).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(2))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(3))).unwrap();
            w.append(&WalRecord::create_checkpoint(3, None)).unwrap();
            w.sync().unwrap();
            assert_eq!(w.record_count(), 4);
        }

        // 2. Corrupt the WAL by appending truncated garbage at the end
        {
            let mut file = OpenOptions::new().append(true).open(&dir).unwrap();
            file.write_all(
                b"\x0a\x00\x00\x00truncated garbage here that fails CRC or is cut off mid-way",
            )
            .unwrap();
        }

        // 3. Re-open the WAL with WalWriter
        {
            let mut w = WalWriter::open(&dir, crate::config::SyncMode::Periodic).unwrap();
            // Must have truncated garbage and loaded the correct record count (4)
            assert_eq!(w.record_count(), 4);

            // Try to write a new record
            w.append(&WalRecord::Insert(UnifiedNode::new(4))).unwrap();
            w.sync().unwrap();
            assert_eq!(w.record_count(), 5);
        }

        // 4. Read with WalReader and verify integrity
        {
            let mut r = WalReader::open(&dir).unwrap();
            let mut records = Vec::new();
            r.replay_all(|rec| {
                records.push(rec);
                Ok(())
            })
            .unwrap();
            assert_eq!(records.len(), 5);
            match &records[4] {
                WalRecord::Insert(node) => assert_eq!(node.id, 4),
                _ => panic!("Expected Insert node at index 4"),
            }
        }

        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn test_corrupt_wal_tail_is_quarantined() {
        let dir = std::env::temp_dir().join(format!(
            "vanta_test_wal_quarantine_{}",
            rand::random::<u32>()
        ));
        let backup = PathBuf::from(format!("{}.corrupt", dir.display()));
        let _ = std::fs::remove_file(&dir);
        let _ = std::fs::remove_file(&backup);

        // 1. Write a valid WAL
        {
            let mut w = WalWriter::open(&dir, crate::config::SyncMode::Periodic).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(2))).unwrap();
            w.sync().unwrap();
            assert_eq!(w.record_count(), 2);
        }

        // 2. Corrupt the tail: append a torn record so there are bytes past EOF
        {
            let mut file = OpenOptions::new().append(true).open(&dir).unwrap();
            file.write_all(b"\x00\xff\xff\xffcorrupt-tail-garbage")
                .unwrap();
        }

        // 3. Run recovery — must truncate AND quarantine the corrupt tail
        {
            let w = WalWriter::open(&dir, crate::config::SyncMode::Periodic).unwrap();
            assert_eq!(w.record_count(), 2);
        }

        // 4. Assert the corrupt tail was backed up, not silently lost
        assert!(backup.exists(), "quarantine backup {:?} must exist", backup);
        let backup_bytes = std::fs::read(&backup).unwrap();
        assert!(
            backup_bytes.len() >= b"\x00\xff\xff\xffcorrupt-tail-garbage".len(),
            "backup should contain the corrupt tail bytes"
        );

        // 5. Recovered WAL parses cleanly
        {
            let mut r = WalReader::open(&dir).unwrap();
            let mut records = Vec::new();
            r.replay_all(|rec| {
                records.push(rec);
                Ok(())
            })
            .unwrap();
            assert_eq!(records.len(), 2);
        }

        let _ = std::fs::remove_file(&dir);
        let _ = std::fs::remove_file(&backup);
    }

    #[test]
    fn test_auto_rotate_triggers_at_limit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vanta.wal");

        let mut w =
            WalWriter::open_with_buffer(&path, crate::config::SyncMode::Periodic, 4096, None)
                .unwrap();
        // Tiny limit — any append triggers rotation
        w.max_segment_size = 1;

        w.append(&WalRecord::Insert(UnifiedNode::new(42))).unwrap();
        w.sync().unwrap();

        // An archived segment must exist
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(
            entries
                .iter()
                .any(|e| e.file_name().to_string_lossy().contains("vanta.wal.")),
            "Expected archived segment, got: {:?}",
            entries
                .iter()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect::<Vec<_>>()
        );

        // bytes_written reset to header-only size
        assert_eq!(w.bytes_written, WalHeader::SIZE as u64);
        assert_eq!(w.record_count, 0);
    }

    #[test]
    fn test_auto_rotate_not_before_limit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vanta.wal");

        let mut w =
            WalWriter::open_with_buffer(&path, crate::config::SyncMode::Periodic, 4096, None)
                .unwrap();
        // Default max_segment_size is 256MB — one small record won't trigger
        w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
        w.sync().unwrap();

        assert!(w.bytes_written > WalHeader::SIZE as u64);
        assert_eq!(w.record_count, 1);

        // No archived segment should exist
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(
            !entries
                .iter()
                .any(|e| e.file_name().to_string_lossy().contains("vanta.wal.")),
            "Unexpected archived segment found: {:?}",
            entries
        );
    }

    #[test]
    fn test_auto_rotate_preserves_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vanta.wal");

        let mut w =
            WalWriter::open_with_buffer(&path, crate::config::SyncMode::Periodic, 4096, None)
                .unwrap();
        // Each 300-float record serializes to ~1260 bytes. With max_segment_size=3700,
        // 3 records fill ~3780 bytes exceeding the limit at the 3rd append. This guarantees
        // the archive contains exactly the 3 original records written before rotation.
        w.max_segment_size = 3700;

        let records = vec![
            WalRecord::Insert(UnifiedNode::with_vector(1, vec![0.0; 300])),
            WalRecord::Insert(UnifiedNode::with_vector(2, vec![0.0; 300])),
            WalRecord::Insert(UnifiedNode::with_vector(3, vec![0.0; 300])),
        ];
        for rec in &records {
            w.append(rec).unwrap();
        }
        w.sync().unwrap();

        // Find the archived segment
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let archive = entries
            .iter()
            .find(|e| e.file_name().to_string_lossy().contains("vanta.wal."))
            .expect("Expected archived segment");
        let archive_path = archive.path();

        // Read the archived segment and verify records
        let mut reader = WalReader::open(&archive_path).unwrap();
        let mut recovered: Vec<WalRecord> = Vec::new();
        while let Some(rec) = reader.next_record().unwrap() {
            recovered.push(rec);
        }

        assert_eq!(recovered.len(), 3, "Archived segment should have 3 records");
        assert!(matches!(&recovered[0], WalRecord::Insert(n) if n.id == 1));
        assert!(matches!(&recovered[1], WalRecord::Insert(n) if n.id == 2));
        assert!(matches!(&recovered[2], WalRecord::Insert(n) if n.id == 3));
    }

    #[test]
    fn test_recover_mid_file_corruption_scan_forward_recovers_tail() {
        // FIND-34: mid-file corrupt bytes must be skipped via Scan-Forward,
        // tail records after the corruption must still be recovered.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vanta.wal");

        // 1. Write 2 valid records normally
        {
            let mut w =
                WalWriter::open_with_buffer(&path, crate::config::SyncMode::Periodic, 4096, None)
                    .unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(10))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(20))).unwrap();
            w.sync().unwrap();
        }

        // 2. Inject 16 bytes of garbage mid-tail, then a valid 3rd record
        //    after the garbage using raw framing (len+payload+crc+chain). The
        //    64-byte chain tail (prev_hash ‖ record_hash) is irrelevant here —
        //    scan-forward validates CRC + deser only, chain is `verify`'s job.
        let third = WalRecord::Insert(UnifiedNode::new(30));
        let payload = postcard::to_allocvec(&third).unwrap();
        let crc = crate::wal::compute_crc32c(&payload);
        {
            let mut file = OpenOptions::new().append(true).open(&path).unwrap();
            // 16 bytes that will never be a valid record (len=0xffffffff > 10M)
            file.write_all(&[0xFFu8; 16]).unwrap();
            file.write_all(&(payload.len() as u32).to_le_bytes())
                .unwrap();
            file.write_all(&payload).unwrap();
            file.write_all(&crc.to_le_bytes()).unwrap();
            // VER-01: v3 frames carry prev_hash ‖ record_hash (64 bytes).
            file.write_all(&[0u8; CHAIN_FRAME_EXTRA]).unwrap();
            file.sync_data().unwrap();
        }

        // 3. Reopen — recover_valid_records must scan-forward past the 16
        //    corrupt bytes and count the 3rd record.
        {
            let w =
                WalWriter::open_with_buffer(&path, crate::config::SyncMode::Periodic, 4096, None)
                    .unwrap();
            assert_eq!(
                w.record_count(),
                3,
                "scan-forward should recover tail after mid-file corruption"
            );
        }

        // 4. WalReader must also yield 3 records in order
        {
            let mut r = WalReader::open(&path).unwrap();
            let mut records = Vec::new();
            r.replay_all(|rec| {
                records.push(rec);
                Ok(())
            })
            .unwrap();
            assert_eq!(records.len(), 3);
            assert!(matches!(&records[0], WalRecord::Insert(n) if n.id == 10));
            assert!(matches!(&records[1], WalRecord::Insert(n) if n.id == 20));
            assert!(matches!(&records[2], WalRecord::Insert(n) if n.id == 30));
        }
    }

    #[test]
    fn test_quarantine_rotates_when_corrupt_exists() {
        // FIND-34: second corruption must not overwrite first .corrupt backup —
        // quarantine_backup_path rotates to .corrupt.1
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vanta.wal");
        let backup0 = PathBuf::from(format!("{}.corrupt", path.display()));
        let backup1 = PathBuf::from(format!("{}.corrupt.1", path.display()));
        let _ = std::fs::remove_file(&backup0);
        let _ = std::fs::remove_file(&backup1);

        // First cycle: 2 valid → corrupt tail → recover creates .corrupt
        {
            let mut w =
                WalWriter::open_with_buffer(&path, crate::config::SyncMode::Periodic, 4096, None)
                    .unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(2))).unwrap();
            w.sync().unwrap();
        }
        {
            let mut file = OpenOptions::new().append(true).open(&path).unwrap();
            file.write_all(b"\x00\xff\xff\xfffirst-corrupt-tail")
                .unwrap();
            file.sync_data().unwrap();
        }
        {
            let w =
                WalWriter::open_with_buffer(&path, crate::config::SyncMode::Periodic, 4096, None)
                    .unwrap();
            assert_eq!(w.record_count(), 2);
        }
        assert!(backup0.exists(), "first quarantine backup must exist");
        let first_backup = std::fs::read(&backup0).unwrap();

        // Second cycle: corrupt again → must create .corrupt.1, keep .corrupt
        {
            let mut file = OpenOptions::new().append(true).open(&path).unwrap();
            file.write_all(b"\x11\x22\x33\x44second-corrupt-tail")
                .unwrap();
            file.sync_data().unwrap();
        }
        {
            let w =
                WalWriter::open_with_buffer(&path, crate::config::SyncMode::Periodic, 4096, None)
                    .unwrap();
            assert_eq!(w.record_count(), 2);
        }
        assert!(
            backup0.exists(),
            ".corrupt must still exist after second quarantine"
        );
        assert!(
            backup1.exists(),
            ".corrupt.1 must be created on second quarantine"
        );
        let second_backup = std::fs::read(&backup1).unwrap();
        assert_ne!(
            first_backup, second_backup,
            "rotated backups must be distinct"
        );
        assert!(
            second_backup
                .windows(b"second-corrupt-tail".len())
                .any(|w| w == b"second-corrupt-tail"),
            "second backup should contain second tail"
        );
    }

    // ─── VER-01: hash-chain framing + verification ─────────────────

    /// (start, end) of every chained frame in a v3 WAL byte image.
    fn chained_frame_bounds(bytes: &[u8]) -> Vec<(usize, usize)> {
        let mut frames = Vec::new();
        let mut off = WalHeader::SIZE;
        while off + 8 + CHAIN_FRAME_EXTRA <= bytes.len() {
            let len = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()) as usize;
            let end = off + 4 + len + 4 + CHAIN_FRAME_EXTRA;
            if end > bytes.len() {
                break;
            }
            frames.push((off, end));
            off = end;
        }
        frames
    }

    /// Write `count` records into `path` (opening — or reopening — the WAL).
    fn write_chain_wal(path: &Path, count: u128) {
        let mut w = WalWriter::open(path, crate::config::SyncMode::Periodic).unwrap();
        for i in 1..=count {
            w.append(&WalRecord::Insert(UnifiedNode::new(i))).unwrap();
        }
        w.sync().unwrap();
    }

    #[test]
    fn wal_chain_verify_clean_wal_is_verified() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clean.wal");
        write_chain_wal(&path, 3);

        let report = verify_wal_file(&path).unwrap();
        assert!(
            matches!(report.status, WalVerifyStatus::Verified),
            "clean chain must verify: {:?}",
            report.status
        );
        assert_eq!(report.records, 3);
        assert_eq!(report.format_version, WAL_FORMAT_VERSION);
    }

    #[test]
    fn wal_chain_verify_detects_altered_record_with_valid_crc() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("altered.wal");
        // Record 2 carries a string field so its payload can be altered in a
        // way that stays postcard-decodable (content tamper, not garbage).
        {
            let mut w = WalWriter::open(&path, crate::config::SyncMode::Periodic).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            let mut node = UnifiedNode::new(2);
            node.set_field("k", crate::node::FieldValue::String("AAAA".to_string()));
            w.append(&WalRecord::Insert(node)).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(3))).unwrap();
            w.sync().unwrap();
        }

        let mut bytes = std::fs::read(&path).unwrap();
        let frames = chained_frame_bounds(&bytes);
        assert_eq!(frames.len(), 3);
        let (start, _) = frames[1];
        let len = u32::from_le_bytes(bytes[start..start + 4].try_into().unwrap()) as usize;
        let payload_start = start + 4;
        // Flip 'AAAA' → 'BAAA' inside record 2's payload: same length, valid
        // UTF-8/postcard, so the frame stays fully decodable...
        let marker = bytes[payload_start..payload_start + len]
            .windows(4)
            .position(|w| w == b"AAAA")
            .expect("string field must be present in record 2 payload");
        bytes[payload_start + marker] = b'B';
        // ...and recompute its CRC32C so the frame is internally consistent.
        let crc = compute_crc32c(&bytes[payload_start..payload_start + len]);
        let crc_start = payload_start + len;
        bytes[crc_start..crc_start + 4].copy_from_slice(&crc.to_le_bytes());
        std::fs::write(&path, &bytes).unwrap();

        // The CRC-recomputed, decodable content tamper is invisible to the
        // replay reader (CRC + deser both pass — it replays tampered data)...
        let mut r = WalReader::open(&path).unwrap();
        let mut count = 0;
        while r.next_record().unwrap().is_some() {
            count += 1;
        }
        assert_eq!(
            count, 3,
            "CRC-recomputed content tamper still replays — CRC alone cannot catch it"
        );

        // ...but the chain catches it at the exact frame.
        let report = verify_wal_file(&path).unwrap();
        match &report.status {
            WalVerifyStatus::Tampered {
                offset,
                record,
                reason,
            } => {
                assert_eq!(
                    *offset, frames[1].0 as u64,
                    "tamper must be reported at record 2's offset"
                );
                assert_eq!(*record, 2);
                assert!(reason.contains("altered"), "reason: {reason}");
            }
            other => panic!("expected Tampered, got {other:?}"),
        }
    }

    #[test]
    fn wal_chain_verify_detects_removed_record() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("removed.wal");
        write_chain_wal(&path, 4);

        let bytes = std::fs::read(&path).unwrap();
        let frames = chained_frame_bounds(&bytes);
        assert_eq!(frames.len(), 4);
        // Extirpate record 2: record 3's prev_hash now points at a record that
        // is no longer on disk.
        let mut tampered = Vec::with_capacity(bytes.len());
        tampered.extend_from_slice(&bytes[..frames[1].0]);
        tampered.extend_from_slice(&bytes[frames[2].0..]);
        std::fs::write(&path, &tampered).unwrap();

        let report = verify_wal_file(&path).unwrap();
        match &report.status {
            WalVerifyStatus::Tampered {
                offset,
                record,
                reason,
            } => {
                assert_eq!(*offset, frames[1].0 as u64);
                assert_eq!(*record, 2);
                assert!(reason.contains("link broken"), "reason: {reason}");
            }
            other => panic!("expected Tampered, got {other:?}"),
        }
    }

    #[test]
    fn wal_chain_continues_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reopen.wal");
        write_chain_wal(&path, 2);
        // Reopen: the writer must resume the chain from the stored record_hash.
        write_chain_wal(&path, 2);

        let report = verify_wal_file(&path).unwrap();
        assert!(
            matches!(report.status, WalVerifyStatus::Verified),
            "chain must continue across reopens: {:?}",
            report.status
        );
        assert_eq!(report.records, 4);
    }

    #[test]
    fn wal_chain_verify_reports_legacy_for_v2_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy_v2.wal");
        let mut bytes = WalHeader::new(2).serialize().to_vec();
        for i in 1..=2u128 {
            let payload = postcard::to_allocvec(&WalRecord::Insert(UnifiedNode::new(i))).unwrap();
            bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&payload);
            bytes.extend_from_slice(&compute_crc32c(&payload).to_le_bytes());
        }
        std::fs::write(&path, &bytes).unwrap();

        // Compat (C4): the v3 reader still replays the legacy file.
        let mut r = WalReader::open(&path).unwrap();
        let mut count = 0;
        while r.next_record().unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 2, "v3 reader must read pre-chain (v2) files");

        let report = verify_wal_file(&path).unwrap();
        assert!(
            matches!(report.status, WalVerifyStatus::Legacy),
            "pre-chain file reports as legacy, never as tampered: {:?}",
            report.status
        );
        assert_eq!(report.records, 2);
    }

    #[test]
    fn wal_writer_keeps_legacy_framing_on_v2_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy_append.wal");
        // Seed a v2 file with one legacy record.
        let mut bytes = WalHeader::new(2).serialize().to_vec();
        let payload = postcard::to_allocvec(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload);
        bytes.extend_from_slice(&compute_crc32c(&payload).to_le_bytes());
        std::fs::write(&path, &bytes).unwrap();

        // Appending to an existing legacy file keeps its (legacy) framing —
        // migration happens via rotation/compaction, never in place.
        {
            let mut w = WalWriter::open(&path, crate::config::SyncMode::Periodic).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(2))).unwrap();
            w.sync().unwrap();
        }
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(
            u16::from_le_bytes([bytes[4], bytes[5]]),
            2,
            "existing file keeps its format version (no in-place upgrade)"
        );
        let mut r = WalReader::open(&path).unwrap();
        let mut count = 0;
        while r.next_record().unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 2, "legacy framing append must replay");
    }

    #[test]
    fn wal_chain_verify_reports_incomplete_tail_not_tamper() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("torn.wal");
        write_chain_wal(&path, 2);
        {
            let mut f = OpenOptions::new().append(true).open(&path).unwrap();
            f.write_all(&[0xAB; 5]).unwrap();
            f.sync_data().unwrap();
        }

        let report = verify_wal_file(&path).unwrap();
        assert!(
            matches!(report.status, WalVerifyStatus::IncompleteTail { .. }),
            "a torn crash tail is not tamper: {:?}",
            report.status
        );
        assert!(report.status.is_ok());
    }

    #[test]
    fn wal_chain_verify_boundary_truncation_is_documented_limit() {
        // Truncating at a clean record boundary deletes records without any
        // in-file trace — undetectable without an external anchor (documented
        // limit of the chain; signature/attestation is a v1.0 follow-up). This
        // test pins that semantics on purpose so nobody mistakes it for a gap.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("truncated.wal");
        write_chain_wal(&path, 3);
        let bytes = std::fs::read(&path).unwrap();
        let frames = chained_frame_bounds(&bytes);

        OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(frames[1].1 as u64)
            .unwrap();

        let report = verify_wal_file(&path).unwrap();
        assert!(matches!(report.status, WalVerifyStatus::Verified));
        assert_eq!(
            report.records, 2,
            "clean-boundary tail truncation is undetectable (documented limit)"
        );
    }

    /// W2 gate: **without a key the WAL bytes do not change**.
    ///
    /// The frame layout is rebuilt here from scratch (an independent
    /// implementation of the v3 layout) and compared byte for byte with the
    /// file the writer produced. The header is compared too, except for its
    /// creation timestamp, which was always variable.
    #[test]
    fn wal_plaintext_path_is_byte_identical_without_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vanta.wal");
        let records = vec![
            WalRecord::Insert(UnifiedNode::new(7)),
            WalRecord::Delete { id: 7 },
        ];

        {
            let mut w = WalWriter::open(&path, crate::config::SyncMode::Periodic).unwrap();
            for record in &records {
                w.append(record).unwrap();
            }
            w.sync().unwrap();
        }

        let bytes = std::fs::read(&path).unwrap();

        // The header keeps the plaintext format version.
        let header = WalHeader::deserialize(&bytes[..WalHeader::SIZE]).unwrap();
        assert_eq!(header.base.format_version, WAL_FORMAT_VERSION);
        assert_eq!(&bytes[..4], b"VWAL");

        // The frames are the v3 layout, byte for byte:
        // [len u32 LE][payload][crc u32 LE][prev_hash][record_hash].
        let mut expected = Vec::new();
        let mut prev = CHAIN_GENESIS;
        for record in &records {
            let payload = postcard::to_allocvec(record).unwrap();
            let len = payload.len() as u32;
            let crc = compute_crc32c(&payload);
            let hash = chain_hash(&prev, &len.to_le_bytes(), &payload, &crc.to_le_bytes());
            expected.extend_from_slice(&len.to_le_bytes());
            expected.extend_from_slice(&payload);
            expected.extend_from_slice(&crc.to_le_bytes());
            expected.extend_from_slice(&prev);
            expected.extend_from_slice(&hash);
            prev = hash;
        }
        assert_eq!(
            &bytes[WalHeader::SIZE..],
            &expected[..],
            "the plaintext frame layout must not change"
        );
    }

    /// W2 gate, sharded path: **without a key the shard bytes do not
    /// change** either — the sharded WAL uses the very same frame layout.
    #[test]
    fn wal_sharded_plaintext_path_is_byte_identical_without_key() {
        use crate::wal_sharded::ShardedWal;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vanta.wal");
        let records: Vec<WalRecord> = (1..=8u128)
            .map(|i| WalRecord::Insert(UnifiedNode::new(i)))
            .collect();

        {
            let sw = ShardedWal::new(&path, 4, crate::config::SyncMode::Periodic).unwrap();
            for record in &records {
                sw.append(record).unwrap();
            }
            sw.flush_all().unwrap();
        }

        for i in 0..4usize {
            let shard = dir.path().join(format!("vanta.shard{i}.wal"));
            let bytes = std::fs::read(&shard).unwrap();
            let header = WalHeader::deserialize(&bytes[..WalHeader::SIZE]).unwrap();
            assert_eq!(header.base.format_version, WAL_FORMAT_VERSION);

            // The round-robin gave shard `i` the records `i` and `i + 4`.
            let mut expected = Vec::new();
            let mut prev = CHAIN_GENESIS;
            for record in [records[i].clone(), records[i + 4].clone()] {
                let payload = postcard::to_allocvec(&record).unwrap();
                let len = payload.len() as u32;
                let crc = compute_crc32c(&payload);
                let hash = chain_hash(&prev, &len.to_le_bytes(), &payload, &crc.to_le_bytes());
                expected.extend_from_slice(&len.to_le_bytes());
                expected.extend_from_slice(&payload);
                expected.extend_from_slice(&crc.to_le_bytes());
                expected.extend_from_slice(&prev);
                expected.extend_from_slice(&hash);
                prev = hash;
            }
            assert_eq!(
                &bytes[WalHeader::SIZE..],
                &expected[..],
                "the plaintext frame layout must not change"
            );
        }
    }
    // ─── FIND-343: physical erasure of the deleted payloads ──────────

    /// A node whose payload carries the canary.
    fn node_with_canary(id: u128, canary: &str) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.set_field(
            "payload",
            crate::node::FieldValue::String(canary.to_string()),
        );
        node
    }

    /// The canary must not survive in the segment any more.
    fn segment_contains(path: &std::path::Path, canary: &str) -> bool {
        let bytes = std::fs::read(path).unwrap_or_default();
        bytes.windows(canary.len()).any(|w| w == canary.as_bytes())
    }

    /// FIND-343: the payload of a deleted node is physically erased, the
    /// record **count** is preserved and the chain still verifies.
    #[test]
    fn redact_erased_replaces_the_payload_with_a_tombstone() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vanta.wal");
        let canary = "FIND343-wal-canary-payload-4b1d";

        let mut writer = WalWriter::open(&path, crate::config::SyncMode::Periodic).unwrap();
        writer
            .append(&WalRecord::Insert(node_with_canary(7, canary)))
            .unwrap();
        writer.append(&WalRecord::Delete { id: 7 }).unwrap();
        writer
            .append(&WalRecord::Insert(node_with_canary(8, "live-payload")))
            .unwrap();
        writer.sync().unwrap();
        let count_before = writer.record_count();
        assert!(segment_contains(&path, canary));

        let outcome = writer
            .redact_erased(&|record| matches!(record, WalRecord::Insert(n) if n.id == 7))
            .unwrap();
        assert_eq!(outcome.redacted, 1, "the deleted node's record is redacted");
        assert_eq!(
            writer.record_count(),
            count_before,
            "the record count must not change: the checkpoint_seq semantics depend on it"
        );
        assert!(
            !segment_contains(&path, canary),
            "the deleted payload survived in the segment"
        );
        assert!(
            segment_contains(&path, "live-payload"),
            "the payload of a live node must survive"
        );

        // The chain still verifies after the rewrite (VER-01).
        let report = verify_wal_file(&path).unwrap();
        assert!(
            matches!(&report.status, WalVerifyStatus::Verified),
            "the rewritten segment must keep a valid chain, got {:?}",
            report.status
        );
        assert_eq!(report.records, count_before);
    }

    /// FIND-343: a node that was **not** deleted keeps its payload — the
    /// erasure only touches the deleted nodes.
    #[test]
    fn redact_erased_keeps_the_payload_of_a_live_node() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vanta.wal");
        let canary = "FIND343-live-canary-9c2e";

        let mut writer = WalWriter::open(&path, crate::config::SyncMode::Periodic).unwrap();
        writer
            .append(&WalRecord::Insert(node_with_canary(9, canary)))
            .unwrap();
        writer.sync().unwrap();

        let outcome = writer.redact_erased(&|_| false).unwrap();
        assert_eq!(outcome.redacted, 0);
        assert!(segment_contains(&path, canary));
    }

    /// FIND-343: the scratch file of a redaction must never be discovered by
    /// the WAL shipper (which matches `*.wal.*` and would read a half-written
    /// file as a segment).
    #[test]
    fn redact_scratch_file_is_not_discoverable_by_the_shipper() {
        for name in ["vanta.wal", "vanta.shard0.wal", "vanta.wal.1700000000000"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join(name);
            let scratch = scratch_path(&path);
            let file_name = scratch
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let shipper_match = file_name.contains(".wal.") && !file_name.ends_with(".wal");
            assert!(
                !shipper_match,
                "the scratch file '{file_name}' would be shipped as a WAL segment"
            );
        }
    }

    /// FIND-343: the replayed state does not change — a redacted node is
    /// absent, exactly like the node that was deleted.
    #[test]
    fn redact_erased_keeps_the_replayed_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vanta.wal");

        let mut writer = WalWriter::open(&path, crate::config::SyncMode::Periodic).unwrap();
        writer
            .append(&WalRecord::Insert(node_with_canary(1, "gone")))
            .unwrap();
        writer.append(&WalRecord::Delete { id: 1 }).unwrap();
        writer
            .append(&WalRecord::Insert(node_with_canary(2, "kept")))
            .unwrap();
        writer.sync().unwrap();

        writer
            .redact_erased(&|record| matches!(record, WalRecord::Insert(n) if n.id == 1))
            .unwrap();

        let mut reader = WalReader::open(&path).unwrap();
        let mut live = std::collections::HashSet::new();
        while let Some(record) = reader.next_record().unwrap() {
            match record {
                WalRecord::Insert(node) => {
                    live.insert(node.id);
                }
                WalRecord::Delete { id } => {
                    live.remove(&id);
                }
                _ => {}
            }
        }
        assert_eq!(
            live,
            std::collections::HashSet::from([2u128]),
            "the replayed state must be identical after the redaction"
        );
    }
}

// ─── W2: the encrypted WAL (format v4) ─────────────────────

#[cfg(all(test, feature = "encryption"))]
mod encryption_tests {
    use super::*;
    use crate::crypto::Cipher;
    use crate::node::UnifiedNode;
    use crate::wal_sharded::ShardedWal;

    /// A 32-byte master key.
    fn master(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    /// The WAL path inside a data dir.
    fn wal_path(dir: &Path) -> PathBuf {
        dir.join("vanta.wal")
    }

    /// Write `records` into an encrypted WAL and return the file bytes.
    fn write_encrypted(dir: &Path, seed: u8, records: &[WalRecord]) -> Vec<u8> {
        let path = wal_path(dir);
        let crypto = WalCrypto::open(dir, &master(seed)).unwrap();
        let mut w =
            WalWriter::open_encrypted(&path, crate::config::SyncMode::Periodic, crypto).unwrap();
        for record in records {
            w.append(record).unwrap();
        }
        w.sync().unwrap();
        std::fs::read(&path).unwrap()
    }

    /// The nonce of every frame of an encrypted WAL.
    fn v4_nonces(bytes: &[u8]) -> Vec<[u8; 12]> {
        let mut out = Vec::new();
        for (start, _) in v4_frames(bytes) {
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(&bytes[start + 8..start + 20]);
            out.push(nonce);
        }
        out
    }

    /// The byte range of every frame of an encrypted WAL.
    fn v4_frames(bytes: &[u8]) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut pos = WalHeader::SIZE;
        while pos + 20 <= bytes.len() {
            let ct_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
            let end = pos + 20 + ct_len + CHAIN_FRAME_EXTRA;
            out.push((pos, end));
            pos = end;
        }
        out
    }

    /// The error of an operation that must fail.
    fn err_of<T>(result: Result<T>) -> Error {
        match result {
            Ok(_) => panic!("expected an error"),
            Err(err) => err,
        }
    }

    /// The records of a WAL, as their postcard bytes.
    fn replay_bytes(path: &Path, key: Option<&WalKey>) -> Vec<Vec<u8>> {
        let mut reader = match key {
            Some(k) => WalReader::open_with_key(path, k.clone()).unwrap(),
            None => WalReader::open(path).unwrap(),
        };
        let mut out = Vec::new();
        reader
            .replay_all(|record| {
                out.push(postcard::to_allocvec(&record).unwrap());
                Ok(())
            })
            .unwrap();
        out
    }

    #[test]
    fn wal_v4_roundtrip_encrypt_decrypt() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        let records = vec![
            WalRecord::Insert(UnifiedNode::new(1)),
            WalRecord::Insert(UnifiedNode::new(2)),
            WalRecord::Delete { id: 1 },
        ];
        let bytes = write_encrypted(dir.path(), 0x11, &records);

        // The file declares the encrypted format in its cleartext header.
        let header = WalHeader::deserialize(&bytes[..WalHeader::SIZE]).unwrap();
        assert_eq!(header.base.format_version, WAL_ENCRYPTED_FORMAT_VERSION);

        // The plaintext never reaches the disk. A canary needs to be long
        // enough not to appear in random ciphertext by chance: 8 bytes give a
        // 2^-64 hit per position, while a 2-byte record would be a coin flip.
        let mut canaries = 0usize;
        for record in &records {
            let payload = postcard::to_allocvec(record).unwrap();
            if payload.len() < 8 {
                continue;
            }
            canaries += 1;
            assert!(
                !bytes
                    .windows(payload.len())
                    .any(|w| w == payload.as_slice()),
                "the plaintext payload must not be in the WAL"
            );
        }
        assert!(canaries > 0, "the canary set must not be empty");

        // And it comes back with the key.
        let key = WalKey::open(dir.path(), &master(0x11)).unwrap();
        let replayed = replay_bytes(&path, Some(&key));
        assert_eq!(replayed.len(), 3);
        for (record, expected) in replayed.iter().zip(records.iter()) {
            assert_eq!(record, &postcard::to_allocvec(expected).unwrap());
        }
    }

    #[test]
    fn wal_v4_replays_to_the_same_records_as_v3() {
        let dir = tempfile::tempdir().unwrap();
        let records: Vec<WalRecord> = (1..=5u128)
            .map(|i| WalRecord::Insert(UnifiedNode::new(i)))
            .collect();

        // The plaintext WAL.
        let plain_path = dir.path().join("plain.wal");
        {
            let mut w = WalWriter::open(&plain_path, crate::config::SyncMode::Periodic).unwrap();
            for record in &records {
                w.append(record).unwrap();
            }
            w.sync().unwrap();
        }
        let plain = replay_bytes(&plain_path, None);

        // The encrypted WAL.
        let crypto = WalCrypto::open(dir.path(), &master(0x12)).unwrap();
        let enc_path = wal_path(dir.path());
        {
            let mut w =
                WalWriter::open_encrypted(&enc_path, crate::config::SyncMode::Periodic, crypto)
                    .unwrap();
            for record in &records {
                w.append(record).unwrap();
            }
            w.sync().unwrap();
        }
        let key = WalKey::open(dir.path(), &master(0x12)).unwrap();
        let encrypted = replay_bytes(&enc_path, Some(&key));

        assert_eq!(plain, encrypted, "the replay must yield the same records");
        assert_eq!(plain.len(), records.len());
    }

    #[test]
    fn wal_v3_file_still_replays_on_a_v4_reader() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        let records = vec![
            WalRecord::Insert(UnifiedNode::new(1)),
            WalRecord::Delete { id: 1 },
        ];
        {
            let mut w = WalWriter::open(&path, crate::config::SyncMode::Periodic).unwrap();
            for record in &records {
                w.append(record).unwrap();
            }
            w.sync().unwrap();
        }

        // A v3 file is read exactly as before — no key involved.
        let bytes = std::fs::read(&path).unwrap();
        let header = WalHeader::deserialize(&bytes[..WalHeader::SIZE]).unwrap();
        assert_eq!(header.base.format_version, WAL_FORMAT_VERSION);
        let replayed = replay_bytes(&path, None);
        assert_eq!(replayed.len(), 2);

        // And a v3 file opened **with** a key stays plaintext: no mixed mode,
        // no implicit migration (ADR-0057 D1/D7).
        let crypto = WalCrypto::open(dir.path(), &master(0x13)).unwrap();
        let mut w =
            WalWriter::open_encrypted(&path, crate::config::SyncMode::Periodic, crypto).unwrap();
        w.append(&WalRecord::Insert(UnifiedNode::new(9))).unwrap();
        w.sync().unwrap();
        drop(w);

        let bytes = std::fs::read(&path).unwrap();
        let header = WalHeader::deserialize(&bytes[..WalHeader::SIZE]).unwrap();
        assert_eq!(
            header.base.format_version, WAL_FORMAT_VERSION,
            "a plaintext WAL must not be upgraded in place"
        );
        let key = WalKey::open(dir.path(), &master(0x13)).unwrap();
        assert_eq!(replay_bytes(&path, Some(&key)).len(), 3);
    }

    #[test]
    fn wal_v4_detects_a_corrupted_ciphertext_as_corrupt_tail() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        let records = vec![
            WalRecord::Insert(UnifiedNode::new(1)),
            WalRecord::Insert(UnifiedNode::new(2)),
            WalRecord::Insert(UnifiedNode::new(3)),
        ];
        write_encrypted(dir.path(), 0x21, &records);

        // Corrupt one byte of the **last** frame's ciphertext: the CRC fails,
        // so the tail is corrupt (truncatable) — not "wrong key".
        let mut bytes = std::fs::read(&path).unwrap();
        let frames = v4_frames(&bytes);
        bytes[frames[2].0 + 20] ^= 0xFF;
        std::fs::write(&path, &bytes).unwrap();

        // Reopening truncates the corrupt tail and keeps the valid records.
        let crypto = WalCrypto::open(dir.path(), &master(0x21)).unwrap();
        let w =
            WalWriter::open_encrypted(&path, crate::config::SyncMode::Periodic, crypto).unwrap();
        assert_eq!(w.record_count(), 2, "the corrupt tail must be dropped");
        drop(w);

        // The tail is quarantined for forensics.
        let quarantine = PathBuf::from(format!("{}.corrupt", path.display()));
        assert!(quarantine.exists(), "the corrupt tail must be quarantined");

        // And the surviving records replay fine.
        let key = WalKey::open(dir.path(), &master(0x21)).unwrap();
        assert_eq!(replay_bytes(&path, Some(&key)).len(), 2);
    }

    #[test]
    fn wal_v4_rejects_a_valid_crc_with_a_wrong_key_as_a_hard_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        write_encrypted(dir.path(), 0x31, &[WalRecord::Insert(UnifiedNode::new(1))]);

        // (a) The wrong **master** key: the registry's key check refuses it —
        // and refuses it as "key rejected", never as "corrupt".
        let err = WalKey::open(dir.path(), &master(0x99)).unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "got: {err}"
        );

        // (b) A DEK that does not match the file: the CRC still passes (the
        // ciphertext bytes are intact) but the AEAD tag fails → a hard error,
        // not a truncatable tail.
        let wrong_dek = WalKey {
            cipher: Cipher::new(&master(0x98)),
        };
        let len_before = std::fs::metadata(&path).unwrap().len();
        let mut reader = WalReader::open_with_key(&path, wrong_dek).unwrap();
        let err = reader.next_record().unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "got: {err}"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().len(),
            len_before,
            "a rejected frame must not truncate the WAL"
        );
    }

    #[test]
    fn wal_v4_without_a_key_fails_with_key_required() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        write_encrypted(dir.path(), 0x81, &[WalRecord::Insert(UnifiedNode::new(1))]);

        // The file says "encrypted" and there is no key: an explicit error,
        // never "corrupt" (ADR-0057 D7).
        let err = err_of(WalReader::open(&path));
        assert!(
            matches!(err, Error::EncryptionKeyRequired { .. }),
            "got: {err}"
        );

        let err = err_of(WalWriter::open(&path, crate::config::SyncMode::Periodic));
        assert!(
            matches!(err, Error::EncryptionKeyRequired { .. }),
            "got: {err}"
        );
    }

    #[test]
    fn wal_v4_chain_verifies_without_the_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        write_encrypted(
            dir.path(),
            0x41,
            &[
                WalRecord::Insert(UnifiedNode::new(1)),
                WalRecord::Insert(UnifiedNode::new(2)),
            ],
        );

        // The chain and the CRC cover the ciphertext: the verification works
        // without the key (ADR-0057 D5).
        let report = verify_wal_file(&path).unwrap();
        assert!(
            matches!(report.status, WalVerifyStatus::Verified),
            "got: {:?}",
            report.status
        );
        assert_eq!(report.records, 2);
        assert_eq!(report.format_version, WAL_ENCRYPTED_FORMAT_VERSION);
    }

    #[test]
    fn wal_v4_chain_detects_a_reordered_frame() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        write_encrypted(
            dir.path(),
            0x42,
            &[
                WalRecord::Insert(UnifiedNode::new(1)),
                WalRecord::Insert(UnifiedNode::new(2)),
            ],
        );

        // Swap the two frames: the chain link of the second frame no longer
        // matches its predecessor.
        let bytes = std::fs::read(&path).unwrap();
        let frames = v4_frames(&bytes);
        assert_eq!(frames[0].1 - frames[0].0, frames[1].1 - frames[1].0);
        let (s0, e0) = frames[0];
        let (s1, e1) = frames[1];
        let mut swapped = bytes[..s0].to_vec();
        swapped.extend_from_slice(&bytes[s1..e1]);
        swapped.extend_from_slice(&bytes[s0..e0]);
        swapped.extend_from_slice(&bytes[e1..]);
        std::fs::write(&path, &swapped).unwrap();

        let report = verify_wal_file(&path).unwrap();
        assert!(
            matches!(report.status, WalVerifyStatus::Tampered { .. }),
            "got: {:?}",
            report.status
        );
    }

    #[test]
    fn wal_v4_frame_cannot_be_moved_to_another_ordinal() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        write_encrypted(
            dir.path(),
            0x43,
            &[
                WalRecord::Insert(UnifiedNode::new(1)),
                WalRecord::Insert(UnifiedNode::new(2)),
            ],
        );

        // Swap the two frames: every frame keeps its own ciphertext (so the
        // CRC still passes) but lands on the other ordinal. The AAD binds the
        // ciphertext to its ordinal → the AEAD tag fails.
        let bytes = std::fs::read(&path).unwrap();
        let frames = v4_frames(&bytes);
        assert_eq!(frames[0].1 - frames[0].0, frames[1].1 - frames[1].0);
        let (s0, e0) = frames[0];
        let (s1, e1) = frames[1];
        let mut swapped = bytes[..s0].to_vec();
        swapped.extend_from_slice(&bytes[s1..e1]);
        swapped.extend_from_slice(&bytes[s0..e0]);
        swapped.extend_from_slice(&bytes[e1..]);
        std::fs::write(&path, &swapped).unwrap();

        let key = WalKey::open(dir.path(), &master(0x43)).unwrap();
        let mut reader = WalReader::open_with_key(&path, key).unwrap();
        let err = reader.next_record().unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "a moved frame must not authenticate: got {err}"
        );
    }

    #[test]
    fn wal_v4_nonce_is_unique_across_10000_frames() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        let crypto = WalCrypto::open(dir.path(), &master(0x51)).unwrap();
        let mut w =
            WalWriter::open_encrypted(&path, crate::config::SyncMode::Periodic, crypto).unwrap();
        for i in 0..10_000u128 {
            w.append(&WalRecord::Insert(UnifiedNode::new(i))).unwrap();
        }
        w.sync().unwrap();
        drop(w);

        let bytes = std::fs::read(&path).unwrap();
        let nonces = v4_nonces(&bytes);
        assert_eq!(nonces.len(), 10_000);
        let unique: std::collections::HashSet<&[u8; 12]> = nonces.iter().collect();
        assert_eq!(unique.len(), 10_000, "a nonce repeated");
    }

    #[test]
    fn wal_v4_counter_overflow_allocates_a_new_epoch() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        let crypto = WalCrypto::open(dir.path(), &master(0x52)).unwrap();
        // Drive the shared counter to the edge: the next allocation must roll
        // the epoch instead of wrapping (ADR-0057 D3).
        crypto.allocator.set_counter_for_test(u32::MAX - 1);

        let mut w =
            WalWriter::open_encrypted(&path, crate::config::SyncMode::Periodic, crypto).unwrap();
        w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
        w.append(&WalRecord::Insert(UnifiedNode::new(2))).unwrap();
        w.sync().unwrap();
        drop(w);

        let bytes = std::fs::read(&path).unwrap();
        let nonces = v4_nonces(&bytes);
        assert_eq!(nonces.len(), 2);
        assert_ne!(nonces[0], nonces[1], "the nonce must not repeat");
        assert_ne!(
            nonces[0][..8],
            nonces[1][..8],
            "the exhausted counter must roll into a new epoch"
        );

        // And the file still replays: the epoch travels in the nonce.
        let key = WalKey::open(dir.path(), &master(0x52)).unwrap();
        assert_eq!(replay_bytes(&path, Some(&key)).len(), 2);
    }

    #[test]
    fn wal_v4_survives_a_crash_tail_truncation() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        write_encrypted(
            dir.path(),
            0x61,
            &[
                WalRecord::Insert(UnifiedNode::new(1)),
                WalRecord::Insert(UnifiedNode::new(2)),
                WalRecord::Insert(UnifiedNode::new(3)),
            ],
        );

        // Simulate a crash in the middle of the last frame.
        let bytes = std::fs::read(&path).unwrap();
        let frames = v4_frames(&bytes);
        let cut = frames[2].0 + 10;
        OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(cut as u64)
            .unwrap();

        // Reopening truncates the torn frame and keeps the valid records.
        let crypto = WalCrypto::open(dir.path(), &master(0x61)).unwrap();
        let w =
            WalWriter::open_encrypted(&path, crate::config::SyncMode::Periodic, crypto).unwrap();
        assert_eq!(w.record_count(), 2, "the torn frame must be dropped");
        drop(w);

        let key = WalKey::open(dir.path(), &master(0x61)).unwrap();
        assert_eq!(replay_bytes(&path, Some(&key)).len(), 2);
    }

    /// The same crash, triggered by the `wal_append_fail` failpoint: the
    /// append dies and the WAL stays consistent (the valid frames survive).
    #[cfg(feature = "failpoints")]
    #[test]
    fn wal_v4_survives_a_crash_tail_truncation_failpoint() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        let crypto = WalCrypto::open(dir.path(), &master(0x62)).unwrap();
        let mut w =
            WalWriter::open_encrypted(&path, crate::config::SyncMode::Periodic, crypto).unwrap();
        w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
        w.sync().unwrap();

        // Arm the failpoint: the next append dies like a crash.
        fail::cfg("wal_append_fail", "return").expect("arm wal_append_fail");
        let err = w
            .append(&WalRecord::Insert(UnifiedNode::new(2)))
            .unwrap_err();
        assert!(
            err.to_string().contains("Simulated WAL append"),
            "got: {err}"
        );
        // Disarm it: the failpoint is a process-wide setting.
        fail::cfg("wal_append_fail", "off").expect("disarm wal_append_fail");
        drop(w);

        // The WAL still replays the frames that were written.
        let key = WalKey::open(dir.path(), &master(0x62)).unwrap();
        assert_eq!(replay_bytes(&path, Some(&key)).len(), 1);
    }

    /// The shipping path reads a WAL segment with a [`WalReader`]: a v4
    /// segment is readable with the key. The shipper itself is given the key
    /// in the activation wave (W6) — this test pins the read contract it
    /// will use.
    #[test]
    fn wal_shipping_reads_v4_segments() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        let records = vec![
            WalRecord::Insert(UnifiedNode::new(1)),
            WalRecord::Insert(UnifiedNode::new(2)),
        ];
        write_encrypted(dir.path(), 0x71, &records);

        let key = WalKey::open(dir.path(), &master(0x71)).unwrap();
        let mut reader = WalReader::open_with_key(&path, key).unwrap();
        let mut shipped = 0u64;
        while let Some(_record) = reader.next_record().unwrap() {
            shipped += 1;
        }
        assert_eq!(shipped, 2);
    }

    #[test]
    fn wal_v4_sharded_roundtrip_encrypt_decrypt() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        let crypto = WalCrypto::open(dir.path(), &master(0x91)).unwrap();
        let sw = ShardedWal::new_encrypted(
            &path,
            4,
            crate::config::SyncMode::Periodic,
            64 * 1024,
            None,
            crypto,
        )
        .unwrap();
        for i in 1..=8u128 {
            sw.append(&WalRecord::Insert(UnifiedNode::new(i))).unwrap();
        }
        sw.flush_all().unwrap();

        // Every shard file is the encrypted format and holds no plaintext.
        for i in 0..4usize {
            let shard = dir.path().join(format!("vanta.shard{i}.wal"));
            let bytes = std::fs::read(&shard).unwrap();
            let header = WalHeader::deserialize(&bytes[..WalHeader::SIZE]).unwrap();
            assert_eq!(header.base.format_version, WAL_ENCRYPTED_FORMAT_VERSION);
            let payload = postcard::to_allocvec(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            assert!(
                payload.len() >= 8,
                "the canary must be long enough not to match by chance"
            );
            assert!(
                !bytes
                    .windows(payload.len())
                    .any(|w| w == payload.as_slice()),
                "the plaintext payload must not be in the shard"
            );
        }

        // The recovery reads them back — the shard readers use the key.
        let mut recovered = 0u64;
        sw.recover(0, |_record| {
            recovered += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(recovered, 8);
    }

    #[test]
    fn wal_v4_sharded_rejects_a_wrong_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        let crypto = WalCrypto::open(dir.path(), &master(0x92)).unwrap();
        let sw = ShardedWal::new_encrypted(
            &path,
            2,
            crate::config::SyncMode::Periodic,
            64 * 1024,
            None,
            crypto,
        )
        .unwrap();
        sw.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
        sw.flush_all().unwrap();
        drop(sw);

        // The wrong master key is refused as "key rejected", never "corrupt".
        let err = WalCrypto::open(dir.path(), &master(0x93)).unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "got: {err}"
        );
    }

    #[test]
    fn wal_v4_sharded_nonces_are_unique_across_shards() {
        let dir = tempfile::tempdir().unwrap();
        let path = wal_path(dir.path());
        let crypto = WalCrypto::open(dir.path(), &master(0x94)).unwrap();
        let sw = ShardedWal::new_encrypted(
            &path,
            4,
            crate::config::SyncMode::Periodic,
            64 * 1024,
            None,
            crypto,
        )
        .unwrap();
        for i in 0..200u128 {
            sw.append(&WalRecord::Insert(UnifiedNode::new(i))).unwrap();
        }
        sw.flush_all().unwrap();
        drop(sw);

        // The shards share one nonce space: no nonce repeats across them.
        let mut all = Vec::new();
        for i in 0..4usize {
            let shard = dir.path().join(format!("vanta.shard{i}.wal"));
            all.extend(v4_nonces(&std::fs::read(&shard).unwrap()));
        }
        assert_eq!(all.len(), 200);
        let unique: std::collections::HashSet<&[u8; 12]> = all.iter().collect();
        assert_eq!(unique.len(), 200, "a nonce repeated across shards");
    }
}
