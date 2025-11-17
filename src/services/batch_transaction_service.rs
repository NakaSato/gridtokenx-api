use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use anchor_lang::prelude::system_instruction;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet, VecDeque};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::services::blockchain_service::BlockchainService;
use crate::services::market_clearing_service::Settlement;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Base fee for a Solana transaction (in lamports)
const BASE_FEE: u64 = 5_000;

/// Maximum instructions per Solana transaction
const MAX_INSTRUCTIONS_PER_TX: usize = 64;

/// Default priority fee per transaction (in lamports)
const DEFAULT_PRIORITY_FEE: u64 = 5_000;

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Configuration for batch transaction processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum number of transactions in a single batch
    pub max_batch_size: usize,
    
    /// Minimum transactions required before submitting batch
    pub min_batch_size: usize,
    
    /// Maximum time to wait for batch to fill (seconds)
    pub max_wait_time: u64,
    
    /// Enable cost-based optimization
    pub cost_optimization: bool,
    
    /// Priority fee per transaction (lamports)
    pub priority_fee: u64,
    
    /// Enable automatic batch submission
    pub auto_submit: bool,
    
    /// Submission interval for scheduler (seconds)
    pub submission_interval: u64,
    
    /// Maximum retry attempts for failed batches
    pub max_retries: u32,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            min_batch_size: 3,
            max_wait_time: 5,
            cost_optimization: true,
            priority_fee: DEFAULT_PRIORITY_FEE,
            auto_submit: true,
            submission_interval: 10,
            max_retries: 3,
        }
    }
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Status of a transaction batch
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum BatchStatus {
    Building,
    Ready,
    Submitted,
    Confirmed,
    Failed,
    Cancelled,
}

impl ToString for BatchStatus {
    fn to_string(&self) -> String {
        match self {
            BatchStatus::Building => "building".to_string(),
            BatchStatus::Ready => "ready".to_string(),
            BatchStatus::Submitted => "submitted".to_string(),
            BatchStatus::Confirmed => "confirmed".to_string(),
            BatchStatus::Failed => "failed".to_string(),
            BatchStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

/// Priority levels for transaction batching
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum TransactionPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Urgent = 4,
}

impl ToString for TransactionPriority {
    fn to_string(&self) -> String {
        match self {
            TransactionPriority::Low => "low".to_string(),
            TransactionPriority::Normal => "normal".to_string(),
            TransactionPriority::High => "high".to_string(),
            TransactionPriority::Urgent => "urgent".to_string(),
        }
    }
}

impl FromStr for TransactionPriority {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "low" => Ok(TransactionPriority::Low),
            "normal" => Ok(TransactionPriority::Normal),
            "high" => Ok(TransactionPriority::High),
            "urgent" => Ok(TransactionPriority::Urgent),
            _ => Err(anyhow!("Invalid priority: {}", s)),
        }
    }
}

/// Represents a batch of settlement transactions
#[derive(Debug, Clone)]
pub struct TransactionBatch {
    /// Unique batch ID
    pub id: Uuid,
    
    /// Settlement transaction IDs in this batch
    pub settlement_tx_ids: Vec<Uuid>,
    
    /// Settlement details for building instructions
    pub settlements: Vec<Settlement>,
    
    /// Current batch status
    pub status: BatchStatus,
    
    /// Blockchain signature (after submission)
    pub signature: Option<String>,
    
    /// Total fee for the batch
    pub total_fee: u64,
    
    /// Retry count
    pub retry_count: u32,
    
    /// When batch was created
    pub created_at: DateTime<Utc>,
    
    /// When batch was submitted
    pub submitted_at: Option<DateTime<Utc>>,
    
    /// When batch was confirmed
    pub confirmed_at: Option<DateTime<Utc>>,
    
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Represents a settlement transaction pending batch inclusion
#[derive(Debug, Clone)]
struct PendingTransaction {
    /// Settlement transaction ID
    settlement_tx_id: Uuid,
    
    /// Settlement details
    settlement: Settlement,
    
