use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tracing::warn;

use serde::{Deserialize, Serialize};

use crate::error::{Result, VantaError};
use crate::node::UnifiedNode;
use crc32c::crc32c;  // ← Importar función específica para evitar conflicto de namespace

/// CRC32C (Castagnoli) using hardware-accelerated crate for performance
/// Falls back to pure Rust implementation if hardware acceleration unavailable
#[inline]
pub fn compute_crc32c(data: &[u8]) -> u32 {
    crc32c::crc32c(data)
}

// ─── WAL Record ────────────────────────────────────────────

/// WAL record types (bincode-serialized)
#[derive(Serialize, Deserialize, Debug)]
pub enum WalRecord {
    Insert(UnifiedNode),
    Update { id: u64, node: UnifiedNode },
    Delete { id: u64 },
    /// Checkpoint with optional index checksum for integrity validation
    /// `index_checksum` is computed over serialized index state; None for backward compat
    /// `timestamp` allows ordering checkpoints for recovery decisions
    Checkpoint {
        node_count: u64,
        index_checksum: Option<u32>,
        timestamp: Option<u64>,
    },
}

// ─── WAL Header ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalHeader {
    pub magic: [u8; 8],     // b"VANTAWAL"
    pub version: u32,       // >= 1
    pub flags: u32,         // 0
    pub crc: u32,           // CRC32C Castagnoli de los primeros 16 bytes
}

impl WalHeader {
    pub const SIZE: usize = 20;

    pub fn new(version: u32) -> Self {
        let magic = *b"VANTAWAL";
        let flags = 0u32;
        let mut header = Self {
            magic,
            version,
            flags,
            crc: 0,
        };
        header.crc = header.compute_crc();
        header
    }

    pub fn compute_crc(&self) -> u32 {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&self.magic);
        bytes[8..12].copy_from_slice(&self.version.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.flags.to_le_bytes());
        crc32c(&bytes)
    }

    pub fn serialize(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..8].copy_from_slice(&self.magic);
        bytes[8..12].copy_from_slice(&self.version.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.flags.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.crc.to_le_bytes());
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::SIZE {
            return Err(VantaError::WalError(format!(
                "Invalid WAL header size: expected {}, got {}",
                Self::SIZE,
                bytes.len()
            )));
        }
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        if &magic != b"VANTAWAL" {
            return Err(VantaError::WALVersionMismatch {
                expected: 1,
                found: 0,
                hint: "Delete WAL dir or run dump/restore before upgrading.".to_string(),
            });
        }
        let version = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let flags = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let crc = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);

        let header = Self {
            magic,
            version,
            flags,
            crc,
        };

        let computed_crc = header.compute_crc();
        if computed_crc != crc {
            return Err(VantaError::WalError(format!(
                "WAL header CRC mismatch: stored={:#x}, computed={:#x}",
                crc, computed_crc
            )));
        }

        if version < 1 {
            return Err(VantaError::WALVersionMismatch {
                expected: 1,
                found: version,
                hint: "Delete WAL dir or run dump/restore before upgrading.".to_string(),
            });
        }

        Ok(header)
    }
}

// ─── WAL Writer ────────────────────────────────────────────

/// Append-only WAL writer with CRC32C integrity checks and structured header.
///
/// File format: [WalHeader(20 bytes)][Record1][Record2]...
/// Record format: [len:u32][payload:bincode][crc:u32]
pub struct WalWriter {
    writer: BufWriter<File>,
    path: PathBuf,
    bytes_written: u64,
    record_count: u64,
}

impl WalWriter {
    /// Open or create WAL file, writing or validating WalHeader.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)  // ← Explicit for Clippy: preserve existing WAL data for recovery
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
            // Leer el header existente
            let mut header_bytes = [0u8; WalHeader::SIZE];
            file.seek(SeekFrom::Start(0))?;
            file.read_exact(&mut header_bytes)?;
            let _header = WalHeader::deserialize(&header_bytes)?;

