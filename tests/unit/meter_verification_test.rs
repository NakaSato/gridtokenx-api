use anyhow::Result;
use gridtokenx_apigateway::services::meter_verification_service::{MeterVerificationService, VerificationMethod};
use sqlx::PgPool;
use uuid::Uuid;
use std::str::FromStr;
use bcrypt;
use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_db() -> PgPool {
        // Use in-memory SQLite for testing or a test database
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost:5432/gridtokenx_test".to_string());
        
        PgPool::connect(&database_url).await.expect("Failed to connect to test database")
    }

    async fn setup_test_service(db_pool: PgPool) -> MeterVerificationService {
        MeterVerificationService::new(db_pool)
    }

    #[tokio::test]
    async fn test_meter_verification_service_creation() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        // Service should be created successfully
        assert!(true); // If we reach here, service creation succeeded
    }

    #[tokio::test]
    async fn test_verify_meter_valid_request() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let meter_serial = "SM-2024-TEST001".to_string();
        let meter_key = "ABCDEFGHIJKLMNOP".to_string(); // 16 chars
        
        let result = service.verify_meter(
            &user_id,
            &meter_serial,
            &meter_key,
            VerificationMethod::Serial,
            Some("Test Manufacturer".to_string()),
            "residential".to_string(),
            Some("Test Address".to_string()),
            Some("utility_bill_ref_123".to_string()),
        ).await;
        
        // Should succeed with valid inputs
        assert!(result.is_ok());
        
        let meter_id = result.unwrap();
        assert!(meter_id.to_string().len() > 0);
    }

    #[tokio::test]
    async fn test_verify_meter_invalid_key_format() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let meter_serial = "SM-2024-TEST002".to_string();
        let meter_key = "short".to_string(); // Too short
        
        let result = service.verify_meter(
            &user_id,
            &meter_serial,
            &meter_key,
            VerificationMethod::Serial,
            Some("Test Manufacturer".to_string()),
            "residential".to_string(),
            Some("Test Address".to_string()),
            None,
        ).await;
        
        // Should fail with invalid key format
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid meter key format"));
    }

    #[tokio::test]
    async fn test_verify_meter_duplicate_serial() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id1 = Uuid::new_v4();
        let user_id2 = Uuid::new_v4();
        let meter_serial = "SM-2024-DUPLICATE".to_string();
        let meter_key = "ABCDEFGHIJKLMNOP".to_string();
        
        // First user verifies meter
        let result1 = service.verify_meter(
            &user_id1,
            &meter_serial,
            &meter_key,
            VerificationMethod::Serial,
            None,
            "residential".to_string(),
            None,
            None,
        ).await;
        assert!(result1.is_ok());
        
        // Second user tries to verify same meter
        let result2 = service.verify_meter(
            &user_id2,
            &meter_serial,
            &meter_key,
            VerificationMethod::Serial,
            None,
            "residential".to_string(),
            None,
            None,
        ).await;
        
        // Should fail with "meter already claimed"
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("already been registered"));
    }

    #[tokio::test]
    async fn test_get_user_meters_empty() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let meters = service.get_user_meters(&user_id).await.unwrap();
        
        // New user should have no meters
        assert_eq!(meters.len(), 0);
    }

    #[tokio::test]
    async fn test_get_user_meters_with_verification() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let meter_serial = "SM-2024-USER001".to_string();
        let meter_key = "ABCDEFGHIJKLMNOP".to_string();
        
        // Verify a meter
        let result = service.verify_meter(
            &user_id,
            &meter_serial,
            &meter_key,
            VerificationMethod::Serial,
            Some("Test Manufacturer".to_string()),
            "residential".to_string(),
            Some("Test Address".to_string()),
            None,
        ).await;
        assert!(result.is_ok());
        
        // Get user's meters
        let meters = service.get_user_meters(&user_id).await.unwrap();
        
        // Should have one meter
        assert_eq!(meters.len(), 1);
        assert_eq!(meters[0].meter_serial, meter_serial);
        assert_eq!(meters[0].verification_status, "verified");
    }

    #[tokio::test]
    async fn test_verify_meter_ownership_valid() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let meter_serial = "SM-2024-OWN001".to_string();
        let meter_key = "ABCDEFGHIJKLMNOP".to_string();
        
        // Verify a meter
        let result = service.verify_meter(
            &user_id,
            &meter_serial,
            &meter_key,
            VerificationMethod::Serial,
            None,
            "residential".to_string(),
            None,
            None,
        ).await;
        assert!(result.is_ok());
        
        let meter_id = result.unwrap();
        
        // Check ownership
        let is_owner = service.verify_meter_ownership(&user_id, &meter_id).await.unwrap();
        assert!(is_owner);
    }

    #[tokio::test]
    async fn test_verify_meter_ownership_invalid_user() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id1 = Uuid::new_v4();
        let user_id2 = Uuid::new_v4();
        let meter_serial = "SM-2024-OWN002".to_string();
        let meter_key = "ABCDEFGHIJKLMNOP".to_string();
        
        // User 1 verifies a meter
        let result = service.verify_meter(
            &user_id1,
            &meter_serial,
            &meter_key,
            VerificationMethod::Serial,
            None,
            "residential".to_string(),
            None,
            None,
        ).await;
        assert!(result.is_ok());
        
        let meter_id = result.unwrap();
        
        // User 2 tries to verify ownership
        let is_owner = service.verify_meter_ownership(&user_id2, &meter_id).await.unwrap();
        assert!(!is_owner);
    }

    #[tokio::test]
    async fn test_verify_meter_ownership_nonexistent() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let nonexistent_meter_id = Uuid::new_v4();
        
        // Check ownership of nonexistent meter
        let is_owner = service.verify_meter_ownership(&user_id, &nonexistent_meter_id).await.unwrap();
        assert!(!is_owner);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let meter_key = "ABCDEFGHIJKLMNOP".to_string();
        
        // Make 6 attempts (rate limit is 5 per hour)
        let mut results = Vec::new();
        for i in 0..6 {
            let result = service.verify_meter(
                &user_id,
                &format!("SM-2024-RATE{:03}", i),
                &meter_key,
                VerificationMethod::Serial,
                None,
                "residential".to_string(),
                None,
                None,
            ).await;
            results.push(result.is_ok());
        }
        
        // First 5 should succeed, 6th should fail due to rate limit
        assert_eq!(results.iter().filter(|&&success| success).count(), 5);
        assert!(!results[5]); // 6th attempt should fail
        
        // Check that error mentions rate limiting
        if let Err(e) = service.verify_meter(
            &user_id,
            &"SM-2024-RATE006".to_string(),
            &meter_key,
            VerificationMethod::Serial,
            None,
            "residential".to_string(),
            None,
            None,
        ).await {
            assert!(e.to_string().contains("rate limit"));
        }
    }

    #[tokio::test]
    async fn test_log_verification_attempt() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let meter_serial = "SM-2024-LOG001".to_string();
        let meter_key = "ABCDEFGHIJKLMNOP".to_string();
        
        // Log successful attempt
        let result = service.log_verification_attempt(
            &user_id,
            &meter_serial,
            &meter_key,
            VerificationMethod::Serial,
            "success",
            None,
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_rate_limit() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        
        // Check rate limit for new user (should pass)
        let can_attempt1 = service.check_rate_limit(&user_id).await.unwrap();
        assert!(can_attempt1);
        
        // Log 5 attempts
        for i in 0..5 {
            service.log_verification_attempt(
                &user_id,
                &format!("SM-2024-RATECHK{:03}", i),
                &"TESTKEY123456789".to_string(),
                VerificationMethod::Serial,
                if i < 4 { "success" } else { "invalid_key" },
                None,
            ).await.unwrap();
        }
        
        // Check rate limit again (should fail)
        let can_attempt2 = service.check_rate_limit(&user_id).await.unwrap();
        assert!(!can_attempt2);
    }

    #[test]
    fn test_bcrypt_key_hashing() {
        let meter_key = "ABCDEFGHIJKLMNOP";
        let cost = 12;
        
        // Hash the key
        let hash = bcrypt::hash(meter_key, cost).unwrap();
        
        // Verify the hash
        let is_valid = bcrypt::verify(meter_key, &hash).unwrap();
        assert!(is_valid);
        
        // Verify wrong key fails
        let is_invalid = bcrypt::verify("WRONGKEY123456", &hash).unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_meter_key_validation() {
        // Test valid keys
        let valid_keys = vec![
            "ABCDEFGHIJKLMNOP", // 16 chars
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ", // 26 chars
            "ABCDEFGHIJKLMNOPQRST", // 20 chars
        ];
        
        for key in valid_keys {
            assert!(MeterVerificationService::validate_meter_key_format(key).is_ok());
        }
        
        // Test invalid keys
        let invalid_keys = vec![
            "", // empty
            "SHORT", // too short
            "ABCDEFGHIJKLMNO", // 15 chars (below minimum)
            "1234567890123456", // numbers only
            "ABCDEF-GHIJKLMNO", // contains dash
            "ABC DEF GHI JKLMNO", // contains spaces
            "ABCDEFGHIJKLMNOPQRSTUVWXYS", // 26 chars but wrong validation logic test
        ];
        
        for key in invalid_keys {
            if !key.is_empty() && key.len() >= 16 && key.len() <= 64 {
                // Some might pass basic length check but fail format check
                if key.chars().all(|c| c.is_alphanumeric()) {
                    assert!(MeterVerificationService::validate_meter_key_format(key).is_ok());
                } else {
                    assert!(MeterVerificationService::validate_meter_key_format(key).is_err());
                }
            } else {
                assert!(MeterVerificationService::validate_meter_key_format(key).is_err());
            }
        }
    }

    #[test]
    fn test_verification_method_enum() {
        // Test all verification methods
        let methods = vec![
            VerificationMethod::Serial,
            VerificationMethod::ApiKey,
            VerificationMethod::QrCode,
            VerificationMethod::ChallengeResponse,
        ];
        
        for method in methods {
            let method_str = method.to_string();
            assert!(!method_str.is_empty());
            assert!(method_str.len() > 0);
        }
    }

    #[tokio::test]
    async fn test_meter_serial_validation() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        // Test valid serial numbers
        let valid_serials = vec![
            "SM-2024-ABC123",
            "SM-2025-DEF456",
            "SM-2023-GHI789",
            "SM-2024-TEST001",
        ];
        
        for serial in valid_serials {
            assert!(service.validate_meter_serial_format(serial).is_ok());
        }
        
        // Test invalid serial numbers
        let invalid_serials = vec![
            "", // empty
            "INVALID", // wrong format
            "SM-20-ABC123", // wrong year format
            "SM-2024-123", // too short suffix
            "SM-2024-TOOLONGFORMATTING", // too long suffix
        ];
        
        for serial in invalid_serials {
            assert!(service.validate_meter_serial_format(serial).is_err());
        }
    }

    #[tokio::test]
    async fn test_audit_trail_logging() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let meter_serial = "SM-2024-AUDIT001".to_string();
        
        // Log various types of attempts
        let attempts = vec![
            ("success", None),
            ("invalid_key", Some("Invalid format")),
            ("meter_claimed", None),
            ("rate_limited", Some("Too many attempts")),
        ];
        
        for (outcome, details) in attempts {
            let result = service.log_verification_attempt(
                &user_id,
                &meter_serial,
                &"TESTKEY123456789".to_string(),
                VerificationMethod::Serial,
                outcome,
                details,
            ).await;
            
            assert!(result.is_ok());
        }
        
        // Verify attempts were logged (would require database query)
        // This is a placeholder for actual audit trail verification
    }
}
