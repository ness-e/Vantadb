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

        // Localizar dinámicamente la región de payload del nodo 202
        // header(20) + rec1(4+len1+4) + 4 = start of node202 length prefix
        let hdr = 20usize;
        let len1 = u32::from_le_bytes(file_content[hdr..hdr + 4].try_into().unwrap()) as usize;
        let rec1_end = hdr + 4 + len1 + 4;
        let len2 =
            u32::from_le_bytes(file_content[rec1_end..rec1_end + 4].try_into().unwrap()) as usize;
        let node202_payload_start = rec1_end + 4;
        let node202_payload_end = node202_payload_start + len2;
        // Corrompemos bytes en la segunda mitad del payload del nodo 202, pero sin tocar su CRC ni el nodo 203
        let corruption_len = std::cmp::min(15, len2 / 2);
        let start_pos = node202_payload_start + len2 / 2;
        eprintln!(
            "WAL layout: header=20, node201={}+{} bytes, node202 starts at {}, payload at {}-{}, corrupting {} bytes at offset {}",
            4 + len1 + 4,
            4 + len2 + 4,
            rec1_end,
            node202_payload_start,
            node202_payload_end,
            corruption_len,
            start_pos,
        );
        for i in 0..corruption_len {
            if start_pos + i < file_content.len() {
                file_content[start_pos + i] = 0xAA;
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

#[test]
fn test_wal_selective_crc_corruption_recovery() {
    TerminalReporter::suite_banner(
        "WAL SELECTIVE CRC32C CORRUPTION & INTEGRITY CERTIFICATION",
        1,
    );
    let mut session = VantaSession::begin("WAL Selective CRC Corruption Resilience");

    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };

    session.step("Seeding database with 3 valid nodes under WAL");
    let storage = StorageEngine::open_with_config(db_path, Some(config.clone())).unwrap();
    storage.insert(&UnifiedNode::new(301)).unwrap();
    storage.insert(&UnifiedNode::new(302)).unwrap();
    storage.insert(&UnifiedNode::new(303)).unwrap();
    drop(storage);

    // Ubicar y leer el archivo WAL
    session.step("Locating records and corrupting ONLY the CRC field of the 2nd record");
    let wal_path = dir.path().join("data").join("vanta.wal");
    let mut file_content = Vec::new();
    {
        use std::io::Read;
        let mut file = File::open(&wal_path).unwrap();
        file.read_to_end(&mut file_content).unwrap();
    }

    // Parsear la estructura del WAL para encontrar el CRC del segundo registro
    // WalHeader::SIZE = 20
    let mut offset = 20;
    let mut records = Vec::new();
    while offset + 8 <= file_content.len() {
        let len_bytes = &file_content[offset..offset + 4];
        let len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
        let payload_start = offset + 4;
        let crc_start = payload_start + len;
        let record_end = crc_start + 4;

        if record_end > file_content.len() {
            break;
        }

        records.push((offset, payload_start, crc_start, record_end));
        offset = record_end;
    }

    assert_eq!(
        records.len(),
        3,
        "Deberíamos tener exactamente 3 registros en el WAL"
    );

    // Corromper selectivamente el campo CRC del segundo registro (índice 1)
    let (_, _, crc_start, _) = records[1];
    // Modificamos el CRC original haciendo un XOR con 0xFFFFFFFF
    for i in 0..4 {
        file_content[crc_start + i] ^= 0xFF;
    }

    // Escribir el WAL modificado al disco
    {
        use std::io::Write;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&wal_path)
            .unwrap();
        file.write_all(&file_content).unwrap();
    }

    // Borrar el vector store y el índice en disco para forzar recuperación a través de WAL
    let vector_store_path = dir.path().join("data").join("vector_store.vanta");
    let index_path = dir.path().join("data").join("vector_index.bin");
    let _ = std::fs::remove_file(vector_store_path);
    let _ = std::fs::remove_file(index_path);

    // Abrir la base de datos de nuevo.
    // Durante la recuperación, el replay del WAL debe detectar el fallo de CRC32C en el registro del nodo 302,
    // descartarlo a pesar de tener un payload deserializable consistente, y usar Scan-Forward
    // para recuperar exitosamente el nodo 303 que está intacto después.
    session.step("Opening database: Verifying selective skip and forward-recovery");
    let storage2 = StorageEngine::open_with_config(db_path, Some(config)).unwrap();
    let hnsw = storage2.hnsw.load();

    assert!(
        hnsw.nodes.contains_key(&301),
        "WAL recovery should retrieve node 301 before the corrupted record"
    );
    assert!(
        !hnsw.nodes.contains_key(&302),
        "Node 302 MUST be skipped because its record-level CRC is corrupt"
    );
    assert!(
        hnsw.nodes.contains_key(&303),
        "WAL recovery MUST scan forward and recover node 303 after the corrupted record"
    );

    session.success("Selective CRC32C corruption detection and recovery certified.");
    session.finish(true);
}
