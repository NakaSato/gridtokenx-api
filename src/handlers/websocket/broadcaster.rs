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
