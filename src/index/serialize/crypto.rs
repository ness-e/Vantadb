//! Encryption at rest for the HNSW index image (ADR-0057 W5).
//!
//! The image is rewritten **whole** on every flush, so the envelope
//! granularity is **per file** (ADR-0057 D1, option B):
//!
//! ```text
//! [16B header: "VNDX" | format_version | schema_version | timestamp]
//! [12B nonce]
//! [ct ‖ 16B tag]
//! ```
//!
//! * The cleartext header is the **mode marker**: `format_version = 8` is the
//!   plaintext image (byte-identical to the layout before this change) and
//!   `format_version = 9` is the encrypted one. The reader decides by the
//!   **bytes on disk**, never by the configuration (ADR-0057 D1/D7).
//! * The **AAD** binds the image to its **path relative to the storage root**
//!   (`data/vector_index.bin`), so an image moved to another place does not
//!   authenticate (ADR-0057 D4).
//! * The nonce is `epoch ‖ counter`; the epoch is persisted **before** the
//!   first encryption under it (ADR-0057 D3).
//!
//! # Zero-copy (ADR-0057 R-3)
//!
//! The cold start of an encrypted image **cannot** be zero-copy: a ciphertext
//! is not a `&[f32]`. The image is read and decrypted **whole** into RAM —
//! the file is rewritten whole on every flush, so this is a whole-file cost
//! paid **once per open**, not a per-query one. The memory cost is bounded
//! by the image size (the ciphertext buffer is released before the index is
//! built), and the *decrypted* payload is the same bytes the plaintext path
//! would have mapped.
//!
//! One context exists per **database**: the DEK and the nonce space belong
//! to the `hnsw` artefact, so the image of a database shares them (the same
//! pattern as the `vfile` artefact in W3).

#[cfg(feature = "encryption")]
use crate::error::{Error, Result};

#[cfg(feature = "encryption")]
use crate::crypto::registry::{AadBuilder, Artifact, KeyRegistry, NonceAllocator};
#[cfg(feature = "encryption")]
use crate::crypto::{Cipher, CryptoError, Nonce};
#[cfg(feature = "encryption")]
use parking_lot::Mutex;
#[cfg(feature = "encryption")]
use std::path::Path;
#[cfg(feature = "encryption")]
use std::sync::Arc;

/// The artefact id used in the error messages of this module.
#[cfg(feature = "encryption")]
pub(crate) const ARTIFACT: &str = "hnsw";

/// Length of the AEAD nonce (ADR-0057 D3).
#[cfg(feature = "encryption")]
pub(crate) const NONCE_LEN: usize = 12;

/// Length of the AEAD tag.
#[cfg(feature = "encryption")]
pub(crate) const TAG_LEN: usize = 16;

/// The encryption context of the `hnsw` artefact (shared by every image of
/// the database).
#[cfg(feature = "encryption")]
pub(crate) type HnswImageCryptoContext = Arc<HnswImageCrypto>;
/// Without the `encryption` feature there is no context at all.
#[cfg(not(feature = "encryption"))]
pub(crate) type HnswImageCryptoContext = ();

/// The encryption context carried by an index.
///
/// It is resolved **once** — at the cold start of the image or, for a fresh
/// index, at the first write — and then kept for the life of the index: the
/// nonce counter lives in it (ADR-0057 D3).
///
/// With the `encryption` feature it holds the real [`HnswImageCrypto`];
/// without it the slot stays empty and every image remains plaintext
/// (byte-identical to the layout before this change).
pub(crate) type HnswImageCryptoSlot = std::sync::OnceLock<Option<HnswImageCryptoContext>>;

/// The resolved context of an index, if any.
#[cfg(feature = "encryption")]
pub(crate) fn resolved(slot: &HnswImageCryptoSlot) -> Option<HnswImageCryptoContext> {
    slot.get().and_then(|ctx| ctx.clone())
}

