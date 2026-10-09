//! Export and import operations for `Embedded`.

use super::super::builder::Embedded;
use super::{
    decode_node_id, export_line_from_record, matches_memory_filters, namespace_index_prefix,
    payload_index_prefix, record_from_export_line, record_from_node, validate_namespace,
};
use crate::backend::BackendPartition;
use crate::error::{Error, Result};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use tracing;
use web_time::Instant;

/// Audit reason for an import (SCH-05): the quarantined count when the import
/// option marked records (T1c), else `None`.
fn quarantine_reason_from_report(
    report: Option<&super::super::types::ImportReport>,
) -> Option<String> {
    match report {
        Some(r) if r.quarantined > 0 => Some(format!("{} quarantined", r.quarantined)),
        _ => None,
    }
}

/// Declared limits of the export integrity manifest (MEMG-15) — always
/// present, never silent (same contract as the VER-02 certificates:
/// integrity, not authenticity).
fn export_manifest_limits() -> Vec<String> {
    vec![
        "integrity, not authenticity: the manifest is an sha256 self-hash over the export bytes — an actor who edits the file and recomputes the manifest is not detected; there is no engine key and no digital signature (same contract as the VER-02 certificates; cryptographic signing is a vanta-audit decision)".into(),
        "the manifest covers the JSONL file bytes only: it does not bind the exporting database instance, the import target, or edits made after export; import_file does not enforce it — verification is explicit".into(),
        "transports that carry only the JSONL (e.g. the MCP inline export) do not carry the sidecar manifest".into(),
    ]
}

/// Sidecar path for an export file: `<path>.manifest.json`.
fn manifest_path_for(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".manifest.json");
    PathBuf::from(os)
}

/// Streaming sha256 of a file with bounded memory (64 KiB chunks) — mirrors
/// the write path, so verifying a large export never materializes the file
/// in memory (MEMG-15).
fn sha256_file(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(Error::Io)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf).map_err(Error::Io)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(crate::attestation::hex_lower(&hasher.finalize()))
}

/// Write the deterministic integrity manifest sidecar of an export file and
/// return its path. Crate-internal so the CLI export path shares the exact
/// same manifest instead of reimplementing it (MEMG-15).
pub(crate) fn write_export_manifest(
    path: &Path,
    sha256: &str,
    records: u64,
    namespaces: &[String],
) -> Result<PathBuf> {
    let mut namespaces: Vec<String> = namespaces.to_vec();
    namespaces.sort();
    namespaces.dedup();
    let manifest = super::super::types::ExportManifest {
        schema_version: super::super::types::EXPORT_MANIFEST_SCHEMA_VERSION,
        format: super::super::types::EXPORT_MANIFEST_FORMAT.to_string(),
        export_schema_version: super::EXPORT_SCHEMA_VERSION,
        records,
        namespaces,
        sha256: sha256.to_string(),
        limits: export_manifest_limits(),
    };
    let manifest_path = manifest_path_for(path);
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(Error::serialization)?;
    std::fs::write(&manifest_path, bytes).map_err(Error::Io)?;
    Ok(manifest_path)
}

impl Embedded {
    /// Validate a path against the configured export base dir, falling back to
    /// bare `..` traversal protection when no base dir is configured.
    /// `pub(crate)` so the sibling `sdk::api` import paths (`bulk_import_file`)
    /// share the same sandbox instead of reimplementing it (WIRE-09).
    pub(crate) fn resolve_export_path(&self, path: &Path) -> Result<PathBuf> {
        match self.config.export_base_dir.as_ref() {
            Some(base) => crate::storage::ops::resolve_against_base(base, path),
            None => {
                crate::storage::ops::prevent_path_traversal(&path.to_string_lossy())?;
                Ok(path.to_path_buf())
            }
        }
    }

