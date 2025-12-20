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
};

/// Forgot Password Handler - generates reset token and sends email
/// POST /api/v1/auth/forgot-password
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
        Ok(Some(user)) => user,
        Ok(None) => {
            // Don't reveal if email exists (security best practice)
            info!("Password reset requested for non-existent email: {}", request.email);
            return Json(VerifyEmailResponse {
                success: true,
                message: "If an account with that email exists, a password reset link has been sent.".to_string(),
            });
        }
        Err(e) => {
            tracing::error!("Database error looking up user: {}", e);
            return Json(VerifyEmailResponse {
                success: false,
                message: "An error occurred. Please try again.".to_string(),
            });
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
        return Json(VerifyEmailResponse {
            success: false,
            message: "Failed to generate reset token. Please try again.".to_string(),
        });
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

    Json(VerifyEmailResponse {
        success: true,
        message: "If an account with that email exists, a password reset link has been sent.".to_string(),
    })
}

/// Reset Password Handler - validates token and updates password
/// POST /api/v1/auth/reset-password
pub async fn reset_password(
    State(state): State<AppState>,
    Json(request): Json<ResetPasswordRequest>,
) -> Json<VerifyEmailResponse> {
    info!("🔐 Password reset attempt with token");

    if request.token.is_empty() {
        return Json(VerifyEmailResponse {
            success: false,
            message: "Reset token is required.".to_string(),
        });
    }

    if request.new_password.len() < 8 {
        return Json(VerifyEmailResponse {
            success: false,
            message: "Password must be at least 8 characters long.".to_string(),
        });
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
            return Json(VerifyEmailResponse {
                success: false,
                message: "Invalid or expired reset token.".to_string(),
            });
        }
        Err(e) => {
            tracing::error!("Database error looking up reset token: {}", e);
            return Json(VerifyEmailResponse {
                success: false,
                message: "An error occurred. Please try again.".to_string(),
            });
        }
    };

    // Check if token is expired
    if Utc::now() > expires_at {
        return Json(VerifyEmailResponse {
            success: false,
            message: "Reset token has expired. Please request a new one.".to_string(),
        });
    }

    // Hash new password using bcrypt
    let password_hash = match PasswordService::hash_password(&request.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Failed to hash new password: {}", e);
            return Json(VerifyEmailResponse {
                success: false,
                message: format!("{}", e),
            });
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
            info!("✅ Password reset successful for user: {}", username);
            Json(VerifyEmailResponse {
                success: true,
                message: "Password has been reset successfully. You can now login with your new password.".to_string(),
            })
        }
        Err(e) => {
            tracing::error!("Failed to update password: {}", e);
            Json(VerifyEmailResponse {
                success: false,
                message: "Failed to reset password. Please try again.".to_string(),
            })
        }
    }
}

/// Change Password Handler - for authenticated users to change their password
/// POST /api/v1/auth/change-password
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
            return Json(VerifyEmailResponse {
                success: false,
                message: "Invalid or expired token. Please log in again.".to_string(),
            });
        }
    };
    
    info!("🔐 Password change request for user: {}", claims.sub);

    // Validate new password
    if request.new_password.len() < 8 {
        return Json(VerifyEmailResponse {
            success: false,
            message: "New password must be at least 8 characters long.".to_string(),
        });
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
            return Json(VerifyEmailResponse {
                success: false,
                message: "User not found.".to_string(),
            });
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return Json(VerifyEmailResponse {
                success: false,
                message: "An error occurred. Please try again.".to_string(),
            });
        }
    };

    // Verify current password
    match PasswordService::verify_password(&request.current_password, &current_hash) {
        Ok(true) => {}
        Ok(false) => {
            return Json(VerifyEmailResponse {
                success: false,
                message: "Current password is incorrect.".to_string(),
            });
        }
        Err(e) => {
            tracing::error!("Password verification error: {}", e);
            return Json(VerifyEmailResponse {
                success: false,
                message: "An error occurred. Please try again.".to_string(),
            });
        }
    }

    // Hash new password
    let new_hash = match PasswordService::hash_password(&request.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Password hashing error: {}", e);
            return Json(VerifyEmailResponse {
                success: false,
                message: format!("{}", e),
            });
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
            info!("✅ Password changed for user: {}", claims.sub);
            Json(VerifyEmailResponse {
                success: true,
                message: "Password changed successfully.".to_string(),
            })
        }
        Err(e) => {
            tracing::error!("Failed to update password: {}", e);
            Json(VerifyEmailResponse {
                success: false,
                message: "Failed to change password. Please try again.".to_string(),
            })
        }
    }
}
