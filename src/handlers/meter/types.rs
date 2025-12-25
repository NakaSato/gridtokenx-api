//! Meter handler types
//!
//! Types for meter minting and reading operations.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to mint tokens from a reading (admin only)
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MintFromReadingRequest {
    /// The reading ID (UUID) to mint tokens from
    pub reading_id: Uuid,
}

/// Response after minting tokens
#[derive(Debug, Serialize, ToSchema)]
pub struct MintResponse {
    /// Success message
    pub message: String,
    /// Transaction signature on Solana
    pub transaction_signature: String,
    /// Amount of kWh minted
    #[schema(value_type = f64)]
    pub kwh_amount: Decimal,
    /// Wallet address that received tokens
    pub wallet_address: String,
}

