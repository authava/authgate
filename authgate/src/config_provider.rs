use crate::types::{AuthGateError, Config, RequireConfig, Route};
use async_trait::async_trait;
use std::env;
use std::fs::File;
use std::sync::Arc;
use tracing::{debug, error, info};

/// ConfigProvider trait defines the interface for loading configuration
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    /// Load configuration from the provider
    async fn load_config(&self) -> Result<Config, AuthGateError>;
}

/// Factory for creating the appropriate config provider
pub struct ConfigProviderFactory {
    postgres_provider: Option<PostgresProvider>,
}

impl ConfigProviderFactory {
    /// Create a new config provider based on environment configuration
    pub fn create() -> (Arc<dyn ConfigProvider>, Self) {
        let config_backend = env::var("AUTHGATE_CONFIG_BACKEND")
            .unwrap_or_else(|_| "json".to_string())
            .to_lowercase();

        match config_backend.as_str() {
            "postgres" => {
                let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://postgres:postgres@localhost/authgate".to_string()
                });

                info!(
                    "Using PostgreSQL config provider with database: {}",
                    database_url
                );
                let postgres_provider = PostgresProvider::new(&database_url);
                let provider_arc = Arc::new(postgres_provider.clone());

                (
                    provider_arc,
                    Self {
                        postgres_provider: Some(postgres_provider),
                    },
                )
            }
            _ => {
                let config_path =
                    env::var("AUTHGATE_CONFIG").unwrap_or_else(|_| "authgate.json".to_string());

                info!("Using JSON file config provider with path: {}", config_path);
                (
                    Arc::new(JsonFileProvider::new(&config_path)),
                    Self {
                        postgres_provider: None,
                    },
                )
            }
        }
    }

    /// Get the PostgreSQL provider if available
    pub fn get_postgres_provider(&self) -> Option<PostgresProvider> {
        self.postgres_provider.clone()
    }
}

/// JSON file implementation of ConfigProvider
pub struct JsonFileProvider {
    config_path: String,
}

impl JsonFileProvider {
    /// Create a new JSON file provider
    pub fn new(config_path: &str) -> Self {
        Self {
            config_path: config_path.to_string(),
        }
    }
}

#[async_trait]
impl ConfigProvider for JsonFileProvider {
    async fn load_config(&self) -> Result<Config, AuthGateError> {
        debug!("Loading configuration from file: {}", self.config_path);

        let file = File::open(&self.config_path).map_err(|e| {
            error!("Failed to open config file: {}", e);
            AuthGateError::ConfigError(format!("Failed to open config file: {}", e))
        })?;

        let config: Config = serde_json::from_reader(file).map_err(|e| {
            error!("Failed to parse config file: {}", e);
            AuthGateError::ConfigError(format!("Failed to parse config file: {}", e))
        })?;

        validate_config(&config)?;

        debug!("Loaded configuration from file: {:?}", config);
        Ok(config)
    }
}

/// PostgreSQL implementation of ConfigProvider
#[derive(Clone)]
pub struct PostgresProvider {
    database_url: String,
}

impl PostgresProvider {
    /// Create a new PostgreSQL provider
    pub fn new(database_url: &str) -> Self {
        Self {
            database_url: database_url.to_string(),
        }
    }

    /// Get all routes from the database
    pub async fn get_all_routes(&self) -> Result<Vec<Route>, AuthGateError> {
        #[cfg(feature = "postgres")]
        {
            // Connect to the database
            let pool = sqlx::PgPool::connect(&self.database_url)
                .await
                .map_err(|e| {
                    error!("Failed to connect to database: {}", e);
                    AuthGateError::DatabaseError(format!("Failed to connect to database: {}", e))
                })?;

            // Query all routes
            let rows = sqlx::query!(
                r#"
                SELECT
                    id,
                    host,
                    path,
                    require
                FROM routes
                ORDER BY host, path
                "#
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                error!("Failed to query routes: {}", e);
                AuthGateError::DatabaseError(format!("Failed to query routes: {}", e))
            })?;

            let routes = rows
                .into_iter()
                .map(|row| {
                    let require: RequireConfig =
                        serde_json::from_value(row.require).map_err(|e| {
                            error!("Failed to parse require JSON: {}", e);
                            AuthGateError::ConfigError(format!(
                                "Failed to parse require JSON: {}",
                                e
                            ))
                        })?;

                    Ok(Route {
                        id: Some(row.id),
                        host: row.host,
                        path: row.path,
                        require: serde_json::to_value(require).map_err(|e| {
                            error!("Failed to serialize require config: {}", e);
                            AuthGateError::ConfigError(format!(
                                "Failed to serialize require config: {}",
                                e
                            ))
                        })?,
                    })
                })
                .collect::<Result<Vec<_>, AuthGateError>>()?;

            Ok(routes)
        }