    /// Stream all node IDs for a namespace prefix-scan, with optional `skip`
    /// and `take` early-exit bounds (zero-allocation: the prefix iterator is
    /// consumed until both bounds are satisfied; unused IDs are never copied
    /// into the returned Vec). `skip` + `take` together enable O(window)
    /// cursor pagination over the index — the previous implementation always
    /// materialized the full candidate set (O(ventana_total)) and sliced in
    /// memory, which made `list(limit=100)` over a 10k namespace allocate and
    /// scan 10k entries per request (FIND-24).
    pub(crate) fn indexed_ids_by_namespace(
        &self,
        engine: &crate::storage::StorageEngine,
        namespace: &str,
        skip: usize,
        take: Option<usize>,
    ) -> Result<(Vec<u128>, bool)> {
        let prefix = namespace_index_prefix(namespace);
        let entries =
            engine.scan_partition_prefix_iter(BackendPartition::NamespaceIndex, &prefix)?;
        let mut ids = Vec::new();
        let has_index_entries = super::super::Embedded::load_derived_index_state(engine)?.is_some();
        crate::metrics::record_derived_prefix_scan();

        let mut skipped = 0usize;
        let mut taken = 0usize;
        for entry in entries {
            let (_key, value) = entry?;
            let Some(node_id) = decode_node_id(&value) else {
                continue;
            };
            if skipped < skip {
                skipped += 1;
                continue;
            }
            ids.push(node_id);
            taken += 1;
            if let Some(t) = take {
                if taken >= t {
                    break;
                }
            }
        }

        Ok((ids, has_index_entries))
    }

    /// Filter variant of `indexed_ids_by_namespace` — same `skip`/`take`
    /// early-exit semantics for cursor pagination over payload-index entries
    /// (FIND-24).
    pub(crate) fn indexed_ids_by_filter(
        &self,
        engine: &crate::storage::StorageEngine,
        namespace: &str,
        field: &str,
        value: &super::super::types::Value,
        skip: usize,
        take: Option<usize>,
    ) -> Result<(Vec<u128>, bool)> {
        let prefix = payload_index_prefix(namespace, field, value)?;
        let entries = engine.scan_partition_prefix_iter(BackendPartition::PayloadIndex, &prefix)?;
        let mut ids = Vec::new();
        let has_index_entries = super::super::Embedded::load_derived_index_state(engine)?.is_some();
        crate::metrics::record_derived_prefix_scan();

        let mut skipped = 0usize;
        let mut taken = 0usize;
        for entry in entries {
            let (_key, value) = entry?;
            let Some(node_id) = decode_node_id(&value) else {
                continue;
            };
            if skipped < skip {
                skipped += 1;
                continue;
            }
            ids.push(node_id);
            taken += 1;
            if let Some(t) = take {
                if taken >= t {
                    break;
                }
            }
        }

        Ok((ids, has_index_entries))
    }

    pub(crate) fn records_for_namespace(
        &self,
        namespace: &str,
        filters: &super::super::types::MemoryMetadata,
    ) -> Result<Vec<super::super::types::MemoryRecord>> {
        let engine = self.engine_handle()?;

        let (candidate_ids, has_index_entries) = if let Some((field, value)) = filters.iter().next()
        {
            self.indexed_ids_by_filter(&engine, namespace, field, value, 0, None)?
        } else {
            self.indexed_ids_by_namespace(&engine, namespace, 0, None)?
        };

        let mut records = Vec::new();
        let mut seen = BTreeSet::new();
        let unique_ids: Vec<u128> = candidate_ids
            .into_iter()
            .filter(|id| seen.insert(*id))
            .collect();

        for node in engine.get_many(&unique_ids)? {
            if let Some(record) = record_from_node(&node) {
                if record.namespace == namespace && matches_memory_filters(&record, filters) {
                    records.push(record);
                }
            }
        }

        if records.is_empty() && !has_index_entries {
            crate::metrics::record_derived_full_scan_fallback();
            for node in engine.scan_nodes()? {
                if let Some(record) = record_from_node(&node) {
                    if record.namespace == namespace && matches_memory_filters(&record, filters) {
                        records.push(record);
                    }
                }
            }
        }

        records.sort_by(|a, b| a.key.cmp(&b.key).then(a.node_id.cmp(&b.node_id)));
        Ok(records)
    }

    /// Export all records in a namespace to a JSONL file.
    ///
    /// When `filter` is `Some`, only records matching the AND-combined filter
    /// items are exported (e.g. `Eq` on a metadata field). `None` (or an empty
    /// filter) exports the full namespace — backward-compatible with the
    /// pre-filter signature. Writes the integrity manifest sidecar
    /// (`<path>.manifest.json`) next to the file — verify with
    /// [`Self::verify_export_integrity`].
    #[tracing::instrument(skip(self, path), err)]
    pub fn export_namespace(
        &self,
        path: impl AsRef<Path>,
        namespace: &str,
        filter: Option<super::super::types::MemoryFilter>,
    ) -> Result<super::super::types::ExportReport> {
        let res = self.export_namespace_inner(path, namespace, filter);
        self.audit(crate::audit::AuditEvent::new(
            "export_namespace",
            namespace,
            "N/A",
            if res.is_ok() { "ok" } else { "err" },
            None,
        ));
        res
    }

