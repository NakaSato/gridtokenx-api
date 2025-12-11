//! GridTokenX API Gateway
//!
//! Main entry point for the P2P Energy Trading System API Gateway.
//! This is a thin entry point that delegates to modular components.

use anyhow::Result;
use std::net::SocketAddr;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

// Use the library crate's public exports
use api_gateway::{
    auth,
    config::Config,
    handlers::{
        self, admin, audit, auth as auth_handlers, blockchain, blockchain_test, epochs, erc,
        governance, health, market_data, meter as meters, oracle, registry, token, trading,
        transactions, user as user_management, wallet_auth,
    },
    middleware, startup, utils,
};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file first
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting GridTokenX API Gateway");

    // Validate secrets and security configuration
    utils::validate_secrets()?;

    // Initialize Prometheus metrics exporter
    let prometheus_builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    if let Err(e) = prometheus_builder.install() {
        error!("Failed to install Prometheus exporter: {}", e);
        warn!("Continuing without metrics export");
    } else {
        info!("Prometheus metrics exporter initialized");
    }

    // Load configuration
    let config = Config::from_env()?;
    info!(
        "Loaded configuration for environment: {}",
        config.environment
    );

    // Initialize all services and create app state
    let app_state = startup::initialize_app(&config).await?;

    // Spawn background tasks
    startup::spawn_background_tasks(&app_state, &config).await;

    // Build API routes

    // Public API routes
    let public_routes = Router::new()
        // Health check routes
        .route("/health", get(health::health_check))
        .route("/metrics", get(handlers::metrics::get_prometheus_metrics))
        .route(
            "/health/metrics",
            get(handlers::metrics::get_health_with_metrics),
        )
        .route(
            "/health/event-processor",
            get(health::get_event_processor_stats),
        )
        .route(
            "/api/dashboard/metrics",
            get(handlers::dashboard::get_dashboard_metrics),
        )
        // Public API routes
        .route("/api/auth/login", post(auth_handlers::login))
        .route("/api/auth/register", post(user_management::register))
        .route(
            "/api/auth/verify-email",
            get(handlers::email_verification::verify_email),
        )
        .route(
            "/api/auth/resend-verification",
            post(handlers::email_verification::resend_verification),
        )
        .route(
            "/api/auth/wallet/login",
            post(wallet_auth::login_with_wallet),
        )
        .route(
            "/api/auth/wallet/register",
            post(wallet_auth::register_with_wallet),
        )
        // Public market endpoints
        .route("/api/market/epoch", get(epochs::get_current_epoch))
        .route("/api/market/epoch/status", get(epochs::get_epoch_status))
        .route(
            "/api/market/orderbook",
            get(handlers::trading::market_data::get_orderbook),
        )
        .route(
            "/api/market/stats",
            get(handlers::trading::market_data::get_market_stats),
        )
        // WebSocket endpoints
        .route(
            "/api/market/ws",
            get(handlers::websocket::market_websocket_handler),
        )
        .route("/ws", get(handlers::websocket::websocket_handler))
        // Swagger UI
        .merge(SwaggerUi::new("/api/docs").url(
            "/api/docs/openapi.json",
            api_gateway::openapi::ApiDoc::openapi(),
        ));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        // Protected auth routes
        .route("/api/auth/profile", get(auth_handlers::get_profile))
        .route(
            "/api/auth/profile/update",
            post(auth_handlers::update_profile),
        )
        .route("/api/auth/password", post(auth_handlers::change_password))
        // user management routes
        .nest(
            "/api/user",
            Router::new()
                .route("/wallet", post(user_management::update_wallet_address))
                .route(
                    "/wallet",
                    axum::routing::delete(user_management::remove_wallet_address),
                )
                .route("/activity", get(user_management::get_my_activity))
                // Meter registration routes
                .route("/meters", post(meters::register_meter))
                .route("/meters", get(meters::get_user_meters)), // .route(
                                                                 //     "/meters/{meter_id}",
                                                                 //     axum::routing::delete(user_management::delete_meter_handler),
                                                                 // ),
        )
        // Admin-only user management routes
        .nest(
            "/api/users",
            Router::new()
                .route("/{id}", get(auth_handlers::get_user))
                .route(
                    "/{id}",
                    axum::routing::put(user_management::admin_update_user),
                )
                .route(
                    "/{id}/deactivate",
                    post(user_management::admin_deactivate_user),
                )
                .route(
                    "/{id}/reactivate",
                    post(user_management::admin_reactivate_user),
                )
                .route("/{id}/activity", get(user_management::get_user_activity))
                .route("/", get(auth_handlers::list_users)),
        )
        // Blockchain interaction routes
        .nest(
            "/api/blockchain",
            Router::new()
                .route("/transactions", post(blockchain::submit_transaction))
                .route("/transactions", get(blockchain::get_transaction_history))
                .route(
                    "/transactions/{signature}",
                    get(blockchain::get_transaction_status),
                )
                .route("/programs/{name}", post(blockchain::interact_with_program))
                .route("/accounts/{address}", get(blockchain::get_account_info))
                .route("/network", get(blockchain::get_network_status))
                // Registry program endpoints
                .route(
                    "/users/{wallet_address}",
                    get(registry::get_blockchain_user),
                ),
        )
        // Blockchain testing routes
        .nest(
            "/api/test",
            Router::new()
                .route(
                    "/transactions",
                    post(blockchain_test::create_test_transaction),
                )
                .route(
                    "/transactions/{signature}",
                    get(blockchain_test::get_test_transaction_status),
                )
                .route("/statistics", get(blockchain_test::get_test_statistics)),
        )
        // Admin-only routes
        .nest(
            "/api/admin",
            Router::new()
                .route("/users/{id}/update-role", post(registry::update_user_role))
                // Governance admin routes
                .route(
                    "/governance/emergency-pause",
                    post(governance::emergency_pause),
                )
                .route("/governance/unpause", post(governance::emergency_unpause))
                // Token admin routes
                .route("/tokens/mint", post(token::mint_tokens))
                // AMM Routes
                .route("/swap/quote", post(handlers::swap::get_quote))
                .route("/swap/execute", post(handlers::swap::execute_swap))
                .route("/swap/pools", get(handlers::swap::list_pools))
                .route("/swap/history", get(handlers::swap::get_swap_history))
                // Transaction routes
                .nest(
                    "/api/tx",
                    Router::new()
                        .route("/{id}/status", get(transactions::get_transaction_status))
                        .route("/user", get(transactions::get_user_transactions))
                        .route("/history", get(transactions::get_transaction_history))
                        .route("/stats", get(transactions::get_transaction_stats))
                        .route("/{id}/retry", post(transactions::retry_transaction)),
                )
                // Trading admin routes
                .route(
                    "/trading/match-orders",
                    post(trading::match_blockchain_orders),
                )
                // Market admin routes
                .route("/market/health", get(admin::get_market_health))
                .route("/market/analytics", get(admin::get_trading_analytics))
                .route("/market/control", post(admin::market_control))
                // Event Processor routes
                .route(
                    "/event-processor/replay",
                    post(admin::trigger_event_replay).get(admin::get_replay_status),
                )
                // Audit log routes
                .route("/audit/user/{user_id}", get(audit::get_user_audit_logs))
                .route(
                    "/audit/type/{event_type}",
                    get(audit::get_audit_logs_by_type),
                )
                .route("/audit/security", get(audit::get_security_events))
                // Epoch management
                .route("/epochs", get(epochs::list_all_epochs))
                .route("/epochs/{epoch_id}/stats", get(epochs::get_epoch_stats))
                .route(
                    "/epochs/{epoch_id}/trigger",
                    post(epochs::trigger_manual_clearing),
                ),
        )
        // Oracle routes
        .nest(
            "/api/oracle",
            Router::new()
                .route("/prices", post(oracle::submit_price))
                .route("/prices/current", get(oracle::get_current_prices))
                .route("/data", get(oracle::get_oracle_data)),
        )
        // Governance routes
        .nest(
            "/api/governance",
            Router::new().route("/status", get(governance::get_governance_status)),
        )
        // P2P Energy Trading routes (authenticated users) - moved to /api/market-data to avoid conflicts
        .nest(
            "/api/market-data",
            Router::new()
                .route("/depth", get(handlers::market_data::get_order_book_depth))
                .route(
                    "/depth-chart",
                    get(handlers::market_data::get_market_depth_chart),
                )
                .route(
                    "/clearing-price",
                    get(handlers::market_data::get_clearing_price),
                )
                .route(
                    "/trades/my-history",
                    get(handlers::market_data::get_my_trade_history),
                ),
        )
        // Simplified Energy Trading routes
        .nest(
            "/api/trading",
            Router::new()
                .route("/orders", post(handlers::trading::orders::create_order))
                .route("/orders", get(handlers::trading::orders::get_user_orders))
                .route(
                    "/orders/{id}",
                    axum::routing::delete(handlers::trading::orders::cancel_order),
                )
                .route(
                    "/orders/{id}",
                    axum::routing::put(handlers::trading::orders::update_order),
                ),
        )
        // Analytics routes
        .route(
            "/api/analytics/market",
            get(handlers::analytics::get_market_analytics),
        )
        .route(
            "/api/analytics/my-stats",
            get(handlers::analytics::get_user_trading_stats),
        )
        // Token routes
        .nest(
            "/api/tokens",
            Router::new()
                .route("/balance/{wallet_address}", get(token::get_token_balance))
                .route("/info", get(token::get_token_info))
                .route("/mint-from-reading", post(token::mint_from_reading)),
        )
        // Energy meter routes - Phase 4
        .nest(
            "/api/meters",
            Router::new()
                .route("/verify", post(handlers::meter::verify_meter_handler))
                .route(
                    "/registry/meters",
                    get(handlers::meter::get_registered_meters_handler),
                )
                .route("/submit-reading", post(meters::submit_reading))
                .route("/my-readings", get(meters::get_my_readings))
                .route(
                    "/readings/{wallet_address}",
                    get(meters::get_readings_by_wallet),
                )
                .route("/stats", get(meters::get_user_stats)),
        )
        // Admin meter routes - Phase 4
        .nest(
            "/api/admin/meters",
            Router::new()
                .route("/unminted", get(meters::get_unminted_readings))
                .route("/mint-from-reading", post(meters::mint_from_reading)),
        )
        // Energy Renewable Certificate (ERC) routes - Phase 4
        .nest(
            "/api/erc",
            Router::new()
                .route("/issue", post(erc::issue_certificate))
                .route("/my-certificates", get(erc::get_my_certificates))
                .route("/my-stats", get(erc::get_my_certificate_stats))
                .route("/{certificate_id}", get(erc::get_certificate))
                .route("/{certificate_id}/retire", post(erc::retire_certificate))
                .route(
                    "/wallet/{wallet_address}",
                    get(erc::get_certificates_by_wallet),
                ),
        )
        .layer(from_fn_with_state(
            app_state.clone(),
            auth::middleware::auth_middleware,
        ))
        .layer(axum::middleware::from_fn(
            middleware::auth_logger_middleware,
        ));

    // Combine all routes
    let app = public_routes
        .merge(protected_routes)
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(
                    middleware::json_validation_middleware,
                ))
                .layer(axum::middleware::from_fn(middleware::add_security_headers))
                .layer(axum::middleware::from_fn(middleware::metrics_middleware))
                .layer(axum::middleware::from_fn(
                    middleware::active_requests_middleware,
                ))
                .layer(axum::middleware::from_fn(
                    middleware::request_logger_middleware,
                ))
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::with_status_code(
                    axum::http::StatusCode::REQUEST_TIMEOUT,
                    std::time::Duration::from_secs(30),
                ))
                .layer(CorsLayer::permissive()),
        )
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting API Gateway server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Setup graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(startup::shutdown_signal())
        .await?;

    Ok(())
}
