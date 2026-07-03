use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;

use crate::error::Result;

const MAGIC_BYTES: &[u8; 8] = b"VTDBv001";
pub const HEADER_SIZE: usize = 72;
pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub const MIN_COMPAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageHeader {
    pub version: u32,
    pub flags: u32,
    pub min_compat_version: u32,
}

impl StorageHeader {
    pub fn current() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
            flags: 0,
            min_compat_version: MIN_COMPAT_VERSION,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(HEADER_SIZE);
        buf.extend_from_slice(MAGIC_BYTES);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.flags.to_le_bytes());
        buf.extend_from_slice(&self.min_compat_version.to_le_bytes());
        buf.resize(HEADER_SIZE, 0);
        buf
    }

    pub fn decode(bytes: &[u8]) -> std::result::Result<Self, SchemaError> {
        if bytes.len() < HEADER_SIZE {
            return Err(SchemaError::Invalid(format!(
                "header too short: {} bytes, expected {HEADER_SIZE}",
                bytes.len()
            )));
        }
        if &bytes[0..8] != MAGIC_BYTES {
            let found = &bytes[0..8];
            return Err(SchemaError::Invalid(format!(
                "bad magic bytes: {found:?}, expected {magic:?}",
                magic = MAGIC_BYTES
            )));
        }
        let version = u32::from_le_bytes(
            bytes[8..12]
                .try_into()
                .map_err(|_| SchemaError::Invalid("cannot read version field".into()))?,
        );
        let flags = u32::from_le_bytes(
            bytes[12..16]
                .try_into()
                .map_err(|_| SchemaError::Invalid("cannot read flags field".into()))?,
        );
        let min_compat_version =
            u32::from_le_bytes(bytes[16..20].try_into().map_err(|_| {
                SchemaError::Invalid("cannot read min_compat_version field".into())
            })?);
        Ok(Self {
            version,
            flags,
            min_compat_version,
        })
    }

    pub fn is_compatible(&self) -> std::result::Result<(), SchemaError> {
        if self.version < self.min_compat_version {
            return Err(SchemaError::TooOld {
                file_version: self.version,
                min_required: self.min_compat_version,
            });
        }
        if self.version > CURRENT_SCHEMA_VERSION {
            return Err(SchemaError::TooNew {
                file_version: self.version,
                max_supported: CURRENT_SCHEMA_VERSION,
            });
        }
        Ok(())
    }

    pub fn read_from(path: &Path) -> std::result::Result<Option<Self>, SchemaError> {
        if !path.exists() {
            return Ok(None);
        }
        let mut file = std::fs::File::open(path)
            .map_err(|e| SchemaError::Invalid(format!("cannot open schema file: {e}")))?;
        let mut buf = vec![0u8; HEADER_SIZE];
        let n = file
            .read(&mut buf)
            .map_err(|e| SchemaError::Invalid(format!("cannot read schema file: {e}")))?;
        if n < HEADER_SIZE {
            return Err(SchemaError::Invalid(format!(
                "schema file truncated: read {n} bytes, expected {HEADER_SIZE}"
            )));
        }
        Ok(Some(Self::decode(&buf)?))
    }

    pub fn write_to(&self, path: &Path) -> std::result::Result<(), SchemaError> {
        let encoded = self.encode();
        let mut file = std::fs::File::create(path)
            .map_err(|e| SchemaError::Invalid(format!("cannot create schema file: {e}")))?;
        file.write_all(&encoded)
            .map_err(|e| SchemaError::Invalid(format!("cannot write schema file: {e}")))?;
        file.sync_all()
            .map_err(|e| SchemaError::Invalid(format!("cannot sync schema file: {e}")))?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Storage schema version {file_version} is too old. Minimum required: {min_required}")]
    TooOld {
        file_version: u32,
        min_required: u32,
    },
    #[error(
        "Storage schema version {file_version} is too new. Maximum supported: {max_supported}"
    )]
    TooNew {
        file_version: u32,
        max_supported: u32,
    },
    #[error("Invalid storage header: {0}")]
    Invalid(String),
}

impl From<SchemaError> for crate::error::VantaError {
    fn from(e: SchemaError) -> Self {
        crate::error::VantaError::SchemaError(e.to_string())
    }
}

pub fn load_or_create_schema(base_path: &Path) -> Result<StorageHeader> {
    let schema_path = base_path.join(".vanta.schema");

    match StorageHeader::read_from(&schema_path)? {
        Some(header) => {
            header.is_compatible()?;
            Ok(header)
        }
        None => {
            let header = StorageHeader::current();
            header.write_to(&schema_path)?;
            Ok(header)
        }
    }
}

pub fn check_schema_compatibility(base_path: &Path) -> Result<StorageHeader> {
    let schema_path = base_path.join(".vanta.schema");

    match StorageHeader::read_from(&schema_path)? {
        Some(header) => {
            header.is_compatible()?;
            Ok(header)
        }
        None => Err(crate::error::VantaError::SchemaError(
            "no schema file found; database may be uninitialised or corrupt".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let h = StorageHeader::current();
        let bytes = h.encode();
        assert_eq!(bytes.len(), HEADER_SIZE);
        let decoded = StorageHeader::decode(&bytes).unwrap();
        assert_eq!(decoded, h);
    }

    #[test]
    fn decode_invalid_magic() {
        let bytes = vec![0u8; HEADER_SIZE];
        let err = StorageHeader::decode(&bytes).unwrap_err();
        assert!(matches!(err, SchemaError::Invalid(_)));
    }

    #[test]
    fn decode_too_short() {
        let bytes = vec![0u8; 10];
        let err = StorageHeader::decode(&bytes).unwrap_err();
        assert!(matches!(err, SchemaError::Invalid(_)));
    }

    #[test]
    fn current_version_is_compatible() {
        let h = StorageHeader::current();
        assert!(h.is_compatible().is_ok());
    }

    #[test]
    fn too_old_is_rejected() {
        let h = StorageHeader {
            version: 0,
            flags: 0,
            min_compat_version: 1,
        };
        let err = h.is_compatible().unwrap_err();
        assert!(matches!(err, SchemaError::TooOld { .. }));
    }

    #[test]
    fn too_new_is_rejected() {
        let h = StorageHeader {
            version: 999,
            flags: 0,
            min_compat_version: 999,
        };
        let err = h.is_compatible().unwrap_err();
        assert!(matches!(err, SchemaError::TooNew { .. }));
    }

    #[test]
    fn file_write_read_roundtrip() {
        let dir = std::env::temp_dir().join("vanta_schema_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(".vanta.schema");

        let h = StorageHeader::current();
        h.write_to(&path).unwrap();
        let read = StorageHeader::read_from(&path).unwrap().unwrap();
        assert_eq!(read, h);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_missing_returns_none() {
        let path = Path::new("/tmp/nonexistent_schema_file_for_testing");
        let result = StorageHeader::read_from(path).unwrap();
        assert!(result.is_none());
    }
}
