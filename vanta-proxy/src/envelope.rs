//! Per-namespace AEAD envelopes for captured originals (VER-03).
//!
//! The write-back persists the redacted text; the pre-redaction original only
//! survives inside an envelope sealed under a key derived for the namespace
//! that stores it. The key is `HKDF-SHA256(master, salt, info = "ns" ‖ ns ‖
//! [version])` over the project's existing AES-256-GCM primitive
//! ([`Cipher`]), so a blob is bound to its namespace and key version by
//! cryptography, not by labels.
//!
//! Degradation is explicit and never silent: with no usable key the envelope
//! reports [`EnvelopeMode::Disarmed`] (the caller logs the startup warning)
//! and seals nothing — the redacted text is the only copy. It never falls
//! back to writing the original in clear.

use serde::{Deserialize, Serialize};

use vantadb::crypto::Cipher;

/// Envelope wiring configuration (VER-03). Disabled by default: the redacted
/// text is the only copy of a captured turn unless explicitly opted in (same
/// opt-in invariant as cache/routing/cost/redact).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct EnvelopeConfig {
    /// Master switch. Requires a raw 32-byte `VANTADB_ENCRYPTION_KEY` and
    /// `[redact] enabled`; without a usable key the proxy degrades to
    /// redacted-only with a startup warning (never clear originals).
    pub enabled: bool,
    /// Key-rotation marker for new envelopes. Bump to rotate: new writes seal
    /// with the new derived key while previous envelopes stay readable under
    /// the same master key.
    pub key_version: u8,
}

impl Default for EnvelopeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            key_version: 1,
        }
    }
}

/// Effective envelope behavior (observable; never implicit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeMode {
    /// `[envelope]` off — declared: the redacted text is the only copy.
    RedactedOnly,
    /// On and keyed: originals are sealed per namespace.
    Active,
    /// On but the key is missing/invalid — explicit degradation at startup.
    /// Behaves as redacted-only; the original is never written in clear.
    Disarmed,
}

impl EnvelopeMode {
    /// Stable label used in logs and the operational snapshot.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RedactedOnly => "redacted_only",
            Self::Active => "active",
            Self::Disarmed => "disarmed",
        }
    }
}

/// Envelope wire version (framing; bumped on format changes).
const ENVELOPE_VERSION: u8 = 1;

/// Per-namespace AEAD envelope. Build once at startup; cheap to hold.
pub struct Envelope {
    /// Master cipher. `None` when inactive (off or disarmed).
    master: Option<Cipher>,
    /// Key version used to seal new envelopes.
    key_version: u8,
    mode: EnvelopeMode,
    /// Why the envelope is disarmed (for the explicit startup warning).
    disarm_reason: Option<String>,
}

/// One sealed original: `{ "v": 1, "k": 1, "ns": "…", "ct": "<hex>" }`.
///
/// `ct` is the hex of [`Cipher::encrypt`] output (`nonce ‖ ciphertext+tag`);
/// the plaintext original never touches the store in clear.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeBlob {
    /// Envelope framing version.
    #[serde(rename = "v")]
    pub version: u8,
    /// Key-rotation version used in the derivation.
    #[serde(rename = "k")]
    pub key_version: u8,
    /// Namespace whose derived key seals (and authenticates) the payload.
    #[serde(rename = "ns")]
    pub namespace: String,
    /// Hex of `Cipher::encrypt` output (`nonce ‖ ciphertext+tag`).
    #[serde(rename = "ct")]
    pub ciphertext: String,
}

/// Envelope errors (typed; never carries key material).
#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    /// Not active (off or disarmed): there is nothing to open.
    #[error("envelope is not active (redacted-only mode)")]
    Inactive,
    /// Malformed envelope (unknown framing version, bad hex).
    #[error("envelope format invalid: {0}")]
    Format(String),
    /// AEAD authentication failed: wrong key, retyped namespace/version, or
    /// tampered ciphertext.
    #[error("envelope decryption failed (wrong key, namespace, version, or tampered payload)")]
    DecryptionFailed,
}

impl Envelope {
    /// Build from configuration, resolving the master key from
    /// `VANTADB_ENCRYPTION_KEY` (via [`Cipher::from_env`]).
    #[must_use]
    pub fn from_config(cfg: &EnvelopeConfig) -> Self {
        if !cfg.enabled {
            return Self::inactive(cfg.key_version, EnvelopeMode::RedactedOnly);
        }
        match Cipher::from_env() {
            Ok(master) => Self::with_master(cfg, Some(master)),
            Err(e) => {
                Self::inactive(cfg.key_version, EnvelopeMode::Disarmed).with_reason(e.to_string())
            }
        }
    }

