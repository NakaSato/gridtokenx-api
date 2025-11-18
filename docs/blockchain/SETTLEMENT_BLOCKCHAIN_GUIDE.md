# Settlement Blockchain Transaction System

**Created**: November 9, 2025  
**Status**: Implemented and Ready for Testing  
**Phase**: Phase 5 - Trading Platform

---

## 📋 Overview

The Settlement Blockchain Transaction System handles the processing of trading settlements on the Solana blockchain. It provides:

- **Automated settlement processing** for matched trading orders
- **Transaction submission and monitoring** with retry logic
- **Status tracking** for all blockchain transactions
- **Error handling and recovery** mechanisms
- **Batch processing** for multiple settlements

---

## 🏗️ Architecture

### Components

1. **SettlementBlockchainService** (`settlement_blockchain_service.rs`)
   - Main service for processing settlements
   - Handles transaction submission to Solana
   - Manages transaction lifecycle (pending → submitted → confirmed)
   - Implements retry logic for failed transactions

2. **Database Schema** (`settlement_transactions` table)
   - Tracks all settlement blockchain transactions
   - Stores transaction signatures and status
   - Maintains retry count and error messages

3. **Integration Tests** (`settlement_blockchain_tests.rs`)
   - Comprehensive test suite
   - Unit tests for conversions
   - Integration tests for full workflow
   - Mock-friendly design for testing

---

## 💾 Database Schema

### `settlement_transactions` Table

```sql
CREATE TABLE settlement_transactions (
    id UUID PRIMARY KEY,
    settlement_id UUID NOT NULL REFERENCES settlements(id),
    transaction_signature TEXT,
    status VARCHAR(20) NOT NULL,  -- pending, submitted, confirmed, failed, expired
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    submitted_at TIMESTAMPTZ,
    confirmed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Indexes
- `idx_settlement_transactions_settlement_id` - Fast lookup by settlement
- `idx_settlement_transactions_status` - Query by status
- `idx_settlement_transactions_signature` - Lookup by transaction signature
- `idx_settlement_transactions_created_at` - Time-based queries

---

## 🔄 Transaction Lifecycle

### 1. **Pending** (Initial State)
- Settlement created from order match
- Transaction record created in database
- Awaiting blockchain submission

### 2. **Submitted**
- Transaction built and signed
- Submitted to Solana network
- Transaction signature recorded
- Background monitoring started

### 3. **Confirmed**
- Transaction confirmed on blockchain
- Settlement status updated to 'completed'
- User balances updated
- Process complete ✅

### 4. **Failed** (Error States)
- Transaction rejected by network
- Insufficient funds
- Invalid signatures
- Can be retried (up to max_retries)

### 5. **Expired**
- Transaction not confirmed within timeout
- Requires manual investigation
- Can be retried

---

## 🚀 Usage

### Basic Settlement Processing

```rust
use crate::services::{SettlementBlockchainService, BlockchainService};
use solana_sdk::signature::Keypair;
use sqlx::PgPool;

// Initialize service
let blockchain_service = BlockchainService::new(rpc_url, "mainnet-beta".to_string())?;
let db_pool = PgPool::connect(&database_url).await?;
let payer_keypair = Keypair::from_bytes(&keypair_bytes)?;

let settlement_service = SettlementBlockchainService::new(
    blockchain_service,
    db_pool,
    payer_keypair,
);

// Process a single settlement
let settlement = get_settlement_from_db(settlement_id).await?;
let tx_result = settlement_service.process_settlement(&settlement).await?;

println!("Transaction: {:?}", tx_result);
```

### Batch Processing

```rust
// Process all pending settlements
let results = settlement_service.process_pending_settlements().await?;

println!("Processed {} settlements", results.len());
for tx in results {
    println!("  - Settlement {}: {:?}", tx.settlement_id, tx.status);
}
```

### Check Transaction Status

```rust
// Get transaction status for a settlement
let tx_status = settlement_service
    .get_settlement_transaction_status(settlement_id)
    .await?;

