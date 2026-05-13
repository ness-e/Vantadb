//! Raw ANN search over mmap vector store (`search_nearest` + `Some(&vs)`) must not return
//! logically deleted rows (disk tombstone flags on `DiskNodeHeader`).

use tempfile::tempdir;
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, EngineConfig, StorageEngine};

#[test]
fn raw_ann_layers_from_vantafile_exclude_tombstoned_neighbors() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let config = EngineConfig {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    let engine = StorageEngine::open_with_config(path, Some(config)).unwrap();

    let query = vec![1.0f32, 0.0f32, 0.0f32];

    // Very close neighbor that will be tombstoned; should dominate similarity if wrongly returned.
    let soon_deleted = UnifiedNode::with_vector(100, vec![0.999f32, 0.0f32, 0.0f32]);
    let survivor = UnifiedNode::with_vector(200, vec![0.8f32, 0.6f32, 0.0f32]);

    engine.insert(&soon_deleted).unwrap();
    engine.insert(&survivor).unwrap();
    engine.flush().unwrap();

    // Hard delete marks VantaFile header tombstone flag; CPIndex node map may still retain graph refs.
    engine.delete(100, "test tombstone for ANN").unwrap();
    engine.flush().unwrap();

    let hnsw = engine.hnsw.read();
    let vs = engine.vector_store.read();
    let hits = hnsw.search_nearest(&query, None, None, 0u128, 8, Some(&vs));

    assert!(
        hits.iter().all(|(id, _)| *id != 100),
        "deleted node id must not surface in tombstone-eligible candidate set {:?}",
        hits
    );

    assert!(
        hits.iter().any(|(id, _)| *id == 200),
        "expected live survivor in ANN ranked list, got {:?}",
        hits
    );
}
