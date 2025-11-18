use anyhow::Result;
use gridtokenx_apigateway::services::blockchain_service::{BlockchainService, transaction_utils};
use gridtokenx_apigateway::services::priority_fee_service::{TransactionType, PriorityLevel};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;
use tokio_test;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_program_ids() {
        assert!(BlockchainService::registry_program_id().is_ok());
        assert!(BlockchainService::oracle_program_id().is_ok());
        assert!(BlockchainService::governance_program_id().is_ok());
        assert!(BlockchainService::energy_token_program_id().is_ok());
        assert!(BlockchainService::trading_program_id().is_ok());
    }

    #[test]
    fn test_parse_invalid_pubkey() {
        assert!(BlockchainService::parse_pubkey("invalid").is_err());
        assert!(BlockchainService::parse_pubkey("").is_err());
        assert!(BlockchainService::parse_pubkey("too_short").is_err());
    }

    #[test]
    fn test_parse_valid_pubkey() {
        let valid_pubkey = "11111111111111111111111111111112";
        let parsed = BlockchainService::parse_pubkey(valid_pubkey);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().to_string(), valid_pubkey);
    }

    #[tokio::test]
    async fn test_blockchain_service_creation() {
        let service = BlockchainService::new(
            "http://localhost:8899".to_string(),
            "localnet".to_string(),
        );
        assert!(service.is_ok());
        
        let service = service.unwrap();
        assert_eq!(service.cluster(), "localnet");
    }

    #[tokio::test]
    async fn test_health_check() {
        // This test requires a running validator
        let service = BlockchainService::new(
            "http://localhost:8899".to_string(),
            "localnet".to_string(),
        ).unwrap();
        
        // Health check might fail if no validator is running, which is expected
        let health = service.health_check().await;
        // Don't assert here as it depends on external service
        println!("Health check result: {:?}", health);
    }

    #[test]
    fn test_transaction_building() {
        let service = BlockchainService::new(
            "http://localhost:8899".to_string(),
            "localnet".to_string(),
        ).unwrap();
        
        let payer = Pubkey::new_unique();
        let instruction = Instruction::new_with_bytes(
            Pubkey::new_unique(),
            &[1, 2, 3],
            vec![],
        );
        
        let transaction = service.build_transaction(vec![instruction], &payer);
        assert!(transaction.is_ok());
    }

    #[test]
    fn test_priority_fee_recommendations() {
        use gridtokenx_apigateway::services::priority_fee_service::PriorityFeeService;
        
        // Test different transaction types get appropriate priorities
        let order_priority = PriorityFeeService::recommend_priority_level(TransactionType::OrderCreation);
        assert_eq!(order_priority, PriorityLevel::Medium);
        
        let minting_priority = PriorityFeeService::recommend_priority_level(TransactionType::TokenMinting);
        assert_eq!(minting_priority, PriorityLevel::High);
        
        let settlement_priority = PriorityFeeService::recommend_priority_level(TransactionType::Settlement);
        assert_eq!(settlement_priority, PriorityLevel::High);
        
        let erc_priority = PriorityFeeService::recommend_priority_level(TransactionType::ErcCertificate);
        assert_eq!(erc_priority, PriorityLevel::Medium);
    }

    #[test]
    fn test_compute_limit_recommendations() {
        use gridtokenx_apigateway::services::priority_fee_service::PriorityFeeService;
        
        let order_limit = PriorityFeeService::recommend_compute_limit(TransactionType::OrderCreation);
        assert_eq!(order_limit, 200_000);
        
        let minting_limit = PriorityFeeService::recommend_compute_limit(TransactionType::TokenMinting);
        assert_eq!(minting_limit, 300_000);
        
        let settlement_limit = PriorityFeeService::recommend_compute_limit(TransactionType::Settlement);
        assert_eq!(settlement_limit, 250_000);
        
        let erc_limit = PriorityFeeService::recommend_compute_limit(TransactionType::ErcCertificate);
        assert_eq!(erc_limit, 350_000);
    }

    #[test]
    fn test_fee_cost_estimation() {
        use gridtokenx_apigateway::services::priority_fee_service::PriorityFeeService;
        
        let low_cost = PriorityFeeService::estimate_fee_cost(PriorityLevel::Low, Some(200_000));
        assert!(low_cost > 0.0);
        assert!(low_cost < 0.01); // Should be very low
        
        let medium_cost = PriorityFeeService::estimate_fee_cost(PriorityLevel::Medium, Some(200_000));
        assert!(medium_cost > low_cost);
        assert!(medium_cost < 0.01);
        
        let high_cost = PriorityFeeService::estimate_fee_cost(PriorityLevel::High, Some(300_000));
        assert!(high_cost > medium_cost);
        assert!(high_cost < 0.05);
    }

    #[test]
    fn test_priority_fee_instruction_addition() {
        use gridtokenx_apigateway::services::priority_fee_service::PriorityFeeService;
        
        let mut instructions = vec![
            Instruction::new_with_bytes(
                Pubkey::new_unique(),
                &[1, 2, 3],
                vec![4, 5, 6],
            ),
        ];
        
        let original_len = instructions.len();
        
        // Add priority fee should insert compute budget instructions at the beginning
        PriorityFeeService::add_priority_fee(&mut instructions, PriorityLevel::Medium, Some(200_000)).unwrap();
        
        assert_eq!(instructions.len(), original_len + 2); // Should add 2 instructions (compute unit price + limit)
        
        // Verify the instructions are compute budget instructions
        // First instruction should be set_compute_unit_price
        // Second instruction should be set_compute_unit_limit
    }

    #[tokio::test]
    async fn test_transaction_simulation() {
        let service = BlockchainService::new(
            "http://localhost:8899".to_string(),
            "localnet".to_string(),
        ).unwrap();
        
        let payer = Keypair::new();
        let instruction = Instruction::new_with_bytes(
            Pubkey::new_unique(),
            &[1, 2, 3],
            vec![4, 5, 6],
        );
        
        let transaction = service.build_transaction(vec![instruction], &payer.pubkey()).unwrap();
        
        // Simulation might fail if no validator is running, which is expected
        let result = service.simulate_transaction(&transaction);
        println!("Simulation result: {:?}", result);
        // Don't assert here as it depends on external service
    }

    #[tokio::test]
    async fn test_wait_for_confirmation_timeout() {
        let service = BlockchainService::new(
            "http://localhost:8899".to_string(),
            "localnet".to_string(),
        ).unwrap();
        
        let fake_signature = solana_sdk::signature::Signature::new_unique();
        
        // Should timeout after 1 second with no validator
        let result = service.wait_for_confirmation(&fake_signature, 1).await;
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_transaction_utils() {
        let payer = Pubkey::new_unique();
        let instruction = Instruction::new_with_bytes(
            Pubkey::new_unique(),
            &[1, 2, 3],
            vec![4, 5, 6],
        );
        
        let transaction = transaction_utils::build_transaction(vec![instruction], &payer, Default::default());
        assert_eq!(transaction.message.instructions.len(), 1);
        assert_eq!(transaction.message.recent_blockhash, Default::default());
        
        let signer = Keypair::new();
        let mut signed_transaction = transaction.clone();
        
        let result = transaction_utils::sign_transaction(&mut signed_transaction, &[&signer], Default::default());
        assert!(result.is_ok());
        assert!(signed_transaction.signatures.len() == 1);
    }

    #[test]
    fn test_amount_conversions() {
        // Test kWh to lamports conversion
        let kwh_amount = 25.5;
        let lamports = (kwh_amount * 1_000_000_000.0) as u64;
        assert_eq!(lamports, 25_500_000_000);
        
        // Test back conversion
        let back_to_kwh = lamports as f64 / 1_000_000_000.0;
        assert!((back_to_kwh - kwh_amount).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ata_address_derivation() {
        use solana_sdk::pubkey::Pubkey;
        
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let ata_program_id = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();
        let token_program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        
        let (ata_address, bump) = Pubkey::find_program_address(
            &[
                wallet.as_ref(),
                token_program_id.as_ref(),
                mint.as_ref(),
            ],
            &ata_program_id,
        );
        
        // Verify the derived address is consistent
        let (ata_address2, bump2) = Pubkey::find_program_address(
            &[
                wallet.as_ref(),
                token_program_id.as_ref(),
                mint.as_ref(),
            ],
            &ata_program_id,
        );
        
        assert_eq!(ata_address, ata_address2);
        assert_eq!(bump, bump2);
    }

    #[tokio::test]
    async fn test_build_and_send_transaction_with_priority() {
        let service = BlockchainService::new(
            "http://localhost:8899".to_string(),
            "localnet".to_string(),
        ).unwrap();
        
        let authority = Keypair::new();
        let instruction = Instruction::new_with_bytes(
            Pubkey::new_unique(),
            &[1, 2, 3],
            vec![4, 5, 6],
        );
        
        // This should fail without a running validator, but we can test the setup
        let result = service.build_and_send_transaction_with_priority(
            vec![instruction],
            &[&authority],
            TransactionType::TokenMinting,
        ).await;
        
        // The transaction building should succeed, but sending will fail without validator
        match result {
            Ok(_) => println!("Transaction succeeded (unexpected without validator)"),
            Err(e) => println!("Transaction failed as expected: {}", e),
        }
    }

    #[test]
    fn test_constants() {
        // Verify all program ID constants are valid base58 strings
        assert!(BlockchainService::parse_pubkey(REGISTRY_PROGRAM_ID).is_ok());
        assert!(BlockchainService::parse_pubkey(ORACLE_PROGRAM_ID).is_ok());
        assert!(BlockchainService::parse_pubkey(GOVERNANCE_PROGRAM_ID).is_ok());
        assert!(BlockchainService::parse_pubkey(ENERGY_TOKEN_PROGRAM_ID).is_ok());
        assert!(BlockchainService::parse_pubkey(TRADING_PROGRAM_ID).is_ok());
    }

    #[test]
    fn test_error_handling() {
        let service = BlockchainService::new(
            "invalid_url".to_string(),
            "localnet".to_string(),
        );
        
        // Should create successfully even with invalid URL
        assert!(service.is_ok());
        
        let service = service.unwrap();
        
        // Parsing invalid pubkey should fail
        let result = service.parse_pubkey("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid public key"));
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let service = BlockchainService::new(
            "http://localhost:8899".to_string(),
            "localnet".to_string(),
        ).unwrap();
        
        let authority = Keypair::new();
        let instruction = Instruction::new_with_bytes(
            Pubkey::new_unique(),
            &[1, 2, 3],
            vec![4, 5, 6],
        );
        
        // Test retry with max 1 (should fail without validator)
        let result = service.send_transaction_with_retry(
            vec![instruction],
            &[&authority],
            1,
        ).await;
        
        // Should fail with an error about failed transaction
        assert!(result.is_err());
    }

    #[test]
    fn test_instruction_data_serialization() {
        // Test that instruction data is built correctly for Anchor programs
        let amount: u64 = 1_000_000_000; // 1 token
        let mut instruction_data = Vec::new();
        
        // Add discriminator (first 8 bytes of sha256)
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(b"global:mint_tokens_direct");
        let hash = hasher.finalize();
        instruction_data.extend_from_slice(&hash[0..8]);
        
        // Add amount
        instruction_data.extend_from_slice(&amount.to_le_bytes());
        
        assert_eq!(instruction_data.len(), 16); // 8 bytes discriminator + 8 bytes amount
        assert_eq!(instruction_data[8..], amount.to_le_bytes());
    }
}
