//! Export and import operations for `VantaEmbedded`.

use super::super::builder::VantaEmbedded;
use super::{
    decode_node_id, export_line_from_record, matches_memory_filters, memory_record_from_node,
    namespace_index_prefix, payload_index_prefix, record_from_export_line, validate_namespace,
};
use crate::backend::BackendPartition;
use crate::error::{Result, VantaError};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing;
use web_time::Instant;

impl VantaEmbedded {
    /// Validate a path against the configured export base dir, falling back to
    /// bare `..` traversal protection when no base dir is configured.
    fn resolve_export_path(&self, path: &Path) -> Result<PathBuf> {
        match self.config.export_base_dir.as_ref() {
            Some(base) => crate::storage::ops::resolve_against_base(base, path),
            None => {
                crate::storage::ops::prevent_path_traversal(&path.to_string_lossy())?;
                Ok(path.to_path_buf())
            }
        }
    }
    pub(crate) fn indexed_ids_by_namespace(
        &self,
        engine: &crate::storage::StorageEngine,
        namespace: &str,
    ) -> Result<(Vec<u128>, bool)> {
        let prefix = namespace_index_prefix(namespace);
        let entries =
            engine.scan_partition_prefix_iter(BackendPartition::NamespaceIndex, &prefix)?;
        let mut ids = Vec::new();
        let has_index_entries =
            super::super::VantaEmbedded::load_derived_index_state(engine)?.is_some();
        crate::metrics::record_derived_prefix_scan();

        for entry in entries {
            let (_key, value) = entry?;
            if let Some(node_id) = decode_node_id(&value) {
                ids.push(node_id);
            }
        }

        Ok((ids, has_index_entries))
    }

    pub(crate) fn indexed_ids_by_filter(
        &self,
        engine: &crate::storage::StorageEngine,
        namespace: &str,
        field: &str,
        value: &super::super::types::VantaValue,
    ) -> Result<(Vec<u128>, bool)> {
        let prefix = payload_index_prefix(namespace, field, value)?;
        let entries = engine.scan_partition_prefix_iter(BackendPartition::PayloadIndex, &prefix)?;
        let mut ids = Vec::new();
        let has_index_entries =
            super::super::VantaEmbedded::load_derived_index_state(engine)?.is_some();
        crate::metrics::record_derived_prefix_scan();

        for entry in entries {
            let (_key, value) = entry?;
            if let Some(node_id) = decode_node_id(&value) {
                ids.push(node_id);
            }
        }

        Ok((ids, has_index_entries))
    }

    pub(crate) fn records_for_namespace(
        &self,
        namespace: &str,
        filters: &super::super::types::VantaMemoryMetadata,
    ) -> Result<Vec<super::super::types::VantaMemoryRecord>> {
        let engine = self.engine_handle()?;

        let (candidate_ids, has_index_entries) = if let Some((field, value)) = filters.iter().next()
        {
            self.indexed_ids_by_filter(&engine, namespace, field, value)?
        } else {
            self.indexed_ids_by_namespace(&engine, namespace)?
        };

        let mut records = Vec::new();
        let mut seen = BTreeSet::new();
        let unique_ids: Vec<u128> = candidate_ids
            .into_iter()
            .filter(|id| seen.insert(*id))
            .collect();

        for node in engine.get_many(&unique_ids)? {
            if let Some(record) = memory_record_from_node(&node) {
                if record.namespace == namespace && matches_memory_filters(&record, filters) {
                    records.push(record);
                }
            }
        }

        if records.is_empty() && !has_index_entries {
            crate::metrics::record_derived_full_scan_fallback();
            for node in engine.scan_nodes()? {
                if let Some(record) = memory_record_from_node(&node) {
                    if record.namespace == namespace && matches_memory_filters(&record, filters) {
                        records.push(record);
                    }
                }
            }
        }

        records.sort_by(|a, b| a.key.cmp(&b.key).then(a.node_id.cmp(&b.node_id)));
        Ok(records)
    }

    #[tracing::instrument(skip(self, path), err)]
    pub fn export_namespace(
        &self,
        path: impl AsRef<Path>,
        namespace: &str,
    ) -> Result<super::super::types::VantaExportReport> {
        let res = self.export_namespace_inner(path, namespace);
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
    ) -> Result<super::super::types::VantaExportReport> {
        validate_namespace(namespace)?;
        let resolved = self.resolve_export_path(path.as_ref())?;
        let started = Instant::now();
        let records = self
            .records_for_namespace(namespace, &super::super::types::VantaMemoryMetadata::new())?;
        self.write_export_file(&resolved, records, vec![namespace.to_string()], started)
    }

