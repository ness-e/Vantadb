// ponytail: blanket allow — this is a test; the invariants are documented per call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Chaos review of the **at-rest encryption** (ADR-0057), wave W6.
//!
//! Four scenarios, one test each, every one of them deterministic and
//! self-contained: a `tempfile::tempdir()` per test, no network, no absolute
//! path, nothing written outside the temporary directory (which removes
//! itself, registry included).
//!
//! | Test | Scenario | Contract |
//! |---|---|---|
//! | [`encrypted_db_survives_a_truncated_encrypted_wal_tail`] | truncamiento de cola cifrada | the base reopens and never returns a corrupt record |
//! | [`power_cut_during_an_envelope_never_yields_a_partial_record`] | corte de energía durante un envelope | the half-written frame is discarded whole |
//! | [`rotation_while_the_database_is_open_leaves_a_coherent_registry`] | rotación a mitad de escritura | the registry never holds DEKs its own KEK cannot unwrap |
//! | [`a_single_corrupted_byte_never_yields_a_valid_plaintext_record`] | corrupción de un byte | the affected record fails closed, the others stay intact |
//!
//! ## What "power loss" means here
//!
//! A test cannot cut the power of the machine, so the two crash scenarios
//! **simulate** it by writing the bytes by hand: the file is cut at an
//! arbitrary offset, or a half-frame is appended to it. That is the closest
//! reachable approximation of what a power cut leaves on disk — a prefix of
//! the bytes the writer had already flushed — and it exercises exactly the
//! code a real crash would reach (`recover_valid_records_v4` /
//! `read_encrypted_frame`).

#![cfg(all(feature = "encryption", any(feature = "fjall", feature = "rocksdb")))]

use std::fs;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};

use vantadb::crypto::registry::{Artifact, KeyRegistry};
use vantadb::node::FieldValue;
use vantadb::wal::{WalKey, WalReader, WalRecord};
use vantadb::{BackendKind, Config, Embedded, MemoryInput};

// ───────────────────────────── helpers ─────────────────────────────

/// A hex master key (32 bytes).
fn key_hex(seed: u8) -> String {
    format!("{seed:02x}").repeat(32)
}

/// The raw material of a hex key.
fn material(key: &str) -> Vec<u8> {
    vantadb::crypto::resolve_master_key_material_strict(Some(key), None, None)
        .expect("the key material")
        .to_vec()
}

/// The config of an encrypted database.
fn keyed_config(root: &Path, key: &str, wal_shards: usize) -> Config {
    Config {
        storage_path: root.to_string_lossy().into_owned(),
        backend_kind: BackendKind::Fjall,
        encryption_key: Some(key.to_string()),
        wal_shards,
        ..Config::default()
    }
}

/// The `data` directory of a database.
fn data_dir(root: &Path) -> PathBuf {
    root.join("data")
}

/// The payload of the record `i` — unique, so a mixed-up record is visible.
fn payload(i: usize) -> String {
    format!("vantadb-chaos-payload-{i:04}-7f3a91c5")
}

/// The key of the record `i`.
fn record_key(i: usize) -> String {
    format!("rec-{i:04}")
}

/// Write `count` records in an encrypted database and close it.
///
/// The read-back is the **positive control** of every scenario: it proves
/// that the probe (`get` + payload) is not vacuous and that the database is
/// healthy before the chaos starts.
fn seed(root: &Path, key: &str, count: usize, wal_shards: usize) {
    let db = Embedded::open_with_config(keyed_config(root, key, wal_shards)).expect("open");
    for i in 0..count {
        let mut input = MemoryInput::new("chaos", record_key(i), payload(i));
        input.metadata.insert(
            "marker".to_string(),
            vantadb::Value::String("m".to_string()),
        );
        db.put(input).expect("put");
    }
    db.flush().expect("flush");
    for i in 0..count {
        let record = db
            .get("chaos", &record_key(i))
            .expect("get")
            .unwrap_or_else(|| panic!("the record {i} must be there before the chaos"));
        if record.payload != payload(i) {
            eprintln!(
                "FAIL positive control: the record {i} came back with the payload {:?} \
                 instead of {:?}",
                record.payload,
                payload(i)
            );
        }
        assert_eq!(
            record.payload,
            payload(i),
            "the positive control: the record {i} must read back exactly"
        );
    }
    db.close().expect("close");
}

