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
