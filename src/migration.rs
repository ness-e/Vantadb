use std::path::{Path, PathBuf};

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::binary_header::Header;
use crate::error::Result;
use crate::index::graph::VECTOR_INDEX_VERSION;
use crate::node::{FieldValue, UnifiedNode};
use crate::sdk::serialization::{
    FIELD_CONFIDENCE_CLASS, FIELD_CREATED_AT_MS, FIELD_INVALID_AT_MS, FIELD_KEY, FIELD_NAMESPACE,
    FIELD_PAYLOAD, FIELD_SUPERSEDED_AT_MS, FIELD_SUPERSEDED_BY, FIELD_VALID_AT_MS,
};
use crate::sdk::types::{default_confidence, ConfidenceClass};
use crate::storage::engine::{BatchInsertOptions, InsertMode};
use crate::storage::vfile::VFILE_VERSION;
use crate::wal::{WalHeader, WAL_POSTCARD_VERSION};
use web_time::Instant;

/// Physical format kinds that can be migrated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKind {
    /// File vector store format.
    File,
    /// HNSW vector index format.
    VectorIndex,
    /// Write-ahead log format.
    Wal,
    /// v2 memory-record backfill + snapshot mirror re-encode (ADR-046).
    Records,
    /// Storage schema format.
    Schema,
}

impl FormatKind {
    /// Return all format kinds.
    pub fn all() -> &'static [FormatKind] {
        &[
            FormatKind::File,
            FormatKind::VectorIndex,
            FormatKind::Wal,
            // ADR-046 §Migration: backfill runs BEFORE the schema bump
            // (expand → backfill → bump); `all()` keeps that order.
            FormatKind::Records,
            FormatKind::Schema,
        ]
    }

    /// Return the human-readable name of this format kind.
    pub fn name(&self) -> &'static str {
        match self {
            FormatKind::File => "vfile",
            FormatKind::VectorIndex => "index",
            FormatKind::Wal => "wal",
            FormatKind::Records => "records",
            FormatKind::Schema => "schema",
        }
    }

    /// Parse a format kind from a string (case-insensitive).
    pub fn from_string(s: &str) -> Option<FormatKind> {
        match s.to_lowercase().as_str() {
            "vfile" | "vantafile" => Some(FormatKind::File),
            "index" | "vectorindex" => Some(FormatKind::VectorIndex),
            "wal" => Some(FormatKind::Wal),
            "records" => Some(FormatKind::Records),
            "schema" => Some(FormatKind::Schema),
            "all" => None,
            _ => None,
        }
    }
}

/// Describes a planned migration.
#[derive(Debug, Clone)]
pub struct MigrationPlan {
    /// Format kind to migrate.
    pub format: FormatKind,
    /// Current format version.
    pub current_version: u16,
    /// Target format version.
    pub target_version: u16,
    /// Human-readable migration action description.
    pub action: String,
}

/// Report returned by the v1 → v2 record/snapshot backfill (ADR-046).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordsBackfillReport {
    /// Memory-record nodes scanned (structural match, TTL-agnostic).
    pub scanned: u64,
    /// Nodes that received (or would receive, in dry-run) the v2 fields.
    pub backfilled: u64,
    /// Nodes that already carried the v2 marker (idempotent skip).
    pub already_v2: u64,
    /// Snapshot entries that needed (or would need) a V2 re-encode.
    pub snapshots_migrated: u64,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

/// Nodes per batched write during the records backfill.
const BACKFILL_BATCH: usize = 1000;

/// Database format migration engine.
pub struct MigrationEngine {
    /// Path to the database directory.
    db_path: PathBuf,
    /// If true, no files are modified.
    dry_run: bool,
}

impl MigrationEngine {
    /// Create a new migration engine for the given database path.
    pub fn new(db_path: impl Into<PathBuf>) -> Self {
        Self {
            db_path: db_path.into(),
            dry_run: false,
        }
    }

    /// Set the dry-run flag (no files are modified when true).
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Returns `true` if dry-run mode is active.
    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    /// Returns the database path.
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// True when the v2 records backfill is pending: a `.vanta.schema` file
    /// exists and its version is below [`crate::schema::CURRENT_SCHEMA_VERSION`]
    /// (ADR-046: the header bump is the "migration complete" marker, so a
    /// pending/partial backfill is exactly `version < current`).
    pub fn records_backfill_pending(&self) -> Result<bool> {
        let schema_path = self.db_path.join(".vanta.schema");
        match crate::schema::StorageHeader::read_from(&schema_path)? {
            Some(header) => Ok(header.version < crate::schema::CURRENT_SCHEMA_VERSION),
            None => Ok(false),
        }
    }