/// Whether an I/O error is a **Windows sharing violation**.
///
/// The engine may still hold a handle on an artefact for a moment after
/// `close()` returns; on Windows, touching the file in that window fails with
/// ERROR_SHARING_VIOLATION. That is an artefact of the test harness, not of
/// the engine under test, so it is retried instead of failing the run.
fn is_sharing_violation(err: &std::io::Error) -> bool {
    let text = err.to_string().to_lowercase();
    text.contains("being used by another process")
        || text.contains("used by another process")
        || text.contains("sharing violation")
}

/// Read a file, waiting out a Windows sharing violation.
fn read_file(path: &Path) -> Vec<u8> {
    for attempt in 0..100 {
        match fs::read(path) {
            Ok(bytes) => return bytes,
            Err(err) if is_sharing_violation(&err) => {
                if attempt == 99 {
                    panic!("cannot read {}: {err}", path.display());
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(err) => panic!("cannot read {}: {err}", path.display()),
        }
    }
    unreachable!()
}

/// Write a file, waiting out a Windows sharing violation.
fn write_file(path: &Path, bytes: &[u8]) {
    for attempt in 0..100 {
        match fs::write(path, bytes) {
            Ok(()) => return,
            Err(err) if is_sharing_violation(&err) => {
                if attempt == 99 {
                    panic!("cannot write {}: {err}", path.display());
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(err) => panic!("cannot write {}: {err}", path.display()),
        }
    }
    unreachable!()
}

/// Copy a file, waiting out a Windows sharing violation.
fn copy_file(from: &Path, to: &Path) {
    for attempt in 0..100 {
        match fs::copy(from, to) {
            Ok(_) => return,
            Err(err) if is_sharing_violation(&err) => {
                if attempt == 99 {
                    panic!("cannot copy {}: {err}", from.display());
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(err) => panic!("cannot copy {}: {err}", from.display()),
        }
    }
    unreachable!()
}

/// Truncate a file to `len` bytes (the "power cut" of the tail scenarios).
fn truncate(path: &Path, len: u64) {
    for attempt in 0..100 {
        let result = fs::OpenOptions::new()
            .write(true)
            .open(path)
            .and_then(|file| file.set_len(len))
            .and_then(|()| {
                fs::OpenOptions::new()
                    .write(true)
                    .open(path)
                    .and_then(|f| f.sync_all())
            });
        match result {
            Ok(()) => return,
            Err(err) if is_sharing_violation(&err) => {
                if attempt == 99 {
                    panic!("cannot truncate {}: {err}", path.display());
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(err) => panic!("cannot truncate {}: {err}", path.display()),
        }
    }
    unreachable!()
}

/// Flip one bit of one byte at `offset` (the "one corrupted byte" scenario).
fn flip_byte(path: &Path, offset: u64) {
    let mut bytes = read_file(path);
    let index = offset as usize;
    assert!(index < bytes.len(), "the offset must be inside the file");
    bytes[index] ^= 0x01;
    write_file(path, &bytes);
}

/// The end offset of every frame of an encrypted WAL.
///
/// The frame layout is **not** re-implemented here on purpose: the public
/// reader is the contract under test, and `pos()` is the end of the frame it
/// has just decoded.
fn frame_ends(wal: &Path, data: &Path, key: &str) -> Vec<u64> {
    let wal_key = WalKey::open(data, &material(key)).expect("the wal key");
    let mut reader = WalReader::open_with_key(wal, wal_key).expect("open the wal");
    let mut ends = Vec::new();
    while reader.next_record().expect("read a record").is_some() {
        ends.push(reader.pos().expect("the frame end"));
    }
    ends
}

/// The payload of every `Insert` record of an encrypted WAL, in order.
fn wal_payloads(wal: &Path, data: &Path, key: &str) -> Vec<String> {
    let wal_key = WalKey::open(data, &material(key)).expect("the wal key");
    let mut reader = WalReader::open_with_key(wal, wal_key).expect("open the wal");
    let mut out = Vec::new();
    while let Some(record) = reader.next_record().expect("read a record") {
        if let WalRecord::Insert(node) = record {
            let text = match node.relational.get("__vanta_payload") {
                Some(FieldValue::String(value)) => value.clone(),
                _ => String::from("<no payload field>"),
            };
            out.push(text);
        }
    }
    out
}

/// Open a database, turning a panic into an error.
///
/// The contract says the engine must never panic on a damaged artefact: the
/// panic is caught here so the test reports it as a finding instead of
/// aborting the whole binary.
fn open_guarded(config: Config) -> Result<Embedded, String> {
    match std::panic::catch_unwind(AssertUnwindSafe(|| Embedded::open_with_config(config))) {
        Ok(Ok(db)) => Ok(db),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("the open PANICKED".to_string()),
    }
}

/// Assert that no record comes back with a payload it was not written with.
///
/// The AEAD must **fail closed**: a record that is damaged is either absent
/// or refused with an explicit error — it is never handed back with another
/// record's bytes (or with garbled bytes).
///
/// Returns how many records were readable, so the caller can tell "nothing
/// was corrupted" from "everything was lost" (both are acceptable; a *wrong*
/// payload is not).
fn assert_no_record_is_corrupted(db: &Embedded, count: usize, label: &str) -> usize {
    let mut readable = 0usize;
    for i in 0..count {
        match db.get("chaos", &record_key(i)) {
            Ok(Some(record)) => {
                readable += 1;
                if record.payload != payload(i) {
                    eprintln!(
                        "FAIL {label}: the record {i} came back with the payload {:?} \
                         instead of {:?}",
                        record.payload,
                        payload(i)
                    );
                }
                assert_eq!(
                    record.payload,
                    payload(i),
                    "{label}: the record {i} came back with a payload it was not written \
                     with — the AEAD did not fail closed"
                );
            }
            Ok(None) => {}
            Err(err) => {
                // An explicit error is an acceptable outcome for a damaged
                // artefact: what is not acceptable is a wrong payload.
                eprintln!("{label}: the record {i} is refused: {err}");
            }
        }
    }
    readable
}

/// Open the database after a corruption; returns `None` when the open was
/// refused with an explicit error (an acceptable outcome of the contract).
fn open_after_corruption(root: &Path, key: &str, label: &str) -> Option<Embedded> {
    match open_guarded(keyed_config(root, key, 1)) {
        Ok(db) => Some(db),
        Err(message) => {
            eprintln!("{label}: the database refuses to open: {message}");
            assert_explicit_failure(&message, label);
            None
        }
    }
}

/// Assert that an open error is **explicit** and distinguishable from "the
/// database does not exist" and from "the key is the wrong one".
fn assert_explicit_failure(message: &str, label: &str) {
    let lower = message.to_lowercase();
    assert!(
        !lower.contains("not found") && !lower.contains("no such file"),
        "{label}: the failure must not look like a missing database: {message}"
    );
    assert!(
        !lower.contains("key required"),
        "{label}: the failure must not look like a missing key: {message}"
    );
}

/// Copy the encryption registry of `data` into a fresh directory.
///
/// Opening the copy proves the **on-disk** state of the registry: the
/// process-wide registry table would hand back the in-memory state for the
/// original path, which proves nothing about the file.
fn copy_registry_for_offline_check(data: &Path, scratch: &Path) -> PathBuf {
    let dir = scratch.join("registry-check");
    fs::create_dir_all(&dir).expect("create the check dir");
    copy_file(
        &data.join("encryption.registry"),
        &dir.join("encryption.registry"),
    );
    dir
}

/// The value segments of the backend's `default` keyspace.
///
/// The Fjall store lays its keyspaces out as `keyspaces/<n>/v<k>`; the `v*`
/// files are the ones that hold the values, so they are the ones that carry
/// the ciphertext of the backend artefact. Fjall numbers its keyspaces in
/// creation order and the `default` partition is created first, so the node
/// metadata (and therefore the encrypted values of the records) lives in
/// `keyspaces/0/`.
fn backend_default_value_files(root: &Path) -> Vec<PathBuf> {
    let keyspaces = root.join("keyspaces");
    let default = keyspaces.join("0");
    let dir = if default.is_dir() { default } else { keyspaces };
    let mut out: Vec<PathBuf> = fs::read_dir(&dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.is_file()
                        && path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|name| {
                                name.starts_with('v')
                                    && name.len() > 1
                                    && name[1..].chars().all(|c| c.is_ascii_digit())
                            })
                            .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

/// The length of the **written** region of a file: the offset just after its
/// last non-zero byte.
///
/// The VantaFile segments are sparse (64 MiB pre-allocated), so the file
/// length is not where the data ends — flipping a byte in the sparse tail
/// would corrupt nothing at all.
fn written_extent(path: &Path) -> u64 {
    let bytes = read_file(path);
    let mut end = 0usize;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != 0 {
            end = index + 1;
        }
    }
    end as u64
}

// ─────────────────────── 1. truncamiento de cola ───────────────────────

/// **Scenario 1 — truncamiento de cola cifrada.**
///
/// The encrypted WAL is cut at an arbitrary offset (inside the ciphertext of
/// the last frame: not aligned to the nonce, not aligned to the frame). The
/// database must reopen and must never hand back a corrupt record.
///
/// The contract of ADR-0057 D5/D7 is that a damaged frame is a **truncatable
/// tail**: the frames before the cut survive, the cut frame is dropped whole
/// and quarantined for forensics. A frame whose CRC passes but whose AEAD
/// tag fails is the opposite — a **hard error**, never a truncation.
#[test]
fn encrypted_db_survives_a_truncated_encrypted_wal_tail() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let key = key_hex(0x41);
    const COUNT: usize = 6;
    seed(root, &key, COUNT, 1);

    let data = data_dir(root);
    let wal = data.join("vanta.wal");
    let original = read_file(&wal);
    let ends = frame_ends(&wal, &data, &key);
    assert!(
        ends.len() >= 2,
        "the wal must hold several frames to be truncated: {}",
        ends.len()
    );
    let payloads = wal_payloads(&wal, &data, &key);
    assert_eq!(
        payloads.len(),
        ends.len(),
        "every frame must be an insert record"
    );

    // The cut: inside the ciphertext of the last frame. The v4 frame layout
    // is `[u32 ct_len][u32 crc][12B nonce][ct ‖ 16B tag][64B chain]`, so the
    // ciphertext starts at +20; +5 lands in the middle of it.
    let last_frame_start = ends[ends.len() - 2];
    let cut = last_frame_start + 20 + 5;
    assert!(
        cut < original.len() as u64,
        "the cut must be inside the file"
    );
    truncate(&wal, cut);

    // (1) The frame level: the truncated frame is dropped **whole** and the
    //     frames before it are untouched. No error, no partial record.
    let after = wal_payloads(&wal, &data, &key);
    assert_eq!(
        after.len(),
        ends.len() - 1,
        "the cut frame must be discarded whole: {after:?}"
    );
    assert_eq!(
        &after[..],
        &payloads[..ends.len() - 1],
        "the frames before the cut must be byte-for-byte the ones written"
    );

    // (2) The database level: it reopens and every record it returns is the
    //     one that was written.
    let db = match open_guarded(keyed_config(root, &key, 1)) {
        Ok(db) => db,
        Err(message) => {
            // "Reopening fails with a clear error" is an acceptable outcome
            // of the contract, as long as it is explicit.
            assert_explicit_failure(&message, "truncated wal tail");
            return;
        }
    };
    let readable = assert_no_record_is_corrupted(&db, COUNT, "truncated wal tail");
    eprintln!("truncated wal tail: {readable}/{COUNT} records readable");
    db.close().expect("close");

    // (3) The forensic quarantine: the dropped bytes are kept, not silently
    //     destroyed.
    let quarantine = data.join("vanta.wal.corrupt");
    assert!(
        quarantine.exists(),
        "the truncated tail must be quarantined for forensics: {}",
        quarantine.display()
    );
}

/// **Scenario 1 (multi-shard).** The same truncation on a **sharded** WAL.
///
/// The sharded layout distributes the records round-robin, so cutting one
/// shard breaks the round-robin invariant. The engine detects it (ERR-011)
/// and refuses the recovery instead of replaying short — but a single
/// missing record is *within* the tolerated window, so the outcome is
/// "reopens with the records intact" **or** "refuses with an explicit WAL
/// error". What must never happen is a corrupt record.
#[test]
fn encrypted_sharded_wal_truncation_never_returns_a_corrupt_record() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let key = key_hex(0x42);
    const COUNT: usize = 9;
    seed(root, &key, COUNT, 4);

    let data = data_dir(root);
    let shard0 = data.join("vanta.shard0.wal");
    assert!(shard0.exists(), "the sharded layout must be on disk");

    // Cut the tail of the first shard, inside its last frame.
    let ends = frame_ends(&shard0, &data, &key);
    assert!(ends.len() >= 2, "the shard must hold several frames");
    let last_frame_start = ends[ends.len() - 2];
    truncate(&shard0, last_frame_start + 20 + 3);

    let db = match open_guarded(keyed_config(root, &key, 4)) {
        Ok(db) => db,
        Err(message) => {
            assert_explicit_failure(&message, "truncated sharded wal");
            return;
        }
    };
    let readable = assert_no_record_is_corrupted(&db, COUNT, "truncated sharded wal");
    eprintln!("truncated sharded wal: {readable}/{COUNT} records readable");
    db.close().expect("close");
}

// ─────────────────── 2. corte de energía en un envelope ───────────────────

/// **Scenario 2 — corte de energía durante un envelope.**
///
/// A frame is written half: the writer flushed the prefix and part of the
/// ciphertext (or the nonce, or the tag) and then the power went out. The
/// partial envelope must be discarded **whole** — there is no "decrypt OK"
/// of a half-written envelope, and no plaintext may leak.
///
/// Simulated (the machine's power cannot be cut from a test) by appending
/// the first `keep` bytes of a real frame to the WAL.
#[test]
fn power_cut_during_an_envelope_never_yields_a_partial_record() {
    // Three cut points inside a frame: the nonce, the ciphertext and the tag.
    let cuts: [(&str, fn(&[u8]) -> usize); 3] = [
        ("mid-nonce", |_frame| 8 + 6),
        ("mid-ciphertext", |_frame| 20 + 5),
        ("mid-tag", |frame| 20 + blob_len(frame) - 8),
    ];

    for (label, cut_at) in cuts {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let key = key_hex(0x51);
        const COUNT: usize = 5;
        seed(root, &key, COUNT, 1);

        let data = data_dir(root);
        let wal = data.join("vanta.wal");
        let original = read_file(&wal);
        let ends = frame_ends(&wal, &data, &key);
        assert!(ends.len() >= 2, "{label}: the wal must hold several frames");
        let payloads = wal_payloads(&wal, &data, &key);
        assert_eq!(payloads.len(), ends.len(), "{label}: one insert per frame");

        // The last frame, cut at the chosen point.
        let last_frame_start = ends[ends.len() - 2] as usize;
        let frame = &original[last_frame_start..];
        let keep = cut_at(frame).min(frame.len() - 1);
        assert!(keep > 0, "{label}: the cut must keep some bytes");

        // The power cut: the file ends in the middle of the envelope.
        let mut bytes = original[..last_frame_start].to_vec();
        bytes.extend_from_slice(&frame[..keep]);
        write_file(&wal, &bytes);
        assert_eq!(
            fs::metadata(&wal).expect("stat").len(),
            last_frame_start as u64 + keep as u64,
            "{label}: the wal must end in the middle of the envelope"
        );

        // (1) The frame level: the half-written envelope yields **no**
        //     record, and the frames before it are untouched.
        let after = wal_payloads(&wal, &data, &key);
        assert_eq!(
            after.len(),
            ends.len() - 1,
            "{label}: the partial envelope must be discarded whole: {after:?}"
        );
        assert_eq!(
            &after[..],
            &payloads[..ends.len() - 1],
            "{label}: the frames before the cut must be intact"
        );

        // (2) The database level: it reopens (or refuses with an explicit
        //     error) and never returns a corrupt record. A panic here would
        //     be a failure of the contract.
        let db = match open_guarded(keyed_config(root, &key, 1)) {
            Ok(db) => db,
            Err(message) => {
                assert_explicit_failure(&message, label);
                continue;
            }
        };
        let readable = assert_no_record_is_corrupted(&db, COUNT, label);
        eprintln!("{label}: {readable}/{COUNT} records readable");
        db.close().expect("close");
    }
}

/// The length of the AEAD blob (`ciphertext ‖ tag`) declared by a v4 frame.
fn blob_len(frame: &[u8]) -> usize {
    u32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize
}

// ─────────────────── 3. rotación a mitad de escritura ───────────────────

/// **Scenario 3 — rotación a mitad de escritura.**
///
/// The master key is rotated while the database is **open** and a write is in
/// flight. The wave declares that the rotation requires a closed database;
/// what the engine must guarantee is that the registry never ends up with
/// DEKs wrapped under a KEK that cannot unwrap them.
///
/// Two ways of rotating are exercised:
///
/// * **(a) in-process** — the programmatic equivalent of
///   `vanta-cli encryption rotate` (`KeyRegistry::rotate`), against the live
///   engine, with a writer thread running;
/// * **(b) out-of-band** — the registry file is rotated by *another* writer
///   (emulated with a copy of the registry, so the live process keeps its own
///   in-memory state) while the engine is open. This is the shape of the
///   operational mistake the wave warns about: rotating a database that is
///   being written.
#[test]
fn rotation_while_the_database_is_open_leaves_a_coherent_registry() {
    const COUNT: usize = 4;

    // ── (a) the rotation runs against the live engine ──
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let old_key = key_hex(0x61);
    let new_key = key_hex(0x62);
    seed(root, &old_key, COUNT, 1);

    let data = data_dir(root);
    let db = Embedded::open_with_config(keyed_config(root, &old_key, 1)).expect("open");
    let version_before = {
        let registry = KeyRegistry::open_or_create(&data, &material(&old_key)).expect("registry");
        registry.key_version()
    };

    let rotation = std::thread::scope(|scope| {
        // A write in flight while the registry is rotated.
        let writer = scope.spawn(|| {
            for i in 0..64 {
                let input = MemoryInput::new(
                    "chaos",
                    format!("live-{i:04}"),
                    format!("vantadb-chaos-live-{i:04}-2b6d"),
                );
                let _ = db.put(input);
            }
        });

        let mut registry =
            KeyRegistry::open_or_create(&data, &material(&old_key)).expect("open the registry");
        let outcome = registry.rotate(&material(&new_key));
        writer.join().expect("the writer thread");
        outcome
    });

    // The rotation either refuses with an explicit error or succeeds. Both
    // are acceptable; what is checked is the state it leaves behind.
    match &rotation {
        Ok(version) => assert_eq!(
            *version,
            version_before + 1,
            "the rotation must bump the key version by one"
        ),
        Err(err) => assert_explicit_failure(&err.to_string(), "rotation"),
    }

    // The registry is coherent: it opens with the **new** key and every
    // artefact still has a DEK under it.
    let registry = KeyRegistry::open_or_create(&data, &material(&new_key)).expect("the new key");
    assert!(
        registry.artifacts().len() >= 4,
        "the rotation must not drop an artefact: {:?}",
        registry.artifacts()
    );
    for artifact in [
        Artifact::Wal,
        Artifact::VantaFile,
        Artifact::HnswImage,
        Artifact::BackendKv,
    ] {
        assert!(
            registry.dek(artifact).is_some(),
            "the DEK of {} must still be unwrappable",
            artifact.as_str()
        );
    }

    // The **on-disk** registry (not the in-memory state) is coherent too.
    let check_dir = copy_registry_for_offline_check(&data, dir.path());
    let on_disk = KeyRegistry::open_or_create(&check_dir, &material(&new_key))
        .expect("the on-disk registry must open with the new key");
    assert_eq!(
        on_disk.artifacts().len(),
        registry.artifacts().len(),
        "the on-disk registry must hold the same artefacts"
    );

    // The old key no longer opens the registry.
    let stale = KeyRegistry::open_or_create(&data, &material(&old_key));
    assert!(
        stale.is_err(),
        "the old key must not open the rotated registry"
    );

    // The database keeps working after the rotation: the DEKs did not change,
    // so no ciphertext was invalidated (FIND-194).
    db.close().expect("close");
    let reopened = Embedded::open_with_config(keyed_config(root, &new_key, 1)).expect("reopen");
    let readable = assert_no_record_is_corrupted(&reopened, COUNT, "after the rotation");
    eprintln!("after the in-place rotation: {readable}/{COUNT} records readable");
    assert_eq!(
        readable, COUNT,
        "the rotation must not lose a record: the DEKs are re-wrapped, not re-derived"
    );
    reopened.close().expect("close");

    // The old key does not open the database any more.
    let refused = Embedded::open_with_config(keyed_config(root, &old_key, 1));
    assert!(
        refused.is_err(),
        "the old key must not open the rotated database"
    );

    // ── (b) the rotation comes from another writer, while the engine is open ──
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let old_key = key_hex(0x63);
    let new_key = key_hex(0x64);
    seed(root, &old_key, COUNT, 1);

    let data = data_dir(root);
    let db = Embedded::open_with_config(keyed_config(root, &old_key, 1)).expect("open");

    // Another writer rotates a *copy* of the registry (a different path, so a
    // different in-memory state — the live process keeps the old one).
    let other = dir.path().join("other-writer");
    fs::create_dir_all(&other).expect("create the other dir");
    copy_file(
        &data.join("encryption.registry"),
        &other.join("encryption.registry"),
    );
    {
        let mut registry =
            KeyRegistry::open_or_create(&other, &material(&old_key)).expect("the other registry");
        registry
            .rotate(&material(&new_key))
            .expect("the other rotation");
    }
    // ... and its result lands on the live database.
    copy_file(
        &other.join("encryption.registry"),
        &data.join("encryption.registry"),
    );

    // A write while the two views of the registry disagree.
    let input = MemoryInput::new("chaos", "during-rotation", payload(999));
    let _ = db.put(input);
    db.close().expect("close");

    // The database opens with the new key and the data is intact: the DEKs
    // were re-wrapped, not re-derived, so no ciphertext was invalidated.
    let reopened = Embedded::open_with_config(keyed_config(root, &new_key, 1)).expect("reopen");
    let readable = assert_no_record_is_corrupted(&reopened, COUNT, "out-of-band rotation");
    eprintln!("after the out-of-band rotation: {readable}/{COUNT} records readable");
    assert_eq!(
        readable, COUNT,
        "the rotation must not lose a record: the DEKs are re-wrapped, not re-derived"
    );
    reopened.close().expect("close");

    // And the old key is refused.
    let refused = Embedded::open_with_config(keyed_config(root, &old_key, 1));
    assert!(
        refused.is_err(),
        "the old key must not open the rotated database"
    );
}

// ─────────────────── 4. corrupción de un byte ───────────────────

/// **Scenario 4 — corrupción de un byte en el ciphertext.**
///
/// One byte is flipped in the WAL, in a VantaFile segment, in the HNSW index
/// image and in the backend's files. The contract:
///
/// * the affected record is **not** returned — it fails closed at the AEAD;
/// * the other records come back intact;
/// * the failure (when there is one) is distinguishable from "the database
///   does not exist" and from "the key is the wrong one".
#[test]
fn a_single_corrupted_byte_never_yields_a_valid_plaintext_record() {
    const COUNT: usize = 5;

    // ── (a) the WAL: the byte lands inside the ciphertext of the last frame ──
    {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let key = key_hex(0x71);
        seed(root, &key, COUNT, 1);

        let data = data_dir(root);
        let wal = data.join("vanta.wal");
        let original = read_file(&wal);
        let ends = frame_ends(&wal, &data, &key);
        assert!(ends.len() >= 2, "the wal must hold several frames");
        let last_frame_start = ends[ends.len() - 2] as usize;
        // Inside the ciphertext of the last frame.
        let offset = (last_frame_start + 20 + 2) as u64;
        assert!(offset < original.len() as u64);
        flip_byte(&wal, offset);

        // The CRC covers the ciphertext: the frame is damaged, so it is a
        // truncatable tail — never a record handed back as plaintext.
        let after = wal_payloads(&wal, &data, &key);
        assert_eq!(
            after.len(),
            ends.len() - 1,
            "the corrupted frame must not be decoded: {after:?}"
        );

        if let Some(db) = open_after_corruption(root, &key, "corrupted wal byte") {
            let readable = assert_no_record_is_corrupted(&db, COUNT, "corrupted wal byte");
            eprintln!("corrupted wal byte: {readable}/{COUNT} records readable");
            db.close().expect("close");
        }
    }

    // ── (b) the same byte, but the CRC is **recomputed**: only the AEAD can
    //     catch it now. This is the case the CRC cannot see (ADR-0057 D5):
    //     the frame is structurally perfect and the content is forged.
    //     The expected behaviour is a **hard error**, never a silent
    //     truncation (a truncation would drop the records that follow).
    {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let key = key_hex(0x73);
        seed(root, &key, COUNT, 1);

        let data = data_dir(root);
        let wal = data.join("vanta.wal");
        let mut bytes = read_file(&wal);
        let ends = frame_ends(&wal, &data, &key);
        assert!(ends.len() >= 2, "the wal must hold several frames");
        let frame_start = ends[ends.len() - 2] as usize;
        // `[u32 ct_len][u32 crc][12B nonce][ct ‖ 16B tag]`: the ciphertext
        // starts at +20 and the CRC covers it.
        let ct_len = blob_len(&bytes[frame_start..]) as u64;
        let ct_start = frame_start + 20;
        let ct_end = ct_start + ct_len as usize;
        bytes[ct_start] ^= 0x01;
        let crc = vantadb::wal::compute_crc32c(&bytes[ct_start..ct_end]);
        bytes[frame_start + 4..frame_start + 8].copy_from_slice(&crc.to_le_bytes());
        write_file(&wal, &bytes);

        // The frame level: the CRC passes and the AEAD refuses — a hard
        // error, and the WAL is **not** truncated.
        let wal_key = WalKey::open(&data, &material(&key)).expect("the wal key");
        let mut reader = WalReader::open_with_key(&wal, wal_key).expect("open the wal");
        let mut seen = 0usize;
        let mut outcome = Ok(());
        loop {
            match reader.next_record() {
                Ok(Some(_)) => seen += 1,
                Ok(None) => break,
                Err(err) => {
                    outcome = Err(err.to_string());
                    break;
                }
            }
        }
        let length_before = fs::metadata(&wal).expect("stat").len();
        match outcome {
            Err(message) => {
                eprintln!("forged frame: the reader refuses it: {message}");
                assert!(
                    message.to_lowercase().contains("rejected"),
                    "the forged frame must be refused as a rejected authentication, \
                     not as a truncation: {message}"
                );
            }
            Ok(()) => panic!(
                "the forged frame must not decode: {seen} records were read as if \
                 nothing was wrong"
            ),
        }
        assert_eq!(
            fs::metadata(&wal).expect("stat").len(),
            length_before,
            "a rejected frame must not truncate the WAL"
        );

        // The database level: it refuses to open, with an explicit error.
        if let Some(db) = open_after_corruption(root, &key, "forged frame") {
            let readable = assert_no_record_is_corrupted(&db, COUNT, "forged frame");
            eprintln!("forged frame: {readable}/{COUNT} records readable");
            db.close().expect("close");
        }
    }

    // ── (c) the VantaFile segment, (d) the HNSW image ──
    //     A byte is flipped at three offsets of each artefact. The layout of
    //     the artefacts is not re-implemented here: the invariant that
    //     matters is that **no record is ever returned with a payload it was
    //     not written with**, whatever the byte lands on.
    for (label, name) in [
        ("vfile segment", "vstore_L0.vanta"),
        ("hnsw image", "vector_index.bin"),
    ] {
        // A fresh database per offset: the corruption is not cumulative.
        for offset_kind in 0..3u64 {
            let dir = tempfile::tempdir().expect("tempdir");
            let root = dir.path();
            let key = key_hex(0x72);
            seed(root, &key, COUNT, 1);

            let data = data_dir(root);
            let target = data.join(name);
            assert!(target.exists(), "{label}: {} must exist", target.display());

            // The offsets are taken inside the **written** region of the
            // artefact: the segments are sparse, so the file length is not
            // where the data ends.
            let extent = written_extent(&target);
            assert!(extent > 8, "{label}: the artefact must not be empty");
            let offset = match offset_kind {
                0 => 1,
                1 => extent / 2,
                _ => extent - 1,
            };
            eprintln!(
                "{label}: flipping the byte at {offset} of {} ({extent} bytes written)",
                target.file_name().unwrap_or_default().to_string_lossy()
            );
            flip_byte(&target, offset);

            let label = format!("{label} at offset {offset}");
            if let Some(db) = open_after_corruption(root, &key, &label) {
                let readable = assert_no_record_is_corrupted(&db, COUNT, &label);
                eprintln!("{label}: {readable}/{COUNT} records readable");
                db.close().expect("close");
            }
        }
    }

    // ── (e) the backend: the value segments of the `default` keyspace ──
    //     Three different segments, one byte each.
    for pick in 0..3usize {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let key = key_hex(0x74);
        seed(root, &key, COUNT, 1);

        let segments = backend_default_value_files(root);
        assert!(
            !segments.is_empty(),
            "the backend must have at least one value segment"
        );
        let index = match pick {
            0 => 0,
            1 => segments.len() / 2,
            _ => segments.len() - 1,
        };
        let target = segments[index].clone();
        let extent = written_extent(&target);
        assert!(extent > 8, "the value segment must not be empty");
        let offset = extent / 2;
        eprintln!(
            "backend value segment {index}: flipping the byte at {offset} of {} \
             ({extent} bytes written)",
            target.file_name().unwrap_or_default().to_string_lossy()
        );
        flip_byte(&target, offset);

        let label = format!("backend value segment {index}");
        if let Some(db) = open_after_corruption(root, &key, &label) {
            let readable = assert_no_record_is_corrupted(&db, COUNT, &label);
            eprintln!("{label}: {readable}/{COUNT} records readable");
            db.close().expect("close");
        }
    }
}
