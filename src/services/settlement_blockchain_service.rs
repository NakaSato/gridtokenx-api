use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use anchor_lang::prelude::system_instruction;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::services::blockchain_service::BlockchainService;
use crate::services::batch_transaction_service::{BatchTransactionService, BatchConfig, TransactionPriority};
use crate::services::market_clearing_service::Settlement;

/// Status of a blockchain settlement transaction
#[derive(Debug, Clone, PartialEq)]
pub enum SettlementTransactionStatus {
    Pending,
    Submitted,
    Confirmed,
    Failed,
    Expired,
}

impl ToString for SettlementTransactionStatus {
    fn to_string(&self) -> String {
        match self {
            SettlementTransactionStatus::Pending => "pending".to_string(),
            SettlementTransactionStatus::Submitted => "submitted".to_string(),
            SettlementTransactionStatus::Confirmed => "confirmed".to_string(),
            SettlementTransactionStatus::Failed => "failed".to_string(),
            SettlementTransactionStatus::Expired => "expired".to_string(),
        }
    }
}

/// Blockchain transaction record for a settlement
#[derive(Debug, Clone)]
pub struct SettlementTransaction {
    pub id: Uuid,
    pub settlement_id: Uuid,
    pub transaction_signature: Option<String>,
    pub status: SettlementTransactionStatus,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Service for handling blockchain transactions for settlements
#[derive(Clone)]
pub struct SettlementBlockchainService {
    blockchain_service: BlockchainService,
    db: PgPool,
    payer_keypair: Arc<Keypair>,
    max_retries: u32,
    
    // Batch service integration
    batch_service: Option<Arc<BatchTransactionService>>,
    batching_enabled: bool,
}

impl SettlementBlockchainService {
    /// Create a new settlement blockchain service (without batching)
    pub fn new(
        blockchain_service: BlockchainService,
        db: PgPool,
        payer_keypair: Keypair,
    ) -> Self {
        Self {
            blockchain_service,
            db,
            payer_keypair: Arc::new(payer_keypair),
            max_retries: 3,
            batch_service: None,
            batching_enabled: false,
        }
    }
    
    /// Create a new settlement blockchain service with batching support
    pub fn new_with_batching(
        blockchain_service: BlockchainService,
        db: PgPool,
        payer_keypair: Keypair,
        batch_config: BatchConfig,
    ) -> Self {
        let payer_arc = Arc::new(payer_keypair);
        let blockchain_arc = Arc::new(blockchain_service.clone());
        
        let batch_service = BatchTransactionService::new(
            db.clone(),
            blockchain_arc,
            payer_arc.clone(),
            batch_config,
        );
        
        Self {
            blockchain_service,
            db,
            payer_keypair: payer_arc,
            max_retries: 3,
            batch_service: Some(Arc::new(batch_service)),
            batching_enabled: true,
        }
    }

    /// Process a settlement by creating and submitting blockchain transaction
    pub async fn process_settlement(&self, settlement: &Settlement) -> Result<SettlementTransaction> {
        info!("Processing settlement {} for blockchain transaction", settlement.id);
        
        // Check if batching is enabled
        if self.batching_enabled {
            if let Some(batch_service) = &self.batch_service {
                // Add to batch with normal priority
                info!("Adding settlement {} to batch", settlement.id);
                let settlement_tx_id = batch_service
                    .add_to_batch(settlement, TransactionPriority::Normal)
                    .await?;
                
                // Return settlement transaction record
                return Ok(SettlementTransaction {
                    id: settlement_tx_id,
                    settlement_id: settlement.id,
                    transaction_signature: None,
                    status: SettlementTransactionStatus::Pending,
                    retry_count: 0,
                    error_message: None,
                    submitted_at: None,
                    confirmed_at: None,
                });
            }
        }
        
        // Fall back to direct submission
        self.process_settlement_direct(settlement).await
    }
    
