use axum::{routing::post, Router};
use crate::handlers::dev::faucet::request_faucet;
use crate::AppState;

/// Dev routes (faucet, etc.)
pub fn dev_routes() -> Router<AppState> {
    Router::new()
        .route("/faucet", post(request_faucet))
}
