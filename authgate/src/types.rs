use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgTypeInfo, Decode, Postgres, Type};

/// Main configuration structure for authgate
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub auth: AuthConfig,
    pub routes: Vec<Route>,
    #[serde(default)]
    pub cookie_name: Option<String>,
}

/// Authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub session_url: String,
    pub login_redirect: String,
}

/// Route definition with matching criteria and requirements
#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Route {
    #[serde(default)]
    pub id: Option<i32>,
    pub host: String,
    pub path: String,
    pub require: serde_json::Value,
}

/// Authorization requirements for a route
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequireConfig {
    #[serde(default)]
    pub roles: Option<Vec<String>>,
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
    #[serde(default)]
    pub scopes: Option<Vec<ScopeRequirement>>,
    #[serde(default)]
    pub teams: Option<Vec<TeamRequirement>>,
}

/// Scope requirement definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScopeRequirement {
    pub resource_type: String,
    pub action: String,
    #[serde(default)]
    pub resource_id: Option<String>,
}

impl<'r> Decode<'r, sqlx::Postgres> for RequireConfig {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let json: serde_json::Value = Decode::decode(value)?;
        let cfg = serde_json::from_value(json)?;
        Ok(cfg)
    }
}

impl Type<Postgres> for RequireConfig {
    fn type_info() -> PgTypeInfo {
        <serde_json::Value as Type<Postgres>>::type_info()
    }
}

/// Team requirement definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TeamRequirement {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub scopes: Option<Vec<ScopeRequirement>>,
}

/// Session response from the authentication service
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionResponse {
    pub user: User,
    pub tenant_id: String,
    pub authority: String,
    #[serde(default)]
    pub redirect_url: Option<String>,
}

/// User information in the session
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub teams: Vec<Team>,
}

/// Team information in the user session
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub is_owner: bool,
    pub scopes: Vec<Scope>,
}

/// Scope definition in a team
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Scope {
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
}

/// Result of an authorization check
#[derive(Debug, Clone)]
pub enum AuthResult {
    Authorized,
    Unauthorized(String),
    Unauthenticated,
    Error(String),
}

/// Request context containing parsed information
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub original_url: String,
    pub host: String,
    pub path: String,
    pub session_token: Option<String>,
    pub session: Option<SessionResponse>,
    pub matched_route: Option<Route>,
}

/// Error types for the application
#[derive(Debug, thiserror::Error)]
pub enum AuthGateError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Proxy error: {0}")]
    ProxyError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
