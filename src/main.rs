use axum::{response::IntoResponse, routing::get, Router};
pub mod chat;
pub mod logging;
pub mod secret_manager;

use crate::secret_manager::Secrets;

#[derive(Clone)]
pub struct AppState {
    redis_client: Option<redis::Client>,
    secret_manager: Secrets,
}

async fn health() -> impl IntoResponse {
    tracing::trace!("health called");
    "Ok"
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

    let redis_client = redis_client();
    let secret_manager = Secrets::from_env();

    let app_state = AppState {
        redis_client,
        secret_manager,
    };

    let chat_router = chat::chat_router(app_state.clone()).await?;

    let app = Router::new()
        .route("/health", get(health))
        .nest("/chat", chat_router);

    let address = &"0.0.0.0:8000".parse().unwrap();
    tracing::info!("listening on {}", address);
    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
