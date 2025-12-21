use axum::{extract::State, response::IntoResponse};
use crate::app_state::AppState;

/// Expose Prometheus metrics
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics", body = String),
    ),
    tag = "system"
)]
pub async fn get_metrics(State(state): State<AppState>) -> impl IntoResponse {
    state.metrics_handle.render()
}
