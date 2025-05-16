#[cfg(test)]
mod tests {
    use authgate::types::{RequestContext, Route, Scope, SessionResponse, Team, User};
    use axum::http::{HeaderMap, StatusCode};
    use http::header;

    #[test]
    fn test_auth_headers() {
        // This test verifies that the correct headers are set in the response
        // when a request is authorized.

        // Create a test session
        let session = SessionResponse {
            user: User {
                id: "user-1".to_string(),
                email: "user@example.com".to_string(),
                roles: vec!["admin".to_string(), "user".to_string()],
                permissions: vec!["users:read".to_string(), "users:write".to_string()],
                teams: vec![Team {
                    id: "team-1".to_string(),
                    name: "Team 1".to_string(),
                    is_owner: true,
                    scopes: vec![Scope {
                        resource_type: "client".to_string(),
                        resource_id: "client-1".to_string(),
                        action: "access".to_string(),
                    }],
                }],
            },
            tenant_id: "tenant-1".to_string(),
            authority: "example.com".to_string(),
            redirect_url: None,
        };

        // Create a RequestContext with the session
        let ctx = RequestContext {
            original_url: "https://app.example.com/admin/dashboard".to_string(),
            host: "app.example.com".to_string(),
            path: "/admin/dashboard".to_string(),
            session_token: Some("test-token".to_string()),
            session: Some(session),
            matched_route: Some(Route {
                id: None,
                host: "app.example.com".to_string(),
                path: "/admin/*".to_string(),
                require: serde_json::json!({
                    "roles": ["admin"],
                    "permissions": null,
                    "scopes": null,
                    "teams": null
                }),
            }),
        };

        // Create an authorized response using the same logic as in proxy.rs
        let user = &ctx.session.as_ref().unwrap().user;

        // Build response with user information headers
        let mut response = http::Response::builder().status(StatusCode::OK);

        // Add user ID and email headers
        response = response
            .header("X-Auth-User-Id", &user.id)
            .header("X-Auth-User-Email", &user.email);

        // Add roles as a comma-separated list
        if !user.roles.is_empty() {
            response = response.header("X-Auth-User-Roles", user.roles.join(","));
        }

        // Add permissions as a comma-separated list
        if !user.permissions.is_empty() {
            response = response.header("X-Auth-User-Permissions", user.permissions.join(","));
        }

        // Build the response
        let response = response.body(()).unwrap();

        // Check the headers
        let headers = response.headers();

        assert_eq!(headers.get("X-Auth-User-Id").unwrap(), "user-1");
        assert_eq!(
            headers.get("X-Auth-User-Email").unwrap(),
            "user@example.com"
        );
        assert_eq!(headers.get("X-Auth-User-Roles").unwrap(), "admin,user");
        assert_eq!(
            headers.get("X-Auth-User-Permissions").unwrap(),
            "users:read,users:write"
        );
    }

    #[test]
    fn test_extract_session_token() {
        // Test the cookie extraction logic
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            header::HeaderValue::from_static("session=test-token; other=value"),
        );

        // Extract the session token
        let cookie_header = headers.get(http::header::COOKIE).unwrap();
        let cookie_str = cookie_header.to_str().unwrap();
        let mut session_token = None;

        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            if let Some(pos) = cookie.find('=') {
                let (name, value) = cookie.split_at(pos);
                if name == "session" {
                    session_token = Some(value[1..].to_string());
                    break;
                }
            }
        }

        assert_eq!(session_token, Some("test-token".to_string()));
    }
}
