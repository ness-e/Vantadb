//! `GET /api/v2/list` application use case (D1c — cleanCA A2/F1).
//!
//! `records_list` in [`crate::server::handlers`] used to fan-out over all
//! namespaces, sort, merge pages and slice the window inline in the HTTP
//! handler. This module holds that orchestration behind a port trait so the
//! handler stays a humble object (DTO → use case → response) and the flow is
//! unit-testable without HTTP. Pattern reused from B1
//! (`super::conversation::StartConversationUseCase`).

use crate::error::Result;
use crate::sdk::{Embedded, MemoryFilter, MemoryListOptions, MemoryListPage, MemoryRecord};

/// Input for [`ListRecordsUseCase::execute`].
///
/// Mirrors [`crate::server::handlers::ListParams`] with the wire parsing
/// already done: `namespace` is `None` = fan-out over all namespaces (stable
/// name order), `Some(ns)` = single namespace. `limit` arrives already
/// clamped via `clamp_limit` in the handler (D5c) — the use case never
/// re-clamps so the wire stays identical.
#[derive(Debug, Clone)]
pub struct ListRecordsCommand {
    /// `None` = all namespaces; `Some(ns)` = single namespace.
    pub namespace: Option<String>,
    /// Advanced metadata filters (passthrough to `Embedded::list`).
    pub filter_ops: Option<MemoryFilter>,
    /// Page size (already clamped by the handler).
    pub limit: usize,
    /// Zero-based offset into the merged (all-ns) or single-ns window.
    pub cursor: Option<usize>,
}

/// Output of [`ListRecordsUseCase::execute`].
///
/// Single-namespace callers ignore `truncated_namespaces` (always empty) and
/// serialize `{records, next_cursor}`; all-namespaces callers serialize
/// `{records, next_cursor, truncated_namespaces}` — wire identical to the
/// inline handler.
#[derive(Debug, Clone, PartialEq)]
pub struct ListRecordsOutput {
    /// Records in the current window.
    pub records: Vec<MemoryRecord>,
    /// Cursor for the next page, or `None` if this was the last page.
    pub next_cursor: Option<usize>,
    /// Namespaces still paginating (their per-ns page had `next_cursor`).
    /// Empty for single-namespace queries.
    pub truncated_namespaces: Vec<String>,
}

/// Port abstracting the SDK surface the use case orchestrates.
///
/// Production: [`ServerListPorts`]. Tests: in-memory fake.
pub trait ListRecordsPorts {
    /// Namespace names in stable ascending order.
    fn list_namespaces_sorted(&self) -> Result<Vec<String>>;
    /// List one namespace with the given options.
    fn list_in(&self, namespace: &str, options: MemoryListOptions) -> Result<MemoryListPage>;
}

/// Merge per-namespace pages (stable namespace-name order) for the
/// `/api/v2/list` all-namespaces fan-out. A namespace whose page still has a
/// `next_cursor` was capped mid-listing — it is reported in the returned
/// `truncated_namespaces` so the client never sees silent truncation.
///
/// Moved verbatim from `handlers.rs` (D1c slice 1); the handler copy is
/// deleted in slice 3.
pub fn merge_all_namespaces_pages(
    pages: Vec<(String, MemoryListPage)>,
) -> (Vec<MemoryRecord>, Vec<String>) {
    let mut records = Vec::new();
    let mut truncated_namespaces = Vec::new();
    for (ns, page) in pages {
        if page.next_cursor.is_some() {
            truncated_namespaces.push(ns);
        }
        records.extend(page.records);
    }
    (records, truncated_namespaces)
}

/// Lists records in one namespace or fans out over all namespaces.
///
/// Order of effects (unchanged from the inline handler): namespaces sorted →
/// per-ns `list` → merge → slice window → `next_cursor`.
pub struct ListRecordsUseCase;

impl ListRecordsUseCase {
    /// Executes the command against `ports`.
    pub fn execute(
        ports: &impl ListRecordsPorts,
        cmd: ListRecordsCommand,
    ) -> Result<ListRecordsOutput> {
        if let Some(ns) = cmd.namespace {
            let options = MemoryListOptions {
                filter_ops: cmd.filter_ops,
                limit: cmd.limit,
                cursor: cmd.cursor,
                ..Default::default()
            };
            let page = ports.list_in(&ns, options)?;
            return Ok(ListRecordsOutput {
                records: page.records,
                next_cursor: page.next_cursor,
                truncated_namespaces: Vec::new(),
            });
        }
        let mut names = ports.list_namespaces_sorted()?;
        names.sort();
        let mut pages = Vec::with_capacity(names.len());
        for ns in names {
            let options = MemoryListOptions {
                filter_ops: cmd.filter_ops.clone(),
                limit: cmd.limit,
                cursor: cmd.cursor,
                ..Default::default()
            };
            let page = ports.list_in(&ns, options)?;
            pages.push((ns, page));
        }
        let (records, truncated_namespaces) = merge_all_namespaces_pages(pages);
        let start = cmd.cursor.unwrap_or(0).min(records.len());
        let end = (start + cmd.limit).min(records.len());
        let window = records[start..end].to_vec();
        let next_cursor = (end < records.len()).then_some(end);
        Ok(ListRecordsOutput {
            records: window,
            next_cursor,
            truncated_namespaces,
        })
    }
}

