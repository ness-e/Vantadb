//! Record-domain SDK types: memory inputs/records, filters, listing, export.
//!
//! Pure move of the record items from `super` (FIND-49). Public paths
//! `crate::sdk::types::X` are preserved via re-exports in `super`.

use super::{u128_serde, MemoryMetadata, Value};
use crate::node::SparseVector;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Comparison operators for metadata filters ([`MemoryFilterItem`]).
///
/// Evaluation compares the **stored** metadata value (`actual`) against the
/// filter value with the derived [`PartialOrd`] on [`Value`] (see
/// `sdk::serialization` filter evaluation):
///
/// - Same-variant comparisons are natural: `String`/`ListString` compare
///   lexicographically, `Int`/`Float`/`ListInt`/`ListFloat` numerically,
///   `Bool`/`ListBool` by value, `DateTime`/`ListDateTime` chronologically.
/// - Cross-variant comparisons fall back to the `Value` declaration order
///   (`String < Int < Float < Bool < DateTime < … < Null`) — use same-typed
///   filter values to avoid surprises.
///
/// Values come from [`Value`] (e.g. `Value::Int(42)`), never from raw JSON
/// numbers; the CLI/adapter layers normalize before constructing a filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterOp {
    /// Field equals the filter value (`==`). Any [`Value`] variant.
    Eq,
    /// Field differs from the filter value (`!=`). Any [`Value`] variant.
    Neq,
    /// Field is strictly greater than the filter value (`>`). Requires
    /// ordering — prefer same-variant values (see enum docs).
    Gt,
    /// Field is strictly less than the filter value (`<`). Requires
    /// ordering — prefer same-variant values (see enum docs).
    Lt,
    /// Field is greater than or equal to the filter value (`>=`). Requires
    /// ordering — prefer same-variant values (see enum docs).
    Gte,
    /// Field is less than or equal to the filter value (`<=`). Requires
    /// ordering — prefer same-variant values (see enum docs).
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

/// Half-open valid-time query window `[from_ms, to_ms)` (ADR-046 §D3, SCH-03).
///
/// Semantics: a record matches when its own validity window
/// `[valid_at_ms, invalid_at_ms)` **intersects** this window — the record was
/// valid at some instant inside `[from_ms, to_ms)`. Empty or inverted windows
/// (`from_ms >= to_ms`) are rejected at the request boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValidWindow {
    /// Start of the query window, inclusive (unix ms).
    pub from_ms: u64,
    /// End of the query window, exclusive (unix ms). Must be `> from_ms`.
    pub to_ms: u64,
}

/// Semantic default confidence for `asserted` records (D_a, ADR-046 §D4c):
/// "trust the writer". Used by `#[serde(default = "default_confidence")]` so a
/// v1 record/line without the field normalizes to `1.0` instead of `0.0`.
pub fn default_confidence() -> f32 {
    1.0
}

/// Derivation discount applied to `min(parent scores)` for `derived` records
/// (ADR-046 §D4a): `score = clamp(min(padres) × DERIVATION_DISCOUNT, 0, 1)`.
/// Module constant — no config in 0.8.0 (VER-08 may promote it).
pub const DERIVATION_DISCOUNT: f32 = 0.9;

/// Maximum accepted derivation-chain depth for `derived` records (V3,
/// ADR-046 §D4): longer chains are rejected with `Error::Validation`.
pub const MAX_DERIVATION_DEPTH: usize = 16;

/// Provenance class of a memory record (ADR-046 §D2):
/// `Asserted` = direct claim from a writer; `Derived` = computed by the engine
/// from `derived_from` parents. `#[non_exhaustive]` — may grow (e.g.
/// `observed`, `inferred`) without a breaking change.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConfidenceClass {
    /// Direct claim from a writer (human/agent/import).
    #[default]
    Asserted,
    /// Computed by the engine as a function of ≥1 existing records.
    Derived,
}