    fn export_namespace_inner(
        &self,
        path: impl AsRef<Path>,
        namespace: &str,
        filter: Option<super::super::types::MemoryFilter>,
    ) -> Result<super::super::types::ExportReport> {
        validate_namespace(namespace)?;
        let resolved = self.resolve_export_path(path.as_ref())?;
        let started = Instant::now();
        let records = match filter {
            Some(ops) if !ops.is_empty() => {
                // Reuse list()'s filter_ops path (index-accelerated via the
                // first Eq op + matches_advanced_filters) — same paginated
                // scan as delete_by_filter. Some(vec![]) == None == full export.
                const PAGE_SIZE: usize = 500;
                let mut cursor: Option<usize> = None;
                let mut records: Vec<super::super::types::MemoryRecord> = Vec::new();
                loop {
                    let page = self.list(
                        namespace,
                        super::super::types::MemoryListOptions {
                            #[allow(deprecated)]
                            filters: super::super::types::MemoryMetadata::new(),
                            filter_ops: Some(ops.clone()),
                            limit: PAGE_SIZE,
                            cursor,
                            exclude_superseded: false,
                            as_of_ms: None,
                            valid_window: None,
                            include_quarantined: true,
                            min_confidence: None,
                        },
                    )?;
                    records.extend(page.records);
                    cursor = page.next_cursor;
                    if cursor.is_none() {
                        break;
                    }
                }
                records
            }
            _ => {
                self.records_for_namespace(namespace, &super::super::types::MemoryMetadata::new())?
            }
        };
        self.write_export_file(&resolved, records, vec![namespace.to_string()], started)
    }

    /// Export all records (every namespace) to a JSONL file, plus its
    /// integrity manifest sidecar (`<path>.manifest.json`) — verify with
    /// [`Self::verify_export_integrity`].
    #[tracing::instrument(skip(self, path), err)]
    pub fn export_all(&self, path: impl AsRef<Path>) -> Result<super::super::types::ExportReport> {
        let res = self.export_all_inner(path);
        self.audit(crate::audit::AuditEvent::new(
            "export_all",
            "N/A",
            "N/A",
            if res.is_ok() { "ok" } else { "err" },
            None,
        ));
        res
    }

    fn export_all_inner(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<super::super::types::ExportReport> {
        let resolved = self.resolve_export_path(path.as_ref())?;
        let started = Instant::now();
        let namespaces = self.list_namespaces()?;
        let mut records = Vec::new();
        for namespace in &namespaces {
            records.extend(
                self.records_for_namespace(namespace, &super::super::types::MemoryMetadata::new())?,
            );
        }
        self.write_export_file(&resolved, records, namespaces, started)
    }

    fn write_export_file(
        &self,
        path: &Path,
        records: Vec<super::super::types::MemoryRecord>,
        namespaces: Vec<String>,
        started: Instant,
    ) -> Result<super::super::types::ExportReport> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(Error::Io)?;
        }

        let file = File::create(path).map_err(Error::Io)?;
        let mut writer = BufWriter::new(file);
        let mut hasher = Sha256::new();
        let records_exported = records.len() as u64;

        // One reusable line buffer: hash and write the exact same bytes.
        let mut buffer: Vec<u8> = Vec::with_capacity(256);
        for record in records {
            let line = export_line_from_record(record);
            buffer.clear();
            serde_json::to_writer(&mut buffer, &line).map_err(Error::serialization)?;
            buffer.push(b'\n');
            hasher.update(&buffer);
            writer.write_all(&buffer).map_err(Error::Io)?;
        }
        writer.flush().map_err(Error::Io)?;
        let sha256 = crate::attestation::hex_lower(&hasher.finalize());
        let manifest_path = write_export_manifest(path, &sha256, records_exported, &namespaces)?;
        crate::metrics::record_export(records_exported);

