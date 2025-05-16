#[cfg(test)]
mod tests {
    use authgate::config::{ConfigManager, DEFAULT_COOKIE_NAME};
    use authgate::config_provider::{ConfigProvider, JsonFileProvider};
    use authgate::types::{AuthConfig, Config, RequireConfig, Route};
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_json_provider_config_loading() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.json");

        // Create a test config file
        let config = Config {
            auth: AuthConfig {
                session_url: "https://auth.example.com/session".to_string(),
                login_redirect: "https://auth.example.com/login".to_string(),
            },
            routes: vec![
                Route {
                    id: None,
                    host: "app.example.com".to_string(),
                    path: "/admin/*".to_string(),
                    require: serde_json::to_value(RequireConfig {
                        roles: Some(vec!["admin".to_string()]),
                        permissions: None,
                        scopes: None,
                        teams: None,
                    })
                    .unwrap(),
                },
                Route {
                    id: None,
                    host: "*.client.example.com".to_string(),
                    path: "/".to_string(),
                    require: serde_json::to_value(RequireConfig {
                        roles: None,
                        permissions: None,
                        scopes: None,
                        teams: Some(vec![]),
                    })
                    .unwrap(),
                },
            ],
            cookie_name: Some("custom-session".to_string()),
        };

        let config_json = serde_json::to_string_pretty(&config).unwrap();
        let mut file = File::create(&config_path).unwrap();
        file.write_all(config_json.as_bytes()).unwrap();

        // Create a JSON file provider and load the config
        let provider = JsonFileProvider::new(config_path.to_str().unwrap());
        let result = provider.load_config().await;

        // Check that the config was loaded successfully
        assert!(result.is_ok());

        // Check that the config values were loaded correctly
        let loaded_config = result.unwrap();
        assert_eq!(
            loaded_config.auth.session_url,
            "https://auth.example.com/session"
        );
        assert_eq!(
            loaded_config.auth.login_redirect,
            "https://auth.example.com/login"
        );
        assert_eq!(loaded_config.routes.len(), 2);
        assert_eq!(loaded_config.routes[0].host, "app.example.com");
        assert_eq!(loaded_config.routes[1].host, "*.client.example.com");
        assert_eq!(
            loaded_config.cookie_name,
            Some("custom-session".to_string())
        );
    }

    #[tokio::test]
    async fn test_config_manager_with_json_provider() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.json");

        // Create a test config file
        let config = Config {
            auth: AuthConfig {
                session_url: "https://auth.example.com/session".to_string(),
                login_redirect: "https://auth.example.com/login".to_string(),
            },
            routes: vec![Route {
                id: None,
                host: "app.example.com".to_string(),
                path: "/admin/*".to_string(),
                require: serde_json::to_value(RequireConfig {
                    roles: Some(vec!["admin".to_string()]),
                    permissions: None,
                    scopes: None,
                    teams: None,
                })
                .unwrap(),
            }],
            cookie_name: Some("custom-session".to_string()),
        };

        let config_json = serde_json::to_string_pretty(&config).unwrap();
        let mut file = File::create(&config_path).unwrap();
        file.write_all(config_json.as_bytes()).unwrap();

        // Set environment variables for the test
        std::env::set_var("AUTHGATE_CONFIG_BACKEND", "json");
        std::env::set_var("AUTHGATE_CONFIG", config_path.to_str().unwrap());

        // Create a config manager and load the config
        let config_manager = ConfigManager::new();
        let result = config_manager.load_config().await;

        // Check that the config was loaded successfully
        assert!(result.is_ok());

        // Check that the config values were loaded correctly
        let loaded_config = config_manager.get_config().await;
        assert_eq!(
            loaded_config.auth.session_url,
            "https://auth.example.com/session"
        );
        assert_eq!(
            loaded_config.cookie_name,
            Some("custom-session".to_string())
        );

        // Check cookie name getter
        let cookie_name = config_manager.get_cookie_name().await;
        assert_eq!(cookie_name, "custom-session");
    }

    #[tokio::test]
    async fn test_default_cookie_name() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.json");

        // Create a test config file without cookie_name
        let config = Config {
            auth: AuthConfig {
                session_url: "https://auth.example.com/session".to_string(),
                login_redirect: "https://auth.example.com/login".to_string(),
            },
            routes: vec![Route {
                id: None,
                host: "app.example.com".to_string(),
                path: "/admin/*".to_string(),
                require: serde_json::to_value(RequireConfig {
                    roles: Some(vec!["admin".to_string()]),
                    permissions: None,
                    scopes: None,
                    teams: None,
                })
                .unwrap(),
            }],
            cookie_name: None,
        };

        let config_json = serde_json::to_string_pretty(&config).unwrap();
        let mut file = File::create(&config_path).unwrap();
        file.write_all(config_json.as_bytes()).unwrap();

        // Set environment variables for the test
        std::env::set_var("AUTHGATE_CONFIG_BACKEND", "json");
        std::env::set_var("AUTHGATE_CONFIG", config_path.to_str().unwrap());

        // Create a config manager and load the config
        let config_manager = ConfigManager::new();
        let result = config_manager.load_config().await;

        // Check that the config was loaded successfully
        assert!(result.is_ok());

        // Check that the default cookie name is used
        let loaded_config = config_manager.get_config().await;
        assert!(loaded_config.cookie_name.is_some());

        // Check cookie name getter
        let cookie_name = config_manager.get_cookie_name().await;
        assert!(!cookie_name.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_config() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid-config.json");

        // Create an invalid JSON file
        let mut file = File::create(&config_path).unwrap();
        file.write_all(b"{invalid json}").unwrap();

        // Set environment variables for the test
        std::env::set_var("AUTHGATE_CONFIG_BACKEND", "json");
        std::env::set_var("AUTHGATE_CONFIG", config_path.to_str().unwrap());

        // Create a config manager and try to load the config
        let config_manager = ConfigManager::new();
        let result = config_manager.load_config().await;

        // Check that loading failed
        assert!(result.is_err());
    }
}
