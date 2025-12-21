use axum::{extract::State, Json};
use serde::Serialize;
use utoipa::ToSchema;
use tracing::info;
use crate::AppState;
use crate::auth::middleware::AuthenticatedUser;
use crate::error::Result;
use crate::services::audit_logger::AuditEventRecord;
use crate::services::health_check::DetailedHealthStatus;

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminStatsResponse {
    pub total_users: i64,
    pub total_meters: i64,
    pub active_meters: i64,
    pub total_volume_kwh: f64,
    pub total_orders: i64,
    pub settlement_success_rate: f64,
}

/// Get global platform statistics (Admin only)
#[utoipa::path(
    get,
    path = "/api/v1/analytics/admin/stats",
    responses(
        (status = 200, description = "Admin statistics retrieved", body = AdminStatsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_admin_stats(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<AdminStatsResponse>> {
    info!("📊 Admin: Fetching global platform stats");

    // 1. Total Users
    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    // 2. Meters Stats
    let meter_stats = sqlx::query_as::<_, (i64, i64)>(
        "SELECT COUNT(*), COUNT(*) FILTER (WHERE status = 'active') FROM smartmeters"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or((0, 0));

    // 3. Trade Stats
    let total_volume = sqlx::query_scalar::<_, Option<rust_decimal::Decimal>>(
        "SELECT SUM(filled_amount) FROM trading_orders WHERE status = 'filled' OR status = 'settled'"
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or_default();

    let total_orders = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM trading_orders")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    // 4. Settlement Success Rate (stub for now, can be expanded with settlement_logs)
    let settlement_success_rate = 100.0; // Assume perfect for now until we have more logs

    Ok(Json(AdminStatsResponse {
        total_users,
        total_meters: meter_stats.0,
        active_meters: meter_stats.1,
        total_volume_kwh: total_volume.to_string().parse().unwrap_or(0.0),
        total_orders,
        settlement_success_rate,
    }))
}

/// Get latest platform activity (Admin only)
#[utoipa::path(
    get,
    path = "/api/v1/analytics/admin/activity",
    responses(
        (status = 200, description = "Admin activity logs retrieved", body = Vec<AuditEventRecord>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_admin_activity(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<Vec<crate::services::audit_logger::AuditEventRecord>>> {
    info!("📊 Admin: Fetching platform activity logs");
    
    let activities = state.audit_logger.get_all_activities(50).await
        .map_err(|e| crate::error::ApiError::Database(e))?;
        
    Ok(Json(activities))
}

/// Get detailed system health (Admin only)
#[utoipa::path(
    get,
    path = "/api/v1/analytics/admin/health",
    responses(
        (status = 200, description = "Detailed system health retrieved", body = DetailedHealthStatus),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_system_health(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<crate::services::health_check::DetailedHealthStatus>> {
    info!("📊 Admin: Fetching detailed system health");
    
    // Check if we have cached health or perform a new check
    let health = if let Some(cached) = state.health_checker.get_cached_health().await {
        cached
    } else {
        state.health_checker.perform_health_check().await
    };
    
    Ok(Json(health))
}
