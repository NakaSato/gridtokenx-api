// Rate limiting middleware with configurable limits per endpoint type
// Prevents brute force, DDoS attacks, and API abuse

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use axum::{
    extract::ConnectInfo,
    middleware::Next,
    response::Response,
    http::Request,
    body::Body,
};
use dashmap::DashMap;
use std::net::SocketAddr;
use crate::error::ApiError;
use redis::Client;

/// Rate limit configuration for different endpoint types
#[derive(Debug, Clone)]
pub enum RateLimitConfig {
    Auth,
    Trading,
    WebSocket,
    Reading,
    Api,
    Admin,
    Meter,
    Public,
}

impl RateLimitConfig {
    pub fn auth() -> Self {
        Self::Auth
    }

    pub fn trading() -> Self {
        Self::Trading
    }

    pub fn websocket() -> Self {
        Self::WebSocket
    }

    pub fn reading() -> Self {
        Self::Reading
    }

    pub fn api() -> Self {
        Self::Api
    }

    pub fn admin() -> Self {
        Self::Admin
    }

    pub fn meter() -> Self {
        Self::Meter
    }

    pub fn public() -> Self {
        Self::Public
    }

    /// Get the rate limit key and max requests for this configuration
    fn limits(&self) -> (u32, u64) {
        match self {
            // (max_requests, window_seconds)
            Self::Auth => (5, 60),           // 5 requests per minute
            Self::Trading => (30, 60),      // 30 requests per minute
            Self::WebSocket => (5, 60),     // 5 connections per minute
            Self::Reading => (60, 60),      // 60 requests per minute
            Self::Api => (100, 60),         // 100 requests per minute
            Self::Admin => (10, 60),        // 10 requests per minute
            Self::Meter => (60, 60),        // 60 requests per minute
            Self::Public => (200, 60),      // 200 requests per minute
        }
    }

    fn key(&self) -> &str {
        match self {
            Self::Auth => "rate_limit:auth",
            Self::Trading => "rate_limit:trading",
            Self::WebSocket => "rate_limit:websocket",
            Self::Reading => "rate_limit:reading",
            Self::Api => "rate_limit:api",
            Self::Admin => "rate_limit:admin",
            Self::Meter => "rate_limit:meter",
            Self::Public => "rate_limit:public",
        }
    }
}

/// Redis-based rate limiter state
#[derive(Clone)]
pub struct RateLimiterState {
    redis_client: Client,
}

impl RateLimiterState {
    pub fn new(redis_client: Client) -> Self {
        Self { redis_client }
    }

    /// Check rate limit for the given IP
    fn check_rate_limit(&self, ip: &str, config: &RateLimitConfig) -> Result<(), ApiError> {
        let (max_requests, window_seconds) = config.limits();
        let key = format!("{}:{}", config.key(), ip);

        let mut conn = match self.redis_client.get_connection() {
            Ok(c) => c,
            Err(_) => return Ok(()), // If Redis fails, allow request
        };

        let current: u32 = redis::cmd("INCR")
            .arg(&key)
            .query(&mut conn)
            .unwrap_or(0);

        // Set expiry on first request
        if current == 1 {
            let _: () = redis::cmd("EXPIRE")
                .arg(&key)
                .arg(window_seconds as i64)
                .query(&mut conn)
                .unwrap_or(());
        }

        if current > max_requests {
            return Err(ApiError::RateLimitExceeded {
                retry_after_seconds: window_seconds,
            });
        }

        Ok(())
    }
}

/// Middleware for rate limiting using Redis
pub async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<RateLimiterState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let ip = addr.ip().to_string();

    // Extract config from request extensions if available, otherwise use API
    let config = request
        .extensions()
        .get::<RateLimitConfig>()
        .cloned()
        .unwrap_or(RateLimitConfig::Api);

    state.check_rate_limit(&ip, &config)?;

    Ok(next.run(request).await)
}

