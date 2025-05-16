#[cfg(test)]
mod tests {
    use authgate::auth::AuthService;
    use authgate::types::{
        AuthResult, RequestContext, RequireConfig, Route, Scope, ScopeRequirement, SessionResponse,
        Team, TeamRequirement, User,
    };

    #[test]
    fn test_role_authorization() {
        let auth_service = AuthService::new();

        // Create a test session with admin role
        let session = create_test_session(vec!["admin".to_string(), "user".to_string()], vec![]);

        // Create a route requiring admin role
        let route = Route {
            id: None,

            host: "app.example.com".to_string(),
            path: "/admin/*".to_string(),
            require: serde_json::json!({
                "roles": ["admin"],
                "permissions": null,
                "scopes": null,
                "teams": null
            }),
        };

        // Create request context
        let ctx = RequestContext {
            original_url: "https://app.example.com/admin/dashboard".to_string(),
            host: "app.example.com".to_string(),
            path: "/admin/dashboard".to_string(),
            session_token: Some("test-token".to_string()),
            session: Some(session),
            matched_route: Some(route),
        };

        // Test authorization
        match auth_service.authorize(&ctx) {
            AuthResult::Authorized => {
                // Test passed
            }
            other => panic!("Expected Authorized, got {:?}", other),
        }
    }

    #[test]
    fn test_role_authorization_failure() {
        let auth_service = AuthService::new();

        // Create a test session with only user role (no admin)
        let session = create_test_session(vec!["user".to_string()], vec![]);

        // Create a route requiring admin role
        let route = Route {
            id: None,

            host: "app.example.com".to_string(),
            path: "/admin/*".to_string(),
            require: serde_json::json!({
                "roles": ["admin"],
                "permissions": null,
                "scopes": null,
                "teams": null
            }),
        };

        // Create request context
        let ctx = RequestContext {
            original_url: "https://app.example.com/admin/dashboard".to_string(),
            host: "app.example.com".to_string(),
            path: "/admin/dashboard".to_string(),
            session_token: Some("test-token".to_string()),
            session: Some(session),
            matched_route: Some(route),
        };

        // Test authorization
        match auth_service.authorize(&ctx) {
            AuthResult::Unauthorized(_) => {
                // Test passed
            }
            other => panic!("Expected Unauthorized, got {:?}", other),
        }
    }

    #[test]
    fn test_permission_authorization() {
        let auth_service = AuthService::new();

        // Create a test session with users:read permission
        let session =
            create_test_session(vec!["admin".to_string()], vec!["users:read".to_string()]);

        // Create a route requiring users:read permission
        let route = Route {
            id: None,

            host: "app.example.com".to_string(),
            path: "/api/users".to_string(),
            require: serde_json::json!({
                "roles": ["admin"],
                "permissions": null,
                "scopes": null,
                "teams": null
            }),
        };

        // Create request context
        let ctx = RequestContext {
            original_url: "https://app.example.com/api/users".to_string(),
            host: "app.example.com".to_string(),
            path: "/api/users".to_string(),
            session_token: Some("test-token".to_string()),
            session: Some(session),
            matched_route: Some(route),
        };

        // Test authorization
        match auth_service.authorize(&ctx) {
            AuthResult::Authorized => {
                // Test passed
            }
            other => panic!("Expected Authorized, got {:?}", other),
        }
    }

    #[test]
    fn test_scope_authorization() {
        let auth_service = AuthService::new();

        // Create a test session with report:view scope
        let mut session = create_test_session(vec![], vec![]);
        session.user.teams[0].scopes.push(Scope {
            resource_type: "report".to_string(),
            resource_id: "123".to_string(),
            action: "view".to_string(),
        });

        // Create a route requiring report:view scope
        let route = Route {
            id: None,

            host: "app.example.com".to_string(),
            path: "/reports".to_string(),
            require: serde_json::json!({
                "roles": null,
                "permissions": null,
                "scopes": [{
                    "resource_type": "report",
                    "action": "view",
                    "resource_id": "123"
                }],
                "teams": null
            }),
        };

        // Create request context
        let ctx = RequestContext {
            original_url: "https://app.example.com/reports".to_string(),
            host: "app.example.com".to_string(),
            path: "/reports".to_string(),
            session_token: Some("test-token".to_string()),
            session: Some(session),
            matched_route: Some(route),
        };

        // Test authorization
        match auth_service.authorize(&ctx) {
            AuthResult::Authorized => {
                // Test passed
            }
            other => panic!("Expected Authorized, got {:?}", other),
        }
    }

