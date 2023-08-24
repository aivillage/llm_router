use axum::{extract::Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    error: String,
}

#[derive(Error, Debug, Deserialize, Serialize)]
pub enum ModelError {
    #[error("Model not found")]
    ModelNotFound,
    #[error("Model had an upstream error")]
    UpstreamModelError,
    #[error("Exceded Rate Limit, try again in {0} miliseconds")]
    HistoryPromptTooLong(u64),
    #[error("Exceded Rate Limit, try again in {0} miliseconds")]
    RateLimitExceeded(u64),
    #[error("Prompt was too long")]
    PromptTooLong,
    #[error("Preprompt was too long")]
    SystemTooLong,
    #[error("Other error: {0}")]
    Other(String),
}

// This should be improved
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
            ModelError::HistoryPromptTooLong(_) => (
                reqwest::StatusCode::UNPROCESSABLE_ENTITY,
                "Historical prompt too long",
            ),
            ModelError::PromptTooLong => {
                (reqwest::StatusCode::UNPROCESSABLE_ENTITY, "Prompt too long")
            }
            ModelError::SystemTooLong => (
                reqwest::StatusCode::UNPROCESSABLE_ENTITY,
                "System prompt too long",
            ),
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

impl ModelError {
    pub fn to_redis_string(&self) -> String {
        format!("ERR:{}", serde_json::to_string(self).unwrap())
    }

    pub fn from_redis_string(s: &str) -> Option<Self> {
        let (status, error) = s.split_once(':')?;

        if status == "ERR" {
            serde_json::from_str(error).ok()
        } else {
            None
        }
    }
}
