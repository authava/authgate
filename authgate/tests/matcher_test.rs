#[cfg(test)]
mod tests {
    use authgate::matcher::RouteMatcher;
    use authgate::types::{AuthConfig, Config, RequireConfig, Route};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_route_matching() {
        // Create a test configuration
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
                    require: serde_json::json!({
                        "roles": ["admin"],
                        "permissions": null,
                        "scopes": null,
                        "teams": null
                    }),
                },
                Route {
                    id: None,
                    host: "*.client.example.com".to_string(),
                    path: "/".to_string(),
                    require: serde_json::json!({
                        "roles": null,
                        "permissions": null,
                        "scopes": null,
                        "teams": []
                    }),
                },
            ],
            cookie_name: Some("session".to_string()),
        };

        let config_lock = Arc::new(RwLock::new(config));
        let matcher = RouteMatcher::new(config_lock);

        // Test exact host match
        let route = matcher.match_route("app.example.com", "/admin/users").await;
        assert!(route.is_some());
        assert_eq!(route.unwrap().host, "app.example.com");

        // Test wildcard host match
        let route = matcher.match_route("client1.client.example.com", "/").await;
        assert!(route.is_some());
        assert_eq!(route.unwrap().host, "*.client.example.com");

        // Test no match
        let route = matcher.match_route("other.example.com", "/").await;
        assert!(route.is_none());
    }
}
