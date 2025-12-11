pub mod types;

use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use tracing::warn;

use crate::{
    auth::{jwt::JwtService, Claims, SecureAuthResponse, SecureUserInfo},
    config::Config,
    error::ApiError,
    services::{token::TokenService, AuditEvent, AuditLogger, EmailService},
};

pub use types::*;

/// Service for handling authentication-related logic
#[derive(Clone)]
pub struct AuthService {
    db: PgPool,
    config: Config,
    email_service: Option<EmailService>,
    jwt_service: JwtService,
    audit_logger: AuditLogger,
}

impl AuthService {
    pub fn new(
        db: PgPool,
        config: Config,
        email_service: Option<EmailService>,
        jwt_service: JwtService,
        audit_logger: AuditLogger,
    ) -> Self {
        Self {
            db,
            config,
            email_service,
            jwt_service,
            audit_logger,
        }
    }

    /// Verify a user's email address using the verification token
    pub async fn verify_email(&self, token: &str) -> Result<VerifyEmailResult, ApiError> {
        // Validate token format (Base58 encoded, reasonable length)
        if token.is_empty() || token.len() > 128 {
            return Err(ApiError::BadRequest("Invalid token format".to_string()));
        }

        // Hash the token to compare with database
        let hashed_token = TokenService::hash_token(token);

        // Find user by verification token
        let user = sqlx::query_as!(
            UserVerificationRecord,
            r#"
            SELECT 
                id,
                email as "email?",
                username as "username?",
                email_verified,
                email_verification_token,
                email_verification_expires_at,
                role::text as "role?"
            FROM users
            WHERE email_verification_token = $1
            "#,
            hashed_token
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::BadRequest("Invalid or expired verification token".to_string()))?;

        // Validate user has email (required for verification)
        let user_email = user
            .email
            .as_ref()
            .ok_or_else(|| ApiError::Internal("User email is missing".to_string()))?;

        // Check if already verified
        if user.email_verified {
            return Err(ApiError::BadRequest("Email already verified".to_string()));
        }

        // Check if token has expired
        if let Some(expires_at) = user.email_verification_expires_at {
            if expires_at < Utc::now() {
                return Err(ApiError::BadRequest(
                    "Verification token has expired. Please request a new one.".to_string(),
                ));
            }
        } else {
            return Err(ApiError::BadRequest(
                "Invalid verification token".to_string(),
            ));
        }

        // Update user: set email_verified = true, clear token, set verified_at
        let verified_at = Utc::now();
        sqlx::query!(
            r#"
            UPDATE users
            SET 
                email_verified = true,
                email_verification_token = NULL,
                email_verification_expires_at = NULL,
                email_verified_at = $1
            WHERE id = $2
            "#,
            verified_at,
            user.id
        )
        .execute(&self.db)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

        // Log email verification to audit logs
        self.audit_logger
            .log_async(AuditEvent::EmailVerified { user_id: user.id });

        // Send welcome email if email service is available
        if let Some(ref email_service) = self.email_service {
            let username = user.username.as_deref().unwrap_or("User");
            if let Err(e) = email_service.send_welcome_email(user_email, username).await {
                // Log error but don't fail the verification
                warn!("Failed to send welcome email: {}", e);
            }
        }

        // Create JWT token for immediate login (optional)
        let auth_response = if self.config.email.auto_login_after_verification {
            let username_str = user.username.clone().unwrap_or_else(|| "User".to_string());
            let role_str = user.role.clone().unwrap_or_else(|| "user".to_string());

            let claims = Claims::new(user.id, username_str.clone(), role_str.clone());
            let access_token = self
                .jwt_service
                .encode_token(&claims)
                .map_err(|e| ApiError::Internal(format!("Failed to generate token: {}", e)))?;

            Some(SecureAuthResponse {
                access_token,
                token_type: "Bearer".to_string(),
                expires_in: self.config.jwt_expiration,
                user: SecureUserInfo {
                    username: username_str,
                    email: user_email.to_string(),
                    role: role_str,
                    blockchain_registered: false, // Wallet will be created on first login
                },
            })
        } else {
            None
        };

        Ok(VerifyEmailResult {
            message: "Email verified successfully! Please login to secure your wallet.".to_string(),
            email_verified: true,
            verified_at: verified_at.to_rfc3339(),
            wallet_address: "".to_string(), // Placeholder, will be generated at login
            auth: auth_response,
        })
    }

