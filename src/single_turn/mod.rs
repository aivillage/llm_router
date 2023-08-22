pub mod single_turn_trait;
pub mod mock;
use single_turn_trait::SingleTurnLlm;
use mock::MockModels;
use crate::errors::ModelError;
use anyhow::Context;
use serde::{Deserialize, Serialize};
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
                _ => {}
            }
        }
        
        Ok(Self { models })
    }

    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse, ModelError> {
        let generation = match self.models.get(request.model.as_str()) {
            Some(model) => model.generate(request.prompt, request.preprompt).await,
            None => Err(ModelError::ModelNotFound),
        }?;
        Ok(GenerateResponse {
            generation,
            uuid: request.uuid,
        })
    }

    pub async fn models(&self) -> Result<ModelsResponse, ModelError> {
        let models: Vec<String> = self.models.keys().map(|s| s.to_string()).collect();
        Ok(ModelsResponse { models })
    }
}