impl ConfidenceClass {
    /// Wire name matching the serde representation (`"Asserted"`/`"Derived"`);
    /// used for the persisted `__vanta_confidence_class` relational field.
    /// Drift against serde is guarded by a unit test.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Asserted => "Asserted",
            Self::Derived => "Derived",
        }
    }

    /// Parse the persisted wire name produced by [`Self::as_wire_str`].
    pub(crate) fn from_wire_str(value: &str) -> Option<Self> {
        match value {
            "Asserted" => Some(Self::Asserted),
            "Derived" => Some(Self::Derived),
            _ => None,
        }
    }
}

/// Outcome of a recalled memory as reported by the host after a recall pass
/// (MEMG-02, outcome loop): the explicit signal that feeds
/// [`crate::Embedded::reinforce`]. `#[non_exhaustive]` — the vocabulary may
/// grow (e.g. a future `PartiallyUsed`) without a breaking change.
///
/// The host declares the outcome; the engine never infers it (no silent
/// feedback). Semantics per variant are declared policy — see
/// `docs/api/scores.md` §Reinforcement:
///
/// - [`Self::Used`] — the recall resolved with this memory: confidence bumps
///   (+0.05, saturated at 1.0) and `last_validated_at_ms` is stamped, at most
///   once per rate window (5 min).
/// - [`Self::Corrected`] — the memory was wrong and had to be corrected:
///   confidence decays (−0.10, floored at 0.0). Failures do not stamp
///   `last_validated_at_ms` (success-only, MGR-12 §3.3).
/// - [`Self::Unused`] — the memory was recalled but not used: neutral (no
///   score change). Not evidence of incorrectness; recorded in the audit
///   trail so the declaration stays falsifiable.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReinforceOutcome {
    /// The recall resolved with this memory (positive signal).
    Used,
    /// The memory was wrong and the host corrected it (negative signal).
    Corrected,
    /// The memory was recalled but not used (neutral signal).
    Unused,
}

impl ReinforceOutcome {
    /// Wire name matching the serde representation (`"used"`/`"corrected"`/
    /// `"unused"`); used for audit reasons and MCP/HTTP payloads. Drift
    /// against serde is guarded by a unit test.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Used => "used",
            Self::Corrected => "corrected",
            Self::Unused => "unused",
        }
    }

    /// Parse a wire name produced by [`Self::as_wire_str`] (exact snake_case
    /// tokens; unknown tokens → `None` — callers reject, never infer).
    pub fn from_wire_str(value: &str) -> Option<Self> {
        match value {
            "used" => Some(Self::Used),
            "corrected" => Some(Self::Corrected),
            "unused" => Some(Self::Unused),
            _ => None,
        }
    }
}

