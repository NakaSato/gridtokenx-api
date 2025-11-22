use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use gridtokenx_apigateway::{
    AppState,
    config::Config,
    handlers::meters::SubmitReadingRequest,
    models::meter::{MeterReading, SubmitMeterReadingRequest},
    services::{
        blockchain_service::BlockchainService, meter_service::MeterService,
        websocket_service::WebSocketService,
    },
};

// Integration test for the complete smart meter flow:
// 1. User submits a meter reading
// 2. Meter polling service processes it
// 3. Tokens are minted
// 4. WebSocket events are broadcast
#[tokio::test]
async fn test_complete_meter_reading_flow() {
    // Setup test database
    let config = Config::from_env().unwrap_or_else(|_| {
        Config {
            environment: "test".to_string(),
            port: 8080,
            database_url:
                "postgresql://gridtokenx_user:gridtokenx_password@localhost:5432/gridtokenx_test"
                    .to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            jwt_secret: "test-secret".to_string(),
            jwt_expiration: 86400,
            solana_rpc_url: "https://api.devnet.solana.com".to_string(),
            solana_ws_url: "wss://api.devnet.solana.com".to_string(),
            energy_token_mint: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            engineering_api_key: "test-key".to_string(),
            max_connections: 10,
            redis_pool_size: 5,
            request_timeout: 30,
            rate_limit_window: 60,
            log_level: "info".to_string(),
            audit_log_enabled: true,
            test_mode: true,
            email: gridtokenx_apigateway::config::EmailConfig::default(),
            tokenization: gridtokenx_apigateway::config::TokenizationConfig {
                kwh_to_token_ratio: 1.0,
                decimals: 9,
                max_reading_kwh: 100.0,
                reading_max_age_days: 7,
                auto_mint_enabled: true,
                polling_interval_secs: 10, // Short for testing
                batch_size: 5,
                max_retry_attempts: 3,
                initial_retry_delay_secs: 60,
                retry_backoff_multiplier: 2.0,
                max_retry_delay_secs: 300,
                transaction_timeout_secs: 60,
                max_transactions_per_batch: 5,
            },
        }
    });

    // Initialize services
    let db_pool = gridtokenx_apigateway::database::setup_database(&config.database_url)
        .await
        .expect("Failed to setup test database");

    let blockchain_service = Arc::new(
        BlockchainService::new(config.solana_rpc_url.clone(), "devnet".to_string())
            .expect("Failed to initialize blockchain service"),
    );

    let websocket_service = Arc::new(WebSocketService::new());
    let meter_service = Arc::new(MeterService::new(db_pool.clone()));

    // Create test user with wallet
    let user_id = Uuid::new_v4();
    let wallet_address = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, wallet_address, role, email_verified)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_id,
        "test@example.com",
        "hashed_password",
        wallet_address,
        "prosumer",
        true
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create test user");

    // Create app state with all services
    let app_state = AppState {
        db: db_pool.clone(),
        timescale_db: None,
        redis: redis::Client::open(config.redis_url.as_str()).unwrap(),
        config: config.clone(),
        jwt_service: gridtokenx_apigateway::auth::jwt::JwtService::new().unwrap(),
        api_key_service: gridtokenx_apigateway::auth::jwt::ApiKeyService::new().unwrap(),
        email_service: None,
        blockchain_service: blockchain_service.clone(),
        wallet_service: gridtokenx_apigateway::services::WalletService::new(&config.solana_rpc_url),
        meter_service: meter_service.clone(),
        meter_verification_service: gridtokenx_apigateway::services::MeterVerificationService::new(
            db_pool.clone(),
        ),
        erc_service: gridtokenx_apigateway::services::ErcService::new(
            db_pool.clone(),
            blockchain_service.clone(),
        ),
        order_matching_engine: gridtokenx_apigateway::services::OrderMatchingEngine::new(
            db_pool.clone(),
        ),
        market_clearing_engine: gridtokenx_apigateway::services::MarketClearingEngine::new(
            db_pool.clone(),
            redis::Client::open(config.redis_url.as_str()).unwrap(),
        ),
        market_clearing_service: gridtokenx_apigateway::services::MarketClearingService::new(
            db_pool.clone(),
        ),
        settlement_service: gridtokenx_apigateway::services::SettlementService::new(
            db_pool.clone(),
            blockchain_service.clone(),
        ),
        websocket_service: websocket_service.clone(),
        health_checker: gridtokenx_apigateway::services::HealthChecker::new(
            db_pool.clone(),
            redis::Client::open(config.redis_url.as_str()).unwrap(),
            config.solana_rpc_url.clone(),
        ),
        audit_logger: gridtokenx_apigateway::services::AuditLogger::new(db_pool.clone()),
        cache_service: gridtokenx_apigateway::services::CacheService::new(&config.redis_url)
            .await
            .unwrap(),
        meter_polling_service: gridtokenx_apigateway::services::MeterPollingService::new(
            db_pool.clone(),
            blockchain_service.clone(),
            meter_service.clone(),
            websocket_service.clone(),
            config.tokenization.clone(),
        ),
    };

    // 1. Submit a meter reading through the handler
    let reading_request = SubmitReadingRequest {
        kwh_amount: "10.5".parse().unwrap(),
        reading_timestamp: chrono::Utc::now(),
        meter_signature: Some("test_signature".to_string()),
        meter_id: None, // Using legacy flow for simplicity
    };

    // Create an authenticated user context
    let user_claims = gridtokenx_apigateway::auth::Claims {
        sub: user_id,
        email: "test@example.com".to_string(),
        role: "prosumer".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    // Mock the authentication middleware by directly creating the user context
    let auth_user = gridtokenx_apigateway::auth::middleware::AuthenticatedUser(user_claims);

    // Submit the reading
    let response = gridtokenx_apigateway::handlers::meters::submit_reading(
        axum::extract::State(app_state),
        auth_user,
        axum::Json(reading_request),
    )
    .await
    .expect("Failed to submit meter reading");

    let reading_response = response.0;
    let reading_id = reading_response.id;

    // Verify reading was stored correctly
    let stored_reading = meter_service
        .get_reading_by_id(reading_id)
        .await
        .expect("Failed to retrieve stored reading");

    assert_eq!(stored_reading.id, reading_id);
    assert_eq!(stored_reading.wallet_address, wallet_address);
    assert_eq!(stored_reading.kwh_amount.unwrap(), "10.5".parse().unwrap());
    assert!(!stored_reading.minted.unwrap_or(true));
    assert!(stored_reading.mint_tx_signature.is_none());

    // Wait a bit for the polling service to pick up the reading
    sleep(Duration::from_secs(5)).await;

    // 2. Trigger the meter polling service manually
    app_state
        .meter_polling_service
        .process_unminted_readings()
        .await
        .expect("Failed to process unminted readings");

    // Wait for blockchain transaction to complete
    sleep(Duration::from_secs(10)).await;

    // 3. Verify the reading was marked as minted
    let updated_reading = meter_service
        .get_reading_by_id(reading_id)
        .await
        .expect("Failed to retrieve updated reading");

    assert!(updated_reading.minted.unwrap_or(false));
    assert!(updated_reading.mint_tx_signature.is_some());

    // 4. Verify the user received tokens
    let user_wallet_pubkey =
        gridtokenx_apigateway::services::BlockchainService::parse_pubkey(wallet_address)
            .expect("Invalid wallet address");

    let token_balance = blockchain_service
        .get_balance(&user_wallet_pubkey)
        .await
        .expect("Failed to get token balance");

    // Should have at least 10.5 tokens (with 9 decimals)
    assert!(token_balance >= 10_500_000_000);

    // 5. Check that WebSocket events were sent
    // In a real test, we would connect to the WebSocket and listen for events
    // For this integration test, we're verifying the complete flow works end-to-end
}