    /// Priority level
    priority: TransactionPriority,
    
    /// When transaction was added to pool
    added_at: DateTime<Utc>,
    
    /// Estimated cost if processed individually
    individual_cost: u64,
}

// ============================================================================
// BATCH TRANSACTION SERVICE
// ============================================================================

/// Service for handling batch transaction processing
#[derive(Clone)]
pub struct BatchTransactionService {
    /// Database connection pool
    db: PgPool,
    
    /// Blockchain service for transaction submission
    blockchain_service: Arc<BlockchainService>,
    
    /// Payer keypair for transactions
    payer_keypair: Arc<Keypair>,
    
    /// Batch configuration
    config: BatchConfig,
    
    /// Pending transactions waiting for batch
    pending_pool: Arc<RwLock<Vec<PendingTransaction>>>,
    
    /// Active batches being processed
    active_batches: Arc<RwLock<HashMap<Uuid, TransactionBatch>>>,
    
    /// Scheduler handle for automatic submission
    scheduler_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl BatchTransactionService {
    /// Create a new batch transaction service
    pub fn new(
        db: PgPool,
        blockchain_service: Arc<BlockchainService>,
        payer_keypair: Arc<Keypair>,
        config: BatchConfig,
    ) -> Self {
        let service = Self {
            db,
            blockchain_service,
            payer_keypair,
            config,
            pending_pool: Arc::new(RwLock::new(Vec::new())),
            active_batches: Arc::new(RwLock::new(HashMap::new())),
            scheduler_handle: Arc::new(Mutex::new(None)),
        };
        
        // Start scheduler if auto_submit is enabled
        if service.config.auto_submit {
            let service_clone = service.clone();
            tokio::spawn(async move {
                if let Err(e) = service_clone.start_scheduler().await {
                    error!("Failed to start batch scheduler: {}", e);
                }
            });
        }
        
        service
    }
    
    /// Add a settlement transaction to the batch pool
    pub async fn add_to_batch(
        &self,
        settlement: &Settlement,
        priority: TransactionPriority,
    ) -> Result<Uuid> {
        info!("Adding settlement {} to batch pool with priority {:?}", settlement.id, priority);
        
        // Create settlement transaction record
        let settlement_tx_id = Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO settlement_transactions (id, settlement_id, status, priority, retry_count)
            VALUES ($1, $2, 'pending', $3, 0)
            "#,
            settlement_tx_id,
            settlement.id,
            priority.to_string()
        )
        .execute(&self.db)
        .await?;
        
        // Add to pending pool
        let pending_tx = PendingTransaction {
            settlement_tx_id,
            settlement: settlement.clone(),
            priority,
            added_at: Utc::now(),
            individual_cost: BASE_FEE + self.config.priority_fee,
        };
        
        {
            let mut pool = self.pending_pool.write().await;
            pool.push(pending_tx);
        }
        
        // Check if batch should be submitted immediately
        self.evaluate_batch_submission().await?;
        
        Ok(settlement_tx_id)
    }
    
    /// Evaluate whether a batch should be submitted
    async fn evaluate_batch_submission(&self) -> Result<()> {
        let pending = self.pending_pool.read().await;
        let pending_count = pending.len();
        
        if pending_count == 0 {
            return Ok(());
        }
        
        let should_submit = self.should_submit_batch(&pending).await;
        
        drop(pending); // Release read lock
        
        if should_submit {
            info!("Batch submission criteria met, building and submitting batch");
            self.build_and_submit_batch().await?;
        }
        
        Ok(())
    }
    
