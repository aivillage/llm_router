use crate::{
    chat::{chat_trait::ChatLlm, errors::ModelError, History},
    secret_manager::Secrets,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use reqwest::Client;

const API_URL_V1: &str = "https://api.openai.com/v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIParameters {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub stream: Option<bool>,
    pub stop: Option<Vec<String>>,
    pub max_tokens: Option<i64>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub logit_bias: Option<HashMap<String, i32>>,
    pub user: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatCompletionMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub enum MessageRole {
    user,
    system,
    assistant,
    function,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletionMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionMessageForResponse {
    pub role: MessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChoice {
    pub index: i64,
    pub message: ChatCompletionMessageForResponse,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum FinishReason {
    stop,
    length,
    function_call,
    content_filter,
    null,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIModel {
    pub name: String,
    pub parameters: OpenAIParameters,
    pub context_size: usize,
}

#[async_trait]
impl ChatLlm for OpenAIModel {
    fn name(&self) -> &str {
        &self.name
    }

    fn context_size(&self) -> usize {
        250
    }

    async fn chat(
        &self,
        secrets: Secrets,
        prompt: String,
        system: Option<String>,
        history: Vec<History>,
    ) -> Result<String, ModelError> {
        let mut messages = Vec::new();
        if let Some(system) = system {
            messages.push(ChatCompletionMessage {
                role: MessageRole::system,
                content: system,
                name: None,
            });
        }
        for h in history {
            messages.push(ChatCompletionMessage {
                role: MessageRole::user,
                content: h.prompt,
                name: None,
            });
            messages.push(ChatCompletionMessage {
                role: MessageRole::assistant,
                content: h.generation,
                name: None,
            });
        }
        messages.push(ChatCompletionMessage {
            role: MessageRole::user,
            content: prompt,
            name: None,
        });
        let request = ChatCompletionRequest {
            model: self.name.clone(),
            messages,
            temperature: self.parameters.temperature,
            top_p: self.parameters.top_p,
            stream: self.parameters.stream,
            stop: self.parameters.stop.clone(),
            max_tokens: self.parameters.max_tokens,
            presence_penalty: self.parameters.presence_penalty,
            frequency_penalty: self.parameters.frequency_penalty,
            logit_bias: self.parameters.logit_bias.clone(),
            user: self.parameters.user.clone(),
        };

        let auth_token = secrets
            .get_secret("OPENAI_API_TOKEN")
            .await
            .ok_or(ModelError::Other("Missing Auth".to_string()))?;

        let client = Client::new();
        let response = client
            .post(format!("{}/chat/completions", API_URL_V1))
            .json(&request)
            .header("Authorization", format!("Bearer {}", auth_token))
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error sending request to openai: {}", e);
                ModelError::UpstreamModelError
            })?;

        if response.status().is_server_error() {
            tracing::error!(
                "Error from openai: {}",
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
                        "Error from openai: {}",
                        response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown".to_string())
                    );
                    return Err(ModelError::UpstreamModelError);
                }
            }
        }

        let mut response: ChatCompletionResponse = response.json().await.map_err(|e| {
            tracing::error!("Error parsing response from openai: {}", e);
            ModelError::UpstreamModelError
        })?;
        
        let response = response.choices.pop().ok_or_else(|| {
            tracing::error!("No generation in response from openai");
            ModelError::UpstreamModelError
        })?;
        let response = response.message;
        response.content.ok_or_else(|| {
            tracing::error!("No generation in response from openai");
            ModelError::UpstreamModelError
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIModels {
    pub models: Vec<OpenAIModel>,
}

impl OpenAIModels {
    pub fn new<P: AsRef<Path> + Send + Sync>(file: P) -> anyhow::Result<Self> {
        let file = std::fs::File::open(file)?;
        let models: Vec<OpenAIModel> = serde_json::from_reader(file)?;
        Ok(Self { models })
    }
}
