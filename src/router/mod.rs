//! Router configuration module - RESTful v1 API
//!
//! Supports both v1 RESTful API and legacy routes for backward compatibility.

use axum::{routing::get, Router, extract::{State, WebSocketUpgrade}, response::IntoResponse, middleware};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod dev;
pub mod public;

use crate::app_state::AppState;
use crate::handlers::{
    // V1 RESTful routes
    v1_auth_routes, v1_users_routes, v1_meters_routes, v1_wallets_routes, v1_status_routes,
    v1_trading_routes,
    // Legacy routes
    auth_routes, token_routes, user_meter_routes, meter_info_routes, meter_routes,
};
use crate::services::WebSocketService;
use crate::auth::middleware::auth_middleware;
use crate::middleware::{metrics_middleware, active_requests_middleware};

/// OpenAPI documentation for GridTokenX API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "GridTokenX API",
        version = "1.0.0",
        description = "GridTokenX Energy Trading Platform API"
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management"),
        (name = "trading", description = "P2P Energy Trading"),
        (name = "meters", description = "Smart Meter management"),
        (name = "dev", description = "Developer tools")
    )
)]
struct ApiDoc;

/// Build the application router with both v1 and legacy routes.
pub fn build_router(app_state: AppState) -> Router {
    // Health check routes (always at root)
    let health = Router::new()
        .route("/health", get(health_check))
        .route("/api/health", get(health_check));

    // WebSocket endpoint
    let ws = Router::new()
        .route("/ws", get(websocket_handler));

    // Swagger UI
    let swagger = SwaggerUi::new("/api/docs")
        .url("/api/docs/openapi.json", ApiDoc::openapi());

    // =========================================================================
    // V1 RESTful API Routes (New)
    // =========================================================================
    let trading_routes = v1_trading_routes()
        .layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware));

    let futures_routes = crate::handlers::futures::routes()
        .layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware));

    let analytics_routes = crate::handlers::analytics::routes()
        .layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware));

    let v1_api = Router::new()
        .nest("/auth", v1_auth_routes())       // POST /api/v1/auth/token, GET /api/v1/auth/verify
        .nest("/users", v1_users_routes())     // POST /api/v1/users, GET /api/v1/users/me
        .nest("/meters", v1_meters_routes())   // POST /api/v1/meters, PATCH /api/v1/meters/{serial}
        .nest("/wallets", v1_wallets_routes()) // GET /api/v1/wallets/{address}/balance
        .nest("/status", v1_status_routes())   // GET /api/v1/status
        .nest("/trading", trading_routes)      // POST /api/v1/trading/orders
        .nest("/futures", futures_routes)      // /api/v1/futures
        .nest("/analytics", analytics_routes)  // /api/v1/analytics
        .nest("/dev", dev::dev_routes());      // POST /api/v1/dev/faucet

    // =========================================================================
    // Legacy Routes (Backward Compatibility - Deprecated)
    // =========================================================================
    let legacy_meters = meter_routes();
    let legacy_auth = auth_routes();
    let legacy_tokens = token_routes();
    let legacy_user = user_meter_routes();
    let legacy_meter_info = meter_info_routes();

    health
        .merge(ws)
        .merge(swagger)  // Swagger UI at /api/docs
        // V1 API
        .nest("/api/v1", v1_api)
        // Legacy routes (deprecated)
        .nest("/api/meters", legacy_meters)
        .nest("/api/meters", legacy_meter_info)
        .nest("/api/auth", legacy_auth)
        .nest("/api/tokens", legacy_tokens)
        .nest("/api/user", legacy_user)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(metrics_middleware))
                .layer(middleware::from_fn(active_requests_middleware))
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::with_status_code(
                    axum::http::StatusCode::REQUEST_TIMEOUT,
                    std::time::Duration::from_secs(900),
                ))
                .layer(CorsLayer::permissive()),
        )
        .with_state(app_state)
}

/// Simple health check endpoint
async fn health_check() -> &'static str {
    "OK - Gateway Running (Real Data Mode)"
}

/// WebSocket handler for real-time updates
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(websocket_service): State<WebSocketService>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        websocket_service.register_client(socket).await;
    })
}
