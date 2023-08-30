use std::sync::Arc;
use vaultrs::{client::{VaultClient, VaultClientSettingsBuilder}, kv2};

use async_trait::async_trait;

#[derive(Clone)]
pub struct Secrets {
    pub secret_manager: Arc<dyn SecretManager + Send + Sync>,
}

impl Default for Secrets {
    fn default() -> Self {
        Self::new()
    }
}

impl Secrets {
    pub fn new() -> Self {
        Self::from_vault().unwrap_or_else(Self::from_env)
    }

    fn from_env() -> Self {
        let secret_manager = Arc::new(EnvSecretManager {});
        Self { secret_manager }
    }

    fn from_vault() -> Option<Self> {
        let address = std::env::var("VAULT_ADDR").ok()?;
        let settings = VaultClientSettingsBuilder::default().address(address).build().unwrap();
        let client = VaultClient::new(settings).unwrap();
        let secret_manager = Arc::new(VaultSecretManager { client });
        Some(Self { secret_manager })
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

pub struct VaultSecretManager {
    client: VaultClient,
}

#[derive(serde::Deserialize)]
struct VaultAPISecret {
    api_key: String
}

#[async_trait]
impl SecretManager for VaultSecretManager {
    async fn get_secret(&self, key: &str) -> Option<String> {
        match kv2::read::<VaultAPISecret>(&self.client, "secret", key).await {
            Ok(secret) => {
                tracing::debug!("Got secret {}", key);
                Some(secret.api_key)
            }
            Err(e) => {
                tracing::error!("Failed to get secret {}: {}", key, e);
                None
            }
        }
    }
}