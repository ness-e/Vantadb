//! Encryption at rest for the VantaFile segments (ADR-0057 W3).
//!
//! The VantaFile is an **offset-addressed** store: the HNSW index keeps the
//! byte offset of every node and the reader jumps straight to it. The
//! envelope granularity therefore has to be **per node**, not per file
//! (ADR-0057 D1, option B): a whole-file blob would destroy the random
//! access the format exists for.
//!
//! ```text
//! v3 node record @ offset `off` (64-byte aligned):
//!
//!   [u32 blob_len][12B nonce][ct ‖ 16B tag]
//!
//!   plaintext = DiskNodeHeader (64 B) ‖ vector payload
//! ```
//!
//! * `blob_len` is the length of the AEAD blob (`ciphertext ‖ tag`).
//! * The **AAD** binds the blob to its **byte offset** (`slot = off`), so a
//!   node moved to another offset fails to authenticate (ADR-0057 D4).
//! * The nonce is `epoch ‖ counter` and the epoch is persisted **before**
//!   the first encryption under it (ADR-0057 D3).
//!
//! One context exists per **database**, not per segment: the DEK and the
//! nonce space belong to the `vfile` artefact, so L0..L3 share them (two
//! segments with their own counters would reuse nonces).

#[cfg(feature = "encryption")]
use crate::error::{Error, Result};

#[cfg(feature = "encryption")]
use crate::crypto::registry::{AadBuilder, Artifact, KeyRegistry, NonceAllocator};
#[cfg(feature = "encryption")]
use crate::crypto::{Cipher, CryptoError, Nonce, Zeroizing};
#[cfg(feature = "encryption")]
use parking_lot::Mutex;
#[cfg(feature = "encryption")]
use std::path::Path;
// `Arc` is needed in both builds: the no-feature slot is `Option<Arc<()>>` (see below).
use std::sync::Arc;

/// The artefact id used in the error messages of this module.
#[cfg(feature = "encryption")]
pub(crate) const ARTIFACT: &str = "vfile";

/// The encryption context carried by a segment.
///
/// With the `encryption` feature it holds the real [`VFileCrypto`]; without
/// it the slot is uninhabited and every segment stays plaintext
/// (byte-identical to the layout before this change).
///
/// The no-feature variant is `Option<Arc<()>>` and not `Option<()>` on purpose:
/// `Option<()>` is `Copy`, so the `.clone()` calls at the call sites would trip
/// clippy's `clone_on_copy` under `-D warnings` **only** in the build without
/// the feature (which is exactly the build the closing gate runs).
#[cfg(feature = "encryption")]
pub(crate) type VFileCryptoSlot = Option<Arc<VFileCrypto>>;
#[cfg(not(feature = "encryption"))]
pub(crate) type VFileCryptoSlot = Option<Arc<()>>;

/// The **write side** of an encrypted VantaFile: the DEK of the `vfile`
/// artefact, the shared nonce allocator and the registry handle needed to
/// roll the epoch.
#[cfg(feature = "encryption")]
pub(crate) struct VFileCrypto {
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
#[cfg(feature = "encryption")]
impl std::fmt::Debug for VFileCrypto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VFileCrypto")
            .field("writable", &self.allocator.is_some())
            .finish()
    }
}

