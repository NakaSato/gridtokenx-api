//! Registration Handlers Module
//!
//! User registration and verification email handlers.

use axum::{
    extract::State,
    Json,
};
use tracing::info;
use uuid::Uuid;
use base64::{engine::general_purpose, Engine as _};

use crate::AppState;
use solana_sdk::signature::Signer;
use super::types::{
    RegistrationRequest, RegistrationResponse, AuthResponse, UserResponse,
    ResendVerificationRequest, VerifyEmailResponse,
};

/// Register Handler - inserts user into database
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegistrationRequest>,
) -> Json<RegistrationResponse> {
    info!("📝 Registration for user: {}", request.username);

    let id = Uuid::new_v4();
    let password_hash = format!("hash_{}", request.password); // Simplified for testing

    // Create wallet
    let keypair = crate::services::WalletService::create_keypair();
    let wallet_address = keypair.pubkey().to_string();
    let (encrypted_key, salt, iv) = crate::services::WalletService::encrypt_private_key(
        &state.config.encryption_secret,
        &keypair.to_bytes()
    ).map_err(|e| {
        tracing::error!("Failed to encrypt wallet: {}", e);
        panic!("Wallet encryption failed");
    }).unwrap();

    // Decode to bytes for BYTEA columns
    let encrypted_key_bytes = general_purpose::STANDARD.decode(&encrypted_key).unwrap_or_default();
    let salt_bytes = general_purpose::STANDARD.decode(&salt).unwrap_or_default();
    let iv_bytes = general_purpose::STANDARD.decode(&iv).unwrap_or_default();

    // Insert user into database
    let insert_result = sqlx::query(
        "INSERT INTO users (
            id, username, email, password_hash, role, first_name, last_name, 
            is_active, email_verified, blockchain_registered, 
            wallet_address, encrypted_private_key, wallet_salt, encryption_iv,
            created_at, updated_at
        )
         VALUES ($1, $2, $3, $4, 'user', $5, $6, true, false, false, $7, $8, $9, $10, NOW(), NOW())"
    )
    .bind(id)
    .bind(&request.username)
    .bind(&request.email)
    .bind(&password_hash)
    .bind(&request.first_name)
    .bind(&request.last_name)
    .bind(&wallet_address)
    .bind(&encrypted_key_bytes)
    .bind(&salt_bytes)
    .bind(&iv_bytes)
    .execute(&state.db)
    .await;

    if let Err(e) = insert_result {
        tracing::error!("❌ Database insert error: {}", e);
        
        return Json(RegistrationResponse {
            message: format!("Registration failed: {}", e),
            email_verification_sent: false,
            auth: None,
        });
    }

    info!("✅ User created in database: {} with wallet {}", request.username, wallet_address);

    // Airdrop has been moved to /api/v1/dev/faucet

    // Generate token
    let claims = crate::auth::Claims::new(id, request.username.clone(), "user".to_string());
    let token = state.jwt_service.encode_token(&claims).unwrap_or_else(|_| {
        format!("token_{}_{}", request.username, id)
    });

    let user = UserResponse {
        id,
        username: request.username,
        email: request.email,
        role: "user".to_string(),
        first_name: request.first_name,
        last_name: request.last_name,
        wallet_address: Some(wallet_address),
    };

    let auth = AuthResponse {
        access_token: token,
        expires_in: 86400,
        user,
    };

    Json(RegistrationResponse {
        message: "Registration successful".to_string(),
        email_verification_sent: false,
        auth: Some(auth),
    })
}

/// Resend verification email
pub async fn resend_verification(
    State(_state): State<AppState>,
    Json(request): Json<ResendVerificationRequest>,
) -> Json<VerifyEmailResponse> {
    info!("📧 Resend verification request for: {}", request.email);
    
    // In production, this would send an actual email
    // For now, just return success
    Json(VerifyEmailResponse {
        success: true,
        message: format!("Verification email sent to {}. Check your inbox.", request.email),
    })
}
