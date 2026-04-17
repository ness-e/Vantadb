//! Structured API v2 Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::{ExecutionResult, Executor};
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn structured_api_v2_certification() {
    let mut harness = VantaHarness::new("API LAYER (STRUCTURED V2)");

    harness.execute("Integration: Relational ID Capture", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
            let executor = Executor::new(&storage);

            TerminalReporter::sub_step("Inserting nodes S1 and S2 via hybrid syntax...");
            executor
                .execute_hybrid("(INSERT :node {:label \"S1\"})")
                .await
                .unwrap();
            executor
                .execute_hybrid("(INSERT :node {:label \"S2\"})")
                .await
                .unwrap();

            let (s1_id, s2_id);
            {
                let cache = storage.volatile_cache.read();
                s1_id = *cache
                    .iter()
                    .find(|(_, n)| n.get_field("label").unwrap().as_str() == Some("S1"))
                    .unwrap()
                    .0;
                s2_id = *cache
                    .iter()
                    .find(|(_, n)| n.get_field("label").unwrap().as_str() == Some("S2"))
                    .unwrap()
                    .0;
            }

            TerminalReporter::sub_step(&format!("Establishing relation {} -> {}...", s1_id, s2_id));
            let relate_query = format!(
                "RELATE NODE#{} --\"test_rel\"--> NODE#{} WEIGHT 0.8",
                s1_id, s2_id
            );
            let res = executor.execute_hybrid(&relate_query).await.unwrap();

            if let ExecutionResult::Write { node_id, .. } = res {
                assert_eq!(node_id, Some(s1_id));
            }
            TerminalReporter::success("Relational result-ID alignment verified.");
        });
    });

    harness.execute("Integration: Message-to-Thread Dispatch", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
            let executor = Executor::new(&storage);

            executor
                .execute_hybrid("(INSERT :node {:type \"Thread\" :id 999})")
                .await
                .unwrap();

            TerminalReporter::sub_step("Dispatching message to THREAD#999...");
            let msg_query = "INSERT MESSAGE USER \"Hola Mundo\" TO THREAD#999";
            let msg_res = executor.execute_hybrid(msg_query).await.unwrap();

            if let ExecutionResult::Write { node_id, .. } = msg_res {
                assert!(node_id.is_some(), "Message ID was not returned");
            }
            TerminalReporter::success("Structured message routing validated.");
        });
    });
}