#[cfg(feature = "encryption")]
impl VFileCrypto {
    /// Open the **write-side** context of the `vfile` artefact in `data_dir`.
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
            .ensure_dek(Artifact::VantaFile)
            .map_err(crypto_error)?;
        registry
            .begin_epoch(Artifact::VantaFile)
            .map_err(crypto_error)?;
        let allocator = registry
            .allocator(Artifact::VantaFile)
            .cloned()
            .ok_or_else(|| {
                Error::generic_error(
                    "encryption registry has no nonce allocator for the vfile artefact",
                )
            })?;
        let dek =
            registry
                .dek(Artifact::VantaFile)
                .ok_or_else(|| Error::EncryptionKeyRequired {
                    artifact: ARTIFACT.to_string(),
                })?;
        Ok(Self {
            cipher: Cipher::new(&dek[..]),
            allocator: Some(arc_swap::ArcSwapAny::new(allocator)),
            registry: Some(Arc::new(Mutex::new(registry))),
        })
    }

    /// Open the **read-side** context of the `vfile` artefact in `data_dir`.
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
                .dek(Artifact::VantaFile)
                .ok_or_else(|| Error::EncryptionKeyRequired {
                    artifact: ARTIFACT.to_string(),
                })?;
        Ok(Self {
            cipher: Cipher::new(&dek[..]),
            allocator: None,
            registry: None,
        })
    }

    /// Seal a node record: `[u32 blob_len][12B nonce][ct ‖ 16B tag]`.
    ///
    /// `offset` is the byte offset the record will live at — it is the
    /// **slot** of the AAD, so a record moved to another offset no longer
    /// authenticates (ADR-0057 D4).
    ///
    /// # Errors
    ///
    /// [`Error::EncryptionKeyRequired`] when the context is read-only and
    /// [`Error::EncryptionCorrupt`] when the registry cannot be written.
    pub(crate) fn seal_node(&self, offset: u64, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = self.next_nonce()?;
        let aad = AadBuilder::new(
            Artifact::VantaFile,
            crate::storage::vfile::VFILE_ENCRYPTED_VERSION,
        )
        .epoch(nonce.epoch())
        .slot(offset)
        .plaintext_len(plaintext.len())
        .build();
        // `ct` is the AEAD blob: `ciphertext ‖ 16-byte tag`.
        let ct = self
            .cipher
            .encrypt_with_nonce(nonce.as_bytes(), plaintext, &aad)
            .map_err(crypto_error)?;
        let mut record = Vec::with_capacity(RECORD_PREFIX_LEN + ct.len());
        record.extend_from_slice(&(ct.len() as u32).to_le_bytes());
        record.extend_from_slice(nonce.as_bytes());
        record.extend_from_slice(&ct);
        Ok(record)
    }

    /// Open a node record: the plaintext `DiskNodeHeader ‖ payload`.
    ///
    /// The returned buffer is wrapped in [`Zeroizing`] so its bytes are wiped
    /// on drop (ADR-0057 C-5 / FIND-618): the AEAD output is the only copy of
    /// the plaintext before it is copied into the `AlignedBlob`; releasing it
    /// without wiping would leave the plaintext in the heap.
    ///
    /// # Errors
    ///
    /// [`Error::EncryptionCorrupt`] when the record is structurally broken
    /// (truncated, impossible length) and [`Error::EncryptionKeyRejected`]
    /// when the AEAD tag does not authenticate (wrong key **or** tampered
    /// ciphertext — a single tag cannot tell them apart).
    pub(crate) fn open_node(&self, offset: u64, record: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        if record.len() < RECORD_PREFIX_LEN + TAG_LEN {
            return Err(corrupt(format!(
                "node record at offset {offset} is truncated ({} bytes)",
                record.len()
            )));
        }
        let blob_len = u32::from_le_bytes([record[0], record[1], record[2], record[3]]) as usize;
        if blob_len < TAG_LEN || RECORD_PREFIX_LEN + blob_len > record.len() {
            return Err(corrupt(format!(
                "node record at offset {offset} declares an impossible blob length {blob_len}"
            )));
        }
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&record[4..RECORD_PREFIX_LEN]);
        let ct = &record[RECORD_PREFIX_LEN..RECORD_PREFIX_LEN + blob_len];
        let aad = AadBuilder::new(
            Artifact::VantaFile,
            crate::storage::vfile::VFILE_ENCRYPTED_VERSION,
        )
        .epoch(Nonce::from_bytes(nonce).epoch())
        .slot(offset)
        .plaintext_len(blob_len - TAG_LEN)
        .build();
        self.cipher
            .decrypt_with_nonce(&nonce, ct, &aad)
            .map(Zeroizing::new)
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
                "the vfile encryption context is read-only: it cannot seal a record",
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
                "the vfile encryption context is read-only: it cannot roll the nonce epoch",
            ));
        };
        let mut registry = registry.lock();
        // Another writer may have rolled the epoch already: adopt its
        // allocator instead of burning a second epoch — the artefact keeps a
        // single, shared nonce sequence.
        if let Some(current) = registry.allocator(Artifact::VantaFile) {
            let mine = allocator.load();
            if !Arc::ptr_eq(current, &*mine) {
                allocator.store(current.clone());
                return Ok(());
            }
        }
        registry
            .begin_epoch(Artifact::VantaFile)
            .map_err(crypto_error)?;
        let fresh = registry
            .allocator(Artifact::VantaFile)
            .cloned()
            .ok_or_else(|| {
                Error::generic_error(
                    "encryption registry has no nonce allocator for the vfile artefact",
                )
            })?;
        allocator.store(fresh);
        Ok(())
    }
}