// ============================================================================
// ENHANCED RATE LIMITER - Per-IP tracking with in-memory storage
// ============================================================================

/// Tracks request timestamps for an IP
#[derive(Debug, Clone)]
struct RateLimitEntry {
    timestamps: Vec<u64>,
    last_cleanup: u64,
}

impl RateLimitEntry {
    fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            timestamps: Vec::new(),
            last_cleanup: now,
        }
    }

    /// Remove timestamps outside the current window
    fn cleanup(&mut self, window_seconds: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cutoff = now.saturating_sub(window_seconds);
        self.timestamps.retain(|&ts| ts > cutoff);
        self.last_cleanup = now;
    }

    /// Check if a request can proceed and add it if so
    fn check_and_add(&mut self, config: &EnhancedRateLimitConfig) -> Result<(), u64> {
        // Cleanup old entries periodically
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now - self.last_cleanup > 60 {
            self.cleanup(config.window_seconds);
        }

        // Remove timestamps outside window
        let cutoff = now.saturating_sub(config.window_seconds);
        self.timestamps.retain(|&ts| ts > cutoff);

        // Check limit
        if self.timestamps.len() >= config.max_requests as usize {
            // Calculate when the oldest request will expire
            if let Some(&oldest) = self.timestamps.first() {
                let retry_after = (oldest + config.window_seconds).saturating_sub(now);
                return Err(retry_after);
            }
            return Err(config.window_seconds);
        }

        // Add current request
        self.timestamps.push(now);
        Ok(())
    }
}

/// Configuration for enhanced rate limiting
#[derive(Debug, Clone)]
pub struct EnhancedRateLimitConfig {
    /// Maximum number of requests allowed
    pub max_requests: u32,
    /// Time window in seconds
    pub window_seconds: u64,
}

impl EnhancedRateLimitConfig {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
        }
    }
}

/// Enhanced rate limiter with per-IP tracking
#[derive(Clone)]
pub struct EnhancedRateLimiter {
    config: EnhancedRateLimitConfig,
    ip_map: Arc<DashMap<String, RateLimitEntry>>,
}

impl EnhancedRateLimiter {
    pub fn new(config: EnhancedRateLimitConfig) -> Self {
        Self {
            config,
            ip_map: Arc::new(DashMap::new()),
        }
    }

    /// Check if a request from an IP is allowed
    pub fn check_rate_limit(&self, ip: &str) -> Result<(), ApiError> {
        let mut entry = self.ip_map
            .entry(ip.to_string())
            .or_insert_with(RateLimitEntry::new);

        match entry.check_and_add(&self.config) {
            Ok(()) => Ok(()),
            Err(retry_after_seconds) => Err(ApiError::RateLimitExceeded {
                retry_after_seconds,
            }),
        }
    }

    /// Periodic cleanup of expired entries
    pub fn cleanup_expired(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.ip_map.retain(|_, entry| {
            // Remove entries that haven't been accessed in 2x the window
            now - entry.last_cleanup < self.config.window_seconds * 2
        });
    }

    /// Middleware factory for use with Axum
    pub async fn middleware(
        self,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        request: Request<Body>,
        next: Next,
    ) -> Result<Response, ApiError> {
        let ip = addr.ip().to_string();

        // Check rate limit
        self.check_rate_limit(&ip)?;

        // Process request
        Ok(next.run(request).await)
    }
}

// Predefined rate limiters for different endpoint types

/// Auth endpoints: 5 requests per minute per IP
pub fn auth_rate_limiter() -> EnhancedRateLimiter {
    EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(5, 60))
}

/// Trading endpoints: 30 requests per minute per IP
pub fn trading_rate_limiter() -> EnhancedRateLimiter {
    EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(30, 60))
}

/// General API endpoints: 100 requests per minute per IP
pub fn api_rate_limiter() -> EnhancedRateLimiter {
    EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(100, 60))
}

