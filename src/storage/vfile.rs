// ponytail: memory-mapped alignment invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Memory-mapped vector store file (File) with read/write and in-memory variants.
//!
//! When the `encryption` feature is enabled, File can optionally hold a
//! `Cipher` instance for transparent at-rest encryption. The cipher is stored
//! for use by the storage layer and can be retrieved via `File::cipher`.
//!
//! The mmap primitives (memmap2 re-export / shim, SIGBUS handler, resident byte
//! accounting, `AlignedBytes`) live in `crate::storage::vfile_mmap` and are
//! re-exported here so `crate::storage::vfile::*` paths keep resolving
//! (REVIEW-04 split).

use crate::binary_header::Header;
#[cfg(feature = "encryption")]
use crate::crypto::{Cipher, EncryptionStream};
use crate::error::{Error, Result};
use crate::node::DiskNodeHeader;
#[cfg(unix)]
use libc;
use std::fs::{File as StdFile, OpenOptions};
use std::path::PathBuf;
#[cfg(feature = "encryption")]
use std::sync::Arc;
use zerocopy::{FromBytes, IntoBytes};

use crate::storage::engine::STORAGE_ALIGNMENT;
#[cfg(feature = "encryption")]
use crate::storage::vfile_crypto::VFileCrypto;
use crate::storage::vfile_crypto::VFileCryptoSlot;

// Re-exports preserved from the pre-split single file: public paths
// (`get_resident_bytes`), crate-internal paths (mmap types, sigbus handler,
// aligned buffer) all keep resolving via `crate::storage::vfile::*`.
#[cfg(unix)]
pub(crate) use crate::storage::vfile_mmap::install_sigbus_handler;
pub use crate::storage::vfile_mmap::{get_resident_bytes, get_resident_bytes_impl};
pub(crate) use crate::storage::vfile_mmap::{
    map_readonly, map_readwrite, AlignedBytes, Mmap, MmapMut,
};

/// Current **plaintext** File format version.
/// Version history:
///   - v1: initial format
///   - v2: migrated (bumped header only, data layout identical to v1)
///
/// The plaintext version is **not** bumped by the at-rest encryption: a
/// segment written without a key keeps its exact byte layout, forever
/// (ADR-0057 D1, rule 1). The encrypted format has its own version,
/// [`VFILE_ENCRYPTED_VERSION`].
pub const VFILE_VERSION: u16 = 2;

/// Format version of the **encrypted** VantaFile (ADR-0057 W3).
///
/// The encrypted format wraps **every node record** in its own AEAD
/// envelope:
///
/// ```text
/// [u32 blob_len][12B nonce][ct ‖ 16B tag]
/// ```
///
/// * `blob_len` is the length of the AEAD blob (`ciphertext ‖ tag`); the
///   plaintext is `DiskNodeHeader(64B) ‖ vector payload`.
/// * The marker lives in the **existing cleartext header**
///   (`Header.format_version`), so a reader decides by the bytes on disk,
///   never by configuration (ADR-0057 D1).
/// * A v2 segment is read exactly as before; a v3 segment without a key is
///   an explicit `EncryptionKeyRequired`, never "corrupt" (ADR-0057 D7).
pub const VFILE_ENCRYPTED_VERSION: u16 = 3;

/// The highest VantaFile format version this build can read.
///
/// v3 (the encrypted format) is only readable with the `encryption` feature:
/// a build without it refuses a v3 segment with an explicit message.
pub const VFILE_MAX_READABLE_VERSION: u16 = if cfg!(feature = "encryption") {
    VFILE_ENCRYPTED_VERSION
} else {
    VFILE_VERSION
};

/// Byte offset of the vector payload **inside** the plaintext blob of an
/// encrypted record (the `DiskNodeHeader` occupies the first 64 bytes).
///
/// In the encrypted format `header.vector_offset` is no longer a file
/// address: the payload lives inside the AEAD envelope, so the reader
/// addresses it inside the decrypted blob (ADR-0057 W3).
#[cfg(feature = "encryption")]
pub(crate) const ENCRYPTED_PAYLOAD_OFFSET: u64 = 64;

// ─── The decrypted-node cache (FIND-598 · Paso 0) ───────────────────────
//
// The envelope of the VantaFile is **per node** (`seal_node`), so every
// candidate the HNSW traversal visits costs one AEAD. The cache of
// decrypted nodes is what makes the second visit free: it used to hold
// **one** entry (the last decrypted node), so the traversal — which visits
// a *different* node on every step — never hit it.
//
// The cache is bounded (rule 4 of `.opencode/rules/memory-budget.md`: every
// resident structure declares a cap) and it hands out `Arc<AlignedBlob>`:
// the consumer that cloned the `Arc` keeps the plaintext alive even when
// the cache evicts (or replaces) the entry.

/// The process-wide effective memory limit (bytes) used to size the
/// decrypted-node cache of every segment.
///
/// `0` = "not configured": the total RAM of the machine is used instead.
/// The storage engine sets it at open with the **effective** limit
/// (`config.memory_limit` or the total RAM), so the cache stays a fraction
/// of the budget the operator declared.
#[cfg(feature = "encryption")]
static DECRYPT_CACHE_MEMORY_LIMIT: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// The minimum byte budget of the decrypted-node cache of one segment.
#[cfg(feature = "encryption")]
const MIN_DECRYPT_CACHE_BUDGET: u64 = 16 * 1024 * 1024;

/// The maximum byte budget of the decrypted-node cache of one segment.
#[cfg(feature = "encryption")]
const MAX_DECRYPT_CACHE_BUDGET: u64 = 256 * 1024 * 1024;

/// The nominal size of a cached node, used to derive the **entry** cap of
/// the LRU from the byte budget (the byte budget is the real bound; this
/// one only keeps the LRU from growing without a ceiling of its own).
#[cfg(feature = "encryption")]
const NOMINAL_NODE_BYTES: u64 = 4096;

/// Declare the effective memory limit used to size the decrypted-node
/// cache of every segment.
///
/// Called by the storage engine at open. Process-wide: the last open wins
/// (the process already shares one key registry and one nonce space).
#[cfg(feature = "encryption")]
pub(crate) fn set_decrypt_cache_memory_limit(bytes: u64) {
    DECRYPT_CACHE_MEMORY_LIMIT.store(bytes, std::sync::atomic::Ordering::Relaxed);
}

/// No-op without the `encryption` feature: there is no decrypted cache.
#[cfg(not(feature = "encryption"))]
pub(crate) fn set_decrypt_cache_memory_limit(_bytes: u64) {}

/// The byte budget of the decrypted-node cache of **one** segment.
///
/// The cache is a *fraction* of the effective memory limit: the HNSW index
/// is 100 % RAM-resident (rule 1 of `.opencode/rules/memory-budget.md`) and
/// the cache is not the only consumer of the memory. The engine opens at
/// most 4 segments (L0..L3), so the worst case is 4 × this budget.
#[cfg(feature = "encryption")]
fn decrypt_cache_budget() -> u64 {
    let declared = DECRYPT_CACHE_MEMORY_LIMIT.load(std::sync::atomic::Ordering::Relaxed);
    let limit = if declared == 0 {
        crate::hardware::HardwareCapabilities::global().total_memory
    } else {
        declared
    };
    (limit / 16).clamp(MIN_DECRYPT_CACHE_BUDGET, MAX_DECRYPT_CACHE_BUDGET)
}

/// The bounded cache of decrypted nodes of an encrypted segment.
#[cfg(feature = "encryption")]
struct DecryptCache {
    /// The decrypted blobs, keyed by the offset of their node.
    entries: lru::LruCache<u64, Arc<AlignedBlob>>,
    /// The bytes currently held by `entries`.
    bytes: u64,
    /// The byte budget (`decrypt_cache_budget`).
    budget: u64,
}

#[cfg(feature = "encryption")]
impl DecryptCache {
    fn new(budget: u64) -> Self {
        let cap = std::num::NonZeroUsize::new((budget / NOMINAL_NODE_BYTES) as usize)
            .unwrap_or(std::num::NonZeroUsize::MIN);
        Self {
            entries: lru::LruCache::new(cap),
            bytes: 0,
            budget,
        }
    }

    /// The decrypted blob of `offset`, if it is cached.
    ///
    /// A hit refreshes the recency: the entry becomes the most recently
    /// used one, so the eviction takes the coldest node first.
    fn get(&mut self, offset: u64) -> Option<Arc<AlignedBlob>> {
        self.entries.get(&offset).cloned()
    }

    /// Cache the decrypted blob of `offset`.
    ///
    /// The budget is enforced by evicting the least recently used entries;
    /// a read never fails because of the cache.
    fn insert(&mut self, offset: u64, blob: Arc<AlignedBlob>) {
        let size = blob.len() as u64;
        if let Some((_, replaced)) = self.entries.push(offset, blob) {
            self.bytes = self.bytes.saturating_sub(replaced.len() as u64);
        }
        self.bytes += size;
        while self.bytes > self.budget {
            match self.entries.pop_lru() {
                Some((_, evicted)) => {
                    self.bytes = self.bytes.saturating_sub(evicted.len() as u64);
                }
                None => break,
            }
        }
    }

    /// Drop the entry of `offset` (its plaintext changed on disk).
    fn invalidate(&mut self, offset: u64) {
        if let Some(replaced) = self.entries.pop(&offset) {
            self.bytes = self.bytes.saturating_sub(replaced.len() as u64);
        }
    }

    /// Drop every entry (the whole segment was replaced).
    fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }
}

/// The copies the encrypted read path performs (test-only instrumentation).
///
/// FIND-598 is justified by a **count**, not by an opinion: this probe
/// counts the copies of the node payload that the encrypted read path
/// performs, so the test can pin the number (5 before the Paso 0, 3 after).
#[cfg(all(test, feature = "encryption"))]
pub(crate) mod copy_probe {
    use std::cell::Cell;

