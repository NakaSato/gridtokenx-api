//! Authentication handler tests
//! Tests for user registration, login, and token management

use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use http::{Request, StatusCode, Method};
use uuid::Uuid;

use api_gateway::{
    auth::{UserClaims, UserRole},
    handlers::auth::{RegisterRequest, LoginRequest},
    models::user::User,
    create_app,
    config::Config,
    services::{AuthService, UserService},
    error::AppError,
};

/// Test database setup utility
async fn setup_test_db() -> PgPool {
    // Use test database configuration
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/test_gridtokenx".to_string());
    
    let pool = PgPool::connect(&database_url).await
        .expect("Failed to connect to test database");
    
    // Clean up any existing test data
    sqlx::query("DELETE FROM users WHERE email LIKE '%@test.com'")
        .execute(&pool)
        .await
        .expect("Failed to clean up test users");
    
    pool
}

/// Create test application state
async fn create_test_app_state() -> api_gateway::AppState {
    let db = setup_test_db().await;
    let config = Config::from_env().expect("Failed to load config");
    
    let auth_service = AuthService::new(db.clone(), &config.jwt_secret);
    let user_service = UserService::new(db.clone());
    
    // Mock other services for testing
    let blockchain_service = api_gateway::services::blockchain_service::BlockchainService::new(
        "http://localhost:8899".to_string(),
    );
    let wallet_service = api_gateway::services::wallet_service::WalletService::new();
    
    api_gateway::AppState {
        db,
        config,
        auth_service,
        user_service,
        blockchain_service,
        wallet_service,
        meter_verification_service: api_gateway::services::meter_verification_service::MeterVerificationService::new(db.clone()),
        meter_service: api_gateway::services::meter_service::MeterService::new(db.clone()),
        settlement_service: api_gateway::services::settlement_service::SettlementService::new(db.clone(), config.settlement.clone()),
        market_clearing_service: api_gateway::services::market_clearing::MarketClearingService::new(db.clone()),
        epoch_scheduler: api_gateway::services::epoch_scheduler::EpochScheduler::new(db.clone()),
        erc_service: api_gateway::services::erc_service::ErcService::new(db.clone()),
        token_service: api_gateway::services::token_service::TokenService::new(),
        audit_logger: api_gateway::services::audit_logger::AuditLogger::new(),
        websocket_service: api_gateway::services::websocket_service::WebSocketService::new(),
        priority_fee_service: api_gateway::services::priority_fee_service::PriorityFeeService::new("http://localhost:8899".to_string()),
        transaction_service: api_gateway::services::transaction_service::TransactionService::new("http://localhost:8899".to_string()),
        cache_service: api_gateway::services::cache_service_simple::SimpleCacheService::new(),
    }
}

#[cfg(test)]
mod auth_tests {
    use super::*;

