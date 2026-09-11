//! Record-domain SDK types: memory inputs/records, filters, listing, export.
//!
//! Pure move of the record items from `super` (FIND-49). Public paths
//! `crate::sdk::types::X` are preserved via re-exports in `super`.

use super::{u128_serde, MemoryMetadata, Value};
use crate::node::SparseVector;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Operadores de comparación para filtros de metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
}

/// Un filtro individual: campo + operador + valor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryFilterItem {
    pub field: String,
    pub op: FilterOp,
    pub value: Value,
}

/// Lista de filtros combinados con AND lógico.
pub type MemoryFilter = Vec<MemoryFilterItem>;

/// Stable persistent memory payload accepted by external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryInput {
    /// Namespace to scope the record under.
    pub namespace: String,
    /// Unique key within the namespace.
    pub key: String,
    /// Payload text content.
    pub payload: String,
    /// Arbitrary metadata key-value pairs.
    pub metadata: MemoryMetadata,
    /// Optional embedding vector.
    pub vector: Option<Vec<f32>>,
    /// Optional sparse term-weight vector (e.g. raw-keyword weights). Sparse
    /// vectors participate in sparse-dot search alongside the dense vector.
    #[serde(default)]
    pub sparse_vector: Option<SparseVector>,
    /// Time-to-live in milliseconds from now.  The system computes
    /// ``expires_at_ms = now_ms() + ttl_ms`` server-side during ``put()``.
    /// ``None`` means the record never expires.
    pub ttl_ms: Option<u64>,
}

impl MemoryInput {
    /// Create a new memory input with the given namespace, key, and payload.
    ///
    /// Metadata defaults to empty, vector is `None`, and TTL is `None` (no expiry).
    pub fn new(
        namespace: impl Into<String>,
        key: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            key: key.into(),
            payload: payload.into(),
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        }
    }
}

/// Stable persistent memory view returned to external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecord {
    /// Namespace the record belongs to.
    pub namespace: String,
    /// Unique key within the namespace.
    pub key: String,
    /// Payload text content.
    pub payload: String,
    /// Arbitrary metadata key-value pairs.
    pub metadata: MemoryMetadata,
    /// Unix-ms creation timestamp.
    pub created_at_ms: u64,
    /// Unix-ms last-update timestamp.
    pub updated_at_ms: u64,
    /// Monotonic version counter.
    pub version: u64,
    /// Deterministic node id derived from namespace and key.
    #[serde(with = "u128_serde")]
    pub node_id: u128,
    /// Optional embedding vector.
    pub vector: Option<Vec<f32>>,
    /// Optional sparse term-weight vector persisted alongside the dense vector.
    #[serde(default)]
    pub sparse_vector: Option<SparseVector>,
    /// Absolute Unix-ms timestamp after which the record is considered
    /// expired.  ``None`` means the record never expires.
    pub expires_at_ms: Option<u64>,
    /// Key (same namespace) of the record that supersedes this one (ADR-028).
    /// ``None`` means the record is current. Superseded records stay in
    /// storage (soft-dead, recoverable) but can be hidden via
    /// ``exclude_superseded`` on search/list.
    #[serde(default)]
    pub superseded_by: Option<String>,
    /// Unix-ms timestamp when the supersession was recorded (ADR-028).
    #[serde(default)]
    pub superseded_at_ms: Option<u64>,
}

/// Stable list options for namespace-scoped memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryListOptions {
    /// Metadata key-value filters to narrow results (legacy).
    #[deprecated(note = "Use filter_ops instead")]
    #[serde(default)]
    pub filters: MemoryMetadata,

    /// Advanced metadata filters with operators.
    #[serde(default)]
    pub filter_ops: Option<MemoryFilter>,

    /// Maximum number of records to return.
    pub limit: usize,
    /// Zero-based cursor for pagination. `None` starts from the beginning.
    pub cursor: Option<usize>,
    /// When true, records marked as superseded (ADR-028) are dropped from the
    /// page. Defaults to false: superseded records remain visible.
    #[serde(default)]
    pub exclude_superseded: bool,
}

impl Default for MemoryListOptions {
    fn default() -> Self {
        Self {
            #[allow(deprecated)]
            filters: MemoryMetadata::new(),
            filter_ops: None,
            limit: 100,
            cursor: None,
            exclude_superseded: false,
        }
    }
}

