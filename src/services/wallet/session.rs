//! Wallet Session Service
//!
//! Manages wallet unlock sessions for secure auto-trading.
//! - Sessions expire after 30 days
//! - New device login invalidates all previous sessions
//! - Cached keys are encrypted with session token

use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use sqlx::{PgPool, Row};
use tracing::{info, warn};
use uuid::Uuid;

use crate::handlers::auth::types::WalletSessionInfo;
use crate::services::WalletService;

/// Default session TTL in days
const DEFAULT_SESSION_TTL_DAYS: i64 = 30;

/// Max unlock attempts before rate limiting
const MAX_UNLOCK_ATTEMPTS: i64 = 5;

/// Rate limit cooldown in minutes
const RATE_LIMIT_COOLDOWN_MINUTES: i64 = 15;

/// Wallet session service for managing unlock sessions
#[derive(Clone)]
pub struct WalletSessionService {
    db: PgPool,
    session_ttl_days: i64,
}

impl WalletSessionService {
    /// Create a new WalletSessionService
    pub fn new(db: PgPool) -> Self {
        let session_ttl_days = std::env::var("WALLET_SESSION_TTL_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_SESSION_TTL_DAYS);

        info!(
            "WalletSessionService initialized with {} day session TTL",
            session_ttl_days
        );

        Self {
            db,
            session_ttl_days,
        }
    }

    pub fn session_ttl_days(&self) -> i64 {
        self.session_ttl_days
    }

    /// Generate a secure random session token
    fn generate_session_token() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect()
    }

    /// Check if user is rate limited for unlock attempts
    pub async fn is_rate_limited(&self, user_id: Uuid, ip_address: Option<&str>) -> Result<bool> {
        let cutoff = Utc::now() - Duration::minutes(RATE_LIMIT_COOLDOWN_MINUTES);

        // Count failed attempts in the cooldown window
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM wallet_unlock_attempts
            WHERE user_id = $1 
              AND attempted_at > $2 
              AND success = false
            "#,
        )
        .bind(user_id)
        .bind(cutoff)
        .fetch_one(&self.db)
        .await?;

        if count >= MAX_UNLOCK_ATTEMPTS {
            warn!(
                "User {} is rate limited for wallet unlock ({} failed attempts)",
                user_id, count
            );
            return Ok(true);
        }

        // Also check by IP if provided
        if let Some(ip) = ip_address {
            let ip_count: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM wallet_unlock_attempts
                WHERE ip_address = $1::inet
                  AND attempted_at > $2 
                  AND success = false
                "#,
            )
            .bind(ip)
            .bind(cutoff)
            .fetch_one(&self.db)
            .await?;

