use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde_json::Value;
use crate::app_state::AppState;
use tracing::{error, debug};

/// Proxy RPC requests to Solana validator
pub async fn rpc_handler(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let rpc_url = &state.config.solana_rpc_url;
    
    debug!("Proxying RPC request to {}", rpc_url);

    let client = reqwest::Client::new();
    
    let res = match client.post(rpc_url)
        .json(&payload)
        .send()
        .await {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to proxy RPC request: {}", e);
                return (
                    axum::http::StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32603,
                            "message": "Internal error proxying request"
                        },
                        "id": payload.get("id")
                    }))
                ).into_response();
            }
        };

    let status = res.status();
    let body: Value = match res.json().await {
        Ok(b) => b,
        Err(e) => {
             error!("Failed to parse RPC response: {}", e);
             return (
                axum::http::StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32603,
                        "message": "Invalid JSON response from upstream"
                    },
                    "id": payload.get("id")
                }))
            ).into_response();
        }
    };

    (status, Json(body)).into_response()
}
