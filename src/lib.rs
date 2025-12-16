//! API Gateway - Minimal build for Simulator → Gateway → Anchor testing

pub mod app_state;
pub mod auth;
pub mod config;
pub mod constants;
pub mod database;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
// pub mod openapi; // Disabled - references disabled modules
pub mod router;
pub mod services;
pub mod startup;
pub mod utils;

pub use app_state::AppState;
pub use config::Config;
pub use error::ApiError;