    /// Process a settlement directly without batching
    async fn process_settlement_direct(&self, settlement: &Settlement) -> Result<SettlementTransaction> {
        info!("Processing settlement {} directly (no batching)", settlement.id);

        // Check if transaction already exists
        if let Some(existing_tx) = self.get_settlement_transaction(settlement.id).await? {
            if existing_tx.status == SettlementTransactionStatus::Confirmed {
                info!("Settlement {} already confirmed", settlement.id);
                return Ok(existing_tx);
            }
            
            if existing_tx.status == SettlementTransactionStatus::Pending 
                || existing_tx.status == SettlementTransactionStatus::Failed {
                // Retry existing transaction
                return self.retry_settlement_transaction(&existing_tx, settlement).await;
            }

            return Ok(existing_tx);
        }

        // Create new settlement transaction record
        let tx_record = self.create_settlement_transaction_record(settlement.id).await?;

        // Build and submit blockchain transaction
        match self.submit_settlement_to_blockchain(settlement).await {
            Ok(signature) => {
                info!("Settlement transaction submitted: {}", signature);
                self.update_transaction_status(
                    tx_record.id,
                    SettlementTransactionStatus::Submitted,
                    Some(signature.to_string()),
                    None,
                ).await?;

                // Start monitoring for confirmation (async)
                let service = self.clone();
                let tx_id = tx_record.id;
                let sig = signature.to_string();
                tokio::spawn(async move {
                    if let Err(e) = service.monitor_transaction_confirmation(tx_id, sig).await {
                        error!("Failed to monitor transaction confirmation: {}", e);
                    }
                });

                Ok(tx_record)
            }
            Err(e) => {
                error!("Failed to submit settlement transaction: {}", e);
                self.update_transaction_status(
                    tx_record.id,
                    SettlementTransactionStatus::Failed,
                    None,
                    Some(e.to_string()),
                ).await?;
                Err(e)
            }
        }
    }

    /// Submit settlement transaction to blockchain
    async fn submit_settlement_to_blockchain(&self, settlement: &Settlement) -> Result<Signature> {
        info!("Building settlement transaction for settlement {}", settlement.id);

        // Get buyer and seller public keys from database
        let buyer_pubkey = self.get_user_wallet_pubkey(settlement.buyer_id).await?;
        let seller_pubkey = self.get_user_wallet_pubkey(settlement.seller_id).await?;

        // Convert amounts to lamports (1 SOL = 1_000_000_000 lamports)
        // For simplicity, we'll treat 1 token = 1 lamport * 1000 for precision
        let amount_lamports = self.bigdecimal_to_lamports(&settlement.net_amount)?;

        info!(
            "Settlement: {} kWh at {} per kWh, total {} lamports",
            settlement.energy_amount, settlement.price_per_kwh, amount_lamports
        );

        // Convert Pubkey to bytes and back to anchor Pubkey
        let buyer_anchor = anchor_lang::prelude::Pubkey::new_from_array(buyer_pubkey.to_bytes());
        let seller_anchor = anchor_lang::prelude::Pubkey::new_from_array(seller_pubkey.to_bytes());
        
        // Create transfer instruction (simplified - in production this would call trading program)
        let anchor_instruction = system_instruction::transfer(
            &buyer_anchor,
            &seller_anchor,
            amount_lamports,
        );
        
        // Convert anchor instruction to solana_sdk instruction
        let transfer_instruction = Instruction {
            program_id: Pubkey::new_from_array(anchor_instruction.program_id.to_bytes()),
            accounts: anchor_instruction.accounts.iter().map(|acc| {
                AccountMeta {
                    pubkey: Pubkey::new_from_array(acc.pubkey.to_bytes()),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                }
            }).collect(),
            data: anchor_instruction.data,
        };

        // For production: This should call the trading program's settle_trade instruction
        // let settle_instruction = self.build_settle_trade_instruction(settlement)?;

        // Get recent blockhash
        let recent_blockhash = self.blockchain_service.get_latest_blockhash()?;

        // Create and sign transaction
        let mut transaction = Transaction::new_with_payer(
            &[transfer_instruction],
            Some(&self.payer_keypair.pubkey()),
        );

        transaction.sign(&[&*self.payer_keypair], recent_blockhash);

        // Send transaction
        let signature = self.blockchain_service.send_and_confirm_transaction(&transaction)?;

        info!("Settlement transaction confirmed: {}", signature);

        Ok(signature)
    }

