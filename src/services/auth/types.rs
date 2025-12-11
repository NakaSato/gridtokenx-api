use crate::auth::SecureAuthResponse;
use chrono::Utc;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyEmailResult {
    pub message: String,
    pub email_verified: bool,
    pub verified_at: String,
    pub wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<SecureAuthResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResendVerificationResult {
    pub message: String,
    pub email: Option<String>,
    pub sent_at: Option<String>,
    pub expires_in_hours: Option<i64>,
    pub status: Option<String>,
    pub retry_after: Option<i64>,
}

#[derive(Debug)]
pub struct UserVerificationRecord {
    pub id: Uuid,
    pub email: Option<String>,
    pub username: Option<String>,
    pub email_verified: bool,
    #[allow(dead_code)]
    pub email_verification_token: Option<String>,
    pub email_verification_expires_at: Option<chrono::DateTime<Utc>>,
    pub role: Option<String>,
}
