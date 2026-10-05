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