if let Some(tx) = tx_status {
    println!("Status: {:?}", tx.status);
    println!("Signature: {:?}", tx.transaction_signature);
    println!("Retry count: {}", tx.retry_count);
}
```

### Retry Failed Transactions

```rust
// Get all transactions that can be retried
let retriable = settlement_service.get_retriable_transactions().await?;

for tx in retriable {
    let settlement = get_settlement_by_id(tx.settlement_id).await?;
    match settlement_service.process_settlement(&settlement).await {
        Ok(_) => println!("Retry successful for {}", tx.id),
        Err(e) => eprintln!("Retry failed: {}", e),
    }
}
```

---

## 🧪 Testing

### Running Tests

```bash
# Run all settlement blockchain tests
cargo test --test settlement_blockchain_tests

# Run specific test
cargo test --test settlement_blockchain_tests test_bigdecimal_conversions

# Run with output
cargo test --test settlement_blockchain_tests -- --nocapture
```

### Test Categories

#### 1. **Unit Tests** (No external dependencies)
- `test_bigdecimal_to_lamports` - Currency conversion
- `test_settlement_transaction_status_updates` - Status transitions
- `test_max_retry_limit` - Retry logic
- `test_duplicate_settlement_processing` - Idempotency

#### 2. **Integration Tests** (Requires database)
- `test_create_settlement_transaction_record` - Database operations
- `test_get_retriable_transactions` - Query operations
- `test_process_pending_settlements_batch` - Batch processing

#### 3. **Full Flow Tests** (Requires Solana validator - marked `#[ignore]`)
- `test_process_settlement_full_flow` - End-to-end blockchain interaction

### Running with Local Validator

```bash
# Start local Solana validator
solana-test-validator

# Run full flow tests
cargo test --test settlement_blockchain_tests -- --ignored --nocapture
```

---

## 🔧 Configuration

### Environment Variables

```bash
# Database
TEST_DATABASE_URL=postgresql://postgres:postgres@localhost/gridtokenx_test
DATABASE_URL=postgresql://postgres:postgres@localhost/gridtokenx

# Solana
SOLANA_RPC_URL=http://localhost:8899  # Local validator
# SOLANA_RPC_URL=https://api.mainnet-beta.solana.com  # Mainnet
# SOLANA_RPC_URL=https://api.devnet.solana.com  # Devnet

# Payer keypair (base58 or file path)
PAYER_KEYPAIR_PATH=/path/to/keypair.json
```

### Service Configuration

```rust
let mut service = SettlementBlockchainService::new(
    blockchain_service,
    db_pool,
    payer_keypair,
);

// Adjust max retries (default: 3)
service.max_retries = 5;
```

---

## 📊 Monitoring & Metrics

### Key Metrics to Track

1. **Transaction Success Rate**
   ```sql
   SELECT 
       status,
       COUNT(*) as count,
       ROUND(COUNT(*) * 100.0 / SUM(COUNT(*)) OVER(), 2) as percentage
   FROM settlement_transactions
   GROUP BY status;
   ```

2. **Average Confirmation Time**
   ```sql
   SELECT 
       AVG(EXTRACT(EPOCH FROM (confirmed_at - submitted_at))) as avg_seconds
   FROM settlement_transactions
   WHERE status = 'confirmed';
   ```

3. **Failed Transactions Needing Retry**
   ```sql
   SELECT COUNT(*) as retriable_count
   FROM settlement_transactions
   WHERE status = 'failed' AND retry_count < 3;
   ```

4. **Pending Settlements**
   ```sql
   SELECT COUNT(*) as pending_count
   FROM settlements
   WHERE status = 'pending';
   ```

### Logging

The service provides detailed logging:
- `INFO`: Normal operations (submission, confirmation)
- `WARN`: Retries, timeout warnings
- `ERROR`: Failed transactions, errors
- `DEBUG`: Detailed transaction monitoring