    /// Deterministic v1 → v2 backfill (ADR-046 §Migration):
    ///
    /// - **memory-record nodes**: `valid_at := created_at`,
    ///   `invalid_at := superseded_at` (when superseded), class `Asserted`,
    ///   `confidence_score := D_a` (1.0). Pure function of the stored values —
    ///   no clock, no randomness; nodes already carrying the v2 marker
    ///   (`FIELD_VALID_AT_MS`) are skipped, so re-runs are idempotent and
    ///   order-independent.
    /// - **snapshot mirror**: V1 values are re-encoded as V2 in place
    ///   (byte-compare idempotency).
    ///
    /// In dry-run mode nothing is written; the report carries the counts.
    pub fn migrate_records(&self) -> Result<RecordsBackfillReport> {
        let started = Instant::now();
        let mut report = RecordsBackfillReport::default();
        if !self.records_backfill_pending()? {
            report.duration_ms = started.elapsed().as_millis() as u64;
            return Ok(report);
        }

        let path_str = self.db_path.to_string_lossy();
        let config = crate::config::Config {
            read_only: self.dry_run,
            ..Default::default()
        };
        let engine =
            crate::storage::StorageEngine::open_with_config(path_str.as_ref(), Some(config))?;

        // ── Node backfill (pure per-record function of stored values) ──
        // ponytail: one-shot offline migration — every record node is
        // materialized in RAM (peak ≈ sum of record nodes) before the batched
        // write. Ceiling accepted for 0.8.0; if datasets ever exceed memory,
        // stream page-by-page (scan_nodes_page) instead of collecting.
        let mut to_backfill: Vec<UnifiedNode> = Vec::new();
        for mut node in engine.scan_nodes()? {
            if !is_memory_record_node(&node) {
                continue;
            }
            report.scanned += 1;
            if node.get_field(FIELD_VALID_AT_MS).is_some() {
                report.already_v2 += 1;
                continue;
            }
            if backfill_record_node(&mut node) {
                to_backfill.push(node);
            }
        }
        report.backfilled = to_backfill.len() as u64;

        // ── Snapshot mirror re-encode (V1 → V2, in place) ──
        let snapshots = engine.scan_partition_prefix(BackendPartition::Versions, &[])?;
        let mut snapshot_ops: Vec<BackendWriteOp> = Vec::new();
        for (key, bytes) in &snapshots {
            let canonical = crate::sdk::version_history::reencode_snapshot_v2(bytes)?;
            if canonical != *bytes {
                report.snapshots_migrated += 1;
                if !self.dry_run {
                    snapshot_ops.push(BackendWriteOp::Put {
                        partition: BackendPartition::Versions,
                        key: key.clone(),
                        value: canonical,
                    });
                }
            }
        }

        if !self.dry_run {
            for chunk in to_backfill.chunks(BACKFILL_BATCH) {
                let opts = BatchInsertOptions {
                    skip_existing_check: false,
                    skip_wal: false,
                    insert_mode: InsertMode::Auto,
                    ..Default::default()
                };
                let needs_rebuild = opts.needs_rebuild(chunk.len());
                engine.batch_insert_with_opts(chunk, opts)?;
                if needs_rebuild {
                    engine.rebuild_vector_index()?;
                }
            }
            for chunk in snapshot_ops.chunks(BACKFILL_BATCH) {
                engine.write_backend_batch(chunk.to_vec())?;
            }
        }

        report.duration_ms = started.elapsed().as_millis() as u64;
        Ok(report)
    }

    /// Read a binary header from a file path, checking the magic bytes.
    fn read_header(&self, path: &Path, expected_magic: [u8; 4]) -> Result<Option<Header>> {
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(path)?;
        if bytes.len() < Header::SIZE {
            return Ok(None);
        }
        let header = Header::deserialize(&bytes[..Header::SIZE])?;
        if header.magic == expected_magic {
            Ok(Some(header))
        } else {
            Ok(None)
        }
    }

    /// Plan all migrations needed to bring formats up to date.
    pub fn plan_all(&self) -> Result<Vec<MigrationPlan>> {
        let mut plans = Vec::new();

        let vfile_path = self.db_path.join("vector_store.vanta");
        if let Some(header) = self.read_header(&vfile_path, *b"VFLE")? {
            if header.format_version < VFILE_VERSION {
                plans.push(MigrationPlan {
                    format: FormatKind::File,
                    current_version: header.format_version,
                    target_version: VFILE_VERSION,
                    action: format!(
                        "Bump format version header v{} → v{}",
                        header.format_version, VFILE_VERSION
                    ),
                });
            }
        }

        if let Some(header) = self.read_header(&self.db_path.join("index.bin"), *b"VNDX")? {
            if header.format_version != VECTOR_INDEX_VERSION {
                plans.push(MigrationPlan {
                    format: FormatKind::VectorIndex,
                    current_version: header.format_version,
                    target_version: VECTOR_INDEX_VERSION,
                    action: format!(
                        "Index version {} differs from current ({}). Rebuild recommended.",
                        header.format_version, VECTOR_INDEX_VERSION
                    ),
                });
            }
        }

        if let Some(header) = self.read_header(&self.db_path.join("wal.log"), *b"VWAL")? {
            if header.format_version < WAL_POSTCARD_VERSION {
                plans.push(MigrationPlan {
                    format: FormatKind::Wal,
                    current_version: header.format_version,
                    target_version: WAL_POSTCARD_VERSION,
                    action: format!(
                        "WAL format version v{} → v{}",
                        header.format_version, WAL_POSTCARD_VERSION
                    ),
                });
            }
        }

        // ADR-046: the records backfill is pending while the header sits below
        // the current schema version (the bump is the migration-complete
        // marker).
        let schema_path = self.db_path.join(".vanta.schema");
        if let Some(header) = crate::schema::StorageHeader::read_from(&schema_path)? {
            if header.version < crate::schema::CURRENT_SCHEMA_VERSION {
                plans.push(MigrationPlan {
                    format: FormatKind::Records,
                    current_version: header.version as u16,
                    target_version: crate::schema::CURRENT_SCHEMA_VERSION as u16,
                    action: "Backfill memory records v1 → v2 (valid_at := created_at, \
                             invalid_at := superseded_at, confidence := D_a)"
                        .to_string(),
                });
            }
        }

        Ok(plans)
    }

