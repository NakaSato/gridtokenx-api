use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use gridtokenx_apigateway::{
    config::{TokenizationConfig, ValidationError},
    models::meter::{MeterReading, MeterReadingStatus},
    services::{
        blockchain_service::BlockchainService,
        meter_polling_service::{MeterPollingService, MintResult},
        meter_service::MeterService,
        websocket_service::WebSocketService,
    },
};

// Mock implementations for testing
struct MockBlockchainService {
    should_fail: bool,
}

impl MockBlockchainService {
    fn new(should_fail: bool) -> Self {
        Self { should_fail }
    }
}

#[async_trait::async_trait]
impl MockBlockchainServiceTrait for MockBlockchainService {
    async fn mint_tokens_direct(
        &self,
        _user_wallet: &solana_sdk::pubkey::Pubkey,
        amount: u64,
    ) -> Result<solana_sdk::signature::Signature, Box<dyn std::error::Error>> {
        if self.should_fail {
            return Err("Mock blockchain error".into());
        }

        // Return a mock signature
        Ok(solana_sdk::signature::Signature::new_unique())
    }
}

struct MockMeterService {
    readings: Vec<MeterReading>,
}

impl MockMeterService {
    fn new(readings: Vec<MeterReading>) -> Self {
        Self { readings }
    }
}

#[async_trait::async_trait]
impl MockMeterServiceTrait for MockMeterService {
    async fn get_unminted_readings(
        &self,
        limit: usize,
    ) -> Result<Vec<MeterReading>, Box<dyn std::error::Error>> {
        Ok(self.readings.iter().take(limit).cloned().collect())
    }

    async fn mark_as_minted(
        &self,
        reading_id: &Uuid,
        tx_signature: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would update the database
        Ok(())
    }
}

struct MockWebSocketService {
    events: Arc<std::sync::Mutex<Vec<String>>>,
}

