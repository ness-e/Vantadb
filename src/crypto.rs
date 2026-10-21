// ponytail: AES-GCM/Aes256Gcm/PBKDF2 calls below return errors only for inputs
// that we statically know cannot fail (32-byte key, fixed nonces, infallible
// cipher). Spreading a per-call `#[allow]` would add 6+ attribute lines for
// zero behavior change. Blanket allow with this rationale is the simpler
// contract — see individual `expect` strings for the specific invariant.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! AES-256-GCM primitives and envelope format for VantaDB.
//!
//! Provides [`Cipher`] for encrypt/decrypt operations and [`EncryptionStream`]
//! for transparent Read/Write wrapping of storage streams.
//!
//! **Scope (honest, 2026-10-20):** these primitives are wired into the
//! `vanta-proxy` envelope. They are **not** applied to the core storage files
//! yet (WAL, HNSW, VantaFile, backend-KV, snapshots stay plaintext) — the only
//! call site of `with_cipher` in `src/` is a test. The wiring is designed in
//! `ADR-0057` and tracked as `FIND-249`; see `docs/user/operations/CONFIGURATION.md`
//! (`encryption_key`) for the user-facing statement.
//!
//! # At-rest primitives (ADR-0057, wave W1)
//!
//! This module also carries the **shared infrastructure** the at-rest wiring
//! needs, still without touching any storage write path:
//!
//! * [`registry`] — the per-artefact DEK registry (master → KEK → DEK), the
//!   `epoch ‖ counter` nonce allocator and the AAD builder.
//! * [`Cipher::encrypt_with_nonce`] / [`Cipher::decrypt_with_nonce`] — AEAD
//!   with a **caller-chosen nonce** and **associated data**.
//! * [`Cipher::derive_kek`] — HKDF-SHA256 with a **caller-supplied salt**.
//! * [`KeySource`] — the three master-key sources (environment → file → command).
//!
//! The existing [`Cipher::encrypt`] / [`Cipher::decrypt`] keep their exact
//! behaviour (random nonce, empty AAD) so `vanta-proxy` and `vanta-memory` are
//! unaffected.
//!
//! # Key Derivation
//!
//! The encryption key is loaded from the `VANTADB_ENCRYPTION_KEY` environment
//! variable as a hex-encoded 32-byte (64 hex char) value.
//!
//! * A value that decodes to exactly 32 bytes is used directly as the AES-256
//! key. This is the recommended form (a real random key, no derivation
//! needed) and keeps the original message format.
//! * A shorter value (e.g. a user passphrase) is key-stretched with
//! PBKDF2-HMAC-SHA256 (210,000 iterations — OWASP 2023 recommendation) and a
//!   per-message random 16-byte salt before use. This defeats offline brute
//!   force on leaked ciphertext, unlike the previous single-pass SHA-256.
//!
//! # Message Format
//!
//! Each encrypted message on the wire is framed as:
//! `[4-byte LE payload length][12-byte nonce][AEAD ciphertext + 16-byte tag]`
//!
//! When the key is derived via PBKDF2 the payload is prefixed with a framing
//! header so decryption knows how to reconstruct the key:
//! `[1-byte version=0x01][16-byte salt][12-byte nonce][AEAD ciphertext + 16-byte tag]`
//!
//! # Backward Compatibility
//!
//! Messages written by older versions that used a raw 32-byte key, or the old
//! single-pass SHA-256 fallback for short keys, remain decryptable: if the
//! PBKDF2-framed path fails to authenticate (or the version marker is absent),
//! decryption falls back to the legacy scheme.

use crate::config::Config;
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce as AesNonce,
};
use rand::RngCore;
use ring::pbkdf2;
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};

/// Key registry, nonce allocator and AAD builder for at-rest encryption
/// (ADR-0057 §D2–§D4).
pub mod registry;

pub use registry::{AadBuilder, Artifact, KeyRegistry, Nonce};

/// Length of the AES-256-GCM authentication tag.
const TAG_LEN: usize = 16;

/// Errors specific to cryptographic operations.
///
/// `#[non_exhaustive]`: the set of variants grows with the at-rest wiring
/// (ADR-0057 C-7) — adding one is additive, never breaking.
#[derive(Debug)]
#[non_exhaustive]
pub enum CryptoError {
    /// The `VANTADB_ENCRYPTION_KEY` environment variable is not set.
    KeyNotSet,
    /// The key string is malformed or cannot be decoded.
    InvalidKey(String),
    /// The ciphertext is too short or malformed.
    InvalidCiphertext(String),
    /// Decryption failed (wrong key, corrupted data, or tampered ciphertext).
    DecryptionFailed,
    /// The key **source** could not be read (missing file, wrong permissions,
    /// failing command). The payload never contains key material.
    KeySourceFailed(String),
    /// The nonce counter of the current epoch is exhausted. The caller must
    /// allocate a **new epoch** — the counter never wraps (ADR-0057 D3).
    NonceEpochExhausted,
    /// The encryption registry is malformed (bad magic, truncated, bad CRC).
    RegistryInvalid(String),
    /// The encryption registry could not be read or written.
    RegistryIo(String),
    /// No nonce epoch has been allocated for the requested artefact yet.
    NoEpoch(String),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::KeyNotSet => write!(f, "VANTADB_ENCRYPTION_KEY not set"),
            CryptoError::InvalidKey(msg) => write!(f, "invalid encryption key: {msg}"),
            CryptoError::InvalidCiphertext(msg) => write!(f, "invalid ciphertext: {msg}"),
            CryptoError::DecryptionFailed => {
                write!(f, "decryption failed (wrong key or tampered ciphertext)")
            }
            CryptoError::KeySourceFailed(msg) => write!(f, "encryption key source failed: {msg}"),
            CryptoError::NonceEpochExhausted => {
                write!(f, "nonce counter exhausted for the current epoch")
            }
            CryptoError::RegistryInvalid(msg) => {
                write!(f, "invalid encryption registry: {msg}")
            }
            CryptoError::RegistryIo(msg) => {
                write!(f, "encryption registry I/O error: {msg}")
            }
            CryptoError::NoEpoch(artifact) => {
                write!(f, "no nonce epoch allocated for {artifact}")
            }
        }
    }
}

impl std::error::Error for CryptoError {}

/// Wipes a value's bytes when it goes out of scope.
///
/// The crate pins no `zeroize` dependency, so this is the local wrapper
/// ADR-0057 C-5 allows: `fill(0)` plus a `std::hint::black_box` barrier, so
/// the compiler cannot elide the wipe as dead code.
pub trait Zeroize {
    /// Overwrite the value's bytes with zeroes.
    fn zeroize(&mut self);
}

impl Zeroize for [u8] {
    fn zeroize(&mut self) {
        self.fill(0);
        // `black_box` forces the compiler to materialise the buffer, so the
        // memset above cannot be optimised away as a dead store.
        std::hint::black_box(self);
    }
}

impl Zeroize for [u8; 32] {
    fn zeroize(&mut self) {
        self.as_mut_slice().zeroize();
    }
}

impl Zeroize for Vec<u8> {
    fn zeroize(&mut self) {
        self.as_mut_slice().zeroize();
    }
}

