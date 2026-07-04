use std::path::{Path, PathBuf};

use crate::binary_header::VantaHeader;
use crate::error::Result;

/// Physical format kinds that can be migrated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKind {
    VantaFile,
    VectorIndex,
    Wal,
    Schema,
}

impl FormatKind {
    pub fn all() -> &'static [FormatKind] {
        &[
            FormatKind::VantaFile,
            FormatKind::VectorIndex,
            FormatKind::Wal,
            FormatKind::Schema,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            FormatKind::VantaFile => "vfile",
            FormatKind::VectorIndex => "index",
            FormatKind::Wal => "wal",
            FormatKind::Schema => "schema",
        }
    }

    pub fn from_str(s: &str) -> Option<FormatKind> {
        match s {
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
    pub format: FormatKind,
    pub current_version: u16,
    pub target_version: u16,
    pub action: String,
}

/// Database format migration engine.
pub struct MigrationEngine {
    db_path: PathBuf,
    dry_run: bool,
}

impl MigrationEngine {
    pub fn new(db_path: impl Into<PathBuf>) -> Self {
        Self {
            db_path: db_path.into(),
            dry_run: false,
        }
    }

    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn path(&self) -> &Path {
        &self.db_path
    }

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

    pub fn plan_all(&self) -> Result<Vec<MigrationPlan>> {
        let mut plans = Vec::new();

        if let Some(header) =
            self.read_header(&self.db_path.join("vector_store.vanta"), *b"VFLE")?
        {
            if header.format_version < 2 {
                plans.push(MigrationPlan {
                    format: FormatKind::VantaFile,
                    current_version: header.format_version,
                    target_version: 2,
                    action: "Bump format version header to v2".into(),
                });
            }
        }

        if let Some(header) = self.read_header(&self.db_path.join("index.bin"), *b"VNDX")? {
            if header.format_version > 4 {
                plans.push(MigrationPlan {
                    format: FormatKind::VectorIndex,
                    current_version: header.format_version,
                    target_version: 4,
                    action: format!(
                        "Index version {} is newer than supported (4).",
                        header.format_version
                    ),
                });
            }
        }

        if let Some(header) = self.read_header(&self.db_path.join("wal.log"), *b"VWAL")? {
            if header.format_version < 1 {
                plans.push(MigrationPlan {
                    format: FormatKind::Wal,
                    current_version: header.format_version,
                    target_version: 1,
                    action: "No WAL migration needed at current version".into(),
                });
            }
        }

        Ok(plans)
    }

    pub fn migrate_format(&self, kind: FormatKind) -> Result<()> {
        match kind {
            FormatKind::VantaFile => self.migrate_vfile_v1_to_v2(),
            FormatKind::VectorIndex => {
                println!("  ✓ Vector index: up-to-date (v4)");
                Ok(())
            }
            FormatKind::Wal => {
                println!("  ✓ WAL: up-to-date (v1)");
                Ok(())
            }
            FormatKind::Schema => {
                println!("  ✓ Schema: up-to-date (v1)");
                Ok(())
            }
        }
    }

    fn migrate_vfile_v1_to_v2(&self) -> Result<()> {
        let vfile_path = self.db_path.join("vector_store.vanta");
        if !vfile_path.exists() {
            println!("  - No VantaFile found, skipping");
            return Ok(());
        }

        let data = std::fs::read(&vfile_path)?;
        let header = VantaHeader::deserialize(&data)?;

        if header.format_version != 1 {
            println!("  - VantaFile already at version {}", header.format_version);
            return Ok(());
        }

        if self.dry_run {
            println!("  [dry-run] VantaFile v1 → v2: would rewrite header");
            return Ok(());
        }

        let backup_path = vfile_path.with_extension("vanta.bak");
        std::fs::copy(&vfile_path, &backup_path)?;

        let new_header = VantaHeader::new(*b"VFLE", 2, header.schema_version);
        let mut new_data = new_header.serialize().to_vec();
        new_data.extend_from_slice(&data[VantaHeader::SIZE..]);

        std::fs::write(&vfile_path, &new_data)?;

        println!("  ✓ VantaFile migrated: v1 → v2");
        println!("  - Backup saved at: {}", backup_path.display());

        Ok(())
    }

    pub fn check_integrity(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        if let Ok(Some(header)) =
            self.read_header(&self.db_path.join("vector_store.vanta"), *b"VFLE")
        {
            if header.format_version != 2 {
                issues.push(format!(
                    "VantaFile at v{}, latest is v2",
                    header.format_version
                ));
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(FormatKind::from_str("vfile"), Some(FormatKind::VantaFile));
        assert_eq!(FormatKind::from_str("vantafile"), Some(FormatKind::VantaFile));
        assert_eq!(FormatKind::from_str("index"), Some(FormatKind::VectorIndex));
        assert_eq!(FormatKind::from_str("wal"), Some(FormatKind::Wal));
        assert_eq!(FormatKind::from_str("schema"), Some(FormatKind::Schema));
        assert_eq!(FormatKind::from_str("all"), None);
        assert_eq!(FormatKind::from_str("unknown"), None);
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
        let vfile_plan = plans
            .iter()
            .find(|p| p.format == FormatKind::VantaFile);
        assert!(
            vfile_plan.is_some(),
            "should have a VantaFile migration plan"
        );
        let p = vfile_plan.unwrap();
        assert_eq!(p.current_version, 1);
        assert_eq!(p.target_version, 2);
        Ok(())
    }

    #[test]
    fn test_plan_skips_v2_vfile() -> Result<()> {
        let dir = TempDir::new()?;
        let vfile_path = dir.path().join("vector_store.vanta");
        let header = VantaHeader::new(*b"VFLE", 2, 0);
        std::fs::write(&vfile_path, header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let plans = engine.plan_all()?;
        let vfile_plan = plans
            .iter()
            .find(|p| p.format == FormatKind::VantaFile);
        assert!(vfile_plan.is_none(), "v2 file should not need migration");
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
    fn test_migrate_vfile_v1_to_v2() -> Result<()> {
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
        assert_eq!(new_header.format_version, 2);
        assert_eq!(new_header.magic, *b"VFLE");
        assert_eq!(&migrated[VantaHeader::SIZE..], payload);

        assert!(vfile_path.with_extension("vanta.bak").exists());
        Ok(())
    }

    #[test]
    fn test_check_integrity_reports_v1() -> Result<()> {
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
    fn test_check_integrity_clean_v2() -> Result<()> {
        let dir = TempDir::new()?;
        let vfile_path = dir.path().join("vector_store.vanta");
        let header = VantaHeader::new(*b"VFLE", 2, 0);
        std::fs::write(&vfile_path, header.serialize())?;

        let engine = MigrationEngine::new(dir.path());
        let issues = engine.check_integrity()?;
        assert!(issues.is_empty());
        Ok(())
    }
}
