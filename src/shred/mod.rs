// ponytail: typed columnar buffer access invariants; documented per-call.

//! Typed columnar storage for metadata fields (JSON Shredding).
//!
//! # Purpose
//!
//! VantaDB stores memory records with metadata (`MemoryMetadata` =
//! `BTreeMap<String, Value>`).  Filtering metadata historically required
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
//! | `4`       | omitted field | **zero bytes** — marker only   |
//!
//! # Omitted-field marker (FIND-604)
//!
//! The column store only represents scalars.  Fields whose value is
//! `Null`, `DateTime`, or a list (`ListString`, `ListInt`, …) are written
//! as a **tag-4 entry with no value bytes** instead of disappearing from
//! the row: a decoded [`ShreddedField::Null`](crate::shred::ShreddedField::Null) therefore means *present in
//! the document but not columnar*, which is observably different from
//! *absent*.  A filter touching such a field cannot be decided by the
//! column store — `filter_verdict` reports `DelegateToDocument` so the
//! caller falls back to the full-document evaluator **only** in that
//! case (filter parity between the fast path and the JSON evaluator).
//!
//! Scalar entries are always emitted **before** marker entries, so a
//! pre-marker reader that truncates at the unknown tag still decodes
//! every scalar column (graceful degradation, not corruption).
//!
//! # Limitations (Phase 1)
//!
//! - **Best-effort**: If shredding fails (e.g. unsupported type), the record
//!   falls through to the existing PostFilter path — no data loss.
//! - **Last-write-wins**: When a field type changes between insertions, the
//!   most recent type is stored.  No type-conflict detection.
//! - **Flat schema only**: Nested JSON is not supported.  List values are
//!   recorded as omitted markers, never as columns.
//!
//! # Integration
//!
//! - **Insert path**: `put_one`/`put_batch_inner` in `sdk/api/memory.rs`
//!   route `ShreddedRowStore::put_op` through
//!   `StorageEngine::write_backend_batch` (FIND-651).
//! - **Filter path**: a fast-path consumer evaluates each filter field
//!   with `filter_verdict` over `ShreddedRowStore::get` output and
//!   must honour `DelegateToDocument` / `Absent` per the parity contract
//!   documented there.
//! - **Delete path** (VER-02, wired): `StorageEngine::delete` /
//!   `delete_batch` purge the shredded row (best-effort single delete, batched
//!   op for batches) and `purge_permanent` removes it in its backend batch.

use crate::backend::{BackendPartition, BackendWriteOp, StorageBackend};
use crate::query::RelOp;
use crate::Value;
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
///
/// `#[repr(u8)]` with explicit discriminants (SH-05 / FIND-608): the
/// on-disk type tag **is** the discriminant, so the values are frozen —
/// reordering or renumbering variants would silently corrupt every
/// persisted shredded row. `Null` (`4`) doubles as the **omitted-field
/// marker** (FIND-604): pre-marker encoders never wrote tag `4`, so no
/// existing byte stream is reinterpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ShreddedFieldType {
    /// Signed 64-bit integer.
    I64 = 0,
    /// 64-bit floating point.
    F64 = 1,
    /// Boolean.
    Bool = 2,
    /// UTF-8 string.
    String = 3,
    /// Non-scalar / untyped — on disk, the omitted-field marker
    /// (zero value bytes).
    Null = 4,
}

impl ShreddedFieldType {
    /// Decode an on-disk type tag. `None` for any tag outside `0..=4` —
    /// the format is closed: an unknown tag is corruption (SH-02), never
    /// a silent truncation point.
    pub(crate) fn from_tag(tag: u8) -> Option<Self> {
        Some(match tag {
            0 => Self::I64,
            1 => Self::F64,
            2 => Self::Bool,
            3 => Self::String,
            4 => Self::Null,
            _ => return None,
        })
    }
}

/// In-memory schema snapshot: field name → field type.
#[derive(Debug, Clone, Default)]
pub struct ShreddedSchema {
    /// Map of field names to their inferred types.
    pub fields: HashMap<String, ShreddedFieldType>,
}

// ─── Type Inference ────────────────────────────────────────────

/// Infer the shredded type from a `Value`.
///
/// `Null`, list values (ListString, ListInt, …) and `DateTime` map to
/// `Null` — the column store only supports scalar types; such fields are
/// persisted as omitted-field markers (FIND-604), not dropped.
pub fn infer_field_type(value: &Value) -> ShreddedFieldType {
    match value {
        Value::Int(_) => ShreddedFieldType::I64,
        Value::Float(_) => ShreddedFieldType::F64,
        Value::Bool(_) => ShreddedFieldType::Bool,
        Value::String(_) => ShreddedFieldType::String,
        _ => ShreddedFieldType::Null,
    }
}