    /// Build with an explicit master (tests, and callers that resolve the key
    /// themselves). Honors the same degradation rules as [`Self::from_config`].
    #[must_use]
    pub fn with_master(cfg: &EnvelopeConfig, master: Option<Cipher>) -> Self {
        if !cfg.enabled {
            return Self::inactive(cfg.key_version, EnvelopeMode::RedactedOnly);
        }
        let Some(master) = master else {
            return Self::inactive(cfg.key_version, EnvelopeMode::Disarmed)
                .with_reason("VANTADB_ENCRYPTION_KEY is not set".into());
        };
        // Probe derivability up front (a passphrase master has no raw key):
        // disarms at construction instead of failing lazily per seal.
        if let Err(e) = master.derive_namespace(crate::capture::TURNS_NAMESPACE, cfg.key_version) {
            return Self::inactive(cfg.key_version, EnvelopeMode::Disarmed)
                .with_reason(e.to_string());
        }
        Self {
            master: Some(master),
            key_version: cfg.key_version,
            mode: EnvelopeMode::Active,
            disarm_reason: None,
        }
    }

    fn inactive(key_version: u8, mode: EnvelopeMode) -> Self {
        Self {
            master: None,
            key_version,
            mode,
            disarm_reason: None,
        }
    }

    fn with_reason(mut self, reason: String) -> Self {
        self.disarm_reason = Some(reason);
        self
    }

    /// Effective mode (observable for tests / operational snapshot).
    #[must_use]
    pub fn mode(&self) -> EnvelopeMode {
        self.mode
    }

    /// Degradation reason — the caller logs it as the explicit startup
    /// warning. `None` for `RedactedOnly`/`Active`.
    #[must_use]
    pub fn disarm_reason(&self) -> Option<&str> {
        self.disarm_reason.as_deref()
    }

    /// Key version new envelopes are sealed with.
    #[must_use]
    pub fn key_version(&self) -> u8 {
        self.key_version
    }

    /// Seal `plaintext` under the key of `namespace` (current key version).
    ///
    /// Returns `None` when not active — the caller keeps the redacted copy
    /// only. The original is never written in clear as a fallback.
    #[must_use]
    pub fn seal(&self, namespace: &str, plaintext: &str) -> Option<EnvelopeBlob> {
        let master = self.master.as_ref()?;
        let cipher = match master.derive_namespace(namespace, self.key_version) {
            Ok(cipher) => cipher,
            Err(e) => {
                tracing::warn!(%e, "envelope seal skipped");
                return None;
            }
        };
        Some(EnvelopeBlob {
            version: ENVELOPE_VERSION,
            key_version: self.key_version,
            namespace: namespace.to_string(),
            ciphertext: encode_hex(&cipher.encrypt(plaintext.as_bytes())),
        })
    }

    /// Open a sealed original.
    ///
    /// The namespace and key version come from the blob, so envelopes written
    /// before a rotation stay readable. Requires the same master key.
    ///
    /// # Errors
    /// - [`EnvelopeError::Inactive`] when the envelope is off/disarmed.
    /// - [`EnvelopeError::Format`] on unknown framing version or bad hex.
    /// - [`EnvelopeError::DecryptionFailed`] when the AEAD tag rejects the
    ///   payload (wrong key, retyped namespace/version, tampering).
    pub fn open(&self, blob: &EnvelopeBlob) -> Result<String, EnvelopeError> {
        let master = self.master.as_ref().ok_or(EnvelopeError::Inactive)?;
        if blob.version != ENVELOPE_VERSION {
            return Err(EnvelopeError::Format(format!(
                "unsupported envelope version {}",
                blob.version
            )));
        }
        let cipher = master
            .derive_namespace(&blob.namespace, blob.key_version)
            .map_err(|e| EnvelopeError::Format(e.to_string()))?;
        let bytes = decode_hex(&blob.ciphertext)
            .ok_or_else(|| EnvelopeError::Format("ciphertext is not valid hex".into()))?;
        let plain = cipher
            .decrypt(&bytes)
            .map_err(|_| EnvelopeError::DecryptionFailed)?;
        String::from_utf8(plain).map_err(|_| EnvelopeError::DecryptionFailed)
    }
}

/// Lowercase hex encoding (ciphertext travels inside JSON).
fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        // `write!` to a String cannot fail.
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Strict hex decode: `None` on odd length or non-hex characters.
///
/// Operates on BYTES (`chunks_exact(2)`), never on `str` slicing — a `ct`
/// with a multibyte UTF-8 char at an even offset must yield `None`, not a
/// char-boundary panic (F-01).
fn decode_hex(s: &str) -> Option<Vec<u8>> {
    let bytes = s.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }
    bytes
        .chunks_exact(2)
        .map(|pair| {
            let chunk = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(chunk, 16).ok()
        })
        .collect()
}