    /// Build settle trade instruction (for production use with trading program)
    #[allow(dead_code)]
    fn build_settle_trade_instruction(&self, settlement: &Settlement) -> Result<Instruction> {
        let trading_program_id = BlockchainService::parse_pubkey(
            crate::services::blockchain_service::TRADING_PROGRAM_ID
        )?;

        // Convert settlement data to instruction data
        let amount_u64 = self.bigdecimal_to_u64(&settlement.energy_amount)?;
        let price_u64 = self.bigdecimal_to_u64(&settlement.price_per_kwh)?;

        // Serialize instruction data (simplified - should use proper Anchor IDL)
        let mut instruction_data = vec![0x01]; // Discriminator for settle_trade
        instruction_data.extend_from_slice(&amount_u64.to_le_bytes());
        instruction_data.extend_from_slice(&price_u64.to_le_bytes());

        // Get accounts
        let buyer_pubkey = Pubkey::new_unique(); // Should get from DB
        let seller_pubkey = Pubkey::new_unique(); // Should get from DB

        let accounts = vec![
            AccountMeta::new(buyer_pubkey, false),
            AccountMeta::new(seller_pubkey, false),
            AccountMeta::new(self.payer_keypair.pubkey(), true),
        ];

        Ok(Instruction::new_with_bytes(
            trading_program_id,
            &instruction_data,
            accounts,
        ))
    }

    /// Retry a failed settlement transaction
    async fn retry_settlement_transaction(
        &self,
        tx_record: &SettlementTransaction,
        settlement: &Settlement,
    ) -> Result<SettlementTransaction> {
        if tx_record.retry_count >= self.max_retries as i32 {
            warn!("Max retries reached for settlement transaction {}", tx_record.id);
            return Err(anyhow!("Max retries exceeded"));
        }

        info!("Retrying settlement transaction {} (attempt {})", tx_record.id, tx_record.retry_count + 1);

        // Increment retry count
        sqlx::query!(
            "UPDATE settlement_transactions SET retry_count = retry_count + 1 WHERE id = $1",
            tx_record.id
        )
        .execute(&self.db)
        .await?;

        // Retry submission
        match self.submit_settlement_to_blockchain(settlement).await {
            Ok(signature) => {
                self.update_transaction_status(
                    tx_record.id,
                    SettlementTransactionStatus::Submitted,
                    Some(signature.to_string()),
                    None,
                ).await?;

                Ok(tx_record.clone())
            }
            Err(e) => {
                self.update_transaction_status(
                    tx_record.id,
                    SettlementTransactionStatus::Failed,
                    None,
                    Some(e.to_string()),
                ).await?;
                Err(e)
            }
        }
    }

    /// Monitor transaction confirmation status
    async fn monitor_transaction_confirmation(
        &self,
        tx_id: Uuid,
        signature_str: String,
    ) -> Result<()> {
        info!("Monitoring transaction confirmation for {}", signature_str);

        let signature = Signature::from_str(&signature_str)?;
        let max_attempts = 30; // 30 attempts * 2 seconds = 60 seconds timeout
        let mut attempts = 0;

        loop {
            attempts += 1;

            match self.blockchain_service.get_signature_status(&signature)? {
                Some(success) => {
                    if success {
                        info!("Transaction {} confirmed successfully", signature_str);
                        self.update_transaction_status(
                            tx_id,
                            SettlementTransactionStatus::Confirmed,
                            Some(signature_str.clone()),
                            None,
                        ).await?;

                        // Update settlement status
                        sqlx::query!(
                            "UPDATE settlements SET status = 'completed' WHERE id = (
                                SELECT settlement_id FROM settlement_transactions WHERE id = $1
                            )",
                            tx_id
                        )
                        .execute(&self.db)
                        .await?;

                        return Ok(());
                    } else {
                        error!("Transaction {} failed on-chain", signature_str);
                        self.update_transaction_status(
                            tx_id,
                            SettlementTransactionStatus::Failed,
                            Some(signature_str),
                            Some("Transaction failed on-chain".to_string()),
                        ).await?;
                        return Err(anyhow!("Transaction failed on-chain"));
                    }
                }
                None => {
                    if attempts >= max_attempts {
                        warn!("Transaction confirmation timeout for {}", signature_str);
                        self.update_transaction_status(
                            tx_id,
                            SettlementTransactionStatus::Expired,
                            Some(signature_str),
                            Some("Confirmation timeout".to_string()),
                        ).await?;
                        return Err(anyhow!("Transaction confirmation timeout"));
                    }

                    debug!("Transaction not confirmed yet, attempt {}/{}", attempts, max_attempts);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }
    }

    /// Create settlement transaction record in database
    async fn create_settlement_transaction_record(&self, settlement_id: Uuid) -> Result<SettlementTransaction> {
        let tx_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO settlement_transactions (id, settlement_id, status, retry_count)
            VALUES ($1, $2, 'pending', 0)
            "#,
            tx_id,
            settlement_id
        )
        .execute(&self.db)
        .await?;

        Ok(SettlementTransaction {
            id: tx_id,
            settlement_id,
            transaction_signature: None,
            status: SettlementTransactionStatus::Pending,
            retry_count: 0,
            error_message: None,
            submitted_at: None,
            confirmed_at: None,
        })
    }

