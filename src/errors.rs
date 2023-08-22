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
    RateLimitExceeded(u64),
    #[error("Prompt was too long")]
    PromptTooLong,
    #[error("Preprompt was too long")]
    PrepromptTooLong,
    #[error("Other error: {0}")]
    Other(String),
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
            ModelError::PromptTooLong => {
                (reqwest::StatusCode::UNPROCESSABLE_ENTITY, "Prompt too long")
            }
            ModelError::PrepromptTooLong => (
                reqwest::StatusCode::UNPROCESSABLE_ENTITY,
                "Preprompt too long",
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
        let mut split = s.splitn(2, ':');
        let status = split.next()?;
        let error = split.next()?;
        if status == "ERR" {
            serde_json::from_str(error).ok()
        } else {
            None
        }
    }
}
