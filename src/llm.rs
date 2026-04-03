use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use crate::error::{ConnectomeError, Result};

#[derive(Serialize)]
struct OllamaEmbeddingRequest<'a> {
    model: &'a str,
    prompt: &'a str,
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
        let base_url = env::var("CONNECTOME_LLM_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        
        // El predeterminado de ollama para embeddings vectoriales es nomic-embed-text o all-minilm
        let default_model = env::var("CONNECTOME_LLM_MODEL")
            .unwrap_or_else(|_| "all-minilm".to_string());
            
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
            prompt: text,
        };

        let response = self.client.post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| ConnectomeError::Execution(format!("Network error communicating with Inference Bridge: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ConnectomeError::Execution(format!(
                "Inference Bridge returned error status: {}", status
            )));
        }

        let result: OllamaEmbeddingResponse = response.json().await
            .map_err(|e| ConnectomeError::Execution(format!("Invalid response format from Inference Bridge: {}", e)))?;

        Ok(result.embedding)
    }
}
