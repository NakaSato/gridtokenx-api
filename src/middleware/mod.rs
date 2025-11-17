// Middleware module - authentication, rate limiting, CORS, logging, security, etc.

pub mod request_logger;
pub mod metrics;
pub mod security_headers;
pub mod rate_limiter;

pub use request_logger::{
    request_logger_middleware, auth_logger_middleware,
    trading_logger_middleware, websocket_logger_middleware,
    performance_logger_middleware, StructuredLogEntry
};
pub use metrics::{
    metrics_middleware, active_requests_middleware,
    track_auth_attempt, track_auth_failure,
    track_trading_operation, track_order_created, track_order_matched,
    track_websocket_connection, track_database_operation,
    track_blockchain_operation, track_cache_operation,
    track_token_mint, track_meter_reading, track_rate_limit_hit
};
pub use security_headers::add_security_headers;
pub use rate_limiter::{
    EnhancedRateLimiter, EnhancedRateLimitConfig,
    auth_rate_limiter, trading_rate_limiter, api_rate_limiter,
    admin_rate_limiter, websocket_rate_limiter, meter_rate_limiter,
    public_rate_limiter, RateLimiterState, RateLimitConfig, rate_limit_middleware
};