/// Build the encryption context of the `vfile` artefact from the
/// configuration, if a key resolves.
///
/// Precedence: environment → key file → key command (ADR-0057 D2). Without a
/// key the segments stay **plaintext and byte-identical** to the layout
/// before this change.
#[cfg(feature = "encryption")]
pub(crate) fn vfile_crypto_from_config(
    data_dir: &Path,
    config: &crate::config::Config,
) -> Result<Option<Arc<VFileCrypto>>> {
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
    let crypto = if config.read_only {
        VFileCrypto::open_read(data_dir, &material[..])?
    } else {
        VFileCrypto::open(data_dir, &material[..])?
    };
    Ok(Some(Arc::new(crypto)))
}

/// Map a crypto failure onto the crate error surface (ADR-0057 D6).
#[cfg(feature = "encryption")]
fn crypto_error(err: CryptoError) -> Error {
    match err {
        CryptoError::DecryptionFailed => Error::EncryptionKeyRejected {
            artifact: ARTIFACT.to_string(),
        },
        CryptoError::RegistryInvalid(detail) => Error::EncryptionCorrupt {
            artifact: ARTIFACT.to_string(),
            detail,
        },
        other => Error::generic_error(format!("VantaFile encryption error: {other}")),
    }
}

/// A structural failure of an encrypted artefact.
#[cfg(feature = "encryption")]
fn corrupt(detail: String) -> Error {
    Error::EncryptionCorrupt {
        artifact: ARTIFACT.to_string(),
        detail,
    }
}

/// Length of the AEAD nonce (ADR-0057 D3).
pub(crate) const NONCE_LEN: usize = 12;

/// Length of the AEAD tag.
pub(crate) const TAG_LEN: usize = 16;

/// Bytes of the record prefix: `[u32 blob_len][12B nonce]`.
pub(crate) const RECORD_PREFIX_LEN: usize = 4 + NONCE_LEN;

// ─── Tests (ADR-0057 W3) ─────────────────────────────────────

#[cfg(all(test, feature = "encryption"))]
mod tests {
    use super::*;
    use crate::binary_header::Header;
    use crate::index::CPIndex;
    use crate::node::{
        DiskNodeHeader, FilterBitset, NodeFlags, UnifiedNode, VectorRepresentations,
    };
    use crate::storage::archive::compact_layout;
    use crate::storage::ops::write_node_to_vstore;
    use crate::storage::vfile::{File, VFILE_ENCRYPTED_VERSION, VFILE_VERSION};
    use zerocopy::IntoBytes;

    /// A 32-byte master key.
    fn material(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    /// A node with a FULL vector.
    fn node(id: u128, vector: Vec<f32>) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.vector = VectorRepresentations::Full(vector);
        node.flags.set(NodeFlags::HAS_VECTOR);
        node
    }

    /// An encrypted segment in `dir` (the registry lives there too).
    fn encrypted_file(dir: &Path, name: &str) -> (File, std::path::PathBuf) {
        let crypto = VFileCrypto::open(dir, &material(0x11)).expect("registry");
        let path = dir.join(name);
        let file = File::open_with_crypto(path.clone(), 64 * 1024, false, Some(Arc::new(crypto)))
            .expect("encrypted segment");
        (file, path)
    }

    /// The f32 words of a payload.
    fn words(payload: &[u8]) -> Vec<f32> {
        payload
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }

    /// ADR-0057 W3: a node written to an encrypted segment round-trips.
    #[test]
    fn vfile_v3_node_roundtrip_encrypt_decrypt() {
        let dir = tempfile::tempdir().unwrap();
        let (mut file, _path) = encrypted_file(dir.path(), "vstore_L0.vanta");
        assert!(file.is_encrypted(), "a keyed segment is encrypted");

        let offset = write_node_to_vstore(&mut file, &node(42, vec![0.5, -1.25, 3.0])).unwrap();
        let record = file.read_node(offset).unwrap().expect("node readable");
        assert_eq!(record.header.id, 42);
        assert_eq!(record.header.vector_len, 3);
        assert_eq!(words(&record.payload), vec![0.5, -1.25, 3.0]);

        // The vector is NOT in the file in cleartext: the record is one AEAD
        // envelope.
        let bytes = file.mmap_bytes();
        let needle = 0.5f32.to_le_bytes();
        assert!(
            !bytes.windows(4).any(|w| w == needle),
            "the vector must not appear in cleartext on disk"
        );
        // The record starts with the length prefix and the nonce.
        let blob_len = u32::from_le_bytes([
            bytes[offset as usize],
            bytes[offset as usize + 1],
            bytes[offset as usize + 2],
            bytes[offset as usize + 3],
        ]);
        assert_eq!(
            blob_len as usize,
            64 + 3 * 4 + TAG_LEN,
            "the blob is the header, the payload and the tag"
        );
    }