    thread_local! {
        /// The number of copy operations.
        static COPIES: Cell<u64> = const { Cell::new(0) };
        /// The total number of bytes copied.
        static BYTES: Cell<u64> = const { Cell::new(0) };
    }

    /// Record one copy of `bytes` bytes.
    ///
    /// The counters are **thread-local**: the test runner runs the tests in
    /// parallel and a global counter would mix their numbers.
    pub(crate) fn record(bytes: usize) {
        COPIES.with(|c| c.set(c.get() + 1));
        BYTES.with(|b| b.set(b.get() + bytes as u64));
    }

    /// Reset the counters.
    pub(crate) fn reset() {
        COPIES.with(|c| c.set(0));
        BYTES.with(|b| b.set(0));
    }

    /// The `(copies, bytes)` counted so far.
    pub(crate) fn snapshot() -> (u64, u64) {
        (COPIES.with(|c| c.get()), BYTES.with(|b| b.get()))
    }
}

/// Record a copy of the node payload (no-op outside the test build).
#[cfg(feature = "encryption")]
#[inline]
fn count_copy(bytes: usize) {
    #[cfg(test)]
    copy_probe::record(bytes);
    #[cfg(not(test))]
    let _ = bytes;
}

/// The payload size (un-aligned) of a persisted header.
///
/// Mirrors the writer in [`crate::storage::ops::write_node_to_vstore`]:
/// ADR-032 dispatches the payload size from the in-memory representation,
/// and legacy files with kind=0 and len>0 are read as FULL.
pub(crate) fn payload_len_for_header(header: &DiskNodeHeader) -> u64 {
    let kind = crate::node::NodeFlags::vector_kind(header.flags);
    // kind==0 is legacy (pre-ADR-032) where FULL vs NONE is distinguished by len
    if kind == 0 {
        if header.vector_len == 0 {
            return 0;
        }
        return (header.vector_len as u64).checked_mul(4).unwrap_or(0);
    }
    match kind {
        crate::node::NodeFlags::VECTOR_KIND_NONE => 0,
        crate::node::NodeFlags::VECTOR_KIND_FULL => {
            (header.vector_len as u64).checked_mul(4).unwrap_or(0)
        }
        crate::node::NodeFlags::VECTOR_KIND_BINARY => {
            (header.vector_len as u64).checked_mul(8).unwrap_or(0)
        }
        crate::node::NodeFlags::VECTOR_KIND_TURBO => header.vector_len as u64,
        crate::node::NodeFlags::VECTOR_KIND_SQ8 => header.vector_len as u64 + 4,
        _ => header.vector_len as u64, // unknown future kind: byte count
    }
}

/// A byte buffer whose base pointer is guaranteed to be **8-byte aligned**.
///
/// The engine reads `f32`/`u64` payloads out of the node bytes with
/// `align_to`, which needs an aligned base: 8 bytes for the `u64` (BINARY)
/// payloads, 4 for the `f32` (FULL) ones. A `Vec<u8>` is only 1-byte
/// aligned, so the blob lives in a `Vec<u64>` (whose alignment is 8) and is
/// viewed as bytes.
#[cfg(feature = "encryption")]
pub(crate) struct AlignedBlob {
    words: Vec<u64>,
    len: usize,
}

#[cfg(feature = "encryption")]
impl std::fmt::Debug for AlignedBlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlignedBlob")
            .field("len", &self.len)
            .finish()
    }
}

#[cfg(feature = "encryption")]
impl AlignedBlob {
    /// Copy `bytes` into an 8-byte aligned buffer.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let words = vec![0u64; bytes.len().div_ceil(8)];
        let mut blob = Self {
            words,
            len: bytes.len(),
        };
        blob.as_mut_slice().copy_from_slice(bytes);
        count_copy(bytes.len());
        blob
    }

    /// The bytes of the blob.
    pub(crate) fn as_slice(&self) -> &[u8] {
        // SAFETY: `align_to` is a transmute of the middle slice's elements.
        // `u8` has no invalid bit patterns and its alignment (1) is weaker
        // than the source's (8), so the prefix and the suffix are empty and
        // the byte view covers the whole buffer.
        let (_, bytes, _) = unsafe { self.words.align_to::<u8>() };
        &bytes[..self.len]
    }

    /// The length of the blob in bytes.
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// The bytes of the blob, mutably.
    pub(crate) fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: same as `as_slice`, on the mutable variant: `u8` has no
        // invalid bit patterns and the prefix/suffix are empty.
        let (_, bytes, _) = unsafe { self.words.align_to_mut::<u8>() };
        &mut bytes[..self.len]
    }
}

/// The bytes a node's vector payload is addressed within.
///
/// * **Plaintext segment**: the whole mapping — the payload lives at the
///   header's `vector_offset`, a *file address*.
/// * **Encrypted segment**: the decrypted node blob — the payload lives at
///   the header's `vector_offset`, which is the *in-blob* offset
///   ([`ENCRYPTED_PAYLOAD_OFFSET`]).
///
/// Both variants are 4-byte aligned (the mmap base is page-aligned, the
/// blob base is 8-byte aligned), so the `f32`/`u64` payload reads keep
/// working unchanged.
pub(crate) enum NodeBytes<'a> {
    /// The whole mapping of a plaintext segment.
    Mapping(&'a [u8]),
    /// The decrypted blob of an encrypted node.
    #[cfg(feature = "encryption")]
    Blob(Arc<AlignedBlob>),
}

impl NodeBytes<'_> {
    /// The bytes the payload is addressed within.
    pub(crate) fn as_slice(&self) -> &[u8] {
        match self {
            NodeBytes::Mapping(bytes) => bytes,
            #[cfg(feature = "encryption")]
            NodeBytes::Blob(blob) => blob.as_slice(),
        }
    }
}

/// Sum of resident mmap bytes across the HNSW index and vector store.
/// Only compiled for tests — production code uses per-File metrics directly.
#[cfg(test)]
pub(crate) fn engine_mmap_resident_bytes(
    hnsw: &crate::index::CPIndex,
    vector_store: &File,
) -> Option<u64> {
    let mut total = None;
    for resident in [
        vector_store.mmap_resident_bytes(),
        hnsw.backend.mmap_resident_bytes(),
    ]
    .into_iter()
    .flatten()
    {
        total = Some(total.unwrap_or(0) + resident);
    }
    total
}

enum FileMap {
    ReadOnly(Mmap),
    ReadWrite(MmapMut),
    InMemory(AlignedBytes),
}

impl FileMap {
    fn as_slice(&self) -> &[u8] {
        match self {
            FileMap::ReadOnly(m) => m,
            FileMap::ReadWrite(m) => m,
            FileMap::InMemory(d) => d.as_slice(),
        }
    }
    fn as_ptr(&self) -> *const u8 {
        match self {
            FileMap::ReadOnly(m) => m.as_ptr(),
            FileMap::ReadWrite(m) => m.as_ptr(),
            FileMap::InMemory(d) => d.as_ptr(),
        }
    }
    fn len(&self) -> usize {
        match self {
            FileMap::ReadOnly(m) => m.len(),
            FileMap::ReadWrite(m) => m.len(),
            FileMap::InMemory(d) => d.len(),
        }
    }
    fn as_mut_slice(&mut self) -> Result<&mut [u8]> {
        match self {
            FileMap::ReadOnly(_) => Err(Error::Validation {
                field: "read_only".into(),
                reason: "File is read-only".into(),
            }),
            FileMap::ReadWrite(m) => Ok(m),
            FileMap::InMemory(d) => Ok(d.as_mut_slice()),
        }
    }
    fn flush(&self) -> Result<()> {
        match self {
            FileMap::ReadOnly(_) => Ok(()),
            FileMap::ReadWrite(m) => m.flush().map_err(Error::Io),
            FileMap::InMemory(_) => Ok(()),
        }
    }
}

/// A memory-mapped vector store file supporting read, write, and in-memory modes.
pub struct File {
    /// Optional backing file handle (None for in-memory mode).
    pub file: Option<StdFile>,
    mmap: FileMap,
    /// File system path to the backing file.
    pub path: PathBuf,
    /// Current file size in bytes.
    pub size: u64,
    /// Byte offset for the next write operation.
    pub write_cursor: u64,
    read_only: bool,
    /// The format version of this segment (2 = plaintext, 3 = encrypted).
    ///
    /// Read from the cleartext header, so the reader decides by the bytes
    /// on disk, never by configuration (ADR-0057 D1).
    format_version: u16,
    /// The encryption context of the `vfile` artefact (`None` = plaintext).
    #[cfg(feature = "encryption")]
    pub(crate) crypto: Option<Arc<VFileCrypto>>,
    /// The bounded cache of decrypted nodes (FIND-598 · Paso 0).
    ///
    /// It replaces the one-entry cache that lived here before: the HNSW
    /// traversal visits a *different* node on every step, so a one-entry
    /// cache never hit and **every** candidate paid a full AEAD.
    #[cfg(feature = "encryption")]
    decrypted: parking_lot::Mutex<DecryptCache>,
    /// AES-256-GCM cipher for at-rest encryption when the `encryption` feature
    /// is enabled and `VANTADB_ENCRYPTION_KEY` is set.
    #[cfg(feature = "encryption")]
    pub cipher: Option<Cipher>,
}

// SAFETY: File owns a `File` handle, a `FileMap` (Mmap/MmapMut/AlignedBytes),
// a `PathBuf`, and an `AtomicBool` — all of which are `Send`. The mmap pointers
// are managed by the memmap2 crate or the in-memory/shim buffers (AlignedBytes,
// `unsafe impl Send + Sync` above), all `Send + Sync`. The cipher field (behind
// `#[cfg(feature = "encryption")]`) is Send by construction, and so is the
// decrypted-node cache (a `Mutex` over an `LruCache` of `Arc<AlignedBlob>`). No
// mutable aliasing crosses threads because all mutations go through `&mut self`
// or the storage engine's locks.
unsafe impl Send for File {}
// SAFETY: same reasoning — all fields are Sync-safe, and the engine serializes
// read-write access through `RwLock<File>`.
unsafe impl Sync for File {}

