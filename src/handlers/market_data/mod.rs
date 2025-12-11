//! Market Data API Module
//!
//! Provides endpoints for:
//! - Real-time market statistics and order book depth
//! - Historical trade data
//! - Market clearing price information

pub mod history;
pub mod latest;
pub mod types;

pub use history::*;
pub use latest::*;
pub use types::*;