/// Keep the resolved context: the first writer wins (a concurrent resolution
/// would allocate a second epoch — safe, but wasteful).
#[cfg(feature = "encryption")]
pub(crate) fn store(slot: &HnswImageCryptoSlot, ctx: Option<HnswImageCryptoContext>) {
    let _ = slot.set(ctx);
}

/// The **write side** of an encrypted image: the DEK of the `hnsw` artefact,
/// the shared nonce allocator and the registry handle needed to roll the
/// epoch.
#[cfg(feature = "encryption")]
pub(crate) struct HnswImageCrypto {
    /// The DEK cipher of the artefact.
    cipher: Cipher,
    /// The nonce allocator of the current epoch. `None` in a read-only
    /// context: a read-only open never allocates an epoch, because that
    /// would write the registry.
    allocator: Option<arc_swap::ArcSwapAny<Arc<NonceAllocator>>>,
    /// The registry — locked only to roll the epoch (never in the hot path).
    registry: Option<Arc<Mutex<KeyRegistry>>>,
    /// The public identifier of the image: its path relative to the storage
    /// root. It travels in the AAD (ADR-0057 D4).
    identifier: Vec<u8>,
}

// The context holds no secret in a `Debug` view: the `Cipher` redacts its
// key material and the registry redacts the DEKs.
#[cfg(feature = "encryption")]
impl std::fmt::Debug for HnswImageCrypto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HnswImageCrypto")
            .field("writable", &self.allocator.is_some())
            .field("identifier", &String::from_utf8_lossy(&self.identifier))
            .finish()
    }
}

#[cfg(feature = "encryption")]
impl HnswImageCrypto {
    /// Open the **write-side** context of the `hnsw` artefact in `data_dir`.
    ///
    /// Allocates a **fresh epoch** and persists it **before** the first
    /// encryption under it (ADR-0057 D3).
    ///
    /// # Errors
    ///
    /// [`Error::EncryptionKeyRejected`] when the master key does not unwrap
    /// the registry (wrong key **or** tampered registry) and
    /// [`Error::EncryptionCorrupt`] when the registry is malformed.
    pub(crate) fn open(data_dir: &Path, key_material: &[u8], identifier: Vec<u8>) -> Result<Self> {
        let mut registry =
            KeyRegistry::open_or_create(data_dir, key_material).map_err(crypto_error)?;
        registry
            .ensure_dek(Artifact::HnswImage)
            .map_err(crypto_error)?;
        registry
            .begin_epoch(Artifact::HnswImage)
            .map_err(crypto_error)?;
        let allocator = registry
            .allocator(Artifact::HnswImage)
            .cloned()
            .ok_or_else(|| {
                Error::generic_error(
                    "encryption registry has no nonce allocator for the hnsw artefact",
                )
            })?;
        let dek =
            registry
                .dek(Artifact::HnswImage)
                .ok_or_else(|| Error::EncryptionKeyRequired {
                    artifact: ARTIFACT.to_string(),
                })?;
        Ok(Self {
            cipher: Cipher::new(&dek[..]),
            allocator: Some(arc_swap::ArcSwapAny::new(allocator)),
            registry: Some(Arc::new(Mutex::new(registry))),
            identifier,
        })
    }

    /// Open the **read-side** context of the `hnsw` artefact in `data_dir`.
    ///
    /// A read-only open must not write anything, so it neither allocates an
    /// epoch nor persists the registry.
    ///
    /// # Errors
    ///
    /// [`Error::EncryptionKeyRequired`] when the registry is missing (the DEK
    /// only lives there) and [`Error::EncryptionKeyRejected`] when the master
    /// key does not unwrap it.
    pub(crate) fn open_read(
        data_dir: &Path,
        key_material: &[u8],
        identifier: Vec<u8>,
    ) -> Result<Self> {
        if !KeyRegistry::path_in(data_dir).exists() {
            return Err(Error::EncryptionKeyRequired {
                artifact: ARTIFACT.to_string(),
            });
        }
        let registry = KeyRegistry::open_or_create(data_dir, key_material).map_err(crypto_error)?;
        let dek =
            registry
                .dek(Artifact::HnswImage)
                .ok_or_else(|| Error::EncryptionKeyRequired {
                    artifact: ARTIFACT.to_string(),
                })?;
        Ok(Self {
            cipher: Cipher::new(&dek[..]),
            allocator: None,
            registry: None,
            identifier,
        })
    }