/// Stable persistent memory payload accepted by external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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
    /// Start of the validity window (ADR-046 §D8). ``None`` defaults to the
    /// record's ``created_at_ms``. ``Some(0)`` is rejected: ``0`` is the v1
    /// "unset" sentinel and is normalized to ``created_at_ms`` on read.
    #[serde(default)]
    pub valid_at_ms: Option<u64>,
    /// Provenance class (ADR-046 §D2). ``None`` = asserted (system default).
    /// ``Some(Derived)`` requires a non-empty ``derived_from`` (V1) and forbids
    /// ``confidence`` (D4b).
    #[serde(default)]
    pub confidence_class: Option<ConfidenceClass>,
    /// Declared confidence range for asserted writes; must be finite in
    /// ``[0,1]``. ``None`` = D_a (1.0) for asserted; forbidden for derived
    /// (score is computed from parents).
    #[serde(default)]
    pub confidence: Option<f32>,
    /// Same-namespace parent keys. Required (non-empty) when class = Derived;
    /// forbidden (must be ``None``/empty) when asserted (V1).
    #[serde(default)]
    pub derived_from: Option<Vec<String>>,
    /// Quarantine write-time flag (ADR-046 §D2/§D5, T1): when ``true`` a new
    /// record enters quarantine (`reason=explicit_write`) and is excluded from
    /// default retrieval until explicitly promoted. Sticky: a plain re-write
    /// of an already-quarantined key preserves the state either way (I2).
    #[serde(default)]
    pub quarantine: bool,
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
            valid_at_ms: None,
            confidence_class: None,
            confidence: None,
            derived_from: None,
            quarantine: false,
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
    /// Start of the validity window `[valid_at_ms, invalid_at_ms)` — when the
    /// content reflects reality per the writer (ADR-046 §D3). Appended v2
    /// field: absent in v1 data ⇒ normalized to `created_at_ms` (the `0`
    /// value is the "unset" sentinel of the v1 boundary).
    #[serde(default)]
    pub valid_at_ms: u64,
    /// End of validity (exclusive); `None` = open (∞). Can be set
    /// retroactively (API v1.0); in 0.8.0 `supersede()` keeps it aligned with
    /// `superseded_at_ms` (ADR-046 §D3).
    #[serde(default)]
    pub invalid_at_ms: Option<u64>,
    /// Provenance class (asserted/derived) — v2 field (ADR-046 §D2).
    #[serde(default)]
    pub confidence_class: ConfidenceClass,
    /// Declared/computed confidence in `[0,1]` (a range, NOT a calibrated
    /// probability). Absent in v1 data ⇒ D_a = 1.0 (ADR-046 §D4c).
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    /// Last SUCCESSFUL re-validation (None = never; failures do not touch it).
    #[serde(default)]
    pub last_validated_at_ms: Option<u64>,
    /// Same-namespace parent keys of a `derived` record (empty for asserted).
    #[serde(default)]
    pub derived_from: Vec<String>,
    /// Quarantine state: `None` = active; `Some(t)` = quarantined since `t`
    /// (sticky — a re-write preserves it; ADR-046 §D5).
    #[serde(default)]
    pub quarantined_at_ms: Option<u64>,
    /// Stable reason code: `explicit_write` | `unreviewed_import` |
    /// `derived_promotion` | `policy_match` (reserved). The code set is open
    /// (N3): producers validate lowercase snake_case; future codes don't
    /// require a schema change.
    #[serde(default)]
    pub quarantine_reason: Option<String>,
    /// Principal that applied the quarantine, or `system:<op>`.
    #[serde(default)]
    pub quarantined_by: Option<String>,
    /// Review-deadline signal (never auto-promotes; ADR-046 §D5d).
    #[serde(default)]
    pub quarantine_review_due_ms: Option<u64>,
}

impl Default for MemoryRecord {
    /// Builder/test convenience mirroring the v1-normalization defaults:
    /// `confidence` = [`default_confidence`] (D_a), everything else zero/empty.
    /// `valid_at_ms: 0` is the documented "unset" sentinel (normalized to
    /// `created_at_ms` by the read/write boundaries).
    fn default() -> Self {
        Self {
            namespace: String::new(),
            key: String::new(),
            payload: String::new(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 0,
            node_id: 0,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            valid_at_ms: 0,
            invalid_at_ms: None,
            confidence_class: ConfidenceClass::Asserted,
            confidence: default_confidence(),
            last_validated_at_ms: None,
            derived_from: Vec::new(),
            quarantined_at_ms: None,
            quarantine_reason: None,
            quarantined_by: None,
            quarantine_review_due_ms: None,
        }
    }
}

impl MemoryRecord {
    /// ADR-046 §D3 predicate (verbatim): the record is valid at `t_ms` iff
    /// `valid_at_ms <= t_ms < invalid_at_ms`; `invalid_at_ms = None` means the
    /// window is open-ended (valid forever after `valid_at_ms`).
    pub fn is_valid_at(&self, t_ms: u64) -> bool {
        self.valid_at_ms <= t_ms && self.invalid_at_ms.is_none_or(|inv| inv > t_ms)
    }

