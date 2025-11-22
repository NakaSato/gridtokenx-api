use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;

use gridtokenx_apigateway::{
    AppState,
    config::{Config, TokenizationConfig},
    handlers::meters::SubmitReadingRequest,
    models::meter::{MeterReading, SubmitMeterReadingRequest},
    services::{
        blockchain_service::BlockchainService, meter_service::MeterService,
        websocket_service::WebSocketService,
    },
};

// Performance test to verify system can handle required throughput
// Target: Process 500+ meter readings per hour
#[tokio::test]
async fn test_meter_processing_throughput() {
    // Setup test configuration optimized for performance
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
            max_connections: 20,
            redis_pool_size: 10,
            request_timeout: 30,
            rate_limit_window: 60,
            log_level: "info".to_string(),
            audit_log_enabled: false, // Disable for performance test
            test_mode: true,
            email: gridtokenx_apigateway::config::EmailConfig::default(),
            tokenization: TokenizationConfig {
                kwh_to_token_ratio: 1.0,
                decimals: 9,
                max_reading_kwh: 100.0,
                reading_max_age_days: 7,
                auto_mint_enabled: true,
                polling_interval_secs: 5, // Short for testing
                batch_size: 100,          // Larger batch for performance
                max_retry_attempts: 3,
                initial_retry_delay_secs: 60,
                retry_backoff_multiplier: 2.0,
                max_retry_delay_secs: 300,
                transaction_timeout_secs: 60,
                max_transactions_per_batch: 50, // More transactions per batch
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

    // Create test users with wallets
    let user_count = 50; // Multiple users to simulate real-world scenario
    let mut user_ids = Vec::with_capacity(user_count);

    for i in 0..user_count {
        let user_id = Uuid::new_v4();
        user_ids.push(user_id);

        // Create user in database
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, wallet_address, role, email_verified)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user_id,
            format!("user{}@example.com", i),
            "hashed_password",
            format!("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM{}", i),
            "prosumer",
            true
        )
        .execute(&db_pool)
        .await
        .expect("Failed to create test user");
    }

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

    // Benchmark: Submit 500 readings (target for 1 hour)
    let readings_per_user = 10; // Total: 50 * 10 = 500 readings
    let total_readings = user_count * readings_per_user;

    let start_time = Instant::now();
    let mut reading_ids = Vec::with_capacity(total_readings);

    // Submit readings in parallel to simulate real-world usage
    let mut submission_tasks = Vec::with_capacity(user_count);

    for (user_index, user_id) in user_ids.into_iter().enumerate() {
        let user_claims = gridtokenx_apigateway::auth::Claims {
            sub: user_id,
            email: format!("user{}@example.com", user_index),
            role: "prosumer".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        };

        let auth_user = gridtokenx_apigateway::auth::middleware::AuthenticatedUser(user_claims);
        let app_state_clone = app_state.clone();

        let task = tokio::spawn(async move {
            let mut user_reading_ids = Vec::with_capacity(readings_per_user);

            for reading_index in 0..readings_per_user {
                let reading_request = SubmitReadingRequest {
                    kwh_amount: format!("{}.{}", reading_index % 10 + 1, reading_index % 10)
                        .parse()
                        .unwrap(),
                    reading_timestamp: chrono::Utc::now(),
                    meter_signature: Some(format!("signature_{}_{}", user_index, reading_index)),
                    meter_id: None,
                };

                // Submit reading
                let response = gridtokenx_apigateway::handlers::meters::submit_reading(
                    axum::extract::State(app_state_clone.clone()),
                    auth_user.clone(),
                    axum::Json(reading_request),
                )
                .await
                .expect("Failed to submit meter reading");

                user_reading_ids.push(response.0.id);

                // Small delay between submissions for realism
                sleep(Duration::from_millis(50)).await;
            }

            user_reading_ids
        });

        submission_tasks.push(task);
    }

    // Wait for all submissions to complete
    for task in submission_tasks {
        let user_reading_ids = task.await.expect("Task panicked");
        reading_ids.extend(user_reading_ids);
    }

    let submission_time = start_time.elapsed();
    println!(
        "Submitted {} readings in {:?}",
        total_readings, submission_time
    );
    println!(
        "Submission rate: {:.2} readings/second",
        total_readings as f64 / submission_time.as_secs_f64()
    );

    // Now trigger processing and measure time to completion
    let processing_start = Instant::now();

    // Process unminted readings in multiple cycles to simulate real operation
    let mut processed_count = 0;
    let max_cycles = 10; // Process for up to 10 cycles

    for cycle in 0..max_cycles {
        println!("Processing cycle {}", cycle + 1);

        // Process unminted readings
        app_state
            .meter_polling_service
            .process_unminted_readings()
            .await
            .expect("Failed to process unminted readings");

        // Wait a bit for processing to complete
        sleep(Duration::from_secs(5)).await;

        // Check how many are now minted
        let minted_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM meter_readings WHERE id = ANY($1) AND minted = TRUE",
            &reading_ids
        )
        .fetch_one(&db_pool)
        .await
        .expect("Failed to check minted readings")
        .count
        .unwrap_or(0);

        processed_count = minted_count;
        println!(
            "Cycle {}: {}/{} readings minted",
            processed_count, total_readings
        );

        // If all readings are processed, break
        if processed_count >= total_readings {
            break;
        }
    }

    let processing_time = processing_start.elapsed();
    let total_time = start_time.elapsed();

    println!(
        "Processed {}/{} readings in {:?}",
        processed_count, total_readings, processing_time
    );
    println!(
        "Total time for {} readings: {:?}",
        total_readings, total_time
    );

    // Calculate throughput
    let processing_rate = processed_count as f64 / processing_time.as_secs_f64();
    let overall_rate = total_readings as f64 / total_time.as_secs_f64();

    println!("Processing rate: {:.2} readings/second", processing_rate);
    println!("Overall rate: {:.2} readings/second", overall_rate);
    println!(
        "Estimated hourly capacity: {:.2} readings/hour",
        overall_rate * 3600.0
    );

    // Verify we meet target of 500 readings per hour
    let hourly_capacity = overall_rate * 3600.0;
    assert!(
        hourly_capacity >= 500.0,
        "System should be able to process at least 500 readings per hour, but capacity is {:.2}",
        hourly_capacity
    );

    // Verify we processed at least 80% of submitted readings
    let processed_percentage = processed_count as f64 / total_readings as f64 * 100.0;
    assert!(
        processed_percentage >= 80.0,
        "At least 80% of readings should be processed, but only {:.1}% were",
        processed_percentage
    );

    // Verify average processing time is reasonable
    let avg_processing_time = processing_time.as_secs_f64() / processed_count as f64;
    assert!(
        avg_processing_time <= 2.0, // Target: < 2 minutes per reading
        "Average processing time should be < 2 seconds per reading, but was {:.2} seconds",
        avg_processing_time
    );
}

