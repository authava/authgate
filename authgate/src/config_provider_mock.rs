use crate::types::{AuthGateError, Config, Route};
use async_trait::async_trait;
use std::sync::Arc;

/// Mock implementation of PostgresProvider for testing
#[derive(Clone)]
pub struct MockPostgresProvider {
    routes: Vec<Route>,
}

impl MockPostgresProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Get all routes
    pub async fn get_all_routes(&self) -> Result<Vec<Route>, AuthGateError> {
        Ok(self.routes.clone())
    }

    /// Get a route by ID
    pub async fn get_route_by_id(&self, id: &str) -> Result<Route, AuthGateError> {
        self.routes
            .iter()
            .find(|r| r.id.as_ref().map(|v| v.to_string()) == Some(id.to_string()))
            .cloned()
            .ok_or_else(|| AuthGateError::NotFound(format!("Route with ID {} not found", id)))
    }

    /// Create a new route
    pub async fn create_route(&self, route: Route) -> Result<Route, AuthGateError> {
        Ok(route)
    }

    /// Update an existing route
    pub async fn update_route(&self, route: Route) -> Result<Route, AuthGateError> {
        // Check if the route exists
        let id = route.id.as_ref().map(|v| v.to_string()).ok_or_else(|| {
            AuthGateError::ConfigError("Route ID is required for update".to_string())
        })?;

        if !self
            .routes
            .iter()
            .any(|r| r.id.as_ref().map(|v| v.to_string()) == Some(id.clone()))
        {
            return Err(AuthGateError::NotFound(format!(
                "Route with ID {} not found",
                id
            )));
        }

        Ok(route)
    }

    /// Delete a route
    pub async fn delete_route(&self, id: &str) -> Result<(), AuthGateError> {
        // Check if the route exists
        if !self
            .routes
            .iter()
            .any(|r| r.id.as_ref().map(|v| v.to_string()) == Some(id.to_string()))
        {
            return Err(AuthGateError::NotFound(format!(
                "Route with ID {} not found",
                id
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl crate::config_provider::ConfigProvider for MockPostgresProvider {
    async fn load_config(&self) -> Result<Config, AuthGateError> {
        Ok(Config {
            auth: crate::types::AuthConfig {
                session_url: "https://auth.example.com/session".to_string(),
                login_redirect: "https://auth.example.com/login".to_string(),
            },
            routes: self.routes.clone(),
            cookie_name: Some("session".to_string()),
        })
    }
}
