use crate::{errors::ModelError, secret_manager::Secrets};
use async_trait::async_trait;

#[async_trait]
pub trait SingleTurnLlm {
    fn name(&self) -> &str;
    async fn generate(
        &self,
        secrets: Secrets,
        prompt: String,
        preprompt: Option<String>,
    ) -> Result<String, ModelError>;
}