    /// Migrate a single format kind to the latest version.
    pub fn migrate_format(&self, kind: FormatKind) -> Result<()> {
        match kind {
            FormatKind::File => self.migrate_vfile_to_latest(),
            FormatKind::VectorIndex => self.migrate_vector_index(),
            FormatKind::Wal => self.migrate_wal(),
            FormatKind::Records => self.migrate_records().map(|_| ()),
            FormatKind::Schema => self.migrate_schema(),
        }
    }

    fn migrate_vfile_to_latest(&self) -> Result<()> {
        let vfile_path = self.db_path.join("vector_store.vanta");
        if !vfile_path.exists() {
            println!("  - No File found, skipping");
            return Ok(());
        }

        let data = std::fs::read(&vfile_path)?;
        let header = Header::deserialize(&data)?;

        if header.format_version >= VFILE_VERSION {
            println!(
                "  - File already at version {} (latest: {})",
                header.format_version, VFILE_VERSION
            );
            return Ok(());
        }

        if self.dry_run {
            println!(
                "  [dry-run] File v{} → v{}: would rewrite header",
                header.format_version, VFILE_VERSION
            );
            return Ok(());
        }

        let backup_path = vfile_path.with_extension("vanta.bak");
        std::fs::copy(&vfile_path, &backup_path)?;

        let new_header = Header::new(*b"VFLE", VFILE_VERSION, header.schema_version);
        let mut new_data = new_header.serialize().to_vec();
        new_data.extend_from_slice(&data[Header::SIZE..]);

        std::fs::write(&vfile_path, &new_data)?;

        println!(
            "  ✓ File migrated: v{} → v{}",
            header.format_version, VFILE_VERSION
        );
        println!("  - Backup saved at: {}", backup_path.display());

        Ok(())
    }

    /// Migrate the WAL file header to the latest format version.
    fn migrate_wal(&self) -> Result<()> {
        let wal_path = self.db_path.join("wal.log");
        if !wal_path.exists() {
            println!("  - No WAL file found, skipping");
            return Ok(());
        }

        let data = std::fs::read(&wal_path)?;
        if data.len() < Header::SIZE {
            println!("  - WAL file too small, skipping");
            return Ok(());
        }

        let header = Header::deserialize(&data[..Header::SIZE])?;
        if header.magic != *b"VWAL" {
            println!("  - WAL file has invalid magic, skipping");
            return Ok(());
        }

        if header.format_version >= WAL_POSTCARD_VERSION {
            println!(
                "  - WAL already at version {} (latest: {})",
                header.format_version, WAL_POSTCARD_VERSION
            );
            return Ok(());
        }

        if self.dry_run {
            println!(
                "  [dry-run] WAL v{} → v{}: would rewrite header",
                header.format_version, WAL_POSTCARD_VERSION
            );
            return Ok(());
        }

        let backup_path = wal_path.with_extension("wal.bak");
        std::fs::copy(&wal_path, &backup_path)?;

        let new_wal_header = WalHeader::new(WAL_POSTCARD_VERSION as u32);
        let mut new_data = new_wal_header.serialize().to_vec();
        let old_header_end = if data.len() >= WalHeader::SIZE {
            WalHeader::SIZE
        } else {
            Header::SIZE
        };
        if data.len() > old_header_end {
            new_data.extend_from_slice(&data[old_header_end..]);
        }

        std::fs::write(&wal_path, &new_data)?;

        println!(
            "  ✓ WAL migrated: v{} → v{}",
            header.format_version, WAL_POSTCARD_VERSION
        );
        println!("  - Backup saved at: {}", backup_path.display());

        Ok(())
    }