        #[cfg(not(feature = "postgres"))]
        {
            // Return empty routes for testing
            Ok(Vec::new())
        }
    }

    /// Get a route by ID
    pub async fn get_route_by_id(&self, id: &i32) -> Result<Route, AuthGateError> {
        #[cfg(feature = "postgres")]
        {
            // Connect to the database
            let pool = sqlx::PgPool::connect(&self.database_url)
                .await
                .map_err(|e| {
                    error!("Failed to connect to database: {}", e);
                    AuthGateError::DatabaseError(format!("Failed to connect to database: {}", e))
                })?;

            // Query the raw values
            let row = sqlx::query!(
                r#"
                SELECT
                    id,
                    host,
                    path,
                    require
                FROM routes
                WHERE id = $1
                "#,
                id
            )
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                error!("Failed to query route: {}", e);
                AuthGateError::DatabaseError(format!("Failed to query route: {}", e))
            })?;

            match row {
                Some(row) => {
                    let require: RequireConfig =
                        serde_json::from_value(row.require).map_err(|e| {
                            error!("Failed to parse require JSON: {}", e);
                            AuthGateError::ConfigError(format!(
                                "Failed to parse require JSON: {}",
                                e
                            ))
                        })?;

                    Ok(Route {
                        id: Some(row.id),
                        host: row.host,
                        path: row.path,
                        require: serde_json::to_value(require).map_err(|e| {
                            error!("Failed to serialize require config: {}", e);
                            AuthGateError::ConfigError(format!(
                                "Failed to serialize require config: {}",
                                e
                            ))
                        })?,
                    })
                }
                None => Err(AuthGateError::NotFound(format!(
                    "Route with ID {} not found",
                    id
                ))),
            }
        }

        #[cfg(not(feature = "postgres"))]
        {
            // Return a mock route for testing
            Ok(Route {
                id: Some(id.to_string()),
                host: "api.example.com".to_string(),
                path: "/api".to_string(),
                require: RequireConfig {
                    roles: Some(vec!["admin".to_string()]),
                    permissions: None,
                    scopes: None,
                    teams: None,
                },
            })
        }
    }

    /// Create a new route
    pub async fn create_route(&self, route: Route) -> Result<Route, AuthGateError> {
        #[cfg(feature = "postgres")]
        {
            // Connect to the database
            let pool = sqlx::PgPool::connect(&self.database_url)
                .await
                .map_err(|e| {
                    error!("Failed to connect to database: {}", e);
                    AuthGateError::DatabaseError(format!("Failed to connect to database: {}", e))
                })?;

            // Serialize `require` into JSON
            let require_json = serde_json::to_value(&route.require).map_err(|e| {
                error!("Failed to serialize require config: {}", e);
                AuthGateError::ConfigError(format!("Failed to serialize require config: {}", e))
            })?;

            // Insert and return raw row
            let row = sqlx::query!(
                r#"
            INSERT INTO routes (host, path, require)
            VALUES ($1, $2, $3)
            RETURNING id, host, path, require
            "#,
                route.host,
                route.path,
                require_json
            )
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                error!("Failed to create route: {}", e);
                AuthGateError::DatabaseError(format!("Failed to create route: {}", e))
            })?;

            // Deserialize require JSON
            let require: RequireConfig = serde_json::from_value(row.require).map_err(|e| {
                error!("Failed to parse require JSON: {}", e);
                AuthGateError::ConfigError(format!("Failed to parse require JSON: {}", e))
            })?;

            Ok(Route {
                id: Some(row.id),
                host: row.host,
                path: row.path,
                require: serde_json::to_value(require).map_err(|e| {
                    error!("Failed to serialize require config: {}", e);
                    AuthGateError::ConfigError(format!("Failed to serialize require config: {}", e))
                })?,
            })
        }

        #[cfg(not(feature = "postgres"))]
        {
            Ok(route)
        }
    }

    /// Update an existing route
    pub async fn update_route(&self, route: Route) -> Result<Route, AuthGateError> {
        #[cfg(feature = "postgres")]
        {
            let pool = sqlx::PgPool::connect(&self.database_url)
                .await
                .map_err(|e| {
                    error!("Failed to connect to database: {}", e);
                    AuthGateError::DatabaseError(format!("Failed to connect to database: {}", e))
                })?;

            let require_json = serde_json::to_value(&route.require).map_err(|e| {
                error!("Failed to serialize require config: {}", e);
                AuthGateError::ConfigError(format!("Failed to serialize require config: {}", e))
            })?;

            let row = sqlx::query!(
                r#"
                UPDATE routes
                SET host = $2, path = $3, require = $4
                WHERE id = $1
                RETURNING id, host, path, require
                "#,
                route.id,
                route.host,
                route.path,
                require_json
            )
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                error!("Failed to update route: {}", e);
                AuthGateError::DatabaseError(format!("Failed to update route: {}", e))
            })?;

            match row {
                Some(row) => {
                    let require: RequireConfig =
                        serde_json::from_value(row.require).map_err(|e| {
                            error!("Failed to parse require JSON: {}", e);
                            AuthGateError::ConfigError(format!(
                                "Failed to parse require JSON: {}",
                                e
                            ))
                        })?;

                    Ok(Route {
                        id: Some(row.id),
                        host: row.host,
                        path: row.path,
                        require: serde_json::to_value(require).map_err(|e| {
                            error!("Failed to serialize require config: {}", e);
                            AuthGateError::ConfigError(format!(
                                "Failed to serialize require config: {}",
                                e
                            ))
                        })?,
                    })
                }
                None => Err(AuthGateError::NotFound(format!(
                    "Route with ID {} not found",
                    route.id.unwrap_or_default()
                ))),
            }
        }

        #[cfg(not(feature = "postgres"))]
        {
            Ok(route)
        }
    }

    /// Delete a route
    pub async fn delete_route(&self, id: &i32) -> Result<(), AuthGateError> {
        #[allow(unused_variables)]
        #[cfg(feature = "postgres")]
        {
            // Connect to the database
            let pool = sqlx::PgPool::connect(&self.database_url)
                .await
                .map_err(|e| {
                    error!("Failed to connect to database: {}", e);
                    AuthGateError::DatabaseError(format!("Failed to connect to database: {}", e))
                })?;

            // Delete the route
            let result = sqlx::query!(
                r#"
                DELETE FROM routes
                WHERE id = $1
                "#,
                id
            )
            .execute(&pool)
            .await
            .map_err(|e| {
                error!("Failed to delete route: {}", e);
                AuthGateError::DatabaseError(format!("Failed to delete route: {}", e))
            })?;

            // Check if the route exists
            if result.rows_affected() == 0 {
                return Err(AuthGateError::NotFound(format!(
                    "Route with ID {} not found",
                    id
                )));
            }

            Ok(())
        }

        #[cfg(not(feature = "postgres"))]
        {
            // Return success for testing
            Ok(())
        }
    }
}