    /// Resend verification email to a user
    pub async fn resend_verification(
        &self,
        email: &str,
    ) -> Result<ResendVerificationResult, ApiError> {
        // Validate email format
        if email.is_empty() || !email.contains('@') {
            return Err(ApiError::BadRequest("Invalid email format".to_string()));
        }

        // Check if email verification is enabled
        if !self.config.email.verification_enabled {
            return Err(ApiError::BadRequest(
                "Email verification is not enabled".to_string(),
            ));
        }

        // Email service must be available
        let email_service = self.email_service.as_ref().ok_or_else(|| {
            ApiError::Configuration("Email service is not configured".to_string())
        })?;

        // Find user by email
        let user = sqlx::query_as!(
            UserVerificationRecord,
            r#"
            SELECT 
                id,
                email as "email?",
                username as "username?",
                email_verified,
                email_verification_token,
                email_verification_expires_at,
                role::text as "role?"
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        // Validate user has email (required for verification)
        let user_email = user
            .email
            .as_ref()
            .ok_or_else(|| ApiError::Internal("User email is missing".to_string()))?;

        // Check if already verified - return success immediately
        if user.email_verified {
            let verified_at = Utc::now();
            return Ok(ResendVerificationResult {
                message: "Email is already verified. No action needed.".to_string(),
                email: Some(user_email.to_string()),
                sent_at: Some(verified_at.to_rfc3339()),
                expires_in_hours: Some(0),
                status: Some("already_verified".to_string()),
                retry_after: None,
            });
        }

        // Check if token has expired
        let is_token_expired = if let Some(expires_at) = user.email_verification_expires_at {
            expires_at < Utc::now()
        } else {
            // No expiry set means no token was sent yet, treat as expired
            true
        };

        // Rate limiting: Check if last email was sent within 10 seconds (to prevent spam)
        // BUT: Skip rate limiting if token has expired (allow immediate resend for expired tokens)
        if !is_token_expired {
            if let Some(expires_at) = user.email_verification_expires_at {
                // Calculate when the email was sent (24 hours before expiry)
                let sent_at = expires_at
                    - Duration::hours(self.config.email.verification_expiry_hours as i64);
                let time_since_sent = Utc::now() - sent_at;

                // Allow resend after 10 seconds
                if time_since_sent < Duration::seconds(10) {
                    let wait_seconds = 10 - time_since_sent.num_seconds();
                    // We need to convey this rate limit to the caller (handler).
                    // Returning a specific "rate limited" result which handler can map to 429.
                    return Ok(ResendVerificationResult {
                        message: format!(
                            "Rate limit exceeded. Please wait {} seconds before retrying",
                            wait_seconds
                        ),
                        email: None,
                        sent_at: None,
                        expires_in_hours: None,
                        status: Some("rate_limited".to_string()),
                        retry_after: Some(wait_seconds),
                    });
                }
            }
        }

        // Generate new verification token
        let token = TokenService::generate_verification_token();
        let hashed_token = TokenService::hash_token(&token);

        // Update user with new token
        let sent_at = Utc::now();
        let expires_at =
            sent_at + Duration::hours(self.config.email.verification_expiry_hours as i64);

        sqlx::query!(
            r#"
            UPDATE users
            SET 
                email_verification_token = $1,
                email_verification_sent_at = $2,
                email_verification_expires_at = $3
            WHERE id = $4
            "#,
            hashed_token,
            sent_at,
            expires_at,
            user.id
        )
        .execute(&self.db)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

        // Send verification email
        let username = user.username.as_deref().unwrap_or("User");
        email_service
            .send_verification_email(user_email, &token, username)
            .await
            .map_err(|e| ApiError::ExternalService(format!("Failed to send email: {}", e)))?;

        // Determine response message and status based on whether token was expired
        let (message, status) = if is_token_expired {
            (
                "Your verification token has expired. A new verification email has been sent! Please check your inbox.".to_string(),
                Some("expired_resent".to_string())
            )
        } else {
            (
                "Verification email sent successfully! Please check your inbox.".to_string(),
                Some("sent".to_string()),
            )
        };

        Ok(ResendVerificationResult {
            message,
            email: Some(user_email.to_string()),
            sent_at: Some(sent_at.to_rfc3339()),
            expires_in_hours: Some(self.config.email.verification_expiry_hours as i64),
            status,
            retry_after: None,
        })
    }
}