    /// Check if batch should be submitted based on strategy
    async fn should_submit_batch(&self, pending: &[PendingTransaction]) -> bool {
        let pending_count = pending.len();
        
        // Always submit if max size reached
        if pending_count >= self.config.max_batch_size {
            debug!("Batch size threshold reached: {}", pending_count);
            return true;
        }
        
        // Check for urgent transactions
        let has_urgent = pending.iter().any(|tx| tx.priority == TransactionPriority::Urgent);
        if has_urgent && pending_count >= self.config.min_batch_size {
            debug!("Urgent transaction found with min size met");
            return true;
        }
        
        // Check time threshold
        if let Some(oldest) = pending.iter().map(|tx| tx.added_at).min() {
            let age = Utc::now() - oldest;
            if age.num_seconds() as u64 >= self.config.max_wait_time && pending_count >= self.config.min_batch_size {
                debug!("Time threshold reached: {} seconds", age.num_seconds());
                return true;
            }
        }
        
        // Check cost optimization
        if self.config.cost_optimization && pending_count >= self.config.min_batch_size {
            let savings = self.calculate_cost_savings(pending);
            if savings >= 50.0 {
                debug!("Cost optimization favorable: {}% savings", savings);
                return true;
            }
        }
        
        false
    }
    
    /// Calculate cost savings percentage for batching
    fn calculate_cost_savings(&self, pending: &[PendingTransaction]) -> f64 {
        let individual_total: u64 = pending.iter().map(|tx| tx.individual_cost).sum();
        
        let batch_cost = BASE_FEE + (self.config.priority_fee * pending.len() as u64);
        
        if individual_total > 0 {
            ((individual_total as f64 - batch_cost as f64) / individual_total as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Build and submit a batch from pending transactions
    pub async fn build_and_submit_batch(&self) -> Result<Uuid> {
        // Take pending transactions
        let pending: Vec<PendingTransaction> = {
            let mut pool = self.pending_pool.write().await;
            
            if pool.is_empty() {
                return Err(anyhow!("No pending transactions to batch"));
            }
            
            // Take up to max_batch_size transactions
            let take_count = pool.len().min(self.config.max_batch_size);
            pool.drain(..take_count).collect()
        };
        
        info!("Building batch with {} transactions", pending.len());
        
        // Optimize order
        let optimized = self.optimize_transaction_order(pending);
        
        // Create batch
        let batch = self.create_batch(optimized).await?;
        
        // Submit batch
        self.submit_batch(batch.id).await?;
        
        Ok(batch.id)
    }
    
    /// Optimize transaction ordering within batch
    fn optimize_transaction_order(&self, mut transactions: Vec<PendingTransaction>) -> Vec<PendingTransaction> {
        // Sort by priority (highest first), then by age (oldest first)
        transactions.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.added_at.cmp(&b.added_at))
        });
        
        transactions
    }
    
    /// Create a batch record in the database
    async fn create_batch(&self, pending: Vec<PendingTransaction>) -> Result<TransactionBatch> {
        let batch_id = Uuid::new_v4();
        let transaction_count = pending.len() as i32;
        let total_fee = BASE_FEE + (self.config.priority_fee * pending.len() as u64);
        
        // Insert batch record
        sqlx::query!(
            r#"
            INSERT INTO batch_transactions (id, transaction_count, total_fee, status, retry_count)
            VALUES ($1, $2, $3, 'ready', 0)
            "#,
            batch_id,
            transaction_count,
            total_fee as i64
        )
        .execute(&self.db)
        .await?;
        
        // Link settlement transactions to batch
        for (index, pending_tx) in pending.iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO batch_transaction_items (batch_id, settlement_transaction_id, position_in_batch)
                VALUES ($1, $2, $3)
                "#,
                batch_id,
                pending_tx.settlement_tx_id,
                index as i32
            )
            .execute(&self.db)
            .await?;
            
            // Update settlement transaction with batch info
            sqlx::query!(
                r#"
                UPDATE settlement_transactions
                SET batch_id = $1, added_to_batch_at = NOW(), updated_at = NOW()
                WHERE id = $2
                "#,
                batch_id,
                pending_tx.settlement_tx_id
            )
            .execute(&self.db)
            .await?;
        }
        
        let batch = TransactionBatch {
            id: batch_id,
            settlement_tx_ids: pending.iter().map(|tx| tx.settlement_tx_id).collect(),
            settlements: pending.iter().map(|tx| tx.settlement.clone()).collect(),
            status: BatchStatus::Ready,
            signature: None,
            total_fee,
            retry_count: 0,
            created_at: Utc::now(),
            submitted_at: None,
            confirmed_at: None,
            error_message: None,
        };
        