    #[tracing::instrument(skip(self, path), err)]
    pub fn export_all(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<super::super::types::VantaExportReport> {
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
    ) -> Result<super::super::types::VantaExportReport> {
        let resolved = self.resolve_export_path(path.as_ref())?;
        let started = Instant::now();
        let namespaces = self.list_namespaces()?;
        let mut records = Vec::new();
        for namespace in &namespaces {
            records.extend(self.records_for_namespace(
                namespace,
                &super::super::types::VantaMemoryMetadata::new(),
            )?);
        }
        self.write_export_file(&resolved, records, namespaces, started)
    }

    fn write_export_file(
        &self,
        path: &Path,
        records: Vec<super::super::types::VantaMemoryRecord>,
        namespaces: Vec<String>,
        started: Instant,
    ) -> Result<super::super::types::VantaExportReport> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(VantaError::IoError)?;
        }

        let file = File::create(path).map_err(VantaError::IoError)?;
        let mut writer = BufWriter::new(file);
        let records_exported = records.len() as u64;

        for record in records {
            let line = export_line_from_record(record);
            serde_json::to_writer(&mut writer, &line).map_err(VantaError::serialization)?;
            writer.write_all(b"\n").map_err(VantaError::IoError)?;
        }
        writer.flush().map_err(VantaError::IoError)?;
        crate::metrics::record_export(records_exported);

