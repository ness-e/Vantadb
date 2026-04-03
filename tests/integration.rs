use connectomedb::integrations::*;
use tokio;

#[tokio::test]
async fn test_langchain_search_handler() {
    let req = SearchRequest {
        query: "What is the capital of VZLA?".to_string(),
        collection: "nodes".to_string(),
        temperature: Some(0.1),
        limit: Some(10),
    };

    let res = search_handler(req).await;
    assert_eq!(res.latency_ms, 5);
}

#[tokio::test]
async fn test_ollama_proxy() {
    let req = OllamaGenerateRequest {
        model: "llama3".to_string(),
        prompt: "Tell me about memory constraints".to_string(),
        stream: Some(false),
    };

    let res = ollama_proxy_handler(req).await;
    assert!(res.contains("Context-Aware"));
}
