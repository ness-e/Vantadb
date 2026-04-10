//! Integration Handlers Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{VantaHarness, TerminalReporter};
use vantadb::integrations::*;

#[tokio::test]
async fn integrations_certification() {
    let mut harness = VantaHarness::new("LOGIC LAYER (HANDLERS & MCP)");

    harness.execute("Search: LangChain Handler Proximity", || {
        futures::executor::block_on(async {
            let req = SearchRequest {
                query: "What is the capital of VZLA?".to_string(),
                collection: "nodes".to_string(),
                temperature: Some(0.1),
                limit: Some(10),
            };

            TerminalReporter::sub_step("Simulating semantic search via LangChain bridge...");
            let res = search_handler(req).await;
            assert_eq!(res.latency_ms, 5);
            TerminalReporter::success("LangChain search handler response validated.");
        });
    });

    harness.execute("Proxy: Ollama Context-Aware Generation", || {
        futures::executor::block_on(async {
            let req = OllamaGenerateRequest {
                model: "llama3".to_string(),
                prompt: "Tell me about memory constraints".to_string(),
                stream: Some(false),
            };

            TerminalReporter::sub_step("Routing generational prompt through Ollama proxy...");
            let res = ollama_proxy_handler(req).await;
            assert!(res.contains("Context-Aware"));
            TerminalReporter::success("Ollama proxy handler consensus reached.");
        });
    });
}
