//! Analytics API Module
//!
//! Provides endpoints for:
//! - Market analytics (volume, prices, etc.)
//! - User trading statistics

pub mod market;
pub mod types;
pub mod user;

pub use market::*;
pub use types::*;
pub use user::*;