impl File {
    /// Open or create a File at the given path with the specified initial size.
    pub fn open(path: PathBuf, initial_size: u64) -> Result<Self> {
        Self::open_with_mode(path, initial_size, false)
    }
    /// Open an existing File in read-only mode.
    pub fn open_read_only(path: PathBuf) -> Result<Self> {
        Self::open_with_mode(path, 0, true)
    }

    /// Open or create a File with an **encryption context** (ADR-0057 W3).
    ///
    /// The context decides the format of a **fresh** segment: with a key the
    /// segment is created in the encrypted format (v3), without one it stays
    /// plaintext (v2) and byte-identical to the layout before this change.
    /// An **existing** segment keeps its own format: a v2 segment is never
    /// migrated in place (ADR-0057 D7) and a v3 segment without a key is an
    /// explicit `EncryptionKeyRequired`.
    pub(crate) fn open_with_crypto(
        path: PathBuf,
        initial_size: u64,
        read_only: bool,
        crypto: VFileCryptoSlot,
    ) -> Result<Self> {
        Self::open_with_mode_and_crypto(path, initial_size, read_only, crypto)
    }

    /// Create a File backed entirely by in-memory storage (no disk I/O).
    pub fn create_in_memory(initial_size: u64) -> Self {
        let size = initial_size.max(STORAGE_ALIGNMENT);
        // `AlignedBytes::zeroed` guarantees a 4-aligned base so `f32` vector
        // reads are never misaligned (AUDIT-03; `Vec<u8>` would only be align-1).
        // Single-use constructor contract: the only failure mode is OOM at
        // construction time (equivalent to `Vec::with_capacity` aborting on
        // allocation failure), so a documented panic here is intentional — the
        // long-lived store paths (`map`/`map_mut`/`grow_to`) propagate instead.
        // INVARIANT (B2b, cat. (b)): see above — intentional, documented, OOM-only.
        let mut data = AlignedBytes::zeroed(size as usize)
            .expect("in-memory vstore allocation failed at construction (OOM)");
        let header = Header::new(*b"VFLE", VFILE_VERSION, 0);
        data.as_mut_slice()[0..16].copy_from_slice(&header.serialize());
        data.as_mut_slice()[16..24].copy_from_slice(&STORAGE_ALIGNMENT.to_le_bytes());
        Self {
            file: None,
            mmap: FileMap::InMemory(data),
            path: PathBuf::new(),
            size,
            write_cursor: STORAGE_ALIGNMENT,
            read_only: false,
            format_version: VFILE_VERSION,
            #[cfg(feature = "encryption")]
            crypto: None,
            #[cfg(feature = "encryption")]
            decrypted: parking_lot::Mutex::new(DecryptCache::new(decrypt_cache_budget())),
            #[cfg(feature = "encryption")]
            cipher: None,
        }
    }

    fn open_with_mode(path: PathBuf, initial_size: u64, read_only: bool) -> Result<Self> {
        Self::open_with_mode_and_crypto(path, initial_size, read_only, None)
    }

