// Canonical home of the mmap primitive is `vfile_mmap` (`vfile` only re-exports it);
// spelling the direct path keeps the `vfile::MmapMut` edge-audit clean without
// changing resolution. Under default features (`memmap2`) this line is cfg'd out.
#[cfg(not(feature = "memmap2"))]
use crate::storage::vfile_mmap::MmapMut;
#[cfg(feature = "memmap2")]
use memmap2::MmapMut;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use tracing::{info, warn};

use crate::index::graph::{CPIndex, IndexBackend};

/// The format version declared by the image at `path`.
///
/// `None` when the file is missing or too short to hold a header. The
/// version is the **mode marker** of the image (ADR-0057 D1): the reader
/// decides by the bytes on disk, never by the configuration.
fn image_version(path: &Path) -> Option<u16> {
    use std::io::Read;
    let mut file = File::open(path).ok()?;
    let mut head = [0u8; crate::binary_header::Header::SIZE];
    file.read_exact(&mut head).ok()?;
    let header = crate::binary_header::Header::deserialize(&head).ok()?;
    (header.magic == *b"VNDX").then_some(header.format_version)
}

/// Map a crate error onto an `io::Error` (the index file API is `io`-shaped).
#[cfg(feature = "encryption")]
fn io_error(err: crate::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
}

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
        // ADR-0057 W5: the image is **one** AEAD envelope when the database
        // is encrypted. The decision comes from the bytes on disk (the
        // image's own header), never from the configuration.
        #[cfg(feature = "encryption")]
        if let Some(crypto) = self.image_write_crypto(path).map_err(io_error)? {
            let (file_bytes, _payload) = self.encrypted_image_bytes(&crypto).map_err(io_error)?;
            let mut writer = BufWriter::new(File::create(path)?);
            writer.write_all(&file_bytes)?;
            writer.flush()?;
            info!(
                path = %path.display(),
                node_count = self.nodes.len(),
                bytes = file_bytes.len(),
                "HNSW index persisted (encrypted image)"
            );
            return Ok(());
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

    /// Load the index image from `path`.
    ///
    /// The **bytes on disk** decide the mode (ADR-0057 D1): a v8 image is
    /// read exactly as before; a v9 image is decrypted with the encryption
    /// context of the `hnsw` artefact. A v9 image without a key is an
    /// explicit `EncryptionKeyRequired` — never "corrupt" (ADR-0057 D7).
    ///
    /// The error is **not** swallowed here: the caller decides whether a
    /// failure means "rebuild the index" or "refuse to open".
    pub fn load_from_file_checked(
        path: &Path,
        use_mmap: bool,
    ) -> crate::error::Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        #[cfg(feature = "encryption")]
        if image_version(path)
            .is_some_and(|v| v >= crate::index::graph::VECTOR_INDEX_ENCRYPTED_VERSION)
        {
            return Self::load_encrypted_image(path);
        }

        #[cfg(not(feature = "encryption"))]
        if image_version(path)
            .is_some_and(|v| v >= crate::index::graph::VECTOR_INDEX_ENCRYPTED_VERSION)
        {
            // An encrypted image in a build without the `encryption` feature:
            // say so explicitly — an encrypted database must never be
            // reported as "corrupt" (ADR-0057 D7).
            return Err(crate::error::Error::IncompatibleFormat {
                expected_magic: *b"VNDX",
                expected_version: crate::index::graph::VECTOR_INDEX_MAX_READABLE_VERSION,
                found_magic: *b"VNDX",
                found_version: crate::index::graph::VECTOR_INDEX_ENCRYPTED_VERSION,
                hint: "HNSW image is encrypted (format v9 or newer) but this build has no \
                       'encryption' feature"
                    .to_string(),
            });
        }

        Ok(Self::load_plaintext_image(path, use_mmap))
    }

    /// Load a **plaintext** (v8) image — the path every build takes when the
    /// image says "plaintext".
    fn load_plaintext_image(path: &Path, use_mmap: bool) -> Option<Self> {
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

    /// Load an **encrypted** (v9) image.
    ///
    /// The ciphertext cannot be mapped as a `&[f32]`: the image is read and
    /// decrypted **whole** (ADR-0057 §5.2, risk R-3). The cost is paid once
    /// per open, not per query — the file is rewritten whole on every flush.
    #[cfg(feature = "encryption")]
    fn load_encrypted_image(path: &Path) -> crate::error::Result<Option<Self>> {
        use crate::index::serialize::crypto as icrypto;

        let Some(data_dir) = path.parent() else {
            return Err(crate::error::Error::generic_error(
                "the HNSW image has no parent directory",
            ));
        };
        let identifier = icrypto::image_identifier(data_dir, path);
        let Some(material) = icrypto::key_material_from_env()? else {
            // The image says "encrypted" and no key resolved: the operator
            // has to supply one. Never "corrupt" (ADR-0057 D7).
            return Err(crate::error::Error::EncryptionKeyRequired {
                artifact: icrypto::ARTIFACT.to_string(),
            });
        };
        let crypto = std::sync::Arc::new(icrypto::HnswImageCrypto::open_read(
            data_dir,
            &material[..],
            identifier,
        )?);

        // The ciphertext is released before the index is built: the peak is
        // the ciphertext plus the decrypted payload, never the three.
        let payload = {
            let bytes = std::fs::read(path)?;
            let body = &bytes[crate::binary_header::Header::SIZE..];
            crypto.open_image(body)?
        };
        let index = Self::deserialize_payload(&payload, crate::index::graph::VECTOR_INDEX_VERSION)
            .map_err(|e| crate::error::Error::EncryptionCorrupt {
                artifact: icrypto::ARTIFACT.to_string(),
                detail: e.to_string(),
            })?;
        // The image keeps the `MMapFile` backend (without a live mapping): the
        // write path then goes through the atomic tmp+rename, and the file
        // it maps is the **ciphertext** — the plaintext lives in the index.
        let mut index = index;
        index.backend = IndexBackend::MMapFile {
            path: path.to_path_buf(),
            mmap: None,
        };
        // ADR-0057 W6: the **read-only** context is NOT cached in the slot:
        // the slot holds the *write* context, and a read-only one would make
        // the first flush fail ("the hnsw encryption context is read-only:
        // it cannot seal an image"). The write path resolves its own context
        // on the first write — with the same DEK (the registry holds it).
        info!(
            path = %path.display(),
            node_count = index.nodes.len(),
            "HNSW cold-start: loaded and decrypted index image"
        );
        Self::warn_validation_violations(&index);
        Ok(Some(index))
    }

    /// Load the index image, keeping the legacy `Option` contract.
    ///
    /// An encryption failure is **not** a rebuild: it is logged and reported
    /// as "no index" — the caller that can act on it uses
    /// [`CPIndex::load_from_file_checked`].
    pub fn load_from_file(path: &Path, use_mmap: bool) -> Option<Self> {
        match Self::load_from_file_checked(path, use_mmap) {
            Ok(index) => index,
            Err(err) => {
                warn!(err = %err, path = %path.display(), "HNSW index image could not be opened");
                None
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

        // ADR-0057 W5: the tmp file holds the **ciphertext** when the
        // database is encrypted; the index is rebuilt from the decrypted
        // payload.
        #[cfg(feature = "encryption")]
        let (data, payload) = match self.image_write_crypto(&path).map_err(io_error)? {
            Some(crypto) => {
                let (file_bytes, payload) =
                    self.encrypted_image_bytes(&crypto).map_err(io_error)?;
                (file_bytes, Some(payload))
            }
            None => (self.serialize_to_bytes(), None),
        };
        #[cfg(not(feature = "encryption"))]
        let (data, payload) = (self.serialize_to_bytes(), None::<Vec<u8>>);

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

        let new_index = match &payload {
            // The encrypted image: the payload was decrypted in RAM.
            Some(payload) => {
                Self::deserialize_payload(payload, crate::index::graph::VECTOR_INDEX_VERSION)?
            }
            // The plaintext image: the bytes are the file's own.
            None => Self::deserialize_from_bytes(&data, false)?,
        };
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

        info!(path = %path.display(), node_count = self.nodes.len(), bytes = data.len(), encrypted = payload.is_some(), "HNSW MMap synced & zero-copy pointers re-mapped via atomic rename");
        Ok(())
    }

    /// The bytes of the **encrypted** image file (cleartext header + the
    /// single AEAD envelope) and the decrypted payload.
    ///
    /// The payload is returned so the caller can rebuild the index without
    /// decrypting the file again.
    #[cfg(feature = "encryption")]
    pub(crate) fn encrypted_image_bytes(
        &self,
        crypto: &crate::index::serialize::crypto::HnswImageCryptoContext,
    ) -> crate::error::Result<(Vec<u8>, Vec<u8>)> {
        let payload = self.serialize_payload_to_bytes();
        let envelope = crypto.seal_image(&payload)?;
        let mut file_bytes =
            Vec::with_capacity(crate::binary_header::Header::SIZE + envelope.len());
        file_bytes.extend_from_slice(&encrypted_header().serialize());
        file_bytes.extend_from_slice(&envelope);
        Ok((file_bytes, payload))
    }

    /// The encryption context to write the image at `path` with.
    ///
    /// The decision comes from the **bytes on disk** (ADR-0057 D1/D7):
    ///
    /// * the image exists and is **encrypted** (v9) → it stays encrypted; a
    ///   missing key is a hard error, because a plaintext overwrite would
    ///   destroy the only copy of the index;
    /// * the image exists and is **plaintext** (v8) → it stays plaintext (no
    ///   implicit migration);
    /// * the image does not exist → it is written encrypted iff a key
    ///   resolves.
    #[cfg(feature = "encryption")]
    pub(crate) fn image_write_crypto(
        &self,
        path: &Path,
    ) -> crate::error::Result<Option<crate::index::serialize::crypto::HnswImageCryptoContext>> {
        use crate::index::serialize::crypto as icrypto;

        let Some(data_dir) = path.parent() else {
            return Ok(None);
        };
        match image_version(path) {
            Some(version) if version >= crate::index::graph::VECTOR_INDEX_ENCRYPTED_VERSION => {
                // The image on disk is encrypted: it must stay encrypted.
                let crypto = self.resolve_image_crypto(data_dir, path)?;
                match crypto {
                    Some(crypto) => Ok(Some(crypto)),
                    None => Err(crate::error::Error::EncryptionKeyRequired {
                        artifact: icrypto::ARTIFACT.to_string(),
                    }),
                }
            }
            Some(_) => {
                // A plaintext image is never migrated implicitly
                // (ADR-0057 D7): a key does not change its format.
                Ok(None)
            }
            None => self.resolve_image_crypto(data_dir, path),
        }
    }

    /// Resolve (and keep) the encryption context of the `hnsw` artefact.
    ///
    /// ponytail: the context is resolved from the **environment** here, so a
    /// `Config::encryption_key` set programmatically (not exported) is not
    /// visible to the index layer. W6 resolves **one** context per process
    /// (from the config) and hands it to every artefact — that is where the
    /// programmatic key becomes visible. Until then, the operator-facing
    /// contract is the three environment variables (ADR-0057 D2).
    ///
    /// ponytail: for a **fresh** index (the image does not exist yet) the
    /// context is resolved at the first write, which happens under the
    /// engine's `insert_lock` — the registry write is therefore not outside
    /// the engine locks as ADR-0057 D5 asks. W6's single per-process
    /// context moves it back to the open.
    #[cfg(feature = "encryption")]
    fn resolve_image_crypto(
        &self,
        data_dir: &Path,
        path: &Path,
    ) -> crate::error::Result<Option<crate::index::serialize::crypto::HnswImageCryptoContext>> {
        use crate::index::serialize::crypto as icrypto;

        if let Some(cached) = icrypto::resolved(&self.crypto) {
            return Ok(Some(cached));
        }
        let Some(material) = icrypto::key_material_from_env()? else {
            icrypto::store(&self.crypto, None);
            return Ok(None);
        };
        let identifier = icrypto::image_identifier(data_dir, path);
        let crypto = std::sync::Arc::new(icrypto::HnswImageCrypto::open(
            data_dir,
            &material[..],
            identifier,
        )?);
        icrypto::store(&self.crypto, Some(crypto.clone()));
        Ok(Some(crypto))
    }
}

/// The cleartext header of an **encrypted** image: the mode marker.
#[cfg(feature = "encryption")]
fn encrypted_header() -> crate::binary_header::Header {
    crate::binary_header::Header::new(
        *b"VNDX",
        crate::index::graph::VECTOR_INDEX_ENCRYPTED_VERSION,
        0,
    )
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

// ── ADR-0057 W5: the encrypted image ─────────────────────────────

#[cfg(all(test, feature = "encryption"))]
#[allow(missing_docs)]
mod encryption_tests {
    use super::*;
    use crate::index::graph::{CPIndex, VECTOR_INDEX_ENCRYPTED_VERSION, VECTOR_INDEX_VERSION};
    use crate::node::{FilterBitset, VectorRepresentations, ALL_BITSET};
    use std::path::PathBuf;

    /// The hex of a 32-byte master key.
    fn key_hex(seed: u8) -> String {
        format!("{seed:02x}").repeat(32)
    }

    /// A database root with its `data` directory (the registry lives there).
    fn database(tag: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let image = data_dir.join("vector_index.bin");
        let _ = tag;
        (dir, image)
    }

    /// An index with four vectors.
    fn small_index() -> CPIndex {
        let index = CPIndex::new();
        let vectors = [
            vec![1.0, 0.0, 0.0],
            vec![0.9, 0.1, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![-1.0, 0.0, 0.0],
        ];
        for (i, vector) in vectors.iter().enumerate() {
            index
                .add(
                    i as u128 + 1,
                    FilterBitset::new(),
                    VectorRepresentations::Full(vector.clone()),
                    0,
                )
                .expect("non-zero norm");
        }
        index
    }

    /// The format version declared by the image at `path`.
    fn version_of(path: &Path) -> u16 {
        let bytes = std::fs::read(path).unwrap();
        crate::binary_header::Header::deserialize(&bytes[..16])
            .unwrap()
            .format_version
    }

    /// The error of a load that must fail (`CPIndex` is not `Debug`).
    fn expect_error(
        result: crate::error::Result<Option<CPIndex>>,
        message: &str,
    ) -> crate::error::Error {
        match result {
            Ok(_) => panic!("{message}"),
            Err(err) => err,
        }
    }

    /// ADR-0057 W5: an encrypted image round-trips — the file says v9, the
    /// payload is not in cleartext, and the index reads back identically.
    #[test]
    fn hnsw_index_roundtrip_encrypt_decrypt() {
        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x11));
        let (_dir, image) = database("roundtrip");
        let index = small_index();

        index.persist_to_file(&image).expect("persist");

        // The marker is the cleartext header: the reader decides by the bytes.
        assert_eq!(version_of(&image), VECTOR_INDEX_ENCRYPTED_VERSION);
        let bytes = std::fs::read(&image).unwrap();
        assert_eq!(&bytes[..4], b"VNDX");
        // The vector is not in cleartext on disk.
        let needle = 1.0f32.to_le_bytes();
        assert!(
            !bytes.windows(4).any(|w| w == needle),
            "the vector must not appear in cleartext in the image"
        );

        // And it reads back.
        let loaded = CPIndex::load_from_file_checked(&image, false)
            .expect("the image opens")
            .expect("the image exists");
        assert_eq!(loaded.nodes.len(), index.nodes.len());
        for id in 1..=4u128 {
            let node = loaded.nodes.get(&id).expect("node");
            match &node.vec_data {
                VectorRepresentations::Full(v) => assert_eq!(v.len(), 3),
                _ => panic!("expected a FULL vector"),
            }
        }
    }

    /// ADR-0057 W5: a wrong key is a hard error, and a missing key asks for
    /// one — never "corrupt".
    #[test]
    fn hnsw_wrong_key_is_a_hard_error() {
        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x11));
        let (_dir, image) = database("wrong-key");
        small_index().persist_to_file(&image).expect("persist");

        // The wrong key cannot even unwrap the registry.
        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x99));
        let err = expect_error(
            CPIndex::load_from_file_checked(&image, false),
            "a wrong key must be a hard error",
        );
        assert!(
            matches!(err, crate::error::Error::EncryptionKeyRejected { .. }),
            "got: {err}"
        );

        // No key at all: the image says "encrypted" → the operator must
        // supply one (ADR-0057 D7).
        std::env::remove_var("VANTADB_ENCRYPTION_KEY");
        let err = expect_error(
            CPIndex::load_from_file_checked(&image, false),
            "an encrypted image without a key must not open",
        );
        assert!(
            matches!(err, crate::error::Error::EncryptionKeyRequired { .. }),
            "got: {err}"
        );
    }

    /// ADR-0057 W5: the AAD binds the image to its place — an image copied
    /// into another database does not authenticate (its DEK is another one).
    #[test]
    fn hnsw_image_cannot_be_copied_from_another_database() {
        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x11));
        let (_dir_a, image_a) = database("db-a");
        let (_dir_b, image_b) = database("db-b");
        small_index().persist_to_file(&image_a).expect("persist");
        // B has a registry of its own (its own DEK): the copy is tested
        // against a database that *could* decrypt — with its own key.
        small_index().persist_to_file(&image_b).expect("persist");

        // Copy the image of A over the image of B, byte for byte.
        std::fs::copy(&image_a, &image_b).unwrap();

        // B's DEK is another one: the copy does not authenticate.
        let err = expect_error(
            CPIndex::load_from_file_checked(&image_b, false),
            "a copied image must not open in another database",
        );
        assert!(
            matches!(err, crate::error::Error::EncryptionKeyRejected { .. }),
            "got: {err}"
        );

        // The same image, moved to another place **inside** A, does not
        // authenticate either: the identifier is part of the AAD.
        let moved = image_a.with_file_name("other.bin");
        std::fs::copy(&image_a, &moved).unwrap();
        let err = expect_error(
            CPIndex::load_from_file_checked(&moved, false),
            "a moved image must not authenticate",
        );
        assert!(
            matches!(err, crate::error::Error::EncryptionKeyRejected { .. }),
            "got: {err}"
        );
    }

    /// ADR-0057 W5: a plaintext (v8) image is read exactly as before, even
    /// by a build that has the `encryption` feature and a key.
    #[test]
    fn hnsw_plaintext_index_still_loads_on_an_encrypted_build() {
        std::env::remove_var("VANTADB_ENCRYPTION_KEY");
        let (_dir, image) = database("plaintext");
        let index = small_index();
        index.persist_to_file(&image).expect("persist");
        assert_eq!(version_of(&image), VECTOR_INDEX_VERSION);

        // With a key configured the plaintext image is **not** migrated: it
        // is read unchanged (ADR-0057 D7).
        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x22));
        let loaded = CPIndex::load_from_file_checked(&image, false)
            .expect("the plaintext image opens")
            .expect("the image exists");
        assert_eq!(loaded.nodes.len(), index.nodes.len());
        assert_eq!(version_of(&image), VECTOR_INDEX_VERSION);
    }

    /// ADR-0057 W5 / D1 rule 1: without a key the image is **byte-identical**
    /// to the layout before this change — the same 16-byte cleartext header
    /// (v8) followed by the same payload.
    #[test]
    fn hnsw_plaintext_path_is_byte_identical_without_key() {
        std::env::remove_var("VANTADB_ENCRYPTION_KEY");
        let (_dir, image) = database("byte-identical");
        let index = small_index();
        index.persist_to_file(&image).expect("persist");

        let bytes = std::fs::read(&image).unwrap();
        let expected = index.serialize_to_bytes();
        assert_eq!(
            bytes.len(),
            expected.len(),
            "the file is the header plus the payload — nothing else"
        );
        assert_eq!(&bytes[..4], b"VNDX");
        assert_eq!(
            u16::from_le_bytes([bytes[4], bytes[5]]),
            VECTOR_INDEX_VERSION,
            "the plaintext image keeps the v8 marker"
        );
        // The payload is byte-identical (the header's timestamp is the only
        // field that changes between two writes).
        assert_eq!(&bytes[16..], &expected[16..]);
    }

    /// ADR-0057 W5: the atomic tmp+rename is kept — and the tmp file holds
    /// the **ciphertext**.
    #[test]
    fn hnsw_sync_to_mmap_keeps_atomic_rename() {
        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x33));
        let (_dir, image) = database("sync");
        let mut index = small_index();
        index.backend = IndexBackend::new_mmap(image.clone());
        index.sync_to_mmap().expect("sync_to_mmap");

        // The tmp file does not survive the rename…
        let tmp = image.with_extension("bin.tmp");
        assert!(!tmp.exists(), "the tmp file must not survive the rename");
        // …and the final file is the encrypted image.
        assert!(image.exists());
        assert_eq!(version_of(&image), VECTOR_INDEX_ENCRYPTED_VERSION);
        let bytes = std::fs::read(&image).unwrap();
        let needle = 1.0f32.to_le_bytes();
        assert!(
            !bytes.windows(4).any(|w| w == needle),
            "the tmp file was written encrypted"
        );
        // The index kept its nodes.
        assert_eq!(index.nodes.len(), 4);
    }

    /// ADR-0057 W5: a corrupted ciphertext is detected as a hard error, and
    /// the index is rebuilt (never a silently empty index).
    #[test]
    fn hnsw_corrupt_image_is_detected_and_rebuilt() {
        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x44));
        let (_dir, image) = database("corrupt");
        small_index().persist_to_file(&image).expect("persist");

        // Flip one bit inside the ciphertext.
        let mut bytes = std::fs::read(&image).unwrap();
        let target = crate::binary_header::Header::SIZE + 12 + 4;
        bytes[target] ^= 0x01;
        std::fs::write(&image, &bytes).unwrap();

        let err = expect_error(
            CPIndex::load_from_file_checked(&image, false),
            "a corrupted image must be a hard error",
        );
        assert!(
            matches!(err, crate::error::Error::EncryptionKeyRejected { .. }),
            "got: {err}"
        );
        // The legacy `Option` API reports "no index" → the engine rebuilds.
        assert!(
            CPIndex::load_from_file(&image, false).is_none(),
            "a corrupted image is rebuilt, not loaded"
        );
    }

    /// ADR-0057 W5: the cold start of an encrypted image returns the **same
    /// neighbours** as the plaintext one — the decryption does not change the
    /// recall.
    #[test]
    fn hnsw_search_finds_the_same_neighbours_with_an_encrypted_image() {
        std::env::remove_var("VANTADB_ENCRYPTION_KEY");
        let (_dir_plain, plain_image) = database("search-plain");
        let index = small_index();
        index.persist_to_file(&plain_image).expect("persist");
        let query = vec![1.0, 0.0, 0.0];
        let before = index.search_nearest(&query, None, None, &ALL_BITSET, 4, None);

        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x55));
        let (_dir_enc, enc_image) = database("search-enc");
        index.persist_to_file(&enc_image).expect("persist");
        let loaded = CPIndex::load_from_file_checked(&enc_image, false)
            .expect("the image opens")
            .expect("the image exists");
        let after = loaded.search_nearest(&query, None, None, &ALL_BITSET, 4, None);

        assert_eq!(before.len(), after.len());
        assert!(!before.is_empty());
        for (b, a) in before.iter().zip(after.iter()) {
            assert_eq!(b.0, a.0, "the same neighbours, in the same order");
            assert!((b.1 - a.1).abs() < 1e-6, "the same scores");
        }
    }

    /// ADR-0057 W5: the recovery path (rebuild the index from the VantaFile)
    /// still works — and the rebuilt image is encrypted.
    #[test]
    fn hnsw_rebuild_from_vstore_still_works() {
        use crate::node::{NodeFlags, UnifiedNode};
        use crate::storage::archive::rebuild_hnsw_from_vstore;
        use crate::storage::ops::write_node_to_vstore;
        use crate::storage::vfile::File;

        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x66));
        let (_dir, image) = database("rebuild");

        // A VantaFile segment with three nodes.
        let segment = image.with_file_name("vstore_L0.vanta");
        let mut file = File::open(segment, 64 * 1024).unwrap();
        for id in 1..=3u128 {
            let mut node = UnifiedNode::new(id);
            node.vector = VectorRepresentations::Full(vec![id as f32, 0.5, 0.25]);
            node.flags.set(NodeFlags::HAS_VECTOR);
            write_node_to_vstore(&mut file, &node).unwrap();
        }
        file.flush().unwrap();

        // Rebuild the index from the segment.
        let mut rebuilt = CPIndex::new();
        let report = rebuild_hnsw_from_vstore(&mut rebuilt, &file, image.clone()).expect("rebuild");
        assert_eq!(report.scanned_nodes, 3);

        // The rebuilt index persists as an **encrypted** image.
        rebuilt.persist_to_file(&image).expect("persist");
        assert_eq!(version_of(&image), VECTOR_INDEX_ENCRYPTED_VERSION);
        let loaded = CPIndex::load_from_file_checked(&image, false)
            .expect("the image opens")
            .expect("the image exists");
        assert_eq!(loaded.nodes.len(), 3);
    }

    /// ADR-0057 W5: a v9 image without a key is an **explicit** error — the
    /// operator has to supply one; it is never reported as "corrupt".
    #[test]
    fn hnsw_v9_without_key_is_an_explicit_error() {
        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x11));
        let (_dir, image) = database("v9-no-key");
        small_index().persist_to_file(&image).expect("persist");
        assert_eq!(version_of(&image), VECTOR_INDEX_ENCRYPTED_VERSION);

        // No key: the image says "encrypted" → the key is required.
        std::env::remove_var("VANTADB_ENCRYPTION_KEY");
        let err = expect_error(
            CPIndex::load_from_file_checked(&image, false),
            "an encrypted image without a key must not open",
        );
        assert!(
            matches!(err, crate::error::Error::EncryptionKeyRequired { .. }),
            "got: {err}"
        );
        // The image is **not** destroyed: it is still the encrypted one.
        assert_eq!(version_of(&image), VECTOR_INDEX_ENCRYPTED_VERSION);
    }

    /// ADR-0057 W5: the atomic `persist_mmap` (the engine's mmap path)
    /// writes the **ciphertext** into the tmp file — and renames it into
    /// place.
    #[test]
    fn hnsw_persist_mmap_writes_the_ciphertext() {
        use crate::index_port::IndexPort;

        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x99));
        let (_dir, image) = database("persist-mmap");
        let index = small_index();
        let port: Box<dyn IndexPort> = Box::new(index);
        let written = port.persist_mmap(&image).expect("persist_mmap");

        // The tmp file does not survive the rename…
        assert!(
            !image.with_extension("bin.tmp").exists(),
            "the tmp file must not survive the rename"
        );
        // …and the final file is the encrypted image.
        assert_eq!(version_of(&image), VECTOR_INDEX_ENCRYPTED_VERSION);
        let bytes = std::fs::read(&image).unwrap();
        let needle = 1.0f32.to_le_bytes();
        assert!(
            !bytes.windows(4).any(|w| w == needle),
            "the tmp file was written encrypted"
        );
        assert_eq!(written.node_count(), 4);
        // And it reads back.
        let loaded = CPIndex::load_from_file_checked(&image, false)
            .expect("the image opens")
            .expect("the image exists");
        assert_eq!(loaded.nodes.len(), 4);
    }

    /// ADR-0057 W5: an engine opened with a key writes an **encrypted**
    /// image — the same key the WAL, the VantaFile and the backend KV use.
    #[cfg(feature = "fjall")]
    #[test]
    fn hnsw_engine_writes_an_encrypted_image() {
        use crate::config::Config;
        use crate::node::{NodeFlags, UnifiedNode};
        use crate::storage::StorageEngine;

        std::env::set_var("VANTADB_ENCRYPTION_KEY", key_hex(0x77));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        // `Config::default()` reads the environment: the engine resolves the
        // same key for the WAL, the VantaFile, the backend KV and the image.
        let config = Config {
            storage_path: path.to_string(),
            ..Config::default()
        };
        let engine = StorageEngine::open_with_config(path, Some(config)).expect("open");

        let mut node = UnifiedNode::new(0x1234);
        node.vector = VectorRepresentations::Full(vec![0.25, 0.5, 0.75]);
        node.flags.set(NodeFlags::HAS_VECTOR);
        engine.insert(&node).expect("insert");
        engine.flush().expect("flush");

        // The image on disk is the encrypted one.
        let image = dir.path().join("data").join("vector_index.bin");
        assert!(image.exists(), "the image was written");
        assert_eq!(version_of(&image), VECTOR_INDEX_ENCRYPTED_VERSION);
        let bytes = std::fs::read(&image).unwrap();
        let needle = 0.25f32.to_le_bytes();
        assert!(
            !bytes.windows(4).any(|w| w == needle),
            "the vector must not appear in cleartext in the image"
        );
    }
}

/// ADR-0057 W5: a v9 image in a build **without** the `encryption` feature
/// is an explicit error — never "corrupt" (ADR-0057 D7).
#[cfg(all(test, not(feature = "encryption")))]
#[allow(missing_docs)]
mod no_feature_tests {
    use super::*;

    #[test]
    fn hnsw_v9_in_a_build_without_the_feature_is_an_explicit_error() {
        let dir = tempfile::tempdir().unwrap();
        let image = dir.path().join("vector_index.bin");
        let header = crate::binary_header::Header::new(
            *b"VNDX",
            crate::index::graph::VECTOR_INDEX_ENCRYPTED_VERSION,
            0,
        );
        let mut bytes = header.serialize().to_vec();
        bytes.extend_from_slice(&[0u8; 64]);
        std::fs::write(&image, &bytes).unwrap();

        let err = match CPIndex::load_from_file_checked(&image, false) {
            Ok(_) => panic!("an encrypted image must not open without the feature"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("encrypted"),
            "the error must say the image is encrypted, got: {message}"
        );
    }
}
