//! Memory backup/restore over the core filesystem-snapshot surface (MEMG-13).
//!
//! Thin delegation by design: the core owns snapshot semantics — quiesce
//! (flush) + mirror + rollback in `StorageEngine` (`src/storage/engine/mod.rs`)
//! — and this module is the memory-scoped entry point plus the flow contract
//! hosts need. No logic is reimplemented here (api-contract R-8: the core owns
//! the behavior; the layer above only exposes it).
//!
//! ```text
//! backup:  create_snapshot(db, name)                  // point-in-time image
//! inspect: list_snapshots(db)                         // names available
//! restore: db.close() -> restore_snapshot(config, name) -> reopened db
//! rollback: db.close() -> rollback_snapshot(config, name) // + declared scope (MEMG-17)
//! ```
//!
//! A snapshot requires an on-disk store (Fjall): `InMemory` engines keep no
//! files. Restore swaps `<storage_path>/data` on disk, so no engine may hold
//! the database open — close first, then restore, which returns a freshly
//! reopened [`Embedded`] with indexes rebuilt from storage (core contract,
//! `Embedded::restore_from`).
//!
//! Scope of the restore (core contract): only the `data/` directory is
//! swapped back; the live backend KV (Fjall LSM) is intentionally left in
//! place (FIND-33 decision). Post-snapshot additions disappear with `data/`,
//! while post-snapshot deletions/supersessions keep their backend tombstones
//! and are **not** rolled back (FIND-287).

use vantadb::config::Config;
use vantadb::error::Result;
use vantadb::sdk::Embedded;
use vantadb::storage::FsSnapshot;

/// Create a point-in-time snapshot of the memory store (core
/// `Embedded::create_snapshot`: quiesce + mirror, FIND-25/FIND-33).
///
/// `name` must be a plain identifier — the core validates it (no path
/// separators, `.`/`..`, or control characters) before touching the
/// filesystem; this wrapper never relaxes that guard.
pub fn create_snapshot(db: &Embedded, name: &str) -> Result<FsSnapshot> {
    db.create_snapshot(name)
}

/// List every snapshot name available to [`restore_snapshot`], sorted.
pub fn list_snapshots(db: &Embedded) -> Result<Vec<String>> {
    db.list_snapshots()
}

/// Restore the memory store from a snapshot and reopen it.
///
/// Expected flow: [`Embedded::close`] the current engine first (the restore
/// swaps the on-disk data directory and requires no open engine / fs2 lock),
/// then call this. Returns a freshly reopened [`Embedded`] — indexes rebuild
/// from storage on open. Fails with `NotFound` when the snapshot does not
/// exist; the live directory is staged aside with rollback-on-failure (core
/// contract).
pub fn restore_snapshot(config: Config, name: &str) -> Result<Embedded> {
    Embedded::restore_from(config, name)
}

/// Declared scope of a [`rollback_snapshot`] (MEMG-17): what the restore
/// reverts and — explicitly — what it does not (FIND-287: the restore is
/// data-only). Always populated, never silent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRollbackReport {
    /// Snapshot name that was restored.
    pub snapshot: String,
    /// What the restore brings back.
    pub reverted: Vec<String>,
    /// What the restore leaves untouched.
    pub not_reverted: Vec<String>,
    /// Flow caveats of the restore.
    pub caveats: Vec<String>,
}

/// Restore the memory store from a snapshot and return the declared rollback
/// scope (MEMG-17, pieza a — snapshot path).
///
/// Thin wrapper over [`restore_snapshot`] (the core owns the semantics, MEMG-13
/// delegation contract): same flow (close → restore → reopen) and same errors;
/// the addition is the [`SnapshotRollbackReport`] making the data-only scope
/// explicit — the `data/` swap is reverted, while the live backend KV
/// (post-snapshot deletes/supersessions keep their tombstones/metadata) is
/// **not** rolled back (FIND-287, FIND-33 decision).
pub fn rollback_snapshot(config: Config, name: &str) -> Result<(Embedded, SnapshotRollbackReport)> {
    let db = restore_snapshot(config, name)?;
    Ok((db, rollback_scope(name)))
}

/// The static declared scope of a snapshot rollback.
fn rollback_scope(name: &str) -> SnapshotRollbackReport {
    SnapshotRollbackReport {
        snapshot: name.to_string(),
        reverted: vec![
            "the `data/` directory is swapped back to the snapshot point: post-snapshot additions disappear; the snapshot's files are back on disk (record visibility still subject to the live backend tombstones — see `not_reverted`)".into(),
            "in-memory indexes rebuild from the restored storage on reopen (fresh `Embedded`)".into(),
        ],
        not_reverted: vec![
            "live backend KV (`<storage_root>/backend/`, Fjall LSM) is left in place: deletes and supersessions recorded after the snapshot keep their tombstones/metadata — data-only restore (FIND-287; FIND-33 decision)".into(),
            "retained version history (Versions partition, live backend) is not rolled back to the snapshot point (same data-only gap, FIND-287)".into(),
            "audit logs, WAL archives and any snapshot other than the restored one are untouched".into(),
        ],
        caveats: vec![
            "requires the engine closed first (fs2 lock) — flow: close -> restore -> reopened `Embedded`".into(),
            "fails with `NotFound` when the snapshot does not exist; the live `data/` directory is staged aside with rollback-on-failure (core contract)".into(),
        ],
    }
}