/// Admin endpoints: 10 requests per minute per IP
pub fn admin_rate_limiter() -> EnhancedRateLimiter {
    EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(10, 60))
}

/// WebSocket endpoints: 5 connections per minute per IP
pub fn websocket_rate_limiter() -> EnhancedRateLimiter {
    EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(5, 60))
}

/// Meter data submission: 60 requests per minute per IP
pub fn meter_rate_limiter() -> EnhancedRateLimiter {
    EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(60, 60))
}

/// Public/health endpoints: 200 requests per minute per IP
pub fn public_rate_limiter() -> EnhancedRateLimiter {
    EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(200, 60))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(3, 60));

        // Should allow first 3 requests
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
    }

    #[test]
    fn test_rate_limiter_blocks_exceeding_limit() {
        let limiter = EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(2, 60));

        // First 2 should pass
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());

        // Third should fail
        let result = limiter.check_rate_limit("127.0.0.1");
        assert!(result.is_err());
        if let Err(ApiError::RateLimitExceeded { retry_after_seconds }) = result {
            assert!(retry_after_seconds > 0 && retry_after_seconds <= 60);
        }
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let limiter = EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(2, 1)); // 1 second window

        // First 2 requests
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());

        // Third should fail
        assert!(limiter.check_rate_limit("127.0.0.1").is_err());

        // Wait for window to expire
        std::thread::sleep(Duration::from_secs(2));

        // Should work again
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
    }

    #[test]
    fn test_rate_limiter_per_ip() {
        let limiter = EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(1, 60));

        // Different IPs should have independent limits
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        assert!(limiter.check_rate_limit("192.168.1.1").is_ok());

        // But same IP should be limited
        assert!(limiter.check_rate_limit("127.0.0.1").is_err());
    }

    #[test]
    fn test_cleanup_expired_entries() {
        let limiter = EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(10, 1));

        // Add some requests
        let _ = limiter.check_rate_limit("127.0.0.1");
        let _ = limiter.check_rate_limit("192.168.1.1");

        assert_eq!(limiter.ip_map.len(), 2);

        // Wait for entries to expire
        std::thread::sleep(Duration::from_secs(3));

        // Cleanup
        limiter.cleanup_expired();

        // Entries should be removed
        assert_eq!(limiter.ip_map.len(), 0);
    }

    #[test]
    fn test_predefined_limiters() {
        // Just verify they can be created
        let _ = auth_rate_limiter();
        let _ = trading_rate_limiter();
        let _ = api_rate_limiter();
        let _ = admin_rate_limiter();
        let _ = websocket_rate_limiter();
        let _ = meter_rate_limiter();
        let _ = public_rate_limiter();
    }

    #[test]
    fn test_auth_limiter_config() {
        let limiter = auth_rate_limiter();
        assert_eq!(limiter.config.max_requests, 5);
        assert_eq!(limiter.config.window_seconds, 60);
    }

    #[test]
    fn test_retry_after_calculation() {
        let limiter = EnhancedRateLimiter::new(EnhancedRateLimitConfig::new(1, 5));

        // First request passes
        assert!(limiter.check_rate_limit("127.0.0.1").is_ok());

        // Second fails with retry_after
        if let Err(ApiError::RateLimitExceeded { retry_after_seconds }) = limiter.check_rate_limit("127.0.0.1") {
            assert!(retry_after_seconds > 0);
            assert!(retry_after_seconds <= 5);
        } else {
            panic!("Expected RateLimitExceeded error");
        }
    }

    #[test]
    fn test_rate_limit_config_auth() {
        let config = RateLimitConfig::auth();
        let (max, window) = config.limits();
        assert_eq!(max, 5);
        assert_eq!(window, 60);
    }

    #[test]
    fn test_rate_limit_config_trading() {
        let config = RateLimitConfig::trading();
        let (max, window) = config.limits();
        assert_eq!(max, 30);
        assert_eq!(window, 60);
    }
}
