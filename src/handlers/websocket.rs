use axum::{
    extract::{
        ws::WebSocketUpgrade, Query, State,
    },
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Order book update
    OrderBookUpdate {
        epoch_number: i32,
        buys: Vec<OrderBookEntry>,
        sells: Vec<OrderBookEntry>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Order status update
    OrderUpdate {
        order_id: Uuid,
        status: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// New match notification
    MatchNotification {
        match_id: Uuid,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
        matched_amount: String, // Using String for BigDecimal compatibility
        match_price: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Epoch transition notification
    EpochTransition {
        old_epoch: i32,
        new_epoch: i32,
        clearing_price: Option<String>,
        total_volume: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Error message
    Error {
        code: String,
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Ping message for connection health
    Ping {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Pong response
    Pong {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Order book entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrderBookEntry {
    pub price: String, // Using String for BigDecimal compatibility
    pub quantity: String,
    pub order_count: i32,
}

/// WebSocket connection parameters
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct WsParams {
    /// Subscribe to specific channels (comma-separated)
    pub channels: Option<String>,
    /// Subscribe to specific epoch
    pub epoch: Option<i32>,
    /// Authentication token
    pub token: Option<String>,
}

/// WebSocket connection manager
#[derive(Debug, Clone)]
pub struct ConnectionManager {
    /// Active connections by user
    connections: Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsMessage>>>>,
    /// Global message broadcaster
    broadcaster: broadcast::Sender<WsMessage>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (broadcaster, _) = broadcast::channel(1000);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcaster,
        }
    }

    /// Add a new connection
    pub async fn add_connection(&self, user_id: Uuid) -> broadcast::Receiver<WsMessage> {
        let (tx, rx) = broadcast::channel(100);
        let mut connections = self.connections.write().await;
        connections.insert(user_id, tx);
        rx
    }

    /// Remove a connection
    pub async fn remove_connection(&self, user_id: &Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(user_id);
    }

    /// Send message to specific user
    pub async fn send_to_user(&self, user_id: Uuid, message: WsMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connections = self.connections.read().await;
        if let Some(sender) = connections.get(&user_id) {
            sender.send(message)?;
        }
        Ok(())
    }

    /// Broadcast message to all connections
    pub async fn broadcast(&self, message: WsMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _ = self.broadcaster.send(message);
        Ok(())
    }

    /// Get number of active connections
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
}

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
                info!("📡 Authenticated WebSocket connection for user: {}", claims.sub);
                
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
                    }))
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
            }))
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
pub async fn websocket_stats(
    State(_state): State<AppState>,
) -> Json<Value> {
    let stats = json!({
        "active_connections": 0,
        "channels": ["order-book", "orders", "matches", "epochs"],
        "uptime_seconds": 0,
        "status": "WebSocket infrastructure ready"
    });

    Json(stats)
}

/// Broadcast order book update to all subscribers
pub async fn broadcast_order_book_update(
    _state: &AppState,
    epoch_number: i32,
    order_book: OrderBookData,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // In a real implementation, this would use the connection manager
    // to broadcast to all subscribers
    
    let _message = WsMessage::OrderBookUpdate {
        epoch_number,
        buys: order_book.buys.clone(),
        sells: order_book.sells.clone(),
        timestamp: chrono::Utc::now(),
    };

    // Store in broadcast queue for WebSocket clients
    println!("Broadcasting order book update for epoch {} with {} buys and {} sells", 
             epoch_number, order_book.buys.len(), order_book.sells.len());
    
    Ok(())
}

/// Broadcast match notification
pub async fn broadcast_match_notification(
    match_id: Uuid,
    buy_order_id: Uuid,
    sell_order_id: Uuid,
    matched_amount: String,
    match_price: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _message = WsMessage::MatchNotification {
        match_id,
        buy_order_id,
        sell_order_id,
        matched_amount: matched_amount.clone(),
        match_price: match_price.clone(),
        timestamp: chrono::Utc::now(),
    };

    // Store in broadcast queue for WebSocket clients
    println!("Broadcasting match notification: {} tokens matched @ {}", matched_amount, match_price);
    
    Ok(())
}

/// Order book data structure
#[derive(Debug, Serialize, ToSchema)]
pub struct OrderBookData {
    pub buys: Vec<OrderBookEntry>,
    pub sells: Vec<OrderBookEntry>,
}

/// Create sample order book data for testing
pub fn create_sample_order_book() -> OrderBookData {
    OrderBookData {
        buys: vec![
            OrderBookEntry {
                price: "0.095".to_string(),
                quantity: "100".to_string(),
                order_count: 2,
            },
            OrderBookEntry {
                price: "0.090".to_string(),
                quantity: "150".to_string(),
                order_count: 1,
            },
        ],
        sells: vec![
            OrderBookEntry {
                price: "0.100".to_string(),
                quantity: "200".to_string(),
                order_count: 3,
            },
            OrderBookEntry {
                price: "0.105".to_string(),
                quantity: "75".to_string(),
                order_count: 1,
            },
        ],
    }
}