    /// Get settlement transaction from database
    async fn get_settlement_transaction(&self, settlement_id: Uuid) -> Result<Option<SettlementTransaction>> {
        let record = sqlx::query!(
            r#"
            SELECT id, settlement_id, transaction_signature, status, retry_count, 
                   error_message, submitted_at, confirmed_at
            FROM settlement_transactions
            WHERE settlement_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            settlement_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(record.map(|r| SettlementTransaction {
            id: r.id,
            settlement_id: r.settlement_id,
            transaction_signature: r.transaction_signature,
            status: match r.status.as_str() {
                "pending" => SettlementTransactionStatus::Pending,
                "submitted" => SettlementTransactionStatus::Submitted,
                "confirmed" => SettlementTransactionStatus::Confirmed,
                "failed" => SettlementTransactionStatus::Failed,
                "expired" => SettlementTransactionStatus::Expired,
                _ => SettlementTransactionStatus::Pending,
            },
            retry_count: r.retry_count,
            error_message: r.error_message,
            submitted_at: r.submitted_at,
            confirmed_at: r.confirmed_at,
        }))
    }

    /// Update transaction status in database
    async fn update_transaction_status(
        &self,
        tx_id: Uuid,
        status: SettlementTransactionStatus,
        signature: Option<String>,
        error: Option<String>,
    ) -> Result<()> {
        let status_str = status.to_string();

        sqlx::query!(
            r#"
            UPDATE settlement_transactions
            SET status = $1::text,
                transaction_signature = COALESCE($2, transaction_signature),
                error_message = $3,
                submitted_at = CASE WHEN $1::text = 'submitted' THEN NOW() ELSE submitted_at END,
                confirmed_at = CASE WHEN $1::text = 'confirmed' THEN NOW() ELSE confirmed_at END,
                updated_at = NOW()
            WHERE id = $4
            "#,
            status_str,
            signature,
            error,
            tx_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get user wallet public key from database
    async fn get_user_wallet_pubkey(&self, user_id: Uuid) -> Result<Pubkey> {
        let user = sqlx::query!(
            "SELECT wallet_address FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(&self.db)
        .await?;

        if let Some(ref wallet_address) = user.wallet_address {
            Pubkey::from_str(wallet_address)
                .map_err(|e| anyhow!("Invalid wallet address: {}", e))
        } else {
            Err(anyhow!("User has no wallet address"))
        }
    }

    /// Convert BigDecimal to lamports (u64)
    fn bigdecimal_to_lamports(&self, value: &BigDecimal) -> Result<u64> {
        // Multiply by 1_000_000_000 to convert to lamports, then convert to u64
        let lamports_decimal = value * BigDecimal::from(1_000_000_000);
        let lamports_str = lamports_decimal.to_string();
        
        // Parse the integer part only
        let int_part: String = lamports_str.chars()
            .take_while(|c| c.is_numeric())
            .collect();

        int_part.parse::<u64>()
            .map_err(|e| anyhow!("Failed to convert to lamports: {}", e))
    }

    /// Convert BigDecimal to u64 (for instruction data)
    fn bigdecimal_to_u64(&self, value: &BigDecimal) -> Result<u64> {
        let value_str = value.to_string();
        let int_part: String = value_str.chars()
            .take_while(|c| c.is_numeric())
            .collect();

        int_part.parse::<u64>()
            .map_err(|e| anyhow!("Failed to convert to u64: {}", e))
    }

    /// Process all pending settlements in batch
    pub async fn process_pending_settlements(&self) -> Result<Vec<SettlementTransaction>> {
        info!("Processing all pending settlements");

        // Get all pending settlements
        let pending_settlements = sqlx::query_as!(
            Settlement,
            r#"
            SELECT id, epoch_id, buyer_id, seller_id, energy_amount,
                   price_per_kwh, total_amount, fee_amount, net_amount, status
            FROM settlements
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT 100
            "#
        )
        .fetch_all(&self.db)
        .await?;

        info!("Found {} pending settlements to process", pending_settlements.len());

        let mut results = Vec::new();

        for settlement in pending_settlements {
            match self.process_settlement(&settlement).await {
                Ok(tx) => {
                    results.push(tx);
                }
                Err(e) => {
                    error!("Failed to process settlement {}: {}", settlement.id, e);
                    // Continue with next settlement
                }
            }

            // Small delay to avoid overwhelming the network
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        info!("Processed {} settlements", results.len());

        Ok(results)
    }

    /// Get transaction status for a settlement
    pub async fn get_settlement_transaction_status(&self, settlement_id: Uuid) -> Result<Option<SettlementTransaction>> {
        self.get_settlement_transaction(settlement_id).await
    }

    /// Get all failed transactions that can be retried
    pub async fn get_retriable_transactions(&self) -> Result<Vec<SettlementTransaction>> {
        let records = sqlx::query!(
            r#"
            SELECT id, settlement_id, transaction_signature, status, retry_count,
                   error_message, submitted_at, confirmed_at
            FROM settlement_transactions
            WHERE status = 'failed' AND retry_count < $1
            ORDER BY created_at ASC
            LIMIT 50
            "#,
            self.max_retries as i32
        )
        .fetch_all(&self.db)
        .await?;

        Ok(records.into_iter().map(|r| SettlementTransaction {
            id: r.id,
            settlement_id: r.settlement_id,
            transaction_signature: r.transaction_signature,
            status: SettlementTransactionStatus::Failed,
            retry_count: r.retry_count,
            error_message: r.error_message,
            submitted_at: r.submitted_at,
            confirmed_at: r.confirmed_at,
        }).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_service() -> SettlementBlockchainService {
        let rpc_url = env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "http://localhost:8899".to_string());
        
        let blockchain_service = BlockchainService::new(
            rpc_url,
            "localnet".to_string(),
        ).unwrap();

        // Use lazy connection for tests
        let database_url = env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/gridtokenx_test".to_string());
        let db = PgPool::connect_lazy(&database_url).unwrap();
        
        let keypair = Keypair::new();

        SettlementBlockchainService::new(blockchain_service, db, keypair)
    }

    #[test]
    fn test_bigdecimal_to_lamports() {
        let service = create_test_service();
        
        // Test 1 SOL = 1_000_000_000 lamports
        let one_sol = BigDecimal::from_str("1.0").unwrap();
        let lamports = service.bigdecimal_to_lamports(&one_sol).unwrap();
        assert_eq!(lamports, 1_000_000_000);

        // Test 0.5 SOL = 500_000_000 lamports
        let half_sol = BigDecimal::from_str("0.5").unwrap();
        let lamports = service.bigdecimal_to_lamports(&half_sol).unwrap();
        assert_eq!(lamports, 500_000_000);

        // Test 0.000000001 SOL = 1 lamport
        let one_lamport = BigDecimal::from_str("0.000000001").unwrap();
        let lamports = service.bigdecimal_to_lamports(&one_lamport).unwrap();
        assert_eq!(lamports, 1);
    }
}
