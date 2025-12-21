use axum::{
    extract::{State, Path, Query},
    routing::{get, post},
    Json, Router
};
use utoipa::{ToSchema, IntoParams};
use uuid::Uuid;
use crate::AppState;
use crate::error::ApiError;
use crate::services::futures::{FuturesProduct, FuturesPosition, Candle, OrderBook, FuturesOrder};
use serde_json::Value;
use serde::Deserialize;
use rust_decimal::Decimal;
use crate::handlers::ApiResponse;
use crate::auth::middleware::AuthenticatedUser;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/products", get(get_products))
        .route("/orders", post(create_order))
        .route("/orders/my", get(get_my_orders))
        .route("/positions", get(get_positions))
        .route("/positions/{id}/close", post(close_position))
        .route("/candles", get(get_candles))
        .route("/orderbook", get(get_order_book))
}

#[derive(Deserialize, ToSchema)]
pub struct CreateFuturesOrderRequest {
    pub product_id: Uuid,
    pub side: String, // 'long' or 'short'
    pub order_type: String, // 'market' or 'limit'
    #[schema(value_type = String)]
    pub quantity: Decimal,
    #[schema(value_type = String)]
    pub price: Decimal,
    pub leverage: i32,
}

/// Get all futures products
#[utoipa::path(
    get,
    path = "/api/v1/futures/products",
    responses(
        (status = 200, description = "List of futures products", body = ApiResponse<Vec<FuturesProduct>>),
    ),
    tag = "futures"
)]
pub async fn get_products(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<FuturesProduct>>>, ApiError> {
    let products = state.futures_service.get_products().await?;
    Ok(Json(ApiResponse::success(products)))
}

/// Create a new futures order
#[utoipa::path(
    post,
    path = "/api/v1/futures/orders",
    request_body = CreateFuturesOrderRequest,
    responses(
        (status = 200, description = "Order created successfully", body = ApiResponse<Value>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = [])),
    tag = "futures"
)]
pub async fn create_order(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateFuturesOrderRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let order_id = state.futures_service.create_order(
        user.0.sub,
        req.product_id,
        req.side,
        req.order_type,
        req.quantity,
        req.price,
        req.leverage,
    ).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({ "order_id": order_id }))))
}

/// Get user's futures positions
#[utoipa::path(
    get,
    path = "/api/v1/futures/positions",
    responses(
        (status = 200, description = "List of user's positions", body = ApiResponse<Vec<FuturesPosition>>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = [])),
    tag = "futures"
)]
pub async fn get_positions(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<FuturesPosition>>>, ApiError> {
    let positions = state.futures_service.get_positions(user.0.sub).await?;
    Ok(Json(ApiResponse::success(positions)))
}

#[derive(Deserialize, IntoParams)]
pub struct GetCandlesRequest {
    pub product_id: Uuid,
    pub interval: String,
}

/// Get candles for a product
#[utoipa::path(
    get,
    path = "/api/v1/futures/candles",
    params(GetCandlesRequest),
    responses(
        (status = 200, description = "Candle data", body = ApiResponse<Vec<Candle>>),
    ),
    tag = "futures"
)]
pub async fn get_candles(
    State(state): State<AppState>,
    Query(req): Query<GetCandlesRequest>,
) -> Result<Json<ApiResponse<Vec<crate::services::futures::Candle>>>, ApiError> {
    let candles = state.futures_service.get_candles(req.product_id, req.interval).await?;
    Ok(Json(ApiResponse::success(candles)))
}

#[derive(Deserialize, IntoParams)]
pub struct GetOrderBookRequest {
    pub product_id: Uuid,
}

/// Get order book for a product
#[utoipa::path(
    get,
    path = "/api/v1/futures/orderbook",
    params(GetOrderBookRequest),
    responses(
        (status = 200, description = "Order book data", body = ApiResponse<OrderBook>),
    ),
    tag = "futures"
)]
pub async fn get_order_book(
    State(state): State<AppState>,
    Query(req): Query<GetOrderBookRequest>,
) -> Result<Json<ApiResponse<crate::services::futures::OrderBook>>, ApiError> {
    let order_book = state.futures_service.get_order_book(req.product_id).await?;
    Ok(Json(ApiResponse::success(order_book)))
}

/// Get user's futures orders
#[utoipa::path(
    get,
    path = "/api/v1/futures/orders/my",
    responses(
        (status = 200, description = "List of user's orders", body = ApiResponse<Vec<FuturesOrder>>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = [])),
    tag = "futures"
)]
pub async fn get_my_orders(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<crate::services::futures::FuturesOrder>>>, ApiError> {
    let orders = state.futures_service.get_user_orders(user.0.sub).await?;
    Ok(Json(ApiResponse::success(orders)))
}

/// Close a futures position
#[utoipa::path(
    post,
    path = "/api/v1/futures/positions/{id}/close",
    params(
        ("id" = Uuid, Path, description = "Position ID")
    ),
    responses(
        (status = 200, description = "Position closed successfully", body = ApiResponse<Value>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = [])),
    tag = "futures"
)]
pub async fn close_position(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(position_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let order_id = state.futures_service.close_position(user.0.sub, position_id).await?;
    Ok(Json(ApiResponse::success(serde_json::json!({ "order_id": order_id }))))
}
