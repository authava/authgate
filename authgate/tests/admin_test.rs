#[cfg(test)]
mod tests {
    use authgate::admin::{create_admin_router_with_enabled, is_admin_api_enabled};
    use axum::{
        body::Body,
        extract::Request,
        http::{header, StatusCode},
    };
    use std::env;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_admin_api_disabled_by_default() {
        // Clear any existing environment variables
        env::remove_var("AUTHGATE_ENABLE_ADMIN_API");
        env::remove_var("AUTHGATE_CONFIG_BACKEND");

        // Check that the admin API is disabled by default
        assert!(!is_admin_api_enabled());

        // Create the admin router with explicit disabled flag
        let app = create_admin_router_with_enabled::<()>(false);

        // Create a request to the health endpoint
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Check that the response is 403 Forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_admin_api_disabled_with_json_backend() {
        std::env::remove_var("AUTHGATE_ENABLE_ADMIN_API");
        std::env::remove_var("AUTHGATE_CONFIG_BACKEND");

        // Set the environment variables
        env::set_var("AUTHGATE_ENABLE_ADMIN_API", "true");
        env::set_var("AUTHGATE_CONFIG_BACKEND", "json");

        // Check that the admin API is disabled with json backend
        assert!(!is_admin_api_enabled());

        // Create the admin router with explicit disabled flag
        let app = create_admin_router_with_enabled::<()>(false);

        // Create a request to the health endpoint
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Check that the response is 403 Forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        std::env::remove_var("AUTHGATE_ENABLE_ADMIN_API");
        std::env::remove_var("AUTHGATE_CONFIG_BACKEND");
    }

    #[tokio::test]
    async fn test_admin_api_token_required() {
        // Clear any existing environment variables
        env::remove_var("AUTHGATE_ENABLE_ADMIN_API");
        env::remove_var("AUTHGATE_CONFIG_BACKEND");
        env::remove_var("AUTHGATE_ADMIN_TOKEN");

        // Set the environment variables
        env::set_var("AUTHGATE_ENABLE_ADMIN_API", "true");
        env::set_var("AUTHGATE_CONFIG_BACKEND", "postgres");
        env::set_var("AUTHGATE_ADMIN_TOKEN", "test-token");

        // We're using explicit enabled flag, so we don't need to check is_admin_api_enabled

        // Create the admin router with explicit enabled flag
        let app = create_admin_router_with_enabled::<()>(true);

        // Create a request without an Authorization header
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Check that the response is 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Check that the WWW-Authenticate header is present
        assert_eq!(
            response.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            "Bearer"
        );
    }

    #[tokio::test]
    async fn test_admin_api_invalid_token() {
        // Clear any existing environment variables
        env::remove_var("AUTHGATE_ENABLE_ADMIN_API");
        env::remove_var("AUTHGATE_CONFIG_BACKEND");
        env::remove_var("AUTHGATE_ADMIN_TOKEN");

        // Set the environment variables
        env::set_var("AUTHGATE_ADMIN_TOKEN", "test-token");

        // Create the admin router with explicit enabled flag
        let app = create_admin_router_with_enabled::<()>(true);

        // Create a request with an invalid Authorization header
        let request = Request::builder()
            .uri("/health")
            .header(header::AUTHORIZATION, "Bearer invalid-token")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Check that the response is 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Check that the WWW-Authenticate header is present
        assert_eq!(
            response.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            "Bearer"
        );
    }

    #[tokio::test]
    async fn test_admin_api_valid_token() {
        // Clear any existing environment variables
        env::remove_var("AUTHGATE_ENABLE_ADMIN_API");
        env::remove_var("AUTHGATE_CONFIG_BACKEND");
        env::remove_var("AUTHGATE_ADMIN_TOKEN");

        // Set the environment variables
        env::set_var("AUTHGATE_ADMIN_TOKEN", "test-token");

        // Create the admin router with explicit enabled flag
        let app = create_admin_router_with_enabled::<()>(true);

        // Create a request with a valid Authorization header
        let request = Request::builder()
            .uri("/health")
            .header(header::AUTHORIZATION, "Bearer test-token")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Check that the response is 200 OK
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_admin_api_fallback_route() {
        // Clear any existing environment variables
        env::remove_var("AUTHGATE_ENABLE_ADMIN_API");
        env::remove_var("AUTHGATE_CONFIG_BACKEND");

        // Create the admin router with explicit disabled flag
        let app = create_admin_router_with_enabled::<()>(false);

        // Create a request to a non-existent endpoint
        let request = Request::builder()
            .uri("/nonexistent")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Check that the response is 403 Forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
