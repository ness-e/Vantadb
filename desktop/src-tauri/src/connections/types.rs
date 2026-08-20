use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

/// Transport / backend a connection speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Native,
    Http,
    Mcp,
    Node,
    Python,
    Wasm,
}

/// Lifecycle status of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
}

/// Reported health of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// A single unit to ingest into the store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngestItem {
    /// Optional explicit id; if absent the backend assigns one.
    #[serde(default)]
    pub id: Option<String>,
    /// Namespace/collection the record belongs to.
    #[serde(default = "default_namespace")]
    pub namespace: String,
    /// The text content.
    pub text: String,
    /// Optional precomputed embedding. Use finite floats — JSON has no NaN/Inf.
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
    /// Arbitrary record metadata (`serde_json::Value` so any JSON-able value roundtrips).
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Input for [`crate::connections::VantaConnection::search`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Semantic / natural-language query text.
    pub query: String,
    /// Optional explicit query vector (takes precedence over `query` text when present).
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
    /// Maximum number of results to return.
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Restrict search to a single namespace.
    #[serde(default)]
    pub namespace: Option<String>,
    /// Metadata filters applied post-search.
    #[serde(default)]
    pub filters: HashMap<String, serde_json::Value>,
    /// When true, each result carries a per-hit score breakdown in
    /// [`SearchResult::explanation`] (core explain mode). Backends that do not
    /// support explain ignore the flag and return `explanation: None`.
    #[serde(default)]
    pub explain: bool,
}

/// A single hit returned by [`crate::connections::VantaConnection::search`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub namespace: String,
    pub text: String,
    /// Relevance score. Higher is better, semantics are backend-defined.
    pub score: f32,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Per-hit score breakdown when the search ran in explain mode
    /// ([`SearchQuery::explain`] = true). `None` for regular searches and for
    /// backends that do not support explain.
    #[serde(default)]
    pub explanation: Option<ExplanationHit>,
}

/// Per-hit score breakdown for explain-mode searches ([`SearchQuery::explain`]).
///
/// Mirrors `VantaSearchExplanationHit` (`src/sdk/types.rs`) 1:1 so the UI can
/// render BM25 term contributions and RRF rank positions per result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplanationHit {
    /// Unique identity string (`namespace\0key`) of the matched record.
    pub identity: String,
    /// Combined relevance score for this hit.
    pub score: f32,
    /// Text snippet surrounding the matched query terms, if available.
    #[serde(default)]
    pub snippet: Option<String>,
    /// Query tokens that matched in this record.
    #[serde(default)]
    pub matched_tokens: Vec<String>,
    /// Query phrases that matched in this record.
    #[serde(default)]
    pub matched_phrases: Vec<String>,
    /// Per-term BM25 scoring breakdown.
    #[serde(default)]
    pub bm25_terms: Vec<Bm25Term>,
    /// Rank of this hit in the text-only result set, if applicable.
    #[serde(default)]
    pub rrf_text_rank: Option<usize>,
    /// Rank of this hit in the vector-only result set, if applicable.
    #[serde(default)]
    pub rrf_vector_rank: Option<usize>,
}

/// Per-term BM25 scoring decomposition for a single explanation hit.
///
/// Mirrors `VantaBm25TermContribution` (`src/sdk/types.rs`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bm25Term {
    /// The query term token.
    pub token: String,
    /// Term frequency in the matched document.
    pub tf: u32,
    /// Document frequency across the namespace.
    pub df: u64,
    /// Total length (in tokens) of the matched document.
    pub doc_len: u32,
    /// BM25 score contribution for this term.
    pub contribution: f32,
}