        Ok(super::super::types::ExportReport {
            records_exported,
            namespaces,
            path: path.to_string_lossy().into_owned(),
            duration_ms: started.elapsed().as_millis() as u64,
            sha256,
            manifest_path: manifest_path.to_string_lossy().into_owned(),
        })
    }

    /// Verify a JSONL export file against its integrity manifest sidecar
    /// (MEMG-15).
    ///
    /// Reads `<path>.manifest.json` when present, recomputes the sha256 over
    /// the file bytes and reports `ok` (digest match), `mismatch` (digest
    /// diverged) or `no_manifest` (pre-manifest export — not verifiable,
    /// never reported as `ok`). Verification is never claim-driven, and
    /// `import_file` does not enforce the manifest: verification is explicit.
    #[tracing::instrument(skip(self, path), err)]
    pub fn verify_export_integrity(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<super::super::types::ExportIntegrityVerification> {
        let resolved = self.resolve_export_path(path.as_ref())?;
        let actual_sha256 = sha256_file(&resolved)?;

        let manifest_path = manifest_path_for(&resolved);
        let manifest = match std::fs::read_to_string(&manifest_path) {
            Ok(raw) => serde_json::from_str::<super::super::types::ExportManifest>(&raw)
                .map_err(Error::serialization)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(super::super::types::ExportIntegrityVerification {
                    schema_version: 0,
                    path: resolved.to_string_lossy().into_owned(),
                    status: "no_manifest".into(),
                    expected_sha256: None,
                    actual_sha256,
                    records: 0,
                    namespaces: Vec::new(),
                    limits: Vec::new(),
                });
            }
            Err(e) => return Err(Error::Io(e)),
        };

        // Discriminator guard (review O2): a sidecar from another tool/format
        // must not be judged by digest semantics it does not follow.
        if manifest.format != super::super::types::EXPORT_MANIFEST_FORMAT {
            return Err(Error::Validation {
                field: "manifest.format".into(),
                reason: format!(
                    "not a VantaDB export manifest (format: {})",
                    manifest.format
                ),
            });
        }

        let status = if manifest.sha256 == actual_sha256 {
            "ok"
        } else {
            "mismatch"
        };
        Ok(super::super::types::ExportIntegrityVerification {
            schema_version: manifest.schema_version,
            path: resolved.to_string_lossy().into_owned(),
            status: status.into(),
            expected_sha256: Some(manifest.sha256),
            actual_sha256,
            records: manifest.records,
            namespaces: manifest.namespaces,
            limits: manifest.limits,
        })
    }

    #[tracing::instrument(skip(self, records), err)]
    pub fn import_records(
        &self,
        records: Vec<super::super::types::MemoryRecord>,
        quarantine: bool,
    ) -> Result<super::super::types::ImportReport> {
        // SCH-05 review F3: the public entry audits itself (the direct
        // `import_records` path was un-audited; `import_file` already audited).
        // `import_file` calls `import_records_inner` so each public entry
        // audits exactly once.
        let res = self.import_records_inner(records, quarantine);
        self.audit(crate::audit::AuditEvent::new(
            "import_records",
            "N/A",
            "N/A",
            if res.is_ok() { "ok" } else { "err" },
            quarantine_reason_from_report(res.as_ref().ok()),
        ));
        res
    }

    fn import_records_inner(
        &self,
        records: Vec<super::super::types::MemoryRecord>,
        quarantine: bool,
    ) -> Result<super::super::types::ImportReport> {
        if self.config.read_only {
            return Err(Error::Validation {
                field: "read_only".into(),
                reason: "import_records is not available when VantaDB is opened read-only".into(),
            });
        }
        let started = Instant::now();
        let mut report = super::super::types::ImportReport {
            inserted: 0,
            updated: 0,
            skipped: 0,
            errors: 0,
            quarantined: 0,
            duration_ms: 0,
        };
        // T1c (ADR-046 §D5, MGR-13 §3.2/§4.3): the import-level quarantine
        // option marks records that do not already carry quarantine state.
        // Records imported with their own state (roundtrip) keep it verbatim —
        // exact transport reproduces the exported state.
        let imported_at_ms = super::now_ms();

        for mut record in records {
            let entered = quarantine && record.quarantined_at_ms.is_none();
            if entered {
                self.enter_quarantine(
                    &mut record,
                    "unreviewed_import",
                    "system:import",
                    imported_at_ms,
                );
            }
            let existed = matches!(self.get(&record.namespace, &record.key), Ok(Some(_)));
            match self.put_record_exact(record) {
                Ok(_) => {
                    // SCH-05 review N1: count only records that actually
                    // persisted (a failed put must not inflate the report).
                    if entered {
                        report.quarantined += 1;
                    }
                    if existed {
                        report.updated += 1;
                    } else {
                        report.inserted += 1;
                    }
                }
                Err(_) => report.errors += 1,
            }
        }

        self.rebuild_derived_indexes()?;
        self.rebuild_text_index()?;
        report.duration_ms = started.elapsed().as_millis() as u64;
        crate::metrics::record_import(report.inserted + report.updated, report.errors);
        Ok(report)
    }

    #[tracing::instrument(skip(self, path), err)]
    pub fn import_file(
        &self,
        path: impl AsRef<Path>,
        quarantine: bool,
    ) -> Result<super::super::types::ImportReport> {
        let res = self.import_file_inner(path, quarantine);
        self.audit(crate::audit::AuditEvent::new(
            "import_file",
            "N/A",
            "N/A",
            if res.is_ok() { "ok" } else { "err" },
            quarantine_reason_from_report(res.as_ref().ok()),
        ));
        res
    }

    fn import_file_inner(
        &self,
        path: impl AsRef<Path>,
        quarantine: bool,
    ) -> Result<super::super::types::ImportReport> {
        let resolved = self.resolve_export_path(path.as_ref())?;
        if self.config.read_only {
            return Err(Error::Validation {
                field: "read_only".into(),
                reason: "import_file is not available when VantaDB is opened read-only".into(),
            });
        }
        let started = Instant::now();
        let file = File::open(&resolved).map_err(Error::Io)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut skipped = 0u64;
        let mut errors = 0u64;

        for line in reader.lines() {
            let line = line.map_err(Error::Io)?;
            if line.trim().is_empty() {
                skipped += 1;
                continue;
            }

            match serde_json::from_str::<super::super::types::MemoryExportLine>(&line)
                .map_err(Error::serialization)
                .and_then(record_from_export_line)
            {
                Ok(record) => records.push(record),
                Err(_) => errors += 1,
            }
        }

        let mut report = self.import_records_inner(records, quarantine)?;
        report.skipped += skipped;
        report.errors += errors;
        if errors > 0 {
            crate::metrics::record_import(0, errors);
        }
        report.duration_ms = started.elapsed().as_millis() as u64;
        Ok(report)
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::super::super::connect::connect;
    use super::super::super::types::*;
    use crate::backend::BackendKind;
    use crate::config::Config;
    use crate::sdk::builder::Embedded;

    fn in_memory_db() -> Embedded {
        connect(":memory:").expect("in-memory db")
    }

    fn sample_input(namespace: &str, key: &str) -> MemoryInput {
        MemoryInput {
            namespace: namespace.into(),
            key: key.into(),
            payload: format!("payload for {key}"),
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        }
    }

    fn sample_input_with_meta(namespace: &str, key: &str, color: &str) -> MemoryInput {
        let mut metadata = MemoryMetadata::new();
        metadata.insert("color".into(), Value::String(color.into()));
        MemoryInput {
            namespace: namespace.into(),
            key: key.into(),
            payload: format!("payload for {key}"),
            metadata,
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        }
    }

    // ─── export_namespace ──────────────────────────────────────

    #[test]
    fn test_export_namespace_happy_path() {
        let db = in_memory_db();
        db.put(sample_input("myns", "key1")).unwrap();
        db.put(sample_input("myns", "key2")).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.jsonl");
        let report = db.export_namespace(&path, "myns", None).unwrap();

        assert_eq!(report.records_exported, 2);
        assert_eq!(report.namespaces, vec!["myns"]);
        assert!(path.exists());

        // Verify file contains two JSON lines
        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("key1"));
        assert!(lines[1].contains("key2"));
    }

    #[test]
    fn test_export_namespace_empty() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.jsonl");
        let report = db.export_namespace(&path, "nonexistent", None).unwrap();
        assert_eq!(report.records_exported, 0);
    }

    #[test]
    fn test_export_namespace_invalid_name() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.jsonl");
        let err = db.export_namespace(&path, "", None).unwrap_err();
        assert!(err.to_string().contains("namespace must not be empty"));
    }

    #[test]
    fn test_export_namespace_with_filter_eq_only_includes_matching() {
        let db = in_memory_db();
        db.put(sample_input_with_meta("myns", "red1", "red"))
            .unwrap();
        db.put(sample_input_with_meta("myns", "blue1", "blue"))
            .unwrap();
        db.put(sample_input("myns", "nocolor")).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("filtered.jsonl");
        let filter = vec![MemoryFilterItem {
            field: "color".into(),
            op: FilterOp::Eq,
            value: Value::String("red".into()),
        }];
        let report = db.export_namespace(&path, "myns", Some(filter)).unwrap();

        assert_eq!(report.records_exported, 1);
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("red1"));
        assert!(!content.contains("blue1"));
        assert!(!content.contains("nocolor"));
    }

    // ─── export_all ────────────────────────────────────────────

    #[test]
    fn test_export_all_happy_path() {
        let db = in_memory_db();
        db.put(sample_input("ns1", "a")).unwrap();
        db.put(sample_input("ns1", "b")).unwrap();
        db.put(sample_input("ns2", "c")).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("all.jsonl");
        let report = db.export_all(&path).unwrap();

        assert_eq!(report.records_exported, 3);
        assert!(report.namespaces.contains(&"ns1".to_string()));
        assert!(report.namespaces.contains(&"ns2".to_string()));
    }

    #[test]
    fn test_export_all_empty_db() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("all.jsonl");
        let report = db.export_all(&path).unwrap();
        assert_eq!(report.records_exported, 0);
        assert!(report.namespaces.is_empty());
    }

    // ─── import_records ────────────────────────────────────────

    #[test]
    fn test_import_records_insert() {
        let db = in_memory_db();
        let record = MemoryRecord {
            namespace: "imp".into(),
            key: "k1".into(),
            payload: "imported".into(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 100,
            updated_at_ms: 100,
            version: 1,
            node_id: crate::sdk::serialization::memory_node_id("imp", "k1"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            ..Default::default()
        };
        let report = db.import_records(vec![record], false).unwrap();
        assert_eq!(report.inserted, 1);
        assert_eq!(report.updated, 0);
        assert_eq!(report.errors, 0);

        // Verify it was stored
        let result = db.get("imp", "k1").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().payload, "imported");
    }

    #[test]
    fn test_import_records_update() {
        let db = in_memory_db();
        db.put(sample_input("upd", "k1")).unwrap();

        // Generate the same node_id by using same namespace+key
        let record = MemoryRecord {
            namespace: "upd".into(),
            key: "k1".into(),
            payload: "updated".into(),
            metadata: {
                let mut m = MemoryMetadata::new();
                m.insert("new".into(), Value::String("field".into()));
                m
            },
            created_at_ms: 100,
            updated_at_ms: 200,
            version: 2,
            node_id: crate::sdk::serialization::memory_node_id("upd", "k1"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            ..Default::default()
        };
        let report = db.import_records(vec![record], false).unwrap();
        assert_eq!(report.updated, 1);

        let retrieved = db.get("upd", "k1").unwrap().unwrap();
        assert_eq!(retrieved.payload, "updated");
    }

    #[test]
    fn test_import_records_rejects_wrong_node_id() {
        let db = in_memory_db();
        let record = MemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "bad".into(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 1,
            node_id: 999, // wrong — doesn't match hash of "ns"/"k"
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            ..Default::default()
        };
        let report = db.import_records(vec![record], false).unwrap();
        assert_eq!(report.errors, 1);
    }

    // ─── import_file ───────────────────────────────────────────

    #[test]
    fn test_import_file_happy_path() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("import.jsonl");

        let record = MemoryRecord {
            namespace: "file".into(),
            key: "k1".into(),
            payload: "from file".into(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 10,
            updated_at_ms: 10,
            version: 1,
            node_id: crate::sdk::serialization::memory_node_id("file", "k1"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            ..Default::default()
        };
        let line = super::super::export_line_from_record(record);
        let json = serde_json::to_string(&line).unwrap();
        std::fs::write(&path, json + "\n").unwrap();

        let report = db.import_file(&path, false).unwrap();
        assert_eq!(report.inserted, 1);
        assert_eq!(report.errors, 0);

        let result = db.get("file", "k1").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_import_file_skips_empty_lines() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty_lines.jsonl");

        let mut content = String::new();
        // One valid record
        let record = MemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "p".into(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 1,
            updated_at_ms: 1,
            version: 1,
            node_id: crate::sdk::serialization::memory_node_id("ns", "k"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            ..Default::default()
        };
        let line = super::super::export_line_from_record(record);
        content.push_str(&serde_json::to_string(&line).unwrap());
        content.push('\n');
        content.push('\n'); // empty line
        content.push('\n'); // another empty line
        std::fs::write(&path, content).unwrap();

        let report = db.import_file(&path, false).unwrap();
        assert_eq!(report.inserted, 1);
        assert_eq!(report.skipped, 2);
    }

    #[test]
    fn test_import_file_handles_malformed_lines() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.jsonl");
        std::fs::write(&path, "not json\n{\"bad\": true}\n").unwrap();

        let report = db.import_file(&path, false).unwrap();
        assert_eq!(report.errors, 2);
    }

    #[test]
    fn test_import_file_read_only_rejected() {
        let config = Config {
            storage_path: ":memory:".to_string(),
            backend_kind: BackendKind::InMemory,
            read_only: true,
            ..Default::default()
        };
        let db = Embedded::open_with_config(config).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.jsonl");
        std::fs::write(&path, "").unwrap();
        let err = db.import_file(&path, false).unwrap_err();
        assert!(err.to_string().contains("read-only"));
    }

    // ─── records_for_namespace ─────────────────────────────────

    #[test]
    fn test_records_for_namespace_with_filter() {
        let db = in_memory_db();
        db.put(sample_input_with_meta("ns", "red", "red")).unwrap();
        db.put(sample_input_with_meta("ns", "blue", "blue"))
            .unwrap();
        db.put(sample_input("ns", "nocolor")).unwrap();

        let mut filters = MemoryMetadata::new();
        filters.insert("color".into(), Value::String("red".into()));
        let records = db
            .records_for_namespace("ns", &filters)
            .expect("filtered records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].key, "red");
    }

    #[test]
    fn test_records_for_namespace_no_filter() {
        let db = in_memory_db();
        db.put(sample_input("ns", "a")).unwrap();
        db.put(sample_input("ns", "b")).unwrap();

        let records = db
            .records_for_namespace("ns", &MemoryMetadata::new())
            .expect("all records");
        assert_eq!(records.len(), 2);
    }

    // ─── export/import roundtrip ───────────────────────────────

    #[test]
    fn test_export_import_roundtrip() {
        let db1 = in_memory_db();
        db1.put(sample_input_with_meta("rt", "k1", "green"))
            .unwrap();
        db1.put(sample_input_with_meta("rt", "k2", "blue")).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rt.jsonl");
        let report = db1.export_namespace(&path, "rt", None).unwrap();
        assert_eq!(report.records_exported, 2);

        let db2 = in_memory_db();
        let import_report = db2.import_file(&path, false).unwrap();
        assert_eq!(import_report.inserted, 2);

        // Verify content
        let r1 = db2.get("rt", "k1").unwrap().unwrap();
        assert_eq!(r1.payload, "payload for k1");
        let r2 = db2.get("rt", "k2").unwrap().unwrap();
        assert_eq!(r2.payload, "payload for k2");
    }

    // ─── export integrity manifest (MEMG-15) ───────────────────

    #[test]
    fn export_writes_integrity_manifest_sidecar() {
        let db = in_memory_db();
        db.put(sample_input("mns", "k1")).unwrap();
        db.put(sample_input("mns", "k2")).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.jsonl");
        let report = db.export_namespace(&path, "mns", None).unwrap();

        assert_eq!(report.sha256.len(), 64, "sha256 hex digest");
        assert!(report.sha256.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(
            std::path::Path::new(&report.manifest_path).exists(),
            "manifest sidecar missing: {}",
            report.manifest_path
        );
        assert!(report.manifest_path.ends_with("export.jsonl.manifest.json"));

        let raw = std::fs::read_to_string(&report.manifest_path).unwrap();
        let manifest: ExportManifest = serde_json::from_str(&raw).unwrap();
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.format, "vantadb-memory-jsonl");
        assert_eq!(manifest.export_schema_version, 2);
        assert_eq!(manifest.records, 2);
        assert_eq!(manifest.namespaces, vec!["mns".to_string()]);
        assert_eq!(manifest.sha256, report.sha256);
        assert!(
            !manifest.limits.is_empty(),
            "declared limits must never be empty"
        );
        assert!(manifest
            .limits
            .iter()
            .any(|l| l.contains("not authenticity")));
    }

    #[test]
    fn verify_export_integrity_ok_then_mismatch_on_tamper() {
        let db = in_memory_db();
        db.put(sample_input("vns", "k1")).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("verify.jsonl");
        db.export_namespace(&path, "vns", None).unwrap();

        let ok = db.verify_export_integrity(&path).unwrap();
        assert_eq!(ok.status, "ok");
        assert_eq!(
            ok.expected_sha256.as_deref(),
            Some(ok.actual_sha256.as_str())
        );
        assert_eq!(ok.records, 1);
        assert_eq!(ok.namespaces, vec!["vns".to_string()]);
        assert!(!ok.limits.is_empty());

        // Tamper one byte in the middle of the file: the digest must diverge.
        let mut bytes = std::fs::read(&path).unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] = bytes[mid].wrapping_add(1);
        std::fs::write(&path, bytes).unwrap();

        let tampered = db.verify_export_integrity(&path).unwrap();
        assert_eq!(tampered.status, "mismatch");
        assert_ne!(tampered.expected_sha256.unwrap(), tampered.actual_sha256);
    }

    #[test]
    fn verify_export_integrity_reports_missing_manifest() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("plain.jsonl");
        db.export_namespace(&path, "nns", None).unwrap();

        // Simulate a pre-manifest export: remove the sidecar.
        std::fs::remove_file(format!("{}.manifest.json", path.display())).unwrap();

        let v = db.verify_export_integrity(&path).unwrap();
        assert_eq!(v.status, "no_manifest");
        assert!(v.expected_sha256.is_none());
        assert_eq!(v.actual_sha256.len(), 64);
    }

    #[test]
    fn verify_export_integrity_rejects_corrupt_manifest() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupt.jsonl");
        db.export_namespace(&path, "cns", None).unwrap();

        // Garbage sidecar: must surface as a mapped error, never as a verdict.
        std::fs::write(format!("{}.manifest.json", path.display()), "{not json").unwrap();
        let err = db.verify_export_integrity(&path).unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Serialization(_)),
            "corrupt manifest must be a Serialization error, got: {err}"
        );
    }

    #[test]
    fn verify_export_integrity_rejects_foreign_manifest_format() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("foreign.jsonl");
        db.export_namespace(&path, "fns", None).unwrap();

        // A well-formed manifest with the wrong format discriminator must not
        // be judged by digest semantics it does not follow (review O2).
        let manifest_path = format!("{}.manifest.json", path.display());
        let raw = std::fs::read_to_string(&manifest_path).unwrap();
        let mut manifest: ExportManifest = serde_json::from_str(&raw).unwrap();
        manifest.format = "some-other-format".into();
        std::fs::write(&manifest_path, serde_json::to_string(&manifest).unwrap()).unwrap();

        let err = db.verify_export_integrity(&path).unwrap_err();
        assert!(
            err.to_string().contains("not a VantaDB export manifest"),
            "got: {err}"
        );
    }

    #[test]
    fn verify_export_integrity_missing_file_is_error() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ghost.jsonl");

        // No file at all: an Io error, never a fabricated status.
        let err = db.verify_export_integrity(&path).unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Io(_)),
            "missing export must be an Io error, got: {err}"
        );
    }

    #[test]
    fn export_verify_import_roundtrip_is_intact_and_deterministic() {
        let db = in_memory_db();
        db.put(sample_input_with_meta("rti", "k1", "green"))
            .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("roundtrip.jsonl");
        let first = db.export_namespace(&path, "rti", None).unwrap();
        assert_eq!(db.verify_export_integrity(&path).unwrap().status, "ok");

        // Re-exporting unchanged data is byte-stable (same sha256).
        let second = db.export_namespace(&path, "rti", None).unwrap();
        assert_eq!(second.sha256, first.sha256);

        // The verified file still imports cleanly into a fresh database.
        let db2 = in_memory_db();
        let report = db2.import_file(&path, false).unwrap();
        assert_eq!(report.inserted, 1);
        assert_eq!(report.errors, 0);
        let record = db2.get("rti", "k1").unwrap().unwrap();
        assert_eq!(record.payload, "payload for k1");
    }
}
