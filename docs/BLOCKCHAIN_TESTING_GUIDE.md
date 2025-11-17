# GridTokenX Blockchain Testing Guide

## Overview

This guide provides comprehensive information about testing blockchain transactions in the GridTokenX P2P Energy Trading System. The API Gateway includes a dedicated testing module that allows you to create and monitor various types of blockchain transactions for development, testing, and validation purposes.

## Available Test Endpoints

### Base URL
```
http://localhost:8080/api/test
```

### Authentication
All test endpoints require authentication. Include your JWT token in the Authorization header:
```
Authorization: Bearer <your-jwt-token>
```

## Test Transaction Types

### 1. Simple Transfer Test
Tests basic SOL transfer between two accounts.

**Endpoint:** `POST /api/test/transactions`

**Request Body:**
```json
{
  "transaction_type": "simple_transfer"
}
```

**Response:**
```json
{
  "success": true,
  "signature": "2ZE7R2H8L9X5Y4W3Q1A6B7C8D9E0F1G2H3I4J5K6L7M8N9O0P",
  "results": {
    "transactions_created": 1,
    "transactions_confirmed": 1,
    "total_sol_transferred": 0.001,
    "network_metrics": {
      "slot": 123456789,
      "avg_confirmation_time_ms": 2000
    }
  },
  "execution_time_ms": 1500
}
```

### 2. Token Mint Test
Tests energy token minting to a user's token account.

**Request Body:**
```json
{
  "transaction_type": "token_mint"
}
```

**Response:**
```json
{
  "success": true,
  "signature": "3ZF8S3I9J0X6Z5A4Q2B7C8D9E0F1G2H3I4J5K6L7M8N9O0P1Q",
  "results": {
    "transactions_created": 1,
    "transactions_confirmed": 1,
    "total_tokens_minted": 100.0,
    "network_metrics": {
      "slot": 123456790,
      "avg_confirmation_time_ms": 3000
    }
  },
  "execution_time_ms": 2500
}
```

### 3. Settlement Test
Tests energy trading settlement transactions.

**Request Body:**
```json
{
  "transaction_type": "settlement",
  "buyer_wallet": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
  "seller_wallet": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "energy_amount": 50.0,
  "price_per_kwh": 0.15
}
```

**Response:**
```json
{
  "success": true,
  "signature": "4AG9T4J0K1Y7B6C3D8E9F0G1H2I3J4K5L6M7N8O9P0Q1R2S3",
  "results": {
    "transactions_created": 1,
    "transactions_confirmed": 1,
    "network_metrics": {
      "slot": 123456791,
      "avg_confirmation_time_ms": 2500
    }
  },
  "execution_time_ms": 2000
}
```

### 4. Batch Settlement Test
Tests batch processing of multiple settlement transactions.

**Request Body:**
```json
{
  "transaction_type": "batch_settlement",
  "use_batch": true,
  "priority": "normal"
}
```

**Response:**
```json
{
  "success": true,
  "batch_id": "123e4567-e89b-12d3-a456-426614174000",
  "results": {
    "transactions_created": 3,
    "batch_stats": {
      "batch_size": 3,
      "total_fee_lamports": 20000,
      "cost_savings_percent": 25.0,
      "batch_confirmation_time_ms": 5000
    },
    "network_metrics": {
      "slot": 123456792,
      "avg_confirmation_time_ms": 5000
    }
  },
  "execution_time_ms": 4500
}
```

### 5. Multi-Transfer Test
Tests a single transaction with multiple transfer instructions.

**Request Body:**
```json
{
  "transaction_type": "multi_transfer"
}
```

**Response:**
```json
{
  "success": true,
  "signature": "5BH0U5K1L2Z8C7D4E9F0G1H2I3J4K5L6M7N8O9P0Q1R2S3T4",
  "results": {
    "transactions_created": 1,
    "transactions_confirmed": 1,
    "total_sol_transferred": 0.0005,
    "network_metrics": {
      "slot": 123456793,
      "avg_confirmation_time_ms": 3000
    }
  },
  "execution_time_ms": 2500
}
```

