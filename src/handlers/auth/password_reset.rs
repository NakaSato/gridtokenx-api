//! Password Reset Handlers Module
//!
//! Handlers for forgot password and password reset functionality.

use axum::{
    extract::State,
    Json,
};
use chrono::{Duration, Utc};
use tracing::info;
use uuid::Uuid;

use crate::AppState;
use crate::auth::password::PasswordService;
use super::types::{
    ForgotPasswordRequest, ResetPasswordRequest, VerifyEmailResponse,
    ChangePasswordRequest,
};

/// Forgot Password Handler - generates reset token and sends email
/// POST /api/v1/auth/forgot-password
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent (if email exists)", body = VerifyEmailResponse),
    ),
    tag = "auth"
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(request): Json<ForgotPasswordRequest>,
) -> Json<VerifyEmailResponse> {
    info!("🔑 Password reset request for: {}", request.email);

    // Look up user by email
    let user_result = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, username FROM users WHERE email = $1 AND is_active = true"
    )
    .bind(&request.email)
    .fetch_optional(&state.db)
    .await;

    let (user_id, username) = match user_result {
        Ok(Some(user)) => {
            info!("🔑 Password reset initiated for user: {} (email: {})", user.1, request.email);
            user
        }
        Ok(None) => {
            // Don't reveal if email exists (security best practice)
            info!("Password reset requested for non-existent email: {}", request.email);
            return Json(VerifyEmailResponse::simple(
                true,
                "If an account with that email exists, a password reset link has been sent."
            ));
        }
        Err(e) => {
            tracing::error!("Database error looking up user: {}", e);
            return Json(VerifyEmailResponse::simple(
                false,
                "An error occurred. Please try again."
            ));
        }
    };

    // Generate reset token
    let reset_token = Uuid::new_v4().to_string();
    let reset_expires_at = Utc::now() + Duration::hours(1); // 1 hour expiry

    // Store reset token in database
    let update_result = sqlx::query(
        "UPDATE users SET 
            password_reset_token = $1, 
            password_reset_expires_at = $2,
            updated_at = NOW()
         WHERE id = $3"
    )
    .bind(&reset_token)
    .bind(reset_expires_at)
    .bind(user_id)
    .execute(&state.db)
    .await;

    if let Err(e) = update_result {
        tracing::error!("Failed to store password reset token: {}", e);
        return Json(VerifyEmailResponse::simple(
            false,
            "Failed to generate reset token. Please try again."
        ));
    }

    // Send password reset email
    if let Some(ref email_service) = state.email_service {
        match email_service.send_password_reset_email(
            &request.email,
            &reset_token,
            &username,
        ).await {
            Ok(()) => {
                info!("📧 Password reset email sent to {}", request.email);
            }
            Err(e) => {
                tracing::error!("❌ Failed to send password reset email: {}", e);
                // Still return success to not reveal if email exists
            }
        }
    } else {
        info!("⚠️ Email service not configured, skipping password reset email");
    }

    Json(VerifyEmailResponse::simple(
        true,
        "If an account with that email exists, a password reset link has been sent."
    ))
}

