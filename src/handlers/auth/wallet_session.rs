//! Wallet Session Handlers
//!
//! API handlers for wallet unlock/lock and session management.

use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    Json,
};
use std::net::SocketAddr;
use tracing::{info, warn};

use crate::auth::middleware::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

use super::types::{
    LockWalletResponse, UnlockWalletRequest, UnlockWalletResponse, WalletSessionInfo,
};

/// Extract IP address from headers or connection
fn get_ip_address(headers: &HeaderMap, addr: &SocketAddr) -> Option<String> {
    // Try X-Forwarded-For first (for reverse proxy)
    if let Some(forwarded) = headers.get("X-Forwarded-For") {
        if let Ok(s) = forwarded.to_str() {
            if let Some(first_ip) = s.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }
    // Fall back to connection address
    Some(addr.ip().to_string())
}

/// Extract user agent from headers
fn get_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract session token from headers
fn get_session_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-wallet-session")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Unlock wallet - creates session, revokes other devices
/// POST /api/v1/users/wallet/unlock
#[utoipa::path(
    post,
    path = "/api/v1/users/wallet/unlock",
    request_body = UnlockWalletRequest,
    responses(
        (status = 200, description = "Wallet unlocked successfully", body = UnlockWalletResponse),
        (status = 400, description = "Invalid password or rate limited"),
        (status = 401, description = "Unauthorized")
    ),
    security(("jwt_token" = [])),
    tag = "wallet"
)]
pub async fn unlock_wallet(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<UnlockWalletRequest>,
) -> Result<Json<UnlockWalletResponse>, ApiError> {
    let user_id = user.0.sub;
    let ip_address = get_ip_address(&headers, &addr);
    let user_agent = get_user_agent(&headers);

    info!("🔓 Wallet unlock request for user {}", user_id);

    // Check rate limiting
    if state
        .wallet_session
        .is_rate_limited(user_id, ip_address.as_deref())
        .await
        .unwrap_or(false)
    {
        // Record failed attempt
        let _ = state
            .wallet_session
            .record_attempt(user_id, ip_address.as_deref(), false)
            .await;

        return Ok(Json(UnlockWalletResponse {
            success: false,
            message: "Too many failed attempts. Please try again in 15 minutes.".to_string(),
            session_token: None,
            expires_at: None,
            revoked_sessions: 0,
        }));
    }

    // Attempt to create session
    match state
        .wallet_session
        .create_session(
            user_id,
            &payload.wallet_password,
            &payload.device_fingerprint,
            payload.device_name.as_deref(),
            ip_address.as_deref(),
            user_agent.as_deref(),
            &state.config.encryption_secret, // For legacy decryption
        )
        .await
    {
        Ok((session_token, expires_at, revoked_count)) => {
            // Record successful attempt
            let _ = state
                .wallet_session
                .record_attempt(user_id, ip_address.as_deref(), true)
                .await;

            info!(
                "✅ Wallet unlocked for user {} (revoked {} other sessions)",
                user_id, revoked_count
            );

            Ok(Json(UnlockWalletResponse {
                success: true,
                message: format!(
                    "Wallet unlocked successfully. Session valid for {} days.",
                    state
                        .wallet_session
                        .session_ttl_days()
                        .to_string()
                        .parse::<i64>()
                        .unwrap_or(30)
                ),
                session_token: Some(session_token),
                expires_at: Some(expires_at),
                revoked_sessions: revoked_count,
            }))
        }
        Err(e) => {
            // Record failed attempt
            let _ = state
                .wallet_session
                .record_attempt(user_id, ip_address.as_deref(), false)
                .await;

            warn!("❌ Wallet unlock failed for user {}: {}", user_id, e);

            Ok(Json(UnlockWalletResponse {
                success: false,
                message: e.to_string(),
                session_token: None,
                expires_at: None,
                revoked_sessions: 0,
            }))
        }
    }
}

/// Lock wallet - revokes current session
/// POST /api/v1/users/wallet/lock
#[utoipa::path(
    post,
    path = "/api/v1/users/wallet/lock",
    responses(
        (status = 200, description = "Wallet locked successfully", body = LockWalletResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("jwt_token" = [])),
    tag = "wallet"
)]
pub async fn lock_wallet(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    headers: HeaderMap,
) -> Result<Json<LockWalletResponse>, ApiError> {
    let user_id = user.0.sub;

    info!("🔒 Wallet lock request for user {}", user_id);

    // Get session token from header
    let session_token = get_session_token(&headers);

    if let Some(token) = session_token {
        let revoked = state
            .wallet_session
            .revoke_session(user_id, &token)
            .await
            .unwrap_or(false);

        if revoked {
            info!("✅ Wallet locked for user {}", user_id);
            return Ok(Json(LockWalletResponse {
                success: true,
                message: "Wallet locked successfully.".to_string(),
            }));
        }
    }

    // If no session token or session not found, still return success
    Ok(Json(LockWalletResponse {
        success: true,
        message: "Wallet is not unlocked or session already expired.".to_string(),
    }))
}

/// Get current session status
/// GET /api/v1/users/wallet/session
#[utoipa::path(
    get,
    path = "/api/v1/users/wallet/session",
    responses(
        (status = 200, description = "Wallet session info", body = WalletSessionInfo),
        (status = 401, description = "Unauthorized")
    ),
    security(("jwt_token" = [])),
    tag = "wallet"
)]
pub async fn get_wallet_session(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<WalletSessionInfo>, ApiError> {
    let user_id = user.0.sub;

    let session_info = state
        .wallet_session
        .get_session_info(user_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get session info: {}", e)))?;

    Ok(Json(session_info))
}

/// Lock all sessions (logout from all devices)
/// POST /api/v1/users/wallet/lock-all
#[utoipa::path(
    post,
    path = "/api/v1/users/wallet/lock-all",
    responses(
        (status = 200, description = "All wallet sessions locked", body = LockWalletResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("jwt_token" = [])),
    tag = "wallet"
)]
pub async fn lock_all_sessions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<LockWalletResponse>, ApiError> {
    let user_id = user.0.sub;

    info!("🔒 Lock all wallet sessions request for user {}", user_id);

    let count = state
        .wallet_session
        .revoke_all_sessions(user_id, "user_logout")
        .await
        .unwrap_or(0);

    info!(
        "✅ Locked {} wallet sessions for user {}",
        count, user_id
    );

    Ok(Json(LockWalletResponse {
        success: true,
        message: format!("Locked {} sessions.", count),
    }))
}
