use std::sync::Arc;

use async_trait::async_trait;

#[derive(Clone)]
pub struct Secrets {
    pub secret_manager: Arc<dyn SecretManager + Send + Sync>,
}

impl Secrets {
    pub fn from_env() -> Self {
        let secret_manager = Arc::new(EnvSecretManager {});
        Self { secret_manager }
    }

    pub async fn get_secret(&self, key: &str) -> Option<String> {
        self.secret_manager.get_secret(key).await
    }
}

#[async_trait]
pub trait SecretManager {
    async fn get_secret(&self, key: &str) -> Option<String>;
}

pub struct EnvSecretManager {}

#[async_trait]
impl SecretManager for EnvSecretManager {
    async fn get_secret(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}
