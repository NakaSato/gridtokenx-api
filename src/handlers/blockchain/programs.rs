use axum::{
    extract::{Path, State},
    response::Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::error::{ApiError, Result};
use crate::AppState;

use super::types::*;

/// Interact with a specific smart contract program
/// POST /api/blockchain/programs/:name
#[utoipa::path(
    post,
    path = "/api/blockchain/programs/{name}",
    tag = "blockchain",
    request_body = ProgramInteractionRequest,
    security(("bearer_auth" = [])),
    params(
        ("name" = String, Path, description = "Program name (registry, trading, energy-token, oracle, governance)")
    ),
    responses(
        (status = 200, description = "Program interaction submitted", body = TransactionResponse),
        (status = 400, description = "Invalid program name or request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn interact_with_program(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(program_name): Path<String>,
    Json(payload): Json<ProgramInteractionRequest>,
) -> Result<Json<TransactionResponse>> {
    tracing::info!(
        "Program interaction request for: {} by user: {}",
        program_name,
        user.0.sub
    );

    // Validate program name
    let valid_programs = vec![
        "registry",
        "trading",
        "energy-token",
        "oracle",
        "governance",
    ];
    if !valid_programs.contains(&program_name.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid program name: {}",
            program_name
        )));
    }

    // Validate instruction
    if payload.instruction.is_empty() {
        return Err(ApiError::BadRequest(
            "Instruction cannot be empty".to_string(),
        ));
    }

    // Simulate program interaction
    let signature = format!(
        "prog_{}_{}",
        program_name,
        Uuid::new_v4().to_string().replace('-', "")
    );

    // Log program interaction
    sqlx::query!(
        r#"
        INSERT INTO blockchain_transactions 
        (signature, user_id, program_id, instruction_name, status, compute_units_consumed, submitted_at)
        VALUES ($1, $2, $3, $4, 'Pending', $5, NOW())
        "#,
        signature,
        user.0.sub,
        program_name,
        payload.instruction.clone(),
        payload.compute_units.unwrap_or(10000) as i32
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to log program interaction: {}", e);
        ApiError::Database(e)
    })?;

    let response = TransactionResponse {
        signature: signature.clone(),
        status: "pending".to_string(),
        submitted_at: Utc::now(),
        estimated_confirmation_time: 15,
    };

    tracing::info!("Program interaction submitted: {}", signature);
    Ok(Json(response))
}
