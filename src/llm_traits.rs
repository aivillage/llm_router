use async_trait::async_trait;
use axum::{extract::Json,response::IntoResponse};
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    error: String,
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Model not found")]
    ModelNotFound,
    #[error("Model had an upstream error")]
    UpstreamModelError,
    #[error("Exceded Rate Limit, try again in {0} miliseconds")]
    RateLimitExceeded(u64),
    #[error("Prompt was too long")]
    PromptTooLong,
    #[error("Preprompt was too long")]
    PrepromptTooLong,
    #[error("Other error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl IntoResponse for ModelError {
    fn into_response(self) -> axum::response::Response {
        let (code, reason) = match self {
            ModelError::UpstreamModelError => (
                reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                "Upstream model error",
            ),
            ModelError::RateLimitExceeded(_) => (
                reqwest::StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded",
            ),
            ModelError::PromptTooLong => (reqwest::StatusCode::UNPROCESSABLE_ENTITY, "Prompt too long"),
            ModelError::PrepromptTooLong => {
                (reqwest::StatusCode::UNPROCESSABLE_ENTITY, "Preprompt too long")
            }
            ModelError::Other(_) => (reqwest::StatusCode::INTERNAL_SERVER_ERROR, "Other error"),
            ModelError::ModelNotFound => (reqwest::StatusCode::NOT_FOUND, "Model not found"),
        };

        let mut response = Json(ErrorResponse {
            error: reason.to_string(),
        })
        .into_response();
        *response.status_mut() = code;
        response
    }
}

#[async_trait]
pub trait SingleTurnLlm {
    fn name(&self) -> &str;
    async fn generate(
        &self,
        prompt: String,
        preprompt: Option<String>,
    ) -> Result<String, ModelError>;
}

pub trait CacheAble {
    fn cache_key(&self) -> &str;

}