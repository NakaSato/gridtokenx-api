//! Meter statistics handlers

use axum::{extract::State, Json};
use tracing::{error, info};

use crate::{
    auth::middleware::AuthenticatedUser,
    error::{ApiError, Result},
    AppState,
};

use super::types::UserStatsResponse;

/// Get user statistics
/// GET /api/meters/stats
#[utoipa::path(
    get,
    path = "/api/meters/stats",
    tag = "meters",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User meter reading statistics", body = UserStatsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_user_stats(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<UserStatsResponse>> {
    info!("User {} fetching meter statistics", user.sub);

    // Get unminted and minted totals
    let unminted_total = state
        .meter_service
        .get_unminted_total(user.sub)
        .await
        .map_err(|e| {
            error!("Failed to calculate unminted total: {}", e);
            ApiError::Internal("Failed to fetch statistics".to_string())
        })?;

    let minted_total = state
        .meter_service
        .get_minted_total(user.sub)
        .await
        .map_err(|e| {
            error!("Failed to calculate minted total: {}", e);
            ApiError::Internal("Failed to fetch statistics".to_string())
        })?;

    let total_kwh = unminted_total + minted_total;

    // Count total readings
    let total_readings = state
        .meter_service
        .count_user_readings(user.sub, None)
        .await
        .map_err(|e| {
            error!("Failed to count user readings: {}", e);
            ApiError::Internal("Failed to fetch statistics".to_string())
        })?;

    Ok(Json(UserStatsResponse {
        total_readings,
        unminted_kwh: unminted_total,
        minted_kwh: minted_total,
        total_kwh,
    }))
}