// Performance test for WebSocket event broadcasting
#[tokio::test]
async fn test_websocket_event_throughput() {
    // Setup test configuration
    let config = TokenizationConfig {
        kwh_to_token_ratio: 1.0,
        decimals: 9,
        max_reading_kwh: 100.0,
        reading_max_age_days: 7,
        auto_mint_enabled: true,
        polling_interval_secs: 5,
        batch_size: 50,
        max_retry_attempts: 3,
        initial_retry_delay_secs: 60,
        retry_backoff_multiplier: 2.0,
        max_retry_delay_secs: 300,
        transaction_timeout_secs: 60,
        max_transactions_per_batch: 20,
    };

    // Initialize WebSocket service
    let websocket_service = Arc::new(WebSocketService::new());

    // Create test data
    let event_count = 1000;
    let user_id = Uuid::new_v4();
    let wallet_address = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
    let meter_serial = "METER-PERF-001";

    // Measure time to broadcast events
    let start_time = Instant::now();

    // Broadcast events in parallel
    let mut broadcast_tasks = Vec::with_capacity(event_count);

    for i in 0..event_count {
        let websocket_service_clone = websocket_service.clone();
        let user_id_clone = user_id;
        let wallet_address_clone = wallet_address.to_string();
        let meter_serial_clone = meter_serial.to_string();
        let kwh_amount = i as f64 % 50.0 + 1.0; // 1-50 kWh
        let tokens_minted = (kwh_amount * 1_000_000_000.0) as u64;
        let tx_signature = format!("signature_{}", i);

        let task = tokio::spawn(async move {
            websocket_service_clone
                .broadcast_tokens_minted(
                    &user_id_clone,
                    &wallet_address_clone,
                    &meter_serial_clone,
                    kwh_amount,
                    tokens_minted,
                    &tx_signature,
                )
                .await
        });

        broadcast_tasks.push(task);
    }

    // Wait for all broadcasts to complete
    for task in broadcast_tasks {
        task.await
            .expect("Broadcast task panicked")
            .expect("Failed to broadcast event");
    }

    let broadcast_time = start_time.elapsed();
    let broadcast_rate = event_count as f64 / broadcast_time.as_secs_f64();

    println!("Broadcast {} events in {:?}", event_count, broadcast_time);
    println!("Broadcast rate: {:.2} events/second", broadcast_rate);

    // Verify we meet target of < 100ms per event
    let avg_time_per_event = broadcast_time.as_millis() as f64 / event_count as f64;
    assert!(
        avg_time_per_event < 100.0,
        "Average event time should be < 100ms, but was {:.2}ms",
        avg_time_per_event
    );
}

