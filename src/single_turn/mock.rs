use crate::errors::ModelError;
use super::SingleTurnLlm;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::path::Path;
use tracing;

#[derive(Debug, Serialize, Deserialize)]
pub struct MockModel {
    pub name: String,
    pub short: String,
    pub long: String,
}

#[async_trait]
impl SingleTurnLlm for MockModel {
    fn name(&self) -> &str {
        &self.name
    }
    async fn generate(
        &self,
        prompt: String,
        _preprompt: Option<String>,
    ) -> Result<String, ModelError> {
        match prompt.as_str() {
            "upstream_error" => {
                tracing::info!("Mocking upstream error");
                Err(ModelError::UpstreamModelError)
            }
            "rate_limit" => {
                tracing::info!("Mocking rate limit");
                Err(ModelError::RateLimitExceeded(1000))
            }
            "prompt_too_long" => {
                tracing::info!("Mocking prompt too long");
                Err(ModelError::PromptTooLong)
            }
            "preprompt_too_long" => {
                tracing::info!("Mocking preprompt too long");
                Err(ModelError::PrepromptTooLong)
            }
            "error" => {
                tracing::info!("Mocking other error");
                Err(ModelError::Other(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "other error",
                ))))
            }
            "long_response" => {
                tracing::info!("Mocking long response");
                Ok(self.long.clone())
            },
            _ => {
                tracing::info!("Mocking short response");
                Ok(self.short.clone())
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockModels {
    pub models: HashMap<String, MockModel>,
}

impl MockModels {
    pub fn new<P: AsRef<Path> + Send + Sync>(file: P) -> Self {
        let contents = std::fs::read_to_string(file).unwrap();
        let models: HashMap<String, MockModel> = serde_json::from_str(&contents).unwrap();
        Self { models }
    }
}
