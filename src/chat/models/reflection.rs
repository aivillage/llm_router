use crate::{
    chat::{chat_trait::ChatLlm, errors::ModelError, History},
    secret_manager::Secrets,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReflectionModel {
    pub name: String
}

#[async_trait]
impl ChatLlm for ReflectionModel {
    fn name(&self) -> &str {
        &self.name
    }

    fn context_size(&self) -> usize {
        250
    }

    async fn chat(
        &self,
        _secrets: Secrets,
        prompt: String,
        _system: Option<String>,
        history: Vec<History>,
    ) -> Result<String, ModelError> {
        // Ok(format!("history: {}, prompt: {}", history, prompt))
        Ok(format!("prompt: {}", prompt))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReflectionModels {
    // pub models: Vec<ReflectionModel>,
    pub models: HashMap<String, ReflectionModel>,
}

impl ReflectionModels {
    pub fn new<P: AsRef<Path> + Send + Sync>(file: P) -> Self {
        let contents = std::fs::read_to_string(file).unwrap();
        // let models: Vec<ReflectionModel> = serde_json::from_str(&contents).unwrap();
        let models: HashMap<String, ReflectionModel> = serde_json::from_str(&contents).unwrap();
        Self { models }
    }
}
