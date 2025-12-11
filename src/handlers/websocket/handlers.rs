use axum::{
    extract::{ws::WebSocketUpgrade, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use tracing::info;

use super::types::WsParams;
use crate::AppState;

/// Authenticated WebSocket endpoint for trading
///
/// Provides real-time updates for authenticated users:
/// - Order updates
/// - Match notifications
/// - Epoch transitions
/// - Personal trading events
///
/// Requires token authentication via query parameter
#[utoipa::path(
    get,
    path = "/ws",
    tag = "websocket",
    params(
        ("token" = String, Query, description = "JWT authentication token")
    ),
    responses(
        (status = 101, description = "WebSocket connection upgraded"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<WsParams>,
) -> Result<Response, Response> {
    // Validate token if provided
    if let Some(token) = &params.token {
        // Decode and validate JWT token using the JWT service from state
        match state.jwt_service.decode_token(token) {
            Ok(claims) => {
                info!(
                    "📡 Authenticated WebSocket connection for user: {}",
                    claims.sub
                );

                // Upgrade to WebSocket
                Ok(ws.on_upgrade(move |socket| async move {
                    state.websocket_service.register_client(socket).await;
                }))
            }
            Err(e) => {
                info!("❌ WebSocket auth failed: {:?}", e);
                let error_response = (
                    axum::http::StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "unauthorized",
                        "message": "Invalid or expired token"
                    })),
                );
                Err(error_response.into_response())
            }
        }
    } else {
        info!("❌ WebSocket connection attempt without token");
        let error_response = (
            axum::http::StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "unauthorized",
                "message": "Token is required"
            })),
        );
        Err(error_response.into_response())
    }
}

/// Real-time market feed WebSocket endpoint
///
/// Provides real-time updates for:
/// - New offers created
/// - New orders placed  
/// - Order matches
/// - Transaction updates
/// - Market statistics
#[utoipa::path(
    get,
    path = "/api/market/ws",
    tag = "websocket",
    responses(
        (status = 101, description = "WebSocket connection upgraded"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn market_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("📡 New WebSocket connection request for market feed");

    ws.on_upgrade(move |socket| async move {
        state.websocket_service.register_client(socket).await;
    })
}

/// Get WebSocket connection statistics
#[utoipa::path(
    get,
    path = "/api/ws/stats",
    tag = "websocket",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "WebSocket statistics")
    )
)]
pub async fn websocket_stats(State(_state): State<AppState>) -> Json<Value> {
    let stats = json!({
        "active_connections": 0,
        "channels": ["order-book", "orders", "matches", "epochs"],
        "uptime_seconds": 0,
        "status": "WebSocket infrastructure ready"
    });

    Json(stats)
}