        // Store in active batches
        {
            let mut active = self.active_batches.write().await;
            active.insert(batch_id, batch.clone());
        }
        
        Ok(batch)
    }
    
    /// Submit a batch to the blockchain
    pub async fn submit_batch(&self, batch_id: Uuid) -> Result<Signature> {
        info!("Submitting batch {}", batch_id);
        
        // Get batch from active batches or database
        let batch = {
            let active = self.active_batches.read().await;
            active.get(&batch_id).cloned()
        };
        
        let mut batch = match batch {
            Some(b) => b,
            None => self.load_batch_from_db(batch_id).await?,
        };
        
        if batch.status != BatchStatus::Ready {
            return Err(anyhow!("Batch is not ready for submission: {:?}", batch.status));
        }
        
        // Build blockchain transaction
        let transaction = self.build_blockchain_transaction(&batch).await?;
        
        // Submit to blockchain
        match self.blockchain_service.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                info!("Batch {} submitted with signature: {}", batch_id, signature);
                
                // Update batch status
                batch.status = BatchStatus::Submitted;
                batch.signature = Some(signature.to_string());
                batch.submitted_at = Some(Utc::now());
                
                self.update_batch_status(&batch).await?;
                
                // Start monitoring for confirmation
                let service = self.clone();
                let sig = signature.to_string();
                tokio::spawn(async move {
                    if let Err(e) = service.monitor_batch_confirmation(batch_id, sig).await {
                        error!("Failed to monitor batch confirmation: {}", e);
                    }
                });
                
                Ok(signature)
            }
            Err(e) => {
                error!("Failed to submit batch {}: {}", batch_id, e);
                
                batch.status = BatchStatus::Failed;
                batch.error_message = Some(e.to_string());
                
                self.update_batch_status(&batch).await?;
                
                Err(e)
            }
        }
    }
    
    /// Build blockchain transaction from batch
    async fn build_blockchain_transaction(&self, batch: &TransactionBatch) -> Result<Transaction> {
        let mut instructions = Vec::new();
        
        // Build instructions for each settlement
        for settlement in &batch.settlements {
            let instruction = self.build_settlement_instruction(settlement).await?;
            instructions.push(instruction);
        }
        
        if instructions.len() > MAX_INSTRUCTIONS_PER_TX {
            return Err(anyhow!(
                "Batch too large: {} instructions exceeds maximum of {}",
                instructions.len(),
                MAX_INSTRUCTIONS_PER_TX
            ));
        }
        
        // Get recent blockhash
        let recent_blockhash = self.blockchain_service.get_latest_blockhash()?;
        
        // Create and sign transaction
        let mut transaction = Transaction::new_with_payer(
            &instructions,
            Some(&self.payer_keypair.pubkey()),
        );
        
        transaction.sign(&[&*self.payer_keypair], recent_blockhash);
        
        Ok(transaction)
    }
    
    /// Build instruction for a settlement
    async fn build_settlement_instruction(&self, settlement: &Settlement) -> Result<Instruction> {
        // Get buyer and seller public keys
        let buyer_pubkey = self.get_user_wallet_pubkey(settlement.buyer_id).await?;
        let seller_pubkey = self.get_user_wallet_pubkey(settlement.seller_id).await?;
        
        // Convert amount to lamports
        let amount_lamports = self.bigdecimal_to_lamports(&settlement.net_amount)?;
        
        // Convert Pubkey to bytes and back to anchor Pubkey
        let buyer_anchor = anchor_lang::prelude::Pubkey::new_from_array(buyer_pubkey.to_bytes());
        let seller_anchor = anchor_lang::prelude::Pubkey::new_from_array(seller_pubkey.to_bytes());
        
        // Create transfer instruction (simplified - production would use trading program)
        let anchor_instruction = system_instruction::transfer(
            &buyer_anchor,
            &seller_anchor,
            amount_lamports,
        );
        
        // Convert anchor instruction to solana_sdk instruction
        Ok(Instruction {
            program_id: anchor_instruction.program_id.to_bytes().into(),
            accounts: anchor_instruction.accounts.iter().map(|acc| {
                solana_sdk::instruction::AccountMeta {
                    pubkey: Pubkey::new_from_array(acc.pubkey.to_bytes()),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                }
            }).collect(),
            data: anchor_instruction.data,
        })
    }
    
    /// Monitor batch confirmation status
    async fn monitor_batch_confirmation(&self, batch_id: Uuid, signature_str: String) -> Result<()> {
        info!("Monitoring batch {} confirmation", batch_id);
        
        let signature = Signature::from_str(&signature_str)?;
        let max_attempts = 30;
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            match self.blockchain_service.get_signature_status(&signature)? {
                Some(success) => {
                    if success {
                        info!("Batch {} confirmed successfully", batch_id);
                        
                        // Update batch status
                        sqlx::query!(
                            r#"
                            UPDATE batch_transactions
                            SET status = 'confirmed', confirmed_at = NOW(), updated_at = NOW()
                            WHERE id = $1
                            "#,
                            batch_id
                        )
                        .execute(&self.db)
                        .await?;
                        
                        // Update all settlement transactions in batch
                        sqlx::query!(
                            r#"
                            UPDATE settlement_transactions
                            SET status = 'confirmed',
                                transaction_signature = $1,
                                confirmed_at = NOW(),
                                updated_at = NOW()
                            WHERE batch_id = $2
                            "#,
                            signature_str,
                            batch_id
                        )
                        .execute(&self.db)
                        .await?;
                        
                        // Update settlements status
                        sqlx::query!(
                            r#"
                            UPDATE settlements
                            SET status = 'completed'
                            WHERE id IN (
                                SELECT settlement_id FROM settlement_transactions WHERE batch_id = $1
                            )
                            "#,
                            batch_id
                        )
                        .execute(&self.db)
                        .await?;
                        
                        // Remove from active batches
                        {
                            let mut active = self.active_batches.write().await;
                            active.remove(&batch_id);
                        }
                        
                        return Ok(());
                    } else {
                        error!("Batch {} failed on-chain", batch_id);
                        
                        sqlx::query!(
                            r#"
                            UPDATE batch_transactions
                            SET status = 'failed',
                                error_message = 'Transaction failed on-chain',
                                updated_at = NOW()
                            WHERE id = $1
                            "#,
                            batch_id
                        )
                        .execute(&self.db)
                        .await?;
                        
                        return Err(anyhow!("Batch transaction failed on-chain"));
                    }
                }
                None => {
                    if attempts >= max_attempts {
                        warn!("Batch {} confirmation timeout", batch_id);
                        
                        sqlx::query!(
                            r#"
                            UPDATE batch_transactions
                            SET status = 'failed',
                                error_message = 'Confirmation timeout',
                                updated_at = NOW()
                            WHERE id = $1
                            "#,
                            batch_id
                        )
                        .execute(&self.db)
                        .await?;
                        
                        return Err(anyhow!("Batch confirmation timeout"));
                    }
                    
                    debug!("Batch not confirmed yet, attempt {}/{}", attempts, max_attempts);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
    
    /// Start automatic batch submission scheduler
    pub async fn start_scheduler(&self) -> Result<()> {
        let mut handle = self.scheduler_handle.lock().await;
        
        if handle.is_some() {
            warn!("Batch scheduler already running");
            return Ok(());
        }
        
        info!("Starting batch submission scheduler");
        
        let service = self.clone();
        let scheduler_task = tokio::spawn(async move {
            service.run_scheduler().await;
        });
        
        *handle = Some(scheduler_task);
        
        Ok(())
    }
    
    /// Stop automatic batch submission scheduler
    pub async fn stop_scheduler(&self) -> Result<()> {
        let mut handle = self.scheduler_handle.lock().await;
        
        if let Some(task) = handle.take() {
            info!("Stopping batch submission scheduler");
            task.abort();
        }
        
        Ok(())
    }
    
    /// Run scheduler loop
    async fn run_scheduler(&self) {
        let interval = tokio::time::Duration::from_secs(self.config.submission_interval);
        
        loop {
            tokio::time::sleep(interval).await;
            
            debug!("Scheduler tick: evaluating batch submission");
            
            if let Err(e) = self.evaluate_batch_submission().await {
                error!("Scheduler evaluation error: {}", e);
            }
        }
    }
    
    /// Load batch from database
    async fn load_batch_from_db(&self, batch_id: Uuid) -> Result<TransactionBatch> {
        let record = sqlx::query!(
            r#"
            SELECT id, batch_signature, transaction_count, total_fee, status,
                   submitted_at, confirmed_at, error_message, retry_count, created_at
            FROM batch_transactions
            WHERE id = $1
            "#,
            batch_id
        )
        .fetch_one(&self.db)
        .await?;
        
        // Get settlement transaction IDs
        let items = sqlx::query!(
            r#"
            SELECT settlement_transaction_id
            FROM batch_transaction_items
            WHERE batch_id = $1
            ORDER BY position_in_batch
            "#,
            batch_id
        )
        .fetch_all(&self.db)
        .await?;
        
        let settlement_tx_ids: Vec<Uuid> = items.iter().map(|item| item.settlement_transaction_id).collect();
        
        // Note: Settlements are not loaded in this simplified version
        // In production, you'd load the full settlement data here
        
        Ok(TransactionBatch {
            id: record.id,
            settlement_tx_ids,
            settlements: Vec::new(), // Would load from DB in production
            status: match record.status.as_str() {
                "building" => BatchStatus::Building,
                "ready" => BatchStatus::Ready,
                "submitted" => BatchStatus::Submitted,
                "confirmed" => BatchStatus::Confirmed,
                "failed" => BatchStatus::Failed,
                "cancelled" => BatchStatus::Cancelled,
                _ => BatchStatus::Building,
            },
            signature: record.batch_signature,
            total_fee: record.total_fee as u64,
            retry_count: record.retry_count as u32,
            created_at: record.created_at,
            submitted_at: record.submitted_at,
            confirmed_at: record.confirmed_at,
            error_message: record.error_message,
        })
    }
    
    /// Update batch status in database
    async fn update_batch_status(&self, batch: &TransactionBatch) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE batch_transactions
            SET status = $1,
                batch_signature = $2,
                submitted_at = $3,
                confirmed_at = $4,
                error_message = $5,
                updated_at = NOW()
            WHERE id = $6
            "#,
            batch.status.to_string(),
            batch.signature,
            batch.submitted_at,
            batch.confirmed_at,
            batch.error_message,
            batch.id
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
    
    /// Convert BigDecimal to lamports
    fn bigdecimal_to_lamports(&self, value: &bigdecimal::BigDecimal) -> Result<u64> {
        use bigdecimal::BigDecimal;
        
        let lamports_decimal = value * BigDecimal::from(1_000_000_000);
        let lamports_str = lamports_decimal.to_string();
        
        let int_part: String = lamports_str.chars()
            .take_while(|c| c.is_numeric())
            .collect();
        
        int_part.parse::<u64>()
            .map_err(|e| anyhow!("Failed to convert to lamports: {}", e))
    }
    
    /// Get pending transaction count
    pub async fn get_pending_count(&self) -> usize {
        let pool = self.pending_pool.read().await;
        pool.len()
    }
    
    /// Get active batch count
    pub async fn get_active_batch_count(&self) -> usize {
        let active = self.active_batches.read().await;
        active.len()
    }
    
    /// Get batch statistics
    pub async fn get_batch_statistics(&self) -> Result<BatchStatistics> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_batches,
                COUNT(*) FILTER (WHERE status = 'confirmed') as confirmed_batches,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_batches,
                AVG(transaction_count) as avg_batch_size,
                SUM(total_fee) as total_fees,
                AVG(EXTRACT(EPOCH FROM (confirmed_at - submitted_at))) 
                    FILTER (WHERE confirmed_at IS NOT NULL AND submitted_at IS NOT NULL)
                    as avg_confirmation_time
            FROM batch_transactions
            WHERE created_at > NOW() - INTERVAL '24 hours'
            "#
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(BatchStatistics {
            total_batches: stats.total_batches.unwrap_or(0) as u64,
            confirmed_batches: stats.confirmed_batches.unwrap_or(0) as u64,
            failed_batches: stats.failed_batches.unwrap_or(0) as u64,
            avg_batch_size: stats.avg_batch_size
                .and_then(|d| d.to_string().parse::<f64>().ok())
                .unwrap_or(0.0),
            total_fees: stats.total_fees
                .and_then(|f| f.to_string().parse::<u64>().ok())
                .unwrap_or(0),
            avg_confirmation_time: stats.avg_confirmation_time
                .and_then(|d| d.to_string().parse::<f64>().ok())
                .unwrap_or(0.0),
        })
    }
}

// ============================================================================
// STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatistics {
    pub total_batches: u64,
    pub confirmed_batches: u64,
    pub failed_batches: u64,
    pub avg_batch_size: f64,
    pub total_fees: u64,
    pub avg_confirmation_time: f64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_priority_ordering() {
        assert!(TransactionPriority::Urgent > TransactionPriority::High);
        assert!(TransactionPriority::High > TransactionPriority::Normal);
        assert!(TransactionPriority::Normal > TransactionPriority::Low);
    }
    
    #[test]
    fn test_batch_config_defaults() {
        let config = BatchConfig::default();
        assert_eq!(config.max_batch_size, 10);
        assert_eq!(config.min_batch_size, 3);
        assert_eq!(config.max_wait_time, 5);
        assert!(config.cost_optimization);
        assert!(config.auto_submit);
    }
    
    #[test]
    fn test_cost_savings_calculation() {
        let config = BatchConfig::default();
        
        let pending = vec![
            PendingTransaction {
                settlement_tx_id: Uuid::new_v4(),
                settlement: create_test_settlement(),
                priority: TransactionPriority::Normal,
                added_at: Utc::now(),
                individual_cost: 10_000,
            },
            PendingTransaction {
                settlement_tx_id: Uuid::new_v4(),
                settlement: create_test_settlement(),
                priority: TransactionPriority::Normal,
                added_at: Utc::now(),
                individual_cost: 10_000,
            },
        ];
        
        // Individual total: 20,000
        // Batch cost: 5,000 (base) + 10,000 (2 * 5,000 priority) = 15,000
        // Savings: (20,000 - 15,000) / 20,000 * 100 = 25%
        
        let individual_total: u64 = pending.iter().map(|tx| tx.individual_cost).sum();
        let batch_cost = BASE_FEE + (config.priority_fee * pending.len() as u64);
        let savings = ((individual_total as f64 - batch_cost as f64) / individual_total as f64) * 100.0;
        
        assert_eq!(individual_total, 20_000);
        assert_eq!(batch_cost, 15_000);
        assert_eq!(savings, 25.0);
    }
    
    fn create_test_settlement() -> Settlement {
        use bigdecimal::BigDecimal;
        use std::str::FromStr;
        
        Settlement {
            id: Uuid::new_v4(),
            epoch_id: Uuid::new_v4(),
            buyer_id: Uuid::new_v4(),
            seller_id: Uuid::new_v4(),
            energy_amount: BigDecimal::from_str("100.0").unwrap(),
            price_per_kwh: BigDecimal::from_str("0.15").unwrap(),
            total_amount: BigDecimal::from_str("15.0").unwrap(),
            fee_amount: BigDecimal::from_str("0.15").unwrap(),
            net_amount: BigDecimal::from_str("14.85").unwrap(),
            status: "pending".to_string(),
        }
    }
}