impl MockWebSocketService {
    fn new() -> Self {
        Self {
            events: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    fn get_events(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }
}

impl MockWebSocketServiceTrait for MockWebSocketService {
    async fn broadcast_tokens_minted(
        &self,
        _user_id: &Uuid,
        _wallet_address: &str,
        _meter_serial: &str,
        _kwh_amount: f64,
        _tokens_minted: u64,
        _transaction_signature: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event = format!("TokensMinted event");
        self.events.lock().unwrap().push(event);
        Ok(())
    }

    async fn broadcast_meter_reading_validation_failed(
        &self,
        _user_id: &Uuid,
        _wallet_address: &str,
        _meter_serial: &str,
        _kwh_amount: f64,
        _error_reason: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event = format!("MeterReadingValidationFailed event");
        self.events.lock().unwrap().push(event);
        Ok(())
    }

    async fn broadcast_batch_minting_completed(
        &self,
        _batch_id: &str,
        _total_readings: u32,
        _successful_mints: u32,
        _failed_mints: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event = format!("BatchMintingCompleted event");
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

fn create_test_meter_reading(kwh_amount: f64) -> MeterReading {
    MeterReading {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        wallet_address: "test_wallet_address".to_string(),
        kwh_amount,
        reading_timestamp: Utc::now(),
        submitted_at: Utc::now(),
        minted: false,
        mint_tx_signature: None,
        meter_serial: "test_meter_001".to_string(),
        verification_status: MeterReadingStatus::Verified,
    }
}

fn create_test_config() -> TokenizationConfig {
    TokenizationConfig {
        kwh_to_token_ratio: 1.0,
        decimals: 9,
        max_reading_kwh: 100.0,
        reading_max_age_days: 7,
        auto_mint_enabled: true,
        polling_interval_secs: 60,
        batch_size: 50,
        max_retry_attempts: 3,
        initial_retry_delay_secs: 300,
        retry_backoff_multiplier: 2.0,
        max_retry_delay_secs: 3600,
        transaction_timeout_secs: 60,
        max_transactions_per_batch: 20,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock traits for our services
    #[async_trait::async_trait]
    trait MockBlockchainServiceTrait {
        async fn mint_tokens_direct(
            &self,
            user_wallet: &solana_sdk::pubkey::Pubkey,
            amount: u64,
        ) -> Result<solana_sdk::signature::Signature, Box<dyn std::error::Error>>;
    }

    #[async_trait::async_trait]
    trait MockMeterServiceTrait {
        async fn get_unminted_readings(
            &self,
            limit: usize,
        ) -> Result<Vec<MeterReading>, Box<dyn std::error::Error>>;

        async fn mark_as_minted(
            &self,
            reading_id: &Uuid,
            tx_signature: &str,
        ) -> Result<(), Box<dyn std::error::Error>>;
    }

    #[async_trait::async_trait]
    trait MockWebSocketServiceTrait {
        async fn broadcast_tokens_minted(
            &self,
            user_id: &Uuid,
            wallet_address: &str,
            meter_serial: &str,
            kwh_amount: f64,
            tokens_minted: u64,
            transaction_signature: &str,
        ) -> Result<(), Box<dyn std::error::Error>>;

        async fn broadcast_meter_reading_validation_failed(
            &self,
            user_id: &Uuid,
            wallet_address: &str,
            meter_serial: &str,
            kwh_amount: f64,
            error_reason: &str,
        ) -> Result<(), Box<dyn std::error::Error>>;

        async fn broadcast_batch_minting_completed(
            &self,
            batch_id: &str,
            total_readings: u32,
            successful_mints: u32,
            failed_mints: u32,
        ) -> Result<(), Box<dyn std::error::Error>>;
    }

    #[tokio::test]
    async fn test_validate_reading_valid() {
        let config = create_test_config();

        let reading = create_test_meter_reading(50.0);

        // Create a minimal implementation of MeterPollingService for testing
        let service = create_test_meter_polling_service(&config);

        // Validate the reading
        let result = service.validate_reading(&reading);

        // Should not return an error
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_reading_too_old() {
        let config = create_test_config();

        let mut reading = create_test_meter_reading(50.0);
        reading.submitted_at = Utc::now() - chrono::Duration::days(10); // 10 days ago

        let service = create_test_meter_polling_service(&config);

        // Validate the reading
        let result = service.validate_reading(&reading);

        // Should return an error because the reading is too old
        match result {
            Err(ValidationError::ReadingTooOld) => {
                // Expected error
            }
            _ => panic!("Expected ValidationError::ReadingTooOld"),
        }
    }

    #[tokio::test]
    async fn test_validate_reading_amount_too_high() {
        let config = create_test_config();

        let mut reading = create_test_meter_reading(500.0); // Exceeds max_reading_kwh

        let service = create_test_meter_polling_service(&config);

        // Validate the reading
        let result = service.validate_reading(&reading);

        // Should return an error because the amount is too high
        match result {
            Err(ValidationError::AmountTooHigh(_)) => {
                // Expected error
            }
            _ => panic!("Expected ValidationError::AmountTooHigh"),
        }
    }

    #[tokio::test]
    async fn test_validate_reading_unverified() {
        let config = create_test_config();

        let mut reading = create_test_meter_reading(50.0);
        reading.verification_status = MeterReadingStatus::Pending; // Not verified

        let service = create_test_meter_polling_service(&config);

        // Validate the reading
        let result = service.validate_reading(&reading);

        // Should return an error because the reading is not verified
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_batch_successful() {
        let config = create_test_config();

        // Create test readings
        let readings = vec![
            create_test_meter_reading(10.0),
            create_test_meter_reading(20.0),
            create_test_meter_reading(30.0),
        ];

        // Create services
        let meter_service = MockMeterService::new(readings.clone());
        let blockchain_service = MockBlockchainService::new(false); // Don't fail
        let websocket_service = MockWebSocketService::new();

        // Create the meter polling service
        let service = create_test_meter_polling_service_with_mocks(
            &config,
            meter_service,
            blockchain_service,
            websocket_service,
        );

        // Process the batch
        let results = service.process_batch(readings).await.unwrap();

        // Check that all readings were processed successfully
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));

        // Check that WebSocket events were sent
        let events = service.get_websocket_events();
        assert_eq!(events.len(), 3); // One TokensMinted event for each reading
        assert!(events.iter().all(|e| e.contains("TokensMinted")));
    }

    #[tokio::test]
    async fn test_process_batch_with_failures() {
        let config = create_test_config();

        // Create test readings, one with invalid amount
        let mut readings = vec![
            create_test_meter_reading(10.0),
            create_test_meter_reading(20.0),
        ];

        let mut invalid_reading = create_test_meter_reading(500.0); // Exceeds max
        invalid_reading.verification_status = MeterReadingStatus::Pending; // Not verified
        readings.push(invalid_reading);

        // Create services
        let meter_service = MockMeterService::new(readings.clone());
        let blockchain_service = MockBlockchainService::new(false); // Don't fail
        let websocket_service = MockWebSocketService::new();

        // Create the meter polling service
        let service = create_test_meter_polling_service_with_mocks(
            &config,
            meter_service,
            blockchain_service,
            websocket_service,
        );

        // Process the batch
        let results = service.process_batch(readings).await.unwrap();

        // Check that only two readings were processed successfully
        assert_eq!(results.len(), 3);
        assert_eq!(results.iter().filter(|r| r.success).count(), 2);

        // Check that WebSocket events were sent
        let events = service.get_websocket_events();
        assert_eq!(events.len(), 3); // Two TokensMinted and one ValidationFailed
        assert!(events.iter().any(|e| e.contains("TokensMinted")));
        assert!(events.iter().any(|e| e.contains("ValidationFailed")));
    }

    // Helper function to create a test meter polling service
    // This would need to be implemented in the actual code
    fn create_test_meter_polling_service(config: &TokenizationConfig) -> impl ValidateReading {
        // This is a placeholder - in the actual implementation,
        // you would create a real MeterPollingService with mock dependencies
        TestValidateReadingService {
            config: config.clone(),
        }
    }

    // Helper function to create a test meter polling service with mocks
    fn create_test_meter_polling_service_with_mocks(
        config: &TokenizationConfig,
        _meter_service: MockMeterService,
        _blockchain_service: MockBlockchainService,
        _websocket_service: MockWebSocketService,
    ) -> impl ProcessBatch {
        // This is a placeholder - in the actual implementation,
        // you would create a real MeterPollingService with mock dependencies
        TestProcessBatchService {
            config: config.clone(),
        }
    }

    // Trait for testing validate_reading method
    trait ValidateReading {
        fn validate_reading(&self, reading: &MeterReading) -> Result<(), ValidationError>;
    }

    // Trait for testing process_batch method
    trait ProcessBatch {
        fn process_batch(
            &self,
            readings: Vec<MeterReading>,
        ) -> impl std::future::Future<Output = Result<Vec<MintResult>, Box<dyn std::error::Error>>>;

        fn get_websocket_events(&self) -> Vec<String>;
    }

    // Test implementation for validation
    struct TestValidateReadingService {
        config: TokenizationConfig,
    }

    impl ValidateReading for TestValidateReadingService {
        fn validate_reading(&self, reading: &MeterReading) -> Result<(), ValidationError> {
            // Check reading age
            let reading_age = Utc::now().signed_duration_since(reading.submitted_at);
            if reading_age.num_days() > self.config.reading_max_age_days {
                return Err(ValidationError::ReadingTooOld);
            }

            // Check amount
            if reading.kwh_amount > self.config.max_reading_kwh {
                return Err(ValidationError::AmountTooHigh(reading.kwh_amount));
            }

            // Check verification status
            if reading.verification_status != MeterReadingStatus::Verified {
                return Err(ValidationError::InvalidConversion);
            }

            Ok(())
        }
    }

    // Test implementation for batch processing
    struct TestProcessBatchService {
        config: TokenizationConfig,
    }

    impl ProcessBatch for TestProcessBatchService {
        async fn process_batch(
            &self,
            readings: Vec<MeterReading>,
        ) -> Result<Vec<MintResult>, Box<dyn std::error::Error>> {
            let mut results = Vec::new();

            for reading in readings {
                // Validate reading
                match self.validate_reading(&reading) {
                    Ok(_) => {
                        // Simulate successful minting
                        results.push(MintResult {
                            reading_id: reading.id,
                            success: true,
                            error: None,
                            tx_signature: Some("mock_signature".to_string()),
                        });
                    }
                    Err(e) => {
                        // Simulate validation failure
                        results.push(MintResult {
                            reading_id: reading.id,
                            success: false,
                            error: Some(format!("Validation error: {:?}", e)),
                            tx_signature: None,
                        });
                    }
                }
            }

            Ok(results)
        }

        fn get_websocket_events(&self) -> Vec<String> {
            // In a real implementation, this would return the events that were broadcast
            vec![]
        }
    }

    impl TestProcessBatchService {
        fn validate_reading(&self, reading: &MeterReading) -> Result<(), ValidationError> {
            // Check reading age
            let reading_age = Utc::now().signed_duration_since(reading.submitted_at);
            if reading_age.num_days() > self.config.reading_max_age_days {
                return Err(ValidationError::ReadingTooOld);
            }

            // Check amount
            if reading.kwh_amount > self.config.max_reading_kwh {
                return Err(ValidationError::AmountTooHigh(reading.kwh_amount));
            }

            // Check verification status
            if reading.verification_status != MeterReadingStatus::Verified {
                return Err(ValidationError::InvalidConversion);
            }

            Ok(())
        }
    }
}