        Ok(super::super::types::VantaExportReport {
            records_exported,
            namespaces,
            path: path.to_string_lossy().into_owned(),
            duration_ms: started.elapsed().as_millis() as u64,
        })
    }

    #[tracing::instrument(skip(self, records), err)]
    pub fn import_records(
        &self,
        records: Vec<super::super::types::VantaMemoryRecord>,
    ) -> Result<super::super::types::VantaImportReport> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "import_records is not available when VantaDB is opened read-only".into(),
            });
        }
        let started = Instant::now();
        let mut report = super::super::types::VantaImportReport {
            inserted: 0,
            updated: 0,
            skipped: 0,
            errors: 0,
            duration_ms: 0,
        };

        for record in records {
            let existed = matches!(self.get(&record.namespace, &record.key), Ok(Some(_)));
            match self.put_record_exact(record) {
                Ok(_) if existed => report.updated += 1,
                Ok(_) => report.inserted += 1,
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
    ) -> Result<super::super::types::VantaImportReport> {
        let res = self.import_file_inner(path);
        self.audit(crate::audit::AuditEvent::new(
            "import_file",
            "N/A",
            "N/A",
            if res.is_ok() { "ok" } else { "err" },
            None,
        ));
        res
    }

    fn import_file_inner(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<super::super::types::VantaImportReport> {
        let resolved = self.resolve_export_path(path.as_ref())?;
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "import_file is not available when VantaDB is opened read-only".into(),
            });
        }
        let started = Instant::now();
        let file = File::open(&resolved).map_err(VantaError::IoError)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut skipped = 0u64;
        let mut errors = 0u64;

        for line in reader.lines() {
            let line = line.map_err(VantaError::IoError)?;
            if line.trim().is_empty() {
                skipped += 1;
                continue;
            }

            match serde_json::from_str::<super::super::types::VantaMemoryExportLine>(&line)
                .map_err(VantaError::serialization)
                .and_then(record_from_export_line)
            {
                Ok(record) => records.push(record),
                Err(_) => errors += 1,
            }
        }

        let mut report = self.import_records(records)?;
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
    use crate::config::VantaConfig;
    use crate::sdk::builder::VantaEmbedded;

    fn in_memory_db() -> VantaEmbedded {
        connect(":memory:").expect("in-memory db")
    }

    fn sample_input(namespace: &str, key: &str) -> VantaMemoryInput {
        VantaMemoryInput {
            namespace: namespace.into(),
            key: key.into(),
            payload: format!("payload for {key}"),
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        }
    }

    fn sample_input_with_meta(namespace: &str, key: &str, color: &str) -> VantaMemoryInput {
        let mut metadata = VantaMemoryMetadata::new();
        metadata.insert("color".into(), VantaValue::String(color.into()));
        VantaMemoryInput {
            namespace: namespace.into(),
            key: key.into(),
            payload: format!("payload for {key}"),
            metadata,
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
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
        let report = db.export_namespace(&path, "myns").unwrap();

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
        let report = db.export_namespace(&path, "nonexistent").unwrap();
        assert_eq!(report.records_exported, 0);
    }

    #[test]
    fn test_export_namespace_invalid_name() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.jsonl");
        let err = db.export_namespace(&path, "").unwrap_err();
        assert!(err.to_string().contains("namespace must not be empty"));
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
        let record = VantaMemoryRecord {
            namespace: "imp".into(),
            key: "k1".into(),
            payload: "imported".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 100,
            updated_at_ms: 100,
            version: 1,
            node_id: crate::sdk::serialization::memory_node_id("imp", "k1"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
        };
        let report = db.import_records(vec![record]).unwrap();
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
        let record = VantaMemoryRecord {
            namespace: "upd".into(),
            key: "k1".into(),
            payload: "updated".into(),
            metadata: {
                let mut m = VantaMemoryMetadata::new();
                m.insert("new".into(), VantaValue::String("field".into()));
                m
            },
            created_at_ms: 100,
            updated_at_ms: 200,
            version: 2,
            node_id: crate::sdk::serialization::memory_node_id("upd", "k1"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
        };
        let report = db.import_records(vec![record]).unwrap();
        assert_eq!(report.updated, 1);

        let retrieved = db.get("upd", "k1").unwrap().unwrap();
        assert_eq!(retrieved.payload, "updated");
    }

    #[test]
    fn test_import_records_rejects_wrong_node_id() {
        let db = in_memory_db();
        let record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "bad".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 1,
            node_id: 999, // wrong — doesn't match hash of "ns"/"k"
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
        };
        let report = db.import_records(vec![record]).unwrap();
        assert_eq!(report.errors, 1);
    }

    // ─── import_file ───────────────────────────────────────────

    #[test]
    fn test_import_file_happy_path() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("import.jsonl");

        let record = VantaMemoryRecord {
            namespace: "file".into(),
            key: "k1".into(),
            payload: "from file".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 10,
            updated_at_ms: 10,
            version: 1,
            node_id: crate::sdk::serialization::memory_node_id("file", "k1"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
        };
        let line = super::super::export_line_from_record(record);
        let json = serde_json::to_string(&line).unwrap();
        std::fs::write(&path, json + "\n").unwrap();

        let report = db.import_file(&path).unwrap();
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
        let record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "p".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 1,
            updated_at_ms: 1,
            version: 1,
            node_id: crate::sdk::serialization::memory_node_id("ns", "k"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
        };
        let line = super::super::export_line_from_record(record);
        content.push_str(&serde_json::to_string(&line).unwrap());
        content.push('\n');
        content.push('\n'); // empty line
        content.push('\n'); // another empty line
        std::fs::write(&path, content).unwrap();

        let report = db.import_file(&path).unwrap();
        assert_eq!(report.inserted, 1);
        assert_eq!(report.skipped, 2);
    }

    #[test]
    fn test_import_file_handles_malformed_lines() {
        let db = in_memory_db();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.jsonl");
        std::fs::write(&path, "not json\n{\"bad\": true}\n").unwrap();

        let report = db.import_file(&path).unwrap();
        assert_eq!(report.errors, 2);
    }

    #[test]
    fn test_import_file_read_only_rejected() {
        let config = VantaConfig {
            storage_path: ":memory:".to_string(),
            backend_kind: BackendKind::InMemory,
            read_only: true,
            ..Default::default()
        };
        let db = VantaEmbedded::open_with_config(config).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.jsonl");
        std::fs::write(&path, "").unwrap();
        let err = db.import_file(&path).unwrap_err();
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

        let mut filters = VantaMemoryMetadata::new();
        filters.insert("color".into(), VantaValue::String("red".into()));
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
            .records_for_namespace("ns", &VantaMemoryMetadata::new())
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
        let report = db1.export_namespace(&path, "rt").unwrap();
        assert_eq!(report.records_exported, 2);

        let db2 = in_memory_db();
        let import_report = db2.import_file(&path).unwrap();
        assert_eq!(import_report.inserted, 2);

        // Verify content
        let r1 = db2.get("rt", "k1").unwrap().unwrap();
        assert_eq!(r1.payload, "payload for k1");
        let r2 = db2.get("rt", "k2").unwrap().unwrap();
        assert_eq!(r2.payload, "payload for k2");
    }
}