### 6. Program Call Test
Tests interaction with Anchor programs.

**Request Body:**
```json
{
  "transaction_type": "program_call"
}
```

**Response:**
```json
{
  "success": true,
  "results": {
    "transactions_created": 1,
    "transactions_confirmed": 1,
    "network_metrics": {
      "slot": 123456794,
      "avg_confirmation_time_ms": 1500
    }
  },
  "execution_time_ms": 1000
}
```

### 7. Stress Test
Tests multiple transactions to assess system performance.

**Request Body:**
```json
{
  "transaction_type": "stress_test"
}
```

**Response:**
```json
{
  "success": true,
  "results": {
    "transactions_created": 10,
    "transactions_confirmed": 8,
    "total_sol_transferred": 0.008,
    "network_metrics": {
      "slot": 123456795,
      "avg_confirmation_time_ms": 4000
    }
  },
  "execution_time_ms": 12000
}
```

## Transaction Status Monitoring

### Get Transaction Status
Monitor the status of any transaction using its signature.

**Endpoint:** `GET /api/test/transactions/{signature}`

**Response:**
```json
{
  "status": "confirmed",
  "signature": "2ZE7R2H8L9X5Y4W3Q1A6B7C8D9E0F1G2H3I4J5K6L7M8N9O0P"
}
```

**Possible Status Values:**
- `pending`: Transaction is being processed
- `confirmed`: Transaction was successful
- `failed`: Transaction failed on-chain

### Get Test Statistics
Get comprehensive statistics about the blockchain testing capabilities.

**Endpoint:** `GET /api/test/statistics`

**Response:**
```json
{
  "cluster": "localnet",
  "health_check": true,
  "current_slot": 123456796,
  "services": {
    "blockchain_service": "available",
    "settlement_service": "available",
    "batch_service": "available"
  },
  "test_capabilities": [
    "simple_transfer",
    "token_mint",
    "settlement",
    "batch_settlement",
    "multi_transfer",
    "program_call",
    "stress_test"
  ]
}
```

## Priority Levels for Batch Transactions

When using batch settlement, you can specify priority levels:

- `urgent`: Highest priority, processed immediately
- `high`: High priority, processed before normal and low
- `normal`: Default priority
- `low`: Lowest priority, processed last

**Example:**
```json
{
  "transaction_type": "batch_settlement",
  "priority": "urgent"
}
```

## Error Handling

### Error Response Format
```json
{
  "success": false,
  "error": "Transaction failed: Insufficient funds",
  "execution_time_ms": 500
}
```

### Common Error Scenarios

1. **Insufficient Funds**
   ```json
   {
     "error": "Transaction failed: Insufficient funds for transfer"
   }
   ```

2. **Invalid Wallet Address**
   ```json
   {
     "error": "Invalid wallet address format"
   }
   ```

3. **Network Connection Issues**
   ```json
   {
     "error": "Failed to connect to Solana network"
   }
   ```

4. **Invalid Transaction Parameters**
   ```json
   {
     "error": "Invalid energy amount: must be positive"
   }
   ```

## Best Practices

### 1. Development Testing
- Use `simple_transfer` for basic connectivity tests
- Use `token_mint` to verify energy token functionality
- Start with individual transactions before testing batches

### 2. Performance Testing
- Use `stress_test` to assess system capacity
- Monitor `execution_time_ms` for performance metrics
- Check `avg_confirmation_time_ms` for network performance

### 3. Integration Testing
- Use `settlement` tests with real wallet addresses
- Test `batch_settlement` with various priority levels
- Verify `program_call` tests for Anchor program integration

### 4. Production Validation
- Always verify transaction signatures on Solana explorer
- Monitor network metrics for anomalies
- Use transaction status endpoints for confirmation

## Configuration

### Environment Variables
Key environment variables that affect blockchain testing:

```bash
# Solana RPC configuration
SOLANA_RPC_URL="http://localhost:8899"
SOLANA_WS_URL="ws://localhost:8899"

# Database configuration
DATABASE_URL="postgresql://user:password@localhost/gridtokenx"

# Redis configuration
REDIS_URL="redis://localhost:6379"

# Logging
RUST_LOG="api_gateway=debug"
```