    /// ADR-0057 W3: a v2 (plaintext) segment is read exactly as before, even
    /// by a build that has the `encryption` feature and a key.
    #[test]
    fn vfile_v2_segment_still_reads_on_a_v3_reader() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vstore_L0.vanta");

        // A segment written without a key: plaintext, version 2.
        let mut file = File::open(path.clone(), 64 * 1024).unwrap();
        assert!(!file.is_encrypted());
        let offset = write_node_to_vstore(&mut file, &node(7, vec![1.0, 2.0])).unwrap();
        file.flush().unwrap();
        drop(file);

        // The bytes on disk say v2 → the reader decides by the bytes, not by
        // the configuration: a key does not migrate the segment.
        let crypto = VFileCrypto::open(dir.path(), &material(0x22)).expect("registry");
        let reopened = File::open_with_crypto(path, 64 * 1024, false, Some(Arc::new(crypto)))
            .expect("a v2 segment opens on a v3 reader");
        assert!(!reopened.is_encrypted());
        let header = reopened.read_header(offset).expect("header readable");
        assert_eq!(header.id, 7);
        assert_eq!(header.vector_len, 2);
        let start = header.vector_offset as usize;
        let bytes = reopened.mmap_bytes();
        let end = start + 2 * 4;
        assert_eq!(words(&bytes[start..end]), vec![1.0, 2.0]);
    }

    /// ADR-0057 W3: a corrupted ciphertext is detected as a hard error, not
    /// as an empty node.
    #[test]
    fn vfile_v3_detects_a_corrupted_node_blob() {
        let dir = tempfile::tempdir().unwrap();
        let (mut file, path) = encrypted_file(dir.path(), "vstore_L0.vanta");
        let offset = write_node_to_vstore(&mut file, &node(11, vec![9.0, 9.0])).unwrap();
        file.flush().unwrap();
        drop(file);

        // Flip one bit inside the ciphertext.
        let mut bytes = std::fs::read(&path).unwrap();
        let target = offset as usize + RECORD_PREFIX_LEN + 4;
        bytes[target] ^= 0x01;
        std::fs::write(&path, &bytes).unwrap();

        let crypto = VFileCrypto::open(dir.path(), &material(0x11)).expect("registry");
        let file = File::open_with_crypto(path, 64 * 1024, false, Some(Arc::new(crypto)))
            .expect("the header is still valid");
        let err = file.read_node(offset).unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "a corrupted blob must be a hard error, got: {err}"
        );
    }

    /// ADR-0057 W3: a wrong key is a hard error, never an empty result; and a
    /// v3 segment without a key says "key required", never "corrupt".
    #[test]
    fn vfile_v3_wrong_key_is_a_hard_error_not_an_empty_result() {
        let dir = tempfile::tempdir().unwrap();
        let (mut file, path) = encrypted_file(dir.path(), "vstore_L0.vanta");
        write_node_to_vstore(&mut file, &node(3, vec![1.0])).unwrap();
        file.flush().unwrap();
        drop(file);

        // The wrong key cannot even unwrap the registry.
        let err = VFileCrypto::open(dir.path(), &material(0x99)).unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "a wrong key must be a hard error, got: {err}"
        );

        // No key at all: the segment says "encrypted" → key required.
        let err = File::open_with_crypto(path, 64 * 1024, false, None)
            .err()
            .expect("an encrypted segment without a key must not open");
        assert!(
            matches!(err, Error::EncryptionKeyRequired { .. }),
            "an encrypted segment without a key must ask for one, got: {err}"
        );
    }

    /// ADR-0057 W3: the AAD binds a record to its offset — a record moved to
    /// another offset does not authenticate.
    #[test]
    fn vfile_v3_node_blob_cannot_be_moved_to_another_offset() {
        let dir = tempfile::tempdir().unwrap();
        let (mut file, _path) = encrypted_file(dir.path(), "vstore_L0.vanta");
        let offset = write_node_to_vstore(&mut file, &node(5, vec![4.0, 5.0])).unwrap();
        let record_len = file.write_cursor - offset;

        // Copy the record to the next slot.
        let target = offset + record_len;
        let record = file.mmap_bytes()[offset as usize..(offset + record_len) as usize].to_vec();
        file.mmap_bytes_mut().unwrap()[target as usize..(target + record_len) as usize]
            .copy_from_slice(&record);

        // Read at the original offset: it authenticates.
        assert!(file.read_node(offset).unwrap().is_some());
        // Read at the new offset: the AAD (slot = offset) no longer matches.
        let err = file.read_node(target).unwrap_err();
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "a moved record must not authenticate, got: {err}"
        );
    }

    /// ADR-0057 W3: the compaction re-encrypts — the result is still an
    /// encrypted segment and it can be read again.
    #[test]
    fn vfile_v3_compaction_roundtrips_and_re_encrypts() {
        let dir = tempfile::tempdir().unwrap();
        let (mut file, path) = encrypted_file(dir.path(), "vstore_L0.vanta");
        let a = write_node_to_vstore(&mut file, &node(1, vec![1.0, 2.0])).unwrap();
        let b = write_node_to_vstore(&mut file, &node(2, vec![3.0, 4.0])).unwrap();
        file.flush().unwrap();

        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![1.0, 2.0]),
            a,
        )
        .expect("non-zero norm");
        hnsw.add(
            2,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![3.0, 4.0]),
            b,
        )
        .expect("non-zero norm");

        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        let (map, _size) =
            compact_layout(&mut file, &hnsw, &[1, 2], header_size).expect("compaction");
        assert_eq!(map.len(), 2);
        drop(file);

        // The compacted file is still an encrypted segment (v3 in the header).
        let bytes = std::fs::read(&path).unwrap();
        let header = Header::deserialize(&bytes[0..16]).expect("header");
        assert_eq!(header.magic, *b"VFLE");
        assert_eq!(
            header.format_version, VFILE_ENCRYPTED_VERSION,
            "the compaction must not fall back to plaintext"
        );

        // And every node can be read again.
        let crypto = VFileCrypto::open(dir.path(), &material(0x11)).expect("registry");
        let reopened =
            File::open_with_crypto(path, 64 * 1024, false, Some(Arc::new(crypto))).expect("reopen");
        assert!(reopened.is_encrypted());
        for (id, offset) in &map {
            let record = reopened.read_node(*offset).unwrap().expect("node");
            assert_eq!(record.header.id, *id);
            assert_eq!(record.header.vector_len, 2);
        }
    }

    /// ADR-0057 W3: the search finds the same neighbours over an encrypted
    /// segment as over a plaintext one — the recall does not change.
    #[test]
    fn vfile_v3_search_finds_the_same_neighbours() {
        use crate::node::ALL_BITSET;
        let vectors: Vec<Vec<f32>> = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.9, 0.1, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![-1.0, 0.0, 0.0],
        ];
        let dir = tempfile::tempdir().unwrap();

        // The plaintext reference.
        let plain_path = dir.path().join("plain.vanta");
        let mut plain = File::open(plain_path, 64 * 1024).unwrap();
        let plain_index = CPIndex::new();
        for (i, vector) in vectors.iter().enumerate() {
            let id = i as u128 + 1;
            let offset = write_node_to_vstore(&mut plain, &node(id, vector.clone())).unwrap();
            plain_index
                .add(
                    id,
                    FilterBitset::new(),
                    VectorRepresentations::Full(vector.clone()),
                    offset,
                )
                .expect("non-zero norm");
        }

        // The encrypted segment with the same content.
        let crypto = VFileCrypto::open(dir.path(), &material(0x11)).expect("registry");
        let enc_path = dir.path().join("enc.vanta");
        let mut enc =
            File::open_with_crypto(enc_path, 64 * 1024, false, Some(Arc::new(crypto))).unwrap();
        let enc_index = CPIndex::new();
        for (i, vector) in vectors.iter().enumerate() {
            let id = i as u128 + 1;
            let offset = write_node_to_vstore(&mut enc, &node(id, vector.clone())).unwrap();
            enc_index
                .add(
                    id,
                    FilterBitset::new(),
                    VectorRepresentations::Full(vector.clone()),
                    offset,
                )
                .expect("non-zero norm");
        }

        let query = vec![1.0, 0.0, 0.0];
        let plain_results =
            plain_index.search_nearest(&query, None, None, &ALL_BITSET, 4, Some(&plain));
        let enc_results = enc_index.search_nearest(&query, None, None, &ALL_BITSET, 4, Some(&enc));
        assert_eq!(plain_results.len(), enc_results.len());
        assert!(!plain_results.is_empty());
        for (p, e) in plain_results.iter().zip(enc_results.iter()) {
            assert_eq!(p.0, e.0, "the same neighbours, in the same order");
            assert!(
                (p.1 - e.1).abs() < 1e-6,
                "the same scores: {} vs {}",
                p.1,
                e.1
            );
        }
    }

    /// ADR-0057 W3: the tombstone write still works — in an encrypted segment
    /// the node is re-sealed instead of being patched in place.
    #[test]
    fn vfile_v3_tombstone_write_still_works() {
        let dir = tempfile::tempdir().unwrap();
        let (mut file, _path) = encrypted_file(dir.path(), "vstore_L0.vanta");
        let offset = write_node_to_vstore(&mut file, &node(8, vec![7.0, 8.0])).unwrap();

        assert!(file.set_tombstone(offset, true).expect("tombstone"));
        let header = file.read_header(offset).expect("header");
        assert_ne!(
            header.flags & NodeFlags::TOMBSTONE,
            0,
            "the node must be tombstoned"
        );
        // The vector is still readable (the node was re-sealed, not dropped).
        let record = file.read_node(offset).unwrap().expect("node");
        assert_eq!(words(&record.payload), vec![7.0, 8.0]);
        // The record kept its length: the following nodes do not move.
        assert_eq!(file.write_cursor, offset + file.node_stride(64, 16));
    }

    /// ADR-0057 W3: `grow_to` and the batch pre-allocation still work with
    /// the bigger encrypted records.
    #[test]
    fn vfile_v3_grow_to_and_prealloc_batch_still_work() {
        let dir = tempfile::tempdir().unwrap();
        let (mut file, _path) = encrypted_file(dir.path(), "vstore_L0.vanta");
        let size_before = file.size;
        crate::storage::engine::insert::prealloc_vstore_batch(&mut file, 64).expect("prealloc");
        assert!(file.size > size_before, "the segment grew");
        for id in 0..8u128 {
            let offset =
                write_node_to_vstore(&mut file, &node(id, vec![id as f32, 0.5, 0.25])).unwrap();
            let record = file.read_node(offset).unwrap().expect("node");
            assert_eq!(record.header.id, id);
            assert_eq!(words(&record.payload), vec![id as f32, 0.5, 0.25]);
        }
    }

    /// ADR-0057 W3: an engine opened with a key writes **encrypted**
    /// segments, and the data round-trips through the normal API.
    #[cfg(any(feature = "fjall", feature = "rocksdb"))]
    #[test]
    fn vfile_v3_engine_roundtrip_with_key() {
        use crate::config::Config;
        use crate::storage::StorageEngine;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let config = Config {
            storage_path: path.to_string(),
            backend_kind: crate::storage::BackendKind::Fjall,
            encryption_key: Some("ab".repeat(32)),
            ..Config::default()
        };
        let engine = StorageEngine::open_with_config(path, Some(config)).expect("open");

        let mut node = UnifiedNode::new(1234);
        node.vector = VectorRepresentations::Full(vec![0.25, 0.5, 0.75]);
        node.flags.set(NodeFlags::HAS_VECTOR);
        engine.insert(&node).expect("insert");

        let read = engine.get(1234).expect("get").expect("node present");
        let slice = read.vector.as_f32_slice().expect("a FULL vector");
        assert_eq!(slice, &[0.25, 0.5, 0.75]);

        // The segment on disk is encrypted: the vector is not in cleartext.
        let segment = dir.path().join("data").join("vstore_L0.vanta");
        let bytes = std::fs::read(&segment).expect("the segment exists");
        let needle = 0.25f32.to_le_bytes();
        assert!(
            !bytes.windows(4).any(|w| w == needle),
            "the vector must not appear in cleartext on disk"
        );
    }

    /// ADR-0057 W3 / D3: the nonce counter never wraps — when it is
    /// exhausted a **new epoch** is allocated (and persisted) instead.
    #[test]
    fn vfile_v3_nonce_epoch_overflow_allocates_a_new_epoch() {
        let dir = tempfile::tempdir().unwrap();
        let crypto = VFileCrypto::open(dir.path(), &material(0x11)).expect("registry");

        // Drive the counter of the current epoch to the edge: two nonces fit
        // in the old epoch, the third exhausts it.
        {
            let guard = crypto.allocator.as_ref().unwrap().load();
            guard.set_counter_for_test(u32::MAX - 2);
        }
        // Two nonces fit in the old epoch…
        let a = crypto.seal_node(64, b"a").expect("seal");
        let b = crypto.seal_node(128, b"b").expect("seal");
        assert_eq!(&a[4..12], &b[4..12], "both use the same epoch");
        assert_ne!(&a[12..16], &b[12..16], "the counter advances");
        // …the third exhausts the counter: a new epoch is rolled.
        let c = crypto.seal_node(192, b"c").expect("seal");
        assert_ne!(
            &c[4..12],
            &a[4..12],
            "the counter must never wrap: a new epoch is allocated"
        );
        // The epoch is persisted before its first use (the registry holds it
        // — W1's `registry_epoch_is_persisted_before_first_use` pins the
        // on-disk form).
        assert!(
            KeyRegistry::path_in(dir.path()).exists(),
            "the registry must exist on disk"
        );
    }

    /// ADR-0057 W3 / D1 rule 1: without a key the plaintext path is
    /// **byte-identical** to the layout before this change. The test rebuilds
    /// the expected v2 bytes and compares them with the file.
    #[test]
    fn vfile_v3_plaintext_path_is_byte_identical_without_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vstore_L0.vanta");
        let mut file = File::open(path.clone(), 4096).unwrap();
        let node = node(7, vec![1.5, -2.25]);
        let offset = write_node_to_vstore(&mut file, &node).unwrap();
        file.flush().unwrap();
        drop(file);

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(offset, 64, "the first node lives at the first aligned slot");

        // Rebuild the expected v2 layout byte by byte.
        let mut expected = vec![0u8; bytes.len()];
        expected[0..4].copy_from_slice(b"VFLE");
        expected[4..6].copy_from_slice(&VFILE_VERSION.to_le_bytes());
        // [6..8] is the schema version (0) and [8..16] the creation
        // timestamp — the only non-deterministic field of the header.
        // The cursor is 64-byte aligned: 64 (header) + 64 (payload) = 128,
        // plus 8 bytes of payload → 136 → aligned to 192.
        let cursor: u64 = 192;
        expected[16..24].copy_from_slice(&cursor.to_le_bytes());
        let mut header = DiskNodeHeader::new(7);
        header.vector_offset = 64 + 64;
        header.vector_len = 2;
        header.flags = NodeFlags::with_vector_kind(node.flags.0, NodeFlags::VECTOR_KIND_FULL);
        header.bitset = node.bitset.to_u128();
        header.confidence_score = node.confidence_score;
        header.importance = node.importance;
        expected[64..128].copy_from_slice(header.as_bytes());
        expected[128..132].copy_from_slice(&1.5f32.to_le_bytes());
        expected[132..136].copy_from_slice(&(-2.25f32).to_le_bytes());

        assert_eq!(&bytes[0..8], &expected[0..8], "magic and version");
        assert_eq!(
            &bytes[16..],
            &expected[16..],
            "the cursor, the node header and the payload must be byte-identical"
        );
    }

    /// FIND-618: `open_node` returns `Zeroizing<Vec<u8>>` so that AEAD-decrypted
    /// plaintext is safely wiped upon drop.
    #[test]
    fn vfile_v3_open_node_returns_zeroizing_plaintext() {
        use crate::crypto::Zeroize;
        let dir = tempfile::tempdir().unwrap();
        let crypto = VFileCrypto::open(dir.path(), &material(0x42)).expect("crypto");
        let payload = b"critical_plaintext_data_must_be_zeroized";
        let record = crypto.seal_node(64, payload).expect("seal");

        let mut opened = crypto.open_node(64, &record).expect("open_node");
        assert_eq!(&*opened, payload);

        // Explicitly zeroize to verify Zeroize trait behavior
        opened.zeroize();
        assert!(opened.iter().all(|&b| b == 0));
    }
}
