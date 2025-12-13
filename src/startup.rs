//! Application startup and initialization logic - Minimal build
//!
//! Only initializes essential services for Simulator → Gateway → Anchor testing.

use anyhow::Result;
use tracing::{error, info, warn};

use crate::app_state::AppState;
use crate::auth::jwt::{ApiKeyService, JwtService};
use crate::config::Config;
use crate::database;
use crate::services;

/// Initialize minimal application services and create the AppState.
pub async fn initialize_app(config: &Config) -> Result<AppState> {
    info!("🚀 Starting minimal Gateway for Simulator → Anchor testing");

    // Setup database connections
    let db_pool = database::setup_database(&config.database_url).await?;
    info!("✅ PostgreSQL connection established");

    // Run database migrations
    database::run_migrations(&db_pool).await?;
    info!("✅ Database migrations completed");

    // Setup Redis connection
    let redis_client = setup_redis(config).await?;
    info!("✅ Redis connection established");

    // Initialize authentication services
    let jwt_service = JwtService::new()?;
    let api_key_service = ApiKeyService::new()?;
    info!("✅ JWT and API key services initialized");

    // Initialize email service (optional)
    let email_service = initialize_email_service(config);

    // Initialize auth service
    let auth = services::AuthService::new(
        db_pool.clone(),
        config.clone(),
        email_service.clone(),
        jwt_service.clone(),
    );
    info!("✅ Auth service initialized");

    // Initialize blockchain service
    let blockchain_service = services::BlockchainService::new(
        config.solana_rpc_url.clone(),
        "localnet".to_string(),
        config.solana_programs.clone(),
    )?;
    info!("✅ Blockchain service initialized (RPC: {})", config.solana_rpc_url);

    // Initialize wallet service
    let wallet_service = if let Ok(path) = std::env::var("AUTHORITY_WALLET_PATH") {
        info!("Loading authority wallet from: {}", path);
        services::WalletService::with_path(&config.solana_rpc_url, path)
    } else {
        services::WalletService::new(&config.solana_rpc_url)
    };
    initialize_wallet(&wallet_service).await;

    // Initialize WebSocket service
    let websocket_service = services::WebSocketService::new();
    info!("✅ WebSocket service initialized");

    // Initialize cache service
    let cache_service = services::CacheService::new(&config.redis_url).await?;
    info!("✅ Cache service initialized");

    // Initialize health checker
    let health_checker = services::HealthChecker::new(
        db_pool.clone(),
        redis_client.clone(),
        config.solana_rpc_url.clone(),
    );
    info!("✅ Health checker initialized");

    // Create minimal application state
    let app_state = AppState {
        db: db_pool,
        redis: redis_client,
        config: config.clone(),
        jwt_service,
        api_key_service,
        auth,
        email_service,
        blockchain_service,
        wallet_service,
        websocket_service,
        cache_service,
        health_checker,
    };

    info!("✅ Minimal AppState created successfully");
    info!("📊 Ready to receive meter readings at /api/meters/submit-reading");

    Ok(app_state)
}

/// Setup Redis connection.
async fn setup_redis(config: &Config) -> Result<redis::Client> {
    let redis_client = redis::Client::open(config.redis_url.as_str())?;

    // Test Redis connection
    match redis_client.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            use redis::AsyncCommands;
            match conn.get::<&str, Option<String>>("health_check").await {
                Ok(_) => info!("Redis connection verified"),
                Err(e) => {
                    error!("Redis connection test failed: {}", e);
                    return Err(anyhow::anyhow!("Redis connection test failed: {}", e));
                }
            }
        }
        Err(e) => {
            error!("Failed to establish Redis connection: {}", e);
            return Err(anyhow::anyhow!("Redis connection failed: {}", e));
        }
    }

    Ok(redis_client)
}

/// Initialize email service (optional).
fn initialize_email_service(config: &Config) -> Option<services::EmailService> {
    match services::EmailService::new(&config.email) {
        Ok(service) => {
            info!("Email service initialized");
            Some(service)
        }
        Err(e) => {
            warn!("Email service disabled: {}", e);
            None
        }
    }
}

/// Initialize wallet service and load authority wallet.
async fn initialize_wallet(wallet_service: &services::WalletService) {
    match wallet_service.initialize_authority().await {
        Ok(()) => {
            if let Ok(pubkey) = wallet_service.get_authority_pubkey_string().await {
                info!("🔑 Authority wallet loaded: {}", pubkey);
            }
        }
        Err(e) => {
            warn!(
                "⚠️ Failed to load authority wallet: {}. Token minting will not be available.",
                e
            );
        }
    }
}

/// Spawn background tasks (minimal version - no background tasks).
pub async fn spawn_background_tasks(_app_state: &AppState, _config: &Config) {
    info!("📌 Background tasks disabled in minimal build");
}

/// Wait for shutdown signal.
pub async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to install Ctrl+C handler: {}", e);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => {
                error!("Failed to install signal handler: {}", e);
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal, shutting down gracefully");
        },
        _ = terminate => {
            info!("Received SIGTERM signal, shutting down gracefully");
        },
    }
}
