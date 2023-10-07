use crate::{
    chat::{chat_trait::ChatLlm, errors::ModelError, History},
    secret_manager::Secrets,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::path::Path;
use tracing;

#[derive(Debug, Serialize, Deserialize)]
pub struct MockModel {
    pub name: String,
}

#[async_trait]
impl ChatLlm for ReflectionModel {
    fn name(&self) -> &str {
        &self.name
    }

    fn context_size(&self) -> usize {
        250
    }

    async fn chat(
        &self,
        _secrets: Secrets,
        prompt: String,
        _system: Option<String>,
        history: Vec<History>,
    ) -> Result<String, ModelError> {
        Ok(format!("history: {}, promt: {}", history, prompt))
        // match prompt.as_str() {
        //     "upstream_error" => {
        //         tracing::info!("Mocking upstream error");
        //         Err(ModelError::UpstreamModelError)
        //     }
        //     "rate_limit" => {
        //         tracing::info!("Mocking rate limit");
        //         Err(ModelError::RateLimitExceeded(1000))
        //     }
        //     "prompt_too_long" => {
        //         tracing::info!("Mocking prompt too long");
        //         Err(ModelError::PromptTooLong)
        //     }
        //     "system_too_long" => {
        //         tracing::info!("Mocking system too long");
        //         Err(ModelError::SystemTooLong)
        //     }
        //     "error" => {
        //         tracing::info!("Mocking other error");
        //         Err(ModelError::Other("Other error".to_string()))
        //     }
        //     "long_response" => {
        //         tracing::info!("Mocking long response");
        //         Ok(format!("response: {}, {}", history.len(),self.long.clone()))
        //     }
        //     _ => {
        //         tracing::info!("Mocking short response");
        //         Ok(format!("response: {}, {}", history.len(),self.short.clone()))
        //     }
        // }
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
