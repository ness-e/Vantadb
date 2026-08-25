//! Optional external LLM integration.
//!
//! This module is not a core dependency of the v0.1.x MVP. Embedding generation and LLM runtime
//! behavior remain external or experimental; the core stores and retrieves provided vectors.
//!
//! ## Embedding providers
//!
//! COMP-010: Abstract [`EmbeddingProvider`] trait with two implementations:
//! - [`OllamaProvider`] — Ollama `/api/embed` (default)
//! - [`OpenAIProvider`] — OpenAI `/v1/embeddings`
//!
//! Select the provider at runtime via `VANTA_EMBEDDING_PROVIDER` (ollama|openai).

use crate::error::{Result, VantaError};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;

// ── EmbeddingProvider trait ────────────────────────────────────────────

/// Abstract embedding provider.
///
/// Implementations convert text into a float vector suitable for HNSW
/// similarity search. Each provider is responsible for its own HTTP
/// transport and authentication.
pub trait EmbeddingProvider: Send + Sync {
    /// Embed `text` and return a dense `f32` vector.
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

// ── Factory ───────────────────────────────────────────────────────────

/// Return the embedding provider selected by `VANTA_EMBEDDING_PROVIDER`.
///
/// | Value   | Provider                                          |
/// |---------|---------------------------------------------------|
/// | `openai`| [`OpenAIProvider`] — requires `VANTA_OPENAI_API_KEY` |
/// | _any_   | [`OllamaProvider`] (default)                      |
pub fn get_embedding_provider() -> Box<dyn EmbeddingProvider> {
    match env::var("VANTA_EMBEDDING_PROVIDER")
        .as_deref()
        .unwrap_or("ollama")
    {
        "openai" => Box::new(OpenAIProvider::new()),
        _ => Box::new(OllamaProvider::new()),
    }
}

// ── OllamaProvider ─────────────────────────────────────────────────────

#[derive(Serialize)]
struct OllamaEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
}

/// Embedding provider backed by a local Ollama server.
///
/// Reads `VANTA_LLM_URL` (default `http://localhost:11434`) and
/// `VANTA_LLM_MODEL` (default `all-minilm`).
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    default_model: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider from environment variables.
    pub fn new() -> Self {
        let base_url =
            env::var("VANTA_LLM_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let default_model =
            env::var("VANTA_LLM_MODEL").unwrap_or_else(|_| "all-minilm".to_string());
        Self {
            client: Client::builder()
                .pool_idle_timeout(Some(std::time::Duration::from_secs(60)))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url,
            default_model,
        }
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingProvider for OllamaProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embed", self.base_url);
        let req_body = OllamaEmbeddingRequest {
            model: &self.default_model,
            input: text,
        };
        let response = self.client.post(&url).json(&req_body).send().map_err(|e| {
            VantaError::generic_error(format!(
                "Network error communicating with Inference Bridge: {}",
                e
            ))
        })?;
        if !response.status().is_success() {
            let status = response.status();
            return Err(VantaError::generic_error(format!(
                "Inference Bridge returned error status: {}",
                status
            )));
        }
        let result: OllamaEmbeddingResponse = response.json().map_err(|e| {
            VantaError::generic_error(format!(
                "Invalid response format from Inference Bridge: {}",
                e
            ))
        })?;
        result
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| VantaError::generic_error("Ollama returned empty embeddings"))
    }
}

// ── OpenAIProvider ─────────────────────────────────────────────────────

/// Embedding provider backed by the OpenAI API.
///
/// Requires `VANTA_OPENAI_API_KEY`.  Reads `VANTA_OPENAI_MODEL`
/// (default `text-embedding-3-small`).
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider from environment variables.
    ///
    /// Panics if `VANTA_OPENAI_API_KEY` is not set.
    pub fn new() -> Self {
        let api_key = env::var("VANTA_OPENAI_API_KEY").expect("VANTA_OPENAI_API_KEY must be set");
        let model =
            env::var("VANTA_OPENAI_MODEL").unwrap_or_else(|_| "text-embedding-3-small".to_string());
        Self {
            client: Client::builder()
                .pool_idle_timeout(Some(std::time::Duration::from_secs(60)))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key,
            model,
        }
    }
}

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingProvider for OpenAIProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        #[derive(Serialize)]
        struct OpenAiRequest {
            model: String,
            input: String,
        }
        #[derive(Deserialize)]
        struct OpenAiResponse {
            data: Vec<OpenAiEmbedding>,
        }
        #[derive(Deserialize)]
        struct OpenAiEmbedding {
            embedding: Vec<f32>,
        }

        let url = "https://api.openai.com/v1/embeddings";
        let req_body = OpenAiRequest {
            model: self.model.clone(),
            input: text.to_string(),
        };

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req_body)
            .send()
            .map_err(|e| {
                VantaError::generic_error(format!("Network error communicating with OpenAI: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(VantaError::generic_error(format!(
                "OpenAI returned error status {}: {}",
                status, body
            )));
        }

        let result: OpenAiResponse = response.json().map_err(|e| {
            VantaError::generic_error(format!("Invalid response format from OpenAI: {}", e))
        })?;

        result
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| VantaError::generic_error("OpenAI returned empty embeddings"))
    }
}

// ── LlmClient (text generation only) ───────────────────────────────────

