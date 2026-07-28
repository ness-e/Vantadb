//! Typed columnar storage for metadata fields (JSON Shredding).
//!
//! # Purpose
//!
//! VantaDB stores memory records with metadata (`VantaMemoryMetadata` =
//! `BTreeMap<String, VantaValue>`).  Filtering metadata historically required
//! scanning every record (PostFilter) or using bitset-based derived indexes
//! (InFilter / PreFilter).  JSON Shredding infers a typed schema from the
//! metadata at insert time and stores values as binary **columns**, enabling
//! fast typed lookups during filtering without deserialising the full record.
//!
//! # Binary format
//!
//! Each `ShreddedRowStore::put` call writes a single key-value entry into the
//! `BackendPartition::InternalMetadata` partition under the key
//! `shred::{node_id}`.  The value is a concatenation of field entries:
//!
//! ```text
//! [key_len:4 LE][key:key_len bytes][type:1][value_bytes]
//! ```
//!
//! | Type code | Rust type     | Value encoding                 |
//! |-----------|---------------|--------------------------------|
//! | `0`       | `i64`         | 8-byte little-endian           |
//! | `1`       | `f64`         | 8-byte little-endian           |
//! | `2`       | `bool`        | 1 byte (`0` or `1`)            |
//! | `3`       | `String`      | 4-byte LE length + UTF-8 bytes |
//!
//! # Limitations (Phase 1)
//!
//! - **Best-effort**: If shredding fails (e.g. unsupported type), the record
//!   falls through to the existing PostFilter path — no data loss.
//! - **Last-write-wins**: When a field type changes between insertions, the
//!   most recent type is stored.  No type-conflict detection.
//! - **Flat schema only**: Nested JSON is not supported.  List types are skipped.
//!
//! # Integration
//!
//! - **Insert path**: `put_one` in `api.rs` calls `ShreddedRowStore::put` after
//!   the node is stored.
//! - **Filter path**: `bitset_from_filters` in `search/mod.rs` tries the
//!   shredded store for single-field filter lookups before falling back to
//!   the full record scan.
//! - **Delete path**: Not yet wired — shredded entries survive node deletion
//!   until garbage collection (Phase 2).

use crate::backend::{BackendPartition, StorageBackend};
use crate::query::RelOp;
use crate::VantaValue;
use std::collections::{BTreeMap, HashMap};

// ─── Schema Types ──────────────────────────────────────────────

/// A single typed field value recovered from the shredded column store.
#[derive(Debug, Clone, PartialEq)]
pub enum ShreddedField {
    /// 64-bit signed integer.
    I64(i64),
    /// 64-bit floating point.
    F64(f64),
    /// Boolean.
    Bool(bool),
    /// UTF-8 string.
    String(String),
    /// Explicit null / unsupported type.
    Null,
}

/// Enum tag for the in-memory schema — which type a field name has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShreddedFieldType {
    /// Signed 64-bit integer.
    I64,
    /// 64-bit floating point.
    F64,
    /// Boolean.
    Bool,
    /// UTF-8 string.
    String,
    /// Null / untyped.
    Null,
}

/// In-memory schema snapshot: field name → field type.
#[derive(Debug, Clone, Default)]
pub struct ShreddedSchema {
    /// Map of field names to their inferred types.
    pub fields: HashMap<String, ShreddedFieldType>,
}

// ─── Type Inference ────────────────────────────────────────────

/// Infer the shredded type from a `VantaValue`.
///
/// List values (ListString, ListInt, …) and `DateTime` fall back to `Null`
/// because the shred store only supports scalar types in Phase 1.
pub fn infer_field_type(value: &VantaValue) -> ShreddedFieldType {
    match value {
        VantaValue::Int(_) => ShreddedFieldType::I64,
        VantaValue::Float(_) => ShreddedFieldType::F64,
        VantaValue::Bool(_) => ShreddedFieldType::Bool,
        VantaValue::String(_) => ShreddedFieldType::String,
        _ => ShreddedFieldType::Null,
    }
}