---

## ⚠️ Error Handling

### Common Errors

1. **Insufficient Funds**
   ```
   Error: Account does not have enough balance
   Solution: Ensure payer account has sufficient SOL
   ```

2. **Invalid Wallet Address**
   ```
   Error: User has no wallet address
   Solution: Ensure users have registered wallet addresses
   ```

3. **Transaction Timeout**
   ```
   Error: Transaction confirmation timeout
   Solution: Check network status, retry transaction
   ```

4. **Network Errors**
   ```
   Error: Failed to send transaction: connection refused
   Solution: Check Solana RPC endpoint is accessible
   ```

### Retry Strategy

- **Max Retries**: 3 attempts (configurable)
- **Retry Conditions**: Failed or expired status
- **Backoff**: 100ms delay between batch settlements
- **Monitoring**: 30 attempts × 2 seconds = 60 second timeout

---

## 🔐 Security Considerations

### 1. **Payer Keypair Protection**
- Store keypair securely (use key management service)
- Never commit keypair to version control
- Rotate keypair periodically
- Use hardware wallet for production

### 2. **Transaction Validation**
- Verify settlement amounts before submission
- Check user wallet addresses are valid
- Validate transaction signatures
- Monitor for suspicious patterns

### 3. **Database Security**
- Encrypt transaction signatures at rest
- Limit access to settlement_transactions table
- Audit all status changes
- Regular security reviews

---

## 🚀 Deployment Checklist

### Pre-Production

- [ ] Set up secure keypair management
- [ ] Configure production RPC endpoint
- [ ] Set up database backups
- [ ] Configure monitoring alerts
- [ ] Test with devnet first
- [ ] Review all error handling paths
- [ ] Set up logging aggregation
- [ ] Document runbook procedures

### Production

- [ ] Use mainnet-beta RPC endpoint
- [ ] Enable transaction monitoring
- [ ] Set up alert thresholds
- [ ] Configure rate limiting
- [ ] Enable audit logging
- [ ] Set up incident response
- [ ] Test failover procedures
- [ ] Monitor performance metrics

---

## 📈 Future Enhancements

### Short Term
- [ ] Add transaction fee estimation
- [ ] Implement priority fee bidding
- [ ] Add transaction bundling
- [ ] Improve retry backoff strategy

### Medium Term
- [ ] Support multiple payer accounts
- [ ] Add transaction mempool monitoring
- [ ] Implement dynamic gas pricing
- [ ] Add settlement batching optimization

### Long Term
- [ ] Multi-chain support (EVM, etc.)
- [ ] Advanced transaction routing
- [ ] MEV protection mechanisms
- [ ] Cross-chain settlement bridges

---

## 🐛 Troubleshooting

### Settlement Not Processing

```bash
# Check pending settlements
psql -d gridtokenx -c "SELECT * FROM settlements WHERE status = 'pending';"

# Check transaction status
psql -d gridtokenx -c "SELECT * FROM settlement_transactions WHERE status IN ('pending', 'failed');"

# Manually trigger processing
cargo run --bin process_settlements
```

### Transaction Stuck

```bash
# Check transaction on Solana explorer
# https://explorer.solana.com/tx/<signature>

# Check RPC health
curl -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
  http://localhost:8899

# Retry failed transactions
cargo run --bin retry_failed_settlements
```

### Database Issues

```bash
# Run migrations
sqlx migrate run

# Check for missing indexes
psql -d gridtokenx -c "\d settlement_transactions"

# Vacuum and analyze
psql -d gridtokenx -c "VACUUM ANALYZE settlement_transactions;"
```

---

## 📚 References

- [Solana Documentation](https://docs.solana.com/)
- [Solana Web3.js](https://solana-labs.github.io/solana-web3.js/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [GridTokenX Architecture Docs](../technical/architecture/backend/)

---

**Last Updated**: November 9, 2025  
**Maintained by**: Development Team  
**Version**: 1.0