    #[tokio::test]
    async fn test_user_registration_success() {
        // Setup
        let app_state = create_test_app_state().await;
        let app = create_app(app_state);
        
        let email = format!("test-{}@test.com", Uuid::new_v4());
        let request_body = RegisterRequest {
            email: email.clone(),
            password: "TestPassword123!".to_string(),
            name: "Test User".to_string(),
        };
        
        // Test successful registration
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&request_body).unwrap().into())
            .expect("Failed to build request");
        
        let response = app.oneshot(request).await.expect("Request failed");
        
        // Verify response
        assert_eq!(response.status(), StatusCode::CREATED);
        
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        
        assert!(response_body["user_id"].is_string());
        assert_eq!(response_body["email"], email);
        assert_eq!(response_body["name"], "Test User");
        assert!(response_body["email_verified"].as_bool().unwrap() == false);
    }
    
    #[tokio::test]
    async fn test_user_registration_duplicate_email() {
        // Setup
        let app_state = create_test_app_state().await;
        let app = create_app(app_state);
        
        let email = format!("test-{}@test.com", Uuid::new_v4());
        let request_body = RegisterRequest {
            email: email.clone(),
            password: "TestPassword123!".to_string(),
            name: "Test User".to_string(),
        };
        
        // Register first user
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&request_body).unwrap().into())
            .expect("Failed to build request");
        
        let response1 = app.clone().oneshot(request).await.expect("Request failed");
        assert_eq!(response1.status(), StatusCode::CREATED);
        
        // Try to register same email again
        let request2 = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&request_body).unwrap().into())
            .expect("Failed to build request");
        
        let response2 = app.oneshot(request2).await.expect("Request failed");
        assert_eq!(response2.status(), StatusCode::CONFLICT);
    }
    
    #[tokio::test]
    async fn test_user_registration_invalid_email() {
        // Setup
        let app_state = create_test_app_state().await;
        let app = create_app(app_state);
        
        let request_body = RegisterRequest {
            email: "invalid-email".to_string(),
            password: "TestPassword123!".to_string(),
            name: "Test User".to_string(),
        };
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&request_body).unwrap().into())
            .expect("Failed to build request");
        
        let response = app.oneshot(request).await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    
    #[tokio::test]
    async fn test_user_registration_weak_password() {
        // Setup
        let app_state = create_test_app_state().await;
        let app = create_app(app_state);
        
        let email = format!("test-{}@test.com", Uuid::new_v4());
        let request_body = RegisterRequest {
            email: email.clone(),
            password: "123".to_string(), // Too short and weak
            name: "Test User".to_string(),
        };
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&request_body).unwrap().into())
            .expect("Failed to build request");
        
        let response = app.oneshot(request).await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    
    #[tokio::test]
    async fn test_user_login_success() {
        // Setup
        let app_state = create_test_app_state().await;
        let app = create_app(app_state);
        
        let email = format!("test-{}@test.com", Uuid::new_v4());
        let password = "TestPassword123!";
        
        // Register user first
        let register_request = RegisterRequest {
            email: email.clone(),
            password: password.to_string(),
            name: "Test User".to_string(),
        };
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&register_request).unwrap().into())
            .expect("Failed to build request");
        
        let register_response = app.clone().oneshot(request).await.expect("Request failed");
        assert_eq!(register_response.status(), StatusCode::CREATED);
        
        // Now test login
        let login_request = LoginRequest {
            email: email.clone(),
            password: password.to_string(),
        };
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&login_request).unwrap().into())
            .expect("Failed to build request");
        
        let response = app.oneshot(request).await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);
        
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        
        assert!(response_body["access_token"].is_string());
        assert!(response_body["refresh_token"].is_string());
        assert!(response_body["expires_in"].is_number());
        assert_eq!(response_body["token_type"], "Bearer");
        assert_eq!(response_body["user"]["email"], email);
    }
    
    #[tokio::test]
    async fn test_user_login_invalid_credentials() {
        // Setup
        let app_state = create_test_app_state().await;
        let app = create_app(app_state);
        
        let login_request = LoginRequest {
            email: "nonexistent@test.com".to_string(),
            password: "WrongPassword123!".to_string(),
        };
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&login_request).unwrap().into())
            .expect("Failed to build request");
        
        let response = app.oneshot(request).await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    
    #[tokio::test]
    async fn test_user_login_missing_fields() {
        // Setup
        let app_state = create_test_app_state().await;
        let app = create_app(app_state);
        
        // Missing password
        let mut login_request = serde_json::Map::new();
        login_request.insert("email".to_string(), serde_json::Value::String("test@test.com".to_string()));
        // password field missing
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&login_request).unwrap().into())
            .expect("Failed to build request");
        
        let response = app.oneshot(request).await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    
    #[tokio::test]
    async fn test_protected_endpoint_without_token() {
        // Setup
        let app_state = create_test_app_state().await;
        let app = create_app(app_state);
        
        // Try to access protected endpoint without token
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/user/profile")
            .body(hyper::Body::empty())
            .expect("Failed to build request");
        
        let response = app.oneshot(request).await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    
    #[tokio::test]
    async fn test_protected_endpoint_with_invalid_token() {
        // Setup
        let app_state = create_test_app_state().await;
        let app = create_app(app_state);
        
        // Try to access protected endpoint with invalid token
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/user/profile")
            .header("authorization", "Bearer invalid.token.here")
            .body(hyper::Body::empty())
            .expect("Failed to build request");
        
        let response = app.oneshot(request).await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    
    #[tokio::test]
    async fn test_protected_endpoint_with_valid_token() {
        // Setup
        let app_state = create_test_app_state().await;
        let app = create_app(app_state);
        
        let email = format!("test-{}@test.com", Uuid::new_v4());
        let password = "TestPassword123!";
        
        // Register and login to get token
        let register_request = RegisterRequest {
            email: email.clone(),
            password: password.to_string(),
            name: "Test User".to_string(),
        };
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&register_request).unwrap().into())
            .expect("Failed to build request");
        
        let register_response = app.clone().oneshot(request).await.expect("Request failed");
        assert_eq!(register_response.status(), StatusCode::CREATED);
        
        let login_request = LoginRequest {
            email: email.clone(),
            password: password.to_string(),
        };
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&login_request).unwrap().into())
            .expect("Failed to build request");
        
        let login_response = app.clone().oneshot(request).await.expect("Request failed");
        assert_eq!(login_response.status(), StatusCode::OK);
        
        let body_bytes = hyper::body::to_bytes(login_response.into_body()).await.unwrap();
        let login_response_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        
        let token = login_response_body["access_token"].as_str().unwrap();
        
        // Now access protected endpoint with valid token
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/user/profile")
            .header("authorization", format!("Bearer {}", token))
            .body(hyper::Body::empty())
            .expect("Failed to build request");
        
        let response = app.oneshot(request).await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[cfg(test)]
