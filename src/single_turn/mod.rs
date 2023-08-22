pub mod mock;
pub mod huggingface;
pub mod single_turn_trait;
use crate::errors::ModelError;
use crate::secret_manager;
use anyhow::Context;
use mock::MockModels;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use single_turn_trait::SingleTurnLlm;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub preprompt: Option<String>,
    pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub generation: String,
    pub uuid: String,
}

impl GenerateResponse {
    fn to_redis_string(&self) -> String {
        format!("OK:{}", self.generation)
    }

    fn from_redis_string(s: &str, uuid: &str) -> Option<Self> {
        let (status, generation) = s.split_once(':')?;
        
        
        if status == "OK" {
            Some(Self {
                generation: generation.to_string(),
                uuid: uuid.to_string(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub models: Vec<String>,
}

pub struct SingleTurnModels {
    models: HashMap<String, Box<dyn SingleTurnLlm + Send + Sync>>,
}

impl SingleTurnModels {
    pub fn new<P: AsRef<Path> + Send + Sync>(models_path: P) -> anyhow::Result<Self> {
        let mut models: HashMap<String, Box<dyn SingleTurnLlm + Send + Sync>> = HashMap::new();

        for file in std::fs::read_dir(&models_path).with_context(|| {
            format!(
                "Failed to read directory {} while opening models",
                models_path.as_ref().display()
            )
        })? {
            let file = file.unwrap();
            let path = file.path();
            match path.file_name().and_then(|s| s.to_str()) {
                Some("mock.json") => {
                    tracing::info!("Found mock.json, loading mock models");
                    let mock_models = MockModels::new(&path);
                    for (name, model) in mock_models.models {
                        models.insert(name, Box::new(model));
                    }
                }
                Some("huggingface.json") => {
                    tracing::info!("Found huggingface.json, loading huggingface models");
                    let huggingface_models = huggingface::HuggingFaceModels::new(&path)?;
                    for model in huggingface_models.models {
                        models.insert(model.name.clone(), Box::new(model));
                    }
                }
                _ => {}
            }
        }

        Ok(Self { models })
    }

    async fn check_cache(
        &self,
        redis_client: &mut redis::Client,
        uuid: &str,
    ) -> anyhow::Result<Option<Result<GenerateResponse, ModelError>>> {
        let mut redis_connection = redis_client
            .get_async_connection()
            .await
            .context("Failure to get redis connection")?;
        let cached_generation: Option<String> = redis_connection
            .get(uuid)
            .await
            .context("Failure to get cached generation")?;

        Ok(cached_generation.map(|generation| {
            tracing::debug!("Found cached generation for {}", uuid);
            if generation.starts_with("ERR:") {
                let error = ModelError::from_redis_string(&generation).unwrap();
                Err(error)
            } else {
                Ok(GenerateResponse::from_redis_string(&generation, uuid).unwrap())
            }
        }))
    }

    async fn cache_generation(
        &self,
        redis_client: &mut redis::Client,
        uuid: &str,
        generation: &Result<GenerateResponse, ModelError>,
    ) -> anyhow::Result<()> {
        let mut redis_connection = redis_client
            .get_async_connection()
            .await
            .context("Failed to get redis connection")?;

        let generation = match generation {
            Ok(generation) => generation.to_redis_string(),
            Err(e) => e.to_redis_string(),
        };
        tracing::debug!("Caching generation for: {}", uuid);

        // If they're looking for this generation after an hour, something has gone wrong
        redis_connection
            .set_ex(uuid, generation, 60 * 60)
            .await
            .context("Failed to set cached generation")?;
        Ok(())
    }

    pub async fn generate(
        &self,
        mut redis_client: Option<redis::Client>,
        secret_manager: secret_manager::Secrets,
        request: GenerateRequest,
    ) -> Result<GenerateResponse, ModelError> {
        if let Some(redis_client) = &mut redis_client {
            let cached_generation = self
                .check_cache(redis_client, &request.uuid)
                .await
                .map_err(|e| tracing::error!("Idempotency error: {:?}", e))
                .unwrap_or(None);
            if let Some(generation) = cached_generation {
                return generation;
            }
        }

        let generation = match self.models.get(request.model.as_str()) {
            Some(model) => model.generate(secret_manager, request.prompt, request.preprompt).await,
            None => Err(ModelError::ModelNotFound),
        };
        let response = match generation {
            Ok(generation) => Ok(GenerateResponse {
                generation,
                uuid: request.uuid.clone(),
            }),
            Err(e) => Err(e),
        };
        if let Some(redis_client) = &mut redis_client {
            self.cache_generation(redis_client, &request.uuid, &response)
                .await
                .map_err(|e| tracing::error!("Failed to cache generation: {:?}", e))
                .unwrap_or(());
        }
        response
    }

    pub async fn models(&self) -> Result<ModelsResponse, ModelError> {
        let models: Vec<String> = self.models.keys().map(|s| s.to_string()).collect();
        Ok(ModelsResponse { models })
    }
}
