use crate::types::{AuthGateError, SessionResponse};
use async_trait::async_trait;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Cache trait defining the interface for session caching
#[async_trait]
pub trait SessionCache: Send + Sync {
    /// Get a session from the cache
    async fn get(&self, token: &str) -> Option<SessionResponse>;

    /// Set a session in the cache with TTL
    async fn set(
        &self,
        token: &str,
        session: SessionResponse,
        ttl: Duration,
    ) -> Result<(), AuthGateError>;

    /// Remove a session from the cache
    async fn remove(&self, token: &str) -> Result<(), AuthGateError>;
}

/// JWT claims structure for extracting expiration time
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: Option<u64>,
    // Other fields can be added as needed
}

/// Cache factory for creating the appropriate cache implementation
pub struct CacheFactory;

impl CacheFactory {
    /// Create a new cache instance based on environment configuration
    pub fn create() -> Arc<dyn SessionCache> {
        let cache_backend =
            env::var("AUTHGATE_CACHE_BACKEND").unwrap_or_else(|_| "memory".to_string());

        match cache_backend.to_lowercase().as_str() {
            "redis" => {
                let redis_url = env::var("AUTHGATE_REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

                info!("Using Redis cache backend at {}", redis_url);
                Arc::new(RedisCache::new(&redis_url))
            }
            _ => {
                info!("Using in-memory cache backend");
                Arc::new(InMemoryCache::new())
            }
        }
    }
}

/// Helper function to extract expiration time from JWT token
pub fn extract_jwt_expiration(token: &str) -> Option<Duration> {
    // First try to decode the token header to get the algorithm
    let header = match decode_header(token) {
        Ok(header) => header,
        Err(e) => {
            warn!("Failed to decode JWT header: {}", e);
            return None;
        }
    };

    // Use a dummy key for decoding - we only care about the claims, not validation
    let dummy_key = DecodingKey::from_secret(&[]);

    // Create a validation that skips signature verification
    let mut validation = Validation::new(header.alg);
    validation.validate_exp = false;
    validation.validate_nbf = false;
    validation.insecure_disable_signature_validation();

    // Decode the token to extract claims
    let token_data = match decode::<Claims>(token, &dummy_key, &validation) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to decode JWT claims: {}", e);
            return None;
        }
    };

    // Extract expiration time
    if let Some(exp) = token_data.claims.exp {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();

        if exp <= now {
            // Token is already expired
            debug!("JWT token is already expired");
            return None;
        }

        // Calculate remaining time
        let remaining_secs = exp - now;
        debug!("JWT token expires in {} seconds", remaining_secs);
        return Some(Duration::from_secs(remaining_secs));
    }

    // No expiration claim found
    warn!("JWT token has no expiration claim");
    None
}

/// In-memory implementation of SessionCache
pub struct InMemoryCache {
    cache: Arc<RwLock<HashMap<String, (SessionResponse, SystemTime)>>>,
}

impl InMemoryCache {
    /// Create a new in-memory cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Clean expired entries from the cache
    async fn clean_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = SystemTime::now();

        // Remove expired entries
        cache.retain(|_, (_, expiry)| {
            match expiry.duration_since(now) {
                Ok(_) => true,   // Not expired
                Err(_) => false, // Expired
            }
        });
    }
}

#[async_trait]
impl SessionCache for InMemoryCache {
    async fn get(&self, token: &str) -> Option<SessionResponse> {
        // Clean expired entries first
        self.clean_expired().await;

        // Try to get the session
        let cache = self.cache.read().await;
        if let Some((session, expiry)) = cache.get(token) {
            // Check if the session is still valid
            if let Ok(_) = expiry.duration_since(SystemTime::now()) {
                debug!("Cache hit for token");
                return Some(session.clone());
            }
        }

        debug!("Cache miss for token");
        None
    }

    async fn set(
        &self,
        token: &str,
        session: SessionResponse,
        ttl: Duration,
    ) -> Result<(), AuthGateError> {
        let expiry = SystemTime::now() + ttl;

        let mut cache = self.cache.write().await;
        cache.insert(token.to_string(), (session, expiry));

        debug!("Cached session with TTL of {} seconds", ttl.as_secs());
        Ok(())
    }

    async fn remove(&self, token: &str) -> Result<(), AuthGateError> {
        let mut cache = self.cache.write().await;
        cache.remove(token);

        debug!("Removed session from cache");
        Ok(())
    }
}

/// Redis implementation of SessionCache
pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    /// Create a new Redis cache
    pub fn new(redis_url: &str) -> Self {
        Self {
            client: redis::Client::open(redis_url).expect("Failed to create Redis client"),
        }
    }
}

#[async_trait]
impl SessionCache for RedisCache {
    async fn get(&self, token: &str) -> Option<SessionResponse> {
        let mut conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to connect to Redis: {}", e);
                return None;
            }
        };

        // Try to get the session from Redis
        let key = format!("authgate:session:{}", token);
        let result: redis::RedisResult<String> =
            redis::cmd("GET").arg(&key).query_async(&mut conn).await;

        match result {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(session) => {
                    debug!("Cache hit for token in Redis");
                    Some(session)
                }
                Err(e) => {
                    error!("Failed to deserialize session from Redis: {}", e);
                    None
                }
            },
            Err(e) => {
                if e.kind() != redis::ErrorKind::TypeError {
                    // Only log if it's not a type error (key not found)
                    debug!("Cache miss for token in Redis: {}", e);
                }
                None
            }
        }
    }

    async fn set(
        &self,
        token: &str,
        session: SessionResponse,
        ttl: Duration,
    ) -> Result<(), AuthGateError> {
        let mut conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                return Err(AuthGateError::ConfigError(format!(
                    "Failed to connect to Redis: {}",
                    e
                )));
            }
        };

        // Serialize the session
        let json = match serde_json::to_string(&session) {
            Ok(json) => json,
            Err(e) => {
                return Err(AuthGateError::SerializationError(e));
            }
        };

        // Store the session in Redis with expiration
        let key = format!("authgate:session:{}", token);
        let result: redis::RedisResult<()> = redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl.as_secs())
            .arg(json)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => {
                debug!(
                    "Cached session in Redis with TTL of {} seconds",
                    ttl.as_secs()
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to cache session in Redis: {}", e);
                Err(AuthGateError::ConfigError(format!(
                    "Failed to cache session in Redis: {}",
                    e
                )))
            }
        }
    }

    async fn remove(&self, token: &str) -> Result<(), AuthGateError> {
        let mut conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                return Err(AuthGateError::ConfigError(format!(
                    "Failed to connect to Redis: {}",
                    e
                )));
            }
        };

        // Remove the session from Redis
        let key = format!("authgate:session:{}", token);
        let result: redis::RedisResult<()> =
            redis::cmd("DEL").arg(&key).query_async(&mut conn).await;

        match result {
            Ok(_) => {
                debug!("Removed session from Redis cache");
                Ok(())
            }
            Err(e) => {
                error!("Failed to remove session from Redis: {}", e);
                Err(AuthGateError::ConfigError(format!(
                    "Failed to remove session from Redis: {}",
                    e
                )))
            }
        }
    }
}