/// Production [`ListRecordsPorts`] backed by the [`Embedded`] SDK handle.
pub struct ServerListPorts<'a> {
    /// Embedded SDK handle (namespace stats + list).
    pub db: &'a Embedded,
}

impl ListRecordsPorts for ServerListPorts<'_> {
    fn list_namespaces_sorted(&self) -> Result<Vec<String>> {
        let mut names: Vec<String> = self.db.namespace_stats(None)?.keys().cloned().collect();
        names.sort();
        Ok(names)
    }

    fn list_in(&self, namespace: &str, options: MemoryListOptions) -> Result<MemoryListPage> {
        self.db.list(namespace, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    fn record(ns: &str, key: &str) -> MemoryRecord {
        MemoryRecord {
            namespace: ns.to_string(),
            key: key.to_string(),
            payload: format!("{ns}/{key}"),
            metadata: Default::default(),
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 1,
            node_id: 1,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        }
    }

    /// In-memory fake: no I/O, no HTTP, no engine (FIRST).
    /// `list_in` emulates `Embedded::list` limit/cursor slicing over the full
    /// per-namespace vec so truncation signals are realistic.
    struct FakePorts {
        data: Mutex<HashMap<String, Vec<MemoryRecord>>>,
        last_filter: Mutex<Option<MemoryFilter>>,
    }

    impl FakePorts {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
                last_filter: Mutex::new(None),
            }
        }

        fn with_ns(mut data: Vec<(String, Vec<MemoryRecord>)>) -> Self {
            let fake = Self::new();
            for (ns, recs) in data.drain(..) {
                fake.data.lock().unwrap().insert(ns, recs);
            }
            fake
        }

        fn cmd(namespace: Option<&str>, limit: usize, cursor: Option<usize>) -> ListRecordsCommand {
            ListRecordsCommand {
                namespace: namespace.map(str::to_string),
                filter_ops: None,
                limit,
                cursor,
            }
        }
    }

    impl ListRecordsPorts for FakePorts {
        fn list_namespaces_sorted(&self) -> Result<Vec<String>> {
            let mut names: Vec<String> = self.data.lock().unwrap().keys().cloned().collect();
            names.sort();
            Ok(names)
        }

        fn list_in(&self, namespace: &str, options: MemoryListOptions) -> Result<MemoryListPage> {
            *self.last_filter.lock().unwrap() = options.filter_ops.clone();
            let data = self.data.lock().unwrap();
            let full = data.get(namespace).cloned().unwrap_or_default();
            let cursor = options.cursor.unwrap_or(0);
            let start = cursor.min(full.len());
            let end = (start + options.limit).min(full.len());
            let records = full[start..end].to_vec();
            let next_cursor = (end < full.len()).then_some(end);
            Ok(MemoryListPage {
                records,
                next_cursor,
            })
        }
    }

    #[test]
    fn single_namespace_returns_page_verbatim_without_truncation() {
        // Arrange
        let ports = FakePorts::with_ns(vec![(
            "a".to_string(),
            vec![record("a", "k1"), record("a", "k2")],
        )]);
        // Act
        let out = ListRecordsUseCase::execute(&ports, FakePorts::cmd(Some("a"), 10, None)).unwrap();
        // Assert
        assert_eq!(out.records.len(), 2);
        assert_eq!(out.next_cursor, None);
        assert!(out.truncated_namespaces.is_empty());
    }

    #[test]
    fn all_namespaces_merges_in_sorted_name_order() {
        // Arrange: insert out of order on purpose.
        let ports = FakePorts::with_ns(vec![
            ("zeta".to_string(), vec![record("zeta", "k1")]),
            ("alpha".to_string(), vec![record("alpha", "k1")]),
        ]);
        // Act
        let out = ListRecordsUseCase::execute(&ports, FakePorts::cmd(None, 10, None)).unwrap();
        // Assert
        assert_eq!(out.records.len(), 2);
        assert_eq!(out.records[0].namespace, "alpha");
        assert_eq!(out.records[1].namespace, "zeta");
        assert_eq!(out.next_cursor, None);
    }

    #[test]
    fn all_namespaces_slices_window_with_identical_next_cursor_wire() {
        // Arrange: 2 ns × 2 records = 4 merged.
        let ports = FakePorts::with_ns(vec![
            ("a".to_string(), vec![record("a", "k1"), record("a", "k2")]),
            ("b".to_string(), vec![record("b", "k1"), record("b", "k2")]),
        ]);
        // Act: limit=3 over 4 → window [0..3], next_cursor Some(3).
        let out = ListRecordsUseCase::execute(&ports, FakePorts::cmd(None, 3, None)).unwrap();
        // Assert: wire formula start=min(cursor, len), end=min(start+limit, len).
        assert_eq!(out.records.len(), 3);
        assert_eq!(out.next_cursor, Some(3));
        // Act: follow the cursor. Quirk preservado del handler original
        // (wire idéntico, D1c no lo cambia): el cursor global se aplica
        // TAMBIÉN por namespace (options_for usaba el mismo `cursor`), así
        // que con cursor=3 cada ns de 2 registros devuelve página vacía y
        // el merge queda vacío con next_cursor None.
        let out2 = ListRecordsUseCase::execute(&ports, FakePorts::cmd(None, 3, Some(3))).unwrap();
        // Assert
        assert!(out2.records.is_empty());
        assert_eq!(out2.next_cursor, None);
    }

    #[test]
    fn all_namespaces_reports_truncated_namespaces() {
        // Arrange: ns "big" has 5 records but per-ns limit=2 → its page
        // carries next_cursor, so it must appear in truncated_namespaces.
        let ports = FakePorts::with_ns(vec![
            (
                "big".to_string(),
                vec![
                    record("big", "k1"),
                    record("big", "k2"),
                    record("big", "k3"),
                    record("big", "k4"),
                    record("big", "k5"),
                ],
            ),
            ("small".to_string(), vec![record("small", "k1")]),
        ]);
        // Act
        let out = ListRecordsUseCase::execute(&ports, FakePorts::cmd(None, 2, None)).unwrap();
        // Assert
        assert!(out.truncated_namespaces.contains(&"big".to_string()));
        assert!(!out.truncated_namespaces.contains(&"small".to_string()));
    }

    #[test]
    fn all_namespaces_empty_returns_empty_page() {
        // Arrange
        let ports = FakePorts::new();
        // Act
        let out = ListRecordsUseCase::execute(&ports, FakePorts::cmd(None, 10, None)).unwrap();
        // Assert
        assert!(out.records.is_empty());
        assert_eq!(out.next_cursor, None);
        assert!(out.truncated_namespaces.is_empty());
    }

    #[test]
    fn cursor_beyond_len_returns_empty_without_cursor() {
        // Arrange
        let ports = FakePorts::with_ns(vec![("a".to_string(), vec![record("a", "k1")])]);
        // Act: cursor=99 over 1 record → start=min(99,1)=1, end=min(1+10,1)=1.
        let out = ListRecordsUseCase::execute(&ports, FakePorts::cmd(None, 10, Some(99))).unwrap();
        // Assert
        assert!(out.records.is_empty());
        assert_eq!(out.next_cursor, None);
    }

    #[test]
    fn forwards_filter_ops_to_per_namespace_list() {
        // Arrange
        use crate::sdk::types::{FilterOp, MemoryFilterItem, Value};
        let ports = FakePorts::with_ns(vec![("a".to_string(), vec![record("a", "k1")])]);
        let filter = vec![MemoryFilterItem {
            field: "kind".to_string(),
            op: FilterOp::Eq,
            value: Value::String("note".to_string()),
        }];
        let cmd = ListRecordsCommand {
            namespace: None,
            filter_ops: Some(filter.clone()),
            limit: 10,
            cursor: None,
        };
        // Act
        let _ = ListRecordsUseCase::execute(&ports, cmd).unwrap();
        // Assert
        assert_eq!(*ports.last_filter.lock().unwrap(), Some(filter));
    }

    #[test]
    fn merge_all_namespaces_pages_marks_capped_namespaces() {
        // Arrange
        let pages = vec![
            (
                "a".to_string(),
                MemoryListPage {
                    records: vec![record("a", "k1")],
                    next_cursor: Some(1),
                },
            ),
            (
                "b".to_string(),
                MemoryListPage {
                    records: vec![record("b", "k1")],
                    next_cursor: None,
                },
            ),
        ];
        // Act
        let (records, truncated) = merge_all_namespaces_pages(pages);
        // Assert
        assert_eq!(records.len(), 2);
        assert_eq!(truncated, vec!["a".to_string()]);
    }
}