/// Check whether a shredded field value matches the expected filter value
/// using the given relational operator.
///
/// Numeric types (`I64`, `F64`) support all six operators.
/// `Bool` and `String` only support `Eq` / `Neq` — other operators return `false`.
/// Type mismatches always return `false`.
pub fn matches_shredded(field: &ShreddedField, op: &RelOp, expected: &VantaValue) -> bool {
    match (field, op, expected) {
        // I64 — all operators
        (ShreddedField::I64(a), RelOp::Eq, VantaValue::Int(b)) => a == b,
        (ShreddedField::I64(a), RelOp::Neq, VantaValue::Int(b)) => a != b,
        (ShreddedField::I64(a), RelOp::Gt, VantaValue::Int(b)) => a > b,
        (ShreddedField::I64(a), RelOp::Lt, VantaValue::Int(b)) => a < b,
        (ShreddedField::I64(a), RelOp::Gte, VantaValue::Int(b)) => a >= b,
        (ShreddedField::I64(a), RelOp::Lte, VantaValue::Int(b)) => a <= b,

        // F64 — all operators
        (ShreddedField::F64(a), RelOp::Eq, VantaValue::Float(b)) => a == b,
        (ShreddedField::F64(a), RelOp::Neq, VantaValue::Float(b)) => a != b,
        (ShreddedField::F64(a), RelOp::Gt, VantaValue::Float(b)) => a > b,
        (ShreddedField::F64(a), RelOp::Lt, VantaValue::Float(b)) => a < b,
        (ShreddedField::F64(a), RelOp::Gte, VantaValue::Float(b)) => a >= b,
        (ShreddedField::F64(a), RelOp::Lte, VantaValue::Float(b)) => a <= b,

        // Bool — only Eq / Neq
        (ShreddedField::Bool(a), RelOp::Eq, VantaValue::Bool(b)) => a == b,
        (ShreddedField::Bool(a), RelOp::Neq, VantaValue::Bool(b)) => a != b,

        // String — only Eq / Neq
        (ShreddedField::String(a), RelOp::Eq, VantaValue::String(b)) => a == b,
        (ShreddedField::String(a), RelOp::Neq, VantaValue::String(b)) => a != b,

        _ => false,
    }
}

// ─── Row Store ─────────────────────────────────────────────────

/// On-disk typed column store for shredded metadata fields.
///
/// Each row is stored as a single binary blob under
/// `BackendPartition::InternalMetadata` with key `shred::{node_id}`.
pub struct ShreddedRowStore;

impl ShreddedRowStore {
    /// Serialise metadata fields to binary and store them in the backend.
    ///
    /// Fields whose type maps to `Null` are silently skipped.
    /// If *all* fields are skipped the operation is a no-op.
    ///
    /// The binary format is:
    /// ```text
    /// [key_len:4 LE][key:key_len][type_tag:1][value_bytes]
    /// ```
    /// repeated for every field.
    pub(crate) fn put(
        node_id: u128,
        fields: &BTreeMap<String, VantaValue>,
        backend: &dyn StorageBackend,
    ) -> crate::error::Result<()> {
        let mut buf = Vec::new();
        for (key, value) in fields {
            let ty = infer_field_type(value);
            let value_bytes = match value {
                VantaValue::Int(v) => v.to_le_bytes().to_vec(),
                VantaValue::Float(v) => v.to_le_bytes().to_vec(),
                VantaValue::Bool(v) => vec![*v as u8],
                VantaValue::String(v) => {
                    let len = v.len() as u32;
                    let mut b = len.to_le_bytes().to_vec();
                    b.extend_from_slice(v.as_bytes());
                    b
                }
                _ => continue, // skip DateTime, lists, Null
            };
            // Format: key_len(4) + key + type(1) + value_bytes
            let key_bytes = key.as_bytes();
            buf.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(key_bytes);
            buf.push(ty as u8);
            buf.extend_from_slice(&value_bytes);
        }
        if buf.is_empty() {
            return Ok(());
        }
        let store_key = format!("shred::{}", node_id).into_bytes();
        backend.put(BackendPartition::InternalMetadata, &store_key, &buf)
    }