/// Check whether a shredded field value matches the expected filter value
/// using the given relational operator.
///
/// Numeric types (`I64`, `F64`) support all six operators.
/// `Bool` and `String` only support `Eq` / `Neq` — other operators return `false`.
/// Type mismatches always return `false`.
pub fn matches_shredded(field: &ShreddedField, op: &RelOp, expected: &Value) -> bool {
    match (field, op, expected) {
        // I64 — all operators
        (ShreddedField::I64(a), RelOp::Eq, Value::Int(b)) => a == b,
        (ShreddedField::I64(a), RelOp::Neq, Value::Int(b)) => a != b,
        (ShreddedField::I64(a), RelOp::Gt, Value::Int(b)) => a > b,
        (ShreddedField::I64(a), RelOp::Lt, Value::Int(b)) => a < b,
        (ShreddedField::I64(a), RelOp::Gte, Value::Int(b)) => a >= b,
        (ShreddedField::I64(a), RelOp::Lte, Value::Int(b)) => a <= b,

        // F64 — all operators
        (ShreddedField::F64(a), RelOp::Eq, Value::Float(b)) => a == b,
        (ShreddedField::F64(a), RelOp::Neq, Value::Float(b)) => a != b,
        (ShreddedField::F64(a), RelOp::Gt, Value::Float(b)) => a > b,
        (ShreddedField::F64(a), RelOp::Lt, Value::Float(b)) => a < b,
        (ShreddedField::F64(a), RelOp::Gte, Value::Float(b)) => a >= b,
        (ShreddedField::F64(a), RelOp::Lte, Value::Float(b)) => a <= b,

        // Bool — only Eq / Neq
        (ShreddedField::Bool(a), RelOp::Eq, Value::Bool(b)) => a == b,
        (ShreddedField::Bool(a), RelOp::Neq, Value::Bool(b)) => a != b,

        // String — only Eq / Neq
        (ShreddedField::String(a), RelOp::Eq, Value::String(b)) => a == b,
        (ShreddedField::String(a), RelOp::Neq, Value::String(b)) => a != b,

        _ => false,
    }
}

/// Verdict of evaluating one filter field against a shredded row
/// (FIND-604 parity contract).
///
/// `#[allow(dead_code)]`: consumed by the unit tests today; the live
/// fast-path consumer lands with the read-side wiring of the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ShreddedFilterVerdict {
    /// The shredded scalar column decides — the payload is the verdict.
    Decided(bool),
    /// The field is present in the source document but holds a
    /// non-scalar value (`Null`, `DateTime`, list) that the column store
    /// omits — the caller MUST evaluate the filter against the full
    /// document; answering here would silently diverge from the JSON
    /// evaluator (e.g. a `ListString` equality filter would return zero
    /// rows from the fast path while the document evaluator narrows
    /// correctly).
    DelegateToDocument,
    /// The field does not appear in the shredded row at all.
    ///
    /// For rows written by this encoder that means the field is absent
    /// from the document; parity with the document evaluators is then
    /// the caller's policy (`matches_memory_filters` and
    /// `matches_advanced_filters` treat an absent field as a non-match
    /// for every op; `physical_plan::evaluate_condition` matches only
    /// `RelOp::Neq`). Rows persisted **before** the omitted-field marker
    /// existed cannot distinguish absent from omitted — a conservative
    /// consumer of such legacy rows delegates `Absent` too.
    Absent,
}

/// Evaluate a single metadata filter `(field, op, expected)` against the
/// fields decoded from a shredded row.
///
/// Returns [`ShreddedFilterVerdict::DelegateToDocument`] **only** when the
/// queried field was recorded as an omitted (non-scalar) marker — scalar
/// fields keep the fast path, so a query over `Int`/`String` columns never
/// pays the document-evaluator cost (pre-mortem risk 3 of the FIND-604
/// task plan).
#[allow(dead_code)] // see ShreddedFilterVerdict — consumer lands with the read wiring
pub(crate) fn filter_verdict(
    shredded: &HashMap<String, ShreddedField>,
    field: &str,
    op: &RelOp,
    expected: &Value,
) -> ShreddedFilterVerdict {
    match shredded.get(field) {
        Some(ShreddedField::Null) => ShreddedFilterVerdict::DelegateToDocument,
        Some(actual) => ShreddedFilterVerdict::Decided(matches_shredded(actual, op, expected)),
        None => ShreddedFilterVerdict::Absent,
    }
}

/// Whether the decoded row carries at least one omitted (non-scalar)
/// field — the row-level projection of the per-field marker, for
/// consumers that gate the fast path on "this row is fully scalar".
#[allow(dead_code)] // see ShreddedFilterVerdict — consumer lands with the read wiring
pub(crate) fn has_omitted_fields(shredded: &HashMap<String, ShreddedField>) -> bool {
    shredded
        .values()
        .any(|field| matches!(field, ShreddedField::Null))
}

// ─── Row Store ─────────────────────────────────────────────────

/// On-disk typed column store for shredded metadata fields.
///
/// Each row is stored as a single binary blob under
/// `BackendPartition::InternalMetadata` with key `shred::{node_id}`.
pub struct ShreddedRowStore;

impl ShreddedRowStore {
    /// Backend key under which a node's shredded fields are stored
    /// (`BackendPartition::InternalMetadata`).
    pub(crate) fn store_key(node_id: u128) -> Vec<u8> {
        format!("shred::{}", node_id).into_bytes()
    }

    /// Write the `[key_len:4 LE][key][type:1]` entry head shared by scalar
    /// values and omitted-field markers.
    fn write_entry_head(buf: &mut Vec<u8>, key: &str, ty: ShreddedFieldType) {
        let key_bytes = key.as_bytes();
        buf.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(key_bytes);
        buf.push(ty as u8);
    }