    #[test]
    fn test_team_authorization() {
        let auth_service = AuthService::new();

        // Create a test session with a specific team
        let session = create_test_session(vec![], vec![]);

        // Create a route requiring that team
        let route = Route {
            id: None,

            host: "client.example.com".to_string(),
            path: "/".to_string(),
            require: serde_json::json!({
                "roles": null,
                "permissions": null,
                "scopes": null,
                "teams": [{
                    "id": "team-1",
                    "name": null,
                    "scopes": null
                }]
            }),
        };

        // Create request context
        let ctx = RequestContext {
            original_url: "https://client.example.com/".to_string(),
            host: "client.example.com".to_string(),
            path: "/".to_string(),
            session_token: Some("test-token".to_string()),
            session: Some(session),
            matched_route: Some(route),
        };

        // Test authorization
        match auth_service.authorize(&ctx) {
            AuthResult::Authorized => {
                // Test passed
            }
            other => panic!("Expected Authorized, got {:?}", other),
        }
    }

    #[test]
    fn test_team_with_scope_authorization() {
        let auth_service = AuthService::new();

        // Create a test session with a specific team and client:access scope
        let mut session = create_test_session(vec![], vec![]);
        session.user.teams[0].scopes.push(Scope {
            resource_type: "client".to_string(),
            resource_id: "client-1".to_string(),
            action: "access".to_string(),
        });

        // Create a route requiring that team with client:access scope
        let route = Route {
            id: None,

            host: "client.example.com".to_string(),
            path: "/".to_string(),
            require: serde_json::json!({
                "roles": null,
                "permissions": null,
                "scopes": null,
                "teams": [{
                    "id": "team-1",
                    "name": null,
                    "scopes": [{
                        "resource_type": "client",
                        "action": "access",
                        "resource_id": null
                    }]
                }]
            }),
        };

        // Create request context
        let ctx = RequestContext {
            original_url: "https://client.example.com/".to_string(),
            host: "client.example.com".to_string(),
            path: "/".to_string(),
            session_token: Some("test-token".to_string()),
            session: Some(session),
            matched_route: Some(route),
        };

        // Test authorization
        match auth_service.authorize(&ctx) {
            AuthResult::Authorized => {
                // Test passed
            }
            other => panic!("Expected Authorized, got {:?}", other),
        }
    }

    #[test]
    fn test_login_redirect_creation() {
        let auth_service = AuthService::new();
        let login_url = "https://auth.example.com/login";
        let original_url = "https://app.example.com/admin/dashboard";

        let redirect_url = auth_service.create_login_redirect(login_url, original_url);

        assert!(redirect_url.starts_with(login_url));
        assert!(redirect_url.contains("next="));
    }

    #[test]
    fn test_extract_session_token() {
        let auth_service = AuthService::new();
        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::COOKIE,
            http::HeaderValue::from_static("session=test-token; other=value"),
        );

        let token = auth_service.extract_session_token(&headers, "session");
        assert_eq!(token, Some("test-token".to_string()));

        // Test with different cookie name
        let token = auth_service.extract_session_token(&headers, "other");
        assert_eq!(token, Some("value".to_string()));

        // Test with non-existent cookie
        let token = auth_service.extract_session_token(&headers, "nonexistent");
        assert_eq!(token, None);
    }

    // Helper function to create a test session
    fn create_test_session(roles: Vec<String>, permissions: Vec<String>) -> SessionResponse {
        SessionResponse {
            user: User {
                id: "user-1".to_string(),
                email: "user@example.com".to_string(),
                roles,
                permissions,
                teams: vec![Team {
                    id: "team-1".to_string(),
                    name: "Team 1".to_string(),
                    is_owner: true,
                    scopes: vec![],
                }],
            },
            tenant_id: "tenant-1".to_string(),
            authority: "example.com".to_string(),
            redirect_url: None,
        }
    }
}