### Batch Configuration
Default batch processing configuration:

```rust
BatchConfig {
    max_batch_size: 10,
    min_batch_size: 3,
    max_wait_time: 5,
    cost_optimization: true,
    priority_fee: 5_000,
    auto_submit: true,
    submission_interval: 10,
    max_retries: 3,
}
```

## Monitoring and Debugging

### Log Levels
Set appropriate log levels for debugging:

```bash
# Debug level for detailed information
RUST_LOG=api_gateway=debug

# Info level for general operation
RUST_LOG=api_gateway=info

# Error level for production
RUST_LOG=api_gateway=error
```

### Key Metrics to Monitor
1. **Transaction Success Rate**: Percentage of successful transactions
2. **Confirmation Time**: Average time for transaction confirmation
3. **Batch Efficiency**: Cost savings from batch processing
4. **Network Health**: Solana cluster connectivity and performance

### Common Debugging Commands

```bash
# Check API Gateway health
curl http://localhost:8080/health

# Get test statistics
curl -H "Authorization: Bearer <token>" \
     http://localhost:8080/api/test/statistics

# Monitor transaction status
curl -H "Authorization: Bearer <token>" \
     http://localhost:8080/api/test/transactions/<signature>

# Run simple transfer test
curl -X POST \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer <token>" \
     -d '{"transaction_type": "simple_transfer"}' \
     http://localhost:8080/api/test/transactions
```

## Security Considerations

1. **Authentication Required**: All test endpoints require valid JWT tokens
2. **Rate Limiting**: Implement rate limiting for stress tests in production
3. **Fund Management**: Use test accounts with limited funds
4. **Network Isolation**: Use separate test networks for production validation

## Troubleshooting

### Common Issues and Solutions

1. **Transaction Not Confirming**
   - Check Solana validator status
   - Verify network connectivity
   - Check transaction fees

2. **Batch Processing Failures**
   - Verify batch configuration
   - Check individual transaction validity
   - Monitor batch submission logs

3. **Program Call Failures**
   - Verify program deployment
   - Check instruction data format
   - Validate account permissions

4. **Performance Issues**
   - Monitor system resources
   - Check network latency
   - Optimize batch sizes

## Integration with Frontend

### Example JavaScript Integration

```javascript
class BlockchainTestClient {
  constructor(baseUrl, authToken) {
    this.baseUrl = baseUrl;
    this.authToken = authToken;
  }

  async createTestTransaction(transactionType, options = {}) {
    const response = await fetch(`${this.baseUrl}/api/test/transactions`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.authToken}`
      },
      body: JSON.stringify({
        transaction_type: transactionType,
        ...options
      })
    });
    
    return await response.json();
  }

  async getTransactionStatus(signature) {
    const response = await fetch(`${this.baseUrl}/api/test/transactions/${signature}`, {
      headers: {
        'Authorization': `Bearer ${this.authToken}`
      }
    });
    
    return await response.json();
  }

  async getStatistics() {
    const response = await fetch(`${this.baseUrl}/api/test/statistics`, {
      headers: {
        'Authorization': `Bearer ${this.authToken}`
      }
    });
    
    return await response.json();
  }
}

// Usage example
const client = new BlockchainTestClient('http://localhost:8080', 'your-jwt-token');

// Run a simple transfer test
const result = await client.createTestTransaction('simple_transfer');
console.log('Transaction result:', result);

// Check transaction status
const status = await client.getTransactionStatus(result.signature);
console.log('Transaction status:', status);

// Get test statistics
const stats = await client.getStatistics();
console.log('Test statistics:', stats);
```

## Conclusion

The GridTokenX blockchain testing module provides comprehensive tools for validating blockchain transactions, monitoring network performance, and ensuring system reliability. Use these endpoints during development, testing, and production validation to ensure your P2P energy trading system operates correctly on the Solana blockchain.

For additional support or questions, refer to the API documentation or contact the development team.