    fn open_with_mode_and_crypto(
        path: PathBuf,
        initial_size: u64,
        read_only: bool,
        crypto: VFileCryptoSlot,
    ) -> Result<Self> {
        let file = if read_only {
            OpenOptions::new()
                .read(true)
                .open(&path)
                .map_err(Error::Io)?
        } else {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&path)
                .map_err(Error::Io)?
        };
        let mut current_size = file.metadata().map_err(Error::Io)?.len();
        let min_header_size = 64u64;
        if current_size < min_header_size {
            if read_only {
                return Err(Error::Validation {
                    field: "file_size".into(),
                    reason: format!("File {} too small", path.display()),
                });
            }
            current_size = initial_size.max(min_header_size);
            file.set_len(current_size).map_err(Error::Io)?;
        }
        // `map_readonly`/`map_readwrite` carry the (memmap2-only) SAFETY
        // contract: `file` is a valid open handle at the correct size, and the
        // returned mapping is stored in `self.mmap` for the `File`'s
        // lifetime.
        let mut mmap = if read_only {
            FileMap::ReadOnly(map_readonly(&file).map_err(Error::Io)?)
        } else {
            FileMap::ReadWrite(map_readwrite(&file).map_err(Error::Io)?)
        };
        // A **fresh** segment takes the format of the context: encrypted (v3)
        // when a key resolved, plaintext (v2) otherwise. An existing segment
        // keeps the format its own header declares (ADR-0057 D1/D7).
        let fresh_version = if crypto.is_some() {
            VFILE_ENCRYPTED_VERSION
        } else {
            VFILE_VERSION
        };
        if !read_only && current_size >= min_header_size && &mmap.as_slice()[0..4] != b"VFLE" {
            let header = Header::new(*b"VFLE", fresh_version, 0);
            mmap.as_mut_slice()?[0..16].copy_from_slice(&header.serialize());
            mmap.as_mut_slice()?[16..24].copy_from_slice(&STORAGE_ALIGNMENT.to_le_bytes());
            // Zero-fill the remainder of the header block (bytes 24..64) to
            // ensure a clean slate for a potentially corrupt or uninitialized file.
            mmap.as_mut_slice()?[24..STORAGE_ALIGNMENT as usize].fill(0);
            mmap.flush()?;
        }
        let header = Header::deserialize(&mmap.as_slice()[0..16])?;
        // An encrypted segment in a build without the `encryption` feature:
        // say so explicitly — an encrypted database must never be reported as
        // "corrupt" (ADR-0057 D7).
        if header.format_version >= VFILE_ENCRYPTED_VERSION && !cfg!(feature = "encryption") {
            return Err(Error::IncompatibleFormat {
                expected_magic: *b"VFLE",
                expected_version: VFILE_MAX_READABLE_VERSION,
                found_magic: header.magic,
                found_version: header.format_version,
                hint: "File is encrypted (format v3 or newer) but this build has no \
                       'encryption' feature"
                    .to_string(),
            });
        }
        // An encrypted segment without a key: the operator has to supply one.
        // Never "corrupt" (ADR-0057 D7).
        if header.format_version >= VFILE_ENCRYPTED_VERSION && crypto.is_none() {
            #[cfg(feature = "encryption")]
            return Err(Error::EncryptionKeyRequired {
                artifact: crate::storage::vfile_crypto::ARTIFACT.to_string(),
            });
            #[cfg(not(feature = "encryption"))]
            return Err(Error::IncompatibleFormat {
                expected_magic: *b"VFLE",
                expected_version: VFILE_MAX_READABLE_VERSION,
                found_magic: header.magic,
                found_version: header.format_version,
                hint: "File is encrypted (format v3 or newer) but no encryption key resolved"
                    .to_string(),
            });
        }
        header.validate_compat(*b"VFLE", VFILE_MAX_READABLE_VERSION, "File format")?;
        let cursor = u64::from_le_bytes(
            mmap.as_slice()[16..24]
                .try_into()
                .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?,
        );
        let write_cursor = if cursor < STORAGE_ALIGNMENT || cursor > current_size {
            STORAGE_ALIGNMENT
        } else {
            (cursor + 63) & !63
        };
        Ok(Self {
            file: Some(file),
            mmap,
            path,
            size: current_size,
            write_cursor,
            read_only,
            format_version: header.format_version,
            #[cfg(feature = "encryption")]
            crypto,
            #[cfg(feature = "encryption")]
            decrypted: parking_lot::Mutex::new(DecryptCache::new(decrypt_cache_budget())),
            #[cfg(feature = "encryption")]
            cipher: None,
        })
    }

    /// Persist the write cursor position into the file header.
    pub fn save_cursor(&mut self) -> Result<()> {
        self.mmap.as_mut_slice()?[16..24].copy_from_slice(&self.write_cursor.to_le_bytes());
        Ok(())
    }
    /// Return a byte slice over the entire mapped region.
    pub fn mmap_bytes(&self) -> &[u8] {
        self.mmap.as_slice()
    }
    /// Return a mutable byte slice over the entire mapped region.
    pub fn mmap_bytes_mut(&mut self) -> Result<&mut [u8]> {
        self.mmap.as_mut_slice()
    }
    /// Re-map the backing file into a new mutable memory mapping.
    pub(crate) fn remap_mut(&mut self) -> Result<()> {
        if self.read_only {
            return Err(Error::Validation {
                field: "read_only".into(),
                reason: "read-only".into(),
            });
        }
        if matches!(&self.mmap, FileMap::InMemory(_)) {
            return Ok(());
        }
        let file = self.file.as_ref().ok_or_else(|| Error::Validation {
            field: "backing_file".into(),
            reason: "no backing file".into(),
        })?;
        // `map_readwrite` carries the (memmap2-only) SAFETY contract: `file` is
        // the existing backing handle at `self.size` bytes; the previous mapping
        // is dropped (safe — memmap2 unmaps on Drop).
        self.mmap = FileMap::ReadWrite(map_readwrite(file).map_err(Error::Io)?);
        Ok(())
    }

    /// Replace the backing file with a new one at the same path and re-map.
    pub(crate) fn replace_backing_file(&mut self, new_size: u64) -> Result<()> {
        if self.read_only {
            return Err(Error::Validation {
                field: "read_only".into(),
                reason: "read-only".into(),
            });
        }
        if matches!(&self.mmap, FileMap::InMemory(_)) {
            self.size = new_size;
            return Ok(());
        }
        let path = self.path.clone();
        let new_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(&path)
            .map_err(Error::Io)?;
        self.file = Some(new_file);
        self.size = new_size;
        // The whole mapping changed: the decrypted cache is stale.
        #[cfg(feature = "encryption")]
        self.decrypted.lock().clear();
        self.remap_mut()
    }

    /// Read a `DiskNodeHeader` from the given aligned offset, if valid.
    ///
    /// Central guard (INV-024 M-1 / AUDIT-03): rejects headers whose vector
    /// payload offset is not 4-byte aligned. `vector_offset` is file data and
    /// is never validated by the writer path on load; a corrupt or adversarial
    /// file could otherwise produce a misaligned `&[f32]` at every
    /// `from_raw_parts` call site (search.rs, archive.rs, engine/ops.rs) — UB
    /// in release. All 7 sites read headers through this function, so the
    /// invariant is enforced in one place.
    ///
    /// In an **encrypted** segment the header lives inside the AEAD
    /// envelope, so it has to be decrypted first. A failure (wrong key,
    /// tampered blob) is reported as "no header" here; the callers that need
    /// the distinction use [`File::read_node`], which propagates the error.
    pub fn read_header(&self, offset: u64) -> Option<DiskNodeHeader> {
        if self.is_encrypted() {
            #[cfg(feature = "encryption")]
            {
                return self.read_header_encrypted(offset).ok().flatten();
            }
            #[cfg(not(feature = "encryption"))]
            {
                // Unreachable: a build without the `encryption` feature
                // refuses to open a v3 segment (the constructor rejects it).
                return None;
            }
        }
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        let end = offset.checked_add(header_size)?;
        if end > self.size || !offset.is_multiple_of(STORAGE_ALIGNMENT) {
            return None;
        }
        let slice = &self.mmap_bytes()[offset as usize..end as usize];
        let header = DiskNodeHeader::read_from_bytes(slice).ok()?;
        if !header.vector_offset.is_multiple_of(4) {
            return None;
        }
        Some(header)
    }

    /// Write a `DiskNodeHeader` at the given aligned offset, replacing existing bytes.
    pub fn write_header(&mut self, offset: u64, header: &DiskNodeHeader) -> Result<()> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        if !offset.is_multiple_of(STORAGE_ALIGNMENT) {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "misaligned",
            )));
        }
        let Some(end) = offset.checked_add(header_size) else {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "out of bounds",
            )));
        };
        if end > self.size {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "out of bounds",
            )));
        }
        self.mmap_bytes_mut()?[offset as usize..end as usize].copy_from_slice(header.as_bytes());
        Ok(())
    }

    /// Whether this segment is encrypted (format v3).
    ///
    /// The answer comes from the **bytes on disk** (the cleartext header),
    /// never from the configuration (ADR-0057 D1).
    pub fn is_encrypted(&self) -> bool {
        self.format_version >= VFILE_ENCRYPTED_VERSION
    }

    /// The distance between two consecutive node records.
    ///
    /// * Plaintext: the 64-byte header plus the 64-byte-aligned payload.
    /// * Encrypted: the record prefix, the header, the payload and the AEAD
    ///   tag, 64-byte aligned — the whole envelope is the record.
    pub(crate) fn node_stride(&self, header_size: u64, payload_len: u64) -> u64 {
        use crate::storage::vfile_crypto::{RECORD_PREFIX_LEN, TAG_LEN};
        if self.is_encrypted() {
            let raw = RECORD_PREFIX_LEN as u64 + header_size + payload_len + TAG_LEN as u64;
            (raw + 63) & !63
        } else {
            header_size + ((payload_len + 63) & !63)
        }
    }

    /// Set (or clear) the tombstone flag of the node at `offset`.
    ///
    /// In an encrypted segment the header lives inside the ciphertext, so the
    /// node is **re-sealed** with a fresh nonce instead of being patched in
    /// place. The record keeps its length, so the offsets of the following
    /// nodes do not move.
    ///
    /// Returns `false` when there is no node at `offset`.
    pub(crate) fn set_tombstone(&mut self, offset: u64, tombstoned: bool) -> Result<bool> {
        if !self.is_encrypted() {
            let Some(mut header) = self.read_header(offset) else {
                return Ok(false);
            };
            let was = (header.flags & crate::storage::engine::FLAG_TOMBSTONE) != 0;
            if was == tombstoned {
                return Ok(true);
            }
            if tombstoned {
                header.flags |= crate::storage::engine::FLAG_TOMBSTONE;
            } else {
                header.flags &= !crate::storage::engine::FLAG_TOMBSTONE;
            }
            self.write_header(offset, &header)?;
            return Ok(true);
        }
        #[cfg(feature = "encryption")]
        {
            let Some(node) = self.read_node(offset)? else {
                return Ok(false);
            };
            let mut header = node.header;
            let was = (header.flags & crate::storage::engine::FLAG_TOMBSTONE) != 0;
            if was == tombstoned {
                return Ok(true);
            }
            if tombstoned {
                header.flags |= crate::storage::engine::FLAG_TOMBSTONE;
            } else {
                header.flags &= !crate::storage::engine::FLAG_TOMBSTONE;
            }
            self.write_node(offset, &header, &node.payload)?;
            Ok(true)
        }
        #[cfg(not(feature = "encryption"))]
        {
            let _ = (offset, tombstoned);
            Ok(false)
        }
    }

    /// The bytes the vector payload of the node at `offset` is addressed
    /// within.
    ///
    /// * Plaintext segment: the whole mapping — the payload lives at the
    ///   header's `vector_offset`, a *file address*.
    /// * Encrypted segment: the decrypted node blob — the payload lives at
    ///   the header's `vector_offset`, which is the *in-blob* offset
    ///   ([`ENCRYPTED_PAYLOAD_OFFSET`]).
    ///
    /// Both variants are 4-byte aligned, so the `f32`/`u64` payload reads of
    /// the engine keep working unchanged.
    pub(crate) fn node_bytes(&self, offset: u64) -> Result<NodeBytes<'_>> {
        if !self.is_encrypted() {
            return Ok(NodeBytes::Mapping(self.mmap_bytes()));
        }
        #[cfg(feature = "encryption")]
        {
            let Some(blob) = self.decrypt_blob(offset)? else {
                return Err(Error::EncryptionCorrupt {
                    artifact: crate::storage::vfile_crypto::ARTIFACT.to_string(),
                    detail: format!("no node record at offset {offset}"),
                });
            };
            Ok(NodeBytes::Blob(blob))
        }
        #[cfg(not(feature = "encryption"))]
        {
            let _ = offset;
            Err(Error::IncompatibleFormat {
                expected_magic: *b"VFLE",
                expected_version: VFILE_MAX_READABLE_VERSION,
                found_magic: *b"VFLE",
                found_version: self.format_version,
                hint: "File is encrypted (format v3 or newer) but this build has no \
                       'encryption' feature"
                    .to_string(),
            })
        }
    }

    /// Extend the file to the given new size, zero-filling added space.
    ///
    /// Shrinking is rejected because the File layout is append-only:
    /// existing node offsets would become invalid. Use `compact_layout` in
    /// `archive.rs` to reclaim space instead.
    pub fn grow_to(&mut self, new_size: u64) -> Result<()> {
        if new_size < self.size {
            return Err(Error::Validation {
                field: "new_size".into(),
                reason: format!(
                    "grow_to called with new_size {} < current size {}",
                    new_size, self.size
                ),
            });
        }
        match &mut self.mmap {
            FileMap::InMemory(data) => {
                data.grow_zeroed(new_size as usize)?;
                self.size = new_size;
                Ok(())
            }
            _ => {
                // AUD-044: flush pending buffer writes before remapping the
                // SAME file. In the no-memmap2 shim, writes live only in the
                // buffer until flush(); remap_mut's drop would silently discard
                // them. This lives in grow_to (not remap_mut) because
                // replace_backing_file's old buffer is stale by design and must
                // be dropped without flushing (it would wastefully rewrite the
                // orphaned inode). In memmap2 builds this is msync on the old
                // mapping — harmless.
                self.mmap.flush()?;
                let file = self.file.as_ref().ok_or_else(|| Error::Validation {
                    field: "backing_file".into(),
                    reason: "no backing file".into(),
                })?;
                file.set_len(new_size).map_err(Error::Io)?;
                self.size = new_size;
                self.remap_mut()
            }
        }
    }

    /// Flush memory-mapped changes to the backing file (no-op for in-memory mode).
    pub fn flush(&self) -> Result<()> {
        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("mmap_flush_fail", |_| Err(Error::Io(
                std::io::Error::other("injected")
            )));
        }
        self.mmap.flush()
    }

    /// Advise the OS to prefetch the given number of bytes from the mapped region.
    pub fn warmup_top_layers(&self, _size: usize) {
        #[cfg(all(unix, feature = "memmap2"))]
        {
            use memmap2::Advice;
            let _ = match &self.mmap {
                FileMap::ReadOnly(m) => m.advise(Advice::WillNeed),
                FileMap::ReadWrite(m) => m.advise(Advice::WillNeed),
                FileMap::InMemory(_) => Ok(()),
            };
        }
        #[cfg(not(unix))]
        {
            let mmap = self.mmap_bytes();
            let len = _size.min(mmap.len());
            let mut _sum = 0u8;
            for i in (0..len).step_by(4096) {
                _sum ^= mmap[i];
            }
        }
    }

    /// Return the number of resident (in-RAM) bytes for this file's mapping.
    pub fn mmap_resident_bytes(&self) -> Option<u64> {
        get_resident_bytes_impl(self.mmap.as_ptr(), self.mmap.len())
    }

    /// Attach an encryption cipher to this File.
    ///
    /// When set, the storage layer should use the cipher to encrypt data before
    /// writing and decrypt after reading. Requires the `encryption` feature.
    #[cfg(feature = "encryption")]
    pub fn with_cipher(mut self, cipher: Cipher) -> Self {
        self.cipher = Some(cipher);
        self
    }

    /// Return a reference to the optional encryption cipher.
    #[cfg(feature = "encryption")]
    pub fn cipher(&self) -> Option<&Cipher> {
        self.cipher.as_ref()
    }

    /// Create an [`EncryptionStream`] wrapping this file's backing [`File`].
    ///
    /// Returns `None` if this File has no backing file (in-memory mode),
    /// or if no cipher is set.
    ///
    /// The stream can be used for transparent encrypt-on-write and
    /// decrypt-on-read operations on the underlying file handle, for example
    /// with WAL or checkpoint files that use stream-based I/O.
    #[cfg(feature = "encryption")]
    pub fn encryption_stream(&self) -> Option<EncryptionStream<&StdFile>> {
        let file = self.file.as_ref()?;
        let cipher = self.cipher.clone()?;
        Some(EncryptionStream::new(file, cipher))
    }
}

