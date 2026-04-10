use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, VantaError};
use crate::node::UnifiedNode;

// ─── WAL Record ────────────────────────────────────────────

/// WAL record types (bincode-serialized)
#[derive(Serialize, Deserialize, Debug)]
pub enum WalRecord {
    Insert(UnifiedNode),
    Update { id: u64, node: UnifiedNode },
    Delete { id: u64 },
    Checkpoint { node_count: u64 },
}

// ─── WAL Writer ────────────────────────────────────────────

/// Append-only WAL writer with CRC32 integrity checks.
///
/// Record format: [len:u32][payload:bincode][crc:u32]
pub struct WalWriter {
    writer: BufWriter<File>,
    path: PathBuf,
    bytes_written: u64,
    record_count: u64,
}

impl WalWriter {
    /// Open or create WAL file for appending
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let bytes_written = file.metadata()?.len();
        Ok(Self {
            writer: BufWriter::with_capacity(64 * 1024, file),
            path,
            bytes_written,
            record_count: 0,
        })
    }

    /// Append a record to the WAL
    pub fn append(&mut self, record: &WalRecord) -> Result<()> {
        let payload = bincode::serialize(record)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        let len = payload.len() as u32;
        let crc = crc32(&payload);

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
        let file = File::open(path)?;
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
        let computed_crc = crc32(&payload);

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

// ─── CRC32 ─────────────────────────────────────────────────

/// Simple CRC32 (IEEE polynomial, non-cryptographic)
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::UnifiedNode;

    #[test]
    fn test_wal_roundtrip() {
        let dir = std::env::temp_dir().join("connectome_test_wal_rt");
        let _ = std::fs::remove_file(&dir);

        {
            let mut w = WalWriter::open(&dir).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(2))).unwrap();
            w.append(&WalRecord::Delete { id: 1 }).unwrap();
            w.sync().unwrap();
            assert_eq!(w.record_count(), 3);
        }

        {
            let mut r = WalReader::open(&dir).unwrap();
            let mut records = Vec::new();
            r.replay_all(|rec| {
                records.push(rec);
                Ok(())
            })
            .unwrap();
            assert_eq!(records.len(), 3);
        }

        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn test_crc32_deterministic() {
        let data = b"connectome wal test";
        assert_eq!(crc32(data), crc32(data));
        assert_ne!(crc32(data), crc32(b"connectome wal tesx"));
    }
}