/// HTTP client for communicating with an Ollama inference server.
///
/// Used exclusively for **text generation** (`summarize_context`).
/// For embeddings see [`EmbeddingProvider`], [`OllamaProvider`], or
/// [`OpenAIProvider`].
pub struct LlmClient {
    client: Client,
    base_url: String,
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmClient {
    /// Create a new client reading `VANTA_LLM_URL` from the environment.
    pub fn new() -> Self {
        let base_url =
            env::var("VANTA_LLM_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        Self {
            client: Client::builder()
                .pool_idle_timeout(Some(std::time::Duration::from_secs(60)))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url,
        }
    }

    /// Invoke the LLM to generate a semantic summary of a group of archived nodes.
    /// The prompt includes importance and keywords so the summary preserves
    /// the priority data rather than being a generic recap.
    pub fn summarize_context(&self, nodes: &[&crate::node::UnifiedNode]) -> Result<String> {
        // Build structured context: each node contributes its content + importance metadata
        let mut context_blocks = Vec::new();
        for (i, node) in nodes.iter().enumerate() {
            let content = node
                .relational
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("[no content]");

            let keywords = node
                .relational
                .get("keywords")
                .and_then(|v| v.as_str())
                .unwrap_or("none");

            let node_type = node
                .relational
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            context_blocks.push(format!(
                "--- Node Fragment #{} ---\nType: {}\nContent: {}\nSemantic Priority: {:.2}\nConfidence Score: {:.2}\nKeywords: {}\nAccess Count: {}",
                i + 1, node_type, content,
                node.importance, node.confidence_score,
                keywords, node.hits
            ));
        }

        let full_context = context_blocks.join("\n\n");

        if full_context.trim().is_empty() {
            return Err(VantaError::InvalidInput(
                "No summarizable content found in node group".to_string(),
            ));
        }

        let system_prompt = "You are VantaDB's Semantic Compression Engine. \
            Your task is to distill a group of related data fragments into a single, \
            dense summary that preserves the most semantically important information. \
            Pay special attention to fragments with high Semantic Priority — these are \
            contextually critical and their essence MUST be preserved. \
            Output ONLY the summary text, no preamble or formatting.";

        let user_prompt = format!(
            "Compress the following {} nodes into a single coherent summary:\n\n{}",
            nodes.len(),
            full_context
        );

        let summarize_model =
            env::var("VANTA_LLM_SUMMARIZE_MODEL").unwrap_or_else(|_| "llama3".to_string());

        let url = format!("{}/api/generate", self.base_url);

        let req_body = OllamaGenerateRequest {
            model: &summarize_model,
            system: system_prompt,
            prompt: &user_prompt,
            stream: false,
        };

        let response = self.client.post(&url).json(&req_body).send().map_err(|e| {
            VantaError::generic_error(format!(
                "Network error during Semantic Summarization: {}",
                e
            ))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(VantaError::generic_error(format!(
                "Inference Bridge returned error status during summarization: {}",
                status
            )));
        }

        let result: OllamaGenerateResponse = response.json().map_err(|e| {
            VantaError::generic_error(format!(
                "Invalid response format from Inference Bridge (summarize): {}",
                e
            ))
        })?;

        Ok(result.response)
    }
}

#[derive(Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    system: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// One-shot HTTP mock: accepts a single request, replies with a fixed
    /// JSON body, and returns the captured (request-line, body) to the caller.
    fn spawn_mock(
        response_body: &'static str,
    ) -> (String, std::thread::JoinHandle<(String, String)>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock listener");
        let addr = listener.local_addr().expect("mock addr");
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 4096];
            let mut raw = Vec::new();
            loop {
                let n = stream.read(&mut buf).expect("read mock");
                if n == 0 {
                    break;
                }
                raw.extend_from_slice(&buf[..n]);
                if raw.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            // Drain the body per Content-Length so the client sees a complete request.
            let text = String::from_utf8_lossy(&raw).to_string();
            let len: usize = text
                .split("\r\n")
                .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))
                .and_then(|l| l.split(':').nth(1))
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0);
            let header_end = text.find("\r\n\r\n").map(|i| i + 4).unwrap_or(text.len());
            while raw.len() < header_end + len {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => raw.extend_from_slice(&buf[..n]),
                }
            }
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
            (
                text.lines().next().unwrap_or_default().to_string(),
                String::from_utf8_lossy(&raw[header_end..]).to_string(),
            )
        });
        (format!("http://{addr}"), handle)
    }

    #[test]
    fn ollama_embed_uses_current_api_contract() {
        let (base_url, handle) = spawn_mock(r#"{"model":"all-minilm","embeddings":[[1.5,-2.25]]}"#);

        let provider = OllamaProvider {
            client: Client::new(),
            base_url,
            default_model: "all-minilm".to_string(),
        };

        let vec = provider.embed("why is the sky blue?").expect("embed ok");
        let (request_line, body) = handle.join().expect("mock thread");

        assert!(
            request_line.starts_with("POST /api/embed "),
            "wrong endpoint: {request_line}"
        );
        let json: serde_json::Value = serde_json::from_str(&body).expect("body is JSON");
        assert_eq!(json["model"], "all-minilm");
        assert_eq!(json["input"], "why is the sky blue?");
        assert!(
            json.get("prompt").is_none(),
            "must not send legacy `prompt` field"
        );
        assert_eq!(vec, vec![1.5, -2.25]);
    }

    #[test]
    fn ollama_embed_rejects_empty_embeddings() {
        let (base_url, handle) = spawn_mock(r#"{"model":"all-minilm","embeddings":[]}"#);

        let provider = OllamaProvider {
            client: Client::new(),
            base_url,
            default_model: "all-minilm".to_string(),
        };

        let err = provider
            .embed("x")
            .expect_err("empty embeddings must error");
        handle.join().expect("mock thread");
        assert!(
            err.to_string().contains("empty embeddings"),
            "unexpected error: {err}"
        );
    }
}
