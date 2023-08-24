use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const GPT3_5_TURBO: &str = "gpt-3.5-turbo";
pub const GPT3_5_TURBO_0301: &str = "gpt-3.5-turbo-0301";
pub const GPT3_5_TURBO_0613: &str = "gpt-3.5-turbo-0613";

pub const GPT4: &str = "gpt-4";
pub const GPT4_0314: &str = "gpt-4-0314";
pub const GPT4_32K: &str = "gpt-4-32k";
pub const GPT4_32K_0314: &str = "gpt-4-32k-0314";
pub const GPT4_0613: &str = "gpt-4-0613";

pub struct OpenAIParameters {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub n: Option<i64>,
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
    pub n: Option<i64>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionMessageForResponse {
    pub role: MessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChoice {
    pub index: i64,
    pub message: ChatCompletionMessageForResponse,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: common::Usage,
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