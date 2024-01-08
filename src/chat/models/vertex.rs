//!
//! 
//! 
//! 
//! 
//! Example input:{
//! "instances": [
//!     {
//!         "prompt": "Write a poem about Valencia."
//!     }
//! ]
//! }
//! 
//! Example output:{
//!  "predictions": [
//!     "Prompt:\nWrite a poem about Valencia.\nOutput:\norange and available to those who visit\nand main ports for those sail"
//!   ],
//!   "deployedModelId": "---",
//!   "model": "projects/---/locations/us-east4/models/llama2-7b-chat",
//!   "modelDisplayName": "llama2-7b-chat",
//!   "modelVersionId": "1"
//!  }


use crate::{
    chat::{chat_trait::ChatLlm, errors::ModelError, History},
    secret_manager::Secrets,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::Path;
use tracing;

use reqwest::Client;


/// Fill this out once we know how.
#[derive(Debug, Serialize, Deserialize)]
pub struct VertexOpenModelParameters {}

#[derive(Debug, Serialize, Deserialize)]
pub struct VertexModel {
    pub name: String,
    pub url: String,
    pub context_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatInstance {
    prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    instances: Vec<ChatInstance>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatResponse {
    predictions: Vec<String>,
}

#[async_trait]
impl ChatLlm for VertexModel {
    fn name(&self) -> &str {
        &self.name
    }

    fn context_size(&self) -> usize {
        self.context_size
    }

    async fn chat(
        &self,
        secrets: Secrets,
        prompt: String,
        _system: Option<String>,
        _history: Vec<History>,
    ) -> Result<String, ModelError> {
        let mut full_prompt = prompt;

        let auth_token = secrets
            .get_secret("VERTEX_API_TOKEN")
            .await
            .ok_or(ModelError::Other("Missing Auth".to_string()))?;

        let client = Client::new();
        let response = client
            .post(&self.url)
            .json(&serde_json::json!({
                "instances": [{"prompt": full_prompt}]
            }))
            .header("Authorization", format!("Bearer {}", auth_token))
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error sending request to vertex: {}", e);
                ModelError::UpstreamModelError
            })?;

        if response.status().is_server_error() {
            tracing::error!(
                "Error from vertex: {}",
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown".to_string())
            );
            return Err(ModelError::UpstreamModelError);
        }

        if response.status().is_client_error() {
            match response.status() {
                reqwest::StatusCode::TOO_MANY_REQUESTS => {
                    tracing::error!("Rate limit exceeded");
                    return Err(ModelError::RateLimitExceeded(1000));
                }
                _ => {
                    tracing::error!(
                        "Error from vertex: {}",
                        response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown".to_string())
                    );
                    return Err(ModelError::UpstreamModelError);
                }
            }
        }

        let mut response: ChatResponse = response.json().await.map_err(|e| {
            tracing::error!("Error parsing response from huggingface: {}", e);
            ModelError::UpstreamModelError
        })?;
        let generation = response.predictions.pop().ok_or_else(|| {
            tracing::error!("No generation in response from huggingface");
            ModelError::UpstreamModelError
        })?;
        let generation = generation.split("Output:").next().unwrap_or_default().to_string();
        Ok(generation)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VertexModels {
    pub models: Vec<VertexModel>,
}

impl VertexModels {
    pub fn new<P: AsRef<Path> + Send + Sync>(file: P) -> anyhow::Result<Self> {
        let file = std::fs::File::open(file)?;
        let models: Vec<VertexModel> = serde_json::from_reader(file)?;
        Ok(Self { models })
    }
}
