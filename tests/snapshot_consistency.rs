//! FIND-33: snapshot filesystem must capture backend KV state.
//!
//! Validation: after `create_snapshot`, the snapshot directory contains both
//! `<snap>/data/` (VantaFile + HNSW + WAL) AND `<snap>/backend/` (Fjall LSM
//! files for namespace_index, internal_metadata, tombstones, etc.). Without
//! the backend capture, a snapshot taken after `compact_wal()` would lose any
//! state that lives only in the KV backend (metadata/edges/checkpoint_seq).
//!
//! The pre-fix behaviour: only `data/` is mirrored; the test fails because
//! `<snap>/backend/` does not exist or is empty.

use std::collections::BTreeMap;
use tempfile::tempdir;
use vantadb::config::VantaConfig;
use vantadb::{BackendKind, VantaEmbedded, VantaMemoryInput};

fn put_record(db: &VantaEmbedded, ns: &str, key: &str, payload: &str, vec: Vec<f32>) {
    db.put(VantaMemoryInput {
        namespace: ns.to_string(),
        key: key.to_string(),
        payload: payload.to_string(),
        vector: Some(vec),
        sparse_vector: None,
        metadata: BTreeMap::new(),
        ttl_ms: None,
    })
    .expect("put");
}

fn fjall_config(storage_path: &str) -> VantaConfig {
    VantaConfig {
        storage_path: storage_path.to_string(),
        backend_kind: BackendKind::Fjall,
        ..VantaConfig::default()
    }
}

#[test]
fn snapshot_captures_backend_kv_state_after_compact_wal() {
    let dir = tempdir().expect("tempdir");
    let storage_path = dir.path().to_string_lossy().into_owned();

    // Phase 1: open + seed + snapshot under Fjall backend (default feature).
    let snap_path = {
        let config = fjall_config(&storage_path);
        let db = VantaEmbedded::open_with_config(config).expect("open fjalldb");

        // 3 records across 2 namespaces — backend (namespace_index) tracks
        // these via Fjall LSM files under `<storage_path>/`.
        put_record(&db, "ns/alpha", "rec-1", "first alpha", vec![1.0, 0.0, 0.0]);
        put_record(
            &db,
            "ns/alpha",
            "rec-2",
            "second alpha",
            vec![0.0, 1.0, 0.0],
        );
        put_record(&db, "ns/beta", "rec-3", "only beta", vec![0.5, 0.5, 0.0]);

        // Bump checkpoint_seq and archive WAL — pre-fix bug: this would orphan
        // any data only in the backend KV (checkpoint_seq, namespace_index).
        db.flush().expect("flush pre-snapshot");
        let snap = db.create_snapshot("post_compact").expect("create snapshot");
        db.compact_wal().expect("compact_wal post-snapshot");

        // Sanity: data survives in live engine.
        assert!(db.get("ns/alpha", "rec-1").expect("get live").is_some());
        assert!(db.get("ns/beta", "rec-3").expect("get live").is_some());

        snap.path
    };

    // Phase 2: the snapshot directory must mirror BOTH data_dir and the
    // backend KV dir. Pre-fix: only data/ exists → assertion fails (RED).
    let snap_data = snap_path.join("data");
    let snap_backend = snap_path.join("backend");
    assert!(
        snap_data.is_dir(),
        "snapshot must contain data/: missing ({})",
        snap_data.display()
    );
    assert!(
        snap_backend.is_dir(),
        "snapshot must also capture backend/ to be recoverable after compact_wal: \
         missing ({}) — this is FIND-33 (backend KV lost on snapshot).",
        snap_backend.display()
    );
    // The backend dir must contain at least one LSM file (otherwise it's an
    // empty mirror, defeating the point).
    let backend_entry_count = std::fs::read_dir(&snap_backend)
        .expect("read backend snapshot dir")
        .count();
    assert!(
        backend_entry_count > 0,
        "snapshot backend/ is empty ({backend_entry_count} entries) — backend KV \
         state not captured. FIND-33 regression: snapshot is unreliable after compact_wal."
    );

    // Phase 3: restore + reopen via VantaEmbedded::restore_from (which calls
    // snapshot_restore + open_with_config). The reopened engine must surface
    // the seeded records — proves the captured backend checkpoint_seq plus
    // replayed WAL reconstruct namespace state.
    let restore_config = fjall_config(&storage_path);
    let restored =
        VantaEmbedded::restore_from(restore_config, "post_compact").expect("restore from snapshot");
    assert!(restored
        .get("ns/alpha", "rec-1")
        .expect("get rec-1")
        .is_some());
    assert!(restored
        .get("ns/alpha", "rec-2")
        .expect("get rec-2")
        .is_some());
    assert!(restored
        .get("ns/beta", "rec-3")
        .expect("get rec-3")
        .is_some());

    let namespaces = restored.list_namespaces().expect("list namespaces");
    assert!(namespaces.contains(&"ns/alpha".to_string()));
    assert!(namespaces.contains(&"ns/beta".to_string()));
}

#[test]
fn snapshot_directory_layout_contains_both_data_and_backend() {
    // Tighter contract: regardless of state, the snapshot layout mirrors the
    // live storage layout (data/ + backend/ as siblings under the snap root).
    let dir = tempdir().expect("tempdir");
    let config = fjall_config(&dir.path().to_string_lossy());
    let db = VantaEmbedded::open_with_config(config).expect("open");
    put_record(&db, "layout", "k", "v", vec![1.0, 0.0]);
    db.flush().expect("flush");
    let snap = db.create_snapshot("layout_check").expect("snapshot");

    let snap_root = snap.path.clone();
    let data = snap_root.join("data");
    let backend = snap_root.join("backend");
    assert!(
        data.is_dir() && backend.is_dir(),
        "snap root must contain both `data/` and `backend/` siblings. \
         data={}, backend={} (FIND-33 layout contract)",
        data.display(),
        backend.display()
    );

    // Live layout invariant: storage_path/ contains `data/` and the LSM files
    // live beside it (under storage_path/, NOT under data/). The snapshot
    // mirror must respect this invariant — else restore cannot reopen.
    let live_data = dir.path().join("data");
    let live_backend_entries: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read live storage_path")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    assert!(
        live_data.is_dir(),
        "live storage_path must contain data/ — sanity check"
    );
    assert!(
        !live_backend_entries.is_empty(),
        "live storage_path must contain backend LSM files at the root \
         (sanity check — Fjall opens under storage_path, sibling of data/)"
    );
}