    /// Retrieve all shredded fields for the given node.
    ///
    /// Returns `None` when no shredded data exists for this node.
    pub(crate) fn get(
        node_id: u128,
        backend: &dyn StorageBackend,
    ) -> crate::error::Result<Option<HashMap<String, ShreddedField>>> {
        let store_key = format!("shred::{}", node_id).into_bytes();
        let data = backend.get(BackendPartition::InternalMetadata, &store_key)?;
        let Some(data) = data else {
            return Ok(None);
        };

        let mut fields = HashMap::new();
        let mut offset = 0usize;
        while offset + 4 <= data.len() {
            let key_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            if offset + key_len + 1 > data.len() {
                break;
            }
            let key = std::str::from_utf8(&data[offset..offset + key_len])
                .unwrap_or("")
                .to_string();
            offset += key_len;
            let ty = data[offset];
            offset += 1;
            match ty {
                0 => {
                    // I64
                    if offset + 8 > data.len() {
                        break;
                    }
                    let v = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
                    fields.insert(key, ShreddedField::I64(v));
                    offset += 8;
                }
                1 => {
                    // F64
                    if offset + 8 > data.len() {
                        break;
                    }
                    let v = f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
                    fields.insert(key, ShreddedField::F64(v));
                    offset += 8;
                }
                2 => {
                    // Bool
                    if offset + 1 > data.len() {
                        break;
                    }
                    fields.insert(key, ShreddedField::Bool(data[offset] != 0));
                    offset += 1;
                }
                3 => {
                    // String
                    if offset + 4 > data.len() {
                        break;
                    }
                    let str_len =
                        u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
                    offset += 4;
                    if offset + str_len > data.len() {
                        break;
                    }
                    let s = std::string::String::from_utf8_lossy(&data[offset..offset + str_len])
                        .to_string();
                    fields.insert(key, ShreddedField::String(s));
                    offset += str_len;
                }
                _ => break, // unknown type tag – truncate
            }
        }
        Ok(Some(fields))
    }