    /// Rebuild the HNSW vector index when its version differs from current.
    fn migrate_vector_index(&self) -> Result<()> {
        let index_path = self.db_path.join("index.bin");
        let header = match self.read_header(&index_path, *b"VNDX")? {
            Some(h) => h,
            None => {
                println!("  - No index file found, skipping");
                return Ok(());
            }
        };

        if header.format_version == VECTOR_INDEX_VERSION {
            println!(
                "  - Vector index already at version {} (latest: {})",
                header.format_version, VECTOR_INDEX_VERSION
            );
            return Ok(());
        }

        if self.dry_run {
            println!(
                "  [dry-run] Vector index v{} → v{}: would rebuild index from File",
                header.format_version, VECTOR_INDEX_VERSION
            );
            return Ok(());
        }

        let path_str = self.db_path.to_string_lossy();
        let config = crate::config::Config {
            read_only: false,
            ..Default::default()
        };

        let engine =
            crate::storage::StorageEngine::open_with_config(path_str.as_ref(), Some(config))?;

        let report = engine.rebuild_vector_index()?;
        drop(engine);

        println!(
            "  ✓ Vector index rebuilt: v{} → v{} ({} nodes, {} vectors, {} ms)",
            header.format_version,
            VECTOR_INDEX_VERSION,
            report.scanned_nodes,
            report.indexed_vectors,
            report.duration_ms,
        );

        Ok(())
    }

    /// Print schema version info (actual migration handled by CLI).
    fn migrate_schema(&self) -> Result<()> {
        let schema_path = self.db_path.join(".vanta.schema");
        if !schema_path.exists() {
            println!("  - No schema file found, skipping");
            return Ok(());
        }

        match crate::schema::StorageHeader::read_from(&schema_path)? {
            Some(header) => {
                println!(
                    "  ✓ Schema: v{} (latest: v{}) — no migration needed",
                    header.version,
                    crate::schema::CURRENT_SCHEMA_VERSION,
                );
            }
            None => {
                println!("  - Schema file is empty or invalid, skipping");
            }
        }

        Ok(())
    }

    /// Check storage integrity and return a list of issues found.
    pub fn check_integrity(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        if let Ok(Some(header)) =
            self.read_header(&self.db_path.join("vector_store.vanta"), *b"VFLE")
        {
            if header.format_version != VFILE_VERSION {
                issues.push(format!(
                    "File at v{}, latest is v{}",
                    header.format_version, VFILE_VERSION
                ));
            }
        }

        if let Ok(Some(header)) = self.read_header(&self.db_path.join("wal.log"), *b"VWAL") {
            if header.format_version != WAL_POSTCARD_VERSION {
                issues.push(format!(
                    "WAL at v{}, latest is v{}",
                    header.format_version, WAL_POSTCARD_VERSION
                ));
            }
        }

        if let Ok(Some(header)) = self.read_header(&self.db_path.join("index.bin"), *b"VNDX") {
            if header.format_version != VECTOR_INDEX_VERSION {
                issues.push(format!(
                    "Index at v{}, latest is v{}",
                    header.format_version, VECTOR_INDEX_VERSION
                ));
            }
        }

        Ok(issues)
    }
}

/// Structural check: is this node a memory record? (namespace + key + payload
/// reserved fields, alive). TTL-agnostic on purpose — the backfill must reach
/// expired records too, since they are only physically purged later.
fn is_memory_record_node(node: &UnifiedNode) -> bool {
    node.is_alive()
        && node.get_field(FIELD_NAMESPACE).is_some()
        && node.get_field(FIELD_KEY).is_some()
        && node.get_field(FIELD_PAYLOAD).is_some()
}

