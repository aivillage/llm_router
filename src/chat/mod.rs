//! Chat module
//!
//! This module contains the chat router and the chat models.

pub mod chat_trait;
pub mod errors;
pub mod models;
pub mod state;
use crate::AppState;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Router,
};
use std::{path::Path, sync::Arc};

use self::state::ChatModels;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub models: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub prompt: String,
    pub generation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub uuid: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub prompt: String,
    #[serde(default = "Vec::new")]
    pub history: Vec<History>,
    // For idempotency
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub generation: String,
    pub uuid: String,
}

async fn chat(
    State(chat_state): State<ChatState>,
    Json(request): Json<ChatRequest>,
) -> Result<Response> {
    tracing::trace!("chat called");
    let redis_client = chat_state.app_state.redis_client.clone();
    let secret_manager = chat_state.app_state.secret_manager.clone();
    match chat_state
        .chat_models
        .chat(redis_client, secret_manager, request)
        .await
    {
        Ok(generation) => Ok(Json(generation).into_response()),
        Err(e) => Ok(e.into_response()),
    }
}

async fn models(State(chat_state): State<ChatState>) -> Result<Response> {
    tracing::trace!("models called");
    let models = chat_state.chat_models.models().await?;
    Ok(Json(models).into_response())
}

#[derive(Clone)]
pub struct ChatState {
    pub chat_models: Arc<ChatModels>,
    pub app_state: AppState,
}

pub async fn chat_router(app_state: AppState) -> anyhow::Result<Router> {
    let model_path = std::env::var("MODEL_DIR").unwrap_or_else(|_| "/opt/models/".to_string());
    let chat_path = Path::new(model_path.as_str()).join("chat");
    let chat_models = Arc::new(ChatModels::new(chat_path)?);

    let chat_state = ChatState {
        chat_models,
        app_state,
    };
    let router = Router::new()
        .route("/generate", post(chat))
        .with_state(chat_state.clone())
        .route("/models", get(models))
        .with_state(chat_state);

    Ok(router)
}
