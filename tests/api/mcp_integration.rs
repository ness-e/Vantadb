//! MCP Protocol Integration Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{VantaHarness, TerminalReporter};
use serde_json::json;
use vantadb::api::mcp::{handle_initialize, handle_tools_call, handle_tools_list};
use vantadb::executor::Executor;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn mcp_protocol_certification() {
    let mut harness = VantaHarness::new("API LAYER (MCP PROTOCOL)");

    harness.execute("Protocol: Handshake & Identity (2024-11-05)", || {
        let init_res = handle_initialize().expect("Initialization failed");
        assert_eq!(init_res["protocolVersion"], "2024-11-05");
        assert_eq!(init_res["serverInfo"]["name"], "connectomedb");
        
        let list_res = handle_tools_list().expect("Tools listing failed");
        let tools = list_res["tools"].as_array().expect("Tools must be an array");
        assert!(tools.iter().any(|t| t["name"] == "query_lisp"));
        
        TerminalReporter::success("MCP handshake and tools definition verified.");
    });

    harness.execute("Protocol: Tool Execution & State Mutability", || {
        futures::executor::block_on(async {
            let temp_dir = tempfile::tempdir().unwrap();
            let storage = StorageEngine::open(temp_dir.path().to_str().unwrap()).unwrap();
            let executor = Executor::new(&storage);

            TerminalReporter::sub_step("Testing get_node_neighbors tool...");
            let mut node = UnifiedNode::new(100);
            node.confidence_score = 0.99;
            storage.insert(&node).unwrap();

            let params = Some(json!({
                "name": "get_node_neighbors",
                "arguments": { "node_id": 100 }
            }));

            let tool_res = handle_tools_call(&params, &executor, &storage).await.expect("Tool call failed");
            let text = tool_res["content"][0]["text"].as_str().unwrap();
            assert!(text.contains("\"confidence_score\":0.99"));

            TerminalReporter::sub_step("Testing query_lisp tool (Insertion)...");
            let lisp_params = Some(json!({
                "name": "query_lisp",
                "arguments": { "query": "(INSERT :node {:label \"MCP_TEST\"})" }
            }));
            let lisp_res = handle_tools_call(&lisp_params, &executor, &storage).await.expect("Lisp execution failed");
            assert!(lisp_res["content"][0]["text"].as_str().unwrap().contains("affected_nodes"));

            TerminalReporter::success("MCP tool dispatcher correctly routed and executed calls.");
        });
    });
}
