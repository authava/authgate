use crate::auth::AuthService;
use crate::config::ConfigManager;
use crate::matcher::RouteMatcher;
use crate::types::{AuthResult, RequestContext};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, Response, StatusCode},
    response::{IntoResponse, Redirect},
};
use http::header;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, warn};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config_manager: Arc<ConfigManager>,
    pub route_matcher: Arc<RouteMatcher>,
    pub auth_service: Arc<AuthService>,
}

/// Query parameters for the forward auth endpoint
#[derive(Debug, Deserialize)]
pub struct ForwardAuthQuery {
    #[serde(rename = "X-Forwarded-Host")]
    pub forwarded_host: Option<String>,
    #[serde(rename = "X-Forwarded-Uri")]
    pub forwarded_uri: Option<String>,
    #[serde(rename = "X-Forwarded-Proto")]
    pub forwarded_proto: Option<String>,
}

/// Handle the forward auth request
pub async fn handle_forward_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: Query<ForwardAuthQuery>,
) -> impl IntoResponse {
    // Extract request information
    let host = query.forwarded_host.clone().unwrap_or_else(|| {
        headers
            .get("X-Forwarded-Host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown-host")
            .to_string()
    });

    let path = query.forwarded_uri.clone().unwrap_or_else(|| {
        headers
            .get("X-Forwarded-Uri")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("/")
            .to_string()
    });

    let proto = query.forwarded_proto.clone().unwrap_or_else(|| {
        headers
            .get("X-Forwarded-Proto")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("http")
            .to_string()
    });

    let original_url = format!("{}://{}{}", proto, host, path);
    debug!("Processing forward auth request for: {}", original_url);

    let callback_domain = std::env::var("AUTHGATE_CALLBACK_DOMAIN").ok();

    /// Encodes a string as base64 URL-safe (without padding)
    fn base64_url_encode(input: &str) -> String {
        URL_SAFE_NO_PAD.encode(input)
    }

    let effective_original_url = if let Some(callback_domain) = callback_domain {
        let encoded = base64_url_encode(&original_url);
        format!("{}/auth/callback?next={}", callback_domain, encoded)
    } else {
        original_url.clone()
    };

    // Match route
    let matched_route = state.route_matcher.match_route(&host, &path).await;

    // Get cookie name from config
    let cookie_name = state.config_manager.get_cookie_name().await;

    // Extract session token from cookies
    let session_token = state
        .auth_service
        .extract_session_token(&headers, &cookie_name);

    // Create request context
    let mut ctx = RequestContext {
        original_url: original_url.clone(),
        host: host.clone(),
        path: path.clone(),
        session_token: session_token.clone(),
        session: None,
        matched_route: matched_route.clone(),
    };

    // If no matching route, allow the request (no protection needed)
    if ctx.matched_route.is_none() {
        debug!("No matching route found, allowing request");
        return Response::builder()
            .status(StatusCode::OK)
            .body(axum::body::Body::empty())
            .unwrap();
    }

    // If no session token, redirect to login
    if ctx.session_token.is_none() {
        debug!("No session token found, redirecting to login");
        let config = state.config_manager.get_config().await;
        let redirect_url = state
            .auth_service
            .create_login_redirect(&config.auth.login_redirect, &effective_original_url);

        return Redirect::to(&redirect_url).into_response();
    }

    // Validate session
    let config = state.config_manager.get_config().await;
    let session_result = state
        .auth_service
        .validate_session(
            &config.auth.session_url,
            &ctx.session_token.clone().unwrap(),
        )
        .await;

    match session_result {
        Ok(session) => {
            ctx.session = Some(session);

            // Authorize the request
            match state.auth_service.authorize(&ctx) {
                AuthResult::Authorized => {
                    debug!("Request authorized for {}", original_url);
                    let user = &ctx.session.as_ref().unwrap().user;

                    // Build response with user information headers
                    let mut response = Response::builder().status(StatusCode::OK);

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
                        response =
                            response.header("X-Auth-User-Permissions", user.permissions.join(","));
                    }

                    // Return the response with headers
                    response.body(axum::body::Body::empty()).unwrap()
                }
                AuthResult::Unauthorized(reason) => {
                    warn!("Request unauthorized: {}", reason);
                    Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .body(axum::body::Body::from(format!("Forbidden: {}", reason)))
                        .unwrap()
                }
                AuthResult::Unauthenticated => {
                    debug!("Session invalid, redirecting to login");
                    let redirect_url = state
                        .auth_service
                        .create_login_redirect(&config.auth.login_redirect, &effective_original_url);

                    Redirect::to(&redirect_url).into_response()
                }
                AuthResult::Error(err) => {
                    error!("Authorization error: {}", err);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .body(axum::body::Body::from(format!(
                            "Internal server error: {}",
                            err
                        )))
                        .unwrap()
                }
            }
        }
        Err(e) => {
            warn!("Session validation failed: {}", e);
            let redirect_url = state
                .auth_service
                .create_login_redirect(&config.auth.login_redirect, &effective_original_url);

            Redirect::to(&redirect_url).into_response()
        }
    }
}