mod token_validation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_jwt_token_generation() {
        let config = Config::from_env().expect("Failed to load config");
        let auth_service = AuthService::new(setup_test_db().await, &config.jwt_secret);
        
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let role = UserRole::User;
        
        let token = auth_service.generate_token(&user_id, &email, role)
            .expect("Failed to generate token");
        
        assert!(!token.is_empty());
        assert!(token.contains('.'));
    }
    
    #[tokio::test]
    async fn test_jwt_token_validation() {
        let config = Config::from_env().expect("Failed to load config");
        let auth_service = AuthService::new(setup_test_db().await, &config.jwt_secret);
        
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let role = UserRole::User;
        
        let token = auth_service.generate_token(&user_id, &email, role)
            .expect("Failed to generate token");
        
        let claims = auth_service.validate_token(&token)
            .expect("Failed to validate token");
        
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.role, role);
    }
    
    #[tokio::test]
    async fn test_jwt_token_invalid_signature() {
        let config = Config::from_env().expect("Failed to load config");
        let auth_service = AuthService::new(setup_test_db().await, &config.jwt_secret);
        
        let invalid_token = "invalid.token.signature";
        
        let result = auth_service.validate_token(invalid_token);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_jwt_token_expired() {
        let config = Config::from_env().expect("Failed to load config");
        
        // Create auth service with very short expiration for testing
        let mut short_config = config.clone();
        short_config.jwt_expiration = 1; // 1 second
        
        let auth_service = AuthService::new(setup_test_db().await, &short_config.jwt_secret);
        
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let role = UserRole::User;
        
        let token = auth_service.generate_token(&user_id, &email, role)
            .expect("Failed to generate token");
        
        // Wait for token to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let result = auth_service.validate_token(&token);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod password_security_tests {
    use super::*;
    use api_gateway::auth::password::PasswordService;
    
    #[tokio::test]
    async fn test_password_hashing() {
        let password = "TestPassword123!";
        let hash = PasswordService::hash_password(password)
            .expect("Failed to hash password");
        
        assert!(!hash.is_empty());
        assert_ne!(hash, password);
        assert!(hash.starts_with("$2b$")); // bcrypt hash prefix
    }
    
    #[tokio::test]
    async fn test_password_verification() {
        let password = "TestPassword123!";
        let hash = PasswordService::hash_password(password)
            .expect("Failed to hash password");
        
        let is_valid = PasswordService::verify_password(password, &hash)
            .expect("Failed to verify password");
        
        assert!(is_valid);
    }
    
    #[tokio::test]
    async fn test_password_verification_wrong_password() {
        let password = "TestPassword123!";
        let wrong_password = "WrongPassword456!";
        let hash = PasswordService::hash_password(password)
            .expect("Failed to hash password");
        
        let is_valid = PasswordService::verify_password(wrong_password, &hash)
            .expect("Failed to verify password");
        
        assert!(!is_valid);
    }
    
    #[tokio::test]
    async fn test_password_strength_validation() {
        // Valid password
        let valid_password = "TestPassword123!";
        assert!(PasswordService::is_strong_password(valid_password));
        
        // Too short
        assert!(!PasswordService::is_strong_password("123"));
        
        // No uppercase
        assert!(!PasswordService::is_strong_password("testpassword123!"));
        
        // No lowercase
        assert!(!PasswordService::is_strong_password("TESTPASSWORD123!"));
        
        // No numbers
        assert!(!PasswordService::is_strong_password("TestPassword!"));
        
        // No special characters
        assert!(!PasswordService::is_strong_password("TestPassword123"));
    }
}
