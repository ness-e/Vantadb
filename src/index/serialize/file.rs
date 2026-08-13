#[cfg(not(feature = "memmap2"))]
use crate::storage::vfile::MmapMut;
#[cfg(feature = "memmap2")]
use memmap2::MmapMut;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use tracing::{info, warn};

use crate::index::graph::{CPIndex, IndexBackend};

impl CPIndex {
    pub fn persist_to_file(&self, path: &Path) -> std::io::Result<()> {
        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("hnsw_serialize_fail", |_| {
                Err(std::io::Error::other(
                    "Injected HNSW persist serialization failure",
                ))
            });
        }
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.serialize_to_writer(&mut writer)?;
        writer.flush()?;
        info!(path = %path.display(), node_count = self.nodes.len(), "HNSW index persisted (streaming)");
        Ok(())
    }

    fn warn_validation_violations(index: &CPIndex) {
        if let Err(violations) = index.validate_index() {
            warn!(
                violation_count = violations.len(),
                "HNSW index has integrity violations after deserialization"
            );
            for v in &violations[..violations.len().min(5)] {
                warn!(violation = %v, "HNSW integrity violation");
            }
        }
    }

    pub fn load_from_file(path: &Path, use_mmap: bool) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        if use_mmap {
            let file = match OpenOptions::new().read(true).write(true).open(path) {
                Ok(f) => f,
                Err(_) => return None,
            };

            let file_len = file.metadata().ok().map(|m| m.len()).unwrap_or(0);
            if file_len < 64 {
                warn!("HNSW index file too small ({file_len} bytes) — will rebuild");
                return None;
            }

            // SAFETY: file size verified above — `map_mut` on a file shorter
            // than the mapping causes SIGBUS on access. We checked file_len >= 64
            // which covers the header, so the mapping cannot fault on header reads.
            let mmap = match unsafe { MmapMut::map_mut(&file) } {
                Ok(m) => m,
                Err(e) => {
                    warn!(err = %e, "Failed to mmap HNSW index file — will rebuild");
                    return None;
                }
            };

            match Self::deserialize_from_bytes(&mmap, false) {
                Ok(mut index) => {
                    info!(path = %path.display(), node_count = index.nodes.len(), "HNSW cold-start: loaded index from mmap file");
                    index.backend = IndexBackend::MMapFile {
                        path: path.to_path_buf(),
                        mmap: Some(mmap),
                    };
                    Self::warn_validation_violations(&index);
                    Some(index)
                }
                Err(e) => {
                    warn!(err = %e, "Corrupt vector_index.bin — will rebuild and overwrite");
                    None
                }
            }
        } else {
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(_) => return None,
            };

            match Self::deserialize_from_bytes(&data, true) {
                Ok(index) => {
                    info!(path = %path.display(), node_count = index.nodes.len(), "HNSW cold-start: loaded memory-copied index from file");
                    Self::warn_validation_violations(&index);
                    Some(index)
                }
                Err(e) => {
                    warn!(err = %e, "Corrupt vector_index.bin — will rebuild and overwrite");
                    None
                }
            }
        }
    }

    pub fn sync_to_mmap(&mut self) -> std::io::Result<()> {
        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("hnsw_serialize_fail", |_| {
                Err(std::io::Error::other(
                    "Injected HNSW sync mmap serialization failure",
                ))
            });
        }
        let path = match &self.backend {
            IndexBackend::MMapFile { path, .. } => path.clone(),
            _ => return Ok(()),
        };

        let data = self.serialize_to_bytes();
        let temp_path = path.with_extension("bin.tmp");

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;
        file.set_len(data.len() as u64)?;

        // SAFETY: `file` is a newly created/truncated handle at `data.len()` bytes;
        // `map_mut` validates the pointer internally.
        let mut mapped = unsafe { MmapMut::map_mut(&file)? };
        mapped.copy_from_slice(&data);
        mapped.flush()?;

        let new_index = Self::deserialize_from_bytes(&mapped, false)?;
        self.nodes = new_index.nodes;
        self.entry_point = new_index.entry_point;

        // Drop mmap and file handle before rename (Windows requires the temp file
        // to have no open handles for rename to succeed). Re-create after.
        drop(mapped);
        drop(file);
        std::fs::rename(&temp_path, &path)?;

        let file = OpenOptions::new().read(true).write(true).open(&path)?;
        let new_mmap = unsafe { MmapMut::map_mut(&file)? };
        if let IndexBackend::MMapFile { ref mut mmap, .. } = self.backend {
            *mmap = Some(new_mmap);
        }

        info!(path = %path.display(), node_count = self.nodes.len(), bytes = data.len(), "HNSW MMap synced & zero-copy pointers re-mapped via atomic rename");
        Ok(())
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::super::test_util::single_full_node_index;
    use super::*;
    use crate::node::VectorRepresentations;

    // ── Persist / load round-trip ──

    #[test]
    fn persist_and_load_roundtrip() {
        let index = single_full_node_index();
        let dir = std::env::temp_dir().join(format!("vantadb_ser_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("index.bin");

        index.persist_to_file(&path).unwrap();
        assert!(path.exists(), "file must exist");

        let loaded = match CPIndex::load_from_file(&path, false) {
            Some(index) => index,
            None => panic!("load should succeed"),
        };
        assert_eq!(loaded.nodes.len(), 2);
        let node = loaded.nodes.get(&42).unwrap();
        match &node.vec_data {
            VectorRepresentations::Full(v) => assert_eq!(v.as_slice(), &[0.1, 0.2, 0.3, 0.4]),
            _ => panic!("expected Full"),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_nonexistent_file_returns_none() {
        let path = std::env::temp_dir().join("vantadb__nonexistent__index.bin");
        let result = CPIndex::load_from_file(&path, false);
        assert!(result.is_none());
    }
    #[test]
    fn load_corrupt_file_returns_none() {
        let dir =
            std::env::temp_dir().join(format!("vantadb_ser_test_corrupt_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("corrupt.bin");
        std::fs::write(&path, [0u8; 32]).unwrap();
        let result = CPIndex::load_from_file(&path, false);
        assert!(result.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
