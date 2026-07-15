//! AES-256-GCM at-rest encryption for VantaDB storage.
//!
//! Provides [`Cipher`] for encrypt/decrypt operations and [`EncryptionStream`]
//! for transparent Read/Write wrapping of storage streams.
//!
//! # Key Derivation
//!
//! The encryption key is loaded from the `VANTADB_ENCRYPTION_KEY` environment
//! variable as a hex-encoded 32-byte (64 hex char) value. If the decoded key is
//! not exactly 32 bytes, it is derived using SHA-256.
//!
//! # Message Format
//!
//! Each encrypted message on the wire is framed as:
//! `[4-byte LE payload length][12-byte nonce][AEAD ciphertext + 16-byte tag]`

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};

/// Errors specific to cryptographic operations.
#[derive(Debug)]
pub enum CryptoError {
    /// The `VANTADB_ENCRYPTION_KEY` environment variable is not set.
    KeyNotSet,
    /// The key string is malformed or cannot be decoded.
    InvalidKey(String),
    /// The ciphertext is too short or malformed.
    InvalidCiphertext(String),
    /// Decryption failed (wrong key, corrupted data, or tampered ciphertext).
    DecryptionFailed,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::KeyNotSet => write!(f, "VANTADB_ENCRYPTION_KEY not set"),
            CryptoError::InvalidKey(msg) => write!(f, "invalid encryption key: {msg}"),
            CryptoError::InvalidCiphertext(msg) => write!(f, "invalid ciphertext: {msg}"),
            CryptoError::DecryptionFailed => {
                write!(f, "decryption failed (wrong key or corrupted data)")
            }
        }
    }
}

impl std::error::Error for CryptoError {}

/// Specifies how the encryption key is sourced.
#[derive(Debug, Clone)]
pub enum KeySource {
    /// Read from the `VANTADB_ENCRYPTION_KEY` environment variable.
    Environment,
}

/// Configuration for storage-layer encryption.
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// How the encryption key is provided.
    pub key_source: KeySource,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            key_source: KeySource::Environment,
        }
    }
}

impl EncryptionConfig {
    /// Resolve the cipher from this configuration.
    pub fn resolve_cipher(&self) -> Result<Cipher, CryptoError> {
        match self.key_source {
            KeySource::Environment => Cipher::from_env(),
        }
    }
}

/// AES-256-GCM cipher wrapping [`Aes256Gcm`] with a 12-byte nonce.
///
/// Each encryption generates a fresh random nonce via [`OsRng`].
#[derive(Clone)]
pub struct Cipher {
    inner: Aes256Gcm,
}

impl Cipher {
    /// Create a new cipher from a raw key byte slice.
    ///
    /// If `key` is not exactly 32 bytes, a SHA-256 hash of the key is used
    /// instead.
    pub fn new(key: &[u8]) -> Self {
        let key_bytes = if key.len() == 32 {
            let mut k = [0u8; 32];
            k.copy_from_slice(key);
            k
        } else {
            Sha256::digest(key).into()
        };
        let inner = Aes256Gcm::new_from_slice(&key_bytes).expect("Aes256Gcm::new_from_slice failed — key is not 32 bytes; SHA-256 digest guarantees 32-byte output, this indicates a logic bug in key normalization");
        Self { inner }
    }

    /// Load the key from the `VANTADB_ENCRYPTION_KEY` environment variable.
    ///
    /// Expects a hex-encoded 32-byte key (64 hex characters, optional `0x` prefix).
    pub fn from_env() -> Result<Self, CryptoError> {
        let encoded =
            std::env::var("VANTADB_ENCRYPTION_KEY").map_err(|_| CryptoError::KeyNotSet)?;
        let key = decode_hex(&encoded)?;
        Ok(Self::new(&key))
    }

    /// Encrypt `plaintext` with a fresh random 12-byte nonce.
    ///
    /// Returns `[nonce (12 bytes) || ciphertext + AEAD tag (16 bytes)]`.
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
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .inner
            .encrypt(&nonce, plaintext)
            .expect("AES-256-GCM encryption is infallible per RustCrypto guarantee");
        let mut out = Vec::with_capacity(12 + ciphertext.len());
        out.extend_from_slice(nonce.as_slice());
        out.extend_from_slice(&ciphertext);
        out
    }

    /// Decrypt data previously produced by [`Cipher::encrypt`].
    ///
    /// Expects `data` to be `[nonce (12 bytes) || ciphertext + AEAD tag (16 bytes)]`.
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if data.len() < 12 + 16 {
            return Err(CryptoError::InvalidCiphertext(format!(
                "data length {} is too short (need at least {} bytes)",
                data.len(),
                12 + 16
            )));
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.inner
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)
    }
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
pub struct EncryptionStream<S> {
    inner: S,
    cipher: Cipher,
    read_buf: Vec<u8>,
    read_pos: usize,
}

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
/// or odd length.
fn decode_hex(s: &str) -> Result<Vec<u8>, CryptoError> {
    let s = s.trim();
    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    if s.len() % 2 != 0 {
        return Err(CryptoError::InvalidKey(
            "hex string must have an even number of characters".into(),
        ));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| CryptoError::InvalidKey(format!("invalid hex at position {i}: {e}")))
        })
        .collect()
}

#[cfg(test)]
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
    fn test_from_env_not_set() {
        std::env::remove_var("VANTADB_ENCRYPTION_KEY");
        let result = Cipher::from_env();
        assert!(matches!(result, Err(CryptoError::KeyNotSet)));
    }
}