/// A holder that wipes its contents on drop (ADR-0057 C-5).
///
/// Used for the master key and the per-artefact DEKs: their in-memory copies
/// must not outlive the process in a readable form.
#[derive(Clone)]
pub struct Zeroizing<T: Zeroize>(T);

impl<T: Zeroize> Zeroizing<T> {
    /// Wrap a value so it is wiped on drop.
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: Zeroize> std::fmt::Debug for Zeroizing<T> {
    /// Redacted: a `Debug` print must never leak key material.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Zeroizing(<redacted>)")
    }
}

impl<T: Zeroize> std::ops::Deref for Zeroizing<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Zeroize> Drop for Zeroizing<T> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Specifies how the encryption key is sourced.
///
/// The three sources are tried in a **fixed order** (ADR-0057 D2):
/// environment → key file → key command. The first one that resolves wins.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum KeySource {
    /// Read from the `VANTADB_ENCRYPTION_KEY` environment variable.
    Environment,
    /// Read from a file holding the hex-encoded key.
    ///
    /// On Unix the file must be a regular file (never a symlink) whose mode
    /// grants no group/other access (`0600`-equivalent). On Windows the check
    /// is limited to "regular file, non-empty" — ACL inspection is out of scope.
    File(std::path::PathBuf),
    /// Read from a command whose stdout is the hex-encoded key
    /// (KMS / secret-manager shim: `aws kms decrypt ...`).
    ///
    /// The command runs **once at open**; its output is captured into a
    /// zeroizing buffer and is never logged.
    Command(String),
}

impl KeySource {
    /// Read the raw key material from this source.
    ///
    /// The material is hex-decoded and returned in a zeroizing holder. It is
    /// never logged and never included in an error message.
    pub fn read_key_material(&self) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
        match self {
            KeySource::Environment => {
                let encoded = Config::default()
                    .encryption_key
                    .ok_or(CryptoError::KeyNotSet)?;
                let key = decode_hex(&encoded)?;
                Ok(Zeroizing::new(key))
            }
            KeySource::File(path) => {
                let key = read_key_file(path)?;
                Ok(Zeroizing::new(key))
            }
            KeySource::Command(cmd) => {
                let key = run_key_command(cmd)?;
                Ok(Zeroizing::new(key))
            }
        }
    }

    /// Resolve the master cipher from this source.
    ///
    /// A 32-byte material is used as the raw AES-256 key; a shorter one is a
    /// passphrase and becomes a PBKDF2-stretched key (see
    /// [`Cipher::new`]).
    pub fn read_cipher(&self) -> Result<Cipher, CryptoError> {
        let material = self.read_key_material()?;
        Ok(Cipher::new(&material))
    }
}

/// Resolve the master key from the three sources, in the fixed precedence
/// **environment → key file → key command** (ADR-0057 D2).
///
/// The environment wins because it is the historical contract; the file is
/// preferred over the command for production (a `0600` file is inspectable,
/// a KMS shim is not). The first source that resolves wins — the others are
/// not even read.
pub fn resolve_master_key(
    env_key: Option<&str>,
    key_file: Option<&std::path::Path>,
    key_cmd: Option<&str>,
) -> Result<Cipher, CryptoError> {
    let material = resolve_master_key_material(env_key, key_file, key_cmd)?;
    Ok(Cipher::new(&material))
}

/// Resolve the master key **material** from the three sources, in the fixed
/// precedence **environment → key file → key command** (ADR-0057 D2).
///
/// Unlike [`resolve_master_key`], this returns the **raw material** (hex
/// decoded) instead of a [`Cipher`]. The at-rest registry needs the material
/// itself: it stretches a passphrase **once** with the database salt
/// (ADR-0057 C-4), and a `Cipher` built from a passphrase has no recoverable
/// raw key (`raw_key_bytes` refuses it). The material is returned in a
/// zeroizing holder and is never logged.
pub fn resolve_master_key_material(
    env_key: Option<&str>,
    key_file: Option<&std::path::Path>,
    key_cmd: Option<&str>,
) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if let Some(encoded) = env_key {
        return Ok(Zeroizing::new(decode_hex(encoded)?));
    }
    if let Some(path) = key_file {
        return Ok(Zeroizing::new(read_key_file(path)?));
    }
    if let Some(cmd) = key_cmd {
        return Ok(Zeroizing::new(run_key_command(cmd)?));
    }
    Err(CryptoError::KeyNotSet)
}

/// Read the hex master key from a file, enforcing the Unix permission rules
/// (ADR-0057 D2): regular file, never a symlink, `0600`-equivalent mode.
fn read_key_file(path: &std::path::Path) -> Result<Vec<u8>, CryptoError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::symlink_metadata(path).map_err(|e| {
            CryptoError::KeySourceFailed(format!("key file {}: {e}", path.display()))
        })?;
        if meta.file_type().is_symlink() {
            return Err(CryptoError::KeySourceFailed(format!(
                "key file {} is a symlink; refusing to follow it",
                path.display()
            )));
        }
        if !meta.is_file() {
            return Err(CryptoError::KeySourceFailed(format!(
                "key file {} is not a regular file",
                path.display()
            )));
        }
        if meta.permissions().mode() & 0o077 != 0 {
            return Err(CryptoError::KeySourceFailed(format!(
                "key file {} is accessible by group or other; restrict it to 0600",
                path.display()
            )));
        }
    }
    #[cfg(not(unix))]
    {
        let meta = std::fs::metadata(path).map_err(|e| {
            CryptoError::KeySourceFailed(format!("key file {}: {e}", path.display()))
        })?;
        if !meta.is_file() {
            return Err(CryptoError::KeySourceFailed(format!(
                "key file {} is not a regular file",
                path.display()
            )));
        }
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| CryptoError::KeySourceFailed(format!("key file {}: {e}", path.display())))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(CryptoError::KeySourceFailed(format!(
            "key file {} is empty",
            path.display()
        )));
    }
    decode_hex(trimmed)
        .map_err(|e| CryptoError::KeySourceFailed(format!("key file {}: {e}", path.display())))
}

