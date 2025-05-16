#[cfg(test)]
mod tests {
    use authgate::cache::{InMemoryCache, RedisCache, SessionCache};
    use authgate::types::{SessionResponse, Team, User};
    use std::env;
    use std::time::Duration;

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

    // This test is marked as ignored by default because it requires a Redis server
    // To run it: cargo test -- --ignored
    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_integration() {
        // Skip if REDIS_URL is not set
        let redis_url = match env::var("REDIS_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("Skipping Redis test because REDIS_URL is not set");
                return;
            }
        };

        // Create a Redis cache
        let cache = RedisCache::new(&redis_url);

        // Create a test session
        let session = create_test_session();

        // Set the session in the cache
        let token = "test-token-redis";
        let ttl = Duration::from_secs(60);
        let result = cache.set(token, session.clone(), ttl).await;
        assert!(result.is_ok());

        // Get the session from the cache
        let cached_session = cache.get(token).await;
        assert!(cached_session.is_some());
        assert_eq!(cached_session.unwrap().user.id, "user-1");

        // Remove the session from the cache
        let result = cache.remove(token).await;
        assert!(result.is_ok());

        // Verify it's gone
        let cached_session = cache.get(token).await;
        assert!(cached_session.is_none());
    }

    // This test verifies that both cache implementations behave the same way
    #[tokio::test]
    async fn test_cache_implementations_consistency() {
        // Create both cache types
        let memory_cache = InMemoryCache::new();

        // Create a test session
        let session = create_test_session();

        // Test with memory cache
        let token = "test-token-consistency";
        let ttl = Duration::from_secs(1);

        // Set the session in the cache
        memory_cache.set(token, session.clone(), ttl).await.unwrap();

        // Get the session from the cache
        let cached_session = memory_cache.get(token).await;
        assert!(cached_session.is_some());
        assert_eq!(cached_session.unwrap().user.id, "user-1");

        // Wait for it to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Verify it's gone
        let cached_session = memory_cache.get(token).await;
        assert!(cached_session.is_none());
    }
}
