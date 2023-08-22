use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
pub mod errors;
pub mod logging;
pub mod single_turn;
use single_turn::SingleTurnModels;

#[derive(Clone)]
struct AppState {
    single_turn_models: Arc<SingleTurnModels>,
    redis_client: Option<redis::Client>,
}

async fn health() -> impl IntoResponse {
    tracing::trace!("health called");
    "Ok"
}

async fn generate(
    State(app_state): State<AppState>,
    Json(request): Json<single_turn::GenerateRequest>,
) -> Result<Response> {
    tracing::trace!("generate called");
    let redis_client = app_state.redis_client.clone();
    match app_state
        .single_turn_models
        .generate(redis_client, request)
        .await
    {
        Ok(generation) => Ok(Json(generation).into_response()),
        Err(e) => Ok(e.into_response()),
    }
}

async fn models(State(app_state): State<AppState>) -> Result<Response> {
    tracing::trace!("models called");
    let models = app_state.single_turn_models.models().await?;
    Ok(Json(models).into_response())
}

fn redis_client() -> Option<redis::Client> {
    let redis_url = std::env::var("REDIS_URL").ok()?;
    tracing::info!("Connecting to redis at {}", redis_url);
    let redis_client = redis::Client::open(redis_url).expect("Unable to connect to redis");
    tracing::info!("Connected to redis, using idempotency");
    Some(redis_client)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    logging::init_logging();
    let model_path = std::env::var("MODEL_DIR").unwrap_or_else(|_| "/opt/models/".to_string());

    let single_turn_models = Arc::new(SingleTurnModels::new(model_path)?);
    let redis_client = redis_client();
    let app_state = AppState {
        single_turn_models,
        redis_client,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/generate", post(generate))
        .with_state(app_state.clone())
        .route("/models", get(models))
        .with_state(app_state);

    let address = &"0.0.0.0:8000".parse().unwrap();
    tracing::debug!("listening on {}", address);
    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
