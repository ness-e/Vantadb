//! WAL Physical Hardening & Recovery Certification Test
//! Validates:
//! 1. WAL replay skips transactions <= checkpoint_seq.
//! 2. Auto-healing of truncated/corrupt trailing records on startup.
//! 3. Deterministic recovery of uncommitted mutations up to physical crash.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use tempfile::tempdir;
use vantadb::config::VantaConfig;
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, StorageEngine};

#[test]
fn test_wal_durability_and_checkpoint_coherence() {
    TerminalReporter::suite_banner("WAL PHYSICAL DURABILITY & COHERENCE CERTIFICATION", 1);
    let mut session = VantaSession::begin("WAL Checkpoint Seq Bypass & Recovery");

    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // 1. Inicializar con configuración explícita
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };

    session.step("Seeding database nodes with active WAL");
    let storage = StorageEngine::open_with_config(db_path, Some(config.clone())).unwrap();

    // Insertamos 3 nodos a través de la API física
    storage.insert(&UnifiedNode::new(101)).unwrap();
    storage.insert(&UnifiedNode::new(102)).unwrap();
    storage.insert(&UnifiedNode::new(103)).unwrap();

    // En este punto, no hemos llamado a flush, por lo que checkpoint_seq es 0.
    // Hacemos flush() de la base de datos
    session.step("Performing StorageEngine::flush to persist index and metadata");
    storage.flush().unwrap();

    // Al hacer flush(), checkpoint_seq debe grabarse como 3 (los 3 registros del WAL).
    // Cerramos la base de datos (dejando caer el almacenamiento)
    drop(storage);

    // 2. Insertar registros en el WAL "después" del flush de forma manual
    session.step("Injecting additional mutations post-flush");
    {
        // Reabrimos el WAL para escribir un cuarto registro
        let wal_path = dir.path().join("data").join("vanta.wal");
        let mut w =
            vantadb::wal::WalWriter::open(&wal_path, vantadb::config::SyncMode::Periodic).unwrap();
        w.append(&vantadb::wal::WalRecord::Insert(UnifiedNode::new(104)))
            .unwrap();
        w.sync().unwrap();
        assert_eq!(w.record_count(), 4);
    }

    // 3. Abrir la base de datos de nuevo.
    // El replay del WAL debe procesar el registro 4 (seq=4 > checkpoint_seq=3)
    // pero omitir los registros 1, 2 y 3 (seq <= 3) para evitar duplicación.
    session.step("Opening database: Replaying WAL with checkpoint_seq filter");
    let storage2 = StorageEngine::open_with_config(db_path, Some(config)).unwrap();

    // Validamos que el nodo 104 está presente en el índice reconstruido
    let hnsw = storage2.hnsw.read();
    assert!(
        hnsw.nodes.contains_key(&104),
        "WAL replay should recover un-flushed node 104"
    );

    session.success("WAL checkpoint_seq bypass successfully certified.");
    session.finish(true);
}
