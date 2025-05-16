#[cfg(test)]
mod tests {
    use authgate::cache::{extract_jwt_expiration, InMemoryCache, SessionCache};
    use authgate::types::{SessionResponse, Team, User};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};
    use std::time::{Duration, SystemTime};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: u64,
        iat: u64,
    }

    fn create_test_session() -> SessionResponse {
        SessionResponse {
            user: User {
                id: "user-1".to_string(),
                email: "user@example.com".to_string(),
                roles: vec!["admin".to_string()],
                permissions: vec!["users:read".to_string()],
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

    fn create_jwt_token(expires_in_secs: u64) -> String {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = Claims {
            sub: "user-1".to_string(),
            exp: now + expires_in_secs,
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("test-secret".as_bytes()),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_in_memory_cache() {
        // Create a cache
        let cache = InMemoryCache::new();

        // Create a test session
        let session = create_test_session();

        // Set the session in the cache
        let token = "test-token";
        let ttl = Duration::from_secs(60);
        cache.set(token, session.clone(), ttl).await.unwrap();

        // Get the session from the cache
        let cached_session = cache.get(token).await;
        assert!(cached_session.is_some());
        assert_eq!(cached_session.unwrap().user.id, "user-1");

        // Remove the session from the cache
        cache.remove(token).await.unwrap();

        // Verify it's gone
        let cached_session = cache.get(token).await;
        assert!(cached_session.is_none());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        // Create a cache
        let cache = InMemoryCache::new();

        // Create a test session
        let session = create_test_session();

        // Set the session in the cache with a very short TTL
        let token = "test-token";
        let ttl = Duration::from_millis(100); // 100ms
        cache.set(token, session.clone(), ttl).await.unwrap();

        // Verify it's in the cache
        let cached_session = cache.get(token).await;
        assert!(cached_session.is_some());

        // Wait for it to expire
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify it's gone
        let cached_session = cache.get(token).await;
        assert!(cached_session.is_none());
    }

    #[tokio::test]
    async fn test_jwt_expiration_extraction() {
        // Create a JWT token that expires in 60 seconds
        let token = create_jwt_token(60);

        // Extract the expiration time
        let ttl = extract_jwt_expiration(&token);
        assert!(ttl.is_some());

        // The TTL should be close to 60 seconds
        let ttl = ttl.unwrap();
        assert!(ttl.as_secs() <= 60 && ttl.as_secs() >= 58);

        // Create an expired token
        let expired_token = create_jwt_token(0);

        // Extract the expiration time
        let ttl = extract_jwt_expiration(&expired_token);
        assert!(ttl.is_none());

        // Create an invalid token
        let invalid_token = "invalid-token";

        // Extract the expiration time
        let ttl = extract_jwt_expiration(invalid_token);
        assert!(ttl.is_none());
    }

    #[tokio::test]
    async fn test_cache_with_jwt_expiration() {
        // Create a cache
        let cache = InMemoryCache::new();

        // Create a test session
        let session = create_test_session();

        // Create a JWT token that expires in 2 seconds
        let token = create_jwt_token(2);

        // Extract the expiration time
        let ttl = extract_jwt_expiration(&token).unwrap();

        // Set the session in the cache with the JWT expiration
        cache.set(&token, session.clone(), ttl).await.unwrap();

        // Verify it's in the cache
        let cached_session = cache.get(&token).await;
        assert!(cached_session.is_some());

        // Wait for it to expire
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Verify it's gone
        let cached_session = cache.get(&token).await;
        assert!(cached_session.is_none());
    }
}