    /// Seal the payload of an image: `[12B nonce][ct ‖ 16B tag]`.
    ///
    /// The **AAD** binds the envelope to the image's path relative to the
    /// storage root (slot = 0 — the image is one single envelope), so an
    /// image moved to another place does not authenticate (ADR-0057 D4).
    ///
    /// # Errors
    ///
    /// [`Error::EncryptionKeyRequired`] when the context is read-only and
    /// [`Error::EncryptionCorrupt`] when the registry cannot be written.
    pub(crate) fn seal_image(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let nonce = self.next_nonce()?;
        let aad = AadBuilder::new(
            Artifact::HnswImage,
            crate::index::graph::VECTOR_INDEX_ENCRYPTED_VERSION,
        )
        .epoch(nonce.epoch())
        .slot(0)
        .plaintext_len(payload.len())
        .identifier(&self.identifier)
        .build();
        // `ct` is the AEAD blob: `ciphertext ‖ 16-byte tag`.
        let ct = self
            .cipher
            .encrypt_with_nonce(nonce.as_bytes(), payload, &aad)
            .map_err(crypto_error)?;
        let mut envelope = Vec::with_capacity(NONCE_LEN + ct.len());
        envelope.extend_from_slice(nonce.as_bytes());
        envelope.extend_from_slice(&ct);
        Ok(envelope)
    }

    /// Open an image envelope: the payload it was sealed from.
    ///
    /// # Errors
    ///
    /// [`Error::EncryptionCorrupt`] when the envelope is structurally broken
    /// (truncated) and [`Error::EncryptionKeyRejected`] when the AEAD tag does
    /// not authenticate (wrong key **or** tampered ciphertext — a single tag
    /// cannot tell them apart).
    pub(crate) fn open_image(&self, envelope: &[u8]) -> Result<Vec<u8>> {
        if envelope.len() < NONCE_LEN + TAG_LEN {
            return Err(corrupt(format!(
                "the HNSW image envelope is truncated ({} bytes)",
                envelope.len()
            )));
        }
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&envelope[..NONCE_LEN]);
        let ct = &envelope[NONCE_LEN..];
        let aad = AadBuilder::new(
            Artifact::HnswImage,
            crate::index::graph::VECTOR_INDEX_ENCRYPTED_VERSION,
        )
        .epoch(Nonce::from_bytes(nonce).epoch())
        .slot(0)
        .plaintext_len(ct.len() - TAG_LEN)
        .identifier(&self.identifier)
        .build();
        self.cipher
            .decrypt_with_nonce(&nonce, ct, &aad)
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
                "the hnsw encryption context is read-only: it cannot seal an image",
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
                "the hnsw encryption context is read-only: it cannot roll the nonce epoch",
            ));
        };
        let mut registry = registry.lock();
        // Another writer may have rolled the epoch already: adopt its
        // allocator instead of burning a second epoch — the artefact keeps a
        // single, shared nonce sequence.
        if let Some(current) = registry.allocator(Artifact::HnswImage) {
            let mine = allocator.load();
            if !Arc::ptr_eq(current, &*mine) {
                allocator.store(current.clone());
                return Ok(());
            }
        }
        registry
            .begin_epoch(Artifact::HnswImage)
            .map_err(crypto_error)?;
        let fresh = registry
            .allocator(Artifact::HnswImage)
            .cloned()
            .ok_or_else(|| {
                Error::generic_error(
                    "encryption registry has no nonce allocator for the hnsw artefact",
                )
            })?;
        allocator.store(fresh);
        Ok(())
    }
}

