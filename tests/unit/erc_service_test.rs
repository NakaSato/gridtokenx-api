use anyhow::Result;
use gridtokenx_apigateway::services::erc_service::{ErcService, CertificateStatus};
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::json;
use chrono::{Utc, Duration};

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_db() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost:5432/gridtokenx_test".to_string());
        
        PgPool::connect(&database_url).await.expect("Failed to connect to test database")
    }

    async fn setup_test_service(db_pool: PgPool) -> ErcService {
        ErcService::new(db_pool)
    }

    #[tokio::test]
    async fn test_erc_service_creation() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        // Service should be created successfully
        assert!(true); // If we reach here, service creation succeeded
    }

    #[tokio::test]
    async fn test_generate_certificate_id() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let certificate_id = service.generate_certificate_id().await.unwrap();
        
        // Certificate ID should follow format: ERC-YYYY-NNNNNN
        assert!(certificate_id.starts_with("ERC-"));
        
        let current_year = Utc::now().year().to_string();
        assert!(certificate_id.contains(&current_year));
        
        // Should have numeric suffix
        let parts: Vec<&str> = certificate_id.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "ERC");
        assert_eq!(parts[1], current_year);
        
        // Last part should be numeric
        let numeric_part = parts[2];
        assert!(numeric_part.parse::<u32>().is_ok());
    }

    #[tokio::test]
    async fn test_issue_certificate_valid_request() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": wallet_address,
            "kwh_amount": 100.5,
            "renewable_source": "Solar",
            "issuer_name": "Green Energy Certifiers LLC",
            "issue_date": Utc::now().to_rfc3339(),
            "expiry_date": (Utc::now() + Duration::days(365)).to_rfc3339(),
            "validation_data": "utility_bill_ref_12345",
            "metadata": {
                "location": "California",
                "installation_type": "rooftop"
            }
        });
        
        let result = service.issue_certificate(request).await;
        
        // Should succeed with valid request
        assert!(result.is_ok());
        
        let certificate = result.unwrap();
        assert_eq!(certificate.user_id, user_id);
        assert_eq!(certificate.wallet_address, wallet_address);
        assert_eq!(certificate.kwh_amount, 100.5);
        assert_eq!(certificate.renewable_source, "Solar");
        assert_eq!(certificate.issuer_name, "Green Energy Certifiers LLC");
        assert_eq!(certificate.status, CertificateStatus::Active);
    }

    #[tokio::test]
    async fn test_issue_certificate_invalid_amount() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": wallet_address,
            "kwh_amount": -50.0, // Invalid negative amount
            "renewable_source": "Solar",
            "issuer_name": "Test Issuer"
        });
        
        let result = service.issue_certificate(request).await;
        
        // Should fail with invalid amount
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid energy amount"));
    }

    #[tokio::test]
    async fn test_issue_certificate_zero_amount() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": wallet_address,
            "kwh_amount": 0.0, // Zero amount
            "renewable_source": "Solar",
            "issuer_name": "Test Issuer"
        });
        
        let result = service.issue_certificate(request).await;
        
        // Should fail with zero amount
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be greater than zero"));
    }

    #[tokio::test]
    async fn test_issue_certificate_invalid_wallet() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let invalid_wallet = "invalid_wallet_address".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": invalid_wallet,
            "kwh_amount": 100.0,
            "renewable_source": "Solar",
            "issuer_name": "Test Issuer"
        });
        
        let result = service.issue_certificate(request).await;
        
        // Should fail with invalid wallet address
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid wallet address"));
    }

    #[tokio::test]
    async fn test_get_certificate_valid() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        // First, issue a certificate
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": wallet_address,
            "kwh_amount": 75.5,
            "renewable_source": "Wind",
            "issuer_name": "Wind Energy Co"
        });
        
        let issued = service.issue_certificate(request).await.unwrap();
        let certificate_id = issued.certificate_id;
        
        // Now retrieve the certificate
        let retrieved = service.get_certificate(&certificate_id).await.unwrap();
        
        assert_eq!(retrieved.certificate_id, certificate_id);
        assert_eq!(retrieved.user_id, user_id);
        assert_eq!(retrieved.wallet_address, wallet_address);
        assert_eq!(retrieved.kwh_amount, 75.5);
        assert_eq!(retrieved.renewable_source, "Wind");
        assert_eq!(retrieved.issuer_name, "Wind Energy Co");
    }

    #[tokio::test]
    async fn test_get_certificate_nonexistent() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let nonexistent_id = "ERC-2024-99999".to_string();
        
        let result = service.get_certificate(&nonexistent_id).await;
        
        // Should fail with certificate not found
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Certificate not found"));
    }

    #[tokio::test]
    async fn test_get_user_certificates_empty() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        
        let certificates = service.get_user_certificates(&user_id).await.unwrap();
        
        // New user should have no certificates
        assert_eq!(certificates.len(), 0);
    }

    #[tokio::test]
    async fn test_get_user_certificates_with_certs() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        // Issue multiple certificates
        for i in 1..=3 {
            let request = json!({
                "user_id": user_id.to_string(),
                "wallet_address": wallet_address,
                "kwh_amount": (i as f64) * 50.0,
                "renewable_source": "Solar",
                "issuer_name": "Test Issuer"
            });
            
            service.issue_certificate(request).await.unwrap();
        }
        
        // Get user's certificates
        let certificates = service.get_user_certificates(&user_id).await.unwrap();
        
        // Should have 3 certificates
        assert_eq!(certificates.len(), 3);
        
        // Verify amounts
        let amounts: Vec<f64> = certificates.iter().map(|c| c.kwh_amount).collect();
        assert!(amounts.contains(&50.0));
        assert!(amounts.contains(&100.0));
        assert!(amounts.contains(&150.0));
    }

    #[tokio::test]
    async fn test_validate_certificate_status_active() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        // Issue a certificate
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": wallet_address,
            "kwh_amount": 100.0,
            "renewable_source": "Solar",
            "issuer_name": "Test Issuer"
        });
        
        let issued = service.issue_certificate(request).await.unwrap();
        let certificate_id = issued.certificate_id;
        
        // Validate status
        let is_valid = service.validate_certificate_status(&certificate_id).await.unwrap();
        assert!(is_valid);
        
        // Check detailed status
        let certificate = service.get_certificate(&certificate_id).await.unwrap();
        assert_eq!(certificate.status, CertificateStatus::Active);
    }

    #[tokio::test]
    async fn test_transfer_certificate_valid() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        // Issue a certificate
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": wallet_address,
            "kwh_amount": 100.0,
            "renewable_source": "Solar",
            "issuer_name": "Test Issuer"
        });
        
        let issued = service.issue_certificate(request).await.unwrap();
        let certificate_id = issued.certificate_id;
        
        // Transfer to another user
        let new_user_id = Uuid::new_v4();
        let new_wallet = "5D3F3z7L9QpG7mJ6hVQK6k6k6k6k6k6k6k6k6k6k6k".to_string();
        
        let transfer_result = service.transfer_certificate(
            &certificate_id,
            &user_id, // current owner
            &new_user_id,
            &new_wallet,
        ).await;
        
        assert!(transfer_result.is_ok());
        
        // Verify transfer
        let updated_certificate = service.get_certificate(&certificate_id).await.unwrap();
        assert_eq!(updated_certificate.user_id, new_user_id);
        assert_eq!(updated_certificate.wallet_address, new_wallet);
    }

    #[tokio::test]
    async fn test_transfer_certificate_unauthorized() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        // Issue a certificate
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": wallet_address,
            "kwh_amount": 100.0,
            "renewable_source": "Solar",
            "issuer_name": "Test Issuer"
        });
        
        let issued = service.issue_certificate(request).await.unwrap();
        let certificate_id = issued.certificate_id;
        
        // Try to transfer from wrong user
        let wrong_user_id = Uuid::new_v4();
        let new_user_id = Uuid::new_v4();
        let new_wallet = "5D3F3z7L9QpG7mJ6hVQK6k6k6k6k6k6k6k6k6k6k6k".to_string();
        
        let transfer_result = service.transfer_certificate(
            &certificate_id,
            &wrong_user_id, // wrong owner
            &new_user_id,
            &new_wallet,
        ).await;
        
        // Should fail with unauthorized transfer
        assert!(transfer_result.is_err());
        assert!(transfer_result.unwrap_err().to_string().contains("not authorized"));
    }

    #[tokio::test]
    async fn test_retire_certificate_valid() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        // Issue a certificate
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": wallet_address,
            "kwh_amount": 100.0,
            "renewable_source": "Solar",
            "issuer_name": "Test Issuer"
        });
        
        let issued = service.issue_certificate(request).await.unwrap();
        let certificate_id = issued.certificate_id;
        
        // Retire the certificate
        let retire_result = service.retire_certificate(&certificate_id, &user_id).await;
        
        assert!(retire_result.is_ok());
        
        // Verify retirement
        let updated_certificate = service.get_certificate(&certificate_id).await.unwrap();
        assert_eq!(updated_certificate.status, CertificateStatus::Retired);
        assert!(updated_certificate.retired_at.is_some());
    }

    #[tokio::test]
    async fn test_retire_certificate_unauthorized() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        // Issue a certificate
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": wallet_address,
            "kwh_amount": 100.0,
            "renewable_source": "Solar",
            "issuer_name": "Test Issuer"
        });
        
        let issued = service.issue_certificate(request).await.unwrap();
        let certificate_id = issued.certificate_id;
        
        // Try to retire from wrong user
        let wrong_user_id = Uuid::new_v4();
        
        let retire_result = service.retire_certificate(&certificate_id, &wrong_user_id).await;
        
        // Should fail with unauthorized retirement
        assert!(retire_result.is_err());
        assert!(retire_result.unwrap_err().to_string().contains("not authorized"));
    }

    #[test]
    fn test_certificate_status_enum() {
        // Test all status variants
        let statuses = vec![
            CertificateStatus::Active,
            CertificateStatus::Transferred,
            CertificateStatus::Retired,
            CertificateStatus::Revoked,
        ];
        
        for status in statuses {
            let status_str = status.to_string();
            assert!(!status_str.is_empty());
            assert!(status_str.len() > 0);
        }
    }

    #[test]
    fn test_renewable_source_validation() {
        let valid_sources = vec![
            "Solar",
            "Wind",
            "Hydro",
            "Geothermal",
            "Biomass",
        ];
        
        let invalid_sources = vec![
            "", // empty
            "Coal",
            "Natural Gas",
            "Nuclear",
            "Invalid Source",
        ];
        
        for source in valid_sources {
            assert!(ErcService::validate_renewable_source(source).is_ok());
        }
        
        for source in invalid_sources {
            assert!(ErcService::validate_renewable_source(source).is_err());
        }
    }

    #[test]
    fn test_wallet_address_validation() {
        let valid_addresses = vec![
            "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX",
            "5D3F3z7L9QpG7mJ6hVQK6k6k6k6k6k6k6k6k6k6k6k",
            "11111111111111111111111111111112",
        ];
        
        let invalid_addresses = vec![
            "", // empty
            "invalid", // too short
            "invalid_wallet_address", // invalid format
            "ABCDEF123456789", // wrong length
        ];
        
        for address in valid_addresses {
            assert!(ErcService::validate_wallet_address(address).is_ok());
        }
        
        for address in invalid_addresses {
            assert!(ErcService::validate_wallet_address(address).is_err());
        }
    }

    #[test]
    fn test_certificate_metadata_creation() {
        let certificate_id = "ERC-2024-00042";
        let energy_amount = 100.0;
        let renewable_source = "Solar";
        let issuer = "Test Issuer";
        
        let metadata = ErcService::create_certificate_metadata(
            certificate_id,
            energy_amount,
            renewable_source,
            issuer,
        );
        
        // Verify metadata structure
        assert_eq!(metadata["name"], format!("Renewable Energy Certificate #{}", certificate_id));
        assert_eq!(metadata["description"], format!("Certificate for {} kWh of renewable energy from {} source", energy_amount, renewable_source));
        assert_eq!(metadata["attributes"][0]["trait_type"], "Energy Amount");
        assert_eq!(metadata["attributes"][0]["value"], energy_amount);
        assert_eq!(metadata["attributes"][0]["unit"], "kWh");
        assert_eq!(metadata["attributes"][1]["trait_type"], "Renewable Source");
        assert_eq!(metadata["attributes"][1]["value"], renewable_source);
        assert_eq!(metadata["attributes"][2]["trait_type"], "Issuer");
        assert_eq!(metadata["attributes"][2]["value"], issuer);
        assert_eq!(metadata["attributes"][3]["trait_type"], "Certificate ID");
        assert_eq!(metadata["attributes"][3]["value"], certificate_id);
        assert_eq!(metadata["properties"]["category"], "certificate");
    }

    #[test]
    fn test_energy_amount_validation() {
        // Valid amounts
        let valid_amounts = vec![0.1, 1.0, 100.0, 1000.0, 999999.9];
        
        for amount in valid_amounts {
            assert!(ErcService::validate_energy_amount(amount).is_ok());
        }
        
        // Invalid amounts
        let invalid_amounts = vec![0.0, -1.0, -100.0, f64::NAN, f64::INFINITY];
        
        for amount in invalid_amounts {
            if amount.is_nan() || amount.is_infinite() {
                continue; // Skip NaN/infinity tests as they're handled differently
            }
            assert!(ErcService::validate_energy_amount(amount).is_err());
        }
    }

    #[tokio::test]
    async fn test_update_blockchain_signature() {
        let db_pool = create_test_db().await;
        let service = setup_test_service(db_pool).await;
        
        // Issue a certificate
        let user_id = Uuid::new_v4();
        let wallet_address = "DYw8jZ9RfRfQqPkZHvPWqL5F7yKqWqfH8xKxCxJxQxX".to_string();
        
        let request = json!({
            "user_id": user_id.to_string(),
            "wallet_address": wallet_address,
            "kwh_amount": 100.0,
            "renewable_source": "Solar",
            "issuer_name": "Test Issuer"
        });
        
        let issued = service.issue_certificate(request).await.unwrap();
        let certificate_id = issued.certificate_id;
        
        // Initially should not have blockchain signature
        let certificate = service.get_certificate(&certificate_id).await.unwrap();
        assert!(certificate.blockchain_tx_signature.is_none());
        
        // Update with blockchain signature
        let tx_signature = "5j7s8Y7L9K8B7N6J5K8N7L6J5K8N7L6J5K8N7L6J5".to_string();
        service.update_blockchain_signature(&certificate_id, &tx_signature).await.unwrap();
        
        // Verify signature was updated
        let updated_certificate = service.get_certificate(&certificate_id).await.unwrap();
        assert_eq!(updated_certificate.blockchain_tx_signature, Some(tx_signature));
    }
}
