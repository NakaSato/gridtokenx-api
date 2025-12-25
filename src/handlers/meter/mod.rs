//! Meter Management Module
//!
//! This module provides API endpoints for meter management including:
//! - Submitting meter readings
//! - Retrieving reading history
//! - Token minting from readings
//! - Meter registration and verification

pub mod minting;
pub mod stub;
pub mod types;

// Re-export from stub module
pub use stub::{
    submit_reading, meter_health,
    SubmitReadingRequest, MeterReadingResponse,
};

// Re-export minting handlers
pub use minting::{mint_from_reading, mint_user_reading};

// Re-export types
pub use types::{MintFromReadingRequest, MintResponse};