/// The public identifier of the image: its path relative to the storage root.
///
/// `data_dir` is the storage root's `data` directory and the image is
/// `data/vector_index.bin` → the identifier is `data/vector_index.bin`.
/// It is **not** a secret: it is public metadata bound into the AAD so an
/// image moved to another place does not authenticate (ADR-0057 D4).
#[cfg(feature = "encryption")]
pub(crate) fn image_identifier(data_dir: &Path, image_path: &Path) -> Vec<u8> {
    let root = data_dir.parent().unwrap_or(data_dir);
    let relative = image_path.strip_prefix(root).unwrap_or(image_path);
    let mut out = Vec::new();
    for (i, component) in relative.components().enumerate() {
        if i > 0 {
            out.push(b'/');
        }
        out.extend_from_slice(component.as_os_str().to_string_lossy().as_bytes());
    }
    out
}

/// Resolve the master key material from the **environment**.
///
/// Precedence: environment → key file → key command (ADR-0057 D2). Returns
/// `None` when no source resolves: the image then stays **plaintext and
/// byte-identical** to the layout before this change.
///
/// The material is resolved **once** (at the open of the image): a KMS shim
/// must not run on every flush.
#[cfg(feature = "encryption")]
pub(crate) fn key_material_from_env() -> Result<Option<crate::crypto::Zeroizing<Vec<u8>>>> {
    let config = crate::config::Config::default();
    let env_key = config
        .encryption_key
        .clone()
        .or_else(|| std::env::var("VANTADB_ENCRYPTION_KEY").ok());
    match crate::crypto::resolve_master_key_material(
        env_key.as_deref(),
        config.encryption_key_file.as_deref(),
        config.encryption_key_cmd.as_deref(),
    ) {
        Ok(material) => Ok(Some(material)),
        Err(CryptoError::KeyNotSet) => Ok(None),
        Err(err) => Err(crypto_error(err)),
    }
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
        other => Error::generic_error(format!("HNSW image encryption error: {other}")),
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

// ─── Tests (ADR-0057 W5) ─────────────────────────────────────

#[cfg(all(test, feature = "encryption"))]
#[allow(missing_docs)]
mod tests {
    use super::*;

    /// A 32-byte master key.
    fn material(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    /// ADR-0057 W5 / D3: the nonce counter never wraps — when it is
    /// exhausted a **new epoch** is allocated (and persisted) instead.
    #[test]
    fn hnsw_nonce_epoch_overflow_allocates_a_new_epoch() {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let image = data_dir.join("vector_index.bin");
        let crypto = HnswImageCrypto::open(
            &data_dir,
            &material(0x88),
            image_identifier(&data_dir, &image),
        )
        .expect("the context opens");

        // Drive the counter of the current epoch to the edge: two nonces fit
        // in the old epoch, the third exhausts it.
        {
            let guard = crypto.allocator.as_ref().unwrap().load();
            guard.set_counter_for_test(u32::MAX - 2);
        }
        let a = crypto.seal_image(b"a").expect("seal");
        let b = crypto.seal_image(b"b").expect("seal");
        assert_eq!(&a[..8], &b[..8], "both use the same epoch");
        assert_ne!(&a[8..12], &b[8..12], "the counter advances");
        // The third exhausts the counter: a new epoch is rolled.
        let c = crypto.seal_image(b"c").expect("seal");
        assert_ne!(
            &c[..8],
            &a[..8],
            "the counter must never wrap: a new epoch is allocated"
        );
        // The epoch is persisted before its first use.
        assert!(
            KeyRegistry::path_in(&data_dir).exists(),
            "the registry must exist on disk"
        );
    }

    /// ADR-0057 W5 / D4: the identifier is the image's path relative to the
    /// storage root — it is public metadata, bound into the AAD.
    #[test]
    fn hnsw_image_identifier_is_the_path_relative_to_the_storage_root() {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().join("data");
        let image = data_dir.join("vector_index.bin");
        assert_eq!(
            image_identifier(&data_dir, &image),
            b"data/vector_index.bin".to_vec()
        );
    }
}