/// Stable list page returned by namespace-scoped scans.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryListPage {
    /// Records in the current page.
    pub records: Vec<MemoryRecord>,
    /// Cursor for the next page, or `None` if this was the last page.
    pub next_cursor: Option<usize>,
}

/// Default "expiring soon" window for [`NamespaceStats`]: 24 hours.
pub const DEFAULT_EXPIRING_SOON_WINDOW_MS: u64 = 24 * 60 * 60 * 1000;

/// Per-namespace memory statistics for overview UIs (e.g. Vanta Studio HOME).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamespaceStats {
    /// Total number of records in the namespace.
    pub count: u64,
    /// Records whose TTL expires within the "expiring soon" window.
    pub expiring_soon: u64,
    /// Records whose TTL has already passed (expired).
    pub expired: u64,
}

/// Map of namespace → [`NamespaceStats`], in BTreeMap (key-sorted) order.
pub type NamespaceStatsMap = BTreeMap<String, NamespaceStats>;

/// Stable report returned by JSONL memory export operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportReport {
    /// Number of records written to the export file.
    pub records_exported: u64,
    /// Namespaces that were included in the export.
    pub namespaces: Vec<String>,
    /// Filesystem path to the export file.
    pub path: String,
    /// Duration of the export in milliseconds.
    pub duration_ms: u64,
}

/// Stable report returned by JSONL memory import operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportReport {
    /// Number of new records inserted.
    pub inserted: u64,
    /// Number of existing records updated.
    pub updated: u64,
    /// Number of lines skipped (empty lines during file import).
    pub skipped: u64,
    /// Number of records that failed to import.
    pub errors: u64,
    /// Duration of the import in milliseconds.
    pub duration_ms: u64,
}

