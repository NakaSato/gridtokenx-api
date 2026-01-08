use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::database::schema::types::{OrderSide, OrderStatus, OrderType};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TradingOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub order_type: OrderType,
    pub side: OrderSide,
    #[schema(value_type = String)]
    pub energy_amount: Decimal,
    #[schema(value_type = String)]
    pub price_per_kwh: Decimal,
    #[schema(value_type = String)]
    pub filled_amount: Decimal,
    pub status: OrderStatus,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub filled_at: Option<DateTime<Utc>>,
    pub epoch_id: Option<Uuid>,
    pub zone_id: Option<i32>,
    pub meter_id: Option<Uuid>,
    pub refund_tx_signature: Option<String>,
    pub order_pda: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TradingOrderDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub energy_amount: Decimal,
    pub price_per_kwh: Decimal,
    pub filled_amount: Option<Decimal>,
    pub status: OrderStatus,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub filled_at: Option<DateTime<Utc>>,
    pub epoch_id: Option<Uuid>,
    pub zone_id: Option<i32>,
    pub meter_id: Option<Uuid>,
    pub refund_tx_signature: Option<String>,
    pub order_pda: Option<String>,
}

impl From<TradingOrderDb> for TradingOrder {
    fn from(db: TradingOrderDb) -> Self {
        Self {
            id: db.id,
            user_id: db.user_id,
            order_type: db.order_type,
            side: db.side,
            energy_amount: db.energy_amount,
            price_per_kwh: db.price_per_kwh,
            filled_amount: db.filled_amount.unwrap_or(Decimal::ZERO),
            status: db.status,
            expires_at: db.expires_at,
            created_at: db.created_at,
            filled_at: db.filled_at,
            epoch_id: db.epoch_id,
            zone_id: db.zone_id,
            meter_id: db.meter_id,
            refund_tx_signature: db.refund_tx_signature,
            order_pda: db.order_pda,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct EscrowRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub order_id: Option<Uuid>,
    #[schema(value_type = String)]
    pub amount: Decimal,
    pub asset_type: String, // 'currency', 'energy'
    pub escrow_type: String, // 'buy_lock', 'sell_lock'
    pub status: String, // 'locked', 'released', 'refunded'
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateOrderRequest {
    pub side: OrderSide,
    
    #[schema(value_type = String, example = "10.5")]
    pub energy_amount: Decimal,
    
    #[schema(value_type = String, example = "0.15")]
    pub price_per_kwh: Option<Decimal>,

    pub order_type: OrderType,

    pub expiry_time: Option<DateTime<Utc>>,

    pub zone_id: Option<i32>,

    pub meter_id: Option<Uuid>,

    /// HMAC-SHA256 signature of the order parameters
    pub signature: Option<String>,
    
    /// Timestamp of when the signature was created
    pub timestamp: Option<i64>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateOrderRequest {
    #[schema(value_type = String)]
    pub energy_amount: Option<Decimal>,
    #[schema(value_type = String)]
    pub price_per_kwh: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MarketData {
    pub current_epoch: u64,
    pub epoch_start_time: DateTime<Utc>,
    pub epoch_end_time: DateTime<Utc>,
    pub status: String,
    pub order_book: OrderBook,
    pub recent_trades: Vec<Trade>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OrderBook {
    pub sell_orders: Vec<TradingOrder>,
    pub buy_orders: Vec<TradingOrder>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Trade {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub price: Decimal,
    #[schema(value_type = String)]
    pub amount: Decimal,
    pub executed_at: DateTime<Utc>,
}

// ==================== Conditional Orders (Stop-Loss/Take-Profit) ====================

/// Type of conditional order trigger
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "trigger_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    /// Sell when price drops below trigger_price (to limit losses)
    StopLoss,
    /// Sell when price rises above trigger_price (to lock in profits)
    TakeProfit,
    /// Dynamic stop that follows price movements
    TrailingStop,
}

/// Status of a conditional order trigger
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "trigger_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TriggerStatus {
    /// Waiting for trigger condition to be met
    Pending,
    /// Trigger condition met, order executed
    Triggered,
    /// Order cancelled by user
    Cancelled,
    /// Order expired before trigger
    Expired,
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerType::StopLoss => write!(f, "stop_loss"),
            TriggerType::TakeProfit => write!(f, "take_profit"),
            TriggerType::TrailingStop => write!(f, "trailing_stop"),
        }
    }
}

impl std::fmt::Display for TriggerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerStatus::Pending => write!(f, "pending"),
            TriggerStatus::Triggered => write!(f, "triggered"),
            TriggerStatus::Cancelled => write!(f, "cancelled"),
            TriggerStatus::Expired => write!(f, "expired"),
        }
    }
}

/// Request to create a conditional (stop-loss/take-profit) order
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateConditionalOrderRequest {
    /// Order side (buy/sell)
    pub side: OrderSide,
    
    /// Amount of energy to trade
    #[schema(value_type = String, example = "10.5")]
    pub energy_amount: Decimal,
    
    /// Price that triggers the order execution
    #[schema(value_type = String, example = "0.10")]
    pub trigger_price: Decimal,
    
    /// Type of conditional order
    pub trigger_type: TriggerType,
    
    /// Optional limit price for the order after triggering (if not set, uses market order)
    #[schema(value_type = String, example = "0.09")]
    pub limit_price: Option<Decimal>,
    
    /// For trailing stop: the offset from peak price
    #[schema(value_type = String, example = "0.02")]
    pub trailing_offset: Option<Decimal>,
    
    /// Optional expiry time for the conditional order
    pub expiry_time: Option<DateTime<Utc>>,
}

/// Response for conditional order creation
#[derive(Debug, Serialize, ToSchema)]
pub struct ConditionalOrderResponse {
    pub id: Uuid,
    pub trigger_type: TriggerType,
    pub trigger_status: TriggerStatus,
    #[schema(value_type = String)]
    pub trigger_price: Decimal,
    pub created_at: DateTime<Utc>,
    pub message: String,
}

/// Full conditional order info
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConditionalOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub side: OrderSide,
    #[schema(value_type = String)]
    pub energy_amount: Decimal,
    #[schema(value_type = String)]
    pub trigger_price: Decimal,
    pub trigger_type: TriggerType,
    pub trigger_status: TriggerStatus,
    #[schema(value_type = String)]
    pub limit_price: Option<Decimal>,
    #[schema(value_type = String)]
    pub trailing_offset: Option<Decimal>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub triggered_at: Option<DateTime<Utc>>,
}