/// A node record read from a segment: its header and its vector payload.
#[cfg(feature = "encryption")]
#[derive(Debug)]
pub(crate) struct NodeRecord {
    /// The persisted header.
    pub header: DiskNodeHeader,
    /// The vector payload bytes — exactly the bytes the writer persisted
    /// (the `f32` words, the `u64` words, the raw bytes or the `i8` data
    /// plus the scale tail, depending on the vector kind).
    pub payload: Vec<u8>,
}

/// The encrypted-segment half of [`File`] (ADR-0057 W3).
///
/// Every method here needs the `encryption` feature: a build without it
/// cannot open a v3 segment at all.
#[cfg(feature = "encryption")]
impl File {
    /// Read the node record at `offset`: its header and its vector payload.
    ///
    /// The record is decrypted (one AEAD envelope per node) and the payload
    /// comes out of the plaintext blob.
    ///
    /// `Ok(None)` = no node at that offset; `Err` = the record is broken
    /// (wrong key, tampered ciphertext, truncated blob).
    pub(crate) fn read_node(&self, offset: u64) -> Result<Option<NodeRecord>> {
        self.read_node_encrypted(offset)
    }

    /// Write a node record at `offset` (the header plus the payload).
    ///
    /// The whole record is sealed into one AEAD envelope whose AAD binds it
    /// to `offset`. Returns the length of the record written.
    pub(crate) fn write_node(
        &mut self,
        offset: u64,
        header: &DiskNodeHeader,
        payload: &[u8],
    ) -> Result<u64> {
        let record = self.seal_node(offset, header, payload)?;
        let end = offset + record.len() as u64;
        if end > self.size {
            self.grow_to(end.max(self.size.saturating_mul(2)))?;
        }
        let start = offset as usize;
        self.mmap_bytes_mut()?[start..start + record.len()].copy_from_slice(&record);
        // The node at `offset` changed: its decrypted blob must not be served
        // from the cache any more.
        self.decrypted.lock().invalidate(offset);
        Ok(record.len() as u64)
    }

    /// Read the header of the node record of an **encrypted** segment.
    ///
    /// The header lives inside the AEAD envelope, so the record has to be
    /// decrypted first — but the **payload is not copied**: the header is
    /// parsed in place out of the decrypted blob. FIND-598 (Paso 0): this
    /// path used to materialise the whole 6 KB payload to read 64 bytes of
    /// header.
    fn read_header_encrypted(&self, offset: u64) -> Result<Option<DiskNodeHeader>> {
        let Some(blob) = self.decrypt_blob(offset)? else {
            return Ok(None);
        };
        Ok(Some(header_from_blob(&blob, offset)?))
    }

    /// Read the node record of an **encrypted** segment.
    fn read_node_encrypted(&self, offset: u64) -> Result<Option<NodeRecord>> {
        let Some(blob) = self.decrypt_blob(offset)? else {
            return Ok(None);
        };
        let header = header_from_blob(&blob, offset)?;
        let header_size = std::mem::size_of::<DiskNodeHeader>();
        // The payload is materialised: the caller owns it (the tombstone
        // write re-seals the node with it). This is **not** the read path of
        // the search (`read_header` + `read_full_vector`).
        let payload = blob.as_slice()[header_size..].to_vec();
        count_copy(payload.len());
        Ok(Some(NodeRecord { header, payload }))
    }

    /// Seal a node record into its AEAD envelope.
    ///
    /// The AAD binds the record to `offset` (the **slot**), so a record moved
    /// to another offset no longer authenticates (ADR-0057 D4).
    pub(crate) fn seal_node(
        &self,
        offset: u64,
        header: &DiskNodeHeader,
        payload: &[u8],
    ) -> Result<Vec<u8>> {
        let Some(crypto) = self.crypto.as_ref() else {
            return Err(Error::EncryptionKeyRequired {
                artifact: crate::storage::vfile_crypto::ARTIFACT.to_string(),
            });
        };
        let mut header = *header;
        // The payload lives inside the blob: `vector_offset` is the in-blob
        // offset, not a file address.
        header.vector_offset = ENCRYPTED_PAYLOAD_OFFSET;
        let mut plaintext = Vec::with_capacity(ENCRYPTED_PAYLOAD_OFFSET as usize + payload.len());
        plaintext.extend_from_slice(header.as_bytes());
        plaintext.extend_from_slice(payload);
        crypto.seal_node(offset, &plaintext)
    }

    /// Decrypt the node record at `offset`.
    ///
    /// The result is cached (a bounded LRU of decrypted nodes): the search
    /// path reads the header and then the vector of the **same** node, and
    /// the traversal visits the same nodes again on the next query — without
    /// the cache every candidate would be decrypted twice.
    ///
    /// The AEAD runs **outside** the cache lock: the critical section is a
    /// hash lookup, never the cipher.
    fn decrypt_blob(&self, offset: u64) -> Result<Option<Arc<AlignedBlob>>> {
        // Fast path: the node is already decrypted.
        if let Some(blob) = self.decrypted.lock().get(offset) {
            return Ok(Some(blob));
        }
        let Some(crypto) = self.crypto.as_ref() else {
            return Err(Error::EncryptionKeyRequired {
                artifact: crate::storage::vfile_crypto::ARTIFACT.to_string(),
            });
        };
        // The ciphertext is read **in place** out of the mapping: no copy.
        let record = self.read_record_bytes(offset)?;
        let plaintext = crypto.open_node(offset, record)?;
        // The AEAD writes the plaintext into a fresh buffer: 1 copy.
        count_copy(plaintext.len());
        // The plaintext is copied into the 8-byte aligned buffer: 1 copy.
        let blob = Arc::new(AlignedBlob::from_bytes(&plaintext));
        // The consumer keeps its own `Arc`: an eviction (or a write) of the
        // entry never invalidates the buffer the consumer is looking at.
        self.decrypted.lock().insert(offset, blob.clone());
        Ok(Some(blob))
    }

    /// The raw bytes of the encrypted record at `offset`:
    /// `[u32 blob_len][12B nonce][ct ‖ 16B tag]`.
    ///
    /// The bytes are **borrowed from the mapping**: the AEAD only reads them,
    /// so there is no reason to copy the ciphertext out first (FIND-598,
    /// Paso 0).
    fn read_record_bytes(&self, offset: u64) -> Result<&[u8]> {
        use crate::storage::vfile_crypto::{RECORD_PREFIX_LEN, TAG_LEN};
        let bytes = self.mmap_bytes();
        let start = offset as usize;
        if start + RECORD_PREFIX_LEN > bytes.len() {
            return Err(Error::EncryptionCorrupt {
                artifact: crate::storage::vfile_crypto::ARTIFACT.to_string(),
                detail: format!("node record at offset {offset} is truncated"),
            });
        }
        let blob_len = u32::from_le_bytes([
            bytes[start],
            bytes[start + 1],
            bytes[start + 2],
            bytes[start + 3],
        ]) as usize;
        if blob_len < TAG_LEN || start + RECORD_PREFIX_LEN + blob_len > bytes.len() {
            return Err(Error::EncryptionCorrupt {
                artifact: crate::storage::vfile_crypto::ARTIFACT.to_string(),
                detail: format!(
                    "node record at offset {offset} declares an impossible blob length {blob_len}"
                ),
            });
        }
        Ok(&bytes[start..start + RECORD_PREFIX_LEN + blob_len])
    }
}

/// The `DiskNodeHeader` of a decrypted node blob.
#[cfg(feature = "encryption")]
fn header_from_blob(blob: &AlignedBlob, offset: u64) -> Result<DiskNodeHeader> {
    let bytes = blob.as_slice();
    let header_size = std::mem::size_of::<DiskNodeHeader>();
    if bytes.len() < header_size {
        return Err(Error::EncryptionCorrupt {
            artifact: crate::storage::vfile_crypto::ARTIFACT.to_string(),
            detail: format!(
                "node record at offset {offset} decrypts to {} bytes, less than the \
                 {header_size}-byte header",
                bytes.len()
            ),
        });
    }
    DiskNodeHeader::read_from_bytes(&bytes[..header_size]).map_err(|e| Error::EncryptionCorrupt {
        artifact: crate::storage::vfile_crypto::ARTIFACT.to_string(),
        detail: format!("node record at offset {offset} has an invalid header: {e}"),
    })
}

impl crate::index_port::sealed::Sealed for File {}

/// `VectorStoreRef` bridge (F3X): the index consumes the store through the leaf
/// trait, so no `index → storage` item-edge escapes through search signatures.
impl crate::index_port::VectorStoreRef for File {
    fn read_header(&self, offset: u64) -> Option<DiskNodeHeader> {
        // Inherent method wins — deliberate delegation.
        self.read_header(offset)
    }

    fn mmap_bytes(&self) -> &[u8] {
        // Same: inherent `File::mmap_bytes`.
        self.mmap_bytes()
    }