            if ip_count >= MAX_UNLOCK_ATTEMPTS * 2 {
                warn!("IP {} is rate limited for wallet unlock", ip);
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Record an unlock attempt
    pub async fn record_attempt(
        &self,
        user_id: Uuid,
        ip_address: Option<&str>,
        success: bool,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO wallet_unlock_attempts (user_id, ip_address, success)
            VALUES ($1, $2::inet, $3)
            "#,
        )
        .bind(user_id)
        .bind(ip_address)
        .bind(success)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Check if a device is new (different from active sessions)
    pub async fn is_new_device(&self, user_id: Uuid, device_fingerprint: &str) -> Result<bool> {
        let existing: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT id FROM wallet_sessions
            WHERE user_id = $1 
              AND device_fingerprint = $2 
              AND is_active = true
              AND expires_at > NOW()
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(device_fingerprint)
        .fetch_optional(&self.db)
        .await?;

        Ok(existing.is_none())
    }

    /// Revoke all active sessions for a user
    pub async fn revoke_all_sessions(&self, user_id: Uuid, reason: &str) -> Result<i64> {
        let result = sqlx::query(
            r#"
            UPDATE wallet_sessions
            SET is_active = false,
            revoked_at = NOW(),
            revoked_reason = $2
            WHERE user_id = $1 AND is_active = true
            "#,
        )
        .bind(user_id)
        .bind(reason)
        .execute(&self.db)
        .await?;

        let count = result.rows_affected() as i64;
        if count > 0 {
            info!(
                "Revoked {} wallet sessions for user {} (reason: {})",
                count, user_id, reason
            );
        }

        Ok(count)
    }

    /// Create a new wallet session (unlock wallet)
    pub async fn create_session(
        &self,
        user_id: Uuid,
        wallet_password: &str,
        device_fingerprint: &str,
        device_name: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        master_secret: &str, // For legacy decryption
    ) -> Result<(String, chrono::DateTime<Utc>, i64)> {
        // 1. Check if new device - if so, revoke all existing sessions
        let is_new = self.is_new_device(user_id, device_fingerprint).await?;
        let mut revoked_count = 0i64;

        if is_new {
            revoked_count = self.revoke_all_sessions(user_id, "new_device").await?;
        }

        // 2. Get user's encrypted private key from database
        let row = sqlx::query(
            r#"
            SELECT encrypted_private_key, wallet_salt, encryption_iv, wallet_encryption_version
            FROM users WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let encrypted_private_key: Option<Vec<u8>> = row.try_get("encrypted_private_key")?;
        let wallet_salt: Option<Vec<u8>> = row.try_get("wallet_salt")?;
        let encryption_iv: Option<Vec<u8>> = row.try_get("encryption_iv")?;
        let wallet_encryption_version: Option<i32> = row.try_get("wallet_encryption_version")?;

        let encrypted_pk = encrypted_private_key.ok_or_else(|| {
            anyhow!("User has no wallet. Please generate a wallet first.")
        })?;

        let encryption_version = wallet_encryption_version.unwrap_or(1);

        // 3. Decrypt the private key
        let decrypted_key = if encryption_version == 2 {
            // User password encryption (new)
            let salt = wallet_salt.ok_or_else(|| anyhow!("Missing salt"))?;
            let iv = encryption_iv.ok_or_else(|| anyhow!("Missing IV"))?;

            use base64::{engine::general_purpose, Engine as _};
            let encrypted_b64 = general_purpose::STANDARD.encode(&encrypted_pk);
            let salt_b64 = general_purpose::STANDARD.encode(&salt);
            let iv_b64 = general_purpose::STANDARD.encode(&iv);

            WalletService::decrypt_private_key(wallet_password, &encrypted_b64, &salt_b64, &iv_b64)
                .map_err(|_| anyhow!("Invalid wallet password"))?
        } else {
            // Legacy master secret encryption - use master secret
            let salt = wallet_salt.ok_or_else(|| anyhow!("Missing salt"))?;
            let iv = encryption_iv.ok_or_else(|| anyhow!("Missing IV"))?;

            use base64::{engine::general_purpose, Engine as _};
            let encrypted_b64 = general_purpose::STANDARD.encode(&encrypted_pk);
            let salt_b64 = general_purpose::STANDARD.encode(&salt);
            let iv_b64 = general_purpose::STANDARD.encode(&iv);

            WalletService::decrypt_private_key(master_secret, &encrypted_b64, &salt_b64, &iv_b64)
                .map_err(|e| anyhow!("Failed to decrypt legacy wallet: {}", e))?
        };

        // 4. Create session token and encrypt key with it
        let session_token = Self::generate_session_token();
        let expires_at = Utc::now() + Duration::days(self.session_ttl_days);

        let (cached_encrypted, cached_salt, cached_iv) =
            WalletService::encrypt_private_key(&session_token, &decrypted_key)
                .map_err(|e| anyhow!("Failed to encrypt session cache: {}", e))?;

        use base64::{engine::general_purpose, Engine as _};
        let cached_key_bytes = general_purpose::STANDARD.decode(&cached_encrypted)?;
        let cached_salt_bytes = general_purpose::STANDARD.decode(&cached_salt)?;
        let cached_iv_bytes = general_purpose::STANDARD.decode(&cached_iv)?;

        // 5. Store session in database
        sqlx::query(
            r#"
            INSERT INTO wallet_sessions (
                user_id, session_token, device_fingerprint, device_name,
                ip_address, user_agent, cached_key_encrypted, key_salt, key_iv, expires_at
            ) VALUES ($1, $2, $3, $4, $5::inet, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(user_id)
        .bind(&session_token)
        .bind(device_fingerprint)
        .bind(device_name)
        .bind(ip_address)
        .bind(user_agent)
        .bind(&cached_key_bytes)
        .bind(&cached_salt_bytes)
        .bind(&cached_iv_bytes)
        .bind(expires_at)
        .execute(&self.db)
        .await?;

        info!(
            "Created wallet session for user {} (device: {}, expires: {})",
            user_id,
            device_name.unwrap_or("unknown"),
            expires_at
        );

        Ok((session_token, expires_at, revoked_count))
    }

    /// Get cached private key from session
    pub async fn get_cached_key(&self, user_id: Uuid, session_token: &str) -> Result<Vec<u8>> {
        let row = sqlx::query(
            r#"
            SELECT cached_key_encrypted, key_salt, key_iv
            FROM wallet_sessions
            WHERE user_id = $1 
              AND session_token = $2 
              AND is_active = true
              AND expires_at > NOW()
            "#,
        )
        .bind(user_id)
        .bind(session_token)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| anyhow!("No valid session found"))?;

        let cached_key_encrypted: Vec<u8> = row.try_get("cached_key_encrypted")?;
        let key_salt: Vec<u8> = row.try_get("key_salt")?;
        let key_iv: Vec<u8> = row.try_get("key_iv")?;

        use base64::{engine::general_purpose, Engine as _};
        let encrypted_b64 = general_purpose::STANDARD.encode(&cached_key_encrypted);
        let salt_b64 = general_purpose::STANDARD.encode(&key_salt);
        let iv_b64 = general_purpose::STANDARD.encode(&key_iv);

        let decrypted =
            WalletService::decrypt_private_key(session_token, &encrypted_b64, &salt_b64, &iv_b64)
                .map_err(|e| anyhow!("Failed to decrypt cached key: {}", e))?;

        // Update last_used_at
        self.touch_session(session_token).await?;

        Ok(decrypted)
    }

    /// Update last_used_at for a session
    pub async fn touch_session(&self, session_token: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE wallet_sessions
            SET last_used_at = NOW()
            WHERE session_token = $1 AND is_active = true
            "#,
        )
        .bind(session_token)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get session info for a user
    pub async fn get_session_info(&self, user_id: Uuid) -> Result<WalletSessionInfo> {
        // Get user's encryption version
        let row = sqlx::query(
            r#"
            SELECT wallet_encryption_version, encrypted_private_key
            FROM users WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let wallet_encryption_version: Option<i32> = row.try_get("wallet_encryption_version")?;
        let encrypted_private_key: Option<Vec<u8>> = row.try_get("encrypted_private_key")?;

        let encryption_version = wallet_encryption_version.unwrap_or(1);
        let has_wallet = encrypted_private_key.is_some();
        let needs_password_setup = has_wallet && encryption_version == 1;

        // Find active session
        let row = sqlx::query(
            r#"
            SELECT expires_at, device_name, created_at
            FROM wallet_sessions
            WHERE user_id = $1 AND is_active = true AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        let is_unlocked = row.is_some();
        let expires_at = row.as_ref().and_then(|r| r.try_get("expires_at").ok());
        let device_name = row.as_ref().and_then(|r| r.try_get("device_name").ok());
        let created_at = row.as_ref().and_then(|r| r.try_get("created_at").ok());

        Ok(WalletSessionInfo {
            is_unlocked,
            needs_password_setup,
            expires_at,
            device_name,
            created_at,
            encryption_version,
        })
    }

    /// Revoke a specific session (lock wallet)
    pub async fn revoke_session(&self, user_id: Uuid, session_token: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE wallet_sessions
            SET is_active = false, revoked_at = NOW(), revoked_reason = 'user_logout'
            WHERE user_id = $1 AND session_token = $2 AND is_active = true
            "#,
        )
        .bind(user_id)
        .bind(session_token)
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Cleanup expired sessions (call periodically)
    pub async fn cleanup_expired_sessions(&self) -> Result<i64> {
        let result = sqlx::query(
            r#"
            UPDATE wallet_sessions
            SET is_active = false, revoked_at = NOW(), revoked_reason = 'expired'
            WHERE is_active = true AND expires_at < NOW()
            "#,
        )
        .execute(&self.db)
        .await?;

        let count = result.rows_affected() as i64;
        if count > 0 {
            info!("Cleaned up {} expired wallet sessions", count);
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_session_token() {
        let token = WalletSessionService::generate_session_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_different_tokens() {
        let token1 = WalletSessionService::generate_session_token();
        let token2 = WalletSessionService::generate_session_token();
        assert_ne!(token1, token2);
    }
}
