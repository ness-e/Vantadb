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
use std::path::Path;
use tracing;
use web_time::Instant;

impl VantaEmbedded {
    pub(crate) fn indexed_ids_by_namespace(
        &self,
        engine: &crate::storage::StorageEngine,
        namespace: &str,
    ) -> Result<(Vec<u128>, bool)> {
        let prefix = namespace_index_prefix(namespace);
        let entries = engine.scan_partition_prefix(BackendPartition::NamespaceIndex, &prefix)?;
        let mut ids = Vec::new();
        let has_index_entries =
            super::super::VantaEmbedded::load_derived_index_state(engine)?.is_some();
        crate::metrics::record_derived_prefix_scan();

        for (_key, value) in entries {
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
        let entries = engine.scan_partition_prefix(BackendPartition::PayloadIndex, &prefix)?;
        let mut ids = Vec::new();
        let has_index_entries =
            super::super::VantaEmbedded::load_derived_index_state(engine)?.is_some();
        crate::metrics::record_derived_prefix_scan();

        for (_key, value) in entries {
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
            if let Some(record) = memory_record_from_node(node) {
                if record.namespace == namespace && matches_memory_filters(&record, filters) {
                    records.push(record);
                }
            }
        }

        if records.is_empty() && !has_index_entries {
            crate::metrics::record_derived_full_scan_fallback();
            for node in engine.scan_nodes()? {
                if let Some(record) = memory_record_from_node(node) {
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
        validate_namespace(namespace)?;
        crate::storage::ops::prevent_path_traversal(&path.as_ref().to_string_lossy())?;
        let started = Instant::now();
        let records = self
            .records_for_namespace(namespace, &super::super::types::VantaMemoryMetadata::new())?;
        self.write_export_file(path.as_ref(), records, vec![namespace.to_string()], started)
    }

    #[tracing::instrument(skip(self, path), err)]
    pub fn export_all(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<super::super::types::VantaExportReport> {
        crate::storage::ops::prevent_path_traversal(&path.as_ref().to_string_lossy())?;
        let started = Instant::now();
        let namespaces = self.list_namespaces()?;
        let mut records = Vec::new();
        for namespace in &namespaces {
            records.extend(self.records_for_namespace(
                namespace,
                &super::super::types::VantaMemoryMetadata::new(),
            )?);
        }
        self.write_export_file(path.as_ref(), records, namespaces, started)
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
            serde_json::to_writer(&mut writer, &line)
                .map_err(|err| VantaError::serialization(err))?;
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
        crate::storage::ops::prevent_path_traversal(&path.as_ref().to_string_lossy())?;
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "import_file is not available when VantaDB is opened read-only".into(),
            });
        }
        let started = Instant::now();
        let file = File::open(path.as_ref()).map_err(VantaError::IoError)?;
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
                .map_err(|err| VantaError::serialization(err))
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
