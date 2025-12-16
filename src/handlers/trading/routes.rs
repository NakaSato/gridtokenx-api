use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::app_state::AppState;
use super::orders::{create_order, cancel_order, update_order, get_order_book, get_user_orders};
use super::blockchain::{get_blockchain_market_data, match_blockchain_orders};

/// Build the v1 trading routes
pub fn v1_trading_routes() -> Router<AppState> {
    Router::new()
        // Orders
        .route("/orders", post(create_order).get(get_user_orders))
        .route("/orders/{id}", delete(cancel_order).put(update_order))
        
        // Order Book
        .route("/orderbook", get(get_order_book))
        
        // Market Data
        .route("/market/blockchain", get(get_blockchain_market_data))
        
        // Admin
        .route("/admin/match-orders", post(match_blockchain_orders))
}
