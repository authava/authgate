#[cfg(test)]
mod tests {
    use authgate::config_provider::{ConfigProvider, JsonFileProvider};
    use authgate::types::{AuthConfig, Config, RequireConfig, Route};
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_json_file_provider() {
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

        // Create a config provider and load the config
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

    // This test is marked as ignored by default because it requires a PostgreSQL server
    // To run it: cargo test -- --ignored
    #[tokio::test]
    #[ignore]
    async fn test_postgres_provider() {
        use authgate::config_provider::PostgresProvider;
        use std::env;

        // Skip if DATABASE_URL is not set
        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping PostgreSQL test because DATABASE_URL is not set");
                return;
            }
        };

        // Create a config provider and load the config
        let provider = PostgresProvider::new(&database_url);
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
        assert!(loaded_config.routes.len() >= 1);

        // Check that at least one route was loaded
        let route = &loaded_config.routes[0];
        assert!(!route.host.is_empty());
        assert!(!route.path.is_empty());
    }
}
