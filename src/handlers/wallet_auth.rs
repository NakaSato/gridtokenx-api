use axum::{
    extract::{State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use solana_sdk::{signature::Signer, pubkey::Pubkey};
use std::str::FromStr;
use crate::services::wallet_service::WalletService;

use crate::auth::{Claims, SecureUserInfo};
use crate::auth::password::PasswordService;
use crate::error::{ApiError, Result};
use crate::AppState;

/// Enhanced registration request with wallet creation option
#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct WalletRegistrationRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    
    #[validate(length(min = 1, max = 20))]
    pub role: String,
    
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    
    /// Create a new Solana wallet for this user
    pub create_wallet: Option<bool>,
    
    /// Amount of SOL to airdrop (development only)
    pub airdrop_amount: Option<f64>,
    
    /// Optional manual wallet address (if not creating new one)
    #[validate(length(min = 32, max = 44))]
    pub wallet_address: Option<String>,
}

/// Response with wallet information for development
#[derive(Debug, Serialize, ToSchema)]
pub struct WalletRegistrationResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: SecureUserInfo,
    pub wallet_info: Option<DevWalletInfo>,
}

/// Development wallet information (DO NOT USE IN PRODUCTION)
#[derive(Debug, Serialize, ToSchema)]
pub struct DevWalletInfo {
    pub address: String,
    pub balance_lamports: u64,
    pub balance_sol: f64,
    pub private_key: String, // Only for development!
    pub airdrop_signature: Option<String>,
    pub created_new: bool,
}

/// Login response with wallet information
#[derive(Debug, Serialize, ToSchema)]
pub struct WalletLoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: SecureUserInfo,
    pub wallet_info: Option<UserWalletInfo>,
}

/// User's wallet information (safe for production)
#[derive(Debug, Serialize, ToSchema)]
pub struct UserWalletInfo {
    pub address: String,
    pub balance_lamports: Option<u64>,
    pub balance_sol: Option<f64>,
}

/// Enhanced registration with automatic wallet creation
#[utoipa::path(
    post,
    path = "/api/auth/register-with-wallet",
    tag = "Authentication",
    request_body = WalletRegistrationRequest,
    responses(
        (status = 200, description = "User registered successfully with optional wallet", body = WalletRegistrationResponse),
        (status = 400, description = "Invalid registration data or user already exists"),
        (status = 500, description = "Internal server error during registration or wallet creation")
    )
)]
pub async fn register_with_wallet(
    State(state): State<AppState>,
    Json(request): Json<WalletRegistrationRequest>,
) -> Result<Json<WalletRegistrationResponse>> {
    // Validate request
    request.validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    // Validate role
    crate::auth::Role::from_str(&request.role)
        .map_err(|_| ApiError::BadRequest("Invalid role".to_string()))?;

    // Check if username already exists
    let existing_user = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE username = $1 OR email = $2"
    )
    .bind(&request.username)
    .bind(&request.email)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

    if existing_user > 0 {
        return Err(ApiError::BadRequest("Username or email already exists".to_string()));
    }

    let mut wallet_address = request.wallet_address.clone();
    let mut wallet_info = None;

    // Create wallet if requested
    if request.create_wallet.unwrap_or(false) && wallet_address.is_none() {
        let wallet_service = WalletService::new(&state.config.solana_rpc_url);
        
        // Check if Solana RPC is available
        if wallet_service.health_check().await.is_err() {
            return Err(ApiError::Internal("Solana RPC not available. Please ensure solana-test-validator is running.".to_string()));
        }

        let keypair = WalletService::create_keypair();
        let pubkey = keypair.pubkey();
        
        // Store wallet address
        wallet_address = Some(pubkey.to_string());
        
        // For development: airdrop some SOL
        let airdrop_amount = request.airdrop_amount.unwrap_or(1.0);
        let airdrop_sig = if airdrop_amount > 0.0 {
            wallet_service.request_airdrop(&pubkey, airdrop_amount).await.ok()
        } else {
            None
        };
        
        // Get balance after airdrop
        let balance_lamports = wallet_service.get_balance(&pubkey).await.unwrap_or(0);
        let balance_sol = crate::services::wallet_service::lamports_to_sol(balance_lamports);
        
        wallet_info = Some(DevWalletInfo {
            address: pubkey.to_string(),
            balance_lamports,
            balance_sol,
            private_key: bs58::encode(keypair.to_bytes()).into_string(),
            airdrop_signature: airdrop_sig.map(|s| s.to_string()),
            created_new: true,
        });
    } else if let Some(provided_address) = &wallet_address {
        // Validate provided wallet address
        if !WalletService::is_valid_address(provided_address) {
            return Err(ApiError::BadRequest("Invalid Solana wallet address format".to_string()));
        }

        // Get balance for provided address
        let wallet_service = WalletService::new(&state.config.solana_rpc_url);
        if let Ok(pubkey) = Pubkey::from_str(provided_address) {
            let balance_lamports = wallet_service.get_balance(&pubkey).await.unwrap_or(0);
            let balance_sol = crate::services::wallet_service::lamports_to_sol(balance_lamports);
            
            wallet_info = Some(DevWalletInfo {
                address: provided_address.clone(),
                balance_lamports,
                balance_sol,
                private_key: "Not available (user provided address)".to_string(),
                airdrop_signature: None,
                created_new: false,
            });
        }
    }

    // Hash password
    let password_hash = PasswordService::hash_password(&request.password)?;

    // Create user with enhanced fields
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role,
                           first_name, last_name, wallet_address, is_active, created_at, updated_at)
         VALUES ($1, $2, $3, $4, ($5)::user_role, $6, $7, $8, true, NOW(), NOW())"
    )
    .bind(user_id)
    .bind(&request.username)
    .bind(&request.email)
    .bind(&password_hash)
    .bind(&request.role)
    .bind(&request.first_name)
    .bind(&request.last_name)
    .bind(&wallet_address)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to create user: {}", e)))?;

    // Create JWT claims
    let claims = Claims::new(user_id, request.username.clone(), request.role.clone());
    
    // Generate token
    let access_token = state.jwt_service.encode_token(&claims)?;

    let response = WalletRegistrationResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 24 * 60 * 60, // 24 hours in seconds
        user: SecureUserInfo {
            username: request.username,
            email: request.email,
            role: request.role,
            blockchain_registered: wallet_address.is_some(),
        },
        wallet_info,
    };

    Ok(Json(response))
}

