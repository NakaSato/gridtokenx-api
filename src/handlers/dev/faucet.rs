use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::error::{ApiError, Result};
use crate::AppState;

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct FaucetRequest {
    pub wallet_address: String,
    pub amount_sol: Option<f64>,
    pub mint_tokens_kwh: Option<f64>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FaucetResponse {
    pub success: bool,
    pub message: String,
    pub sol_tx_signature: Option<String>,
    pub token_tx_signature: Option<String>,
}

/// Request funds from the developer faucet
/// POST /api/dev/faucet
#[utoipa::path(
    post,
    path = "/api/dev/faucet",
    tag = "dev",
    request_body = FaucetRequest,
    responses(
        (status = 200, description = "Funds requested successfully", body = FaucetResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn request_faucet(
    State(state): State<AppState>,
    Json(payload): Json<FaucetRequest>,
) -> Result<Json<FaucetResponse>> {
    tracing::info!("Faucet request for wallet: {}", payload.wallet_address);

    let wallet_pubkey = Pubkey::from_str(&payload.wallet_address)
        .map_err(|_| ApiError::BadRequest("Invalid wallet address".to_string()))?;

    let mut sol_sig = None;
    let mut token_sig = None;
    let mut messages = Vec::new();

    // 1. Airdrop SOL
    if let Some(amount) = payload.amount_sol {
        if amount > 0.0 {
            match state
                .wallet_service
                .request_airdrop(&wallet_pubkey, amount)
                .await
            {
                Ok(sig) => {
                    sol_sig = Some(sig.to_string());
                    messages.push(format!("Airdropped {} SOL", amount));
                }
                Err(e) => {
                    tracing::error!("Faucet Airdrop failed: {}", e);
                    // Don't fail the whole request, but note it? 
                    // Or fail? Let's fail if requested explicitly.
                    return Err(ApiError::Internal(format!("Failed to airdrop SOL: {}", e)));
                }
            }
        }
    }

    // 2. Mint Tokens
    if let Some(kwh) = payload.mint_tokens_kwh {
        if kwh > 0.0 {
            // Calculate atomic amount: kwh * 10^9
            let amount_atomic = (kwh * 1_000_000_000.0) as u64;
            
            match state
                .blockchain_service
                .mint_tokens_direct(&wallet_pubkey, amount_atomic)
                .await
            {
                Ok(sig) => {
                    token_sig = Some(sig.to_string());
                    messages.push(format!("Minted {} kWh tokens", kwh));
                }
                Err(e) => {
                    tracing::error!("Faucet Minting failed: {}", e);
                     return Err(ApiError::Internal(format!("Failed to mint tokens: {}", e)));
                }
            }
        }
    }

    Ok(Json(FaucetResponse {
        success: true,
        message: if messages.is_empty() {
            "No actions requested".to_string()
        } else {
            messages.join(", ")
        },
        sol_tx_signature: sol_sig,
        token_tx_signature: token_sig,
    }))
}