            // Escanear para contar registros válidos y detectar corrupción final (truncar si es necesario)
            let mut valid_bytes_limit = WalHeader::SIZE as u64;
            {
                let mut reader = BufReader::new(File::open(&path)?);
                // Saltamos el header
                let mut tmp_header = [0u8; WalHeader::SIZE];
                let _ = reader.read_exact(&mut tmp_header);

                let mut current_offset = WalHeader::SIZE as u64;
                loop {
                    let mut len_buf = [0u8; 4];
                    match reader.read_exact(&mut len_buf) {
                        Ok(_) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                        Err(_) => break,
                    }
                    let len = u32::from_le_bytes(len_buf) as u64;

                    let mut record_bytes = vec![0u8; len as usize + 4];
                    match reader.read_exact(&mut record_bytes) {
                        Ok(_) => {
                            let payload = &record_bytes[0..len as usize];
                            let crc_bytes: [u8; 4] = record_bytes[len as usize..len as usize + 4].try_into().unwrap();
                            let stored_crc = u32::from_le_bytes(crc_bytes);
                            let computed_crc = crc32c(payload);
                            if stored_crc == computed_crc {
                                record_count += 1;
                                current_offset += 4 + len + 4;
                                valid_bytes_limit = current_offset;
                            } else {
                                break;
                            }
                        }
                        Err(_) => {
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

        Ok(Self {
            writer: BufWriter::with_capacity(64 * 1024, file),
            path,
            bytes_written,
            record_count,
        })
    }

    /// Append a record to the WAL
    pub fn append(&mut self, record: &WalRecord) -> Result<()> {
        let payload = bincode::serialize(record)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        let len = payload.len() as u32;
        let crc = crc32c(&payload);

        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(&payload)?;
        self.writer.write_all(&crc.to_le_bytes())?;

        self.bytes_written += 4 + payload.len() as u64 + 4;
        self.record_count += 1;
        Ok(())
    }

    /// Flush buffer and fsync to disk
    pub fn sync(&mut self) -> Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;
        Ok(())
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
    pub fn record_count(&self) -> u64 {
        self.record_count
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}

// ─── WAL Reader ────────────────────────────────────────────

/// Sequential WAL reader for crash recovery
pub struct WalReader {
    reader: BufReader<File>,
    records_read: u64,
}

impl WalReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path)?;
        let file_len = file.metadata()?.len();

        if file_len < WalHeader::SIZE as u64 {
            return Err(VantaError::WalError("WAL file is truncated or too small for header".to_string()));
        }

        // Leer y validar el header
        let mut header_bytes = [0u8; WalHeader::SIZE];
        file.read_exact(&mut header_bytes)?;
        let _header = WalHeader::deserialize(&header_bytes)?;

        Ok(Self {
            reader: BufReader::with_capacity(64 * 1024, file),
            records_read: 0,
        })
    }

    /// Read next record. Returns None at EOF.
    pub fn next_record(&mut self) -> Result<Option<WalRecord>> {
        // Read length prefix
        let mut len_buf = [0u8; 4];
        match self.reader.read_exact(&mut len_buf) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }
        let len = u32::from_le_bytes(len_buf) as usize;

        // Read payload
        let mut payload = vec![0u8; len];
        self.reader.read_exact(&mut payload)?;

        // Read and verify CRC
        let mut crc_buf = [0u8; 4];
        self.reader.read_exact(&mut crc_buf)?;
        let stored_crc = u32::from_le_bytes(crc_buf);
        let computed_crc = crc32c(&payload);

        if stored_crc != computed_crc {
            return Err(VantaError::WalError(format!(
                "CRC mismatch at record {}: stored={:#x}, computed={:#x}",
                self.records_read, stored_crc, computed_crc
            )));
        }

        let record: WalRecord = bincode::deserialize(&payload)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.records_read += 1;
        Ok(Some(record))
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
        let timestamp = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64);
        
        WalRecord::Checkpoint {
            node_count,
            index_checksum,
            timestamp,
        }
    }

    /// Validate checkpoint integrity if checksum is present
    pub fn validate_checkpoint(&self, index_state: &[u8]) -> Result<()> {
        if let WalRecord::Checkpoint { index_checksum: Some(expected), .. } = self {
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
mod tests {
    use super::*;
    use crate::node::UnifiedNode;

    #[test]
    fn test_wal_roundtrip() {
        let dir = std::env::temp_dir().join("vanta_test_wal_rt");
        let _ = std::fs::remove_file(&dir);

        {
            let mut w = WalWriter::open(&dir).unwrap();
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
        let dir = std::env::temp_dir().join("vanta_test_wal_mismatch");
        let _ = std::fs::remove_file(&dir);

        {
            // Escribir un WAL sin firma válida (versión 0 o archivo genérico)
            let mut file = File::create(&dir).unwrap();
            file.write_all(b"NOT_A_VALID_MAGIC_BYTES_123456").unwrap();
        }

        {
            // Intentar abrir el WAL debe lanzar error WALVersionMismatch
            let r = WalReader::open(&dir);
            assert!(r.is_err());
            match r.err().unwrap() {
                VantaError::WALVersionMismatch { expected, found, .. } => {
                    assert_eq!(expected, 1);
                    assert_eq!(found, 0);
                }
                other => panic!("Expected WALVersionMismatch, got {:?}", other),
            }
        }

        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn test_wal_auto_healing_and_recovery() {
        let dir = std::env::temp_dir().join("vanta_test_wal_healing");
        let _ = std::fs::remove_file(&dir);

        // 1. Escribir 3 registros válidos + checkpoint
        {
            let mut w = WalWriter::open(&dir).unwrap();
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
            file.write_all(b"\x0a\x00\x00\x00truncated garbage here that fails CRC or is cut off mid-way").unwrap();
        }

        // 3. Abrir el WAL de nuevo con WalWriter
        {
            let mut w = WalWriter::open(&dir).unwrap();
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
            }).unwrap();
            assert_eq!(records.len(), 5);
            match &records[4] {
                WalRecord::Insert(node) => assert_eq!(node.id, 4),
                _ => panic!("Expected Insert node at index 4"),
            }
        }

        let _ = std::fs::remove_file(&dir);
    }
}