/// A single JSONL export line representing one memory record at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryExportLine {
    /// Export format schema version for forward compatibility.
    pub schema_version: u32,
    /// Namespace the record belongs to.
    pub namespace: String,
    /// Unique key within the namespace.
    pub key: String,
    /// Payload text content.
    pub payload: String,
    /// Arbitrary metadata key-value pairs.
    pub metadata: MemoryMetadata,
    /// Optional embedding vector.
    pub vector: Option<Vec<f32>>,
    /// Optional sparse vector.
    #[serde(default)]
    pub sparse_vector: Option<SparseVector>,
    /// Unix-ms creation timestamp.
    pub created_at_ms: u64,
    /// Unix-ms last-update timestamp.
    pub updated_at_ms: u64,
    /// Monotonic version counter.
    pub version: u64,
    /// Optional Unix-ms expiry deadline.
    pub expires_at_ms: Option<u64>,
    /// Key (same namespace) of the record that supersedes this one (ADR-028).
    #[serde(default)]
    pub superseded_by: Option<String>,
    /// Unix-ms timestamp when the supersession was recorded (ADR-028).
    #[serde(default)]
    pub superseded_at_ms: Option<u64>,
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    // ---- MemoryInput ----

    #[test]
    fn test_memory_input_new() {
        let input = MemoryInput::new("ns1", "key1", "payload text");
        assert_eq!(input.namespace, "ns1");
        assert_eq!(input.key, "key1");
        assert_eq!(input.payload, "payload text");
        assert!(input.metadata.is_empty());
        assert!(input.vector.is_none());
        assert!(input.ttl_ms.is_none());
    }

    #[test]
    fn test_memory_input_clone() {
        let input = MemoryInput::new("ns", "k", "p");
        let cloned = input.clone();
        assert_eq!(input, cloned);
    }

    // ---- MemoryListOptions ----

    #[test]
    fn test_memory_list_options_default() {
        let opts = MemoryListOptions::default();
        #[allow(deprecated)]
        let _ = opts.filters.is_empty();
        assert!(opts.filter_ops.is_none());
        assert_eq!(opts.limit, 100);
        assert!(opts.cursor.is_none());
    }

    // ---- MemoryListPage ----

    #[test]
    fn test_memory_list_page_empty() {
        let page = MemoryListPage {
            records: vec![],
            next_cursor: None,
        };
        assert!(page.records.is_empty());
        assert!(page.next_cursor.is_none());
    }

    // ---- Reports ----

    #[test]
    fn test_export_report() {
        let r = ExportReport {
            records_exported: 500,
            namespaces: vec!["ns1".into()],
            path: "/tmp/export.jsonl".into(),
            duration_ms: 250,
        };
        assert_eq!(r.records_exported, 500);
        assert_eq!(r.namespaces, vec!["ns1"]);
    }

    #[test]
    fn test_import_report() {
        let r = ImportReport {
            inserted: 100,
            updated: 10,
            skipped: 2,
            errors: 1,
            duration_ms: 300,
        };
        assert_eq!(r.inserted, 100);
        assert_eq!(r.updated, 10);
        assert_eq!(r.errors, 1);
    }

    // ---- MemoryRecord ----

    #[test]
    fn test_memory_record_fields() {
        let rec = MemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 1,
            node_id: 42,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        assert_eq!(rec.namespace, "ns");
        assert_eq!(rec.node_id, 42);
        assert_eq!(rec.version, 1);
    }

    // ---- MemoryExportLine ----

    #[test]
    fn test_export_line() {
        let line = MemoryExportLine {
            schema_version: 1,
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            created_at_ms: 1000,
            updated_at_ms: 1000,
            version: 1,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        assert_eq!(line.schema_version, 1);
        assert_eq!(line.namespace, "ns");
    }

    // ---- MemoryInput with vector and ttl ----

    #[test]
    fn test_memory_input_with_vector_ttl() {
        let input = MemoryInput {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: [("lang".into(), Value::String("en".into()))].into(),
            vector: Some(vec![0.1, 0.2, 0.3]),
            sparse_vector: None,
            ttl_ms: Some(60000),
        };
        assert_eq!(input.namespace, "ns");
        assert!(input.vector.is_some());
        assert_eq!(input.vector.as_ref().unwrap().len(), 3);
        assert_eq!(input.ttl_ms, Some(60000));
        assert_eq!(
            input.metadata.get("lang").unwrap(),
            &Value::String("en".into())
        );
    }

    // ---- MemoryRecord with expiry ----

    #[test]
    fn test_memory_record_with_expiry() {
        let rec = MemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 5,
            node_id: 42,
            vector: Some(vec![0.5, 0.6]),
            sparse_vector: None,
            expires_at_ms: Some(99999),
            superseded_by: None,
            superseded_at_ms: None,
        };
        assert_eq!(rec.version, 5);
        assert_eq!(rec.expires_at_ms, Some(99999));
        assert_eq!(rec.vector.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_memory_record_clone() {
        let rec = MemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 1,
            node_id: 42,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        let cloned = rec.clone();
        assert_eq!(rec, cloned);
    }

    // ---- MemoryListOptions custom ----

    #[test]
    fn test_memory_list_options_custom() {
        let opts = MemoryListOptions {
            #[allow(deprecated)]
            filters: [("type".into(), Value::String("doc".into()))].into(),
            filter_ops: None,
            limit: 50,
            cursor: Some(10),
            exclude_superseded: false,
        };
        assert_eq!(opts.limit, 50);
        assert_eq!(opts.cursor, Some(10));
        #[allow(deprecated)]
        let _ = opts.filters.get("type").unwrap() == &Value::String("doc".into());
    }

    // ---- ExportReport clone ----

    #[test]
    fn test_export_report_clone() {
        let r = ExportReport {
            records_exported: 100,
            namespaces: vec!["ns1".into()],
            path: "/tmp/x.jsonl".into(),
            duration_ms: 50,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    // ---- ImportReport clone ----

    #[test]
    fn test_import_report_clone() {
        let r = ImportReport {
            inserted: 10,
            updated: 5,
            skipped: 1,
            errors: 0,
            duration_ms: 100,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    // ---- MemoryExportLine all fields ----

    #[test]
    fn test_export_line_full() {
        let line = MemoryExportLine {
            schema_version: 2,
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: [("score".into(), Value::Float(9.5))].into(),
            vector: Some(vec![0.1, 0.2]),
            sparse_vector: None,
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 3,
            expires_at_ms: Some(99999),
            superseded_by: None,
            superseded_at_ms: None,
        };
        assert_eq!(line.schema_version, 2);
        assert_eq!(line.version, 3);
        assert!(line.vector.is_some());
        assert!(line.expires_at_ms.is_some());
    }

    // ---- MemoryListPage with data ----

    #[test]
    fn test_memory_list_page_with_data() {
        let rec = MemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "p".into(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 1,
            updated_at_ms: 2,
            version: 1,
            node_id: 1,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        let page = MemoryListPage {
            records: vec![rec],
            next_cursor: Some(1),
        };
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.next_cursor, Some(1));
    }
}