/// Pure v2 backfill of a v1 record node (ADR-046 §Migration):
/// `valid_at := created_at`; `invalid_at := superseded_at` when the record is
/// superseded; class `Asserted`; `confidence_score := D_a`. No clock reads,
/// no randomness — the output is a pure function of the stored values.
/// Returns `false` (node untouched) when there is no valid `created_at`.
fn backfill_record_node(node: &mut UnifiedNode) -> bool {
    let created_at = match node.get_field(FIELD_CREATED_AT_MS) {
        Some(FieldValue::Int(ms)) if *ms >= 0 => *ms as u64,
        _ => return false,
    };
    node.set_field(FIELD_VALID_AT_MS, FieldValue::Int(created_at as i64));
    if node.get_field(FIELD_SUPERSEDED_BY).is_some() {
        if let Some(FieldValue::Int(ms)) = node.get_field(FIELD_SUPERSEDED_AT_MS) {
            if *ms >= 0 {
                node.set_field(FIELD_INVALID_AT_MS, FieldValue::Int(*ms));
            }
        }
    }
    node.set_field(
        FIELD_CONFIDENCE_CLASS,
        FieldValue::String(ConfidenceClass::Asserted.as_wire_str().to_string()),
    );
    node.confidence_score = default_confidence();
    true
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::binary_header::Header;
    use crate::sdk::serialization::{FIELD_UPDATED_AT_MS, FIELD_VERSION};
    use tempfile::TempDir;

    #[test]
    fn test_migration_engine_creation() {
        let engine = MigrationEngine::new("/tmp/test");
        assert_eq!(engine.path(), std::path::Path::new("/tmp/test"));
    }

    #[test]
    fn test_dry_run_flag() {
        let mut engine = MigrationEngine::new("/tmp/test");
        assert!(!engine.dry_run());
        engine.set_dry_run(true);
        assert!(engine.dry_run());
    }

    #[test]
    fn test_format_kind_names() {
        assert_eq!(FormatKind::File.name(), "vfile");
        assert_eq!(FormatKind::VectorIndex.name(), "index");
        assert_eq!(FormatKind::Wal.name(), "wal");
        assert_eq!(FormatKind::Records.name(), "records");
        assert_eq!(FormatKind::Schema.name(), "schema");
    }

    #[test]
    fn test_all_formats() {
        let all = FormatKind::all();
        assert_eq!(all.len(), 5);
        // ADR-046 §Migration: backfill runs BEFORE the schema bump.
        let records = all.iter().position(|f| *f == FormatKind::Records).unwrap();
        let schema = all.iter().position(|f| *f == FormatKind::Schema).unwrap();
        assert!(records < schema, "records must precede the schema bump");
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!(FormatKind::from_string("vfile"), Some(FormatKind::File));
        assert_eq!(FormatKind::from_string("vantafile"), Some(FormatKind::File));
        assert_eq!(
            FormatKind::from_string("index"),
            Some(FormatKind::VectorIndex)
        );
        assert_eq!(FormatKind::from_string("wal"), Some(FormatKind::Wal));
        assert_eq!(
            FormatKind::from_string("records"),
            Some(FormatKind::Records)
        );
        assert_eq!(FormatKind::from_string("schema"), Some(FormatKind::Schema));
        assert_eq!(FormatKind::from_string("all"), None);
        assert_eq!(FormatKind::from_string("unknown"), None);
    }

    #[test]
    fn test_format_from_str_case_insensitive() {
        assert_eq!(FormatKind::from_string("VFILE"), Some(FormatKind::File));
        assert_eq!(
            FormatKind::from_string("Index"),
            Some(FormatKind::VectorIndex)
        );
        assert_eq!(FormatKind::from_string("WAL"), Some(FormatKind::Wal));
        assert_eq!(FormatKind::from_string("Schema"), Some(FormatKind::Schema));
        assert_eq!(FormatKind::from_string("ALL"), None);
    }

    #[test]
    fn test_plan_on_empty_dir() -> Result<()> {
        let dir = TempDir::new()?;
        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        assert!(plans.is_empty());
        Ok(())
    }

    #[test]
    fn test_plan_with_v1_vfile() -> Result<()> {
        let dir = TempDir::new()?;
        let vfile_path = dir.path().join("vector_store.vanta");
        let header = Header::new(*b"VFLE", 1, 0);
        std::fs::write(&vfile_path, header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        let vfile_plan = plans.iter().find(|p| p.format == FormatKind::File);
        assert!(vfile_plan.is_some(), "should have a File migration plan");
        let p = vfile_plan.unwrap();
        assert_eq!(p.current_version, 1);
        assert_eq!(p.target_version, VFILE_VERSION);
        Ok(())
    }

    #[test]
    fn test_plan_skips_current_vfile() -> Result<()> {
        let dir = TempDir::new()?;
        let vfile_path = dir.path().join("vector_store.vanta");
        let header = Header::new(*b"VFLE", VFILE_VERSION, 0);
        std::fs::write(&vfile_path, header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        let vfile_plan = plans.iter().find(|p| p.format == FormatKind::File);
        assert!(
            vfile_plan.is_none(),
            "v{VFILE_VERSION} file should not need migration"
        );
        Ok(())
    }

    #[test]
    fn test_plan_includes_index() -> Result<()> {
        let dir = TempDir::new()?;
        let index_path = dir.path().join("index.bin");
        let old_version = 1u16;
        let header = Header::new(*b"VNDX", old_version, 0);
        std::fs::write(&index_path, header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        let index_plan = plans.iter().find(|p| p.format == FormatKind::VectorIndex);
        assert!(
            index_plan.is_some(),
            "should have a VectorIndex migration plan"
        );
        let p = index_plan.unwrap();
        assert_eq!(p.current_version, old_version);
        assert_eq!(p.target_version, VECTOR_INDEX_VERSION);
        Ok(())
    }

    #[test]
    fn test_plan_includes_wal() -> Result<()> {
        let dir = TempDir::new()?;
        let wal_path = dir.path().join("wal.log");
        let old_base = Header::new(*b"VWAL", 0, WAL_POSTCARD_VERSION);
        let base_bytes = old_base.serialize();
        let crc = crc32c::crc32c(&base_bytes);
        let mut data = base_bytes.to_vec();
        data.extend_from_slice(&crc.to_le_bytes());
        std::fs::write(&wal_path, &data)?;

        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        let wal_plan = plans.iter().find(|p| p.format == FormatKind::Wal);
        assert!(wal_plan.is_some(), "should have a WAL migration plan");
        let p = wal_plan.unwrap();
        assert_eq!(p.current_version, 0);
        assert_eq!(p.target_version, WAL_POSTCARD_VERSION);
        Ok(())
    }

    #[test]
    fn test_migrate_vfile_nonexistent() -> Result<()> {
        let dir = TempDir::new()?;
        let engine = MigrationEngine::new(dir.path());
        engine.migrate_format(FormatKind::File)?;
        Ok(())
    }

    #[test]
    fn test_migrate_vfile_to_latest() -> Result<()> {
        let dir = TempDir::new()?;
        let vfile_path = dir.path().join("vector_store.vanta");
        let payload = b"some record data here";
        let header = Header::new(*b"VFLE", 1, 0);
        let mut data = header.serialize().to_vec();
        data.extend_from_slice(payload);
        std::fs::write(&vfile_path, &data)?;

        let engine = MigrationEngine::new(dir.path());
        engine.migrate_format(FormatKind::File)?;

        let migrated = std::fs::read(&vfile_path)?;
        let new_header = Header::deserialize(&migrated)?;
        assert_eq!(new_header.format_version, VFILE_VERSION);
        assert_eq!(new_header.magic, *b"VFLE");
        assert_eq!(&migrated[Header::SIZE..], payload);

        assert!(vfile_path.with_extension("vanta.bak").exists());
        Ok(())
    }

    #[test]
    fn test_migrate_wal_updates_header() -> Result<()> {
        let dir = TempDir::new()?;
        let wal_path = dir.path().join("wal.log");

        let old_base = Header::new(*b"VWAL", 0, WAL_POSTCARD_VERSION);
        let base_bytes = old_base.serialize();
        let crc = crc32c::crc32c(&base_bytes);
        let mut data = base_bytes.to_vec();
        data.extend_from_slice(&crc.to_le_bytes());
        std::fs::write(&wal_path, &data)?;

        let engine = MigrationEngine::new(dir.path());
        engine.migrate_format(FormatKind::Wal)?;

        let migrated = std::fs::read(&wal_path)?;
        let migrated_wal = WalHeader::deserialize(&migrated[..WalHeader::SIZE])?;
        assert_eq!(migrated_wal.base.format_version, WAL_POSTCARD_VERSION);
        assert_eq!(migrated_wal.base.magic, *b"VWAL");

        assert!(wal_path.with_extension("wal.bak").exists());
        Ok(())
    }

    #[test]
    fn test_migrate_wal_skips_current() -> Result<()> {
        let dir = TempDir::new()?;
        let wal_path = dir.path().join("wal.log");

        let header = WalHeader::new(WAL_POSTCARD_VERSION as u32);
        std::fs::write(&wal_path, header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        engine.migrate_format(FormatKind::Wal)?;

        assert!(!wal_path.with_extension("wal.bak").exists());
        Ok(())
    }

    #[test]
    fn test_migrate_wal_dry_run() -> Result<()> {
        let dir = TempDir::new()?;
        let wal_path = dir.path().join("wal.log");

        let old_base = Header::new(*b"VWAL", 0, WAL_POSTCARD_VERSION);
        let base_bytes = old_base.serialize();
        let crc = crc32c::crc32c(&base_bytes);
        let mut data = base_bytes.to_vec();
        data.extend_from_slice(&crc.to_le_bytes());
        std::fs::write(&wal_path, &data)?;

        let mut engine = MigrationEngine::new(dir.path());
        engine.set_dry_run(true);
        engine.migrate_format(FormatKind::Wal)?;

        let not_migrated = std::fs::read(&wal_path)?;
        let not_migrated_header = Header::deserialize(&not_migrated[..Header::SIZE])?;
        assert_eq!(not_migrated_header.format_version, 0);
        assert!(!wal_path.with_extension("wal.bak").exists());
        Ok(())
    }

    #[test]
    fn test_migrate_index_dry_run_with_plan() -> Result<()> {
        let dir = TempDir::new()?;
        let index_path = dir.path().join("index.bin");
        let header = Header::new(*b"VNDX", 1, 0);
        std::fs::write(&index_path, header.serialize())?;

        let mut engine = MigrationEngine::new(dir.path());
        engine.set_dry_run(true);
        engine.migrate_format(FormatKind::VectorIndex)?;

        let same = std::fs::read(&index_path)?;
        let same_header = Header::deserialize(&same)?;
        assert_eq!(same_header.format_version, 1);
        Ok(())
    }

    #[test]
    fn test_check_integrity_reports_old_version() -> Result<()> {
        let dir = TempDir::new()?;
        let vfile_path = dir.path().join("vector_store.vanta");
        let header = Header::new(*b"VFLE", 1, 0);
        std::fs::write(&vfile_path, header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert!(!issues.is_empty());
        assert!(issues[0].contains("v1"));
        Ok(())
    }

    #[test]
    fn test_check_integrity_clean_current() -> Result<()> {
        let dir = TempDir::new()?;
        let vfile_path = dir.path().join("vector_store.vanta");
        let header = Header::new(*b"VFLE", VFILE_VERSION, 0);
        std::fs::write(&vfile_path, header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert!(issues.is_empty());
        Ok(())
    }

    #[test]
    fn test_check_integrity_reports_wal_and_index() -> Result<()> {
        let dir = TempDir::new()?;

        let wal_path = dir.path().join("wal.log");
        let old_base = Header::new(*b"VWAL", 0, WAL_POSTCARD_VERSION);
        let base_bytes = old_base.serialize();
        let crc = crc32c::crc32c(&base_bytes);
        let mut data = base_bytes.to_vec();
        data.extend_from_slice(&crc.to_le_bytes());
        std::fs::write(&wal_path, &data)?;

        let index_path = dir.path().join("index.bin");
        let idx_header = Header::new(*b"VNDX", 1, 0);
        std::fs::write(&index_path, idx_header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert_eq!(issues.len(), 2);
        Ok(())
    }

    // ─── ADR-046: v1 → v2 records backfill ────────────────────────

    fn open_engine(path: &str) -> crate::storage::StorageEngine {
        let config = crate::config::Config {
            backend_kind: crate::backend::BackendKind::Fjall,
            ..Default::default()
        };
        crate::storage::StorageEngine::open_with_config(path, Some(config)).expect("open engine")
    }

    fn write_v1_schema(dir: &Path) {
        let header = crate::schema::StorageHeader {
            version: 1,
            flags: 0,
            min_compat_version: 1,
        };
        header
            .write_to(&dir.join(".vanta.schema"))
            .expect("write v1 header");
    }

    fn v1_record_node(
        id: u128,
        ns: &str,
        key: &str,
        created_at: u64,
        superseded: Option<(&str, u64)>,
    ) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.set_field(FIELD_NAMESPACE, FieldValue::String(ns.into()));
        node.set_field(FIELD_KEY, FieldValue::String(key.into()));
        node.set_field(FIELD_PAYLOAD, FieldValue::String("payload".into()));
        node.set_field(FIELD_CREATED_AT_MS, FieldValue::Int(created_at as i64));
        node.set_field(FIELD_UPDATED_AT_MS, FieldValue::Int(created_at as i64));
        node.set_field(FIELD_VERSION, FieldValue::Int(1));
        if let Some((by, at)) = superseded {
            node.set_field(FIELD_SUPERSEDED_BY, FieldValue::String(by.into()));
            node.set_field(FIELD_SUPERSEDED_AT_MS, FieldValue::Int(at as i64));
        }
        node
    }

    fn copy_dir_all(src: &Path, dst: &Path) {
        std::fs::create_dir_all(dst).expect("create dst");
        for entry in std::fs::read_dir(src).expect("read src") {
            let entry = entry.expect("entry");
            let target = dst.join(entry.file_name());
            if entry.file_type().expect("file type").is_dir() {
                copy_dir_all(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).expect("copy file");
            }
        }
    }

    #[test]
    fn test_plan_includes_records_when_schema_pending() -> Result<()> {
        let dir = TempDir::new()?;
        write_v1_schema(dir.path());
        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        let plan = plans
            .iter()
            .find(|p| p.format == FormatKind::Records)
            .expect("records plan present while header is v1");
        assert_eq!(plan.current_version, 1);
        assert_eq!(
            plan.target_version,
            crate::schema::CURRENT_SCHEMA_VERSION as u16
        );
        Ok(())
    }

    #[test]
    fn test_records_backfill_normalizes_v1_nodes_and_is_idempotent() -> Result<()> {
        let dir = TempDir::new()?;
        let db_path = dir.path().to_str().unwrap();
        write_v1_schema(dir.path());
        {
            let engine = open_engine(db_path);
            engine.insert(&v1_record_node(1, "ns", "a", 1000, Some(("b", 1500))))?;
            engine.insert(&v1_record_node(2, "ns", "b", 1100, None))?;
            engine.insert(&UnifiedNode::new(3))?; // non-record node: untouched
        }

        let engine = MigrationEngine::new(dir.path());
        assert!(engine.records_backfill_pending()?);
        let report = engine.migrate_records()?;
        assert_eq!(report.scanned, 2);
        assert_eq!(report.backfilled, 2);
        assert_eq!(report.already_v2, 0);

        let raw = open_engine(db_path);
        let node = raw.get(1)?.expect("node 1");
        assert_eq!(
            node.get_field(FIELD_VALID_AT_MS),
            Some(&FieldValue::Int(1000)),
            "valid_at := created_at"
        );
        assert_eq!(
            node.get_field(FIELD_INVALID_AT_MS),
            Some(&FieldValue::Int(1500)),
            "invalid_at := superseded_at when superseded"
        );
        assert_eq!(
            node.get_field(FIELD_CONFIDENCE_CLASS),
            Some(&FieldValue::String("Asserted".into()))
        );
        assert_eq!(node.confidence_score, 1.0, "backfill confidence := D_a");

        let node2 = raw.get(2)?.expect("node 2");
        assert_eq!(
            node2.get_field(FIELD_VALID_AT_MS),
            Some(&FieldValue::Int(1100))
        );
        assert!(
            node2.get_field(FIELD_INVALID_AT_MS).is_none(),
            "non-superseded records keep an open window"
        );
        assert!(
            raw.get(3)?
                .expect("node 3")
                .get_field(FIELD_VALID_AT_MS)
                .is_none(),
            "non-record nodes are untouched"
        );
        drop(raw);

        // Idempotent re-run: skip nodes that already carry the v2 marker.
        let report2 = engine.migrate_records()?;
        assert_eq!(report2.backfilled, 0);
        assert_eq!(report2.already_v2, 2);
        Ok(())
    }

    #[test]
    fn test_records_backfill_dry_run_makes_no_changes() -> Result<()> {
        let dir = TempDir::new()?;
        let db_path = dir.path().to_str().unwrap();
        write_v1_schema(dir.path());
        {
            let engine = open_engine(db_path);
            engine.insert(&v1_record_node(1, "ns", "a", 1000, None))?;
        }

        let mut engine = MigrationEngine::new(dir.path());
        engine.set_dry_run(true);
        let report = engine.migrate_records()?;
        assert_eq!(report.backfilled, 1, "dry-run reports what would change");

        let raw = open_engine(db_path);
        assert!(
            raw.get(1)?
                .expect("node 1")
                .get_field(FIELD_VALID_AT_MS)
                .is_none(),
            "dry-run must not write"
        );
        Ok(())
    }

    #[test]
    fn test_records_backfill_deterministic_across_copies() -> Result<()> {
        let dir = TempDir::new()?;
        let copy_dir = TempDir::new()?;
        let db_path = dir.path().to_str().unwrap();
        write_v1_schema(dir.path());
        {
            let engine = open_engine(db_path);
            engine.insert(&v1_record_node(1, "ns", "a", 1000, Some(("b", 1500))))?;
            engine.insert(&v1_record_node(2, "ns", "b", 1100, None))?;
            drop(engine);
        }
        copy_dir_all(dir.path(), copy_dir.path());

        MigrationEngine::new(dir.path()).migrate_records()?;
        MigrationEngine::new(copy_dir.path()).migrate_records()?;

        let a = open_engine(dir.path().to_str().unwrap());
        let b = open_engine(copy_dir.path().to_str().unwrap());
        for id in [1u128, 2] {
            let na = a.get(id)?.expect("node a");
            let nb = b.get(id)?.expect("node b");
            assert_eq!(
                na.get_field(FIELD_VALID_AT_MS),
                nb.get_field(FIELD_VALID_AT_MS)
            );
            assert_eq!(
                na.get_field(FIELD_INVALID_AT_MS),
                nb.get_field(FIELD_INVALID_AT_MS)
            );
            assert_eq!(
                na.get_field(FIELD_CONFIDENCE_CLASS),
                nb.get_field(FIELD_CONFIDENCE_CLASS)
            );
            assert_eq!(na.confidence_score, nb.confidence_score);
        }
        Ok(())
    }

    #[test]
    fn test_records_backfill_rewrites_v1_snapshots() -> Result<()> {
        // V1 wire bytes of the snapshot mirror (11 fields, node_id as string).
        #[derive(serde::Serialize)]
        struct V1SnapshotBytes {
            namespace: String,
            key: String,
            payload: String,
            metadata: crate::sdk::types::MemoryMetadata,
            created_at_ms: u64,
            updated_at_ms: u64,
            version: u64,
            node_id: String,
            vector: Option<Vec<f32>>,
            sparse_vector: Option<crate::node::SparseVector>,
            expires_at_ms: Option<u64>,
        }

        let dir = TempDir::new()?;
        let db_path = dir.path().to_str().unwrap();
        write_v1_schema(dir.path());
        let v1_bytes = postcard::to_allocvec(&V1SnapshotBytes {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "p".into(),
            metadata: Default::default(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 1,
            node_id: "42".into(),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
        })
        .expect("serialize v1 snapshot");
        let snapshot_key = crate::sdk::version_history::version_key("ns", "k", 1);
        {
            let engine = open_engine(db_path);
            engine.put_to_partition(
                crate::backend::BackendPartition::Versions,
                &snapshot_key,
                &v1_bytes,
            )?;
        }

        let report = MigrationEngine::new(dir.path()).migrate_records()?;
        assert_eq!(report.snapshots_migrated, 1);

        let raw = open_engine(db_path);
        let migrated = raw
            .get_from_partition(crate::backend::BackendPartition::Versions, &snapshot_key)?
            .expect("snapshot present");
        assert_ne!(migrated, v1_bytes, "v1 snapshot must be re-encoded as v2");
        assert_eq!(
            crate::sdk::version_history::reencode_snapshot_v2(&migrated)?,
            migrated,
            "re-encoded snapshot is canonical (idempotent skip on re-run)"
        );
        drop(raw);

        // Second run: no snapshot changes.
        let report2 = MigrationEngine::new(dir.path()).migrate_records()?;
        assert_eq!(report2.snapshots_migrated, 0);
        Ok(())
    }
}
