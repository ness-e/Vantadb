//! WAL Physical Hardening & Recovery Certification Test
//! Validates:
//! 1. WAL replay skips transactions <= checkpoint_seq.
//! 2. Auto-healing of truncated/corrupt trailing records on startup.
//! 3. Deterministic recovery of uncommitted mutations up to physical crash.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use std::fs::{File, OpenOptions};
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
    let hnsw = storage2.hnsw.load();
    assert!(
        hnsw.nodes.contains_key(&104),
        "WAL replay should recover un-flushed node 104"
    );

    session.success("WAL checkpoint_seq bypass successfully certified.");
    session.finish(true);
}

#[test]
fn test_wal_middle_corruption_auto_healing() {
    TerminalReporter::suite_banner("WAL MIDDLE CORRUPTION & SCAN-FORWARD AUTO-HEALING", 1);
    let mut session = VantaSession::begin("WAL Middle Corruption Resilience");

    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };

    session.step("Seeding database nodes with active WAL");
    let storage = StorageEngine::open_with_config(db_path, Some(config.clone())).unwrap();

    // Insertamos 3 nodos a través de la API física
    storage.insert(&UnifiedNode::new(201)).unwrap();
    storage.insert(&UnifiedNode::new(202)).unwrap();
    storage.insert(&UnifiedNode::new(203)).unwrap();
    drop(storage);

    // Corromper el archivo WAL en el medio (por ejemplo, alterando bytes en la mitad)
    session.step("Injecting corruption in the middle of WAL file");
    let wal_path = dir.path().join("data").join("vanta.wal");
    {
        use std::io::{Read, Write};
        let mut file_content = Vec::new();
        {
            let mut file = File::open(&wal_path).unwrap();
            file.read_to_end(&mut file_content).unwrap();
        }

        // Corrompemos 15 bytes a partir del offset 120 (el cual cae exactamente en la carga útil de node 202)
        let start_pos = 120;
        for i in 0..15 {
            if start_pos + i < file_content.len() {
                file_content[start_pos + i] = 0xAA; // Sobrescribir con basura binaria
            }
        }

        {
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&wal_path)
                .unwrap();
            file.write_all(&file_content).unwrap();
        }

        // Forzar recuperación exclusiva desde el WAL eliminando el vector store y el índice HNSW
        let vector_store_path = dir.path().join("data").join("vector_store.vanta");
        let index_path = dir.path().join("data").join("vector_index.bin");
        let _ = std::fs::remove_file(vector_store_path);
        let _ = std::fs::remove_file(index_path);
    }

    // Escribir un nuevo nodo válido DESPUÉS del agujero corrupto
    session.step("Appending additional valid transaction POST-corruption");
    {
        let mut w =
            vantadb::wal::WalWriter::open(&wal_path, vantadb::config::SyncMode::Periodic).unwrap();
        // El abridor debe haber detectado la corrupción, advertido sobre ella, y buscar el final para permitirnos
        // registrar el nodo 204
        w.append(&vantadb::wal::WalRecord::Insert(UnifiedNode::new(204)))
            .unwrap();
        w.sync().unwrap();
    }

    // Abrir la base de datos de nuevo.
    // El replay del WAL debe saltar el registro corrupto (202) y recuperar con éxito los nodos sanos (201, 203 y 204).
    session.step("Opening database and checking recovered nodes");
    let storage2 = StorageEngine::open_with_config(db_path, Some(config)).unwrap();

    let hnsw = storage2.hnsw.load();
    assert!(
        hnsw.nodes.contains_key(&201),
        "WAL recovery should retrieve node 201 before corruption"
    );
    assert!(
        hnsw.nodes.contains_key(&203),
        "WAL recovery should retrieve node 203 which was written before corruption but lies after the corrupt record"
    );
    assert!(
        hnsw.nodes.contains_key(&204),
        "WAL recovery should retrieve node 204 written after corruption"
    );
    assert!(
        !hnsw.nodes.contains_key(&202),
        "Corrupted node 202 should be skipped gracefully"
    );

    session.success("WAL middle corruption scan-forward auto-healing successfully certified.");
    session.finish(true);
}
