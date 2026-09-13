#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Structured API v2 Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::{ExecutionResult, Executor};
use vantadb::storage::StorageEngine;

#[test]
fn structured_api_v2_certification() {
    let mut harness = VantaHarness::new("API LAYER (STRUCTURED V2)");

    harness.execute("Integration: Relational ID Capture", || {
        let dir = tempdir().unwrap();
        let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
        let executor = Executor::new(&storage);

        TerminalReporter::sub_step("Inserting nodes S1 and S2 via hybrid syntax...");
        // C2S3b: ids come from the public Write result (the volatile cache is
        // now sealed inside `CacheLayer`, no longer `pub` on `StorageEngine`).
        let s1_id = match executor
            .execute_hybrid("INSERT NODE#1 TYPE node { label: \"S1\" }")
            .unwrap()
        {
            ExecutionResult::Write {
                node_id: Some(id), ..
            } => id,
            _ => panic!("S1 insert did not return a node id"),
        };
        let s2_id = match executor
            .execute_hybrid("INSERT NODE#2 TYPE node { label: \"S2\" }")
            .unwrap()
        {
            ExecutionResult::Write {
                node_id: Some(id), ..
            } => id,
            _ => panic!("S2 insert did not return a node id"),
        };

        TerminalReporter::sub_step(&format!("Establishing relation {} -> {}...", s1_id, s2_id));
        let relate_query = format!(
            "RELATE NODE#{} --\"test_rel\"--> NODE#{} WEIGHT 0.8",
            s1_id, s2_id
        );
        let res = executor.execute_hybrid(&relate_query).unwrap();

        if let ExecutionResult::Write { node_id, .. } = res {
            assert_eq!(node_id, Some(s1_id));
        }
        TerminalReporter::success("Relational result-ID alignment verified.");
    });
}

#[test]
#[ignore = "Requires external Ollama LLM service running"]
fn ollama_integration() {
    let mut harness = VantaHarness::new("LLM INTEGRATION (OLLAMA)");

    harness.execute("Integration: Message-to-Thread Dispatch", || {
        let dir = tempdir().unwrap();
        let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
        let executor = Executor::new(&storage);

        executor
            .execute_hybrid("INSERT NODE#999 TYPE Thread { type: \"Thread\" }")
            .unwrap();

        TerminalReporter::sub_step("Dispatching message to THREAD#999...");
        let msg_query = "INSERT MESSAGE USER \"Hola Mundo\" TO THREAD#999";
        let msg_res = executor.execute_hybrid(msg_query).unwrap();

        if let ExecutionResult::Write { node_id, .. } = msg_res {
            assert!(node_id.is_some(), "Message ID was not returned");
        }
        TerminalReporter::success("Structured message routing validated.");
    });
}