/// A stored memory record returned by `get` / `list`.
///
/// Mirrors `VantaMemoryRecord` (`src/sdk/types.rs`) so the UI can render
/// version, update time, TTL and vector data. Fields are `Option` + serde
/// default because the server backend (IQL nodes) cannot supply them — the
/// native backend fills every one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub namespace: String,
    pub text: String,
    #[serde(default)]
    pub vector: Option<Vec<f32>>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation time as unix milliseconds.
    #[serde(default)]
    pub created_at_ms: Option<u64>,
    /// Last-update time as unix milliseconds.
    #[serde(default)]
    pub updated_at_ms: Option<u64>,
    /// Monotonic version counter.
    #[serde(default)]
    pub version: Option<u64>,
    /// Deterministic node id derived from namespace and key (string: the core
    /// serializes `u128` node ids as strings to avoid JS precision loss).
    #[serde(default)]
    pub node_id: Option<String>,
    /// Optional sparse term-weight vector (dimension → coefficient).
    #[serde(default)]
    pub sparse_vector: Option<HashMap<u32, f32>>,
    /// Absolute unix-ms timestamp after which the record is considered
    /// expired. `None` means the record never expires.
    #[serde(default)]
    pub expires_at_ms: Option<u64>,
}

/// A page of records returned by `list`, with the cursor for the next page.
///
/// Mirrors `VantaMemoryListPage` (`src/sdk/types.rs`) so the UI can paginate
/// virtualized grids. `next_cursor` is a zero-based offset into the
/// namespace's stable id order; `None` means this was the last page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListPage {
    pub records: Vec<MemoryRecord>,
    #[serde(default)]
    pub next_cursor: Option<usize>,
}

/// Per-namespace record statistics (VS-CORE-02).
///
/// Mirrors `VantaNamespaceStats` (`src/sdk/types.rs`) 1:1 so the UI can show
/// real counts, expiring-soon and expired buckets per namespace without a
/// client-side `list()` scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NamespaceStats {
    /// Total records in the namespace, including expired (not-yet-purged) ones.
    pub count: u64,
    /// Records expiring within the `expiring_soon_window_ms` window (core
    /// default: 24h).
    pub expiring_soon: u64,
    /// Records already past their expiry (still present until purged).
    pub expired: u64,
}

/// Namespace → stats map (mirror of `VantaNamespaceStatsMap`).
pub type NamespaceStatsMap = BTreeMap<String, NamespaceStats>;

/// A single AND-combined metadata filter item for export/delete operations
/// (VS-CORE-04/05).
///
/// `op` reuses the core `VantaFilterOp` so wire values stay PascalCase
/// (`"Eq"`, `"Neq"`, `"Gt"`, ...) — the same shape the UI query builder emits
/// (`desktop/src/components/search/filters-core.ts`). `value` is untagged JSON
/// so any JSON-able value roundtrips.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryFilterItem {
    pub field: String,
    pub op: vantadb::sdk::VantaFilterOp,
    pub value: serde_json::Value,
}

/// Result of a namespace export (VS-CORE-04).
///
/// Mirrors `VantaExportReport` (`src/sdk/types.rs`) 1:1 so the UI can show
/// counts, path and duration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportReport {
    pub records_exported: u64,
    pub namespaces: Vec<String>,
    pub path: String,
    pub duration_ms: u64,
}

/// A single audit-log entry (VS-12).
///
/// Mirrors `vantadb::audit::AuditEvent` (`src/audit.rs`). The bridge carries its
/// own `Deserialize` because the core event is serialize-only; the wire JSONL
/// shape is identical (`record()` writes one object per line).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    /// ISO 8601 UTC timestamp (e.g. `2026-08-02T12:34:56Z`).
    pub timestamp: String,
    /// Operation name: `put`, `delete`, `delete_by_filter`, `put_batch`,
    /// `export_namespace`, `export_all`, `import_file`.
    pub op: String,
    pub namespace: String,
    /// Target record key, or `"N/A"` for operations without a single key.
    pub key: String,
    /// `"ok"` or `"err"`.
    pub outcome: String,
    /// Optional reason (e.g. the delete reason).
    #[serde(default)]
    pub reason: Option<String>,
}

/// A page of audit events with the cursor for the next page (VS-12).
///
/// Events are ordered newest-first (the audit log tail). `next_cursor` is the
/// offset into the *filtered* newest-first list for the next older page;
/// `None` means this was the last (oldest) page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditPage {
    /// Events ordered newest-first (audit log tail).
    pub events: Vec<AuditEvent>,
    /// Offset of the next older page; `None` = no older events.
    #[serde(default)]
    pub next_cursor: Option<usize>,
}