/// Reset Password Handler - validates token and updates password
/// POST /api/v1/auth/reset-password
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful", body = VerifyEmailResponse),
        (status = 400, description = "Invalid token or password too short")
    ),
    tag = "auth"
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(request): Json<ResetPasswordRequest>,
) -> Json<VerifyEmailResponse> {
    info!("🔐 Password reset attempt with token");

    if request.token.is_empty() {
        return Json(VerifyEmailResponse::simple(
            false,
            "Reset token is required."
        ));
    }

    if request.new_password.len() < 8 {
        return Json(VerifyEmailResponse::simple(
            false,
            "Password must be at least 8 characters long."
        ));
    }

    // Look up user by reset token
    let user_result = sqlx::query_as::<_, (Uuid, String, chrono::DateTime<Utc>)>(
        "SELECT id, username, password_reset_expires_at 
         FROM users 
         WHERE password_reset_token = $1 
           AND password_reset_expires_at IS NOT NULL
           AND is_active = true"
    )
    .bind(&request.token)
    .fetch_optional(&state.db)
    .await;

    let (user_id, username, expires_at) = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Json(VerifyEmailResponse::simple(
                false,
                "Invalid or expired reset token."
            ));
        }
        Err(e) => {
            tracing::error!("Database error looking up reset token: {}", e);
            return Json(VerifyEmailResponse::simple(
                false,
                "An error occurred. Please try again."
            ));
        }
    };

    // Check if token is expired
    if Utc::now() > expires_at {
        return Json(VerifyEmailResponse::simple(
            false,
            "Reset token has expired. Please request a new one."
        ));
    }

    // Hash new password using bcrypt
    let password_hash = match PasswordService::hash_password(&request.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Failed to hash new password: {}", e);
            return Json(VerifyEmailResponse::simple(
                false,
                format!("{}", e)
            ));
        }
    };

    // Update password and clear reset token
    let update_result = sqlx::query(
        "UPDATE users SET 
            password_hash = $1,
            password_reset_token = NULL,
            password_reset_expires_at = NULL,
            updated_at = NOW()
         WHERE id = $2"
    )
    .bind(&password_hash)
    .bind(user_id)
    .execute(&state.db)
    .await;

    match update_result {
        Ok(_) => {
            info!("✅ Password reset successful for user: {} (id: {})", username, user_id);
            Json(VerifyEmailResponse::simple(
                true,
                "Password has been reset successfully. You can now login with your new password."
            ))
        }
        Err(e) => {
            tracing::error!("Failed to update password: {}", e);
            Json(VerifyEmailResponse::simple(
                false,
                "Failed to reset password. Please try again."
            ))
        }
    }
}

/// Change Password Handler - for authenticated users to change their password
/// POST /api/v1/auth/change-password
#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = VerifyEmailResponse),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Incorrect current password")
    ),
    security(
        ("jwt_token" = [])
    ),
    tag = "auth"
)]
pub async fn change_password(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(request): Json<super::types::ChangePasswordRequest>,
) -> Json<VerifyEmailResponse> {
    // Extract token from Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    let token = auth_header.strip_prefix("Bearer ").unwrap_or(auth_header);
    
    // Decode token to get user ID
    let claims = match state.jwt_service.decode_token(token) {
        Ok(c) => c,
        Err(_) => {
            return Json(VerifyEmailResponse::simple(
                false,
                "Invalid or expired token. Please log in again."
            ));
        }
    };
    
    info!("🔐 Password change request for user: {} (username: {})", claims.sub, claims.username);

    // Validate new password
    if request.new_password.len() < 8 {
        return Json(VerifyEmailResponse::simple(
            false,
            "New password must be at least 8 characters long."
        ));
    }

    // Get user's current password hash
    let user_result = sqlx::query_as::<_, (String,)>(
        "SELECT password_hash FROM users WHERE id = $1 AND is_active = true"
    )
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await;

    let current_hash = match user_result {
        Ok(Some(row)) => row.0,
        Ok(None) => {
            return Json(VerifyEmailResponse::simple(
                false,
                "User not found."
            ));
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return Json(VerifyEmailResponse::simple(
                false,
                "An error occurred. Please try again."
            ));
        }
    };

    // Verify current password
    match PasswordService::verify_password(&request.current_password, &current_hash) {
        Ok(true) => {}
        Ok(false) => {
            return Json(VerifyEmailResponse::simple(
                false,
                "Current password is incorrect."
            ));
        }
        Err(e) => {
            tracing::error!("Password verification error: {}", e);
            return Json(VerifyEmailResponse::simple(
                false,
                "An error occurred. Please try again."
            ));
        }
    }

    // Hash new password
    let new_hash = match PasswordService::hash_password(&request.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Password hashing error: {}", e);
            return Json(VerifyEmailResponse::simple(
                false,
                format!("{}", e)
            ));
        }
    };

    // Update password
    let update_result = sqlx::query(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(&new_hash)
    .bind(claims.sub)
    .execute(&state.db)
    .await;

    match update_result {
        Ok(_) => {
            info!("✅ Password changed for user: {} (username: {})", claims.sub, claims.username);
            Json(VerifyEmailResponse::simple(
                true,
                "Password changed successfully."
            ))
        }
        Err(e) => {
            tracing::error!("Failed to update password: {}", e);
            Json(VerifyEmailResponse::simple(
                false,
                "Failed to change password. Please try again."
            ))
        }
    }
}
