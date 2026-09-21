//! VantaDB Integrations (Ollama, LangChain)
use serde::{Deserialize, Serialize};

/// Request mapping for a simple LangChain vector store search
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SearchRequest {
    /// The search query string
    pub query: String,
    /// The namespace/collection to search within
    pub collection: String,
    /// Optional temperature parameter for downstream generation
    pub temperature: Option<f32>,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

/// The search response payload with results and latency
#[derive(Serialize, Clone, Debug)]
pub struct SearchResponse {
    /// The list of search result entries
    pub results: Vec<serde_json::Value>,
    /// Round-trip latency in milliseconds
    pub latency_ms: u64,
}

/// Simulated Axum handler for vector retrieval with structured filters
pub async fn search_handler(_payload: SearchRequest) -> SearchResponse {
    // Converts hybrid text query to logical plan here
    SearchResponse {
        results: vec![],
        latency_ms: 5,
    }
}

/// Request for proxied Ollama generation
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OllamaGenerateRequest {
    /// The Ollama model name to use
    pub model: String,
    /// The input prompt for generation
    pub prompt: String,
    /// Whether to stream the response tokens
    pub stream: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── SearchRequest ──

    #[test]
    fn test_search_request_defaults() {
        let req: SearchRequest =
            serde_json::from_str(r#"{"query": "hello", "collection": "docs"}"#).unwrap();
        assert_eq!(req.query, "hello");
        assert_eq!(req.collection, "docs");
        assert!(req.temperature.is_none());
        assert!(req.limit.is_none());
    }

    #[test]
    fn test_search_request_all_fields() {
        let req = SearchRequest {
            query: "test".into(),
            collection: "ns".into(),
            temperature: Some(0.7),
            limit: Some(10),
        };
        assert_eq!(req.query, "test");
        assert_eq!(req.temperature, Some(0.7));
        assert_eq!(req.limit, Some(10));
    }

    #[test]
    fn test_search_request_serialization_roundtrip() {
        let req = SearchRequest {
            query: "rust".into(),
            collection: "code".into(),
            temperature: None,
            limit: Some(5),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: SearchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.query, deserialized.query);
        assert_eq!(req.collection, deserialized.collection);
        assert_eq!(req.limit, deserialized.limit);
    }

    // ── SearchResponse ──

    #[test]
    fn test_search_response_empty() {
        let resp = SearchResponse {
            results: vec![],
            latency_ms: 5,
        };
        assert!(resp.results.is_empty());
        assert_eq!(resp.latency_ms, 5);
    }

    #[test]
    fn test_search_response_with_results() {
        let resp = SearchResponse {
            results: vec![serde_json::json!({"id": 1, "score": 0.9})],
            latency_ms: 10,
        };
        assert_eq!(resp.results.len(), 1);
        assert_eq!(resp.results[0]["id"], 1);
    }

    // ── OllamaGenerateRequest ──

    #[test]
    fn test_ollama_request_defaults() {
        let req: OllamaGenerateRequest =
            serde_json::from_str(r#"{"model": "llama3", "prompt": "hi"}"#).unwrap();
        assert_eq!(req.model, "llama3");
        assert_eq!(req.prompt, "hi");
        assert!(req.stream.is_none());
    }

    #[test]
    fn test_ollama_request_with_stream() {
        let req = OllamaGenerateRequest {
            model: "mistral".into(),
            prompt: "hello".into(),
            stream: Some(true),
        };
        assert_eq!(req.model, "mistral");
        assert_eq!(req.stream, Some(true));
    }

    #[test]
    fn test_ollama_request_serialization_roundtrip() {
        let req = OllamaGenerateRequest {
            model: "codellama".into(),
            prompt: "write code".into(),
            stream: Some(false),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: OllamaGenerateRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.model, deserialized.model);
        assert_eq!(req.prompt, deserialized.prompt);
        assert_eq!(req.stream, deserialized.stream);
    }

    // ── Clone/Debug ──

    #[test]
    fn test_search_request_clone_debug() {
        let req = SearchRequest {
            query: "q".into(),
            collection: "c".into(),
            temperature: Some(0.5),
            limit: Some(3),
        };
        let cloned = req.clone();
        assert_eq!(req.query, cloned.query);
        let dbg = format!("{:?}", req);
        assert!(dbg.contains("SearchRequest"));
    }

    #[test]
    fn test_search_response_clone_debug() {
        let resp = SearchResponse {
            results: vec![serde_json::json!({"k": "v"})],
            latency_ms: 42,
        };
        let cloned = resp.clone();
        assert_eq!(cloned.latency_ms, 42);
        let dbg = format!("{:?}", resp);
        assert!(dbg.contains("latency_ms"));
    }

    #[test]
    fn test_ollama_request_clone_debug() {
        let req = OllamaGenerateRequest {
            model: "m".into(),
            prompt: "p".into(),
            stream: Some(true),
        };
        let cloned = req.clone();
        assert_eq!(req.model, cloned.model);
        let dbg = format!("{:?}", req);
        assert!(dbg.contains("OllamaGenerateRequest"));
    }
}
