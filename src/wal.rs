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

        // Range-based compatibility check: accepts any format_version ≤ WAL_FORMAT_VERSION
        // with matching magic. Future-format files (version > current) are rejected;
        // older files with matching magic are accepted for forward-compatible reads.
        base.validate_compat(*b"VWAL", WAL_FORMAT_VERSION, "WAL format")?;

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
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false) // ← Explicit for Clippy: preserve existing WAL data for recovery
            .open(&path)?;

        let file_len = file.metadata()?.len();
        let (bytes_written, record_count, format_version, chain_head) = if file_len == 0 {
            let header = WalHeader::new(WAL_FORMAT_VERSION as u32);
            file.write_all(&header.serialize())?;
            file.flush()?;
            // POSIX: fsync on the file does NOT make its directory entry durable
            // ("Calling fsync() does not necessarily ensure that the entry in the
            // directory containing the file has also reached disk" — man fsync(2)).
            // Without this, a freshly created WAL whose appends were already
            // sync_data'd can vanish on power loss, losing acknowledged writes.
            // Same pattern as rotate() below and archive.rs (AUDREP-35).
            crate::utils::fs::sync_parent_dir(&path)?;
            (
                WalHeader::SIZE as u64,
                0u64,
                WAL_FORMAT_VERSION,
                CHAIN_GENESIS,
            )
        } else {
            // Read the existing header
            let mut header_bytes = [0u8; WalHeader::SIZE];
            file.seek(SeekFrom::Start(0))?;
            file.read_exact(&mut header_bytes)?;
            let header = WalHeader::deserialize(&header_bytes)?;
            let version = header.base.format_version;

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

        Self::open(&old_path, sync_mode)
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

        let header = WalHeader::new(WAL_FORMAT_VERSION as u32);
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
        // Fresh segment → fresh chain (VER-01): v3 framing from genesis.
        self.format_version = WAL_FORMAT_VERSION;
        self.chain_head = CHAIN_GENESIS;

        Ok(true)
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

// ─── WAL Reader ────────────────────────────────────────────

/// Sequential WAL reader for crash recovery.
pub struct WalReader {
    reader: BufReader<File>,
    records_read: u64,
    /// Per-frame chain overhead for this file's format version
    /// ([`frame_extra_for`]): 0 for legacy v1/v2, 64 for chained v3+.
    frame_extra: usize,
}

impl WalReader {
    /// Open a WAL file and validate its header.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
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

        Ok(Self {
            reader: BufReader::with_capacity(64 * KIB, file),
            records_read: 0,
            frame_extra: frame_extra_for(header.base.format_version),
        })
    }

    /// Read next record with Scan-Forward Auto-healing. Returns None at EOF.
    pub fn next_record(&mut self) -> Result<Option<WalRecord>> {
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
                    // WAL v2 (RES-01): expected_version tracks the current format
                    // version. The test file in this fixture uses v0, so the
                    // reader should report its CURRENT version (v2) as the
                    // expected version when rejecting.
                    assert_eq!(expected_version, WAL_FORMAT_VERSION);
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
}
