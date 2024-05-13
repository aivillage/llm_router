use crate::{
    chat::{chat_trait::ChatLlm, errors::ModelError, History},
    secret_manager::Secrets,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use reqwest::Client;

const API_URL_V1: &str = "https://api.anthropic.com/v1";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ClaudeParameters {
    pub temperature: Option<f64>,
    pub top_k: Option<i64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
    pub metadata: Option<HashMap<String, String>>,
    pub system: Option<String>,
    pub tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Default)]
pub struct MessageRequest {
    pub model: String,
    pub messages: Vec<MessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub enum MessageRole {
    user,
    assistant,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageContent {
    pub role: MessageRole,
    pub content: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: MessageRole,
    pub content: serde_json::Value,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: i32,
    pub output_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeModel {
    pub name: String,
    pub model: String,
    pub parameters: ClaudeParameters,
    pub context_size: usize,
}

#[async_trait]
impl ChatLlm for ClaudeModel {
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
        let mut messages = Vec::new();
        for h in history {
            messages.push(MessageContent {
                role: MessageRole::user,
                content: serde_json::to_value(h.prompt).unwrap(),
            });
            messages.push(MessageContent {
                role: MessageRole::assistant,
                content: serde_json::to_value(h.generation).unwrap(),
            });
        }
        messages.push(MessageContent {
            role: MessageRole::user,
            content: serde_json::to_value(prompt).unwrap(),
        });
        let request = MessageRequest {
            model: self.model.clone(),
            messages,
            temperature: self.parameters.temperature,
            top_k: self.parameters.top_k,
            top_p: self.parameters.top_p,
            max_tokens: self.parameters.max_tokens,
            metadata: self.parameters.metadata.clone(),
            stop_sequences: None,
            stream: None,
            system: system.or(self.parameters.system.clone()),
            tools: self.parameters.tools.clone(),
        };

        let auth_token = secrets
            .get_secret("ANTHROPIC_API_TOKEN")
            .await
            .ok_or(ModelError::Other("Missing Auth".to_string()))?;

        let client = Client::new();
        let response = client
            .post(format!("{}/messages", API_URL_V1))
            .json(&request)
            .header("x-api-key", auth_token)
            .header("anthropic-version", "2023-06-01") 
            .header("anthropic-beta", "tools-2024-04-04")
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error sending request to Claude: {}", e);
                ModelError::UpstreamModelError
            })?;

        if response.status().is_server_error() {
            tracing::error!(
                "Error from Claude: {}",
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
                        "Error from Claude: {}",
                        response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown".to_string())
                    );
                    return Err(ModelError::UpstreamModelError);
                }
            }
        }

        let response: MessageResponse = response.json().await.map_err(|e| {
            tracing::error!("Error parsing response from Claude: {}", e);
            ModelError::UpstreamModelError
        })?;

        serde_json::to_string(&response.content).map_err(|_| {
            tracing::error!("Error converting Claude response to string");
            ModelError::UpstreamModelError
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeModels {
    pub models: Vec<ClaudeModel>,
}

impl ClaudeModels {
    pub fn new<P: AsRef<Path> + Send + Sync>(file: P) -> anyhow::Result<Self> {
        let file = std::fs::File::open(file)?;
        let models: Vec<ClaudeModel> = serde_json::from_reader(file)?;
        Ok(Self { models })
    }
}