/// Login with wallet information
#[utoipa::path(
    post,
    path = "/api/auth/login-with-wallet",
    tag = "Authentication",
    request_body = crate::handlers::auth::LoginRequest,
    responses(
        (status = 200, description = "Login successful with wallet information", body = WalletLoginResponse),
        (status = 401, description = "Invalid credentials or account inactive"),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error during login")
    )
)]
pub async fn login_with_wallet(
    State(state): State<AppState>,
    Json(request): Json<crate::handlers::auth::LoginRequest>,
) -> Result<Json<WalletLoginResponse>> {
    // Validate request
    request.validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation error: {}", e)))?;

    // Get user from database with proper type casting
    let user = sqlx::query!(
        "SELECT id, username, email, password_hash, role::text as role,
                first_name, last_name, wallet_address, is_active
         FROM users WHERE username = $1",
        request.username
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?
    .ok_or_else(|| ApiError::Unauthorized("Invalid username or password".to_string()))?;

    // Check if user is active
    if user.is_active == Some(false) || !user.is_active.unwrap_or(true) {
        return Err(ApiError::Unauthorized("Account is deactivated".to_string()));
    }

    // Verify password
    if !PasswordService::verify_password(&request.password, &user.password_hash)? {
        return Err(ApiError::Unauthorized("Invalid username or password".to_string()));
    }

    // Get wallet information if user has a wallet
    let mut wallet_info = None;
    if let Some(wallet_addr) = &user.wallet_address {
        let wallet_service = WalletService::new(&state.config.solana_rpc_url);
        if let Ok(pubkey) = Pubkey::from_str(wallet_addr) {
            let balance_lamports = wallet_service.get_balance(&pubkey).await.ok();
            let balance_sol = balance_lamports.map(crate::services::wallet_service::lamports_to_sol);
            
            wallet_info = Some(UserWalletInfo {
                address: wallet_addr.to_string(),
                balance_lamports,
                balance_sol,
            });
        }
    }

    // Create JWT claims
    let claims = Claims::new(
        user.id, 
        user.username.clone(),
        user.role.clone().unwrap_or_default()
    );
    
    // Generate token
    let access_token = state.jwt_service.encode_token(&claims)?;

    let response = WalletLoginResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 24 * 60 * 60,
        user: SecureUserInfo {
            username: user.username,
            email: user.email,
            role: user.role.unwrap_or_default(),
            blockchain_registered: user.wallet_address.is_some(),
        },
        wallet_info,
    };

    Ok(Json(response))
}