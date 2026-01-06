//! WebSocket API Module
//! 
//! Provides:
//! - Connection management
//! - Real-time market updates
//! - Authenticated user notifications

pub mod broadcaster;
pub mod handlers;
pub mod manager;
pub mod types;

pub use broadcaster::*;
pub use handlers::*;
pub use manager::*;
pub use types::*;

use once_cell::sync::Lazy;
use std::sync::Arc;

/// Global connection manager for routing messages to users
static CONNECTION_MANAGER: Lazy<Arc<ConnectionManager>> = Lazy::new(|| {
    Arc::new(ConnectionManager::new())
});

/// Get the global connection manager
pub fn get_connection_manager() -> Arc<ConnectionManager> {
    CONNECTION_MANAGER.clone()
}
