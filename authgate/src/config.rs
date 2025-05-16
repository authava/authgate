use crate::config_provider::{ConfigProviderFactory, PostgresProvider};
use crate::types::{AuthGateError, Config};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Default cookie name if not specified in config
pub const DEFAULT_COOKIE_NAME: &str = "session";

/// ConfigManager handles loading and reloading of configuration
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_provider: Arc<dyn crate::config_provider::ConfigProvider>,
    provider_factory: Option<ConfigProviderFactory>,
}

impl ConfigManager {
    /// Create a new ConfigManager
    pub fn new() -> Self {
        // Create a config provider based on environment
        let (config_provider, provider_factory) = ConfigProviderFactory::create();

        Self {
            config: Arc::new(RwLock::new(Config {
                auth: crate::types::AuthConfig {
                    session_url: String::new(),
                    login_redirect: String::new(),
                },
                routes: Vec::new(),
                cookie_name: None,
            })),
            config_provider,
            provider_factory: Some(provider_factory),
        }
    }

    /// Load configuration from the provider
    pub async fn load_config(&self) -> Result<(), AuthGateError> {
        let config = self.config_provider.load_config().await?;

        // Set default cookie name if not specified
        let config = Config {
            cookie_name: config.cookie_name.or(Some(DEFAULT_COOKIE_NAME.to_string())),
            ..config
        };

        let mut writable_config = self.config.write().await;
        *writable_config = config;

        info!("Configuration loaded successfully");
        Ok(())
    }

    /// Get a clone of the current configuration
    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    /// Get a clone of the current configuration synchronously
    pub fn get_config_sync(&self) -> Config {
        // Use blocking to get the config synchronously
        // This is safe because the lock is held for a very short time
        let config = self.config.blocking_read();
        config.clone()
    }

    /// Get the cookie name from configuration
    pub async fn get_cookie_name(&self) -> String {
        self.config
            .read()
            .await
            .cookie_name
            .clone()
            .unwrap_or_else(|| DEFAULT_COOKIE_NAME.to_string())
    }

    /// Get the PostgreSQL provider if available
    pub fn get_postgres_provider(&self) -> Option<PostgresProvider> {
        // Check if we're using a PostgreSQL provider
        if let Some(provider_factory) = &self.provider_factory {
            if let Some(postgres_provider) = provider_factory.get_postgres_provider() {
                return Some(postgres_provider);
            }
        }

        debug!("PostgreSQL provider not available");
        None
    }

    /// Get a reference to the config for sharing
    pub fn get_config_ref(&self) -> Arc<RwLock<Config>> {
        self.config.clone()
    }
}

/// Setup config watcher for reloading
#[cfg(feature = "config_reload")]
pub async fn setup_config_watcher(config_manager: Arc<ConfigManager>) -> Result<(), AuthGateError> {
    use std::time::Duration;

    // For now, we'll just periodically reload the config
    // In the future, this could be enhanced to watch for changes in different ways
    // depending on the config provider type

    tokio::spawn(async move {
        loop {
            // Sleep for a while before checking for changes
            tokio::time::sleep(Duration::from_secs(60)).await;

            info!("Checking for configuration changes...");
            if let Err(e) = config_manager.load_config().await {
                error!("Failed to reload config: {}", e);
            } else {
                info!("Config reloaded successfully");
            }
        }
    });

    Ok(())
}
