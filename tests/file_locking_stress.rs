/// Stress test for file locking under concurrent access patterns.
/// Simulates scenarios that could occur with antivirus scanning or
/// backup software holding temporary file locks.
use std::fs::OpenOptions;
use std::time::Duration;
use tempfile::TempDir;
use vantadb::config::VantaConfig;
use vantadb::error::VantaError;
use vantadb::sdk::VantaMemoryInput;
use vantadb::VantaEmbedded;

#[test]
fn test_file_lock_exclusive_prevents_second_open() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_str().unwrap().to_string();

    let db1 = VantaEmbedded::open(&path).unwrap();

    let result = VantaEmbedded::open(&path);
    assert!(
        result.is_err(),
        "Second open should fail when first holds exclusive lock"
    );
    match result.err().unwrap() {
        VantaError::DatabaseBusy(msg) => {
            assert!(
                msg.contains("locked by another process"),
                "Expected 'locked by another process' in error message, got: {}",
                msg
            );
        }
        other => panic!(
            "Expected VantaError::DatabaseBusy for lock failure, got: {:?}",
            other
        ),
    }

    drop(db1);

    let db2 = VantaEmbedded::open(&path).unwrap();
    drop(db2);
}

#[test]
fn test_read_only_shared_lock_prevents_exclusive_writer() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_str().unwrap().to_string();

    // Initialize the database with a writer
    let db_w = VantaEmbedded::open(&path).unwrap();
    db_w.put(VantaMemoryInput::new("ns", "k1", "v1")).unwrap();
    db_w.close().unwrap();

    // Open read-only (acquires shared lock on .vanta.lock)
    let config_ro = VantaConfig {
        storage_path: path.clone(),
        read_only: true,
        ..Default::default()
    };
    let db_ro = VantaEmbedded::open_with_config(config_ro).unwrap();

    // While read-only holds a shared lock, a new writer should fail
    let result = VantaEmbedded::open(&path);
    assert!(
        result.is_err(),
        "Writer open should fail when read-only holds the shared lock"
    );

    // Drop the read-only handle (releases shared lock)
    drop(db_ro);

    // Now the writer should succeed
    let db_w2 = VantaEmbedded::open(&path).unwrap();
    db_w2.close().unwrap();
}

#[test]
fn test_file_lock_timeout_eventually_succeeds() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_str().unwrap().to_string();

    for _ in 0..10 {
        let db = VantaEmbedded::open(&path).unwrap();
        db.put(VantaMemoryInput::new("ns", "k", "v")).unwrap();
        db.close().unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn test_vanta_lock_file_shared_and_exclusive_os_level() {
    let dir = TempDir::new().unwrap();
    let lock_path = dir.path().join(".vanta.lock");

    // Open two file handles to the same lock file
    let f1 = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)
        .unwrap();
    let f2 = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)
        .unwrap();

    // f1 acquires a shared lock (read-only)
    fs2::FileExt::try_lock_shared(&f1).expect("f1 should acquire shared lock");

    // f2 should also be able to acquire a shared lock (multiple readers allowed)
    fs2::FileExt::try_lock_shared(&f2)
        .expect("f2 should acquire shared lock while f1 holds shared");

    // Release both shared locks
    fs2::FileExt::unlock(&f2).expect("f2 unlock");
    fs2::FileExt::unlock(&f1).expect("f1 unlock");

    // f3 acquires an exclusive lock
    let f3 = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)
        .unwrap();
    fs2::FileExt::try_lock_exclusive(&f3)
        .expect("f3 should acquire exclusive lock when no other handles hold locks");

    // f4 should NOT be able to acquire an exclusive lock while f3 holds it
    let f4 = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)
        .unwrap();
    let exclusive_while_exclusive = fs2::FileExt::try_lock_exclusive(&f4);
    assert!(
        exclusive_while_exclusive.is_err(),
        "Second exclusive lock should fail while first exclusive lock is held"
    );

    // f4 should NOT be able to acquire a shared lock while f3 holds exclusive
    let shared_while_exclusive = fs2::FileExt::try_lock_shared(&f4);
    assert!(
        shared_while_exclusive.is_err(),
        "Shared lock should fail while exclusive lock is held"
    );

    fs2::FileExt::unlock(&f3).expect("f3 unlock");
}