/// Result of [`crate::connections::VantaConnection::health`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    /// Backend engine the report describes (e.g. `"fjall"` for native embedded).
    #[serde(default = "default_backend")]
    pub backend: String,
    /// Round-trip latency in milliseconds.
    pub latency_ms: u64,
    /// Time the check was performed, unix milliseconds.
    pub checked_at_ms: u64,
    /// Optional human-readable diagnostic detail.
    #[serde(default)]
    pub message: Option<String>,
}

/// Static metadata describing a connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub name: String,
    /// Transport bridge the connection uses.
    pub via: Capability,
    pub status: ConnectionStatus,
    #[serde(default)]
    pub description: Option<String>,
}

/// Wire result of an IQL statement (VS-CORE-06).
///
/// Mirrors the core `VantaQueryResult` enum externally-tagged (no
/// `rename_all`): variant names serialize exactly as `Read` / `Write` /
/// `StaleContext`. `node_id` is a string on the wire (u128 ids exceed JS
/// `Number.MAX_SAFE_INTEGER`), consistent with [`MemoryRecord::node_id`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VantaQueryResult {
    /// Successful SELECT/QUERY/FETCH — the matching nodes.
    Read(Vec<MemoryRecord>),
    /// Successful INSERT/UPDATE/DELETE/RELATE — how many nodes were affected.
    Write {
        affected_nodes: u64,
        message: String,
        node_id: Option<String>,
    },
    /// Context-aware write returned a stale-context marker (client must
    /// re-sync the node's revision before retrying).
    StaleContext { node_id: String },
}

/// A single graph node on the wire (GRAFO-01).
///
/// `id` is a string on the wire (u128 ids exceed JS `Number.MAX_SAFE_INTEGER`,
/// consistent with [`MemoryRecord::node_id`]). `degree` carries the
/// in+out degree centrality when produced by a degree query; traversals leave
/// it at its default (0) since computing it requires a full-neighborhood scan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaGraphNodeInfo {
    /// Unique numeric node id, serialized as a string.
    pub id: String,
    /// Display label for the visor (content/text/__vanta_payload field, id fallback).
    pub label: String,
    /// Grouping key for coloring (namespace or node `type`), when known.
    #[serde(default)]
    pub group: Option<String>,
    /// In+out degree centrality (0 when not computed by the backend).
    #[serde(default)]
    pub degree: u64,
}

/// A directed graph edge on the wire (GRAFO-01).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaGraphEdgeInfo {
    /// Source node id (string — u128 on the core side).
    pub source: String,
    /// Target node id (string — u128 on the core side).
    pub target: String,
    /// Edge label, when the backend exposes one.
    #[serde(default)]
    pub label: Option<String>,
    /// Edge weight, when the backend exposes one.
    #[serde(default)]
    pub weight: Option<f32>,
}

/// Result of a graph traversal (bfs/dfs) on the wire (GRAFO-01).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VantaGraphTraversalResult {
    pub nodes: Vec<VantaGraphNodeInfo>,
    pub edges: Vec<VantaGraphEdgeInfo>,
}

fn default_namespace() -> String {
    "default".to_string()
}

fn default_top_k() -> usize {
    10
}