    /// The FULL vector of the node at `offset`.
    ///
    /// A plaintext segment keeps the **zero-copy** view of the mapping; an
    /// encrypted segment materialises the payload (the ciphertext is not a
    /// `&[f32]` — this is the zero-copy tier that ADR-0057 §5.2 declares
    /// forfeited for encrypted stores).
    fn read_full_vector(
        &self,
        offset: u64,
        header: &DiskNodeHeader,
    ) -> Option<std::borrow::Cow<'_, [f32]>> {
        if !self.is_encrypted() {
            return crate::index_port::full_vector_from_bytes(self.mmap_bytes(), header);
        }
        #[cfg(feature = "encryption")]
        {
            let blob = self.decrypt_blob(offset).ok().flatten()?;
            let payload = blob.as_slice();
            let payload = payload.get(ENCRYPTED_PAYLOAD_OFFSET as usize..)?;
            let len = header.vector_len as usize;
            let bytes = payload.get(..len.checked_mul(4)?)?;
            // SAFETY: `align_to` is a transmute of the middle slice's
            // elements. Every bit pattern is a valid `f32`. The base of the
            // blob is 8-byte aligned (it lives in a `Vec<u64>`) and the
            // payload starts at `ENCRYPTED_PAYLOAD_OFFSET` (64), a multiple
            // of 4 ⇒ the slice is 4-byte aligned ⇒ the prefix and the suffix
            // are empty. Same pattern as the plaintext path
            // (`crate::index_port::full_vector_from_bytes`).
            let (_, words, _) = unsafe { bytes.align_to::<f32>() };
            if words.len() != len {
                return None;
            }
            // The words cannot be borrowed: the trait's lifetime is tied to
            // `&self` and the cache may evict the blob. They are
            // materialised — with a `memcpy`, not with the byte-by-byte loop
            // this path used to run (FIND-598, Paso 0).
            let out = words.to_vec();
            count_copy(out.len() * 4);
            Some(std::borrow::Cow::Owned(out))
        }
        #[cfg(not(feature = "encryption"))]
        {
            let _ = (offset, header);
            None
        }
    }
}

