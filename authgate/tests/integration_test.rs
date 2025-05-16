#[cfg(test)]
mod tests {
    use authgate::auth::AuthService;
    use authgate::config::ConfigManager;
    use authgate::matcher::RouteMatcher;
    use authgate::types::{AuthConfig, Config, RequireConfig, Route};
    use std::fs::File;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_integration_flow() {
        // This test verifies the basic integration flow between components

        // 1. Create a test configuration file
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.json");

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
            cookie_name: Some("session".to_string()),
        };

        let config_json = serde_json::to_string_pretty(&config).unwrap();
        let mut file = File::create(&config_path).unwrap();
        file.write_all(config_json.as_bytes()).unwrap();

        // 2. Set environment variables for the test
        std::env::set_var("AUTHGATE_CONFIG_BACKEND", "json");
        std::env::set_var("AUTHGATE_CONFIG", config_path.to_str().unwrap());

        // Initialize the config manager
        let config_manager = Arc::new(ConfigManager::new());
        let result = config_manager.load_config().await;
        assert!(result.is_ok());

        // 3. Initialize the route matcher
        let route_matcher = Arc::new(RouteMatcher::new(config_manager.get_config_ref()));

        // 4. Test route matching
        let route = route_matcher
            .match_route("app.example.com", "/admin/dashboard")
            .await;
        assert!(route.is_some());
        assert_eq!(route.as_ref().unwrap().host, "app.example.com");

        // 5. Test wildcard route matching
        let route = route_matcher
            .match_route("client1.client.example.com", "/")
            .await;
        assert!(route.is_some());
        assert_eq!(route.as_ref().unwrap().host, "*.client.example.com");

        // 6. Test no match
        let route = route_matcher.match_route("other.example.com", "/").await;
        assert!(route.is_none());

        // 7. Test auth service initialization
        let auth_service = AuthService::new();

        // 8. Test login redirect creation
        let login_url = "https://auth.example.com/login";
        let original_url = "https://app.example.com/admin/dashboard";
        let redirect_url = auth_service.create_login_redirect(login_url, original_url);
        assert!(redirect_url.starts_with(login_url));
        assert!(redirect_url.contains("next="));

        // 9. Test cookie extraction
        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::COOKIE,
            http::HeaderValue::from_static("session=test-token; other=value"),
        );

        let token = auth_service.extract_session_token(&headers, "session");
        assert_eq!(token, Some("test-token".to_string()));
    }
}
