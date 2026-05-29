use std::io::{BufRead, BufReader};
use std::process::Command;
use tempfile::tempdir;
use vantadb::config::VantaConfig;
use vantadb::error::VantaError;
use vantadb::storage::StorageEngine;

/// Certifica que:
/// 1. Un segundo escritor NO puede abrir la base de datos mientras el primero tiene el lock.
/// 2. El error devuelto es claro e identificable como lock failure.
/// 3. Tras liberar el primer escritor, el directorio puede abrirse de nuevo.
#[test]
fn test_exclusive_writer_lock_prevents_second_writer() {
    let dir = tempdir().expect("Failed to create temp directory");
    let path_str = dir.path().to_str().unwrap();

    // 1. Abrimos el primer StorageEngine de escritura (adquiere el lock exclusivo)
    let config1 = VantaConfig {
        read_only: false,
        ..Default::default()
    };
    let _engine1 = StorageEngine::open_with_config(path_str, Some(config1))
        .expect("First writer should open without issues");

    // 2. Intentamos abrir un segundo escritor sobre el mismo directorio.
    //    Esto debe fallar con un error de lock explícito.
    let config2 = VantaConfig {
        read_only: false,
        ..Default::default()
    };
    let engine2_res = StorageEngine::open_with_config(path_str, Some(config2));
    assert!(
        engine2_res.is_err(),
        "Second writer should NOT be able to open the locked database"
    );

    // 3. Validar que el error sea explícito y contenga información de lock.
    match engine2_res.err().unwrap() {
        VantaError::Execution(msg) => {
            assert!(
                msg.contains("locked by another process"),
                "Expected 'locked by another process' in error message, got: {}",
                msg
            );
        }
        other => panic!(
            "Expected VantaError::Execution for lock failure, got: {:?}",
            other
        ),
    }

    // 4. Liberamos el primer writer dejando que _engine1 se drop aquí implícitamente.
    drop(_engine1);

    // 5. Tras liberar el lock, debe ser posible abrir de nuevo.
    let config3 = VantaConfig {
        read_only: false,
        ..Default::default()
    };
    let engine3_res = StorageEngine::open_with_config(path_str, Some(config3));
    assert!(
        engine3_res.is_ok(),
        "Should be able to open after releasing the lock"
    );
}

/// Certifica de forma empírica y multiproceso que:
/// 1. Un subproceso independiente que abre StorageEngine bloquea físicamente la base de datos (.vanta.lock).
/// 2. El proceso del test no puede abrir el motor sobre el mismo directorio mientras el subproceso esté activo.
/// 3. Otro subproceso también es rechazado con código de salida 2 (fallo controlado).
/// 4. Tras la muerte del subproceso inicial, el lock se libera en el OS y la base de datos es accesible.
#[test]
fn test_exclusive_writer_lock_prevents_second_writer_multi_process() {
    let dir = tempdir().expect("Failed to create temp directory");
    let path_str = dir.path().to_str().unwrap();

    // 1. Lanzamos el primer lock_helper (Proceso P1) que abrirá y retendrá el lock por 5000 ms.
    let helper_path = env!("CARGO_BIN_EXE_lock_helper");
    let mut p1 = Command::new(helper_path)
        .arg(path_str)
        .arg("5000") // Tiempo suficiente para que se ejecute el test
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn lock_helper P1");

    // 2. Leemos stdout de P1 hasta capturar "LOCK_HELPER: SUCCESS_LOCK"
    // Esto nos garantiza que P1 ya posee el bloqueo antes de continuar.
    let stdout = p1.stdout.take().expect("Failed to open stdout of P1");
    let mut reader = BufReader::new(stdout);
    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .expect("Failed to read from P1");
    assert!(
        first_line.contains("LOCK_HELPER: SUCCESS_LOCK"),
        "Expected P1 to lock database successfully, got: {}",
        first_line
    );

    // 3. Con P1 en ejecución sosteniendo el lock, intentamos abrir el motor desde el proceso del test.
    // Esto debe fallar debido al bloqueo a nivel de OS.
    let config2 = VantaConfig {
        read_only: false,
        ..Default::default()
    };
    let engine_res = StorageEngine::open_with_config(path_str, Some(config2));
    assert!(
        engine_res.is_err(),
        "Current process should NOT be able to open the database while P1 holds the lock"
    );
    if let Err(VantaError::Execution(msg)) = engine_res {
        assert!(
            msg.contains("locked by another process"),
            "Expected lock failure message, got: {}",
            msg
        );
    } else {
        panic!("Expected VantaError::Execution for lock failure");
    }

    // 4. Lanzamos un segundo lock_helper (Proceso P2) sobre el mismo directorio.
    // P2 debe fallar de inmediato al intentar adquirir el lock y salir con código 2.
    let p2_output = Command::new(helper_path)
        .arg(path_str)
        .arg("1000")
        .output()
        .expect("Failed to execute lock_helper P2");

    assert_eq!(
        p2_output.status.code(),
        Some(2),
        "P2 should fail with exit code 2 due to locking conflict"
    );
    let p2_stdout = String::from_utf8_lossy(&p2_output.stdout);
    assert!(
        p2_stdout.contains("LOCK_HELPER: FAILED_LOCK"),
        "P2 stdout should report lock failure, got: {}",
        p2_stdout
    );

    // 5. Matamos a P1 para forzar la liberación del lock en el OS de inmediato.
    p1.kill().expect("Failed to kill P1");
    let _ = p1.wait();

    // 6. Con el lock liberado, la apertura desde el proceso del test debe tener éxito.
    let config3 = VantaConfig {
        read_only: false,
        ..Default::default()
    };
    let engine3_res = StorageEngine::open_with_config(path_str, Some(config3));
    assert!(
        engine3_res.is_ok(),
        "Should be able to open database after P1 was killed and the lock was released"
    );
}
