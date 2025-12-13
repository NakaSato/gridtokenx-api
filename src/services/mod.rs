//! Services module - Minimal build for testing Simulator → Gateway → Anchor flow
//!
//! Many services are disabled to avoid SQLx compile-time checking issues.

// Core services that don't use SQLx macros heavily
pub mod auth;
pub mod blockchain;
pub mod cache;
pub mod email;
pub mod health_check;
pub mod wallet;
pub mod websocket;

// Re-exports
pub use auth::AuthService;
pub use blockchain::BlockchainService;
pub use cache::CacheService;
pub use email::EmailService;
pub use health_check::HealthChecker;
pub use wallet::WalletService;
pub use websocket::WebSocketService;
