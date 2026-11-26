use std::path::{Path, PathBuf};

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::binary_header::Header;
use crate::error::Result;
use crate::index::graph::{VECTOR_INDEX_ENCRYPTED_VERSION, VECTOR_INDEX_VERSION};
use crate::node::{FieldValue, UnifiedNode};
use crate::sdk::serialization::{
    FIELD_CONFIDENCE_CLASS, FIELD_CREATED_AT_MS, FIELD_INVALID_AT_MS, FIELD_KEY, FIELD_NAMESPACE,
    FIELD_PAYLOAD, FIELD_SUPERSEDED_AT_MS, FIELD_SUPERSEDED_BY, FIELD_VALID_AT_MS,
};
use crate::sdk::types::{default_confidence, ConfidenceClass};
use crate::storage::engine::{BatchInsertOptions, InsertMode};
use crate::storage::vfile::{VFILE_ENCRYPTED_VERSION, VFILE_VERSION};
use crate::wal::{WAL_ENCRYPTED_FORMAT_VERSION, WAL_FORMAT_VERSION};
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

/// Version classification of an on-disk artefact against the formats this
/// build knows (FIND-499).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormatStatus {
    /// A current format — plaintext or encrypted (ADR-0057 D7).
    Current,
    /// An older plaintext-lineage version that can be migrated.
    NeedsMigration,
    /// A version newer than everything this build knows.
    Newer,
}

/// The version lines a migration/integrity check accepts for one artefact.
///
/// Rules (ADR-0057 D1/D7 — cited by ADR-0060 / FIND-631):
/// * the accepted **current** set is `{plain_current, encrypted_current}`;
/// * `v < plain_current` → `NeedsMigration` — only the *plaintext* lineage is
///   migratable: an encrypted artefact is never rewritten implicitly;
/// * `v > max(plain, encrypted)` → `Newer`: the file was written by a newer
///   build — an integrity *issue*, never a "corrupt" verdict and never a
///   migration target.
struct ArtifactSpec {
    /// Human-readable artefact name for messages.
    label: &'static str,
    /// Current plaintext format version.
    plain_current: u16,
    /// Current encrypted format version (`== plain_current` when the
    /// artefact has no encrypted variant).
    encrypted_current: u16,
}

impl ArtifactSpec {
    /// The highest format version this build knows for the artefact.
    fn max_known(&self) -> u16 {
        self.plain_current.max(self.encrypted_current)
    }

    /// Classify an on-disk `format_version` (see the struct doc for rules).
    fn classify(&self, version: u16) -> FormatStatus {
        if version == self.plain_current || version == self.encrypted_current {
            FormatStatus::Current
        } else if version < self.plain_current {
            FormatStatus::NeedsMigration
        } else {
            FormatStatus::Newer
        }
    }
}

/// VantaFile segments: plaintext v2, encrypted v3 (`VFILE_*` consts).
const VFILE_SPEC: ArtifactSpec = ArtifactSpec {
    label: "VantaFile",
    plain_current: VFILE_VERSION,
    encrypted_current: VFILE_ENCRYPTED_VERSION,
};

/// HNSW index image: plaintext v8, encrypted v9 (`VECTOR_INDEX_*` consts).
const INDEX_SPEC: ArtifactSpec = ArtifactSpec {
    label: "Vector index",
    plain_current: VECTOR_INDEX_VERSION,
    encrypted_current: VECTOR_INDEX_ENCRYPTED_VERSION,
};

