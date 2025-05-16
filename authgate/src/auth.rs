use crate::cache::{extract_jwt_expiration, CacheFactory, SessionCache};
use crate::types::{
    AuthGateError, AuthResult, RequestContext, Scope, ScopeRequirement, SessionResponse,
    TeamRequirement,
};
use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use http::HeaderMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// AuthService handles authentication and authorization
pub struct AuthService {
    client: reqwest::Client,
    cache: Arc<dyn SessionCache>,
    cache_enabled: bool,
}

impl AuthService {
    /// Create a new AuthService
    pub fn new() -> Self {
        // Check if caching is enabled
        let cache_enabled = env::var("AUTHGATE_CACHE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase()
            == "true";

        if cache_enabled {
            info!("Session caching is enabled");
        } else {
            info!("Session caching is disabled");
        }

        // Create the cache
        let cache = CacheFactory::create();

        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            cache,
            cache_enabled,
        }
    }

    /// Validate a session by calling the session endpoint
    pub async fn validate_session(
        &self,
        session_url: &str,
        session_token: &str,
    ) -> Result<SessionResponse, AuthGateError> {
        // Check cache first if enabled
        if self.cache_enabled {
            if let Some(cached_session) = self.cache.get(session_token).await {
                debug!(
                    "Using cached session for user: {}",
                    cached_session.user.email
                );
                return Ok(cached_session);
            }
        }

        debug!("Validating session at {}", session_url);

        let response = self
            .client
            .get(session_url)
            .header("Cookie", format!("session={}", session_token))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send session validation request: {}", e);
                AuthGateError::AuthError(format!("Failed to validate session: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("Session validation failed with status: {}", status);
            return Err(AuthGateError::AuthError(format!(
                "Session validation failed with status: {}",
                status
            )));
        }

        let session: SessionResponse = response.json().await.map_err(|e| {
            error!("Failed to parse session response: {}", e);
            AuthGateError::AuthError(format!("Failed to parse session response: {}", e))
        })?;

        debug!(
            "Session validated successfully for user: {}",
            session.user.email
        );

        // Cache the session if caching is enabled
        if self.cache_enabled {
            // Extract JWT expiration time for TTL
            if let Some(ttl) = extract_jwt_expiration(session_token) {
                // Cache with the extracted TTL
                if let Err(e) = self.cache.set(session_token, session.clone(), ttl).await {
                    warn!("Failed to cache session: {}", e);
                }
            } else {
                // If we can't extract expiration, use a default TTL
                let default_ttl = Duration::from_secs(300); // 5 minutes
                if let Err(e) = self
                    .cache
                    .set(session_token, session.clone(), default_ttl)
                    .await
                {
                    warn!("Failed to cache session with default TTL: {}", e);
                }
            }
        }

        Ok(session)
    }

    /// Authorize a request based on the matched route and session
    pub fn authorize(&self, ctx: &RequestContext) -> AuthResult {
        let session = match &ctx.session {
            Some(session) => session,
            None => return AuthResult::Unauthenticated,
        };

        let route = match &ctx.matched_route {
            Some(route) => route,
            None => return AuthResult::Error("No matching route found".to_string()),
        };

        // Check if the user has the required roles
        if let Some(required_roles) = route.require.get("roles").and_then(|v| v.as_array()) {
            let required_roles: Vec<String> = required_roles
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if !self.has_any_role(&session.user.roles, &required_roles) {
                return AuthResult::Unauthorized(format!(
                    "User does not have any of the required roles: {:?}",
                    required_roles
                ));
            }
        }

        // Check if the user has the required permissions
        if let Some(required_permissions) =
            route.require.get("permissions").and_then(|v| v.as_array())
        {
            let required_permissions: Vec<String> = required_permissions
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if !self.has_any_permission(&session.user.permissions, &required_permissions) {
                return AuthResult::Unauthorized(format!(
                    "User does not have any of the required permissions: {:?}",
                    required_permissions
                ));
            }
        }

        // Check if the user has the required scopes
        if let Some(required_scopes_value) = route.require.get("scopes") {
            if let Some(required_scopes_array) = required_scopes_value.as_array() {
                // Convert JSON array to Vec<ScopeRequirement>
                let mut required_scopes: Vec<ScopeRequirement> = Vec::new();
                for scope_val in required_scopes_array {
                    if let Ok(scope_req) =
                        serde_json::from_value::<ScopeRequirement>(scope_val.clone())
                    {
                        required_scopes.push(scope_req);
                    } else {
                        return AuthResult::Error("Invalid scope requirement format".to_string());
                    }
                }

                // Collect all scopes from all teams
                let all_scopes: Vec<Scope> = session
                    .user
                    .teams
                    .iter()
                    .flat_map(|team| team.scopes.clone())
                    .collect();

                if !self.has_required_scopes(&all_scopes, &required_scopes) {
                    return AuthResult::Unauthorized(format!(
                        "User does not have the required scopes: {:?}",
                        required_scopes
                    ));
                }
            }
        }

        // Check if the user is in any of the required teams with the required scopes
        if let Some(required_teams_value) = route.require.get("teams") {
            if let Some(required_teams_array) = required_teams_value.as_array() {
                // Convert JSON array to Vec<TeamRequirement>
                let mut required_teams: Vec<TeamRequirement> = Vec::new();
                for team_val in required_teams_array {
                    if let Ok(team_req) =
                        serde_json::from_value::<TeamRequirement>(team_val.clone())
                    {
                        required_teams.push(team_req);
                    } else {
                        return AuthResult::Error("Invalid team requirement format".to_string());
                    }
                }

                if !self.has_team_access(&session.user.teams, &required_teams) {
                    return AuthResult::Unauthorized(format!(
                        "User does not have access through any of the required teams: {:?}",
                        required_teams
                    ));
                }
            }
        }

        // If we've made it here, the user is authorized
        AuthResult::Authorized
    }

    /// Check if the user has any of the required roles
    fn has_any_role(&self, user_roles: &[String], required_roles: &[String]) -> bool {
        for role in required_roles {
            if user_roles.contains(role) {
                debug!("User has required role: {}", role);
                return true;
            }
        }
        false
    }

    /// Check if the user has any of the required permissions
    fn has_any_permission(
        &self,
        user_permissions: &[String],
        required_permissions: &[String],
    ) -> bool {
        for permission in required_permissions {
            if user_permissions.contains(permission) {
                debug!("User has required permission: {}", permission);
                return true;
            }
        }
        false
    }

    /// Check if the user has the required scopes
    fn has_required_scopes(
        &self,
        user_scopes: &[Scope],
        required_scopes: &[ScopeRequirement],
    ) -> bool {
        for required_scope in required_scopes {
            let mut found = false;

            for user_scope in user_scopes {
                // Match resource type and action
                if user_scope.resource_type == required_scope.resource_type
                    && user_scope.action == required_scope.action
                {
                    // If resource_id is specified, it must match
                    if let Some(required_resource_id) = &required_scope.resource_id {
                        if &user_scope.resource_id == required_resource_id {
                            found = true;
                            break;
                        }
                    } else {
                        // No specific resource_id required
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                return false;
            }
        }

        true
    }

    /// Check if the user has access through any of the required teams
    fn has_team_access(
        &self,
        user_teams: &[crate::types::Team],
        required_teams: &[TeamRequirement],
    ) -> bool {
        for team_req in required_teams {
            for user_team in user_teams {
                let id_match = team_req.id.as_ref().map_or(false, |id| id == &user_team.id);
                let name_match = team_req
                    .name
                    .as_ref()
                    .map_or(false, |name| name == &user_team.name);

                // If either ID or name matches
                if id_match || name_match {
                    // If scopes are required, check them
                    if let Some(required_scopes) = &team_req.scopes {
                        if self.has_required_scopes(&user_team.scopes, required_scopes) {
                            debug!("User has access through team: {}", user_team.name);
                            return true;
                        }
                    } else {
                        // No scopes required, team membership is enough
                        debug!("User has access through team: {}", user_team.name);
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Create a login redirect URL with the next parameter
    pub fn create_login_redirect(&self, login_url: &str, original_url: &str) -> String {
        let encoded_url = URL_SAFE_NO_PAD.encode(original_url);

        if login_url.contains('?') {
            format!("{}&next={}", login_url, encoded_url)
        } else {
            format!("{}?next={}", login_url, encoded_url)
        }
    }

    /// Extract session token from cookies
    pub fn extract_session_token(&self, headers: &HeaderMap, cookie_name: &str) -> Option<String> {
        let cookie_header = headers.get(http::header::COOKIE)?;
        let cookie_str = cookie_header.to_str().ok()?;

        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            if let Some(pos) = cookie.find('=') {
                let (name, value) = cookie.split_at(pos);
                if name == cookie_name {
                    return Some(value[1..].to_string());
                }
            }
        }

        None
    }
}
