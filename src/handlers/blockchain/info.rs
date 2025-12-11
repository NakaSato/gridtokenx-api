use axum::{
    extract::{Path, State},
    response::Json,
};
use chrono::Utc;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::auth::middleware::AuthenticatedUser;
use crate::error::{ApiError, Result};
use crate::AppState;

use super::types::*;

/// Get account information for a given address
/// GET /api/blockchain/accounts/:address
#[utoipa::path(
    get,
    path = "/api/blockchain/accounts/{address}",
    tag = "blockchain",
    security(("bearer_auth" = [])),
    params(
        ("address" = String, Path, description = "Solana account address (base58)")
    ),
    responses(
        (status = 200, description = "Account information", body = AccountInfo),
        (status = 400, description = "Invalid address format"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_account_info(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(address): Path<String>,
) -> Result<Json<AccountInfo>> {
    tracing::info!(
        "Fetching account info for address: {} by user: {}",
        address,
        user.0.sub
    );

    // Validate address format
    let pubkey = Pubkey::from_str(&address)
        .map_err(|_| ApiError::BadRequest("Invalid address format".to_string()))?;

    // Fetch real account info
    let balance_lamports = state
        .blockchain_service
        .get_balance(&pubkey)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch balance: {}", e)))?;

    let data = state
        .blockchain_service
        .get_account_data(&pubkey)
        .await
        .unwrap_or_default();

    let owner = "11111111111111111111111111111111".to_string(); // Default to system program if unknown
                                                                // In a full implementation we would fetch the full Account object to get owner, executable, etc.

    let account_info = AccountInfo {
        address: address.clone(),
        balance: rust_decimal::Decimal::from(balance_lamports)
            / rust_decimal::Decimal::from(1_000_000_000),
        executable: false, // Placeholder
        owner,
        rent_epoch: 0,
        data_length: data.len(),
    };

    Ok(Json(account_info))
}

/// Get current network status
/// GET /api/blockchain/network
#[utoipa::path(
    get,
    path = "/api/blockchain/network",
    tag = "blockchain",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current blockchain network status", body = NetworkStatus),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_network_status(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<NetworkStatus>> {
    tracing::info!("Fetching network status");

    let slot = state
        .blockchain_service
        .get_slot()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get slot: {}", e)))?;

    let is_healthy = state
        .blockchain_service
        .health_check()
        .await
        .unwrap_or(false);

    let network_status = NetworkStatus {
        cluster: state.blockchain_service.cluster().to_string(),
        block_height: slot,
        block_time: Utc::now(),
        tps: 0.0, // Would need more complex calculation
        health: if is_healthy {
            "ok".to_string()
        } else {
            "degraded".to_string()
        },
        version: "unknown".to_string(),
    };

    Ok(Json(network_status))
}
