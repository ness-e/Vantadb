use crate::error::{Result, VantaError};
use web_time::SystemTime;

/// Unified 16-byte binary header for all VantaDB persisted files.
/// Ensures format, schema and data integrity on load/recovery.
///
/// Layout:
/// - Magic bytes: 4 bytes (e.g. b"VWAL", b"VNDX", b"VFLE")
/// - Format version: 2 bytes (u16, little-endian)
/// - Schema version: 2 bytes (u16, little-endian)
/// - Timestamp: 8 bytes (u64, little-endian, creation epoch in ms)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VantaHeader {
    pub magic: [u8; 4],
    pub format_version: u16,
    pub schema_version: u16,
    pub timestamp: u64,
}

impl VantaHeader {
    pub const SIZE: usize = 16;

    /// Create a new header with current system timestamp.
    pub fn new(magic: [u8; 4], format_version: u16, schema_version: u16) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            magic,
            format_version,
            schema_version,
            timestamp,
        }
    }

    /// Serialize into a static 16-byte array.
    pub fn serialize(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..6].copy_from_slice(&self.format_version.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.schema_version.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.timestamp.to_le_bytes());
        bytes
    }

    /// Deserialize from a slice of bytes.
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Binary header slice is too short (less than 16 bytes)",
            )));
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);
        let format_version = u16::from_le_bytes([bytes[4], bytes[5]]);
        let schema_version = u16::from_le_bytes([bytes[6], bytes[7]]);
        let timestamp = u64::from_le_bytes(bytes[8..16].try_into().expect("header bytes slice fits u64"));
        Ok(Self {
            magic,
            format_version,
            schema_version,
            timestamp,
        })
    }

    /// Validates the magic bytes and format version against expected values.
    /// Returns VantaError::IncompatibleFormat on mismatch.
    pub fn validate(
        &self,
        expected_magic: [u8; 4],
        expected_version: u16,
        hint: &str,
    ) -> Result<()> {
        if self.magic != expected_magic || self.format_version != expected_version {
            return Err(VantaError::IncompatibleFormat {
                expected_magic,
                expected_version,
                found_magic: self.magic,
                found_version: self.format_version,
                hint: hint.to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_roundtrip() {
        let h = VantaHeader::new(*b"VWAL", 2, 1);
        let bytes = h.serialize();
        assert_eq!(bytes.len(), 16);
        let deserialized = VantaHeader::deserialize(&bytes).unwrap();
        assert_eq!(deserialized.magic, h.magic);
        assert_eq!(deserialized.format_version, h.format_version);
        assert_eq!(deserialized.schema_version, h.schema_version);
        assert_eq!(deserialized.timestamp, h.timestamp);
    }

    #[test]
    fn deserialize_too_short() {
        let result = VantaHeader::deserialize(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn validate_accepts_matching() {
        let h = VantaHeader::new(*b"VFLE", 1, 0);
        assert!(h.validate(*b"VFLE", 1, "vfile").is_ok());
    }

    #[test]
    fn validate_rejects_mismatched_magic() {
        let h = VantaHeader::new(*b"VFLE", 1, 0);
        let err = h.validate(*b"VWAL", 1, "expected VWAL").unwrap_err();
        assert!(err.to_string().contains("expected VWAL"));
    }

    #[test]
    fn validate_rejects_mismatched_version() {
        let h = VantaHeader::new(*b"VFLE", 1, 0);
        let err = h.validate(*b"VFLE", 2, "version mismatch").unwrap_err();
        assert!(err.to_string().contains("version mismatch"));
    }

    #[test]
    fn header_size_constant() {
        assert_eq!(VantaHeader::SIZE, 16);
    }

    #[test]
    fn schema_version_is_preserved() {
        let h = VantaHeader::new(*b"VNDX", 3, 42);
        assert_eq!(h.schema_version, 42);
    }

    #[test]
    fn timestamp_is_recent() {
        let h = VantaHeader::new(*b"VTST", 1, 0);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system time is after unix epoch")
            .as_millis() as u64;
        // Allow 5 seconds of clock drift
        assert!(
            h.timestamp + 5000 >= now,
            "timestamp should be within 5s of now"
        );
        assert!(
            h.timestamp <= now + 5000,
            "timestamp should not be in the future"
        );
    }
}
