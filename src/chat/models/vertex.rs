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
pub struct OpenPromptFormat {
    pub system_token: String,
    pub prompt_token: String,
    pub assistant_token: String,
    pub stop_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HuggingFaceModel {
    pub name: String,
    pub url: String,
    pub prompt_format: HuggingFacePromptFormat,
}

#[derive(Debug, Serialize, Deserialize)]
struct Generation {
    generated_text: String,
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

type HuggingFaceResponse = Vec<Generation>;

#[async_trait]
impl ChatLlm for HuggingFaceModel {
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
        system: Option<String>,
        history: Vec<History>,
    ) -> Result<String, ModelError> {
        let mut full_prompt = String::new();
        if let Some(system) = system {
            full_prompt.push_str(&format!(
                "{}{}{}",
                self.prompt_format.system_token, system, self.prompt_format.stop_token
            ));
        }
        for h in history {
            full_prompt.push_str(&format!(
                "{}{}{}",
                self.prompt_format.prompt_token, h.prompt, self.prompt_format.stop_token
            ));
            full_prompt.push_str(&format!(
                "{}{}{}",
                self.prompt_format.assistant_token, h.generation, self.prompt_format.stop_token
            ));
        }
        full_prompt.push_str(&format!(
            "{}{}{}",
            self.prompt_format.prompt_token, prompt, self.prompt_format.stop_token
        ));
        full_prompt.push_str(self.prompt_format.assistant_token.as_str());

        let auth_token = secrets
            .get_secret("VERTEX_API_TOKEN")
            .await
            .ok_or(ModelError::Other("Missing Auth".to_string()))?;

        let client = Client::new();
        let response = client
            .post(&self.url)
            .json(&serde_json::json!({
                "inputs": full_prompt,
                "parameters": self.parameters,
                "stream": false,
            }))
            .header("Authorization", format!("Bearer {}", auth_token))
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error sending request to huggingface: {}", e);
                ModelError::UpstreamModelError
            })?;

        if response.status().is_server_error() {
            tracing::error!(
                "Error from huggingface: {}",
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
                        "Error from huggingface: {}",
                        response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown".to_string())
                    );
                    return Err(ModelError::UpstreamModelError);
                }
            }
        }

        let mut response: HuggingFaceResponse = response.json().await.map_err(|e| {
            tracing::error!("Error parsing response from huggingface: {}", e);
            ModelError::UpstreamModelError
        })?;
        let generation = response.pop().ok_or_else(|| {
            tracing::error!("No generation in response from huggingface");
            ModelError::UpstreamModelError
        })?;
        Ok(generation.generated_text)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HuggingFaceModels {
    pub models: Vec<HuggingFaceModel>,
}

impl HuggingFaceModels {
    pub fn new<P: AsRef<Path> + Send + Sync>(file: P) -> anyhow::Result<Self> {
        let file = std::fs::File::open(file)?;
        let models: Vec<HuggingFaceModel> = serde_json::from_reader(file)?;
        Ok(Self { models })
    }
}