// Integration test for error handling when blockchain operations fail
#[tokio::test]
async fn test_meter_reading_with_blockchain_failure() {
    // Setup similar to the first test but with a blockchain service that fails
    let config = Config::from_env().unwrap_or_else(|_| {
        Config {
            environment: "test".to_string(),
            port: 8080,
            database_url: "postgresql://test:test@localhost:5432/gridtokenx_test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            jwt_secret: "test-secret".to_string(),
            jwt_expiration: 86400,
            solana_rpc_url: "https://invalid-rpc-url.com".to_string(), // Invalid URL to cause failure
            solana_ws_url: "wss://invalid-ws-url.com".to_string(),
            energy_token_mint: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            engineering_api_key: "test-key".to_string(),
            max_connections: 10,
            redis_pool_size: 5,
            request_timeout: 30,
            rate_limit_window: 60,
            log_level: "info".to_string(),
            audit_log_enabled: true,
            test_mode: true,
            email: gridtokenx_apigateway::config::EmailConfig::default(),
            tokenization: gridtokenx_apigateway::config::TokenizationConfig::default(),
        }
    });

    // Initialize services
    let db_pool = gridtokenx_apigateway::database::setup_database(&config.database_url)
        .await
        .expect("Failed to setup test database");

    let blockchain_service = Arc::new(
        BlockchainService::new(config.solana_rpc_url.clone(), "devnet".to_string())
            .expect("Failed to initialize blockchain service"),
    );

    let websocket_service = Arc::new(WebSocketService::new());
    let meter_service = Arc::new(MeterService::new(db_pool.clone()));

    // Create test user
    let user_id = Uuid::new_v4();
    let wallet_address = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, wallet_address, role, email_verified)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_id,
        "test@example.com",
        "hashed_password",
        wallet_address,
        "prosumer",
        true
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create test user");

    // Create app state
    let app_state = AppState {
        db: db_pool.clone(),
        timescale_db: None,
        redis: redis::Client::open(config.redis_url.as_str()).unwrap(),
        config: config.clone(),
        jwt_service: gridtokenx_apigateway::auth::jwt::JwtService::new().unwrap(),
        api_key_service: gridtokenx_apigateway::auth::jwt::ApiKeyService::new().unwrap(),
        email_service: None,
        blockchain_service: blockchain_service.clone(),
        wallet_service: gridtokenx_apigateway::services::WalletService::new(&config.solana_rpc_url),
        meter_service: meter_service.clone(),
        meter_verification_service: gridtokenx_apigateway::services::MeterVerificationService::new(
            db_pool.clone(),
        ),
        erc_service: gridtokenx_apigateway::services::ErcService::new(
            db_pool.clone(),
            blockchain_service.clone(),
        ),
        order_matching_engine: gridtokenx_apigateway::services::OrderMatchingEngine::new(
            db_pool.clone(),
        ),
        market_clearing_engine: gridtokenx_apigateway::services::MarketClearingEngine::new(
            db_pool.clone(),
            redis::Client::open(config.redis_url.as_str()).unwrap(),
        ),
        market_clearing_service: gridtokenx_apigateway::services::MarketClearingService::new(
            db_pool.clone(),
        ),
        settlement_service: gridtokenx_apigateway::services::SettlementService::new(
            db_pool.clone(),
            blockchain_service.clone(),
        ),
        websocket_service: websocket_service.clone(),
        health_checker: gridtokenx_apigateway::services::HealthChecker::new(
            db_pool.clone(),
            redis::Client::open(config.redis_url.as_str()).unwrap(),
            config.solana_rpc_url.clone(),
        ),
        audit_logger: gridtokenx_apigateway::services::AuditLogger::new(db_pool.clone()),
        cache_service: gridtokenx_apigateway::services::CacheService::new(&config.redis_url)
            .await
            .unwrap(),
        meter_polling_service: gridtokenx_apigateway::services::MeterPollingService::new(
            db_pool.clone(),
            blockchain_service.clone(),
            meter_service.clone(),
            websocket_service.clone(),
            config.tokenization.clone(),
        ),
    };

    // Submit a meter reading
    let reading_request = SubmitReadingRequest {
        kwh_amount: "5.0".parse().unwrap(),
        reading_timestamp: chrono::Utc::now(),
        meter_signature: Some("test_signature".to_string()),
        meter_id: None,
    };

    let user_claims = gridtokenx_apigateway::auth::Claims {
        sub: user_id,
        email: "test@example.com".to_string(),
        role: "prosumer".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let auth_user = gridtokenx_apigateway::auth::middleware::AuthenticatedUser(user_claims);

    let response = gridtokenx_apigateway::handlers::meters::submit_reading(
        axum::extract::State(app_state.clone()),
        auth_user,
        axum::Json(reading_request),
    )
    .await
    .expect("Failed to submit meter reading");

    let reading_id = response.0.id;

    // Process unminted readings
    app_state
        .meter_polling_service
        .process_unminted_readings()
        .await
        .expect("Failed to process unminted readings");

    // Wait for processing to complete
    sleep(Duration::from_secs(5)).await;

    // Check that the reading is still not minted
    let updated_reading = meter_service
        .get_reading_by_id(reading_id)
        .await
        .expect("Failed to retrieve updated reading");

    assert!(!updated_reading.minted.unwrap_or(true));
    assert!(updated_reading.mint_tx_signature.is_none());

    // Check that the reading was added to the retry queue
    let retry_queue_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM minting_retry_queue WHERE reading_id = $1",
        reading_id
    )
    .fetch_one(&db_pool)
    .await
    .expect("Failed to check retry queue")
    .count
    .unwrap_or(0);

    assert_eq!(retry_queue_count, 1, "Reading should be in the retry queue");
}

