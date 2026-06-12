//! Crash-injection verification test suite (AUD-02)
//! Runs a helper subprocess writing to a database, terminates it via SIGKILL/TerminateProcess,
//! and verifies cold-start recovery integrity.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tempfile::tempdir;
use vantadb::storage::StorageEngine;

#[test]
fn test_crash_injection_and_cold_recovery_loop() {
    TerminalReporter::suite_banner("WAL CRASH-INJECTION & RECOVERY INTEGRITY CERTIFICATION (AUD-02)", 1);
    let mut session = VantaSession::begin("Crash-Injection Loop (100 Iterations)");

    session.step("Building crash_helper binary...");
    let release_mode = cfg!(not(debug_assertions));
    let mut build_args = vec!["build", "--bin", "crash_helper"];
    if release_mode {
        build_args.push("--release");
    }

    let build_status = Command::new("cargo")
        .args(&build_args)
        .status()
        .expect("Failed to build crash_helper");

    assert!(build_status.success(), "Failed to compile crash_helper");

    let exe_name = if cfg!(windows) { "crash_helper.exe" } else { "crash_helper" };
    let profile = if release_mode { "release" } else { "debug" };
    let helper_path = std::env::current_dir()
        .unwrap()
        .join("target")
        .join(profile)
        .join(exe_name);

    assert!(helper_path.exists(), "crash_helper binary not found at {:?}", helper_path);

    session.step("Running 100 iterations of crash injection...");

    // Sembrar aleatoriedad simple sin dependencias externas pesadas
    let mut seed = 12345u64;

    for i in 1..=100 {
        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();

        // Lanzamos el crash_helper escribiendo 150 registros
        let mut child = Command::new(&helper_path)
            .arg(db_path)
            .arg("150")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn crash_helper");

        let stdout = child.stdout.take().expect("Failed to open stdout");
        let reader = BufReader::new(stdout);

        let mut written_nodes = Vec::new();

        // Generar un pseudo-random target_writes entre 10 y 80 usando un LCG LITE
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let target_writes = 10 + (seed % 70) as usize;

        for line in reader.lines() {
            if let Ok(l) = line {
                if l.starts_with("WRITTEN:") {
                    if let Some(id_str) = l.strip_prefix("WRITTEN:") {
                        if let Ok(id) = id_str.parse::<u64>() {
                            written_nodes.push(id);
                            if written_nodes.len() >= target_writes {
                                break;
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }

        // ¡Crash! Matamos al proceso hijo forzosamente
        child.kill().expect("Failed to kill crash_helper process");
        let _ = child.wait(); // Esperar a que se liberen los recursos del SO

        // Validamos la consistencia abriendo la base de datos
        let engine = StorageEngine::open(db_path)
            .expect(&format!("Iteration {}: Failed to reopen StorageEngine after crash", i));

        // Verificamos que todos los registros reportados por stdout estén legibles
        for &node_id in &written_nodes {
            let node = engine.get(node_id)
                .expect(&format!("Iteration {}: Error getting node {}", i, node_id));
            assert!(
                node.is_some(),
                "Iteration {}: Node {} was reported as WRITTEN but was not recovered after cold start!",
                i,
                node_id
            );
            let n = node.unwrap();
            assert_eq!(n.id, node_id, "Iteration {}: Node ID mismatch", i);
        }

        // Además, verifiquemos que la estructura HNSW interna sea estructuralmente válida
        let hnsw = engine.hnsw.load();
        assert!(
            hnsw.validate_index().is_ok(),
            "Iteration {}: HNSW structural index validation failed post-crash",
            i
        );

        if i % 20 == 0 {
            TerminalReporter::sub_step(&format!("Completed iteration {}/100 successfully", i));
        }
    }

    session.success("Crash-injection loop completed. 100/100 iterations recovered consistently.");
    session.finish(true);
}