    /// Serialise `fields` to the row-store binary format. `None` only when
    /// `fields` is empty (same no-op contract as [`Self::put`]).
    ///
    /// FIND-604: non-scalar fields (`Null`, `DateTime`, lists) are written
    /// as zero-length `Null`-tag marker entries — never silently dropped,
    /// so a reader can distinguish *omitted* from *absent*. All scalar
    /// entries are emitted first and all markers last: a pre-marker
    /// decoder truncates at the first unknown tag yet still recovers
    /// every scalar column.
    fn encode_fields(fields: &BTreeMap<String, Value>) -> Option<Vec<u8>> {
        if fields.is_empty() {
            return None;
        }
        let mut buf = Vec::new();
        // Pass 1: scalar columns.
        for (key, value) in fields {
            let value_bytes = match value {
                Value::Int(v) => v.to_le_bytes().to_vec(),
                Value::Float(v) => v.to_le_bytes().to_vec(),
                Value::Bool(v) => vec![*v as u8],
                Value::String(v) => {
                    let len = v.len() as u32;
                    let mut b = len.to_le_bytes().to_vec();
                    b.extend_from_slice(v.as_bytes());
                    b
                }
                _ => continue, // non-scalar — emitted as a marker in pass 2
            };
            Self::write_entry_head(&mut buf, key, infer_field_type(value));
            buf.extend_from_slice(&value_bytes);
        }
        // Pass 2: omitted-field markers (tag 4, zero value bytes).
        for (key, value) in fields {
            if infer_field_type(value) == ShreddedFieldType::Null {
                Self::write_entry_head(&mut buf, key, ShreddedFieldType::Null);
            }
        }
        Some(buf)
    }

    /// Serialise metadata fields to binary and store them in the backend.
    ///
    /// Non-scalar fields are recorded as omitted-field markers (FIND-604);
    /// a call with empty `fields` is a no-op.
    ///
    /// The binary format is:
    /// ```text
    /// [key_len:4 LE][key:key_len][type_tag:1][value_bytes]
    /// ```
    /// repeated for every field.
    ///
    /// NOTE (FIND-651): this writes `backend` directly, bypassing the engine's
    /// `insert_lock` quiesce barrier — it is therefore **test-only**. Live
    /// engine callers must route the write through [`Self::put_op`] +
    /// `StorageEngine::write_backend_batch` (which holds the lock).
    #[cfg(test)]
    pub(crate) fn put(
        node_id: u128,
        fields: &BTreeMap<String, Value>,
        backend: &dyn StorageBackend,
    ) -> crate::error::Result<()> {
        let Some(buf) = Self::encode_fields(fields) else {
            return Ok(());
        };
        backend.put(
            BackendPartition::InternalMetadata,
            &Self::store_key(node_id),
            &buf,
        )
    }

    /// Batch form of [`Self::put`]: the same backend put as a
    /// `BackendWriteOp` so live-engine callers can route it through
    /// `StorageEngine::write_backend_batch`, which acquires `insert_lock`
    /// (FIND-651 — keeps the write out of a concurrent `create_snapshot`
    /// mirror). `None` when there is nothing to store.
    pub(crate) fn put_op(
        node_id: u128,
        fields: &BTreeMap<String, Value>,
    ) -> Option<BackendWriteOp> {
        Self::encode_fields(fields).map(|value| BackendWriteOp::Put {
            partition: BackendPartition::InternalMetadata,
            key: Self::store_key(node_id),
            value,
        })
    }