    /// Delete shredded fields for a node from the backend.
    ///
    /// Safe to call even when no shredded data exists for this node.
    #[allow(dead_code)]
    pub(crate) fn delete(node_id: u128, backend: &dyn StorageBackend) -> crate::error::Result<()> {
        let store_key = format!("shred::{}", node_id).into_bytes();
        backend.delete(BackendPartition::InternalMetadata, &store_key)
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::backends::in_memory::InMemoryBackend;
    use crate::VantaValue;

    fn test_backend() -> InMemoryBackend {
        InMemoryBackend::new()
    }

    // ── infer_field_type ──────────────────────────────────────

    #[test]
    fn test_infer_field_types() {
        assert_eq!(
            infer_field_type(&VantaValue::Int(42)),
            ShreddedFieldType::I64
        );
        assert_eq!(
            infer_field_type(&VantaValue::Float(std::f64::consts::PI)),
            ShreddedFieldType::F64
        );
        assert_eq!(
            infer_field_type(&VantaValue::Bool(true)),
            ShreddedFieldType::Bool
        );
        assert_eq!(
            infer_field_type(&VantaValue::String("hi".into())),
            ShreddedFieldType::String
        );
        assert_eq!(infer_field_type(&VantaValue::Null), ShreddedFieldType::Null);
        // List types → Null (not supported in Phase 1)
        assert_eq!(
            infer_field_type(&VantaValue::ListInt(vec![1, 2])),
            ShreddedFieldType::Null
        );
    }

    // ── put / get round-trip ──────────────────────────────────

    #[test]
    fn test_shredded_roundtrip() {
        let backend = test_backend();
        let mut fields = BTreeMap::new();
        fields.insert("age".into(), VantaValue::Int(30));
        fields.insert("score".into(), VantaValue::Float(9.5));
        fields.insert("active".into(), VantaValue::Bool(true));
        fields.insert("name".into(), VantaValue::String("Alice".into()));

        ShreddedRowStore::put(42, &fields, &backend).unwrap();

        let result = ShreddedRowStore::get(42, &backend).unwrap().unwrap();
        assert_eq!(result.get("age"), Some(&ShreddedField::I64(30)));
        assert_eq!(result.get("score"), Some(&ShreddedField::F64(9.5)));
        assert_eq!(result.get("active"), Some(&ShreddedField::Bool(true)));
        assert_eq!(
            result.get("name"),
            Some(&ShreddedField::String("Alice".into()))
        );
        assert_eq!(result.len(), 4);
    }

    // ── delete ────────────────────────────────────────────────

    #[test]
    fn test_shredded_delete() {
        let backend = test_backend();
        let mut fields = BTreeMap::new();
        fields.insert("x".into(), VantaValue::Int(1));
        ShreddedRowStore::put(7, &fields, &backend).unwrap();
        assert!(ShreddedRowStore::get(7, &backend).unwrap().is_some());

        ShreddedRowStore::delete(7, &backend).unwrap();
        assert!(ShreddedRowStore::get(7, &backend).unwrap().is_none());
    }

    // ── schema evolution (last-write-wins) ────────────────────

    #[test]
    fn test_shredded_schema_evolution() {
        let backend = test_backend();

        // First write: "a" is an integer
        let mut f1 = BTreeMap::new();
        f1.insert("a".into(), VantaValue::Int(42));
        ShreddedRowStore::put(1, &f1, &backend).unwrap();
        let r1 = ShreddedRowStore::get(1, &backend).unwrap().unwrap();
        assert_eq!(r1.get("a"), Some(&ShreddedField::I64(42)));

        // Second write: "a" is now a string  (last-write-wins)
        let mut f2 = BTreeMap::new();
        f2.insert("a".into(), VantaValue::String("hello".into()));
        ShreddedRowStore::put(1, &f2, &backend).unwrap();
        let r2 = ShreddedRowStore::get(1, &backend).unwrap().unwrap();
        assert_eq!(r2.get("a"), Some(&ShreddedField::String("hello".into())));
    }

    // ── multiple independent nodes ────────────────────────────

    #[test]
    fn test_shredded_multiple_nodes() {
        let backend = test_backend();

        let mut f1 = BTreeMap::new();
        f1.insert("color".into(), VantaValue::String("red".into()));
        ShreddedRowStore::put(10, &f1, &backend).unwrap();

        let mut f2 = BTreeMap::new();
        f2.insert("color".into(), VantaValue::String("blue".into()));
        f2.insert("size".into(), VantaValue::Int(5));
        ShreddedRowStore::put(20, &f2, &backend).unwrap();

        let mut f3 = BTreeMap::new();
        f3.insert("active".into(), VantaValue::Bool(false));
        ShreddedRowStore::put(30, &f3, &backend).unwrap();

        // Verify each node independently
        let r1 = ShreddedRowStore::get(10, &backend).unwrap().unwrap();
        assert_eq!(r1.len(), 1);
        assert_eq!(r1.get("color"), Some(&ShreddedField::String("red".into())));

        let r2 = ShreddedRowStore::get(20, &backend).unwrap().unwrap();
        assert_eq!(r2.len(), 2);
        assert_eq!(r2.get("color"), Some(&ShreddedField::String("blue".into())));
        assert_eq!(r2.get("size"), Some(&ShreddedField::I64(5)));

        let r3 = ShreddedRowStore::get(30, &backend).unwrap().unwrap();
        assert_eq!(r3.len(), 1);
        assert_eq!(r3.get("active"), Some(&ShreddedField::Bool(false)));

        // Deleted node returns None
        ShreddedRowStore::delete(10, &backend).unwrap();
        assert!(ShreddedRowStore::get(10, &backend).unwrap().is_none());
    }

    // ── large string values ───────────────────────────────────

    #[test]
    fn test_shredded_large_string() {
        let backend = test_backend();
        let large = "A".repeat(100_000);

        let mut fields = BTreeMap::new();
        fields.insert("data".into(), VantaValue::String(large.clone()));
        ShreddedRowStore::put(99, &fields, &backend).unwrap();

        let result = ShreddedRowStore::get(99, &backend).unwrap().unwrap();
        assert_eq!(result.get("data"), Some(&ShreddedField::String(large)));
    }

    // ── matches_shredded (equality, legacy compat) ───────────

    #[test]
    fn test_matches_shredded_eq() {
        // I64 Eq
        assert!(matches_shredded(
            &ShreddedField::I64(10),
            &RelOp::Eq,
            &VantaValue::Int(10)
        ));
        assert!(!matches_shredded(
            &ShreddedField::I64(10),
            &RelOp::Eq,
            &VantaValue::Int(20)
        ));

        // F64 Eq
        assert!(matches_shredded(
            &ShreddedField::F64(3.5),
            &RelOp::Eq,
            &VantaValue::Float(3.5)
        ));
        assert!(!matches_shredded(
            &ShreddedField::F64(3.5),
            &RelOp::Eq,
            &VantaValue::Float(4.0)
        ));

        // Bool Eq
        assert!(matches_shredded(
            &ShreddedField::Bool(true),
            &RelOp::Eq,
            &VantaValue::Bool(true)
        ));
        assert!(!matches_shredded(
            &ShreddedField::Bool(true),
            &RelOp::Eq,
            &VantaValue::Bool(false)
        ));

        // String Eq
        assert!(matches_shredded(
            &ShreddedField::String("hi".into()),
            &RelOp::Eq,
            &VantaValue::String("hi".into())
        ));
        assert!(!matches_shredded(
            &ShreddedField::String("hi".into()),
            &RelOp::Eq,
            &VantaValue::String("bye".into())
        ));

        // type mismatch → false
        assert!(!matches_shredded(
            &ShreddedField::I64(1),
            &RelOp::Eq,
            &VantaValue::String("1".into())
        ));
        assert!(!matches_shredded(
            &ShreddedField::Null,
            &RelOp::Eq,
            &VantaValue::Int(0)
        ));
    }

    // ── I64 comparisons ───────────────────────────────────────

    #[test]
    fn test_matches_shredded_i64_comparisons() {
        let val_10 = ShreddedField::I64(10);
        let val_20 = ShreddedField::I64(20);

        // Eq
        assert!(matches_shredded(&val_10, &RelOp::Eq, &VantaValue::Int(10)));
        assert!(!matches_shredded(&val_10, &RelOp::Eq, &VantaValue::Int(20)));

        // Neq
        assert!(matches_shredded(&val_10, &RelOp::Neq, &VantaValue::Int(20)));
        assert!(!matches_shredded(
            &val_10,
            &RelOp::Neq,
            &VantaValue::Int(10)
        ));

        // Gt
        assert!(matches_shredded(&val_20, &RelOp::Gt, &VantaValue::Int(15)));
        assert!(!matches_shredded(&val_10, &RelOp::Gt, &VantaValue::Int(10)));
        assert!(!matches_shredded(&val_10, &RelOp::Gt, &VantaValue::Int(15)));

        // Lt
        assert!(matches_shredded(&val_10, &RelOp::Lt, &VantaValue::Int(15)));
        assert!(!matches_shredded(&val_20, &RelOp::Lt, &VantaValue::Int(15)));
        assert!(!matches_shredded(&val_10, &RelOp::Lt, &VantaValue::Int(10)));

        // Gte
        assert!(matches_shredded(&val_20, &RelOp::Gte, &VantaValue::Int(15)));
        assert!(matches_shredded(&val_20, &RelOp::Gte, &VantaValue::Int(20)));
        assert!(!matches_shredded(
            &val_10,
            &RelOp::Gte,
            &VantaValue::Int(15)
        ));

        // Lte
        assert!(matches_shredded(&val_10, &RelOp::Lte, &VantaValue::Int(15)));
        assert!(matches_shredded(&val_10, &RelOp::Lte, &VantaValue::Int(10)));
        assert!(!matches_shredded(
            &val_20,
            &RelOp::Lte,
            &VantaValue::Int(15)
        ));
    }

    // ── F64 comparisons ───────────────────────────────────────

    #[test]
    fn test_matches_shredded_f64_comparisons() {
        let val_1_5 = ShreddedField::F64(1.5);
        let val_3_0 = ShreddedField::F64(3.0);

        // Eq
        assert!(matches_shredded(
            &val_3_0,
            &RelOp::Eq,
            &VantaValue::Float(3.0)
        ));
        assert!(!matches_shredded(
            &val_1_5,
            &RelOp::Eq,
            &VantaValue::Float(3.0)
        ));

        // Neq
        assert!(matches_shredded(
            &val_1_5,
            &RelOp::Neq,
            &VantaValue::Float(3.0)
        ));
        assert!(!matches_shredded(
            &val_3_0,
            &RelOp::Neq,
            &VantaValue::Float(3.0)
        ));

        // Gt
        assert!(matches_shredded(
            &val_3_0,
            &RelOp::Gt,
            &VantaValue::Float(2.0)
        ));
        assert!(!matches_shredded(
            &val_1_5,
            &RelOp::Gt,
            &VantaValue::Float(2.0)
        ));
        assert!(!matches_shredded(
            &val_3_0,
            &RelOp::Gt,
            &VantaValue::Float(3.0)
        ));

        // Lt
        assert!(matches_shredded(
            &val_1_5,
            &RelOp::Lt,
            &VantaValue::Float(2.0)
        ));
        assert!(!matches_shredded(
            &val_3_0,
            &RelOp::Lt,
            &VantaValue::Float(2.0)
        ));
        assert!(!matches_shredded(
            &val_1_5,
            &RelOp::Lt,
            &VantaValue::Float(1.5)
        ));

        // Gte
        assert!(matches_shredded(
            &val_3_0,
            &RelOp::Gte,
            &VantaValue::Float(2.0)
        ));
        assert!(matches_shredded(
            &val_3_0,
            &RelOp::Gte,
            &VantaValue::Float(3.0)
        ));
        assert!(!matches_shredded(
            &val_1_5,
            &RelOp::Gte,
            &VantaValue::Float(2.0)
        ));

        // Lte
        assert!(matches_shredded(
            &val_1_5,
            &RelOp::Lte,
            &VantaValue::Float(2.0)
        ));
        assert!(matches_shredded(
            &val_1_5,
            &RelOp::Lte,
            &VantaValue::Float(1.5)
        ));
        assert!(!matches_shredded(
            &val_3_0,
            &RelOp::Lte,
            &VantaValue::Float(2.0)
        ));
    }

    // ── Bool Eq / Neq ─────────────────────────────────────────

    #[test]
    fn test_matches_shredded_bool_eq_ne() {
        let t = ShreddedField::Bool(true);
        let f = ShreddedField::Bool(false);

        assert!(matches_shredded(&t, &RelOp::Eq, &VantaValue::Bool(true)));
        assert!(!matches_shredded(&t, &RelOp::Eq, &VantaValue::Bool(false)));

        assert!(matches_shredded(&t, &RelOp::Neq, &VantaValue::Bool(false)));
        assert!(!matches_shredded(&t, &RelOp::Neq, &VantaValue::Bool(true)));

        // Gt/Lt on Bool — unsupported → false
        assert!(!matches_shredded(&t, &RelOp::Gt, &VantaValue::Bool(false)));
        assert!(!matches_shredded(&f, &RelOp::Lt, &VantaValue::Bool(true)));
    }

    // ── String Eq / Neq ───────────────────────────────────────

    #[test]
    fn test_matches_shredded_string_eq_ne() {
        let hi = ShreddedField::String("hi".into());
        let bye = ShreddedField::String("bye".into());

        assert!(matches_shredded(
            &hi,
            &RelOp::Eq,
            &VantaValue::String("hi".into())
        ));
        assert!(!matches_shredded(
            &hi,
            &RelOp::Eq,
            &VantaValue::String("bye".into())
        ));

        assert!(matches_shredded(
            &hi,
            &RelOp::Neq,
            &VantaValue::String("bye".into())
        ));
        assert!(!matches_shredded(
            &bye,
            &RelOp::Neq,
            &VantaValue::String("bye".into())
        ));

        // Gt on String — unsupported → false
        assert!(!matches_shredded(
            &hi,
            &RelOp::Gt,
            &VantaValue::String("bye".into())
        ));
    }

    // ── empty metadata (no-op put) ────────────────────────────

    #[test]
    fn test_shredded_empty_metadata() {
        let backend = test_backend();
        let empty = BTreeMap::new();
        ShreddedRowStore::put(0, &empty, &backend).unwrap();
        // get returns None because nothing was written
        assert!(ShreddedRowStore::get(0, &backend).unwrap().is_none());
    }

    // ── integration: shredded fast path with comparison filters ─

    #[test]
    fn test_shredded_comparison_filter_integration() {
        let backend = test_backend();

        // Insert 3 records with numeric price metadata
        let mut r1 = BTreeMap::new();
        r1.insert("price".into(), VantaValue::Int(100));
        ShreddedRowStore::put(1, &r1, &backend).unwrap();

        let mut r2 = BTreeMap::new();
        r2.insert("price".into(), VantaValue::Int(200));
        ShreddedRowStore::put(2, &r2, &backend).unwrap();

        let mut r3 = BTreeMap::new();
        r3.insert("price".into(), VantaValue::Int(50));
        ShreddedRowStore::put(3, &r3, &backend).unwrap();

        // Retrieve shredded data and apply comparison filter price > 100
        let nodes = [1u128, 2, 3];
        let expected: Vec<bool> = nodes
            .iter()
            .map(|&id| {
                let shredded = ShreddedRowStore::get(id, &backend).unwrap().unwrap();
                shredded
                    .get("price")
                    .is_some_and(|field| matches_shredded(field, &RelOp::Gt, &VantaValue::Int(100)))
            })
            .collect();

        // Only node 2 (price=200) should match price > 100
        assert_eq!(expected, vec![false, true, false]);

        // Also verify price < 150: nodes 1 (100) and 3 (50) should match
        let expected_lt: Vec<bool> = nodes
            .iter()
            .map(|&id| {
                let shredded = ShreddedRowStore::get(id, &backend).unwrap().unwrap();
                shredded
                    .get("price")
                    .is_some_and(|field| matches_shredded(field, &RelOp::Lt, &VantaValue::Int(150)))
            })
            .collect();

        assert_eq!(expected_lt, vec![true, false, true]);

        // Verify price >= 100 (Gte): nodes 1 (100) and 2 (200)
        let expected_gte: Vec<bool> = nodes
            .iter()
            .map(|&id| {
                let shredded = ShreddedRowStore::get(id, &backend).unwrap().unwrap();
                shredded.get("price").is_some_and(|field| {
                    matches_shredded(field, &RelOp::Gte, &VantaValue::Int(100))
                })
            })
            .collect();

        assert_eq!(expected_gte, vec![true, true, false]);
    }
}