/// Run a key command through the platform shell and hex-decode its stdout.
///
/// The command comes from the operator's own environment, so the shell is the
/// expected execution context (`aws kms decrypt --key-id ...`). Its stdout is
/// captured into a zeroizing buffer and **never logged**; a failing command
/// reports only its exit status.
fn run_key_command(cmd: &str) -> Result<Vec<u8>, CryptoError> {
    let output = run_shell(cmd)
        .map_err(|e| CryptoError::KeySourceFailed(format!("key command could not run: {e}")))?;
    if !output.status.success() {
        return Err(CryptoError::KeySourceFailed(format!(
            "key command exited with status {}",
            output.status
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(CryptoError::KeySourceFailed(
            "key command produced no output".into(),
        ));
    }
    decode_hex(trimmed).map_err(|e| {
        CryptoError::KeySourceFailed(format!("key command output is not a hex key: {e}"))
    })
}

/// Execute a command line through the platform shell.
fn run_shell(cmd: &str) -> std::io::Result<std::process::Output> {
    #[cfg(windows)]
    {
        std::process::Command::new("cmd").args(["/C", cmd]).output()
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("sh").args(["-c", cmd]).output()
    }
}

/// Configuration for storage-layer encryption.
///
/// The three sources are tried in the fixed order of ADR-0057 D2:
/// environment → key file → key command.
#[derive(Debug, Clone, Default)]
pub struct EncryptionConfig {
    /// Source 1 — the `VANTADB_ENCRYPTION_KEY` environment variable (hex).
    /// `None` means "read the environment variable".
    pub env_key: Option<String>,
    /// Source 2 — a file holding the hex master key.
    pub key_file: Option<std::path::PathBuf>,
    /// Source 3 — a command whose stdout is the hex master key.
    pub key_cmd: Option<String>,
}

impl EncryptionConfig {
    /// Resolve the cipher from this configuration.
    ///
    /// When [`EncryptionConfig::env_key`] is `None` the
    /// `VANTADB_ENCRYPTION_KEY` environment variable is used (the historical
    /// contract); the file and the command are the fallbacks.
    pub fn resolve_cipher(&self) -> Result<Cipher, CryptoError> {
        let env_key = self
            .env_key
            .clone()
            .or_else(|| Config::default().encryption_key);
        resolve_master_key(
            env_key.as_deref(),
            self.key_file.as_deref(),
            self.key_cmd.as_deref(),
        )
    }
}

/// Attributes of the PBKDF2 key-stretching scheme used for short keys.
///
/// The version marker is the first byte of a derived-key message and lets
/// decryption distinguish it from a legacy message produced by older versions.
const KDF_FRAME_VERSION: u8 = 0x01;
/// PBKDF2 salt length in bytes (16 = 128-bit salt).
const KDF_SALT_LEN: usize = 16;
/// PBKDF2-HMAC-SHA256 iteration count (OWASP 2023 recommendation for SHA-256).
pub(crate) const KDF_ITERATIONS: u32 = 210_000;
/// `[version(1)][salt(16)][nonce(12)][tag(16)]` — minimum length of a
/// PBKDF2-framed message (empty plaintext).
const KDF_FRAME_MIN: usize = 1 + KDF_SALT_LEN + 12 + 16;

/// Upper bound on a single encrypted frame length (plaintext + nonce + tag).
///
/// The reader trusts a 4-byte LE length from the wire; without a bound a
/// corrupt/hostile header could ask for an allocation of up to `u32::MAX`
/// (4 GiB) and OOM the process. Frames are written only by
/// [`EncryptionStream::write`], which holds one chunk per frame — realistic
/// sizes are far below this. 512 MiB is 8× the practical maximum and safely
/// rejects forged lengths before allocation.
///
/// This is separate from [`KDF_FRAME_MIN`] which is a *minimum* for the
/// PBKDF2-framed payload.
const MAX_FRAME_LEN: usize = 512 * 1024 * 1024;

/// HKDF salt for namespace envelope key derivation (VER-03).
///
/// Domain-separates derived namespace keys from the at-rest storage key usage
/// (same primitive, different purpose).
const NS_ENVELOPE_SALT: &[u8] = b"vanta-namespace-envelope-v1";
/// HKDF info marker prefixed to the namespace in the derivation.
const NS_ENVELOPE_INFO: &[u8] = b"ns";
/// HKDF info marker of the at-rest key-encryption key (ADR-0057 D2).
///
/// Domain-separates the KEK from the namespace envelopes and from the DEK
/// wrapping, so no two derivations in the crate can collide.
const KEK_INFO: &[u8] = b"vanta-kek-v1";

/// 32-byte OKM marker for [`Cipher::derive_namespace`]
/// (`ring::hkdf::KeyType`; ring does not implement it for `usize`).
struct Aes256KeyType;

impl ring::hkdf::KeyType for Aes256KeyType {
    fn len(&self) -> usize {
        32
    }
}

/// AES-256-GCM cipher wrapping [`Aes256Gcm`] with a 12-byte nonce.
///
/// Each encryption generates a fresh random nonce from the OS CSPRNG
/// via `rand::rng()` (thread-local CSPRNG, same entropy source the removed
/// `AeadCore::generate_nonce` used).
///
/// The key material lives in a [`Zeroizing`] holder, so it is wiped when the
/// cipher is dropped (ADR-0057 C-5). The cipher stays **immutable and
/// stateless**: it holds no counter, no epoch and no mutable field.
#[derive(Clone)]
pub struct Cipher {
    /// Raw 32-byte AES key. `Some` when the caller supplied exactly 32 bytes;
    /// the legacy `[nonce][ct+tag]` framing is used, unchanged from v0.4.
    raw_key: Option<Zeroizing<[u8; 32]>>,
    /// Passphrase to key-stretch via PBKDF2. `Some` when the caller supplied a
    /// short key (fewer than 32 bytes); messages are PBKDF2-framed.
    password: Option<Zeroizing<Vec<u8>>>,
}

impl std::fmt::Debug for Cipher {
    /// Redacted: a `Debug` print must never leak the key material.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cipher")
            .field(
                "key",
                &if self.raw_key.is_some() {
                    "raw-32-bytes"
                } else {
                    "passphrase"
                },
            )
            .finish()
    }
}

impl Cipher {
    /// Create a new cipher from a raw key byte slice.
    ///
    /// A 32-byte key is used directly (recommended — a real random key needs
    /// no derivation). Any other length is treated as a passphrase and
    /// key-stretched with PBKDF2-HMAC-SHA256 (210,000 iterations) plus a random
    /// per-message salt to resist offline brute force.
    pub fn new(key: &[u8]) -> Self {
        if key.len() == 32 {
            let mut k = [0u8; 32];
            k.copy_from_slice(key);
            Self {
                raw_key: Some(Zeroizing::new(k)),
                password: None,
            }
        } else {
            Self {
                raw_key: None,
                password: Some(Zeroizing::new(key.to_vec())),
            }
        }
    }

    /// Load the key from the `VANTADB_ENCRYPTION_KEY` environment variable.
    ///
    /// Expects a hex-encoded 32-byte key (64 hex characters, optional `0x` prefix).
    pub fn from_env() -> Result<Self, CryptoError> {
        let encoded = Config::default()
            .encryption_key
            .ok_or(CryptoError::KeyNotSet)?;
        let key = decode_hex(&encoded)?;
        Ok(Self::new(&key))
    }

    /// Derive a per-namespace cipher from this master cipher (VER-03).
    ///
    /// `key(namespace, version) = HKDF-SHA256(IKM = master, salt =
    /// `[`NS_ENVELOPE_SALT`]`, info = "ns" ‖ namespace ‖ [version])`.
    /// Namespace and version participate in the key: a blob re-labelled with
    /// another namespace or version does not authenticate.
    ///
    /// # Errors
    ///
    /// [`CryptoError::InvalidKey`] when this master is not a raw 32-byte key
    /// (a passphrase has no stable raw key to derive from). Callers must
    /// degrade explicitly — never silently.
    pub fn derive_namespace(&self, namespace: &str, version: u8) -> Result<Self, CryptoError> {
        let master = self.raw_key.as_ref().ok_or_else(|| {
            CryptoError::InvalidKey(
                "namespace key derivation requires a raw 32-byte key \
                 (passphrase keys are not supported)"
                    .into(),
            )
        })?;
        let salt = ring::hkdf::Salt::new(ring::hkdf::HKDF_SHA256, NS_ENVELOPE_SALT);
        let prk = salt.extract(&master[..]);
        let info: [&[u8]; 3] = [NS_ENVELOPE_INFO, namespace.as_bytes(), &[version]];
        let okm = prk
            .expand(&info, Aes256KeyType)
            .map_err(|_| CryptoError::InvalidKey("HKDF expand failed".into()))?;
        let mut key = [0u8; 32];
        okm.fill(&mut key)
            .map_err(|_| CryptoError::InvalidKey("HKDF fill failed".into()))?;
        Ok(Self::new(&key))
    }

    /// Derive the **key-encryption key** (KEK) of a database (ADR-0057 D2).
    ///
    /// `KEK = HKDF-SHA256(IKM = master, salt = db_salt, info = "vanta-kek-v1" ‖ key_version)`.
    ///
    /// The salt is **caller-supplied** (the database salt) — unlike
    /// [`Cipher::derive_namespace`], whose salt is fixed — so one master key
    /// can protect several databases without sharing a KEK. The key version
    /// participates in the derivation, so a rotation produces a different KEK.
    ///
    /// # Errors
    ///
    /// [`CryptoError::InvalidKey`] when this master is not a raw 32-byte key
    /// (a passphrase has no stable raw key to derive from).
    pub fn derive_kek(&self, salt: &[u8], key_version: u8) -> Result<Self, CryptoError> {
        let master = self.raw_key.as_ref().ok_or_else(|| {
            CryptoError::InvalidKey(
                "KEK derivation requires a raw 32-byte key \
                 (passphrase keys are not supported)"
                    .into(),
            )
        })?;
        let hkdf_salt = ring::hkdf::Salt::new(ring::hkdf::HKDF_SHA256, salt);
        let prk = hkdf_salt.extract(&master[..]);
        let info: [&[u8]; 2] = [KEK_INFO, &[key_version]];
        let okm = prk
            .expand(&info, Aes256KeyType)
            .map_err(|_| CryptoError::InvalidKey("HKDF expand failed".into()))?;
        let mut key = [0u8; 32];
        okm.fill(&mut key)
            .map_err(|_| CryptoError::InvalidKey("HKDF fill failed".into()))?;
        Ok(Self::new(&key))
    }

    /// The raw 32-byte master key (ADR-0057 C-4).
    ///
    /// # Errors
    ///
    /// [`CryptoError::InvalidKey`] for a passphrase cipher: a passphrase has no
    /// stable raw key, so the caller must stretch it first (the at-rest path
    /// stretches it **once** with the database salt).
    pub fn raw_key_bytes(&self) -> Result<[u8; 32], CryptoError> {
        let raw = self.raw_key.as_ref().ok_or_else(|| {
            CryptoError::InvalidKey(
                "raw key access requires a raw 32-byte key \
                 (passphrase keys are not supported)"
                    .into(),
            )
        })?;
        Ok(**raw)
    }

    /// Encrypt `plaintext` with a **caller-chosen nonce** and associated data.
    ///
    /// Returns `ciphertext ‖ AEAD tag (16 bytes)`. The caller owns the nonce:
    /// it must be unique per key (ADR-0057 D3) — the registry's allocator
    /// provides it.
    ///
    /// # Errors
    ///
    /// [`CryptoError::InvalidKey`] for a passphrase cipher: the passphrase
    /// framing (per-message salt) is incompatible with a caller-chosen nonce.
    /// Use a raw 32-byte key, or stretch the passphrase once with
    /// [`registry::KeyRegistry`].
    pub fn encrypt_with_nonce(
        &self,
        nonce: &[u8; 12],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let raw = self.raw_key.as_ref().ok_or_else(|| {
            CryptoError::InvalidKey(
                "caller-chosen nonce encryption requires a raw 32-byte key \
                 (passphrase keys are not supported)"
                    .into(),
            )
        })?;
        let inner = cipher_from_raw(raw);
        inner
            .encrypt(
                &AesNonce::from(*nonce),
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| CryptoError::InvalidCiphertext("AES-256-GCM encryption failed".into()))
    }

    /// Decrypt a ciphertext produced by [`Cipher::encrypt_with_nonce`].
    ///
    /// `ciphertext` is `ciphertext ‖ AEAD tag (16 bytes)`. The AAD must be
    /// **byte-identical** to the one used to encrypt: a different AAD fails
    /// authentication (that is the point — the AAD binds a ciphertext to its
    /// slot, see ADR-0057 D4).
    ///
    /// # Errors
    ///
    /// [`CryptoError::DecryptionFailed`] when the tag does not authenticate —
    /// wrong key **or** tampered ciphertext, which a single AEAD tag cannot
    /// tell apart. [`CryptoError::InvalidCiphertext`] when the input is
    /// shorter than the tag.
    pub fn decrypt_with_nonce(
        &self,
        nonce: &[u8; 12],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let raw = self.raw_key.as_ref().ok_or_else(|| {
            CryptoError::InvalidKey(
                "caller-chosen nonce decryption requires a raw 32-byte key \
                 (passphrase keys are not supported)"
                    .into(),
            )
        })?;
        if ciphertext.len() < TAG_LEN {
            return Err(CryptoError::InvalidCiphertext(format!(
                "ciphertext shorter than the {TAG_LEN}-byte AEAD tag"
            )));
        }
        let inner = cipher_from_raw(raw);
        inner
            .decrypt(
                &AesNonce::from(*nonce),
                Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| CryptoError::DecryptionFailed)
    }

    /// Encrypt `plaintext` with a fresh random 12-byte nonce.
    ///
    /// Returns `[nonce (12 bytes) || ciphertext + AEAD tag (16 bytes)]` for
    /// raw 32-byte keys, or
    /// `[version (1) || salt (16) || nonce (12) || ciphertext + tag]` for
    /// PBKDF2-derived keys.
    ///
    /// # Infallibility
    ///
    /// This method does **not** return `Result` because AEAD encryption with a
    /// validly constructed cipher and a fresh random nonce can only fail on
    /// out-of-memory — the same class of failure as [`Vec::push`] or
    /// [`Vec::extend_from_slice`]. The underlying [`Aead::encrypt`] call from
    /// RustCrypto's `aes-gcm` crate guarantees this: with correct parameters,
    /// encryption is a pure computation that cannot produce an "error" in the
    /// cryptographic sense (see [RustCRepo/aead#100]).
    ///
    /// If OOM occurs, the process is irrecoverable regardless of how we handle
    /// it — unwinding will abort in most configurations. Callers who need a
    /// fallible interface should use [`EncryptionStream`], which wraps this
    /// call into [`io::Result`] at the stream boundary.
    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        // aead 0.6 removed `AeadCore::generate_nonce`: fill from the OS CSPRNG
        // directly (same source `Generate` would use) via rand 0.9.
        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = AesNonce::from(nonce_bytes);
        match &self.raw_key {
            Some(raw) => {
                let inner = cipher_from_raw(raw);
                let ciphertext = inner
                    // INVARIANT (B2b, cat. (b)): AES-256-GCM encryption is
                    // infallible per RustCrypto guarantee (only decryption fails).
                    .encrypt(&nonce, plaintext)
                    .expect("AES-256-GCM encryption is infallible per RustCrypto guarantee");
                let mut out = Vec::with_capacity(12 + ciphertext.len());
                out.extend_from_slice(nonce.as_slice());
                out.extend_from_slice(&ciphertext);
                out
            }
            None => {
                // INVARIANT (B2b, cat. (b)): `Cipher` holds exactly one of
                // `raw_key`/`password` by construction, so `None` here means
                // `raw_key` is `None` and `password` is `Some`.
                let password = self
                    .password
                    .as_deref()
                    .expect("Cipher holds exactly one of raw_key/password");
                // Per-message random salt defeats precomputed rainbow tables for
                // a short passphrase.
                let mut salt = [0u8; KDF_SALT_LEN];
                rand::rng().fill_bytes(&mut salt);
                let inner = derive_from_password(password, &salt);
                let ciphertext = inner
                    // INVARIANT (B2b, cat. (b)): same RustCrypto infallibility
                    // guarantee as the raw-key arm above.
                    .encrypt(&nonce, plaintext)
                    .expect("AES-256-GCM encryption is infallible per RustCrypto guarantee");
                let mut out = Vec::with_capacity(KDF_FRAME_MIN - 16 + ciphertext.len());
                out.push(KDF_FRAME_VERSION);
                out.extend_from_slice(&salt);
                out.extend_from_slice(nonce.as_slice());
                out.extend_from_slice(&ciphertext);
                out
            }
        }
    }

    /// Decrypt data previously produced by [`Cipher::encrypt`].
    ///
    /// Accepts both the current PBKDF2-framed format and the legacy formats
    /// produced by older versions (raw 32-byte key framing and the old
    /// SHA-256 fallback for short keys). Returns [`CryptoError::DecryptionFailed`]
    /// if no scheme authenticates.
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.try_decrypt(data).ok_or(CryptoError::DecryptionFailed)
    }

    fn try_decrypt(&self, data: &[u8]) -> Option<Vec<u8>> {
        match &self.raw_key {
            Some(raw) => decrypt_legacy(data, raw),
            None => {
                let password = self.password.as_deref()?;
                // Try the current PBKDF2-framed format first.
                if data.len() >= KDF_FRAME_MIN && data[0] == KDF_FRAME_VERSION {
                    let (header, body) = data.split_at(1 + KDF_SALT_LEN);
                    let salt = &header[1..];
                    let inner = derive_from_password(password, salt);
                    let (nonce_bytes, ciphertext) = body.split_at(12);
                    let nonce_array: [u8; 12] = nonce_bytes.try_into().ok()?;
                    if let Ok(pt) = inner.decrypt(&AesNonce::from(nonce_array), ciphertext) {
                        return Some(pt);
                    }
                    // Fall through: a legacy message whose first nonce byte
                    // coincides with the version marker (1/256) will not
                    // authenticate as a PBKDF2 frame — try legacy.
                }
                // Legacy formats: a raw 32-byte key honored directly, or the
                // old single-pass SHA-256 fallback for short keys.
                let legacy_raw: [u8; 32] = Sha256::digest(password).into();
                decrypt_legacy(data, &legacy_raw)
            }
        }
    }
}

