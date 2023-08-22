use super::SingleTurnLlm;
use crate::{errors::ModelError, secret_manager::Secrets};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::Path;
use tracing;

use reqwest::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct HuggingFaceModelParameters {
    max_new_tokens: Option<u64>,
    repitition_penalty: Option<f64>,
    temperature: Option<f64>,
    return_full_text: Option<bool>,
    top_k: Option<u64>,
    top_p: Option<f64>,
    stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HuggingFacePromptFormat {
    pub system_token: String,
    pub prompt_token: String,
    pub assistant_token: String,
    pub stop_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HuggingFaceModel {
    pub name: String,
    pub url: String,
    pub parameters: HuggingFaceModelParameters,
    pub prompt_format: HuggingFacePromptFormat,
}

#[derive(Debug, Serialize, Deserialize)]
struct Generation {
    generated_text: String,
}

type HuggingFaceResponse = Vec<Generation>;


#[async_trait]
impl SingleTurnLlm for HuggingFaceModel {
    fn name(&self) -> &str {
        &self.name
    }

    async fn generate(
        &self,
        secrets: Secrets,
        prompt: String,
        preprompt: Option<String>,
    ) -> Result<String, ModelError> {
        let mut full_prompt = String::new();
        if let Some(preprompt) = preprompt {
            full_prompt.push_str(&format!("{}{}{}", self.prompt_format.system_token, preprompt, self.prompt_format.stop_token));
        }
        full_prompt.push_str(&format!("{}{}{}", self.prompt_format.prompt_token, prompt, self.prompt_format.stop_token));
        full_prompt.push_str(self.prompt_format.assistant_token.as_str());
        
        let client = Client::new();
        let auth_token = secrets.get_secret("HUGGINGFACE_API_TOKEN").await.ok_or(ModelError::Other("Missing Auth".to_string()))?;
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
            tracing::error!("Error from huggingface: {}", response.text().await.unwrap_or_else(|_| "Unknown".to_string()));
            return Err(ModelError::UpstreamModelError);
        }

        if response.status().is_client_error() {
            match response.status() {
                reqwest::StatusCode::TOO_MANY_REQUESTS => {
                    tracing::error!("Rate limit exceeded");
                    return Err(ModelError::RateLimitExceeded(1000));
                }
                _ => {
                    tracing::error!("Error from huggingface: {}", response.text().await.unwrap_or_else(|_| "Unknown".to_string()));
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
