use std::path::{Path, PathBuf};

use crate::binary_header::VantaHeader;
use crate::error::Result;
use crate::index::core::VECTOR_INDEX_VERSION;
use crate::storage::vfile::VFILE_VERSION;
use crate::wal::{WalHeader, WAL_POSTCARD_VERSION};

/// Physical format kinds that can be migrated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKind {
    /// VantaFile vector store format.
    VantaFile,
    /// HNSW vector index format.
    VectorIndex,
    /// Write-ahead log format.
    Wal,
    /// Storage schema format.
    Schema,
}

impl FormatKind {
    /// Return all format kinds.
    pub fn all() -> &'static [FormatKind] {
        &[
            FormatKind::VantaFile,
            FormatKind::VectorIndex,
            FormatKind::Wal,
            FormatKind::Schema,
        ]
    }

    /// Return the human-readable name of this format kind.
    pub fn name(&self) -> &'static str {
        match self {
            FormatKind::VantaFile => "vfile",
            FormatKind::VectorIndex => "index",
            FormatKind::Wal => "wal",
            FormatKind::Schema => "schema",
        }
    }

    /// Parse a format kind from a string (case-insensitive).
    pub fn from_string(s: &str) -> Option<FormatKind> {
        match s.to_lowercase().as_str() {
            "vfile" | "vantafile" => Some(FormatKind::VantaFile),
            "index" | "vectorindex" => Some(FormatKind::VectorIndex),
            "wal" => Some(FormatKind::Wal),
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

    /// Read a binary header from a file path, checking the magic bytes.
    fn read_header(&self, path: &Path, expected_magic: [u8; 4]) -> Result<Option<VantaHeader>> {
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(path)?;
        if bytes.len() < VantaHeader::SIZE {
            return Ok(None);
        }
        let header = VantaHeader::deserialize(&bytes[..VantaHeader::SIZE])?;
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
                    format: FormatKind::VantaFile,
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

        Ok(plans)
    }

    /// Migrate a single format kind to the latest version.
    pub fn migrate_format(&self, kind: FormatKind) -> Result<()> {
        match kind {
            FormatKind::VantaFile => self.migrate_vfile_to_latest(),
            FormatKind::VectorIndex => self.migrate_vector_index(),
            FormatKind::Wal => self.migrate_wal(),
            FormatKind::Schema => self.migrate_schema(),
        }
    }

    fn migrate_vfile_to_latest(&self) -> Result<()> {
        let vfile_path = self.db_path.join("vector_store.vanta");
        if !vfile_path.exists() {
            println!("  - No VantaFile found, skipping");
            return Ok(());
        }

        let data = std::fs::read(&vfile_path)?;
        let header = VantaHeader::deserialize(&data)?;

        if header.format_version >= VFILE_VERSION {
            println!(
                "  - VantaFile already at version {} (latest: {})",
                header.format_version, VFILE_VERSION
            );
            return Ok(());
        }

        if self.dry_run {
            println!(
                "  [dry-run] VantaFile v{} → v{}: would rewrite header",
                header.format_version, VFILE_VERSION
            );
            return Ok(());
        }

        let backup_path = vfile_path.with_extension("vanta.bak");
        std::fs::copy(&vfile_path, &backup_path)?;

        let new_header = VantaHeader::new(*b"VFLE", VFILE_VERSION, header.schema_version);
        let mut new_data = new_header.serialize().to_vec();
        new_data.extend_from_slice(&data[VantaHeader::SIZE..]);

        std::fs::write(&vfile_path, &new_data)?;

        println!(
            "  ✓ VantaFile migrated: v{} → v{}",
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
        if data.len() < VantaHeader::SIZE {
            println!("  - WAL file too small, skipping");
            return Ok(());
        }

        let header = VantaHeader::deserialize(&data[..VantaHeader::SIZE])?;
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
            VantaHeader::SIZE
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
                "  [dry-run] Vector index v{} → v{}: would rebuild index from VantaFile",
                header.format_version, VECTOR_INDEX_VERSION
            );
            return Ok(());
        }

        let path_str = self.db_path.to_string_lossy();
        let config = crate::config::VantaConfig {
            read_only: false,
            ..Default::default()
        };

        let engine = crate::storage::StorageEngine::open_with_config(
            path_str.as_ref(),
            Some(config),
        )?;

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
                    "VantaFile at v{}, latest is v{}",
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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::binary_header::VantaHeader;
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
        assert_eq!(FormatKind::VantaFile.name(), "vfile");
        assert_eq!(FormatKind::VectorIndex.name(), "index");
        assert_eq!(FormatKind::Wal.name(), "wal");
        assert_eq!(FormatKind::Schema.name(), "schema");
    }

    #[test]
    fn test_all_formats() {
        let all = FormatKind::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!(
            FormatKind::from_string("vfile"),
            Some(FormatKind::VantaFile)
        );
        assert_eq!(
            FormatKind::from_string("vantafile"),
            Some(FormatKind::VantaFile)
        );
        assert_eq!(
            FormatKind::from_string("index"),
            Some(FormatKind::VectorIndex)
        );
        assert_eq!(FormatKind::from_string("wal"), Some(FormatKind::Wal));
        assert_eq!(FormatKind::from_string("schema"), Some(FormatKind::Schema));
        assert_eq!(FormatKind::from_string("all"), None);
        assert_eq!(FormatKind::from_string("unknown"), None);
    }

    #[test]
    fn test_format_from_str_case_insensitive() {
        assert_eq!(
            FormatKind::from_string("VFILE"),
            Some(FormatKind::VantaFile)
        );
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
        let header = VantaHeader::new(*b"VFLE", 1, 0);
        std::fs::write(&vfile_path, header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        let vfile_plan = plans.iter().find(|p| p.format == FormatKind::VantaFile);
        assert!(
            vfile_plan.is_some(),
            "should have a VantaFile migration plan"
        );
        let p = vfile_plan.unwrap();
        assert_eq!(p.current_version, 1);
        assert_eq!(p.target_version, VFILE_VERSION);
        Ok(())
    }

    #[test]
    fn test_plan_skips_current_vfile() -> Result<()> {
        let dir = TempDir::new()?;
        let vfile_path = dir.path().join("vector_store.vanta");
        let header = VantaHeader::new(*b"VFLE", VFILE_VERSION, 0);
        std::fs::write(&vfile_path, header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        let vfile_plan = plans.iter().find(|p| p.format == FormatKind::VantaFile);
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
        let header = VantaHeader::new(*b"VNDX", old_version, 0);
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
        let old_base = VantaHeader::new(*b"VWAL", 0, WAL_POSTCARD_VERSION);
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
        engine.migrate_format(FormatKind::VantaFile)?;
        Ok(())
    }

    #[test]
    fn test_migrate_vfile_to_latest() -> Result<()> {
        let dir = TempDir::new()?;
        let vfile_path = dir.path().join("vector_store.vanta");
        let payload = b"some record data here";
        let header = VantaHeader::new(*b"VFLE", 1, 0);
        let mut data = header.serialize().to_vec();
        data.extend_from_slice(payload);
        std::fs::write(&vfile_path, &data)?;

        let engine = MigrationEngine::new(dir.path());
        engine.migrate_format(FormatKind::VantaFile)?;

        let migrated = std::fs::read(&vfile_path)?;
        let new_header = VantaHeader::deserialize(&migrated)?;
        assert_eq!(new_header.format_version, VFILE_VERSION);
        assert_eq!(new_header.magic, *b"VFLE");
        assert_eq!(&migrated[VantaHeader::SIZE..], payload);

        assert!(vfile_path.with_extension("vanta.bak").exists());
        Ok(())
    }

    #[test]
    fn test_migrate_wal_updates_header() -> Result<()> {
        let dir = TempDir::new()?;
        let wal_path = dir.path().join("wal.log");

        let old_base = VantaHeader::new(*b"VWAL", 0, WAL_POSTCARD_VERSION);
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

        let old_base = VantaHeader::new(*b"VWAL", 0, WAL_POSTCARD_VERSION);
        let base_bytes = old_base.serialize();
        let crc = crc32c::crc32c(&base_bytes);
        let mut data = base_bytes.to_vec();
        data.extend_from_slice(&crc.to_le_bytes());
        std::fs::write(&wal_path, &data)?;

        let mut engine = MigrationEngine::new(dir.path());
        engine.set_dry_run(true);
        engine.migrate_format(FormatKind::Wal)?;

        let not_migrated = std::fs::read(&wal_path)?;
        let not_migrated_header =
            VantaHeader::deserialize(&not_migrated[..VantaHeader::SIZE])?;
        assert_eq!(not_migrated_header.format_version, 0);
        assert!(!wal_path.with_extension("wal.bak").exists());
        Ok(())
    }

    #[test]
    fn test_migrate_index_dry_run_with_plan() -> Result<()> {
        let dir = TempDir::new()?;
        let index_path = dir.path().join("index.bin");
        let header = VantaHeader::new(*b"VNDX", 1, 0);
        std::fs::write(&index_path, header.serialize())?;

        let mut engine = MigrationEngine::new(dir.path());
        engine.set_dry_run(true);
        engine.migrate_format(FormatKind::VectorIndex)?;

        let same = std::fs::read(&index_path)?;
        let same_header = VantaHeader::deserialize(&same)?;
        assert_eq!(same_header.format_version, 1);
        Ok(())
    }

    #[test]
    fn test_check_integrity_reports_old_version() -> Result<()> {
        let dir = TempDir::new()?;
        let vfile_path = dir.path().join("vector_store.vanta");
        let header = VantaHeader::new(*b"VFLE", 1, 0);
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
        let header = VantaHeader::new(*b"VFLE", VFILE_VERSION, 0);
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
        let old_base = VantaHeader::new(*b"VWAL", 0, WAL_POSTCARD_VERSION);
        let base_bytes = old_base.serialize();
        let crc = crc32c::crc32c(&base_bytes);
        let mut data = base_bytes.to_vec();
        data.extend_from_slice(&crc.to_le_bytes());
        std::fs::write(&wal_path, &data)?;

        let index_path = dir.path().join("index.bin");
        let idx_header = VantaHeader::new(*b"VNDX", 1, 0);
        std::fs::write(&index_path, idx_header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert_eq!(issues.len(), 2);
        Ok(())
    }
}