/// Build an AES-256-GCM cipher from a raw 32-byte key.
fn cipher_from_raw(key: &[u8; 32]) -> Aes256Gcm {
    // INVARIANT (B2b, cat. (b)): 32-byte key always satisfies AES-256 length.
    Aes256Gcm::new_from_slice(key)
        .expect("Aes256Gcm::new_from_slice with a 32-byte key cannot fail")
}

/// Key-stretch a passphrase via PBKDF2-HMAC-SHA256 into a 32-byte AES key.
fn derive_from_password(password: &[u8], salt: &[u8]) -> Aes256Gcm {
    let mut key = [0u8; 32];
    // INVARIANT (B2b, cat. (b)): `KDF_ITERATIONS` is a non-zero const.
    let iterations = std::num::NonZeroU32::new(KDF_ITERATIONS).expect("KDF_ITERATIONS is non-zero");
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        password,
        &mut key,
    );
    cipher_from_raw(&key)
}

/// Decrypt a legacy (or raw-key) message `[nonce(12)][ct + tag]`.
fn decrypt_legacy(data: &[u8], key: &[u8; 32]) -> Option<Vec<u8>> {
    if data.len() < 12 + 16 {
        return None;
    }
    let inner = Aes256Gcm::new_from_slice(key).ok()?;
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce_array: [u8; 12] = nonce_bytes.try_into().ok()?;
    inner.decrypt(&AesNonce::from(nonce_array), ciphertext).ok()
}