fn default_backend() -> String {
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use std::fmt::Debug;

    /// JSON roundtrip: serialize → deserialize must yield an equal value.
    fn rt<T: Serialize + DeserializeOwned + PartialEq + Debug>(v: &T) -> T {
        let json = serde_json::to_string(v).expect("serialize");
        serde_json::from_str(&json).expect("deserialize")
    }

    fn json<T: Serialize>(v: &T) -> String {
        serde_json::to_string(v).expect("serialize")
    }

    #[test]
    fn capability_roundtrips() {
        for cap in [
            Capability::Native,
            Capability::Http,
            Capability::Mcp,
            Capability::Node,
            Capability::Python,
            Capability::Wasm,
        ] {
            assert_eq!(rt(&cap), cap);
        }
        assert_eq!(json(&Capability::Http), r#""http""#);
    }

    #[test]
    fn connection_status_roundtrips() {
        for s in [
            ConnectionStatus::Connected,
            ConnectionStatus::Disconnected,
            ConnectionStatus::Error,
        ] {
            assert_eq!(rt(&s), s);
        }
    }

    #[test]
    fn health_status_roundtrips() {
        for s in [
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Unhealthy,
        ] {
            assert_eq!(rt(&s), s);
        }
    }

    #[test]
    fn ingest_item_roundtrip() {
        let item = IngestItem {
            id: Some("k1".into()),
            namespace: "mem".into(),
            text: "hello world".into(),
            embedding: Some(vec![0.5, -1.25, 3.0]),
            metadata: [("lang".to_string(), serde_json::Value::from("en"))]
                .into_iter()
                .collect(),
        };
        assert_eq!(rt(&item), item);
    }

    #[test]
    fn ingest_item_defaults_deserialize_absent_fields() {
        // Fields with `#[serde(default)]` must deserialize when absent in JSON.
        let json = r#"{"text":"hi"}"#;
        let item: IngestItem = serde_json::from_str(json).expect("deserialize");
        assert_eq!(item.namespace, "default");
        assert!(item.id.is_none());
        assert!(item.embedding.is_none());
        assert!(item.metadata.is_empty());
    }

    #[test]
    fn search_query_roundtrip() {
        let q = SearchQuery {
            query: "cats".into(),
            embedding: Some(vec![0.1, 0.2, 0.3]),
            top_k: 5,
            namespace: Some("mem".into()),
            filters: [("scope".into(), serde_json::Value::from("test"))]
                .into_iter()
                .collect(),
            explain: true,
        };
        assert_eq!(rt(&q), q);
    }

    #[test]
    fn search_query_explain_defaults_false_when_absent() {
        // Backward compat: JSON without `explain` must deserialize to false.
        let json = r#"{"query":"cats","top_k":5}"#;
        let q: SearchQuery = serde_json::from_str(json).expect("deserialize");
        assert!(!q.explain);
    }

    #[test]
    fn search_result_roundtrip() {
        let r = SearchResult {
            id: "r1".into(),
            namespace: "mem".into(),
            text: "cats".into(),
            score: 0.987,
            metadata: [("k".to_string(), serde_json::Value::from(42))]
                .into_iter()
                .collect(),
            explanation: None,
        };
        assert_eq!(rt(&r), r);
    }

    #[test]
    fn search_result_explanation_wire_shape() {
        // Explain-mode result: full ExplanationHit roundtrips and the wire shape
        // guards the vanta.ts contract (snake_case fields, bm25_terms array).
        let r = SearchResult {
            id: "r1".into(),
            namespace: "mem".into(),
            text: "cats sleep".into(),
            score: 1.25,
            metadata: HashMap::new(),
            explanation: Some(ExplanationHit {
                identity: "mem\0r1".into(),
                score: 1.25,
                snippet: Some("cats sleep".into()),
                matched_tokens: vec!["cats".into()],
                matched_phrases: vec!["cats sleep".into()],
                bm25_terms: vec![Bm25Term {
                    token: "cats".into(),
                    tf: 1,
                    df: 2,
                    doc_len: 2,
                    contribution: 0.75,
                }],
                rrf_text_rank: Some(1),
                rrf_vector_rank: Some(2),
            }),
        };
        assert_eq!(rt(&r), r);
        let json = json(&r);
        // `\0` in the identity string is JSON-escaped as `\u0000` by serde_json.
        assert!(json.contains(r#""explanation":{"identity":"mem\u0000r1""#));
        assert!(json.contains(
            r#""bm25_terms":[{"token":"cats","tf":1,"df":2,"doc_len":2,"contribution":0.75}]"#
        ));
        assert!(json.contains(r#""rrf_text_rank":1,"rrf_vector_rank":2"#));
    }

    #[test]
    fn memory_record_roundtrip() {
        let rec = MemoryRecord {
            id: "r1".into(),
            namespace: "mem".into(),
            text: "cats".into(),
            vector: Some(vec![0.1, -0.2]),
            metadata: HashMap::new(),
            created_at_ms: Some(1_700_000_000_000),
            updated_at_ms: Some(1_700_000_000_100),
            version: Some(3),
            node_id: Some("42".into()),
            sparse_vector: Some([(0u32, 0.5f32), (5, 1.25)].into_iter().collect()),
            expires_at_ms: Some(1_700_000_100_000),
        };
        assert_eq!(rt(&rec), rec);
        // Wire shape guards the vanta.ts contract: node_id is a string,
        // version/updated_at are numbers, expires/vector may be null.
        let json = json(&rec);
        assert!(json.contains(r#""version":3"#));
        assert!(json.contains(r#""node_id":"42""#));
        assert!(json.contains(r#""expires_at_ms":1700000100000"#));
        assert!(json.contains(r#""vector":[0.1,-0.2]"#));
    }

    #[test]
    fn audit_event_roundtrip() {
        let e = AuditEvent {
            timestamp: "2026-08-02T12:34:56Z".into(),
            op: "put".into(),
            namespace: "mem".into(),
            key: "k1".into(),
            outcome: "ok".into(),
            reason: Some("created".into()),
        };
        assert_eq!(rt(&e), e);
        // reason null/absent both deserialize to None.
        let json = r#"{"timestamp":"t","op":"put","namespace":"n","key":"k","outcome":"ok"}"#;
        let parsed: AuditEvent = serde_json::from_str(json).expect("deserialize");
        assert_eq!(parsed.reason, None);
    }

    #[test]
    fn audit_page_roundtrip() {
        let page = AuditPage {
            events: vec![AuditEvent {
                timestamp: "2026-08-02T12:34:56Z".into(),
                op: "put".into(),
                namespace: "mem".into(),
                key: "k1".into(),
                outcome: "ok".into(),
                reason: None,
            }],
            next_cursor: Some(3),
        };
        assert_eq!(rt(&page), page);
    }

    #[test]
    fn health_report_roundtrip() {
        let h = HealthReport {
            status: HealthStatus::Healthy,
            backend: "fjall".into(),
            latency_ms: 12,
            checked_at_ms: 1_700_000_000_000,
            message: Some("ok".into()),
        };
        assert_eq!(rt(&h), h);
    }

    #[test]
    fn connection_info_roundtrip() {
        let c = ConnectionInfo {
            id: "c1".into(),
            name: "local".into(),
            via: Capability::Native,
            status: ConnectionStatus::Connected,
            description: Some("embedded".into()),
        };
        assert_eq!(rt(&c), c);
    }

    #[test]
    fn memory_filter_item_roundtrip_wire_shape() {
        let item = MemoryFilterItem {
            field: "color".into(),
            op: vantadb::VantaFilterOp::Eq,
            value: serde_json::Value::from("red"),
        };
        assert_eq!(rt(&item), item);
        // Wire ops stay PascalCase — matches the UI query builder (filters-core.ts).
        let json = json(&item);
        assert_eq!(json, r#"{"field":"color","op":"Eq","value":"red"}"#);
    }

    #[test]
    fn export_report_roundtrip() {
        let r = ExportReport {
            records_exported: 2,
            namespaces: vec!["mem".into()],
            path: "C:/tmp/export.jsonl".into(),
            duration_ms: 7,
        };
        assert_eq!(rt(&r), r);
    }

    // ─── VantaQueryResult (VS-CORE-06) ───────────────────────────

    #[test]
    fn query_result_read_roundtrip() {
        let rec = MemoryRecord {
            id: "100".into(),
            namespace: "default".into(),
            text: "hello iql".into(),
            vector: None,
            metadata: [("name".to_string(), serde_json::json!("Ada"))].into(),
            created_at_ms: None,
            updated_at_ms: Some(1_700_000_000_000),
            version: Some(1),
            node_id: Some("100".into()),
            sparse_vector: None,
            expires_at_ms: None,
        };
        let res = VantaQueryResult::Read(vec![rec]);
        assert_eq!(rt(&res), res);
        // Wire shape mirrors the core enum: externally tagged variant names.
        let json = json(&res);
        assert!(json.starts_with(r#"{"Read":"#), "got {json}");
    }

    #[test]
    fn query_result_write_roundtrip() {
        let res = VantaQueryResult::Write {
            affected_nodes: 1,
            message: "inserted".into(),
            node_id: Some("100".into()),
        };
        assert_eq!(rt(&res), res);
        let json = json(&res);
        assert!(json.starts_with(r#"{"Write":"#), "got {json}");
        assert!(json.contains(r#""node_id":"100""#));
    }

    #[test]
    fn query_result_stale_context_wire_shape() {
        let res = VantaQueryResult::StaleContext {
            node_id: "42".into(),
        };
        assert_eq!(rt(&res), res);
        // Exact wire shape guards the vanta.ts contract.
        assert_eq!(json(&res), r#"{"StaleContext":{"node_id":"42"}}"#);
    }

    // ─── Graph DTOs (GRAFO-01) ────────────────────────────────────

    #[test]
    fn graph_node_info_roundtrip() {
        let n = VantaGraphNodeInfo {
            id: "42".into(),
            label: "Ada".into(),
            group: Some("people".into()),
            degree: 3,
        };
        assert_eq!(rt(&n), n);
        // Wire shape guards the vanta.ts contract: degree serialized, group present.
        let json = json(&n);
        assert_eq!(
            json,
            r#"{"id":"42","label":"Ada","group":"people","degree":3}"#
        );
    }

    #[test]
    fn graph_node_info_defaults_when_absent() {
        // Backward compat: group/degree optional on the wire.
        let json = r#"{"id":"42","label":"Ada"}"#;
        let n: VantaGraphNodeInfo = serde_json::from_str(json).expect("deserialize");
        assert_eq!(n.group, None);
        assert_eq!(n.degree, 0);
    }

    #[test]
    fn graph_edge_info_roundtrip() {
        let e = VantaGraphEdgeInfo {
            source: "1".into(),
            target: "2".into(),
            label: Some("knows".into()),
            weight: Some(0.5),
        };
        assert_eq!(rt(&e), e);
        // Optional label/weight absent → null/None on the wire.
        let json = json(&e);
        assert_eq!(
            json,
            r#"{"source":"1","target":"2","label":"knows","weight":0.5}"#
        );
    }

    #[test]
    fn graph_traversal_result_roundtrip() {
        let r = VantaGraphTraversalResult {
            nodes: vec![VantaGraphNodeInfo {
                id: "1".into(),
                label: "a".into(),
                group: None,
                degree: 0,
            }],
            edges: vec![VantaGraphEdgeInfo {
                source: "1".into(),
                target: "2".into(),
                label: None,
                weight: None,
            }],
        };
        assert_eq!(rt(&r), r);
        // Empty traversal serializes to empty arrays (not null) — the visor
        // expects nodes/edges to always be iterable.
        let json = json(&VantaGraphTraversalResult::default());
        assert_eq!(json, r#"{"nodes":[],"edges":[]}"#);
    }

    // ─── Namespace stats (VS-CORE-02) ────────────────────────────────

    #[test]
    fn namespace_stats_roundtrip_and_wire_shape() {
        let stats = NamespaceStats {
            count: 12,
            expiring_soon: 3,
            expired: 1,
        };
        assert_eq!(rt(&stats), stats);
        // Wire shape guards the vanta.ts contract (snake_case fields).
        assert_eq!(
            json(&stats),
            r#"{"count":12,"expiring_soon":3,"expired":1}"#
        );
    }

    #[test]
    fn namespace_stats_map_roundtrip() {
        let mut map = NamespaceStatsMap::new();
        map.insert(
            "docs".into(),
            NamespaceStats {
                count: 2,
                expiring_soon: 0,
                expired: 0,
            },
        );
        assert_eq!(rt(&map), map);
        assert_eq!(
            json(&map),
            r#"{"docs":{"count":2,"expiring_soon":0,"expired":0}}"#
        );
    }
}
