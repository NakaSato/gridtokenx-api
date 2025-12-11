use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

/// Query parameters for transaction history
#[derive(Debug, Deserialize, Validate, ToSchema, IntoParams)]
pub struct TransactionQuery {
    pub program_id: Option<String>,
    pub status: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Response for transaction submission
#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionResponse {
    pub signature: String,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub estimated_confirmation_time: i32, // seconds
}

/// Account information response
#[derive(Debug, Serialize, ToSchema)]
pub struct AccountInfo {
    pub address: String,
    #[schema(value_type = String)]
    pub balance: rust_decimal::Decimal,
    pub executable: bool,
    pub owner: String,
    pub rent_epoch: u64,
    pub data_length: usize,
}

/// Network status response
#[derive(Debug, Serialize, ToSchema)]
pub struct NetworkStatus {
    pub cluster: String,
    pub block_height: u64,
    pub block_time: DateTime<Utc>,
    pub tps: f64,
    pub health: String,
    pub version: String,
}

/// Program interaction request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ProgramInteractionRequest {
    pub program_name: String,
    pub instruction: String,
    pub accounts: Vec<String>,
    pub data: serde_json::Value,
    #[validate(range(min = 1000, max = 1000000))]
    pub compute_units: Option<u32>,
}