/// A transparent Read/Write wrapper that encrypts data on write and decrypts
/// on read using a framing protocol.
///
/// # Wire Format
///
/// Each frame written to the inner stream:
/// `[4-byte LE payload length][12-byte nonce][ciphertext + 16-byte AEAD tag]`
///
/// The reader reads one frame at a time and caches partial results for
/// subsequent `read` calls.
///
/// # Deprecated
///
/// ADR-0057 C-6: this framing expresses **no AAD, no slot and no epoch**, so
/// it cannot be used for at-rest encryption — a ciphertext moved to another
/// position would still authenticate. The at-rest path uses the per-artefact
/// envelope built on [`Cipher::encrypt_with_nonce`] plus the AAD builder
/// ([`registry::AadBuilder`]). The type is kept (not removed) because
/// `storage::vfile` still exposes it; it will be retired with the VantaFile
/// wave.
#[deprecated(
    since = "0.9.1",
    note = "ADR-0057 C-6: the framing has no AAD/slot/epoch; use Cipher::encrypt_with_nonce with a registry::AadBuilder for at-rest encryption"
)]
pub struct EncryptionStream<S> {
    inner: S,
    cipher: Cipher,
    read_buf: Vec<u8>,
    read_pos: usize,
}

// The deprecated stream still implements its own contract; the allow is
// scoped to these impls (ADR-0057 C-6 keeps the type until the VantaFile wave).
#[allow(deprecated)]
impl<S> EncryptionStream<S> {
    /// Wrap an existing stream with the given cipher.
    pub fn new(inner: S, cipher: Cipher) -> Self {
        Self {
            inner,
            cipher,
            read_buf: Vec::new(),
            read_pos: 0,
        }
    }

    /// Borrow the inner stream.
    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    /// Mutably borrow the inner stream.
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Consume the wrapper and return the inner stream.
    pub fn into_inner(self) -> S {
        self.inner
    }
}

#[allow(deprecated)]
impl<S: Read + Write> Write for EncryptionStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let encrypted = self.cipher.encrypt(buf);
        let len_bytes = (encrypted.len() as u32).to_le_bytes();
        self.inner.write_all(&len_bytes)?;
        self.inner.write_all(&encrypted)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[allow(deprecated)]
impl<S: Read + Write> Read for EncryptionStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_pos >= self.read_buf.len() {
            let mut len_buf = [0u8; 4];
            if let Err(e) = self.inner.read_exact(&mut len_buf) {
                return if e.kind() == io::ErrorKind::UnexpectedEof {
                    Ok(0)
                } else {
                    Err(e)
                };
            }
            let frame_len = u32::from_le_bytes(len_buf) as usize;
            if frame_len > MAX_FRAME_LEN {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    CryptoError::InvalidCiphertext(format!(
                        "frame length {frame_len} exceeds limit {MAX_FRAME_LEN}"
                    )),
                ));
            }
            let mut frame = vec![0u8; frame_len];
            self.inner.read_exact(&mut frame)?;
            self.read_buf = self
                .cipher
                .decrypt(&frame)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            self.read_pos = 0;
        }
        let avail = self.read_buf.len() - self.read_pos;
        let n = avail.min(buf.len());
        buf[..n].copy_from_slice(&self.read_buf[self.read_pos..self.read_pos + n]);
        self.read_pos += n;
        Ok(n)
    }
}

