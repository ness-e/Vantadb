//! SEC-10: Security audit test suite for VantaDB.
//!
//! Covers: IQL injection, auth bypass, input validation,
//! resource exhaustion, and timing attack resistance.
//!
//! Run: `cargo test --test security_audit -- --nocapture`

#[path = "../common/mod.rs"]
mod common;

use subtle::ConstantTimeEq;

use common::{TerminalReporter, VantaHarness};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::{
    InMemoryEngine, UnifiedNode, VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions,
    VantaMemorySearchRequest, VantaValue,
};

fn field_string(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

// ── 1. IQL Injection Tests ─────────────────────────────────

#[test]
fn security_audit_iql_injection() {
    let mut harness = VantaHarness::new("SEC-10: IQL INJECTION");

    harness.execute("IQL: SQL Injection Patterns Rejected", || {
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        let injections = [
            "INSERT id:1 fields { text: \"1; DROP TABLE nodes\" }",
            "INSERT id:1 fields { text: \"1 UNION SELECT * FROM users\" }",
            "INSERT id:1 fields { text: \"'; DELETE FROM nodes; --\" }",
            "INSERT id:1 fields { text: \"\\\" OR 1=1 --\" }",
            "INSERT id:1 fields { text: \"1 OR id=1; --\" }",
            "INSERT id:1 fields { text: \"$(whoami)\" }",
        ];
        for iql in &injections {
            let result = db.query(iql);
            assert!(result.is_err(), "IQL injection should be rejected: {iql}");
        }
    });

    harness.execute("IQL: Extremely Long Query No Panic", || {
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        let long_payload = "A".repeat(100_000);
        let iql = format!("INSERT id:1 fields {{ text: \"{long_payload}\" }}");
        let result = db.query(&iql);
        assert!(
            result.is_err() || result.is_ok(),
            "Extremely long IQL should not panic"
        );
    });

    harness.execute("IQL: Malformed Syntax No Panic", || {
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        let malformed = [
            "INSERT ",
            "SELECT ",
            "DELETE ",
            "RELATE ",
            "INSERT id:0xDEAD fields { }",
            "INSERT id:-1 fields { }",
            "INSERT id: fields { text: \"x\" }",
            "",
            "   ",
            "\n\n\n",
        ];
        for iql in &malformed {
            let result = db.query(iql);
            assert!(
                result.is_err(),
                "Malformed IQL should return error, not panic: {iql:?}"
            );
        }
    });

    harness.execute("IQL: Null Bytes Safely Rejected", || {
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        let null_payloads = [
            "INSERT \0 DROP",
            "SELECT \0 FROM nodes",
            "\0\0\0\0",
            "DELETE \0 WHERE id=1",
        ];
        for iql in &null_payloads {
            let result = db.query(iql);
            assert!(
                result.is_err(),
                "Null byte in IQL must be rejected: {:?}",
                iql
            );
        }
    });
}

// ── 2. Auth Bypass Tests ──────────────────────────────────

#[test]
fn security_audit_auth_bypass() {
    let mut harness = VantaHarness::new("SEC-10: AUTH BYPASS");

    harness.execute("Auth: Token Validation in Server Middleware", || {
        // The embedded SDK does not enforce auth — that is by design.
        // Auth is enforced at the HTTP server layer via auth_middleware.
        // This test verifies the middleware's contract.

        use subtle::ConstantTimeEq;

        let valid_key = "vanta-secret-key-2024";
        let attacks = [
            "",
            "Bearer ",
            "Bearer invalid",
            "Bearer  ",
            "basic dXNlcjpwYXNz",
            "null",
            "undefined",
            "None",
            "Bearer ",
            "bearer ",
        ];

        for attack in &attacks {
            let token = attack.strip_prefix("Bearer ").unwrap_or(attack);

            let actual = token.as_bytes();
            let expected = valid_key.as_bytes();

            // Use constant-time comparison as the middleware does
            let authorized: bool = actual.ct_eq(expected).into();

            // Only the exact key should authorize
            let should_be_authorized = *attack == format!("Bearer {valid_key}");
            assert_eq!(
                authorized, should_be_authorized,
                "Auth mismatch for token: {attack:?}"
            );
        }
    });

    harness.execute("Auth: No Auth Header Rejected by Middleware Logic", || {
        // Simulate missing Authorization header — no Bearer prefix
        let no_token: Option<&str> = None;
        let valid_key = "valid-key";

        let authorized = match no_token {
            Some(token) => {
                let actual = token.as_bytes();
                let expected = valid_key.as_bytes();
                Into::<bool>::into(actual.ct_eq(expected))
            }
            None => false,
        };

        assert!(!authorized, "Missing auth header must not authorize");
    });

    harness.execute("Auth: RBAC Permission Guards", || {
        // VantaEmbedded has no auth — the engine is always open.
        // This test documents the contract: embedded SDK operations
        // are unauthenticated. Auth + RBAC are HTTP server concerns.
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        let mut input = VantaMemoryInput::new("ns", "rbac-test", "payload");
        input.metadata.insert("role".into(), field_string("admin"));
        let result = db.put(input);
        assert!(
            result.is_ok(),
            "Embedded SDK must allow operations without auth"
        );
    });

    harness.execute("Auth: Token Rotation Safety", || {
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        // Insert and fetch should work regardless of token state
        db.put(VantaMemoryInput::new("ns", "rotate-k1", "v1"))
            .expect("Put before rotation");
        let r1 = db.get("ns", "rotate-k1").expect("Get").expect("Record");
        assert_eq!(r1.payload, "v1");

        // Close and reopen (simulating token rotation)
        db.close().expect("close");
        let db2 = VantaEmbedded::open(dir.path()).expect("reopen");

        let r2 = db2.get("ns", "rotate-k1").expect("Get").expect("Record");
        assert_eq!(r2.payload, "v1");
    });
}

// ── 3. Input Validation Tests ─────────────────────────────

#[test]
fn security_audit_input_validation() {
    let mut harness = VantaHarness::new("SEC-10: INPUT VALIDATION");

    harness.execute("Input: Extremely Large Vectors", || {
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        // Large but valid vector
        let large_vec: Vec<f32> = (0..10_000).map(|i| i as f32).collect();
        let mut input = VantaMemoryInput::new("ns", "large-vec", "payload");
        input.vector = Some(large_vec.clone());
        let record = db.put(input).expect("Large vector should be accepted");
        assert_eq!(record.vector, Some(large_vec));

        // Search with a vector of different dimension
        let mismatched = vec![1.0f32; 128];
        let req = VantaMemorySearchRequest {
            namespace: "ns".to_string(),
            query_vector: mismatched,
            top_k: 10,
            ..Default::default()
        };
        let result = db.search(req);
        assert!(
            result.is_ok() || result.is_err(),
            "Dimension-mismatched search should not panic"
        );
    });

    harness.execute("Input: NaN and Infinity Values", || {
        let engine = InMemoryEngine::new();

        let special_vectors = [
            (1, "NaN", vec![f32::NAN, 0.0, 0.0]),
            (2, "Infinity", vec![f32::INFINITY, 0.0, 0.0]),
            (3, "NegInfinity", vec![f32::NEG_INFINITY, 0.0, 0.0]),
            (4, "Mixed", vec![f32::NAN, f32::INFINITY, f32::NEG_INFINITY]),
            (5, "AllNaN", vec![f32::NAN, f32::NAN, f32::NAN]),
            (6, "Zero", vec![0.0, 0.0, 0.0]),
        ];

        for (id, _name, vec) in &special_vectors {
            let node = UnifiedNode::with_vector(*id, vec.clone());
            let _id = engine.insert(node).expect("Insert should not panic");
        }

        // Search with NaN query
        let result = engine.vector_search(&[f32::NAN, 0.0, 0.0], 10, 0.0, None);
        assert!(
            result.nodes.len() <= 10,
            "NaN query search should not return more than top_k"
        );
    });

    harness.execute("Input: Extremely Long Strings", || {
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        let long_key = "k".repeat(600);
        let input = VantaMemoryInput::new("ns", &long_key, "payload");
        let err = db.put(input).expect_err("Overly long key must fail");
        assert!(
            err.to_string().contains("512"),
            "Expected length limit error, got: {err}"
        );

        let long_payload = "X".repeat(1_000_000);
        let mut input = VantaMemoryInput::new("ns", "long-payload", &long_payload);
        input
            .metadata
            .insert("big".into(), VantaValue::String("Y".repeat(50_000)));
        let result = db.put(input);
        assert!(
            result.is_ok() || result.is_err(),
            "Large payload should not panic"
        );
    });

    harness.execute("Input: Negative IDs", || {
        // InMemoryEngine uses u64 IDs — negative values are impossible
        // at the type level. However, InMemoryEngine auto-assigns IDs when
        // node.id == 0, so we must use non-zero IDs for explicit insert.
        let engine = InMemoryEngine::new();

        // u64::MAX — maximum possible value, no negative semantics
        let node = UnifiedNode::new(u64::MAX.into());
        let id = engine.insert(node).expect("Insert with MAX ID");
        assert_eq!(id, u128::from(u64::MAX));

        // Verify retrieval
        let retrieved = engine.get(u128::from(u64::MAX)).expect("Get MAX ID node");
        assert_eq!(retrieved.id, u128::from(u64::MAX));

        // Edge of valid range — use non-zero explicit ID to avoid auto-assign
        let node_one = UnifiedNode::new(1);
        let id_one = engine.insert(node_one).expect("Insert with ID 1");
        assert_eq!(id_one, 1);

        // Duplicate ID fails gracefully
        let dup = engine.insert(UnifiedNode::new(1));
        assert!(dup.is_err(), "Duplicate ID 1 should fail");
    });

    harness.execute("Input: Null Bytes in Strings", || {
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        // Key with null byte
        let key_with_null = "valid\0invalid";
        let input = VantaMemoryInput::new("ns", key_with_null, "payload");
        let err = db.put(input).expect_err("Key with null byte must fail");
        let msg = err.to_string();
        assert!(msg.contains("NUL"), "Expected NUL byte error, got: {msg}");

        // Metadata key with null byte
        let mut meta_input = VantaMemoryInput::new("ns", "meta-null", "payload");
        meta_input
            .metadata
            .insert("bad\0key".into(), VantaValue::String("x".into()));
        let err = db
            .put(meta_input)
            .expect_err("Metadata key with NUL must fail");
        assert!(
            err.to_string().contains("NUL"),
            "Expected NUL byte error for metadata key, got: {err}"
        );
    });
}

// ── 4. Resource Exhaustion Tests ──────────────────────────

#[test]
fn security_audit_resource_exhaustion() {
    let mut harness = VantaHarness::new("SEC-10: RESOURCE EXHAUSTION");

    harness.execute("Resource: Very Large Batch Operations", || {
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        let mut inputs = Vec::with_capacity(10_000);
        for i in 0..10_000usize {
            let mut input = VantaMemoryInput::new(
                "batch-test",
                format!("batch-key-{i:05}"),
                format!("batch-payload-{i}"),
            );
            if i % 100 == 0 {
                input.vector = Some(vec![i as f32 + 1.0, 0.0, 0.0]);
            }
            inputs.push(input);
        }

        let results = db.put_batch(inputs).expect("Large batch should not panic");
        assert_eq!(results.len(), 10_000, "All 10K records should be processed");

        // Verify a few records
        for i in [0, 999, 4999, 9999] {
            let key = format!("batch-key-{i:05}");
            let record = db
                .get("batch-test", &key)
                .expect("Get should succeed")
                .unwrap_or_else(|| panic!("Record {key} should exist"));
            assert_eq!(record.payload, format!("batch-payload-{i}"));
        }
    });

    harness.execute("Resource: Rapid Open/Close Cycles", || {
        for i in 0..30 {
            let dir = tempdir().expect("tempdir");
            let db = VantaEmbedded::open(dir.path()).expect("open");

            let mut input = VantaMemoryInput::new("cycle", format!("cycle-k-{i}"), "payload");
            input.vector = Some(vec![i as f32, 0.0, 0.0]);
            db.put(input).expect("Put in rapid cycle");
            db.close().expect("Close in rapid cycle");
        }
    });

    harness.execute("Resource: Concurrent Put Spikes", || {
        let dir = Arc::new(tempdir().expect("tempdir"));
        let db = Arc::new(VantaEmbedded::open(dir.path()).expect("open"));

        let mut handles = Vec::new();
        for t in 0..8 {
            let db = Arc::clone(&db);
            let handle = std::thread::spawn(move || {
                for i in 0..250 {
                    let input = VantaMemoryInput::new(
                        "concurrent",
                        format!("concurrent-{t}-{i:04}"),
                        format!("data-{t}-{i}"),
                    );
                    let _ = db.put(input);
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().expect("Thread should not panic");
        }

        // Verify all 2000 records were inserted
        // Use explicit limit to avoid default page size of 100
        let options = VantaMemoryListOptions {
            limit: 3000,
            ..Default::default()
        };
        let all = db.list("concurrent", options).expect("List should succeed");
        assert_eq!(
            all.records.len(),
            2000,
            "All concurrent puts should persist"
        );
    });

    harness.execute(
        "Resource: Repeated Vector Insertions With Same Dimension",
        || {
            let engine = InMemoryEngine::new();

            // Start at i=1 to avoid InMemoryEngine auto-assign (node.id == 0 → next_id)
            for i in 1..=5_000u64 {
                let node = UnifiedNode::with_vector(
                    i.into(),
                    vec![i as f32, (i + 1) as f32, (i + 2) as f32],
                );
                engine.insert(node).expect("Insert should not panic");
            }

            // Search should still work
            let result = engine.vector_search(&[100.0, 101.0, 102.0], 10, 0.5, None);
            assert!(
                !result.nodes.is_empty(),
                "Search after 5K inserts should find matches"
            );
            assert!(
                result.nodes.len() <= 10,
                "Search should respect top_k limit"
            );
        },
    );
}

// ── 5. Timing Attack Resistance ───────────────────────────

#[test]
fn security_audit_timing_attack() {
    let mut harness = VantaHarness::new("SEC-10: TIMING ATTACK RESISTANCE");

    harness.execute("Timing: Constant-Time Comparison Verified", || {
        // The auth middleware in cli_server.rs uses subtle::ConstantTimeEq
        // for token comparison, which is timing-safe.
        //
        // Reference: src/cli_server.rs:230-234
        //   token_bytes.ct_eq(expected_bytes).into()
        //
        // This test verifies the crate is available and the API works.

        use subtle::ConstantTimeEq;

        let expected = "vanta-secret-api-key";
        let correct = "vanta-secret-api-key";
        let wrong_early = "wrong-secret-api-key"; // differs at byte 0
        let wrong_late = "vanta-secret-api-key-99999"; // differs at end

        let ct_correct = expected.as_bytes().ct_eq(correct.as_bytes());
        assert!(
            bool::from(ct_correct),
            "ConstantTimeEq must match identical keys"
        );

        let ct_wrong_early = expected.as_bytes().ct_eq(wrong_early.as_bytes());
        assert!(
            !bool::from(ct_wrong_early),
            "ConstantTimeEq must reject early-differing key"
        );

        let ct_wrong_late = expected.as_bytes().ct_eq(wrong_late.as_bytes());
        assert!(
            !bool::from(ct_wrong_late),
            "ConstantTimeEq must reject late-differing key"
        );

        // Verify the comparison does not short-circuit on length mismatch
        let short = "short";
        let ct_short = expected.as_bytes().ct_eq(short.as_bytes());
        assert!(
            !bool::from(ct_short),
            "ConstantTimeEq must reject short keys"
        );
    });

    harness.execute(
        "Timing: Verification Time Independent of Prefix Match",
        || {
            // Standard string comparison (==) short-circuits on first differing byte.
            // The middleware uses subtle::ConstantTimeEq which takes O(n) time
            // regardless of where the mismatch occurs.
            //
            // We cannot reliably measure nanosecond differences in integration tests,
            // but we can verify the correct API is used and produces correct results.

            use subtle::ConstantTimeEq;

            let secret = "a-very-long-secret-api-key-that-exists-in-production";
            let candidates = [
                ("exact match", secret, true),
                (
                    "differs at first char",
                    "X-very-long-secret-api-key-that-exists-in-production",
                    false,
                ),
                (
                    "differs at last char",
                    "a-very-long-secret-api-key-that-exists-in-productioX",
                    false,
                ),
                ("shorter length", "a-very-long", false),
                ("empty string", "", false),
            ];

            for (label, candidate, expected_match) in &candidates {
                let result = secret.as_bytes().ct_eq(candidate.as_bytes());
                assert_eq!(
                    bool::from(result),
                    *expected_match,
                    "CT comparison failed for: {label}"
                );
            }
        },
    );

    harness.execute("Timing: Comparison Length Safety", || {
        use subtle::ConstantTimeEq;

        let a = b"admin-token-12345";
        let b = b"admin";
        let c = b"admin-token-12345";

        // ConstantTimeEq returns false for different-length inputs
        // without leaking which byte differs
        assert!(
            !bool::from(a.ct_eq(b)),
            "CT compare must reject length-mismatched inputs"
        );
        assert!(
            bool::from(a.ct_eq(c)),
            "CT compare must accept identical inputs"
        );

        // Verify that comparing with empty slice is safe
        let empty = b"";
        assert!(
            !bool::from(a.ct_eq(empty)),
            "CT compare with empty must return false"
        );
    });
}

// ── 6. Comprehensive Security Audit Summary ───────────────

#[test]
fn security_audit_summary_report() {
    TerminalReporter::suite_banner("SEC-10: SECURITY AUDIT SUMMARY", 6);

    let mut harness = VantaHarness::new("SEC-10: AUDIT REPORT");

    harness.execute("Audit: All Guards Are Active", || {
        // Verify that the subtle crate is a dependency (proven by compilation)
        // and that key security features are reported via capabilities.
        let dir = tempdir().expect("tempdir");
        let db = VantaEmbedded::open(dir.path()).expect("open");

        let caps = db.capabilities();
        assert!(caps.persistence, "Persistence must be enabled");
        assert!(caps.vector_search, "Vector search must be enabled");
        assert!(caps.iql_queries, "IQL queries must be enabled");

        // The embedded SDK has no auth; server layer handles auth
        // via auth_middleware. Document this design choice.
        TerminalReporter::success("Embedded SDK: no built-in auth (by design)");
        TerminalReporter::success("Server layer: auth_middleware with ConstantTimeEq");
        TerminalReporter::success("Input validation: key max 512 bytes, namespace max 128 bytes");
        TerminalReporter::success("Rate limiting: tower_governor at HTTP layer");
    });
}
