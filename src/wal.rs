use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tracing::warn;

use serde::{Deserialize, Serialize};

use crate::error::{Result, VantaError};
use crate::node::UnifiedNode;

/// Tracks the postcard wire format version used for WAL record serialization.
/// Increment this when upgrading postcard to a potentially incompatible version.
/// Stored in VantaHeader.schema_version for forward-compatibility detection.
pub(crate) const WAL_POSTCARD_VERSION: u16 = 1;

const KIB: usize = 1024;
use crc32c::crc32c; // ← Import specific function to avoid namespace conflict

/// CRC32C (Castagnoli) using hardware-accelerated crate for performance
/// Falls back to pure Rust implementation if hardware acceleration unavailable
#[inline]
pub fn compute_crc32c(data: &[u8]) -> u32 {
    crc32c::crc32c(data)
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
}

// ─── WAL Header ────────────────────────────────────────────

/// WAL file header with magic bytes, version, schema version, and CRC integrity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalHeader {
    /// 16-byte VantaHeader (magic = `b"VWAL"`, version = 1, schema = 0, timestamp).
    pub base: crate::binary_header::VantaHeader,
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
        let base = crate::binary_header::VantaHeader::new(
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
            return Err(VantaError::WalError(format!(
                "Invalid WAL header size: expected {}, got {}",
                Self::SIZE,
                bytes.len()
            )));
        }

        let base = crate::binary_header::VantaHeader::deserialize(&bytes[0..16])?;
        if &base.magic != b"VWAL" {
            return Err(VantaError::IncompatibleFormat {
                expected_magic: *b"VWAL",
                expected_version: 1,
                found_magic: base.magic,
                found_version: base.format_version,
                hint: "Delete WAL dir or run dump/restore before upgrading.".to_string(),
            });
        }

        let crc = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);

        let header = Self { base, crc };

        let computed_crc = header.compute_crc();
        if computed_crc != crc {
            return Err(VantaError::WalError(format!(
                "WAL header CRC mismatch: stored={:#x}, computed={:#x}",
                crc, computed_crc
            )));
        }

        if header.base.format_version < 1 {
            return Err(VantaError::WALVersionMismatch {
                expected: 1,
                found: header.base.format_version as u32,
                hint: "Delete WAL dir or run dump/restore before upgrading.".to_string(),
            });
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
    flush_threshold: Option<usize>,
}