/// WAL segments: plaintext v3, encrypted v4 (`WAL_*` consts). Note that
/// `WAL_POSTCARD_VERSION` is *not* the format version — it lives in the
/// header's `schema_version` field.
const WAL_SPEC: ArtifactSpec = ArtifactSpec {
    label: "WAL",
    plain_current: WAL_FORMAT_VERSION,
    encrypted_current: WAL_ENCRYPTED_FORMAT_VERSION,
};

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

    /// The `data/` directory where the engine persists its artefacts
    /// (`<db>/data`, see `src/storage/engine/init.rs`).
    fn data_dir(&self) -> PathBuf {
        self.db_path.join("data")
    }

    /// VantaFile segments present on disk: `vstore_L{0..3}.vanta` plus the
    /// legacy `vector_store.vanta` (renamed to `vstore_L0.vanta` by `lsm.rs`
    /// on open, but still present in never-reopened bases).
    fn vfile_paths(&self) -> Vec<PathBuf> {
        let data = self.data_dir();
        (0..4)
            .map(|level| data.join(format!("vstore_L{level}.vanta")))
            .chain(std::iter::once(data.join("vector_store.vanta")))
            .filter(|p| p.exists())
            .collect()
    }

    /// Active WAL segments: `vanta.wal` (single shard) or `vanta.shardN.wal`.
    ///
    /// Rotated archives (`vanta.wal.<millis>`) are excluded on purpose: they
    /// are historical segments that readers still understand (range-based
    /// compat), not instances of the "current format".
    fn wal_paths(&self) -> Vec<PathBuf> {
        let data = self.data_dir();
        let mut paths = Vec::new();
        let live = data.join("vanta.wal");
        if live.exists() {
            paths.push(live);
        }
        if let Ok(entries) = std::fs::read_dir(&data) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with("vanta.shard") && name.ends_with(".wal") {
                    paths.push(entry.path());
                }
            }
        }
        paths.sort();
        paths
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
    ///
    /// Only *plaintext* lineage below the current plaintext version is a
    /// migration target (see `ArtifactSpec::classify`): current plaintext
    /// and encrypted formats need nothing, and anything newer than this
    /// build is reported by [`Self::check_integrity`], never migrated.
    pub fn plan_all(&self) -> Result<Vec<MigrationPlan>> {
        let mut plans = Vec::new();

        for path in self.vfile_paths() {
            if let Some(header) = self.read_header(&path, *b"VFLE")? {
                if VFILE_SPEC.classify(header.format_version) == FormatStatus::NeedsMigration {
                    plans.push(MigrationPlan {
                        format: FormatKind::File,
                        current_version: header.format_version,
                        target_version: VFILE_VERSION,
                        action: format!(
                            "Bump {} format version header v{} → v{}",
                            path.file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_default(),
                            header.format_version,
                            VFILE_VERSION
                        ),
                    });
                }
            }
        }

        let index_path = self.data_dir().join("vector_index.bin");
        if let Some(header) = self.read_header(&index_path, *b"VNDX")? {
            if INDEX_SPEC.classify(header.format_version) == FormatStatus::NeedsMigration {
                plans.push(MigrationPlan {
                    format: FormatKind::VectorIndex,
                    current_version: header.format_version,
                    target_version: VECTOR_INDEX_VERSION,
                    action: format!(
                        "Index version {} predates current ({}). Rebuild recommended.",
                        header.format_version, VECTOR_INDEX_VERSION
                    ),
                });
            }
        }

        for path in self.wal_paths() {
            if let Some(header) = self.read_header(&path, *b"VWAL")? {
                if WAL_SPEC.classify(header.format_version) == FormatStatus::NeedsMigration {
                    plans.push(MigrationPlan {
                        format: FormatKind::Wal,
                        current_version: header.format_version,
                        target_version: WAL_FORMAT_VERSION,
                        action: format!(
                            "WAL {} at v{} predates the chained frame layout \
                             (v{}): archive it and let the engine replay into \
                             a fresh segment — an in-place header rewrite is unsafe",
                            path.file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_default(),
                            header.format_version,
                            WAL_FORMAT_VERSION
                        ),
                    });
                }
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
        let vfile_paths = self.vfile_paths();
        if vfile_paths.is_empty() {
            println!("  - No File found, skipping");
            return Ok(());
        }

        for vfile_path in vfile_paths {
            let data = std::fs::read(&vfile_path)?;
            let header = Header::deserialize(&data)?;
            let name = vfile_path.display().to_string();

            match VFILE_SPEC.classify(header.format_version) {
                FormatStatus::NeedsMigration => {}
                FormatStatus::Current => {
                    println!(
                        "  - {name} already at a current version ({}, latest: v{}/v{})",
                        header.format_version,
                        VFILE_SPEC.plain_current,
                        VFILE_SPEC.encrypted_current,
                    );
                    continue;
                }
                FormatStatus::Newer => {
                    println!(
                        "  - {name} at v{} — newer than this build, skipping",
                        header.format_version,
                    );
                    continue;
                }
            }

            if self.dry_run {
                println!(
                    "  [dry-run] {name} v{} → v{}: would rewrite header",
                    header.format_version, VFILE_VERSION
                );
                continue;
            }

            let backup_path = vfile_path.with_extension("vanta.bak");
            std::fs::copy(&vfile_path, &backup_path)?;

            let new_header = Header::new(*b"VFLE", VFILE_VERSION, header.schema_version);
            let mut new_data = new_header.serialize().to_vec();
            new_data.extend_from_slice(&data[Header::SIZE..]);

            std::fs::write(&vfile_path, &new_data)?;

            println!(
                "  ✓ {name} migrated: v{} → v{}",
                header.format_version, VFILE_VERSION
            );
            println!("  - Backup saved at: {}", backup_path.display());
        }

        Ok(())
    }

    /// Migrate the WAL to the latest format version.
    ///
    /// The WAL header carries the *frame layout* version: a bump across
    /// [`crate::wal::WAL_CHAIN_VERSION`] (v3) would make the reader expect
    /// chained frames that are not there — an in-place header rewrite is
    /// corrupting, not migrating. Pre-v3 WALs must be archived and replayed
    /// into a fresh segment, which is engine work, not a file rewrite; this
    /// path is therefore deliberately non-destructive. Encrypted (v4) and
    /// current (v3) segments are skipped untouched (ADR-0057 D7).
    fn migrate_wal(&self) -> Result<()> {
        let wal_paths = self.wal_paths();
        if wal_paths.is_empty() {
            println!("  - No WAL file found, skipping");
            return Ok(());
        }

        for wal_path in wal_paths {
            let name = wal_path.display().to_string();
            let header = match self.read_header(&wal_path, *b"VWAL")? {
                Some(h) => h,
                None => {
                    println!("  - {name} has no valid WAL header, skipping");
                    continue;
                }
            };

            match WAL_SPEC.classify(header.format_version) {
                FormatStatus::NeedsMigration => println!(
                    "  - {name} at v{} predates the chained frame layout \
                     (v{}). It is NOT rewritten in place: archive it and let \
                     the engine replay into a fresh segment.",
                    header.format_version, WAL_FORMAT_VERSION
                ),
                FormatStatus::Current => println!(
                    "  - {name} already at a current version ({}, latest: v{}/v{})",
                    header.format_version, WAL_SPEC.plain_current, WAL_SPEC.encrypted_current,
                ),
                FormatStatus::Newer => println!(
                    "  - {name} at v{} — newer than this build, skipping",
                    header.format_version
                ),
            }
        }

        Ok(())
    }

    /// Rebuild the HNSW vector index when its version is below the current
    /// plaintext one. A v9 (encrypted) or newer-than-known image is *not* a
    /// migration target: rewriting it with a v8 header would corrupt it
    /// (ADR-0057 D7).
    fn migrate_vector_index(&self) -> Result<()> {
        let index_path = self.data_dir().join("vector_index.bin");
        let header = match self.read_header(&index_path, *b"VNDX")? {
            Some(h) => h,
            None => {
                println!("  - No index file found, skipping");
                return Ok(());
            }
        };

        match INDEX_SPEC.classify(header.format_version) {
            FormatStatus::NeedsMigration => {}
            FormatStatus::Current => {
                println!(
                    "  - Vector index already at a current version ({}, latest: v{}/v{})",
                    header.format_version, INDEX_SPEC.plain_current, INDEX_SPEC.encrypted_current,
                );
                return Ok(());
            }
            FormatStatus::Newer => {
                println!(
                    "  - Vector index at v{} — newer than this build, skipping",
                    header.format_version
                );
                return Ok(());
            }
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
    ///
    /// Per artefact the accepted *current* set is `{plaintext, encrypted}`
    /// (`ArtifactSpec::classify`): an old plaintext version is reported as
    /// needing migration, a version beyond everything this build knows is
    /// reported as *newer* — never silently accepted, never "corrupt".
    pub fn check_integrity(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        let mut check = |spec: &ArtifactSpec, path: &Path, magic: [u8; 4]| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            match self.read_header(path, magic) {
                Ok(Some(header)) => match spec.classify(header.format_version) {
                    FormatStatus::Current => {}
                    FormatStatus::NeedsMigration => issues.push(format!(
                        "{} {} at v{}, current formats are v{} (plaintext) \
                         and v{} (encrypted)",
                        spec.label,
                        name,
                        header.format_version,
                        spec.plain_current,
                        spec.encrypted_current
                    )),
                    FormatStatus::Newer => issues.push(format!(
                        "{} {} at v{}, newer than this build (max known: v{})",
                        spec.label,
                        name,
                        header.format_version,
                        spec.max_known()
                    )),
                },
                // No header (missing/short/foreign-magic file) is not an issue.
                Ok(None) => {}
                // A present artefact that can't even be read IS an integrity issue.
                Err(e) => issues.push(format!("{} {} unreadable: {}", spec.label, name, e)),
            }
        };

        for path in self.vfile_paths() {
            check(&VFILE_SPEC, &path, *b"VFLE");
        }
        for path in self.wal_paths() {
            check(&WAL_SPEC, &path, *b"VWAL");
        }
        check(
            &INDEX_SPEC,
            &self.data_dir().join("vector_index.bin"),
            *b"VNDX",
        );

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
    use crate::index::graph::VECTOR_INDEX_ENCRYPTED_VERSION;
    use crate::sdk::serialization::{FIELD_UPDATED_AT_MS, FIELD_VERSION};
    use crate::wal::{
        WalHeader, WAL_ENCRYPTED_FORMAT_VERSION, WAL_FORMAT_VERSION, WAL_POSTCARD_VERSION,
    };
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

    // ─── Artefact layout helpers (real on-disk layout: `<db>/data/` ──

    /// The `data/` dir of a temp database, created on demand.
    fn data_dir(dir: &TempDir) -> PathBuf {
        let d = dir.path().join("data");
        std::fs::create_dir_all(&d).expect("create data dir");
        d
    }

    /// Write a bare `Header` (magic + version) as a fake artefact.
    fn write_header_file(path: &Path, magic: [u8; 4], format_version: u16) {
        let header = Header::new(magic, format_version, 0);
        std::fs::write(path, header.serialize()).expect("write header");
    }

    /// Write a WAL artefact with a `WalHeader` (header + CRC) at `path`.
    fn write_wal_file(path: &Path, format_version: u16) {
        let base = Header::new(*b"VWAL", format_version, WAL_POSTCARD_VERSION);
        let base_bytes = base.serialize();
        let crc = crc32c::crc32c(&base_bytes);
        let mut data = base_bytes.to_vec();
        data.extend_from_slice(&crc.to_le_bytes());
        std::fs::write(path, &data).expect("write wal");
    }

    /// A real on-disk backend for engine-based tests.
    #[cfg(feature = "fjall")]
    fn disk_backend() -> crate::backend::BackendKind {
        crate::backend::BackendKind::Fjall
    }
    #[cfg(all(not(feature = "fjall"), feature = "rocksdb"))]
    fn disk_backend() -> crate::backend::BackendKind {
        crate::backend::BackendKind::RocksDb
    }

    fn engine_config(path: &str, encryption_key: Option<String>) -> crate::config::Config {
        crate::config::Config {
            storage_path: path.to_string(),
            backend_kind: disk_backend(),
            encryption_key,
            ..Default::default()
        }
    }

    fn seed_node(id: u128) -> UnifiedNode {
        UnifiedNode::new(id)
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
        write_header_file(&data_dir(&dir).join("vstore_L0.vanta"), *b"VFLE", 1);

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
        write_header_file(
            &data_dir(&dir).join("vstore_L0.vanta"),
            *b"VFLE",
            VFILE_VERSION,
        );

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
        let old_version = 1u16;
        write_header_file(
            &data_dir(&dir).join("vector_index.bin"),
            *b"VNDX",
            old_version,
        );

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
        write_wal_file(&data_dir(&dir).join("vanta.wal"), 0);

        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        let wal_plan = plans.iter().find(|p| p.format == FormatKind::Wal);
        assert!(wal_plan.is_some(), "should have a WAL migration plan");
        let p = wal_plan.unwrap();
        assert_eq!(p.current_version, 0);
        assert_eq!(p.target_version, WAL_FORMAT_VERSION);
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
        let vfile_path = data_dir(&dir).join("vstore_L0.vanta");
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

    /// The WAL header carries the *frame layout* version: a bump across
    /// `WAL_CHAIN_VERSION` would make the reader expect chained frames that
    /// are not there. `migrate_wal` must therefore never rewrite an old WAL
    /// header in place — migration requires replay into a fresh segment.
    #[test]
    fn test_migrate_wal_never_rewrites_an_old_header() -> Result<()> {
        let dir = TempDir::new()?;
        let wal_path = data_dir(&dir).join("vanta.wal");
        write_wal_file(&wal_path, 1);

        let engine = MigrationEngine::new(dir.path());
        engine.migrate_format(FormatKind::Wal)?;

        let migrated = std::fs::read(&wal_path)?;
        let migrated_wal = WalHeader::deserialize(&migrated[..WalHeader::SIZE])?;
        assert_eq!(
            migrated_wal.base.format_version, 1,
            "an old WAL header must not be bumped in place"
        );
        assert!(!wal_path.with_extension("wal.bak").exists());
        Ok(())
    }

    /// An encrypted WAL (v4) is never touched by the migration engine
    /// (ADR-0057 D7: no implicit migration of encrypted artefacts).
    #[test]
    fn test_migrate_wal_never_touches_encrypted() -> Result<()> {
        let dir = TempDir::new()?;
        let wal_path = data_dir(&dir).join("vanta.wal");
        write_wal_file(&wal_path, WAL_ENCRYPTED_FORMAT_VERSION);
        let before = std::fs::read(&wal_path)?;

        let engine = MigrationEngine::new(dir.path());
        engine.migrate_format(FormatKind::Wal)?;

        assert_eq!(std::fs::read(&wal_path)?, before);
        assert!(!wal_path.with_extension("wal.bak").exists());
        Ok(())
    }

    #[test]
    fn test_migrate_wal_skips_current() -> Result<()> {
        let dir = TempDir::new()?;
        let wal_path = data_dir(&dir).join("vanta.wal");
        write_wal_file(&wal_path, WAL_FORMAT_VERSION);

        let engine = MigrationEngine::new(dir.path());
        engine.migrate_format(FormatKind::Wal)?;

        assert!(!wal_path.with_extension("wal.bak").exists());
        Ok(())
    }

    #[test]
    fn test_migrate_wal_dry_run() -> Result<()> {
        let dir = TempDir::new()?;
        let wal_path = data_dir(&dir).join("vanta.wal");
        write_wal_file(&wal_path, 0);

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
        let index_path = data_dir(&dir).join("vector_index.bin");
        write_header_file(&index_path, *b"VNDX", 1);

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
        write_header_file(&data_dir(&dir).join("vstore_L0.vanta"), *b"VFLE", 1);

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert!(!issues.is_empty());
        assert!(issues[0].contains("v1"));
        Ok(())
    }

    #[test]
    fn test_check_integrity_clean_current() -> Result<()> {
        let dir = TempDir::new()?;
        let data = data_dir(&dir);
        write_header_file(&data.join("vstore_L0.vanta"), *b"VFLE", VFILE_VERSION);
        write_wal_file(&data.join("vanta.wal"), WAL_FORMAT_VERSION);
        write_header_file(
            &data.join("vector_index.bin"),
            *b"VNDX",
            VECTOR_INDEX_VERSION,
        );

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
        Ok(())
    }

    /// FIND-499: the integrity check must actually *find* the artefacts where
    /// the engine writes them (`data/vstore_L*.vanta`, `data/vanta.wal`,
    /// `data/vector_index.bin`) — the previous code looked for
    /// `vector_store.vanta`/`index.bin`/`wal.log` at the db root, which the
    /// engine never writes: the check was dead code.
    #[test]
    fn test_integrity_finds_artifacts_under_data_dir() -> Result<()> {
        let dir = TempDir::new()?;
        let data = data_dir(&dir);
        write_header_file(&data.join("vstore_L0.vanta"), *b"VFLE", 1);
        write_wal_file(&data.join("vanta.wal"), 1);
        write_header_file(&data.join("vector_index.bin"), *b"VNDX", 1);

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert_eq!(issues.len(), 3, "all three artifacts flagged: {issues:?}");
        Ok(())
    }

    #[test]
    fn test_check_integrity_reports_wal_and_index() -> Result<()> {
        let dir = TempDir::new()?;
        let data = data_dir(&dir);
        write_wal_file(&data.join("vanta.wal"), 0);
        write_header_file(&data.join("vector_index.bin"), *b"VNDX", 1);

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert_eq!(issues.len(), 2);
        Ok(())
    }

    // ─── FIND-499: current plaintext + encrypted formats are accepted ──

    /// (a) A database created *with* a key produces encrypted artefacts
    /// (VantaFile v3, WAL v4, index v9): `check_integrity` must report zero
    /// issues — an encrypted artefact is a *current* format, never an
    /// integrity violation (ADR-0057 D7).
    #[cfg(feature = "encryption")]
    #[test]
    fn test_integrity_accepts_encrypted_v3_v4_v9() -> Result<()> {
        let dir = TempDir::new()?;
        let path = dir.path().to_str().expect("utf8 path").to_string();
        let key = "aa".repeat(32);
        {
            let engine = crate::storage::StorageEngine::open_with_config(
                &path,
                Some(engine_config(&path, Some(key))),
            )?;
            engine.insert(&seed_node(1))?;
            engine.flush()?;
        }

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert!(
            issues.is_empty(),
            "an encrypted database must report zero integrity issues: {issues:?}"
        );
        Ok(())
    }

    /// (b) A database created *without* a key: every artefact sits at its
    /// current plaintext version — including the WAL at v3, which the old
    /// check compared against `WAL_POSTCARD_VERSION` (= 1).
    #[test]
    fn test_integrity_accepts_plaintext_current_wal() -> Result<()> {
        let dir = TempDir::new()?;
        let path = dir.path().to_str().expect("utf8 path").to_string();
        {
            let engine = crate::storage::StorageEngine::open_with_config(
                &path,
                Some(engine_config(&path, None)),
            )?;
            engine.insert(&seed_node(1))?;
            engine.flush()?;
        }

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert!(
            issues.is_empty(),
            "a current plaintext database must report zero issues: {issues:?}"
        );
        Ok(())
    }

    /// (c) An encrypted index image (v9) is *current*: no migration plan and
    /// no integrity issue (the old `!= VECTOR_INDEX_VERSION` check would have
    /// scheduled a "rebuild" that rewrites the image with a v8 header —
    /// corrupting it).
    #[test]
    fn test_migration_never_targets_encrypted_index() -> Result<()> {
        let dir = TempDir::new()?;
        write_header_file(
            &data_dir(&dir).join("vector_index.bin"),
            *b"VNDX",
            VECTOR_INDEX_ENCRYPTED_VERSION,
        );

        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        assert!(
            !plans.iter().any(|p| p.format == FormatKind::VectorIndex),
            "an encrypted index must never be a migration target: {plans:?}"
        );
        let issues = engine.check_integrity()?;
        assert!(
            issues.is_empty(),
            "an encrypted index must report zero issues: {issues:?}"
        );
        Ok(())
    }

    /// (d) A header version beyond everything this build knows is reported as
    /// "newer than this build" — never silently accepted, never "corrupt".
    #[test]
    fn test_integrity_flags_future_version() -> Result<()> {
        let dir = TempDir::new()?;
        write_header_file(&data_dir(&dir).join("vstore_L0.vanta"), *b"VFLE", 99);

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert_eq!(issues.len(), 1, "one issue expected: {issues:?}");
        assert!(
            issues[0].contains("newer"),
            "future version must be flagged as newer-than-build: {}",
            issues[0]
        );
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