#[async_trait]
impl ConfigProvider for PostgresProvider {
    async fn load_config(&self) -> Result<Config, AuthGateError> {
        debug!("Loading configuration from PostgreSQL database");

        // Create a connection pool
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.database_url)
            .await
            .map_err(|e| {
                error!("Failed to connect to database: {}", e);
                AuthGateError::ConfigError(format!("Failed to connect to database: {}", e))
            })?;

        // Load auth configuration
        let auth_config = sqlx::query_as::<_, (String, String, Option<String>)>(
            "SELECT session_url, login_redirect, cookie_name FROM auth_config LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!("Failed to load auth configuration from database: {}", e);
            AuthGateError::ConfigError(format!(
                "Failed to load auth configuration from database: {}",
                e
            ))
        })?;

        // Load routes
        let routes = sqlx::query_as::<_, (String, String, serde_json::Value)>(
            "SELECT host, path, require FROM routes",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!("Failed to load routes from database: {}", e);
            AuthGateError::ConfigError(format!("Failed to load routes from database: {}", e))
        })?;

        // Parse routes from JSON
        let mut parsed_routes = Vec::new();
        for (host, path, require_json) in routes {
            let host_clone = host.clone();
            let require: crate::types::RequireConfig = serde_json::from_value(require_json)
                .map_err(|e| {
                    error!(
                        "Failed to parse require JSON for route {}: {}",
                        host_clone, e
                    );
                    AuthGateError::ConfigError(format!("Failed to parse require JSON: {}", e))
                })?;

            let host_clone2 = host.clone();
            parsed_routes.push(crate::types::Route {
                id: None, // No ID for routes loaded from JSON
                host,
                path,
                require: serde_json::to_value(&require).map_err(|e| {
                    error!(
                        "Failed to convert require config to JSON for route {}: {}",
                        host_clone2, e
                    );
                    AuthGateError::ConfigError(format!(
                        "Failed to convert require config to JSON: {}",
                        e
                    ))
                })?,
            });
        }

        // Create the config
        let (session_url, login_redirect, cookie_name) = auth_config;
        let config = Config {
            auth: crate::types::AuthConfig {
                session_url,
                login_redirect,
            },
            routes: parsed_routes,
            cookie_name,
        };

        validate_config(&config)?;

        debug!("Loaded configuration from PostgreSQL: {:?}", config);
        Ok(config)
    }
}

/// Validate the configuration
fn validate_config(config: &Config) -> Result<(), AuthGateError> {
    // Validate auth configuration
    if config.auth.session_url.is_empty() {
        return Err(AuthGateError::ConfigError(
            "session_url cannot be empty".to_string(),
        ));
    }

    if config.auth.login_redirect.is_empty() {
        return Err(AuthGateError::ConfigError(
            "login_redirect cannot be empty".to_string(),
        ));
    }

    // Validate routes
    if config.routes.is_empty() {
        return Err(AuthGateError::ConfigError(
            "At least one route must be defined".to_string(),
        ));
    }

    for (i, route) in config.routes.iter().enumerate() {
        if route.host.is_empty() {
            return Err(AuthGateError::ConfigError(format!(
                "Host cannot be empty for route {}",
                i
            )));
        }

        // Validate require block has at least one requirement
        let require = &route.require;
        let has_requirements = require.get("roles").is_some()
            || require.get("permissions").is_some()
            || require.get("scopes").is_some()
            || require.get("teams").is_some();

        if !has_requirements {
            return Err(AuthGateError::ConfigError(format!(
                "Route {} must have at least one requirement",
                i
            )));
        }
    }

    Ok(())
}