/// Advise the OS that a vector range is no longer needed (moved from
/// `index::graph::prefetch`, F3X: sole caller is storage-side consolidation).
#[inline(always)]
/// # Safety
///
/// `mmap_ptr` must point to a valid mmap region, and `offset + len` must be
/// within that region. The caller must ensure the mapping is not concurrently
/// unmapped or resized.
#[allow(unused_variables)]
pub unsafe fn release_mmap_vector(mmap_ptr: *const u8, offset: usize, len: usize) {
    #[cfg(unix)]
    {
        // SAFETY: caller guarantees `mmap_ptr` + `offset + len` is within a valid
        // mmap region. `madvise` with `MADV_DONTNEED` is async-signal-safe; the
        // mapping itself remains valid after the hint.
        unsafe {
            libc::madvise(
                mmap_ptr.add(offset) as *mut libc::c_void,
                len,
                libc::MADV_DONTNEED,
            );
        }
    }

    #[cfg(windows)]
    {
        let _ = (mmap_ptr, offset, len);
    }

    #[cfg(not(any(unix, windows)))]
    let _ = (mmap_ptr, offset, len);
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::binary_header::Header;
    use crate::node::DiskNodeHeader;
    use crate::storage::engine::STORAGE_ALIGNMENT;

    // ── In-Memory File ──

    #[test]
    fn test_vfile_create_in_memory() {
        let vf = File::create_in_memory(STORAGE_ALIGNMENT);
        assert!(vf.file.is_none());
        assert_eq!(vf.size, STORAGE_ALIGNMENT);
        assert_eq!(vf.write_cursor, STORAGE_ALIGNMENT);
        assert!(!vf.read_only);
        assert!(vf.path.as_os_str().is_empty());
        // Header should be valid
        let header = Header::deserialize(&vf.mmap_bytes()[0..16]).unwrap();
        assert_eq!(header.magic, *b"VFLE");
        // create_in_memory writes current VFILE_VERSION
        assert_eq!(header.format_version, VFILE_VERSION);
    }

    #[test]
    fn test_vfile_in_memory_larger_initial_size() {
        // When initial_size > STORAGE_ALIGNMENT, size should match
        let vf = File::create_in_memory(1024);
        assert!(vf.size >= 1024);
        assert_eq!(vf.write_cursor, STORAGE_ALIGNMENT);
    }

    #[test]
    fn test_vfile_in_memory_mmap_bytes() {
        let vf = File::create_in_memory(128);
        let bytes = vf.mmap_bytes();
        assert!(!bytes.is_empty());
        assert_eq!(bytes.len(), vf.size as usize);
        // Header area matches
        assert_eq!(&bytes[0..4], b"VFLE");
    }

    #[test]
    fn test_vfile_in_memory_mmap_bytes_mut() {
        let mut vf = File::create_in_memory(128);
        let bytes = vf.mmap_bytes_mut().unwrap();
        assert!(!bytes.is_empty());
        bytes[0] = b'X';
        // Verify the write was applied
        assert_eq!(vf.mmap_bytes()[0], b'X');
    }

    #[test]
    fn test_vfile_in_memory_save_cursor() {
        let mut vf = File::create_in_memory(256);
        vf.write_cursor = 192;
        vf.save_cursor().unwrap();
        // Cursor position is stored at bytes 16..24
        let stored = u64::from_le_bytes(vf.mmap_bytes()[16..24].try_into().unwrap());
        assert_eq!(stored, 192);
    }

    #[test]
    fn test_vfile_in_memory_flush() {
        let vf = File::create_in_memory(64);
        // In-memory flush is a no-op
        assert!(vf.flush().is_ok());
    }

    #[test]
    fn test_vfile_in_memory_resident_bytes() {
        let vf = File::create_in_memory(128);
        let bytes = vf.mmap_resident_bytes();
        // In-memory mode: always Some (all bytes are in process heap)
        assert!(bytes.is_some());
    }

    // ── File Growth (In-Memory) ──

    #[test]
    fn test_vfile_in_memory_grow() {
        let mut vf = File::create_in_memory(64);
        assert_eq!(vf.size, 64);

        vf.grow_to(256).unwrap();
        assert_eq!(vf.size, 256);
        // New region should be zero-filled
        assert_eq!(vf.mmap_bytes()[128], 0);
        assert_eq!(vf.mmap_bytes()[255], 0);
    }

    #[test]
    fn test_vfile_in_memory_grow_noop_equal_size() {
        let mut vf = File::create_in_memory(64);
        assert!(vf.grow_to(64).is_ok());
        assert_eq!(vf.size, 64);
    }

    #[test]
    fn test_vfile_grow_to_rejects_shrink() {
        let mut vf = File::create_in_memory(256);
        let err = vf.grow_to(128).unwrap_err();
        assert!(
            err.to_string().contains("grow_to"),
            "shrink should be rejected: {}",
            err
        );
    }

    // ── DiskNodeHeader read/write (In-Memory) ──

    #[test]
    fn test_vfile_in_memory_write_read_header() {
        let mut vf = File::create_in_memory(256);
        let header = DiskNodeHeader::new(42);
        // Write at an aligned offset past the header area
        let offset: u64 = STORAGE_ALIGNMENT;
        vf.write_header(offset, &header).unwrap();

        let read_back = vf.read_header(offset).expect("should read header");
        assert_eq!(read_back.id, 42);
        assert!((read_back.confidence_score - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vfile_read_header_misaligned_offset() {
        let vf = File::create_in_memory(256);
        // Offset not a multiple of STORAGE_ALIGNMENT ⇒ None
        assert!(vf.read_header(1).is_none());
        assert!(vf.read_header(STORAGE_ALIGNMENT + 1).is_none());
    }

    #[test]
    fn test_vfile_read_header_rejects_misaligned_vector_offset() {
        // INV-024 M-1 / AUDIT-03: a header whose vector payload offset is not a
        // multiple of 4 must be rejected centrally — otherwise the
        // `from_raw_parts(.. as *const f32)` cast at the 7 vector read sites
        // would produce a misaligned `&[f32]` (UB in release) on a corrupt or
        // adversarial file.
        let mut vf = File::create_in_memory(256);
        let mut header = DiskNodeHeader::new(1);
        header.vector_offset = 2; // NOT a multiple of 4 → corrupt payload pointer
        header.vector_len = 4;
        vf.write_header(STORAGE_ALIGNMENT, &header).unwrap();

        assert!(
            vf.read_header(STORAGE_ALIGNMENT).is_none(),
            "misaligned vector_offset must be rejected"
        );

        // Control: a multiple-of-4 offset is accepted.
        header.vector_offset = 4;
        vf.write_header(STORAGE_ALIGNMENT, &header).unwrap();
        assert!(vf.read_header(STORAGE_ALIGNMENT).is_some());
    }

    #[test]
    fn test_vfile_read_header_out_of_bounds() {
        let vf = File::create_in_memory(128);
        // Offset past end of file
        assert!(vf.read_header(200).is_none());
    }

    #[test]
    fn test_vfile_write_header_misaligned() {
        let mut vf = File::create_in_memory(256);
        let header = DiskNodeHeader::new(1);
        let err = vf.write_header(1, &header).unwrap_err();
        assert!(
            err.to_string().contains("misaligned"),
            "should reject misaligned: {}",
            err
        );
    }

    #[test]
    fn test_vfile_write_header_out_of_bounds() {
        // 128-byte file, DiskNodeHeader is 64 bytes, offset 128 is aligned but OOB
        let mut vf = File::create_in_memory(128);
        let header = DiskNodeHeader::new(1);
        let err = vf.write_header(128, &header).unwrap_err();
        assert!(
            err.to_string().contains("out of bounds"),
            "should reject OOB: {}",
            err
        );
    }

    #[test]
    fn test_vfile_write_header_multiple_offsets() {
        let mut vf = File::create_in_memory(256);
        let h1 = DiskNodeHeader::new(10);
        let h2 = DiskNodeHeader::new(20);
        vf.write_header(STORAGE_ALIGNMENT, &h1).unwrap();
        vf.write_header(STORAGE_ALIGNMENT * 2, &h2).unwrap();

        assert_eq!(vf.read_header(STORAGE_ALIGNMENT).unwrap().id, 10);
        assert_eq!(vf.read_header(STORAGE_ALIGNMENT * 2).unwrap().id, 20);
    }

    // ── File-Backed File ──

    #[test]
    fn test_vfile_create_and_open() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.vfle");

        // Create new file with 256-byte initial size
        let mut vf = File::open(path.clone(), 256).unwrap();
        assert!(vf.file.is_some());
        assert_eq!(vf.size, 256);
        assert!(!vf.read_only);
        assert_eq!(vf.write_cursor, STORAGE_ALIGNMENT);

        // Write a header and verify
        let header = DiskNodeHeader::new(99);
        vf.write_header(STORAGE_ALIGNMENT, &header).unwrap();
        vf.flush().unwrap();

        // Reopen and verify data persists
        let vf2 = File::open(path, 256).unwrap();
        let read = vf2.read_header(STORAGE_ALIGNMENT).unwrap();
        assert_eq!(read.id, 99);
    }

    #[test]
    fn test_vfile_open_reuses_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("existing.vfle");

        // Create file first
        let _ = File::open(path.clone(), 512).unwrap();

        // Re-open with different initial_size (should use existing file size)
        let vf = File::open(path, 0).unwrap();
        assert_eq!(vf.size, 512);
    }

    #[test]
    fn test_vfile_open_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro_test.vfle");

        // Create a file first
        let mut create = File::open(path.clone(), 128).unwrap();
        let header = DiskNodeHeader::new(7);
        create.write_header(STORAGE_ALIGNMENT, &header).unwrap();
        create.flush().unwrap();
        drop(create);

        // Open read-only
        let ro = File::open_read_only(path).unwrap();
        assert!(ro.read_only);
        assert_eq!(ro.read_header(STORAGE_ALIGNMENT).unwrap().id, 7);
    }

    #[test]
    fn test_vfile_read_only_write_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro_write.vfle");

        let create = File::open(path.clone(), 128).unwrap();
        create.flush().unwrap();
        drop(create);

        // mmap_bytes_mut requires &mut self — read_only file can only use mmap_bytes
        let _ = File::open_read_only(path).unwrap().mmap_bytes();
        // Confirm read_only can't get mutable access by design (compile-time check)
    }

    #[test]
    fn test_vfile_remap_mut_on_read_only_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro_remap.vfle");

        let create = File::open(path.clone(), 128).unwrap();
        create.flush().unwrap();
        drop(create);

        let mut ro = File::open_read_only(path).unwrap();
        let err = ro.remap_mut().unwrap_err();
        assert!(
            err.to_string().contains("read_only"),
            "remap_mut on read-only should fail: {}",
            err
        );
    }

    #[test]
    fn test_vfile_replace_backing_file_on_read_only_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro_replace.vfle");

        let create = File::open(path.clone(), 128).unwrap();
        create.flush().unwrap();
        drop(create);

        let mut ro = File::open_read_only(path).unwrap();
        let err = ro.replace_backing_file(256).unwrap_err();
        assert!(
            err.to_string().contains("read_only"),
            "replace on read-only should fail: {}",
            err
        );
    }

    #[test]
    fn test_vfile_grow_to_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("grow.vfle");

        let mut vf = File::open(path, 128).unwrap();
        assert_eq!(vf.size, 128);

        vf.grow_to(512).unwrap();
        assert_eq!(vf.size, 512);

        // Verify we can write at the new offset
        let header = DiskNodeHeader::new(200);
        vf.write_header(STORAGE_ALIGNMENT * 4, &header).unwrap();
        assert_eq!(vf.read_header(STORAGE_ALIGNMENT * 4).unwrap().id, 200);
    }

    #[test]
    fn test_vfile_open_nonexistent_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("_does_not_exist_.vfle");
        // Ensure it doesn't exist
        let _ = std::fs::remove_file(&path);
        let result = File::open_read_only(path);
        assert!(result.is_err(), "should fail for nonexistent file");
    }

    #[cfg(not(feature = "memmap2"))]
    #[test]
    fn shim_grow_to_preserves_pending_buffer_writes() {
        // AUD-044: grow_to/remap_mut replaces the mapping; in the no-memmap2
        // shim, pending buffer writes must be flushed first or they are
        // silently discarded (the file on disk still holds pre-grow content).
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("grow_preserve.vanta");
        let mut vf = File::open(path, 128).unwrap();
        let header = DiskNodeHeader::new(42);
        vf.write_header(STORAGE_ALIGNMENT, &header).unwrap();
        vf.grow_to(512).unwrap();
        let read = vf.read_header(STORAGE_ALIGNMENT).unwrap();
        assert_eq!(
            read.id, 42,
            "grow_to must preserve pending buffer writes (shim)"
        );
    }

    #[test]
    fn test_vfile_save_and_reload_cursor() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cursor_test.vfle");

        let mut vf = File::open(path.clone(), 256).unwrap();
        vf.write_cursor = 200;
        vf.save_cursor().unwrap();
        vf.flush().unwrap();
        drop(vf);

        // Reopen and verify cursor is restored (rounded up to alignment: (200+63)&!63 = 256)
        let vf2 = File::open(path, 256).unwrap();
        assert!(
            vf2.write_cursor == STORAGE_ALIGNMENT || vf2.write_cursor == 256,
            "cursor should be restored (200) or clamped (64), got {}",
            vf2.write_cursor
        );
    }

    // ── File Version ──

    #[test]
    fn test_vfile_version_constant() {
        assert_eq!(VFILE_VERSION, 2);
    }

    // ── File warmup_top_layers ──

    #[test]
    fn test_vfile_warmup_top_layers() {
        let vf = File::create_in_memory(256);
        // Should not panic
        vf.warmup_top_layers(128);
    }

    // ── engine_mmap_resident_bytes ──

    #[test]
    fn test_engine_mmap_resident_bytes_basic() {
        // In-memory vfile reports Some, in-memory index reports None → total is Some
        let index = crate::index::CPIndex::with_backend(crate::index::IndexBackend::InMemory);
        let vf = File::create_in_memory(128);
        let bytes = engine_mmap_resident_bytes(&index, &vf);
        assert!(bytes.is_some());
    }

    #[cfg(feature = "encryption")]
    #[test]
    fn test_vfile_encryption_stream_uses_attached_cipher() {
        // DUR-02: the stream must use the cipher attached via `with_cipher`,
        // not an ambient env-resolved one (`Cipher::from_env`). Regression for
        // the pre-fix behavior where `self.cipher` was ignored entirely.
        use std::io::{Read, Seek, SeekFrom, Write};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("enc_stream.vfle");
        let payload = b"dur02 encrypted payload";
        let cipher = Cipher::new(&[0x5Au8; 32]);
        let vf = File::open(path.clone(), 128)
            .unwrap()
            .with_cipher(cipher.clone());

        // Act: write through the stream obtained from the File.
        {
            let mut stream = vf
                .encryption_stream()
                .expect("an attached cipher must yield an encryption stream");
            stream.write_all(payload).unwrap();
            stream.flush().unwrap();
        }

        // Assert 1: the on-disk frame decrypts with the ATTACHED cipher.
        let raw = std::fs::read(&path).unwrap();
        let frame_len = u32::from_le_bytes(raw[0..4].try_into().unwrap()) as usize;
        let frame = &raw[4..4 + frame_len];
        let decrypted = cipher
            .decrypt(frame)
            .expect("attached cipher must decrypt the stream output");
        assert_eq!(decrypted, payload);

        // Assert 2: no plaintext on disk.
        assert!(
            !raw.windows(payload.len()).any(|w| w == payload),
            "encryption stream must not write plaintext"
        );

        // Assert 3: the read side round-trips through the same attached cipher.
        vf.file.as_ref().unwrap().seek(SeekFrom::Start(0)).unwrap();
        let mut reader = vf.encryption_stream().expect("stream for read");
        let mut out = vec![0u8; payload.len()];
        reader.read_exact(&mut out).unwrap();
        assert_eq!(&out, payload);
    }
}

// ─── FIND-598 (Paso 0): the encrypted read path ─────────────────────────

#[cfg(all(test, feature = "encryption"))]
#[allow(missing_docs)]
mod encryption_tests {
    use super::*;
    use crate::index::CPIndex;
    use crate::index_port::VectorStoreRef;
    use crate::node::{FilterBitset, NodeFlags, UnifiedNode, VectorRepresentations, ALL_BITSET};
    use crate::storage::ops::write_node_to_vstore;
    use crate::storage::vfile_crypto::VFileCrypto;

