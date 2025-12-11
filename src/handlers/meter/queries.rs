//! Meter reading query handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use tracing::{error, info};

use crate::{
    auth::middleware::AuthenticatedUser,
    error::{ApiError, Result},
    handlers::{require_role, SortOrder},
    utils::PaginationParams,
    AppState,
};

use super::types::{GetReadingsQuery, MeterReadingResponse, MeterReadingsResponse};

/// Get meter readings for current user
/// GET /api/meters/my-readings
#[utoipa::path(
    get,
    path = "/api/meters/my-readings",
    tag = "meters",
    params(GetReadingsQuery),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of user's meter readings", body = Vec<MeterReadingResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_my_readings(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(mut query): Query<GetReadingsQuery>,
) -> Result<Json<MeterReadingsResponse>> {
    info!("User {} fetching their meter readings", user.sub);

    // Validate query parameters
    query.validate()?;

    let limit = query.limit();
    let offset = query.offset();
    let sort_by = query.get_sort_field();
    let sort_order = query.sort_direction();

    // Get total count
    let total = state
        .meter_service
        .count_user_readings(user.sub, query.minted)
        .await
        .map_err(|e| {
            error!("Failed to count user readings: {}", e);
            ApiError::Internal(format!("Failed to count readings: {}", e))
        })?;

    // Get readings with pagination
    let readings = state
        .meter_service
        .get_user_readings(user.sub, limit, offset, sort_by, sort_order, query.minted)
        .await
        .map_err(|e| {
            error!("Failed to fetch user readings: {}", e);
            ApiError::Internal(format!("Failed to fetch readings: {}", e))
        })?;

    let data: Vec<MeterReadingResponse> = readings
        .into_iter()
        .filter_map(|r| {
            // Only include readings with all required fields
            Some(MeterReadingResponse {
                id: r.id,
                user_id: r.user_id?,
                wallet_address: r.wallet_address,
                kwh_amount: r.kwh_amount?,
                reading_timestamp: r.reading_timestamp?,
                submitted_at: r.submitted_at?,
                minted: r.minted.unwrap_or(false),
                mint_tx_signature: r.mint_tx_signature,
            })
        })
        .collect();

    // Create pagination metadata
    let sort_order_util = match query.sort_order {
        SortOrder::Asc => crate::utils::SortOrder::Asc,
        SortOrder::Desc => crate::utils::SortOrder::Desc,
    };
    let pagination = crate::utils::PaginationMeta::new(
        &PaginationParams {
            page: query.page,
            page_size: query.per_page,
            sort_by: Some(query.sort_by.clone()),
            sort_order: sort_order_util,
        },
        total,
    );

    Ok(Json(MeterReadingsResponse { data, pagination }))
}

/// Get meter readings by wallet address
/// GET /api/meters/readings/:wallet_address
#[utoipa::path(
    get,
    path = "/api/meters/readings/{wallet_address}",
    tag = "meters",
    params(
        ("wallet_address" = String, Path, description = "Solana wallet address"),
        GetReadingsQuery
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Meter readings for specified wallet", body = Vec<MeterReadingResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_readings_by_wallet(
    State(state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(wallet_address): Path<String>,
    Query(mut query): Query<GetReadingsQuery>,
) -> Result<Json<MeterReadingsResponse>> {
    info!("Fetching readings for wallet: {}", wallet_address);

    // Validate query parameters
    query.validate()?;

    let limit = query.limit();
    let offset = query.offset();
    let sort_by = query.get_sort_field();
    let sort_order = query.sort_direction();

    // Get total count
    let total = state
        .meter_service
        .count_wallet_readings(&wallet_address, query.minted)
        .await
        .map_err(|e| {
            error!("Failed to count wallet readings: {}", e);
            ApiError::Internal(format!("Failed to count readings: {}", e))
        })?;

    // Get readings with pagination
    let readings = state
        .meter_service
        .get_readings_by_wallet(
            &wallet_address,
            limit,
            offset,
            sort_by,
            sort_order,
            query.minted,
        )
        .await
        .map_err(|e| {
            error!("Failed to fetch wallet readings: {}", e);
            ApiError::Internal(format!("Failed to fetch readings: {}", e))
        })?;

    let data: Vec<MeterReadingResponse> = readings
        .into_iter()
        .filter_map(|r| {
            Some(MeterReadingResponse {
                id: r.id,
                user_id: r.user_id?,
                wallet_address: r.wallet_address,
                kwh_amount: r.kwh_amount?,
                reading_timestamp: r.reading_timestamp?,
                submitted_at: r.submitted_at?,
                minted: r.minted.unwrap_or(false),
                mint_tx_signature: r.mint_tx_signature,
            })
        })
        .collect();

    // Create pagination metadata
    let sort_order_util = match query.sort_order {
        SortOrder::Asc => crate::utils::SortOrder::Asc,
        SortOrder::Desc => crate::utils::SortOrder::Desc,
    };
    let pagination = crate::utils::PaginationMeta::new(
        &PaginationParams {
            page: query.page,
            page_size: query.per_page,
            sort_by: Some(query.sort_by.clone()),
            sort_order: sort_order_util,
        },
        total,
    );

    Ok(Json(MeterReadingsResponse { data, pagination }))
}

/// Get unminted readings (admin only)
/// GET /api/admin/meters/unminted
#[utoipa::path(
    get,
    path = "/api/admin/meters/unminted",
    tag = "meters",
    params(GetReadingsQuery),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of unminted meter readings", body = Vec<MeterReadingResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_unminted_readings(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(mut query): Query<GetReadingsQuery>,
) -> Result<Json<Vec<MeterReadingResponse>>> {
    // Check admin permission
    require_role(&user, "admin")?;

    info!("Admin {} fetching unminted readings", user.sub);

    // Validate query parameters
    query.validate()?;

    let readings = state
        .meter_service
        .get_unminted_readings(query.limit())
        .await
        .map_err(|e| {
            error!("Failed to fetch unminted readings: {}", e);
            ApiError::Internal(format!("Failed to fetch readings: {}", e))
        })?;

    let response: Vec<MeterReadingResponse> = readings
        .into_iter()
        .filter_map(|r| {
            // Only include readings with all required fields
            Some(MeterReadingResponse {
                id: r.id,
                user_id: r.user_id?,
                wallet_address: r.wallet_address,
                kwh_amount: r.kwh_amount?,
                reading_timestamp: r.reading_timestamp?,
                submitted_at: r.submitted_at?,
                minted: r.minted.unwrap_or(false),
                mint_tx_signature: r.mint_tx_signature,
            })
        })
        .collect();

    Ok(Json(response))
}
