use crate::error::{Result, VantaError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct OllamaEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

pub struct LlmClient {
    client: Client,
    base_url: String,
    default_model: String,
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmClient {
    pub fn new() -> Self {
        let base_url =
            env::var("VANTA_LLM_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

        // El predeterminado de ollama para embeddings vectoriales es nomic-embed-text o all-minilm
        let default_model =
            env::var("VANTA_LLM_MODEL").unwrap_or_else(|_| "all-minilm".to_string());

        Self {
            client: Client::builder()
                .pool_idle_timeout(Some(std::time::Duration::from_secs(60)))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url,
            default_model,
        }
    }

    /// Comunica al LLM para traducir un texto nativo a un vector HNSW compatible.
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);

        let req_body = OllamaEmbeddingRequest {
            model: &self.default_model,
            input: text,
        };

        let response = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                VantaError::Execution(format!(
                    "Network error communicating with Inference Bridge: {}",
                    e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(VantaError::Execution(format!(
                "Inference Bridge returned error status: {}",
                status
            )));
        }

        let result: OllamaEmbeddingResponse = response.json().await.map_err(|e| {
            VantaError::Execution(format!(
                "Invalid response format from Inference Bridge: {}",
                e
            ))
        })?;

        Ok(result.embedding)
    }

    /// Invoke the LLM to generate a semantic summary of a group of archived nodes.
    /// The prompt includes importance and keywords so the summary preserves
    /// the priority data rather than being a generic recap.
    pub async fn summarize_context(&self, nodes: &[&crate::node::UnifiedNode]) -> Result<String> {
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
            return Err(VantaError::Execution(
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

        let response = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                VantaError::Execution(format!("Network error during Semantic Summarization: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(VantaError::Execution(format!(
                "Inference Bridge returned error status during summarization: {}",
                status
            )));
        }

        let result: OllamaGenerateResponse = response.json().await.map_err(|e| {
            VantaError::Execution(format!(
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