// Performance test for batch processing optimization
#[tokio::test]
async fn test_batch_processing_optimization() {
    // Setup test configuration
    let config = TokenizationConfig {
        kwh_to_token_ratio: 1.0,
        decimals: 9,
        max_reading_kwh: 100.0,
        reading_max_age_days: 7,
        auto_mint_enabled: true,
        polling_interval_secs: 5,
        batch_size: 100, // Large batch size
        max_retry_attempts: 3,
        initial_retry_delay_secs: 60,
        retry_backoff_multiplier: 2.0,
        max_retry_delay_secs: 300,
        transaction_timeout_secs: 60,
        max_transactions_per_batch: 50, // More transactions per batch
    };

    // Initialize services
    let db_pool = gridtokenx_apigateway::database::setup_database(
        "postgresql://test:test@localhost:5432/gridtokenx_test",
    )
    .await
    .expect("Failed to setup test database");

    let blockchain_service = Arc::new(
        BlockchainService::new(
            "https://api.devnet.solana.com".to_string(),
            "devnet".to_string(),
        )
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
        redis: redis::Client::open("redis://localhost:6379").unwrap(),
        config: gridtokenx_apigateway::config::Config {
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
            max_connections: 20,
            redis_pool_size: 10,
            request_timeout: 30,
            rate_limit_window: 60,
            log_level: "info".to_string(),
            audit_log_enabled: false,
            test_mode: true,
            email: gridtokenx_apigateway::config::EmailConfig::default(),
            tokenization: config.clone(),
        },
        jwt_service: gridtokenx_apigateway::auth::jwt::JwtService::new().unwrap(),
        api_key_service: gridtokenx_apigateway::auth::jwt::ApiKeyService::new().unwrap(),
        email_service: None,
        blockchain_service: blockchain_service.clone(),
        wallet_service: gridtokenx_apigateway::services::WalletService::new(
            "https://api.devnet.solana.com",
        ),
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
            redis::Client::open("redis://localhost:6379").unwrap(),
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
            redis::Client::open("redis://localhost:6379").unwrap(),
            "https://api.devnet.solana.com".to_string(),
        ),
        audit_logger: gridtokenx_apigateway::services::AuditLogger::new(db_pool.clone()),
        cache_service: gridtokenx_apigateway::services::CacheService::new("redis://localhost:6379")
            .await
            .unwrap(),
        meter_polling_service: gridtokenx_apigateway::services::MeterPollingService::new(
            db_pool.clone(),
            blockchain_service.clone(),
            meter_service.clone(),
            websocket_service.clone(),
            config.clone(),
        ),
    };

    // Submit 100 readings for the same user (should be grouped in batches)
    let reading_count = 100;
    let mut reading_ids = Vec::with_capacity(reading_count);

    let user_claims = gridtokenx_apigateway::auth::Claims {
        sub: user_id,
        email: "test@example.com".to_string(),
        role: "prosumer".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let auth_user = gridtokenx_apigateway::auth::middleware::AuthenticatedUser(user_claims);

    for i in 0..reading_count {
        let reading_request = SubmitReadingRequest {
            kwh_amount: format!("{}.0", i % 10 + 1).parse().unwrap(), // 1-10 kWh
            reading_timestamp: chrono::Utc::now(),
            meter_signature: Some(format!("signature_{}", i)),
            meter_id: None,
        };

        let response = gridtokenx_apigateway::handlers::meters::submit_reading(
            axum::extract::State(app_state.clone()),
            auth_user.clone(),
            axum::Json(reading_request),
        )
        .await
        .expect("Failed to submit meter reading");

        reading_ids.push(response.0.id);
    }

    // Process unminted readings
    let processing_start = Instant::now();

    app_state
        .meter_polling_service
        .process_unminted_readings()
        .await
        .expect("Failed to process unminted readings");

    // Wait for blockchain transactions to complete
    sleep(Duration::from_secs(30)).await;

    let processing_time = processing_start.elapsed();

    // Check how many were minted
    let minted_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM meter_readings WHERE id = ANY($1) AND minted = TRUE",
        &reading_ids
    )
    .fetch_one(&db_pool)
    .await
    .expect("Failed to check minted readings")
    .count
    .unwrap_or(0);

    println!(
        "Batch processed {}/{} readings in {:?}",
        minted_count, reading_count, processing_time
    );
    println!(
        "Processing rate: {:.2} readings/second",
        minted_count as f64 / processing_time.as_secs_f64()
    );

    // Verify batch processing is more efficient than individual processing
    let avg_time_per_reading = processing_time.as_millis() as f64 / minted_count as f64;
    assert!(
        avg_time_per_reading < 500.0, // Target: < 500ms per reading in batch
        "Batch processing should be faster, but average time was {:.2}ms",
        avg_time_per_reading
    );

    // Verify high success rate
    let success_rate = minted_count as f64 / reading_count as f64 * 100.0;
    assert!(
        success_rate >= 90.0,
        "Batch processing should have high success rate, but only {:.1}% succeeded",
        success_rate
    );
}
