use uuid::Uuid;

use super::types::{OrderBookData, OrderBookEntry, WsMessage};
use crate::AppState;

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
    println!(
        "Broadcasting order book update for epoch {} with {} buys and {} sells",
        epoch_number,
        order_book.buys.len(),
        order_book.sells.len()
    );

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
    println!(
        "Broadcasting match notification: {} tokens matched @ {}",
        matched_amount, match_price
    );

    Ok(())
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

/// Broadcast transaction status update to the relevant user
pub async fn broadcast_transaction_status_update(
    operation_id: Uuid,
    transaction_type: String,
    old_status: String,
    new_status: String,
    signature: Option<String>,
    error_message: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _message = WsMessage::TransactionStatusUpdate {
        operation_id,
        transaction_type: transaction_type.clone(),
        old_status: old_status.clone(),
        new_status: new_status.clone(),
        signature,
        error_message,
        timestamp: chrono::Utc::now(),
    };

    // TODO: Use connection manager to send to specific user's WebSocket
    tracing::info!(
        "Broadcasting transaction status update: {} {} -> {}",
        transaction_type,
        old_status,
        new_status
    );

    Ok(())
}

/// Broadcast P2P order update to the order owner
pub async fn broadcast_p2p_order_update(
    order_id: Uuid,
    user_id: Uuid,
    side: String,
    status: String,
    original_amount: String,
    filled_amount: String,
    remaining_amount: String,
    price_per_kwh: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _message = WsMessage::P2POrderUpdate {
        order_id,
        user_id,
        side: side.clone(),
        status: status.clone(),
        original_amount: original_amount.clone(),
        filled_amount: filled_amount.clone(),
        remaining_amount: remaining_amount.clone(),
        price_per_kwh,
        timestamp: chrono::Utc::now(),
    };

    // TODO: Use connection manager to send to specific user's WebSocket
    tracing::info!(
        "Broadcasting P2P order update: {} {} order {} - filled {}/{}",
        side,
        status,
        order_id,
        filled_amount,
        original_amount
    );

    Ok(())
}

/// Broadcast settlement completion to both buyer and seller
pub async fn broadcast_settlement_complete(
    settlement_id: Uuid,
    buyer_id: Uuid,
    seller_id: Uuid,
    energy_amount: String,
    total_cost: String,
    transaction_signature: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _message = WsMessage::SettlementComplete {
        settlement_id,
        buyer_id,
        seller_id,
        energy_amount: energy_amount.clone(),
        total_cost: total_cost.clone(),
        transaction_signature,
        timestamp: chrono::Utc::now(),
    };

    // TODO: Use connection manager to send to both buyer and seller WebSockets
    tracing::info!(
        "Broadcasting settlement complete: {} - {} kWh for {} THB",
        settlement_id,
        energy_amount,
        total_cost
    );

    Ok(())
}

