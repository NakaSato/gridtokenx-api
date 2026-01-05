use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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
    /// Transaction status update (for user's own transactions)
    TransactionStatusUpdate {
        operation_id: Uuid,
        transaction_type: String, // "EnergyTrade", "TokenMint", "Stake", etc.
        old_status: String,
        new_status: String,
        signature: Option<String>,
        error_message: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// P2P order lifecycle update
    P2POrderUpdate {
        order_id: Uuid,
        user_id: Uuid,
        side: String,              // "buy" or "sell"
        status: String,            // "open", "partially_filled", "filled", "cancelled"
        original_amount: String,
        filled_amount: String,
        remaining_amount: String,
        price_per_kwh: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Settlement completed notification
    SettlementComplete {
        settlement_id: Uuid,
        buyer_id: Uuid,
        seller_id: Uuid,
        energy_amount: String,
        total_cost: String,
        transaction_signature: Option<String>,
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

/// Order book data structure
#[derive(Debug, Serialize, ToSchema)]
pub struct OrderBookData {
    pub buys: Vec<OrderBookEntry>,
    pub sells: Vec<OrderBookEntry>,
}
