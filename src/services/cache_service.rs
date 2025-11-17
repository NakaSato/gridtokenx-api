use redis::{AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

use crate::error::{ApiError, Result};

/// Cache service for Redis-based caching
/// Provides methods to cache data, retrieve cached data, and manage cache keys
#[derive(Clone)]
pub struct CacheService {
    client: Client,
    default_ttl: Duration,
}

/// Cache key prefixes for different data types
pub mod keys {
    pub const USER_PROFILE: &str = "user:profile";
    pub const USER_WALLET: &str = "user:wallet";
    pub const METER_STATS: &str = "meter:stats";
    pub const ORDER_BOOK: &str = "order:book";
    pub const ACTIVE_OFFERS: &str = "offers:active";
    pub const MARKET_DATA: &str = "market:data";
    pub const HEALTH_CHECK: &str = "health:check";
    pub const RATE_LIMIT: &str = "rate_limit";
}

impl CacheService {
    /// Create a new cache service with Redis client
    pub fn new(redis_url: &str, default_ttl_seconds: u64) -> Result<Self> {
        let client = Client::open(redis_url).map_err(|e| {
            error!("Failed to create Redis client: {}", e);
            ApiError::external_service("Failed to connect to cache service")
        })?;

        Ok(Self {
            client,
            default_ttl: Duration::from_secs(default_ttl_seconds),
        })
    }

    /// Get a connection to Redis
    async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                error!("Failed to get Redis connection: {}", e);
                ApiError::external_service("Cache service unavailable")
            })
    }

    /// Set a value in cache with default TTL
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Set a value in cache with custom TTL
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<()> {
        let serialized = serde_json::to_string(value).map_err(|e| {
            error!("Failed to serialize value for cache: {}", e);
            ApiError::internal("Failed to cache data")
        })?;

        let mut conn = self.get_connection().await?;
        let ttl_seconds = ttl.as_secs() as usize;

        conn.set_ex(key, serialized, ttl_seconds)
            .await
            .map_err(|e| {
                error!("Failed to set cache key {}: {}", key, e);
                ApiError::external_service("Failed to cache data")
            })?;

        debug!("Cached key: {} (TTL: {}s)", key, ttl_seconds);
        Ok(())
    }

    /// Get a value from cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.get_connection().await?;

        let value: Option<String> = conn.get(key).await.map_err(|e| {
            error!("Failed to get cache key {}: {}", key, e);
            ApiError::external_service("Failed to retrieve cached data")
        })?;

        match value {
            Some(data) => {
                let deserialized = serde_json::from_str(&data).map_err(|e| {
                    error!("Failed to deserialize cached value for key {}: {}", key, e);
                    ApiError::internal("Invalid cached data")
                })?;

                debug!("Cache hit: {}", key);
                Ok(Some(deserialized))
            }
            None => {
                debug!("Cache miss: {}", key);
                Ok(None)
            }
        }
    }

    /// Delete a key from cache
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;

        conn.del(key).await.map_err(|e| {
            error!("Failed to delete cache key {}: {}", key, e);
            ApiError::external_service("Failed to invalidate cache")
        })?;

        debug!("Deleted cache key: {}", key);
        Ok(())
    }

    /// Delete multiple keys matching a pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64> {
        let mut conn = self.get_connection().await?;

        // Get all keys matching pattern
        let keys: Vec<String> = conn.keys(pattern).await.map_err(|e| {
            error!("Failed to get keys matching pattern {}: {}", pattern, e);
            ApiError::external_service("Failed to invalidate cache")
        })?;

        if keys.is_empty() {
            debug!("No keys found matching pattern: {}", pattern);
            return Ok(0);
        }

        // Delete all matching keys
        let count = keys.len() as u64;
        conn.del(&keys).await.map_err(|e| {
            error!("Failed to delete keys matching pattern {}: {}", pattern, e);
            ApiError::external_service("Failed to invalidate cache")
        })?;

        debug!("Deleted {} keys matching pattern: {}", count, pattern);
        Ok(count)
    }

    /// Check if a key exists in cache
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;

        let exists: bool = conn.exists(key).await.map_err(|e| {
            error!("Failed to check if key exists {}: {}", key, e);
            ApiError::external_service("Failed to check cache")
        })?;

        Ok(exists)
    }

    /// Get remaining TTL for a key (in seconds)
    pub async fn ttl(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection().await?;

        let ttl: i64 = conn.ttl(key).await.map_err(|e| {
            error!("Failed to get TTL for key {}: {}", key, e);
            ApiError::external_service("Failed to get cache TTL")
        })?;

        Ok(ttl)
    }

    /// Set expiration time for an existing key
    pub async fn expire(&self, key: &str, ttl: Duration) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let ttl_seconds = ttl.as_secs() as usize;

        conn.expire(key, ttl_seconds).await.map_err(|e| {
            error!("Failed to set expiration for key {}: {}", key, e);
            ApiError::external_service("Failed to update cache expiration")
        })?;

        debug!("Set expiration for key: {} (TTL: {}s)", key, ttl_seconds);
        Ok(())
    }

    /// Increment a counter in cache
    pub async fn increment(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection().await?;

        let value: i64 = conn.incr(key, 1).await.map_err(|e| {
            error!("Failed to increment key {}: {}", key, e);
            ApiError::external_service("Failed to increment counter")
        })?;

        Ok(value)
    }

    /// Increment a counter with expiration
    pub async fn increment_with_ttl(&self, key: &str, ttl: Duration) -> Result<i64> {
        let mut conn = self.get_connection().await?;

        let value: i64 = conn.incr(key, 1).await.map_err(|e| {
            error!("Failed to increment key {}: {}", key, e);
            ApiError::external_service("Failed to increment counter")
        })?;

        // Set expiration only if this is the first increment (value == 1)
        if value == 1 {
            let ttl_seconds = ttl.as_secs() as usize;
            conn.expire(key, ttl_seconds).await.map_err(|e| {
                error!("Failed to set expiration for key {}: {}", key, e);
                ApiError::external_service("Failed to set counter expiration")
            })?;
        }

        Ok(value)
    }

    /// Get or compute a value (cache-aside pattern)
    /// If the value is in cache, return it. Otherwise, compute it, cache it, and return it.
    pub async fn get_or_compute<T, F, Fut>(
        &self,
        key: &str,
        compute_fn: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Try to get from cache first
        if let Some(cached) = self.get::<T>(key).await? {
            return Ok(cached);
        }

        // Compute the value
        let value = compute_fn().await?;

        // Cache the computed value (ignore cache errors)
        if let Err(e) = self.set(key, &value).await {
            warn!("Failed to cache computed value for key {}: {}", key, e);
        }

        Ok(value)
    }

    /// Get or compute a value with custom TTL
    pub async fn get_or_compute_with_ttl<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        compute_fn: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Try to get from cache first
        if let Some(cached) = self.get::<T>(key).await? {
            return Ok(cached);
        }

        // Compute the value
        let value = compute_fn().await?;

        // Cache the computed value with custom TTL (ignore cache errors)
        if let Err(e) = self.set_with_ttl(key, &value, ttl).await {
            warn!("Failed to cache computed value for key {}: {}", key, e);
        }

        Ok(value)
    }

    /// Build a cache key with prefix and identifier
    pub fn build_key(prefix: &str, id: &str) -> String {
        format!("{}:{}", prefix, id)
    }

    /// Build a cache key with multiple parts
    pub fn build_key_parts(parts: &[&str]) -> String {
        parts.join(":")
    }

    /// Flush all cache (use with caution in production!)
    pub async fn flush_all(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        redis::cmd("FLUSHDB")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to flush cache: {}", e);
                ApiError::external_service("Failed to flush cache")
            })?;

        warn!("Flushed all cache data");
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats> {
        let mut conn = self.get_connection().await?;

        let info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get cache stats: {}", e);
                ApiError::external_service("Failed to get cache statistics")
            })?;

        // Parse basic stats from INFO output
        let mut stats = CacheStats::default();

        for line in info.lines() {
            if line.starts_with("keyspace_hits:") {
                stats.hits = line.split(':').nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            } else if line.starts_with("keyspace_misses:") {
                stats.misses = line.split(':').nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            }
        }

        stats.hit_rate = if stats.hits + stats.misses > 0 {
            (stats.hits as f64 / (stats.hits + stats.misses) as f64) * 100.0
        } else {
            0.0
        };

        Ok(stats)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_key_building() {
        let key = CacheService::build_key("user", "123");
        assert_eq!(key, "user:123");

        let key = CacheService::build_key_parts(&["order", "book", "solar"]);
        assert_eq!(key, "order:book:solar");
    }
}