    /// Corruption of a stored shredded row: must surface, never degrade
    /// into a silent partial map (SH-02 / FIND-607 — a partial decode used
    /// to make records disappear from results with no error).
    fn corrupt_row(reason: &str) -> crate::error::Error {
        crate::error::Error::serialization(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("shredded row: {reason}"),
        ))
    }

    /// Consume exactly `n` bytes at `*offset` — the arity guard for the
    /// whole decoder (SH-02). Truncation and length overflow are errors,
    /// not silent stops.
    fn take<'a>(data: &'a [u8], offset: &mut usize, n: usize) -> crate::error::Result<&'a [u8]> {
        let end = offset
            .checked_add(n)
            .ok_or_else(|| Self::corrupt_row("length overflow"))?;
        let slice = data
            .get(*offset..end)
            .ok_or_else(|| Self::corrupt_row("truncated entry"))?;
        *offset = end;
        Ok(slice)
    }

    /// [`Self::take`] specialised to fixed-width values — no `try_into`,
    /// no `unwrap`: the slice `take` returns is exactly `N` bytes.
    fn take_array<const N: usize>(
        data: &[u8],
        offset: &mut usize,
    ) -> crate::error::Result<[u8; N]> {
        let slice = Self::take(data, offset, N)?;
        let mut arr = [0u8; N];
        arr.copy_from_slice(slice);
        Ok(arr)
    }

    /// Retrieve all shredded fields for the given node.
    ///
    /// Returns `None` when no shredded data exists for this node.
    /// Omitted (non-scalar) fields decode as [`ShreddedField::Null`]
    /// marker entries — see the module docs for the FIND-604 parity
    /// contract. A malformed row is an `Err`, never a partial map (SH-02).
    pub(crate) fn get(
        node_id: u128,
        backend: &dyn StorageBackend,
    ) -> crate::error::Result<Option<HashMap<String, ShreddedField>>> {
        let store_key = Self::store_key(node_id);
        let data = backend.get(BackendPartition::InternalMetadata, &store_key)?;
        let Some(data) = data else {
            return Ok(None);
        };

        let mut fields = HashMap::new();
        let mut offset = 0usize;
        while offset < data.len() {
            let key_len = u32::from_le_bytes(Self::take_array::<4>(&data, &mut offset)?) as usize;
            let key_bytes = Self::take(&data, &mut offset, key_len)?;
            // SH-02: a non-UTF-8 field name is corruption — surface it.
            // The old `unwrap_or("")` aliased every bad key to "" instead
            // of failing.
            let key = std::str::from_utf8(key_bytes)
                .map_err(|_| Self::corrupt_row("field name is not valid UTF-8"))?
                .to_string();
            let tag = *Self::take(&data, &mut offset, 1)?
                .first()
                .ok_or_else(|| Self::corrupt_row("missing type tag"))?;
            let ty = ShreddedFieldType::from_tag(tag)
                .ok_or_else(|| Self::corrupt_row("unknown type tag"))?;
            let field = match ty {
                ShreddedFieldType::I64 => ShreddedField::I64(i64::from_le_bytes(
                    Self::take_array::<8>(&data, &mut offset)?,
                )),
                ShreddedFieldType::F64 => ShreddedField::F64(f64::from_le_bytes(
                    Self::take_array::<8>(&data, &mut offset)?,
                )),
                ShreddedFieldType::Bool => {
                    let byte = *Self::take(&data, &mut offset, 1)?
                        .first()
                        .ok_or_else(|| Self::corrupt_row("missing bool value"))?;
                    ShreddedField::Bool(byte != 0)
                }
                ShreddedFieldType::String => {
                    let str_len =
                        u32::from_le_bytes(Self::take_array::<4>(&data, &mut offset)?) as usize;
                    let bytes = Self::take(&data, &mut offset, str_len)?;
                    // SH-02: strict UTF-8 — lossy decoding hid corruption.
                    let s = String::from_utf8(bytes.to_vec())
                        .map_err(|_| Self::corrupt_row("string value is not valid UTF-8"))?;
                    ShreddedField::String(s)
                }
                // FIND-604 omitted-field marker: tag present, zero value bytes.
                ShreddedFieldType::Null => ShreddedField::Null,
            };
            fields.insert(key, field);
        }
        Ok(Some(fields))
    }

    /// Delete shredded fields for a node from the backend.
    ///
    /// Safe to call even when no shredded data exists for this node.
    /// Wired into the delete path by VER-02 (`StorageEngine::delete`).
    pub(crate) fn delete(node_id: u128, backend: &dyn StorageBackend) -> crate::error::Result<()> {
        backend.delete(
            BackendPartition::InternalMetadata,
            &Self::store_key(node_id),
        )
    }

    /// Batch form of [`Self::delete`] (VER-02): the same backend delete as a
    /// `BackendWriteOp` so `delete_batch`/`purge_permanent` can purge the
    /// shredded row atomically inside their existing batch.
    pub(crate) fn delete_op(node_id: u128) -> BackendWriteOp {
        BackendWriteOp::Delete {
            partition: BackendPartition::InternalMetadata,
            key: Self::store_key(node_id),
        }
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
#[allow(missing_docs, clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::backends::in_memory::InMemoryBackend;
    use crate::Value;

    fn test_backend() -> InMemoryBackend {
        InMemoryBackend::new()
    }

    // ── infer_field_type ──────────────────────────────────────

    #[test]
    fn test_infer_field_types() {
        assert_eq!(infer_field_type(&Value::Int(42)), ShreddedFieldType::I64);
        assert_eq!(
            infer_field_type(&Value::Float(std::f64::consts::PI)),
            ShreddedFieldType::F64
        );
        assert_eq!(
            infer_field_type(&Value::Bool(true)),
            ShreddedFieldType::Bool
        );
        assert_eq!(
            infer_field_type(&Value::String("hi".into())),
            ShreddedFieldType::String
        );
        assert_eq!(infer_field_type(&Value::Null), ShreddedFieldType::Null);
        // List types → Null (not supported in Phase 1)
        assert_eq!(
            infer_field_type(&Value::ListInt(vec![1, 2])),
            ShreddedFieldType::Null
        );
    }

    // ── put / get round-trip ──────────────────────────────────

    #[test]
    fn test_shredded_roundtrip() {
        let backend = test_backend();
        let mut fields = BTreeMap::new();
        fields.insert("age".into(), Value::Int(30));
        fields.insert("score".into(), Value::Float(9.5));
        fields.insert("active".into(), Value::Bool(true));
        fields.insert("name".into(), Value::String("Alice".into()));

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
        fields.insert("x".into(), Value::Int(1));
        ShreddedRowStore::put(7, &fields, &backend).unwrap();
        assert!(ShreddedRowStore::get(7, &backend).unwrap().is_some());

        ShreddedRowStore::delete(7, &backend).unwrap();
        assert!(ShreddedRowStore::get(7, &backend).unwrap().is_none());
    }

    #[test]
    fn test_shredded_delete_missing_entry_is_noop() {
        let backend = test_backend();
        // No shredded row seeded: delete must be a safe no-op.
        ShreddedRowStore::delete(123, &backend).unwrap();
    }

    #[test]
    fn test_shredded_delete_op_targets_internal_metadata() {
        // VER-02: the batch form must address the same key/partition as
        // `delete()` so `delete_batch`/`purge_permanent` purge the same row.
        match ShreddedRowStore::delete_op(7) {
            BackendWriteOp::Delete { partition, key } => {
                assert_eq!(partition, BackendPartition::InternalMetadata);
                assert_eq!(key, ShreddedRowStore::store_key(7));
            }
            _ => panic!("expected BackendWriteOp::Delete"),
        }
    }

    // ── schema evolution (last-write-wins) ────────────────────

    #[test]
    fn test_shredded_schema_evolution() {
        let backend = test_backend();

        // First write: "a" is an integer
        let mut f1 = BTreeMap::new();
        f1.insert("a".into(), Value::Int(42));
        ShreddedRowStore::put(1, &f1, &backend).unwrap();
        let r1 = ShreddedRowStore::get(1, &backend).unwrap().unwrap();
        assert_eq!(r1.get("a"), Some(&ShreddedField::I64(42)));

        // Second write: "a" is now a string  (last-write-wins)
        let mut f2 = BTreeMap::new();
        f2.insert("a".into(), Value::String("hello".into()));
        ShreddedRowStore::put(1, &f2, &backend).unwrap();
        let r2 = ShreddedRowStore::get(1, &backend).unwrap().unwrap();
        assert_eq!(r2.get("a"), Some(&ShreddedField::String("hello".into())));
    }

    // ── multiple independent nodes ────────────────────────────

    #[test]
    fn test_shredded_multiple_nodes() {
        let backend = test_backend();

        let mut f1 = BTreeMap::new();
        f1.insert("color".into(), Value::String("red".into()));
        ShreddedRowStore::put(10, &f1, &backend).unwrap();

        let mut f2 = BTreeMap::new();
        f2.insert("color".into(), Value::String("blue".into()));
        f2.insert("size".into(), Value::Int(5));
        ShreddedRowStore::put(20, &f2, &backend).unwrap();

        let mut f3 = BTreeMap::new();
        f3.insert("active".into(), Value::Bool(false));
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
        fields.insert("data".into(), Value::String(large.clone()));
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
            &Value::Int(10)
        ));
        assert!(!matches_shredded(
            &ShreddedField::I64(10),
            &RelOp::Eq,
            &Value::Int(20)
        ));

        // F64 Eq
        assert!(matches_shredded(
            &ShreddedField::F64(3.5),
            &RelOp::Eq,
            &Value::Float(3.5)
        ));
        assert!(!matches_shredded(
            &ShreddedField::F64(3.5),
            &RelOp::Eq,
            &Value::Float(4.0)
        ));

        // Bool Eq
        assert!(matches_shredded(
            &ShreddedField::Bool(true),
            &RelOp::Eq,
            &Value::Bool(true)
        ));
        assert!(!matches_shredded(
            &ShreddedField::Bool(true),
            &RelOp::Eq,
            &Value::Bool(false)
        ));

        // String Eq
        assert!(matches_shredded(
            &ShreddedField::String("hi".into()),
            &RelOp::Eq,
            &Value::String("hi".into())
        ));
        assert!(!matches_shredded(
            &ShreddedField::String("hi".into()),
            &RelOp::Eq,
            &Value::String("bye".into())
        ));

        // type mismatch → false
        assert!(!matches_shredded(
            &ShreddedField::I64(1),
            &RelOp::Eq,
            &Value::String("1".into())
        ));
        assert!(!matches_shredded(
            &ShreddedField::Null,
            &RelOp::Eq,
            &Value::Int(0)
        ));
    }

    // ── I64 comparisons ───────────────────────────────────────

    #[test]
    fn test_matches_shredded_i64_comparisons() {
        let val_10 = ShreddedField::I64(10);
        let val_20 = ShreddedField::I64(20);

        // Eq
        assert!(matches_shredded(&val_10, &RelOp::Eq, &Value::Int(10)));
        assert!(!matches_shredded(&val_10, &RelOp::Eq, &Value::Int(20)));

        // Neq
        assert!(matches_shredded(&val_10, &RelOp::Neq, &Value::Int(20)));
        assert!(!matches_shredded(&val_10, &RelOp::Neq, &Value::Int(10)));

        // Gt
        assert!(matches_shredded(&val_20, &RelOp::Gt, &Value::Int(15)));
        assert!(!matches_shredded(&val_10, &RelOp::Gt, &Value::Int(10)));
        assert!(!matches_shredded(&val_10, &RelOp::Gt, &Value::Int(15)));

        // Lt
        assert!(matches_shredded(&val_10, &RelOp::Lt, &Value::Int(15)));
        assert!(!matches_shredded(&val_20, &RelOp::Lt, &Value::Int(15)));
        assert!(!matches_shredded(&val_10, &RelOp::Lt, &Value::Int(10)));

        // Gte
        assert!(matches_shredded(&val_20, &RelOp::Gte, &Value::Int(15)));
        assert!(matches_shredded(&val_20, &RelOp::Gte, &Value::Int(20)));
        assert!(!matches_shredded(&val_10, &RelOp::Gte, &Value::Int(15)));

        // Lte
        assert!(matches_shredded(&val_10, &RelOp::Lte, &Value::Int(15)));
        assert!(matches_shredded(&val_10, &RelOp::Lte, &Value::Int(10)));
        assert!(!matches_shredded(&val_20, &RelOp::Lte, &Value::Int(15)));
    }

    // ── F64 comparisons ───────────────────────────────────────

    #[test]
    fn test_matches_shredded_f64_comparisons() {
        let val_1_5 = ShreddedField::F64(1.5);
        let val_3_0 = ShreddedField::F64(3.0);

        // Eq
        assert!(matches_shredded(&val_3_0, &RelOp::Eq, &Value::Float(3.0)));
        assert!(!matches_shredded(&val_1_5, &RelOp::Eq, &Value::Float(3.0)));

        // Neq
        assert!(matches_shredded(&val_1_5, &RelOp::Neq, &Value::Float(3.0)));
        assert!(!matches_shredded(&val_3_0, &RelOp::Neq, &Value::Float(3.0)));

        // Gt
        assert!(matches_shredded(&val_3_0, &RelOp::Gt, &Value::Float(2.0)));
        assert!(!matches_shredded(&val_1_5, &RelOp::Gt, &Value::Float(2.0)));
        assert!(!matches_shredded(&val_3_0, &RelOp::Gt, &Value::Float(3.0)));

        // Lt
        assert!(matches_shredded(&val_1_5, &RelOp::Lt, &Value::Float(2.0)));
        assert!(!matches_shredded(&val_3_0, &RelOp::Lt, &Value::Float(2.0)));
        assert!(!matches_shredded(&val_1_5, &RelOp::Lt, &Value::Float(1.5)));

        // Gte
        assert!(matches_shredded(&val_3_0, &RelOp::Gte, &Value::Float(2.0)));
        assert!(matches_shredded(&val_3_0, &RelOp::Gte, &Value::Float(3.0)));
        assert!(!matches_shredded(&val_1_5, &RelOp::Gte, &Value::Float(2.0)));

        // Lte
        assert!(matches_shredded(&val_1_5, &RelOp::Lte, &Value::Float(2.0)));
        assert!(matches_shredded(&val_1_5, &RelOp::Lte, &Value::Float(1.5)));
        assert!(!matches_shredded(&val_3_0, &RelOp::Lte, &Value::Float(2.0)));
    }

    // ── Bool Eq / Neq ─────────────────────────────────────────

    #[test]
    fn test_matches_shredded_bool_eq_ne() {
        let t = ShreddedField::Bool(true);
        let f = ShreddedField::Bool(false);

        assert!(matches_shredded(&t, &RelOp::Eq, &Value::Bool(true)));
        assert!(!matches_shredded(&t, &RelOp::Eq, &Value::Bool(false)));

        assert!(matches_shredded(&t, &RelOp::Neq, &Value::Bool(false)));
        assert!(!matches_shredded(&t, &RelOp::Neq, &Value::Bool(true)));

        // Gt/Lt on Bool — unsupported → false
        assert!(!matches_shredded(&t, &RelOp::Gt, &Value::Bool(false)));
        assert!(!matches_shredded(&f, &RelOp::Lt, &Value::Bool(true)));
    }

    // ── String Eq / Neq ───────────────────────────────────────

    #[test]
    fn test_matches_shredded_string_eq_ne() {
        let hi = ShreddedField::String("hi".into());
        let bye = ShreddedField::String("bye".into());

        assert!(matches_shredded(
            &hi,
            &RelOp::Eq,
            &Value::String("hi".into())
        ));
        assert!(!matches_shredded(
            &hi,
            &RelOp::Eq,
            &Value::String("bye".into())
        ));

        assert!(matches_shredded(
            &hi,
            &RelOp::Neq,
            &Value::String("bye".into())
        ));
        assert!(!matches_shredded(
            &bye,
            &RelOp::Neq,
            &Value::String("bye".into())
        ));

        // Gt on String — unsupported → false
        assert!(!matches_shredded(
            &hi,
            &RelOp::Gt,
            &Value::String("bye".into())
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
        r1.insert("price".into(), Value::Int(100));
        ShreddedRowStore::put(1, &r1, &backend).unwrap();

        let mut r2 = BTreeMap::new();
        r2.insert("price".into(), Value::Int(200));
        ShreddedRowStore::put(2, &r2, &backend).unwrap();

        let mut r3 = BTreeMap::new();
        r3.insert("price".into(), Value::Int(50));
        ShreddedRowStore::put(3, &r3, &backend).unwrap();

        // Retrieve shredded data and apply comparison filter price > 100
        let nodes = [1u128, 2, 3];
        let expected: Vec<bool> = nodes
            .iter()
            .map(|&id| {
                let shredded = ShreddedRowStore::get(id, &backend).unwrap().unwrap();
                shredded
                    .get("price")
                    .is_some_and(|field| matches_shredded(field, &RelOp::Gt, &Value::Int(100)))
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
                    .is_some_and(|field| matches_shredded(field, &RelOp::Lt, &Value::Int(150)))
            })
            .collect();

        assert_eq!(expected_lt, vec![true, false, true]);

        // Verify price >= 100 (Gte): nodes 1 (100) and 2 (200)
        let expected_gte: Vec<bool> = nodes
            .iter()
            .map(|&id| {
                let shredded = ShreddedRowStore::get(id, &backend).unwrap().unwrap();
                shredded
                    .get("price")
                    .is_some_and(|field| matches_shredded(field, &RelOp::Gte, &Value::Int(100)))
            })
            .collect();

        assert_eq!(expected_gte, vec![true, true, false]);
    }

    // ── SH-05: repr(u8) discriminants are the on-disk contract ────────

    #[test]
    fn test_shredded_field_type_discriminants_are_byte_stable() {
        // The discriminant IS the persisted type tag. These values must
        // never change — renumbering corrupts every stored shredded row.
        assert_eq!(ShreddedFieldType::I64 as u8, 0);
        assert_eq!(ShreddedFieldType::F64 as u8, 1);
        assert_eq!(ShreddedFieldType::Bool as u8, 2);
        assert_eq!(ShreddedFieldType::String as u8, 3);
        assert_eq!(ShreddedFieldType::Null as u8, 4);
    }

    #[test]
    fn test_shredded_field_type_from_tag_roundtrip() {
        for ty in [
            ShreddedFieldType::I64,
            ShreddedFieldType::F64,
            ShreddedFieldType::Bool,
            ShreddedFieldType::String,
            ShreddedFieldType::Null,
        ] {
            assert_eq!(ShreddedFieldType::from_tag(ty as u8), Some(ty));
        }
        // The format is closed: tags outside 0..=4 are corruption (SH-02).
        assert_eq!(ShreddedFieldType::from_tag(5), None);
        assert_eq!(ShreddedFieldType::from_tag(u8::MAX), None);
    }

    // ── FIND-604: omitted-field markers ─────────────────────────────

    #[test]
    fn test_shredded_omitted_fields_decode_as_null_markers() {
        let backend = test_backend();
        let mut fields = BTreeMap::new();
        fields.insert("age".into(), Value::Int(30));
        fields.insert(
            "tags".into(),
            Value::ListString(vec!["a".into(), "b".into()]),
        );
        fields.insert("maybe".into(), Value::Null);
        fields.insert("rank".into(), Value::ListInt(vec![1, 2]));
        fields.insert(
            "at".into(),
            Value::DateTime(jiff::Timestamp::from_second(1_700_000_000).unwrap()),
        );
        ShreddedRowStore::put(11, &fields, &backend).unwrap();

        let row = ShreddedRowStore::get(11, &backend).unwrap().unwrap();
        // Scalars stay scalar; every non-scalar field is present as a
        // Null marker — observable as "omitted", not "absent".
        assert_eq!(row.get("age"), Some(&ShreddedField::I64(30)));
        assert_eq!(row.get("tags"), Some(&ShreddedField::Null));
        assert_eq!(row.get("maybe"), Some(&ShreddedField::Null));
        assert_eq!(row.get("rank"), Some(&ShreddedField::Null));
        assert_eq!(row.get("at"), Some(&ShreddedField::Null));
        assert_eq!(row.len(), 5);
        assert!(has_omitted_fields(&row));
    }

    #[test]
    fn test_shredded_markers_are_emitted_after_scalar_entries() {
        // Ordering invariant: scalar entries first so a pre-marker decoder
        // truncating at the unknown tag still recovers every scalar.
        // "aaa" sorts BEFORE "zzz" in the BTreeMap, so marker-last is
        // only observable because encode partitions the passes.
        let mut fields = BTreeMap::new();
        fields.insert("aaa".into(), Value::ListString(vec!["x".into()]));
        fields.insert("zzz".into(), Value::Int(7));
        let buf = ShreddedRowStore::encode_fields(&fields).unwrap();

        // Expected layout: [zzz|I64|7][aaa|Null]
        let mut expected = Vec::new();
        expected.extend_from_slice(&3u32.to_le_bytes());
        expected.extend_from_slice(b"zzz");
        expected.push(ShreddedFieldType::I64 as u8);
        expected.extend_from_slice(&7i64.to_le_bytes());
        expected.extend_from_slice(&3u32.to_le_bytes());
        expected.extend_from_slice(b"aaa");
        expected.push(ShreddedFieldType::Null as u8);
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_shredded_scalar_only_rows_keep_legacy_byte_layout() {
        // Byte-compat: metadata with only scalars must encode to exactly
        // the pre-marker format (FIND-604 persistence invariant).
        let mut fields = BTreeMap::new();
        fields.insert("n".into(), Value::Int(-5));
        fields.insert("s".into(), Value::String("hi".into()));
        let buf = ShreddedRowStore::encode_fields(&fields).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&1u32.to_le_bytes());
        expected.extend_from_slice(b"n");
        expected.push(0); // legacy I64 tag
        expected.extend_from_slice(&(-5i64).to_le_bytes());
        expected.extend_from_slice(&1u32.to_le_bytes());
        expected.extend_from_slice(b"s");
        expected.push(3); // legacy String tag
        expected.extend_from_slice(&2u32.to_le_bytes());
        expected.extend_from_slice(b"hi");
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_shredded_decodes_legacy_row_without_markers() {
        // A row written by the pre-marker encoder (scalars only, no tag 4)
        // decodes identically under the strict decoder.
        let backend = test_backend();
        let mut raw = Vec::new();
        raw.extend_from_slice(&3u32.to_le_bytes());
        raw.extend_from_slice(b"age");
        raw.push(0u8);
        raw.extend_from_slice(&42i64.to_le_bytes());
        backend
            .put(
                BackendPartition::InternalMetadata,
                &ShreddedRowStore::store_key(5),
                &raw,
            )
            .unwrap();

        let row = ShreddedRowStore::get(5, &backend).unwrap().unwrap();
        assert_eq!(row.get("age"), Some(&ShreddedField::I64(42)));
        assert!(!has_omitted_fields(&row));
    }

    #[test]
    fn test_shredded_all_non_scalar_metadata_still_writes_markers() {
        // Before FIND-604 a record whose metadata is entirely non-scalar
        // produced no row at all; now the markers preserve field presence.
        let backend = test_backend();
        let mut fields = BTreeMap::new();
        fields.insert("tags".into(), Value::ListString(vec!["x".into()]));
        fields.insert("flag".into(), Value::Null);
        ShreddedRowStore::put(21, &fields, &backend).unwrap();

        let row = ShreddedRowStore::get(21, &backend).unwrap().unwrap();
        assert_eq!(row.get("tags"), Some(&ShreddedField::Null));
        assert_eq!(row.get("flag"), Some(&ShreddedField::Null));
        assert!(has_omitted_fields(&row));
    }

    // ── FIND-604: filter parity (ListString / Null) ──────────────────

    #[test]
    fn test_filter_verdict_delegates_on_list_and_null_fields() {
        let backend = test_backend();
        let mut fields = BTreeMap::new();
        fields.insert("price".into(), Value::Int(100));
        fields.insert(
            "tags".into(),
            Value::ListString(vec!["a".into(), "b".into()]),
        );
        fields.insert("deleted".into(), Value::Null);
        ShreddedRowStore::put(31, &fields, &backend).unwrap();
        let row = ShreddedRowStore::get(31, &backend).unwrap().unwrap();

        // Scalar filter → decided by the column (no document needed).
        assert_eq!(
            filter_verdict(&row, "price", &RelOp::Eq, &Value::Int(100)),
            ShreddedFilterVerdict::Decided(true)
        );
        assert_eq!(
            filter_verdict(&row, "price", &RelOp::Gt, &Value::Int(50)),
            ShreddedFilterVerdict::Decided(true)
        );
        assert_eq!(
            filter_verdict(&row, "price", &RelOp::Lt, &Value::Int(50)),
            ShreddedFilterVerdict::Decided(false)
        );

        // Parity: a filter on a ListString field must NOT be decided by
        // the column store — the old code answered `false` here, which is
        // where the fast path diverged to zero results. Now it delegates.
        assert_eq!(
            filter_verdict(
                &row,
                "tags",
                &RelOp::Eq,
                &Value::ListString(vec!["a".into(), "b".into()])
            ),
            ShreddedFilterVerdict::DelegateToDocument
        );
        assert_eq!(
            filter_verdict(&row, "deleted", &RelOp::Eq, &Value::Null),
            ShreddedFilterVerdict::DelegateToDocument
        );

        // A field that is genuinely absent is reported distinctly.
        assert_eq!(
            filter_verdict(&row, "missing", &RelOp::Eq, &Value::Int(1)),
            ShreddedFilterVerdict::Absent
        );
    }

    #[test]
    fn test_filter_verdict_scalar_only_row_never_delegates() {
        // Regression guard for pre-mortem risk 3: a row whose fields are
        // all scalar must keep the fast path for every filter field.
        let backend = test_backend();
        let mut fields = BTreeMap::new();
        fields.insert("price".into(), Value::Int(100));
        fields.insert("name".into(), Value::String("n".into()));
        ShreddedRowStore::put(32, &fields, &backend).unwrap();
        let row = ShreddedRowStore::get(32, &backend).unwrap().unwrap();

        assert!(!has_omitted_fields(&row));
        assert_eq!(
            filter_verdict(&row, "name", &RelOp::Eq, &Value::String("n".into())),
            ShreddedFilterVerdict::Decided(true)
        );
    }

    // ── SH-02: strict decode — corruption is an error, not a partial map ─

    fn seed_raw_row(backend: &InMemoryBackend, node_id: u128, bytes: &[u8]) {
        backend
            .put(
                BackendPartition::InternalMetadata,
                &ShreddedRowStore::store_key(node_id),
                bytes,
            )
            .unwrap();
    }

    #[test]
    fn test_shredded_truncated_row_is_an_error_not_partial_map() {
        let backend = test_backend();
        // One complete entry followed by a truncated second entry: the
        // decoder must fail instead of returning the partial map.
        let mut raw = Vec::new();
        raw.extend_from_slice(&3u32.to_le_bytes());
        raw.extend_from_slice(b"age");
        raw.push(0u8);
        raw.extend_from_slice(&42i64.to_le_bytes());
        // Truncated second entry: key_len says 5 but only 2 bytes follow.
        raw.extend_from_slice(&5u32.to_le_bytes());
        raw.extend_from_slice(b"ab");
        seed_raw_row(&backend, 41, &raw);

        assert!(ShreddedRowStore::get(41, &backend).is_err());
    }

    #[test]
    fn test_shredded_truncated_value_is_an_error() {
        let backend = test_backend();
        // I64 tag but only 3 value bytes follow.
        let mut raw = Vec::new();
        raw.extend_from_slice(&1u32.to_le_bytes());
        raw.extend_from_slice(b"x");
        raw.push(0u8);
        raw.extend_from_slice(&[1u8, 2, 3]);
        seed_raw_row(&backend, 42, &raw);

        assert!(ShreddedRowStore::get(42, &backend).is_err());
    }

    #[test]
    fn test_shredded_unknown_tag_is_an_error() {
        let backend = test_backend();
        let mut raw = Vec::new();
        raw.extend_from_slice(&1u32.to_le_bytes());
        raw.extend_from_slice(b"x");
        raw.push(9u8); // unknown tag — not a truncation point
        seed_raw_row(&backend, 43, &raw);

        assert!(ShreddedRowStore::get(43, &backend).is_err());
    }

    #[test]
    fn test_shredded_non_utf8_key_is_an_error() {
        let backend = test_backend();
        let mut raw = Vec::new();
        raw.extend_from_slice(&2u32.to_le_bytes());
        raw.extend_from_slice(&[0xff, 0xfe]); // invalid UTF-8 field name
        raw.push(0u8);
        raw.extend_from_slice(&1i64.to_le_bytes());
        seed_raw_row(&backend, 44, &raw);

        assert!(ShreddedRowStore::get(44, &backend).is_err());
    }

    #[test]
    fn test_shredded_non_utf8_string_value_is_an_error() {
        let backend = test_backend();
        let mut raw = Vec::new();
        raw.extend_from_slice(&1u32.to_le_bytes());
        raw.extend_from_slice(b"s");
        raw.push(3u8);
        raw.extend_from_slice(&2u32.to_le_bytes());
        raw.extend_from_slice(&[0xff, 0xfe]);
        seed_raw_row(&backend, 45, &raw);

        assert!(ShreddedRowStore::get(45, &backend).is_err());
    }
}