    /// True when the record's validity window `[valid_at_ms, invalid_at_ms)`
    /// intersects the query window `[from_ms, to_ms)` (ADR-046 §D3, SCH-03).
    /// Callers validate `from_ms < to_ms` at the request boundary.
    pub fn validity_overlaps(&self, from_ms: u64, to_ms: u64) -> bool {
        self.valid_at_ms < to_ms && self.invalid_at_ms.is_none_or(|inv| inv > from_ms)
    }
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
    /// When true, records that are no longer current are dropped: superseded
    /// records (ADR-028) **and** records whose validity window ended at or
    /// before now (`invalid_at_ms <= now`, ADR-046 §D3-6, SCH-03). Defaults to
    /// false: superseded/ended records remain visible.
    #[serde(default)]
    pub exclude_superseded: bool,
    /// Valid-time point (ADR-046 §D3, SCH-03): when set, only records whose
    /// validity window contains `as_of_ms` are listed
    /// (`valid_at_ms <= as_of_ms < invalid_at_ms`; `None` window end = open).
    /// `None` = no temporal filter (default unchanged).
    #[serde(default)]
    pub as_of_ms: Option<u64>,
    /// Valid-time window overlap filter (ADR-046 §D3, SCH-03): only records
    /// valid at some instant inside `[from_ms, to_ms)`. Boundary-validated
    /// (`from_ms < to_ms`). `None` = no filter (default unchanged).
    #[serde(default)]
    pub valid_window: Option<ValidWindow>,
    /// SCH-05 (ADR-046 §D5): when `false` (default) quarantined records are
    /// excluded from the page — same post-filter position as
    /// `exclude_superseded`. Opt-in with `true` to inspect the quarantine queue.
    #[serde(default)]
    pub include_quarantined: bool,
    /// Opt-in confidence filter (ADR-046 §D2, SCH-07): keep only records whose
    /// `confidence` is `>= min_confidence` (finite, within `[0, 1]` — validated
    /// at the boundary, never clamped). `None` = no filter (default unchanged).
    #[serde(default)]
    pub min_confidence: Option<f32>,
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
            as_of_ms: None,
            valid_window: None,
            include_quarantined: false,
            min_confidence: None,
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
    /// Number of records that entered quarantine because of the import
    /// `quarantine` option (T1c, ADR-046 §D5; additive v2 field).
    #[serde(default)]
    pub quarantined: u64,
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
    /// Start of the validity window (v2). Absent/0 in v1 lines ⇒ normalized to
    /// `created_at_ms` by `record_from_export_line` (ADR-046 §D7).
    #[serde(default)]
    pub valid_at_ms: u64,
    /// End of validity (v2); v1 normalization derives it from `superseded_at_ms`.
    #[serde(default)]
    pub invalid_at_ms: Option<u64>,
    /// Provenance class (v2); v1 normalization ⇒ Asserted.
    #[serde(default)]
    pub confidence_class: ConfidenceClass,
    /// Confidence in `[0,1]` (v2); v1 normalization ⇒ D_a = 1.0.
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    /// Last successful re-validation (v2); v1 normalization ⇒ None.
    #[serde(default)]
    pub last_validated_at_ms: Option<u64>,
    /// Parent keys for derived records (v2); v1 normalization ⇒ empty.
    #[serde(default)]
    pub derived_from: Vec<String>,
    /// Quarantine fields (v2); v1 normalization ⇒ None ×4.
    #[serde(default)]
    pub quarantined_at_ms: Option<u64>,
    /// Stable quarantine reason code (v2).
    #[serde(default)]
    pub quarantine_reason: Option<String>,
    /// Principal that applied the quarantine (v2).
    #[serde(default)]
    pub quarantined_by: Option<String>,
    /// Review-deadline signal (v2).
    #[serde(default)]
    pub quarantine_review_due_ms: Option<u64>,
}

impl Default for MemoryExportLine {
    /// Test/builder convenience mirroring the v1-normalization defaults
    /// (`confidence` = [`default_confidence`], rest zero/empty/`None`).
    fn default() -> Self {
        Self {
            schema_version: 1,
            namespace: String::new(),
            key: String::new(),
            payload: String::new(),
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 0,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            valid_at_ms: 0,
            invalid_at_ms: None,
            confidence_class: ConfidenceClass::Asserted,
            confidence: default_confidence(),
            last_validated_at_ms: None,
            derived_from: Vec::new(),
            quarantined_at_ms: None,
            quarantine_reason: None,
            quarantined_by: None,
            quarantine_review_due_ms: None,
        }
    }
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
        assert!(!input.quarantine, "T1 flag defaults off (zero breaking)");
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
            quarantined: 3,
        };
        assert_eq!(r.inserted, 100);
        assert_eq!(r.updated, 10);
        assert_eq!(r.errors, 1);
        assert_eq!(r.quarantined, 3);
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            as_of_ms: None,
            valid_window: None,
            include_quarantined: false,
            min_confidence: None,
        };
        assert_eq!(opts.limit, 50);
        assert_eq!(opts.cursor, Some(10));
        #[allow(deprecated)]
        let _ = opts.filters.get("type").unwrap() == &Value::String("doc".into());
    }

    // ---- SCH-03: valid-time helpers + ValidWindow ----

    fn record_with_window(valid_at_ms: u64, invalid_at_ms: Option<u64>) -> MemoryRecord {
        MemoryRecord {
            valid_at_ms,
            invalid_at_ms,
            ..Default::default()
        }
    }

    #[test]
    fn is_valid_at_is_inclusive_at_start_exclusive_at_end() {
        let rec = record_with_window(1000, Some(2000));
        assert!(!rec.is_valid_at(999), "before start is invalid");
        assert!(rec.is_valid_at(1000), "start is inclusive");
        assert!(rec.is_valid_at(1999), "inside the window");
        assert!(!rec.is_valid_at(2000), "end is exclusive");
        assert!(!rec.is_valid_at(2001), "after end is invalid");
    }

    #[test]
    fn is_valid_at_open_ended_window_never_expires() {
        let rec = record_with_window(1000, None);
        assert!(!rec.is_valid_at(999));
        assert!(rec.is_valid_at(1000));
        assert!(rec.is_valid_at(u64::MAX));
    }

    #[test]
    fn validity_overlaps_is_half_open_intersection() {
        let rec = record_with_window(1000, Some(2000));
        assert!(!rec.validity_overlaps(0, 1000), "window ends at start");
        assert!(rec.validity_overlaps(999, 1001));
        assert!(rec.validity_overlaps(1500, 1501));
        assert!(rec.validity_overlaps(1999, 2000), "touching at end");
        assert!(!rec.validity_overlaps(2000, 3000), "window starts at end");

        let open = record_with_window(1000, None);
        assert!(
            open.validity_overlaps(0, 1001),
            "open window reaches any query window extending past valid_at"
        );
        assert!(
            !open.validity_overlaps(0, 1000),
            "query window ending exactly at valid_at does not intersect"
        );
    }

    #[test]
    fn valid_window_serialization_roundtrip_and_hash() {
        let window = ValidWindow {
            from_ms: 10,
            to_ms: 20,
        };
        let json = serde_json::to_string(&window).unwrap();
        let back: ValidWindow = serde_json::from_str(&json).unwrap();
        assert_eq!(back, window);
        let mut set = std::collections::HashSet::new();
        assert!(set.insert(window), "ValidWindow must be hashable");
    }

    #[test]
    fn memory_list_options_temporal_fields_default_to_none_and_roundtrip() {
        let opts = MemoryListOptions::default();
        assert_eq!(opts.as_of_ms, None);
        assert_eq!(opts.valid_window, None);

        let opts = MemoryListOptions {
            as_of_ms: Some(42),
            valid_window: Some(ValidWindow {
                from_ms: 1,
                to_ms: 2,
            }),
            ..Default::default()
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: MemoryListOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back, opts);

        // Legacy JSON (pre-SCH-03) deserializes with `None` temporal defaults.
        let legacy = r#"{"filters":{},"filter_ops":null,"limit":10,"cursor":null,"exclude_superseded":true}"#;
        let back: MemoryListOptions = serde_json::from_str(legacy).unwrap();
        assert_eq!(back.as_of_ms, None);
        assert_eq!(back.valid_window, None);
        assert!(
            !back.include_quarantined,
            "legacy JSON defaults to the default-exclude view (SCH-05)"
        );
        assert!(back.exclude_superseded);
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
            quarantined: 0,
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
            ..Default::default()
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
            ..Default::default()
        };
        let page = MemoryListPage {
            records: vec![rec],
            next_cursor: Some(1),
        };
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.next_cursor, Some(1));
    }

    // ---- v2 field defaults (ADR-046 §D2: #[serde(default)] en TODO campo nuevo) ----

    #[test]
    fn memory_record_json_v1_payload_normalizes_defaults() {
        // A v1 record JSON (13 fields, no v2 fields) deserializes with the
        // D2 defaults: confidence = D_a (1.0), class Asserted, valid_at 0
        // (unset sentinel), all Option fields None.
        let json = r#"{
            "namespace": "docs", "key": "k", "payload": "p", "metadata": {},
            "created_at_ms": 1000, "updated_at_ms": 2000, "version": 1,
            "node_id": "42", "vector": null, "expires_at_ms": null
        }"#;
        let record: MemoryRecord = serde_json::from_str(json).expect("v1 record JSON parses");
        assert_eq!(record.confidence, 1.0, "v1 absent confidence ⇒ D_a");
        assert_eq!(record.confidence_class, ConfidenceClass::Asserted);
        assert_eq!(
            record.valid_at_ms, 0,
            "0 = unset sentinel, normalized on read"
        );
        assert_eq!(record.invalid_at_ms, None);
        assert_eq!(record.last_validated_at_ms, None);
        assert!(record.derived_from.is_empty());
        assert_eq!(record.quarantined_at_ms, None);
        assert_eq!(record.quarantine_reason, None);
        assert_eq!(record.quarantined_by, None);
        assert_eq!(record.quarantine_review_due_ms, None);
    }

    #[test]
    fn memory_input_json_v1_payload_defaults_new_fields() {
        let json = r#"{"namespace": "ns", "key": "k", "payload": "p", "metadata": {}, "vector": null, "ttl_ms": null}"#;
        let input: MemoryInput = serde_json::from_str(json).expect("v1 input JSON parses");
        assert_eq!(input.valid_at_ms, None);
        assert_eq!(input.confidence_class, None);
        assert_eq!(input.confidence, None);
        assert_eq!(input.derived_from, None);
        assert!(!input.quarantine, "v1 absent quarantine flag ⇒ false (T1)");
    }

    #[test]
    fn memory_export_line_json_v1_defaults_confidence_to_d_a() {
        let json = r#"{"schema_version":1,"namespace":"ns","key":"k","payload":"p","metadata":{},"vector":null,"created_at_ms":1,"updated_at_ms":1,"version":1,"expires_at_ms":null}"#;
        let line: MemoryExportLine = serde_json::from_str(json).expect("v1 line parses");
        assert_eq!(line.confidence, 1.0);
        assert_eq!(line.confidence_class, ConfidenceClass::Asserted);
        assert_eq!(line.valid_at_ms, 0);
        assert_eq!(line.quarantined_at_ms, None);
    }

    #[test]
    fn confidence_class_wire_str_matches_serde() {
        // Drift guard: the persisted relational field value must equal the
        // serde wire name.
        for class in [ConfidenceClass::Asserted, ConfidenceClass::Derived] {
            let json = serde_json::to_string(&class).expect("serialize class");
            assert_eq!(json, format!("\"{}\"", class.as_wire_str()));
            assert_eq!(
                ConfidenceClass::from_wire_str(class.as_wire_str()),
                Some(class)
            );
        }
        assert_eq!(ConfidenceClass::from_wire_str("bogus"), None);
    }

    #[test]
    fn reinforce_outcome_wire_str_matches_serde() {
        // Drift guard: the audit/MCP wire names must equal the serde
        // representation, and `from_wire_str` must round-trip them.
        for outcome in [
            ReinforceOutcome::Used,
            ReinforceOutcome::Corrected,
            ReinforceOutcome::Unused,
        ] {
            let json = serde_json::to_string(&outcome).expect("serialize outcome");
            assert_eq!(json, format!("\"{}\"", outcome.as_wire_str()));
            assert_eq!(
                ReinforceOutcome::from_wire_str(outcome.as_wire_str()),
                Some(outcome)
            );
        }
        assert_eq!(
            ReinforceOutcome::from_wire_str("bogus"),
            None,
            "unknown tokens are rejected, never inferred"
        );
    }

    #[test]
    fn memory_record_default_uses_d_a_confidence() {
        let record = MemoryRecord::default();
        assert_eq!(record.confidence, default_confidence());
        assert_eq!(record.confidence_class, ConfidenceClass::Asserted);
        assert_eq!(record.valid_at_ms, 0);
    }

    #[test]
    fn confidence_class_default_is_asserted() {
        assert_eq!(ConfidenceClass::default(), ConfidenceClass::Asserted);
    }
}