    /// A node with a FULL vector.
    fn node(id: u128, vector: Vec<f32>) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.vector = VectorRepresentations::Full(vector);
        node.flags.set(NodeFlags::HAS_VECTOR);
        node
    }

    /// The vectors of the search tests.
    ///
    /// Their cosines with the query `[1, 0, 0]` are **distinct** (1.0,
    /// ~0.994, 0.8, 0.0, -1.0), so the expected order of the results is
    /// unambiguous — no tie to break.
    fn vectors() -> Vec<Vec<f32>> {
        vec![
            vec![1.0, 0.0, 0.0],
            vec![0.9, 0.1, 0.0],
            vec![0.8, 0.6, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![-1.0, 0.0, 0.0],
        ]
    }

    /// A segment with the given vectors written to it, plus the index that
    /// points at them.
    ///
    /// `encrypted` picks the format: a keyed segment is v3 (encrypted), a
    /// plain one is v2 — the **same plaintext** in both cases.
    fn segment(
        dir: &std::path::Path,
        name: &str,
        vectors: &[Vec<f32>],
        encrypted: bool,
    ) -> (File, CPIndex, Vec<u64>) {
        let crypto = if encrypted {
            Some(Arc::new(
                VFileCrypto::open(dir, &[0x11; 32]).expect("the encryption registry"),
            ))
        } else {
            None
        };
        let path = dir.join(name);
        let mut file =
            File::open_with_crypto(path, 64 * 1024, false, crypto).expect("the segment opens");
        let index = CPIndex::new();
        let mut offsets = Vec::with_capacity(vectors.len());
        for (i, vector) in vectors.iter().enumerate() {
            let id = i as u128 + 1;
            let offset = write_node_to_vstore(&mut file, &node(id, vector.clone())).expect("write");
            index
                .add(
                    id,
                    FilterBitset::new(),
                    VectorRepresentations::Full(vector.clone()),
                    offset,
                )
                .expect("non-zero norm");
            offsets.push(offset);
        }
        (file, index, offsets)
    }

    /// The `f32` words of a decrypted node blob (the payload is at
    /// `ENCRYPTED_PAYLOAD_OFFSET`).
    fn words_of(blob: &AlignedBlob, len: usize) -> Vec<f32> {
        let bytes = &blob.as_slice()[ENCRYPTED_PAYLOAD_OFFSET as usize..];
        bytes[..len * 4]
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }

    /// The **pre-Paso 0** read path, replicated so the copies it did can be
    /// **measured** with the same probe.
    ///
    /// It is the original code, verbatim (`read_record_bytes` copying the
    /// ciphertext out of the mapping, the payload of `read_header` copied
    /// to reach 64 bytes of header, and the `f32` words rebuilt byte by
    /// byte). The Paso 0 removed two of the five copies; this helper is the
    /// "before" of the measurement the test pins.
    fn legacy_read(file: &File, offset: u64) -> Result<(DiskNodeHeader, Vec<f32>)> {
        use crate::storage::vfile_crypto::RECORD_PREFIX_LEN;
        // 1) the ciphertext is copied out of the mapping.
        let bytes = file.mmap_bytes();
        let start = offset as usize;
        let blob_len = u32::from_le_bytes([
            bytes[start],
            bytes[start + 1],
            bytes[start + 2],
            bytes[start + 3],
        ]) as usize;
        let record = bytes[start..start + RECORD_PREFIX_LEN + blob_len].to_vec();
        count_copy(record.len());
        // 2) the AEAD writes the plaintext into a fresh buffer.
        let crypto = file
            .crypto
            .as_ref()
            .ok_or_else(|| Error::EncryptionKeyRequired {
                artifact: crate::storage::vfile_crypto::ARTIFACT.to_string(),
            })?;
        let plaintext = crypto.open_node(offset, &record)?;
        count_copy(plaintext.len());
        // 3) the plaintext is copied into the aligned buffer.
        let blob = AlignedBlob::from_bytes(&plaintext);
        let header = header_from_blob(&blob, offset)?;
        // 4) the payload is copied to reach the 64 bytes of header.
        let header_size = std::mem::size_of::<DiskNodeHeader>();
        let payload = blob.as_slice()[header_size..].to_vec();
        count_copy(payload.len());
        // 5) the `f32` words are rebuilt byte by byte.
        let mut out = Vec::with_capacity(header.vector_len as usize);
        for chunk in payload.chunks_exact(4) {
            out.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        count_copy(out.len() * 4);
        Ok((header, out))
    }

    /// FIND-598 (Paso 0): the fix must not change **what** a search returns.
    ///
    /// The encrypted segment returns the same ids, in the same order, with
    /// the same scores as the plaintext one — and the same before and after
    /// the cache is warm (the cache must not change the result either).
    #[test]
    fn encrypted_search_returns_the_same_ids_order_and_scores() {
        let dir = tempfile::tempdir().unwrap();
        let vectors = vectors();
        let query = vec![1.0, 0.0, 0.0];

        // The reference: the same content in a plaintext segment.
        let (plain, plain_index, _) = segment(dir.path(), "plain.vanta", &vectors, false);
        let expected = plain_index.search_nearest(&query, None, None, &ALL_BITSET, 3, Some(&plain));

        // The same content, encrypted.
        let (enc, enc_index, _) = segment(dir.path(), "enc.vanta", &vectors, true);
        // Cold: every candidate is decrypted on the spot.
        let cold = enc_index.search_nearest(&query, None, None, &ALL_BITSET, 3, Some(&enc));
        // Warm: the same nodes come from the cache.
        let warm = enc_index.search_nearest(&query, None, None, &ALL_BITSET, 3, Some(&enc));

        assert_eq!(expected.len(), 3, "the search returns 3 neighbours");
        // The ids, in the order: the exact match first, then the closest ones.
        let ids: Vec<u128> = expected.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec![1, 2, 3], "the same ids, in the same order");
        // The scores: the exact match scores 1.0 and the rest decrease.
        assert!(
            (expected[0].1 - 1.0).abs() < 1e-6,
            "the exact match scores 1.0"
        );
        assert!(
            expected[0].1 > expected[1].1 && expected[1].1 > expected[2].1,
            "the scores are ordered: {expected:?}"
        );

        // The encrypted path returns the same thing.
        assert_eq!(cold.len(), expected.len());
        for (plain_hit, enc_hit) in expected.iter().zip(cold.iter()) {
            assert_eq!(plain_hit.0, enc_hit.0, "the same id");
            assert!(
                (plain_hit.1 - enc_hit.1).abs() < 1e-6,
                "the same score: {} vs {}",
                plain_hit.1,
                enc_hit.1
            );
        }
        // And the cache does not change it.
        assert_eq!(cold, warm, "the cache must not change the result");
    }

    /// FIND-598 (Paso 0): the cache must never hand out a buffer that dies
    /// while the consumer is looking at it.
    ///
    /// The consumer owns an `Arc` of the decrypted blob: when the cache
    /// replaces the entry (a write invalidates it) the consumer's buffer is
    /// still alive and unchanged — and the cache serves the **new**
    /// plaintext, not the stale one.
    #[test]
    fn decrypt_cache_never_hands_out_a_dangling_buffer() {
        let dir = tempfile::tempdir().unwrap();
        let (mut file, _index, offsets) = segment(dir.path(), "enc.vanta", &vectors(), true);
        let offset = offsets[0];

        // The consumer takes the decrypted blob (its own `Arc`).
        let held = file
            .decrypt_blob(offset)
            .expect("the node decrypts")
            .expect("the node is there");
        assert_eq!(words_of(&held, 3), vec![1.0, 0.0, 0.0]);

        // The node is rewritten with a different vector of the **same
        // length** (the record keeps its length: the offsets of the
        // following nodes do not move).
        let header = file.read_header(offset).expect("header");
        let new_payload: Vec<u8> = [-1.0f32, 0.0, 0.0]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        file.write_node(offset, &header, &new_payload)
            .expect("the node is rewritten");

        // The cache was invalidated: it serves the NEW plaintext.
        let fresh = file
            .decrypt_blob(offset)
            .expect("the node decrypts")
            .expect("the node is there");
        assert_eq!(
            words_of(&fresh, 3),
            vec![-1.0, 0.0, 0.0],
            "the cache must not serve the stale plaintext"
        );

        // The buffer the consumer was holding is still alive and unchanged.
        assert_eq!(
            words_of(&held, 3),
            vec![1.0, 0.0, 0.0],
            "the consumer's buffer must stay valid after the cache replaced the entry"
        );
    }

    /// FIND-598 (Paso 0): the number of copies of the node payload in the
    /// encrypted read path is **pinned**, before and after.
    ///
    /// It was **5** before the fix (the ciphertext out of the mapping, the
    /// AEAD output, the aligned blob, the payload of `read_header` and the
    /// `f32` words). It is **3** now: the ciphertext is read in place, the
    /// header is parsed in place, and the `f32` words are a `memcpy`.
    ///
    /// Both numbers are **measured with the same probe**: the "before" is
    /// the original code, replicated verbatim in `legacy_read`.
    #[test]
    fn encrypted_read_path_copies_are_pinned() {
        let dir = tempfile::tempdir().unwrap();
        let (file, _index, offsets) = segment(dir.path(), "enc.vanta", &vectors(), true);
        let offset = offsets[0];
        // 64 bytes of header + 12 bytes of payload.
        const PLAINTEXT: u64 = 76;
        const WORDS: u64 = 12;

        // ── Before: the pre-Paso 0 path ──
        copy_probe::reset();
        let (legacy_header, legacy_vector) = legacy_read(&file, offset).expect("the legacy read");
        assert_eq!(legacy_header.vector_len, 3);
        assert_eq!(legacy_vector, vec![1.0, 0.0, 0.0]);
        let (copies, bytes) = copy_probe::snapshot();
        assert_eq!(
            copies, 5,
            "the pre-Paso 0 path copied the payload 5 times (ciphertext, AEAD output, \
             aligned blob, read_header payload, f32 words)"
        );
        assert_eq!(bytes, (PLAINTEXT + 32) + PLAINTEXT * 2 + WORDS * 2);

        // ── After: cold (the node is not cached) ──
        file.decrypted.lock().clear();
        copy_probe::reset();
        let header = file.read_header(offset).expect("header");
        let vector = file
            .read_full_vector(offset, &header)
            .expect("the vector is readable");
        assert_eq!(vector.len(), 3);
        let (copies, bytes) = copy_probe::snapshot();
        assert_eq!(
            copies, 3,
            "the encrypted read path copies the payload 3 times (AEAD output, \
             aligned blob, f32 words)"
        );
        assert_eq!(bytes, PLAINTEXT * 2 + WORDS);

        // ── After: warm (the node comes from the cache) ──
        copy_probe::reset();
        let header = file.read_header(offset).expect("header");
        let _ = file
            .read_full_vector(offset, &header)
            .expect("the vector is readable");
        let (copies, bytes) = copy_probe::snapshot();
        assert_eq!(copies, 1, "a cache hit copies only the `f32` words");
        assert_eq!(bytes, WORDS);
    }
}
