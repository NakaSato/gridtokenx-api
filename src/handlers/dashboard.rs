use crate::error::{ApiError, Result};
use crate::services::dashboard::{DashboardMetrics, DashboardService};
use axum::{extract::State, routing::get, Json, Router};

/// Routes for dashboard metrics
pub fn v1_dashboard_routes() -> Router<crate::AppState> {
    Router::new().route("/metrics", get(get_dashboard_metrics))
}

/// Get dashboard metrics
#[utoipa::path(
    get,
    path = "/api/dashboard/metrics",
    tag = "Dashboard",
    responses(
        (status = 200, description = "Dashboard metrics", body = DashboardMetrics),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_dashboard_metrics(
    State(dashboard_service): State<DashboardService>,
) -> Result<Json<DashboardMetrics>> {
    let metrics = dashboard_service
        .get_metrics()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(metrics))
}
