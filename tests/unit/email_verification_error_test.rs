use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use axum_test::TestServer;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use api_gateway::{AppState, error::ApiError};
use api_gateway::handlers::user_management;
use api_gateway::auth::Claims;
use sqlx::PgPool;

async fn create_test_app(db_pool: PgPool) -> Router {
    let config = api_gateway::config::Config::default();
    let state = AppState {
        db: db_pool,
        config,
        jwt_service: api_gateway::auth::jwt::JwtService::new("test_secret"),
        email_service: None,
        api_key_service: api_gateway::auth::ApiKeyService::new(),
        blockchain_service: api_gateway::services::blockchain_service::BlockchainService::new("http://localhost:8899"),
        cache_service: None,
    };

    Router::new()
        .route("/api/user/wallet", axum::routing::post(user_management::update_wallet_address))
        .with_state(state)
}

async fn create_unverified_user(db_pool: &PgPool) -> Result<Uuid> {
    let user_id = Uuid::new_v4();
    
    // Create user with email_verified = false
    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, role, 
                           first_name, last_name, is_active, email_verified, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, true, false, NOW(), NOW())",
        user_id,
        "testuser_unverified",
        "unverified@example.com",
        "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4KIVHSZMRK", // "password123"
        "user",
        "Test",
        "User"
    )
    .execute(db_pool)
    .await?;

    Ok(user_id)
}

fn create_jwt_token(user_id: Uuid) -> String {
    let claims = Claims::new(user_id, "testuser_unverified".to_string(), "user".to_string());
    
    // Simple JWT encoding for test (in real app this would use the proper service)
    serde_json::json!({
        "sub": user_id,
        "username": "testuser_unverified",
        "role": "user",
        "exp": (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp()
    }).to_string()
}

#[tokio::test]
async fn test_email_not_verified_returns_401() -> Result<()> {
    // Create test database
    let db_pool = create_test_db().await;
    
    // Create app
    let app = create_test_app(db_pool.clone()).await;
    
    // Create unverified user
    let user_id = create_unverified_user(&db_pool).await?;
    
    // Create JWT token for unverified user
    let token = create_jwt_token(user_id);
    
    // Make request to wallet endpoint
    let request = Request::builder()
        .method("POST")
        .uri("/api/user/wallet")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(json!({
            "wallet_address": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVLcdzNac"
        }).to_string()))?;

    // Send request
    let response = app.oneshot(request).await?;
    
    // Assert status code is 401 Unauthorized (not 500)
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    // Parse response body
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let error_response: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    
    // Assert error structure
    assert_eq!(error_response["error"]["code"], "AUTH_1005");
    assert_eq!(error_response["error"]["code_number"], 1005);
    assert_eq!(error_response["error"]["message"], "Email not verified");
    
    Ok(())
}

#[tokio::test] 
async fn test_verified_user_can_connect_wallet() -> Result<()> {
    // Create test database
    let db_pool = create_test_db().await;
    
    // Create app
    let app = create_test_app(db_pool.clone()).await;
    
    // Create verified user
    let user_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, role, 
                           first_name, last_name, is_active, email_verified, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, true, true, NOW(), NOW())",
        user_id,
        "testuser_verified",
        "verified@example.com", 
        "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4KIVHSZMRK",
        "user",
        "Test",
        "User"
    )
    .execute(&db_pool)
    .await?;
    
    // Create JWT token for verified user
    let token = create_jwt_token(user_id);
    
    // Make request to wallet endpoint
    let request = Request::builder()
        .method("POST")
        .uri("/api/user/wallet")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(json!({
            "wallet_address": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVLcdzNac"
        }).to_string()))?;

    // Send request
    let response = app.oneshot(request).await?;
    
    // Assert status code is 200 OK
    assert_eq!(response.status(), StatusCode::OK);
    
    Ok(())
}

async fn create_test_db() -> PgPool {
    // Use an in-memory SQLite or test PostgreSQL database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://test:test@localhost/test_gridtokenx".to_string());
    
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_api_error_email_not_verified_status_code() {
    let error = ApiError::email_not_verified();
    
    // Test that the error returns the correct status code
    let response = error.into_response();
    
    // Should return 401 Unauthorized, not 500 Internal Server Error
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    // Parse response body to verify structure
    let (parts, body) = response.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    let error_response: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    
    // Verify error structure
    assert_eq!(error_response["error"]["code"], "AUTH_1005");
    assert_eq!(error_response["error"]["code_number"], 1005);
    assert_eq!(error_response["error"]["message"], "Email not verified");
    assert!(error_response["request_id"].is_string());
    assert!(error_response["timestamp"].is_string());
}

#[tokio::test]
async fn test_error_code_email_not_verified() {
    use api_gateway::error::ErrorCode;
    
    let code = ErrorCode::EmailNotVerified;
    
    assert_eq!(code.code(), 1005);
    assert_eq!(code.message(), "Please verify your email address before proceeding");
}
