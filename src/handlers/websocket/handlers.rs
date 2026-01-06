use axum::{
    extract::{ws::{WebSocketUpgrade, Message, WebSocket}, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tracing::{info, error};
use uuid::Uuid;


use super::types::WsParams;
use super::get_connection_manager;
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
                let user_id = claims.sub;
                info!(
                    "📡 Authenticated WebSocket connection for user: {}",
                    user_id
                );

                // Upgrade to WebSocket with user context
                Ok(ws.on_upgrade(move |socket| async move {
                    handle_authenticated_socket(socket, user_id, state).await;
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

/// Handle authenticated WebSocket connection
async fn handle_authenticated_socket(socket: WebSocket, user_id: Uuid, _state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Register with connection manager
    let manager = get_connection_manager();
    let mut broadcast_rx = manager.add_connection(user_id).await;
    
    info!("📡 User {} connected via WebSocket", user_id);

    // Also register with the general WebSocket service for market broadcasts
    // The state.websocket_service handles general market events
    
    // Spawn task to forward broadcasts to this client
    let forward_task = tokio::spawn(async move {
        while let Ok(message) = broadcast_rx.recv().await {
            // Serialize message to JSON
            if let Ok(json) = serde_json::to_string(&message) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break; // Connection closed
                }
            }
        }
    });

    // Handle incoming messages from client
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Handle client messages (ping, subscribe, etc.)
                if text.contains("ping") {
                    // Pong handled automatically by axum
                }
            }
            Ok(Message::Close(_)) => {
                break;
            }
            Err(e) => {
                error!("WebSocket error for user {}: {}", user_id, e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup on disconnect
    forward_task.abort();
    manager.remove_connection(&user_id).await;
    info!("📡 User {} disconnected from WebSocket", user_id);
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