// Integration test for batch processing of multiple readings
#[tokio::test]
async fn test_batch_processing_of_meter_readings() {
    // Setup similar to previous tests
    let config = Config::from_env().unwrap_or_else(|_| {
        Config {
            environment: "test".to_string(),
            port: 8080,
            database_url: "postgresql://test:test@localhost:5432/gridtokenx_test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            jwt_secret: "test-secret".to_string(),
            jwt_expiration: 86400,
            solana_rpc_url: "https://api.devnet.solana.com".to_string(),
            solana_ws_url: "wss://api.devnet.solana.com".to_string(),
            energy_token_mint: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            engineering_api_key: "test-key".to_string(),
            max_connections: 10,
            redis_pool_size: 5,
            request_timeout: 30,
            rate_limit_window: 60,
            log_level: "info".to_string(),
            audit_log_enabled: true,
            test_mode: true,
            email: gridtokenx_apigateway::config::EmailConfig::default(),
            tokenization: gridtokenx_apigateway::config::TokenizationConfig {
                kwh_to_token_ratio: 1.0,
                decimals: 9,
                max_reading_kwh: 100.0,
                reading_max_age_days: 7,
                auto_mint_enabled: true,
                polling_interval_secs: 10, // Short for testing
                batch_size: 5,
                max_retry_attempts: 3,
                initial_retry_delay_secs: 60,
                retry_backoff_multiplier: 2.0,
                max_retry_delay_secs: 300,
                transaction_timeout_secs: 60,
                max_transactions_per_batch: 5,
            },
        }
    });

    // Initialize services
    let db_pool = gridtokenx_apigateway::database::setup_database(&config.database_url)
        .await
        .expect("Failed to setup test database");

    let blockchain_service = Arc::new(
        BlockchainService::new(config.solana_rpc_url.clone(), "devnet".to_string())
            .expect("Failed to initialize blockchain service"),
    );

    let websocket_service = Arc::new(WebSocketService::new());
    let meter_service = Arc::new(MeterService::new(db_pool.clone()));

    // Create test user
    let user_id = Uuid::new_v4();
    let wallet_address = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, wallet_address, role, email_verified)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_id,
        "test@example.com",
        "hashed_password",
        wallet_address,
        "prosumer",
        true
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create test user");

    // Create app state
    let app_state = AppState {
        db: db_pool.clone(),
        timescale_db: None,
        redis: redis::Client::open(config.redis_url.as_str()).unwrap(),
        config: config.clone(),
        jwt_service: gridtokenx_apigateway::auth::jwt::JwtService::new().unwrap(),
        api_key_service: gridtokenx_apigateway::auth::jwt::ApiKeyService::new().unwrap(),
        email_service: None,
        blockchain_service: blockchain_service.clone(),
        wallet_service: gridtokenx_apigateway::services::WalletService::new(&config.solana_rpc_url),
        meter_service: meter_service.clone(),
        meter_verification_service: gridtokenx_apigateway::services::MeterVerificationService::new(
            db_pool.clone(),
        ),
        erc_service: gridtokenx_apigateway::services::ErcService::new(
            db_pool.clone(),
            blockchain_service.clone(),
        ),
        order_matching_engine: gridtokenx_apigateway::services::OrderMatchingEngine::new(
            db_pool.clone(),
        ),
        market_clearing_engine: gridtokenx_apigateway::services::MarketClearingEngine::new(
            db_pool.clone(),
            redis::Client::open(config.redis_url.as_str()).unwrap(),
        ),
        market_clearing_service: gridtokenx_apigateway::services::MarketClearingService::new(
            db_pool.clone(),
        ),
        settlement_service: gridtokenx_apigateway::services::SettlementService::new(
            db_pool.clone(),
            blockchain_service.clone(),
        ),
        websocket_service: websocket_service.clone(),
        health_checker: gridtokenx_apigateway::services::HealthChecker::new(
            db_pool.clone(),
            redis::Client::open(config.redis_url.as_str()).unwrap(),
            config.solana_rpc_url.clone(),
        ),
        audit_logger: gridtokenx_apigateway::services::AuditLogger::new(db_pool.clone()),
        cache_service: gridtokenx_apigateway::services::CacheService::new(&config.redis_url)
            .await
            .unwrap(),
        meter_polling_service: gridtokenx_apigateway::services::MeterPollingService::new(
            db_pool.clone(),
            blockchain_service.clone(),
            meter_service.clone(),
            websocket_service.clone(),
            config.tokenization.clone(),
        ),
    };

    // Submit multiple meter readings
    let reading_count = 5;
    let mut reading_ids = Vec::new();

    for i in 1..=reading_count {
        let reading_request = SubmitReadingRequest {
            kwh_amount: format!("{}.0", i * 2).parse().unwrap(), // 2, 4, 6, 8, 10 kWh
            reading_timestamp: chrono::Utc::now(),
            meter_signature: Some(format!("test_signature_{}", i)),
            meter_id: None,
        };

        let user_claims = gridtokenx_apigateway::auth::Claims {
            sub: user_id,
            email: "test@example.com".to_string(),
            role: "prosumer".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        };

        let auth_user = gridtokenx_apigateway::auth::middleware::AuthenticatedUser(user_claims);

        let response = gridtokenx_apigateway::handlers::meters::submit_reading(
            axum::extract::State(app_state.clone()),
            auth_user,
            axum::Json(reading_request),
        )
        .await
        .expect("Failed to submit meter reading");

        reading_ids.push(response.0.id);
    }

    // Process unminted readings
    app_state
        .meter_polling_service
        .process_unminted_readings()
        .await
        .expect("Failed to process unminted readings");

    // Wait for blockchain transactions to complete
    sleep(Duration::from_secs(15)).await;

    // Check that all readings were minted
    for reading_id in reading_ids {
        let updated_reading = meter_service
            .get_reading_by_id(reading_id)
            .await
            .expect("Failed to retrieve updated reading");

        assert!(
            updated_reading.minted.unwrap_or(false),
            "Reading {} should be minted",
            reading_id
        );
        assert!(
            updated_reading.mint_tx_signature.is_some(),
            "Reading {} should have a transaction signature",
            reading_id
        );
    }

    // Check that the user received the correct total amount of tokens
    let user_wallet_pubkey =
        gridtokenx_apigateway::services::BlockchainService::parse_pubkey(wallet_address)
            .expect("Invalid wallet address");

    let token_balance = blockchain_service
        .get_balance(&user_wallet_pubkey)
        .await
        .expect("Failed to get token balance");

    // Should have at least 30 tokens (2+4+6+8+10)
    assert!(token_balance >= 30_000_000_000);
}