impl WalWriter {
    /// Open or create WAL file, writing or validating WalHeader.
    pub fn open(
        path: impl AsRef<Path>,
        sync_mode: crate::config::SyncMode,
    ) -> Result<Self> {
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
        let bytes_written;
        let mut record_count = 0u64;

        if file_len == 0 {
            let header = WalHeader::new(1);
            file.write_all(&header.serialize())?;
            file.flush()?;
            bytes_written = WalHeader::SIZE as u64;
        } else {
            // Read the existing header
            let mut header_bytes = [0u8; WalHeader::SIZE];
            file.seek(SeekFrom::Start(0))?;
            file.read_exact(&mut header_bytes)?;
            let _header = WalHeader::deserialize(&header_bytes)?;

            // Scan to count valid records and detect tail or mid-file corruption (Scan-Forward Auto-healing)
            let mut valid_bytes_limit = WalHeader::SIZE as u64;
            {
                let mut file_handle = File::open(&path)?;
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

                    let mut is_valid = false;
                    // FMEA-03: Avoid OOM by limiting max corruption-scan size to 10MB
                    if len > 0 && len <= 10_000_000 && current_offset + 4 + len + 4 <= file_len {
                        let mut record_bytes = vec![0u8; len as usize + 4];
                        if file_handle.read_exact(&mut record_bytes).is_ok() {
                            let payload = &record_bytes[0..len as usize];
                            let crc_bytes: [u8; 4] = record_bytes[len as usize..len as usize + 4]
                                .try_into()
                                .map_err(|_| {
                                    VantaError::WalError(
                                        "CRC bytes slice expected 4 bytes in WAL record"
                                            .to_string(),
                                    )
                                })?;
                            let stored_crc = u32::from_le_bytes(crc_bytes);
                            let computed_crc = crc32c(payload);

                            if stored_crc == computed_crc {
                                // FMEA-02: Validate structural deserialization to avoid accidental CRC collisions
                                if postcard::from_bytes::<WalRecord>(payload).is_ok() {
                                    is_valid = true;
                                }
                            }
                        }
                    }

                    if is_valid {
                        record_count += 1;
                        current_offset += 4 + len + 4;
                        valid_bytes_limit = current_offset;
                    } else {
                        // Entering Scan-Forward mode (scan forward byte by byte)
                        warn!(
                            path = %path.display(),
                            offset = current_offset,
                            "Corrupt record detected in WAL. Entering Scan-Forward mode to locate next valid transaction..."
                        );

                        let mut found_next = false;
                        let mut scan_pos = current_offset + 1;

                        while scan_pos + 8 <= file_len {
                            if file_handle.seek(SeekFrom::Start(scan_pos)).is_err() {
                                break;
                            }
                            let mut test_len_buf = [0u8; 4];
                            if file_handle.read_exact(&mut test_len_buf).is_ok() {
                                let test_len = u32::from_le_bytes(test_len_buf) as u64;
                                if test_len > 0
                                    && test_len <= 10_000_000
                                    && scan_pos + 4 + test_len + 4 <= file_len
                                {
                                    let mut test_bytes = vec![0u8; test_len as usize + 4];
                                    if file_handle.read_exact(&mut test_bytes).is_ok() {
                                        let payload = &test_bytes[0..test_len as usize];
                                        let crc_bytes: [u8; 4] = test_bytes
                                            [test_len as usize..test_len as usize + 4]
                                            .try_into()
                                            .map_err(|_| {
                                                VantaError::WalError(
                                                    "CRC bytes slice expected 4 bytes in WAL scan-forward"
                                                        .to_string(),
                                                )
                                            })?;
                                        let stored_crc = u32::from_le_bytes(crc_bytes);
                                        let computed_crc = crc32c(payload);

                                        if stored_crc == computed_crc
                                            && postcard::from_bytes::<WalRecord>(payload).is_ok()
                                        {
                                            warn!(
                                                path = %path.display(),
                                                skipped_corrupt_bytes = scan_pos - current_offset,
                                                recovered_offset = scan_pos,
                                                "Successfully bypassed corrupt segment and recovered next transaction."
                                            );
                                            current_offset = scan_pos;
                                            found_next = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            scan_pos += 1;
                        }

                        if !found_next {
                            // No more valid records in the file. Corruption is tail/truncated.
                            break;
                        }
                    }
                }
            }

            if file_len > valid_bytes_limit {
                warn!(
                    path = %path.display(),
                    expected_len = file_len,
                    valid_len = valid_bytes_limit,
                    "Truncating corrupt or incomplete records at the end of WAL"
                );
                file.set_len(valid_bytes_limit)?;
            }

            bytes_written = valid_bytes_limit;
            file.seek(SeekFrom::Start(bytes_written))?;
        }

        let buffer_size = buffer_size.max(KIB).min(32 * 1024 * KIB);

        Ok(Self {
            writer: BufWriter::with_capacity(buffer_size, file),
            path,
            bytes_written,
            record_count,
            sync_mode,
            records_since_sync: 0,
            flush_threshold,
        })
    }

    /// Append a single record to the WAL
    pub fn append(&mut self, record: &WalRecord) -> Result<()> {
        #[cfg(feature = "failpoints")]
        fail::fail_point!("wal_append_fail", |_| {
            Err(VantaError::IoError(std::io::Error::other(
                "Simulated WAL append catastrophic I/O failure",
            )))
        });

        let payload = postcard::to_allocvec(record)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        let len = payload.len() as u32;
        let crc = crc32c(&payload);

        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(&payload)?;
        self.writer.write_all(&crc.to_le_bytes())?;

        self.bytes_written += 4 + payload.len() as u64 + 4;
        self.record_count += 1;
        self.records_since_sync += 1;

        if self.sync_mode == crate::config::SyncMode::Always {
            self.sync()?;
        } else if let Some(threshold) = self.flush_threshold {
            if self.records_since_sync >= threshold as u64 {
                self.sync()?;
            }
        }
        Ok(())
    }

    /// Append multiple records in a single write call to reduce I/O overhead.
    pub fn batch_append(&mut self, records: &[WalRecord]) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        #[cfg(feature = "failpoints")]
        fail::fail_point!("wal_append_fail", |_| {
            Err(VantaError::IoError(std::io::Error::other(
                "Simulated WAL append catastrophic I/O failure",
            )))
        });

        let estimated = records.len() * 128;
        let mut buf = Vec::with_capacity(estimated);

        for record in records {
            let payload = postcard::to_allocvec(record)
                .map_err(|e| VantaError::SerializationError(e.to_string()))?;
            let len = payload.len() as u32;
            let crc = crc32c(&payload);
            buf.extend_from_slice(&len.to_le_bytes());
            buf.extend_from_slice(&payload);
            buf.extend_from_slice(&crc.to_le_bytes());
        }

        self.writer.write_all(&buf)?;
        self.bytes_written += buf.len() as u64;
        self.record_count += records.len() as u64;
        self.records_since_sync += records.len() as u64;

        if self.sync_mode == crate::config::SyncMode::Always {
            self.sync()?;
        } else if let Some(threshold) = self.flush_threshold {
            if self.records_since_sync >= threshold as u64 {
                self.sync()?;
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

        Self::open(&old_path, sync_mode)
    }
}

// ─── WAL Reader ────────────────────────────────────────────

/// Sequential WAL reader for crash recovery.
pub struct WalReader {
    reader: BufReader<File>,
    records_read: u64,
}

impl WalReader {
    /// Open a WAL file and validate its header.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path)?;
        let file_len = file.metadata()?.len();

        if file_len < WalHeader::SIZE as u64 {
            return Err(VantaError::WalError(
                "WAL file is truncated or too small for header".to_string(),
            ));
        }

        // Read and validate the header
        let mut header_bytes = [0u8; WalHeader::SIZE];
        file.read_exact(&mut header_bytes)?;
        let _header = WalHeader::deserialize(&header_bytes)?;

        Ok(Self {
            reader: BufReader::with_capacity(64 * KIB, file),
            records_read: 0,
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
            if len > 0 && len <= 10_000_000 && current_pos + 4 + len + 4 <= file_len {
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
                            is_valid = true;
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
                let record: WalRecord = postcard::from_bytes(&payload)
                    .map_err(|e| VantaError::SerializationError(e.to_string()))?;
                self.records_read += 1;
                return Ok(Some(record));
            } else {
                // Entering Scan-Forward mode to skip corruption and find the next consistent block
                warn!(
                    offset = current_pos,
                    "WalReader detected corrupt record. Scanning forward to recover next valid transaction..."
                );

                let mut scan_pos = current_pos + 1;
                let mut found_next = false;

                while scan_pos + 8 <= file_len {
                    if self.reader.seek(SeekFrom::Start(scan_pos)).is_ok() {
                        let mut test_len_buf = [0u8; 4];
                        if self.reader.read_exact(&mut test_len_buf).is_ok() {
                            let test_len = u32::from_le_bytes(test_len_buf) as u64;
                            if test_len > 0
                                && test_len <= 10_000_000
                                && scan_pos + 4 + test_len + 4 <= file_len
                            {
                                let mut test_payload = vec![0u8; test_len as usize];
                                if self.reader.read_exact(&mut test_payload).is_ok() {
                                    let mut test_crc_buf = [0u8; 4];
                                    if self.reader.read_exact(&mut test_crc_buf).is_ok() {
                                        let stored_crc = u32::from_le_bytes(test_crc_buf);
                                        let computed_crc = crc32c(&test_payload);
                                        if stored_crc == computed_crc
                                            && postcard::from_bytes::<WalRecord>(&test_payload)
                                                .is_ok()
                                        {
                                            warn!(
                                                corrupt_bytes_skipped = scan_pos - current_pos,
                                                recovered_offset = scan_pos,
                                                "WalReader successfully bypassed corrupt bytes and resumed recovery."
                                            );
                                            self.reader.seek(SeekFrom::Start(scan_pos))?;
                                            found_next = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    scan_pos += 1;
                }

                if !found_next {
                    // No more valid records in the entire file, actual end of stream
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
                return Err(VantaError::WalError(format!(
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
            // Escribir un WAL sin firma válida (versión 0 o archivo genérico)
            let mut file = File::create(&dir).unwrap();
            file.write_all(b"NOT_A_VALID_MAGIC_BYTES_123456").unwrap();
        }

        {
            // Intentar abrir el WAL debe lanzar error IncompatibleFormat
            let r = WalReader::open(&dir);
            assert!(r.is_err());
            match r.err().unwrap() {
                VantaError::IncompatibleFormat {
                    expected_magic,
                    expected_version,
                    ..
                } => {
                    assert_eq!(expected_magic, *b"VWAL");
                    assert_eq!(expected_version, 1);
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

        // 1. Escribir 3 registros válidos + checkpoint
        {
            let mut w = WalWriter::open(&dir, crate::config::SyncMode::Periodic).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(2))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(3))).unwrap();
            w.append(&WalRecord::create_checkpoint(3, None)).unwrap();
            w.sync().unwrap();
            assert_eq!(w.record_count(), 4);
        }

        // 2. Corromper el WAL agregando basura trunca al final
        {
            let mut file = OpenOptions::new().append(true).open(&dir).unwrap();
            file.write_all(
                b"\x0a\x00\x00\x00truncated garbage here that fails CRC or is cut off mid-way",
            )
            .unwrap();
        }

        // 3. Abrir el WAL de nuevo con WalWriter
        {
            let mut w = WalWriter::open(&dir, crate::config::SyncMode::Periodic).unwrap();
            // Debe haber truncado la basura y cargado la cantidad correcta de registros (4)
            assert_eq!(w.record_count(), 4);

            // Intentar escribir un nuevo registro
            w.append(&WalRecord::Insert(UnifiedNode::new(4))).unwrap();
            w.sync().unwrap();
            assert_eq!(w.record_count(), 5);
        }

        // 4. Leer con WalReader y verificar integridad
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
}
