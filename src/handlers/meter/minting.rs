//! Token minting from meter readings

use axum::{extract::State, Json};
use rust_decimal::prelude::ToPrimitive;
use tracing::{error, info};

use crate::{
    auth::middleware::AuthenticatedUser,
    error::{ApiError, Result},
    handlers::require_role,
    services::BlockchainService,
    AppState,
};

use super::types::{MintFromReadingRequest, MintResponse};

/// Mint tokens from a meter reading (admin only)
/// POST /api/admin/meters/mint-from-reading
///
/// This endpoint mints energy tokens based on a submitted meter reading
#[utoipa::path(
    post,
    path = "/api/admin/meters/mint-from-reading",
    tag = "meters",
    request_body = MintFromReadingRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Tokens minted successfully", body = MintResponse),
        (status = 400, description = "Invalid reading or already minted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Reading not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn mint_from_reading(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(request): Json<MintFromReadingRequest>,
) -> Result<Json<MintResponse>> {
    // Check admin permission
    require_role(&user, "admin")?;

    info!(
        "Admin {} minting tokens for reading {}",
        user.sub, request.reading_id
    );

    // Get reading details
    let reading = state
        .meter_service
        .get_reading_by_id(request.reading_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch reading: {}", e);
            ApiError::NotFound("Reading not found".to_string())
        })?;

    // Check if already minted
    if reading.minted.unwrap_or(false) {
        return Err(ApiError::BadRequest(
            "Reading has already been minted".to_string(),
        ));
    }

    let kwh_amount = reading
        .kwh_amount
        .ok_or_else(|| ApiError::Internal("Missing kwh_amount".to_string()))?;

    let wallet_address = reading.wallet_address;

    // Get authority keypair
    let authority_keypair = state
        .wallet_service
        .get_authority_keypair()
        .await
        .map_err(|e| {
            error!("Failed to get authority keypair: {}", e);
            ApiError::Internal("Failed to access blockchain".to_string())
        })?;

    // Parse addresses
    let token_mint = BlockchainService::parse_pubkey(&state.config.energy_token_mint)
        .map_err(|e| ApiError::Internal(format!("Invalid token mint: {}", e)))?;

    let wallet_pubkey = BlockchainService::parse_pubkey(&wallet_address)
        .map_err(|e| ApiError::BadRequest(format!("Invalid wallet address: {}", e)))?;

    // Ensure user token account exists
    let user_token_account = state
        .blockchain_service
        .ensure_token_account_exists(&authority_keypair, &wallet_pubkey, &token_mint)
        .await
        .map_err(|e| {
            error!("Failed to ensure token account: {}", e);
            ApiError::Internal("Failed to create token account".to_string())
        })?;

    // Mint tokens
    let amount_f64 = kwh_amount
        .to_f64()
        .ok_or_else(|| ApiError::Internal("Failed to convert amount".to_string()))?;

    let signature = state
        .blockchain_service
        .mint_energy_tokens(
            &authority_keypair,
            &user_token_account,
            &wallet_pubkey,
            &token_mint,
            amount_f64,
        )
        .await
        .map_err(|e| {
            error!("Failed to mint tokens: {}", e);
            ApiError::Internal(format!("Blockchain minting failed: {}", e))
        })?;

    let sig_str = signature.to_string();
    info!(
        "Minted {} kWh for reading {}: {}",
        amount_f64, request.reading_id, sig_str
    );

    // Mark reading as minted
    state
        .meter_service
        .mark_as_minted(request.reading_id, &sig_str)
        .await
        .map_err(|e| {
            error!("Failed to update reading: {}", e);
            ApiError::Internal("Failed to update reading status".to_string())
        })?;

    Ok(Json(MintResponse {
        message: "Tokens minted successfully".to_string(),
        transaction_signature: sig_str,
        kwh_amount,
        wallet_address,
    }))
}