/// Decode a hex string into bytes.
///
/// Accepts an optional `0x` prefix. Returns an error on invalid hex characters
/// or odd length. Operates on BYTES (`chunks_exact(2)`) — a multibyte UTF-8
/// key at an even offset degrades to `InvalidKey`, never a char-boundary
/// panic at startup (F-01 hardening; env-fed input).
fn decode_hex(s: &str) -> Result<Vec<u8>, CryptoError> {
    let s = s.trim();
    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    let bytes = s.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err(CryptoError::InvalidKey(
            "hex string must have an even number of characters".into(),
        ));
    }
    bytes
        .chunks_exact(2)
        .enumerate()
        .map(|(i, pair)| {
            let offset = i * 2;
            let chunk = std::str::from_utf8(pair).map_err(|_| {
                CryptoError::InvalidKey(format!("invalid hex at position {offset}"))
            })?;
            u8::from_str_radix(chunk, 16).map_err(|e| {
                CryptoError::InvalidKey(format!("invalid hex at position {offset}: {e}"))
            })
        })
        .collect()
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read, Write};

    #[test]
    fn test_encode_decode_hex() {
        let hex_str = "a1b2c3d4e5f6789012345678abcdef0123456789abcdef0123456789abcdef01";
        assert_eq!(hex_str.len(), 64);
        let decoded = decode_hex(hex_str).unwrap();
        assert_eq!(decoded.len(), 32);
        let re_encoded: String = decoded.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(re_encoded, hex_str);
    }

    #[test]
    fn test_decode_hex_with_0x_prefix() {
        let hex_str = "0xa1b2c3d4e5f6789012345678abcdef0123456789abcdef0123456789abcdef01";
        let decoded = decode_hex(hex_str).unwrap();
        assert_eq!(decoded.len(), 32);
    }

    #[test]
    fn test_decode_hex_odd_length() {
        let result = decode_hex("abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_hex_invalid_char() {
        let result = decode_hex("zzzz");
        assert!(result.is_err());
    }

    #[test]
    fn test_cipher_roundtrip() {
        let key = b"this is exactly 32 bytes in len!";
        assert_eq!(key.len(), 32);
        let cipher = Cipher::new(key);
        let plaintext = b"Hello, VantaDB encryption!";
        let encrypted = cipher.encrypt(plaintext);
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_cipher_short_key_derivation() {
        let short_key = b"short key";
        let cipher = Cipher::new(short_key);
        let data = b"some data";
        let encrypted = cipher.encrypt(data);
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_short_key_uses_kdf_framing() {
        // A derived (short) key must produce a PBKDF2-framed message: the
        // version byte plus a 16-byte random salt precede the nonce.
        let cipher = Cipher::new(b"short key");
        let encrypted = cipher.encrypt(b"data");
        assert_eq!(encrypted[0], KDF_FRAME_VERSION);
        assert!(encrypted.len() >= KDF_FRAME_MIN);
        // Salt must be fresh per message (two encryptions, different salt).
        let second = cipher.encrypt(b"data");
        assert_ne!(
            &encrypted[1..1 + KDF_SALT_LEN],
            &second[1..1 + KDF_SALT_LEN]
        );
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, b"data");
    }

    #[test]
    fn test_legacy_sha256_message_still_decrypts() {
        // Backward compatibility (AUDREP-10): a message written by the old
        // format — key = SHA-256(passphrase), framing = [nonce][ct+tag] — must
        // still decrypt with the new PBKDF2 cipher.
        let password = b"short key";
        let legacy_key: [u8; 32] = Sha256::digest(password).into();
        let legacy_cipher = Aes256Gcm::new_from_slice(&legacy_key).unwrap();
        let mut legacy_nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut legacy_nonce_bytes);
        let nonce = AesNonce::from(legacy_nonce_bytes);
        let ct = legacy_cipher
            .encrypt(&nonce, b"legacy payload".as_slice())
            .expect("legacy encryption is infallible");
        let mut legacy_message = Vec::with_capacity(12 + ct.len());
        legacy_message.extend_from_slice(nonce.as_slice());
        legacy_message.extend_from_slice(&ct);

        let new_cipher = Cipher::new(password);
        let decrypted = new_cipher.decrypt(&legacy_message).unwrap();
        assert_eq!(decrypted, b"legacy payload");
    }

    #[test]
    fn test_raw_key_message_still_decrypts_without_header() {
        // The 32-byte raw-key path keeps the original framing (no version/salt
        // header) — the same bytes old versions produced, so existing files
        // encrypted with a proper 32-byte key remain fully compatible.
        let raw = [0x42u8; 32];
        let cipher = Cipher::new(&raw);
        let encrypted = cipher.encrypt(b"raw payload");
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, b"raw payload");
    }

    #[test]
    fn test_cipher_different_keys_fail() {
        let key_a = [0u8; 32];
        let key_b = [1u8; 32];
        let cipher_a = Cipher::new(&key_a);
        let cipher_b = Cipher::new(&key_b);
        let encrypted = cipher_a.encrypt(b"secret");
        assert!(cipher_b.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_cipher_empty_data() {
        let key = [0x42u8; 32];
        let cipher = Cipher::new(&key);
        let encrypted = cipher.encrypt(b"");
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_decrypt_truncated_data() {
        let key = [0u8; 32];
        let cipher = Cipher::new(&key);
        let result = cipher.decrypt(&[0u8; 20]);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_stream_roundtrip() {
        let key = [0xABu8; 32];
        let cipher = Cipher::new(&key);
        let mut buf = Cursor::new(Vec::new());
        {
            let mut writer = EncryptionStream::new(&mut buf, cipher);
            writer.write_all(b"hello ").unwrap();
            writer.write_all(b"world").unwrap();
            writer.flush().unwrap();
        }
        let encrypted_bytes = buf.into_inner();
        // Now read back using a cursor over the encrypted bytes
        let cipher = Cipher::new(&[0xABu8; 32]);
        let read_cursor = Cursor::new(encrypted_bytes);
        let mut reader = EncryptionStream::new(read_cursor, cipher);
        let mut output = String::new();
        reader.read_to_string(&mut output).unwrap();
        assert_eq!(output, "hello world");
    }

    #[test]
    fn test_encryption_stream_wrong_key() {
        let key_a = [0xAAu8; 32];
        let key_b = [0xBBu8; 32];
        let cipher = Cipher::new(&key_a);
        let mut buf = Cursor::new(Vec::new());
        {
            let mut writer = EncryptionStream::new(&mut buf, cipher);
            writer.write_all(b"secret data").unwrap();
            writer.flush().unwrap();
        }
        let encrypted_bytes = buf.into_inner();
        let read_cursor = Cursor::new(encrypted_bytes);
        let cipher_b = Cipher::new(&key_b);
        let mut reader = EncryptionStream::new(read_cursor, cipher_b);
        let mut output = Vec::new();
        let result = reader.read_to_end(&mut output);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_stream_partial_reads() {
        let key = [0xCDu8; 32];
        let cipher = Cipher::new(&key);
        let mut buf = Cursor::new(Vec::new());
        {
            let mut writer = EncryptionStream::new(&mut buf, cipher);
            writer
                .write_all(b"this is a longer message to test partial reads")
                .unwrap();
            writer.flush().unwrap();
        }
        let encrypted_bytes = buf.into_inner();
        let read_cursor = Cursor::new(encrypted_bytes);
        let cipher = Cipher::new(&[0xCDu8; 32]);
        let mut reader = EncryptionStream::new(read_cursor, cipher);
        let mut chunk = [0u8; 10];
        let mut result = Vec::new();
        loop {
            let n = reader.read(&mut chunk).unwrap();
            if n == 0 {
                break;
            }
            result.extend_from_slice(&chunk[..n]);
        }
        assert_eq!(result, b"this is a longer message to test partial reads");
    }

    #[test]
    fn test_encryption_stream_rejects_oversized_frame() {
        // AUDREP-31: a forged header with an absurd frame length must fail
        // before allocating, not OOM (previously up to u32::MAX / 4GiB).
        let forged_header = u32::MAX.to_le_bytes();

        let read_cursor = Cursor::new(forged_header);
        let cipher = Cipher::new(&[0xEEu8; 32]);
        let mut reader = EncryptionStream::new(read_cursor, cipher);
        let mut output = Vec::new();
        let err = reader.read_to_end(&mut output).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(output.is_empty());
    }

    #[test]
    fn test_from_env_not_set() {
        std::env::remove_var("VANTADB_ENCRYPTION_KEY");
        let result = Cipher::from_env();
        assert!(matches!(result, Err(CryptoError::KeyNotSet)));
    }

    #[test]
    fn test_derive_namespace_binds_namespace_and_version() {
        let master = Cipher::new(&[0x11u8; 32]);
        let a1 = master.derive_namespace("ns-a", 1).unwrap();
        let b1 = master.derive_namespace("ns-b", 1).unwrap();
        let a2 = master.derive_namespace("ns-a", 2).unwrap();
        let encrypted = a1.encrypt(b"secret");
        assert_eq!(a1.decrypt(&encrypted).unwrap(), b"secret");
        assert!(
            b1.decrypt(&encrypted).is_err(),
            "namespace participates in the key"
        );
        assert!(
            a2.decrypt(&encrypted).is_err(),
            "version participates in the key"
        );
    }

    #[test]
    fn test_derive_namespace_is_deterministic() {
        let master = Cipher::new(&[0x22u8; 32]);
        let first = master.derive_namespace("ns", 7).unwrap();
        let second = master.derive_namespace("ns", 7).unwrap();
        let encrypted = first.encrypt(b"x");
        assert_eq!(second.decrypt(&encrypted).unwrap(), b"x");
    }

    #[test]
    fn test_derive_namespace_rejects_passphrase_master() {
        let master = Cipher::new(b"short passphrase");
        let result = master.derive_namespace("ns", 1);
        assert!(matches!(result, Err(CryptoError::InvalidKey(_))));
    }

    #[test]
    fn test_decode_hex_multibyte_degrades_instead_of_panicking() {
        // F-01: a multibyte char at an even byte offset must be an error,
        // not a char-boundary panic (env-fed key path).
        let result = decode_hex("aa€x");
        assert!(matches!(result, Err(CryptoError::InvalidKey(_))));
    }

    #[test]
    fn test_encrypt_uses_fresh_nonce_per_message() {
        let cipher = Cipher::new(&[0x33u8; 32]);
        let first = cipher.encrypt(b"same plaintext");
        let second = cipher.encrypt(b"same plaintext");
        assert_ne!(
            &first[..12],
            &second[..12],
            "fresh random nonce per message (12-byte prefix)"
        );
        assert_eq!(cipher.decrypt(&first).unwrap(), b"same plaintext");
        assert_eq!(cipher.decrypt(&second).unwrap(), b"same plaintext");
    }

    // ── ADR-0057 W1: at-rest primitives ──────────────────────────────────

    /// Hex of a 32-byte key.
    fn hex_of(key: &[u8; 32]) -> String {
        key.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// A shell command that prints `text` without a trailing newline.
    fn echo_cmd(text: &str) -> String {
        if cfg!(windows) {
            format!("echo {text}")
        } else {
            format!("printf %s {text}")
        }
    }

    #[test]
    fn key_from_file_reads_hex_and_rejects_world_readable_on_unix() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("master.key");
        let hex = hex_of(
            &[0xa1, 0xb2, 0xc3, 0xd4, 0x11, 0x22, 0x33, 0x44]
                .repeat(4)
                .try_into()
                .unwrap(),
        );
        std::fs::write(&path, format!("{hex}\n")).unwrap();

        // A regular file holding the hex key resolves to the key bytes.
        let material = KeySource::File(path.clone()).read_key_material().unwrap();
        assert_eq!(material.len(), 32);
        assert_eq!(material[..4], [0xa1, 0xb2, 0xc3, 0xd4]);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // Group/other access is refused (ADR-0057 D2).
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
            let err = KeySource::File(path.clone())
                .read_key_material()
                .unwrap_err();
            assert!(matches!(err, CryptoError::KeySourceFailed(_)), "got: {err}");
            assert!(err.to_string().contains("group or other"), "got: {err}");

            // 0600 is accepted.
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
            assert!(KeySource::File(path.clone()).read_key_material().is_ok());

            // A symlink is refused.
            let link = dir.path().join("link.key");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            let err = KeySource::File(link).read_key_material().unwrap_err();
            assert!(matches!(err, CryptoError::KeySourceFailed(_)), "got: {err}");
            assert!(err.to_string().contains("symlink"), "got: {err}");
        }
    }

    #[test]
    fn key_from_command_reads_stdout_and_never_logs_it() {
        let key = [0x5au8; 32];
        let hex = hex_of(&key);
        let material = KeySource::Command(echo_cmd(&hex))
            .read_key_material()
            .unwrap();
        assert_eq!(material.len(), 32);
        assert_eq!(material[..], key[..]);

        // Output that is not a hex key fails, and the error never carries the
        // printed material.
        let secret = "s3cr3t-not-hex";
        let err = KeySource::Command(echo_cmd(secret))
            .read_key_material()
            .unwrap_err();
        assert!(matches!(err, CryptoError::KeySourceFailed(_)), "got: {err}");
        assert!(
            !err.to_string().contains(secret),
            "the key material leaked into the error: {err}"
        );

        // A failing command reports only its exit status.
        let err = KeySource::Command("exit 3".to_string())
            .read_key_material()
            .unwrap_err();
        assert!(matches!(err, CryptoError::KeySourceFailed(_)), "got: {err}");
        assert!(err.to_string().contains("status"), "got: {err}");
    }

    #[test]
    fn key_sources_have_fixed_precedence_env_file_cmd() {
        let dir = tempfile::tempdir().unwrap();
        let env_key = [0x11u8; 32];
        let file_key = [0x22u8; 32];
        let cmd_key = [0x33u8; 32];

        let path = dir.path().join("master.key");
        std::fs::write(&path, hex_of(&file_key)).unwrap();
        let cmd = echo_cmd(&hex_of(&cmd_key));

        // 1. The environment wins.
        let cipher = resolve_master_key(Some(&hex_of(&env_key)), Some(&path), Some(&cmd)).unwrap();
        assert_eq!(cipher.raw_key_bytes().unwrap(), env_key);

        // 2. Without the environment, the file wins.
        let cipher = resolve_master_key(None, Some(&path), Some(&cmd)).unwrap();
        assert_eq!(cipher.raw_key_bytes().unwrap(), file_key);

        // 3. Without environment and file, the command wins.
        let cipher = resolve_master_key(None, None, Some(&cmd)).unwrap();
        assert_eq!(cipher.raw_key_bytes().unwrap(), cmd_key);

        // 4. With no source at all there is no key.
        let err = resolve_master_key(None, None, None).unwrap_err();
        assert!(matches!(err, CryptoError::KeyNotSet), "got: {err}");
    }

    #[test]
    fn cipher_encrypt_decrypt_legacy_behaviour_is_unchanged() {
        // Raw 32-byte key: [nonce(12)][ct+tag] — unchanged since v0.4.
        let cipher = Cipher::new(&[0x42u8; 32]);
        let plaintext = b"legacy framing";
        let encrypted = cipher.encrypt(plaintext);
        assert_eq!(encrypted.len(), 12 + plaintext.len() + 16);
        assert_eq!(cipher.decrypt(&encrypted).unwrap(), plaintext);

        // Two encryptions of the same plaintext use different nonces.
        let other = cipher.encrypt(plaintext);
        assert_ne!(&encrypted[..12], &other[..12]);

        // Short key: the PBKDF2 frame is still [version][salt(16)][nonce][ct+tag].
        let short = Cipher::new(b"short key");
        let framed = short.encrypt(plaintext);
        assert_eq!(framed[0], KDF_FRAME_VERSION);
        assert!(framed.len() >= KDF_FRAME_MIN);
        assert_eq!(short.decrypt(&framed).unwrap(), plaintext);
    }

    #[test]
    fn derive_kek_uses_the_caller_supplied_salt() {
        let master = Cipher::new(&[0x55u8; 32]);
        let salt_a = [0xABu8; 32];
        let salt_b = [0xCDu8; 32];

        let kek_a = master.derive_kek(&salt_a, 1).unwrap();
        let kek_a_again = master.derive_kek(&salt_a, 1).unwrap();
        let kek_b = master.derive_kek(&salt_b, 1).unwrap();
        let kek_v2 = master.derive_kek(&salt_a, 2).unwrap();

        // Deterministic for the same salt and version.
        let sealed = kek_a
            .encrypt_with_nonce(&[7u8; 12], b"payload", b"aad")
            .unwrap();
        assert_eq!(
            kek_a_again
                .decrypt_with_nonce(&[7u8; 12], &sealed, b"aad")
                .unwrap(),
            b"payload"
        );

        // A different salt (or version) yields a different key: the sealed
        // payload does not authenticate.
        assert!(kek_b
            .decrypt_with_nonce(&[7u8; 12], &sealed, b"aad")
            .is_err());
        assert!(kek_v2
            .decrypt_with_nonce(&[7u8; 12], &sealed, b"aad")
            .is_err());

        // The KEK is not the namespace-envelope key either.
        let ns = master.derive_namespace("ns", 1).unwrap();
        assert!(ns.decrypt_with_nonce(&[7u8; 12], &sealed, b"aad").is_err());
    }

    #[test]
    fn raw_key_bytes_rejects_a_passphrase_cipher() {
        let raw = [0x77u8; 32];
        assert_eq!(Cipher::new(&raw).raw_key_bytes().unwrap(), raw);

        let err = Cipher::new(b"a passphrase").raw_key_bytes().unwrap_err();
        assert!(matches!(err, CryptoError::InvalidKey(_)), "got: {err}");

        // The same rule applies to the KEK derivation.
        let err = Cipher::new(b"a passphrase")
            .derive_kek(&[0u8; 32], 1)
            .unwrap_err();
        assert!(matches!(err, CryptoError::InvalidKey(_)), "got: {err}");
    }

    #[test]
    fn error_variants_map_to_stable_codes() {
        use crate::error::Error;

        // The four encryption variants reuse the canonical code set: the wire
        // contract (10 codes, docs/api/ERROR_HANDLING.md §1.1) does not grow,
        // so no binding has to learn a new code.
        assert_eq!(
            Error::EncryptionKeyRequired {
                artifact: "wal".into()
            }
            .code(),
            "VANTADB_VALIDATION_ERROR"
        );
        assert_eq!(
            Error::EncryptionKeyRejected {
                artifact: "wal".into()
            }
            .code(),
            "VANTADB_VALIDATION_ERROR"
        );
        assert_eq!(
            Error::EncryptionCorrupt {
                artifact: "wal".into(),
                detail: "crc".into()
            }
            .code(),
            "VANTADB_CORRUPT"
        );
        assert_eq!(
            Error::EncryptionUnsupported {
                artifact: "wal".into(),
                found_version: 9
            }
            .code(),
            "VANTADB_VALIDATION_ERROR"
        );

        // "Wrong key" and "tampered" are indistinguishable — the message says so.
        let rejected = Error::EncryptionKeyRejected {
            artifact: "wal".into(),
        };
        assert!(
            rejected.to_string().contains("wrong key or tampered"),
            "got: {rejected}"
        );

        // Every code stays inside the canonical `VANTADB_*` family.
        for code in [
            rejected.code(),
            Error::EncryptionKeyRequired {
                artifact: "wal".into(),
            }
            .code(),
            Error::EncryptionCorrupt {
                artifact: "wal".into(),
                detail: String::new(),
            }
            .code(),
            Error::EncryptionUnsupported {
                artifact: "wal".into(),
                found_version: 9,
            }
            .code(),
        ] {
            assert!(code.starts_with("VANTADB_"), "got: {code}");
        }
    }

    #[test]
    fn plaintext_path_is_byte_identical_without_key() {
        // The feature compiled but **no key configured**: the storage path
        // stays plaintext — the payload is written verbatim, with no envelope
        // and no extra bytes. The feature is inert without a key.
        use crate::config::Config;
        use crate::sdk::{Embedded, MemoryInput};

        let dir = tempfile::tempdir().unwrap();
        let canary = "VANTADB-PLAINTEXT-CANARY-8f3a1c";
        let engine = Embedded::open_with_config(
            Config::default().with_storage_path(dir.path().to_string_lossy().to_string()),
        )
        .unwrap();
        engine.put(MemoryInput::new("ns", "key", canary)).unwrap();
        engine.close().unwrap();

        let mut files = Vec::new();
        collect_files(dir.path(), &mut files);
        assert!(!files.is_empty(), "the engine wrote no artefact at all");
        let found = files.iter().any(|path| {
            let bytes = std::fs::read(path).unwrap_or_default();
            bytes.windows(canary.len()).any(|w| w == canary.as_bytes())
        });
        assert!(
            found,
            "without a key the payload must be stored in the clear (no envelope)"
        );
    }

    /// Collect every file under `dir` (recursively).
    fn collect_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, out);
            } else {
                out.push(path);
            }
        }
    }
}
