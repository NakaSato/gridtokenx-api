# API Usage Examples

This document provides practical examples for using the GridTokenX Platform API.

## Authentication Flow

### 1. Register a New User

```bash
curl -X POST http://localhost:8080/api/users/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "prosumer@example.com",
    "password": "SecurePassword123!",
    "role": "prosumer",
    "wallet_address": "5yW8R9jk..."
  }'
```

**Response:**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "email": "prosumer@example.com",
  "role": "prosumer",
  "created_at": "2025-11-10T10:30:00Z"
}
```

### 2. Login

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "prosumer@example.com",
    "password": "SecurePassword123!"
  }'
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "prosumer@example.com",
    "role": "prosumer"
  }
}
```

### 3. Use JWT Token

Store the token and include it in subsequent requests:

```bash
export JWT_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

curl -X GET http://localhost:8080/api/auth/profile \
  -H "Authorization: Bearer $JWT_TOKEN"
```

## Energy Trading

### 1. Create a Buy Order

```bash
curl -X POST http://localhost:8080/api/v1/trading/orders \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "order_type": "limit",
    "side": "buy",
    "amount": "100.50",
    "price": "0.15",
    "expiry": "2025-11-10T18:00:00Z"
  }'
```

**Response:**
```json
{
  "id": "ord_123e4567",
  "status": "pending",
  "created_at": "2025-11-10T10:35:00Z",
  "message": "Order created successfully"
}
```

### 2. Get Your Orders

```bash
curl -X GET "http://localhost:8080/api/v1/trading/orders?status=pending&limit=10" \
  -H "Authorization: Bearer $JWT_TOKEN"
```

**Response:**
```json
{
  "orders": [
    {
      "id": "ord_123e4567",
      "side": "buy",
      "amount": "100.50",
      "price": "0.15",
      "status": "pending",
      "created_at": "2025-11-10T10:35:00Z"
    }
  ],
  "total": 1
}
```

### 3. Get Market Data

```bash
curl -X GET http://localhost:8080/api/v1/trading/market-data \
  -H "Authorization: Bearer $JWT_TOKEN"
```

**Response:**
```json
{
  "epoch_number": 42,
  "clearing_price": "0.145",
  "total_volume": "5420.75",
  "order_book": {
    "bids": [
      {"price": "0.14", "quantity": "500.00", "order_count": 3}
    ],
    "asks": [
      {"price": "0.15", "quantity": "300.00", "order_count": 2}
    ]
  },
  "last_updated": "2025-11-10T10:40:00Z"
}
```

## Smart Meter Readings

### 1. Submit a Reading

```bash
curl -X POST http://localhost:8080/api/meters/submit-reading \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "kwh_amount": "25.5",
    "reading_timestamp": "2025-11-10T10:00:00Z",
    "meter_signature": "sig_abc123..."
  }'
```

**Response:**
```json
{
  "id": "read_123e4567",
  "user_id": "123e4567-e89b-12d3-a456-426614174000",
  "wallet_address": "5yW8R9jk...",
  "kwh_amount": "25.5",
  "reading_timestamp": "2025-11-10T10:00:00Z",
  "submitted_at": "2025-11-10T10:45:00Z",
  "minted": false,
  "mint_tx_signature": null
}
```

### 2. Mint Tokens from Reading

```bash
curl -X POST http://localhost:8080/api/meters/mint-from-reading \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "reading_id": "read_123e4567"
  }'
```

**Response:**
```json
{
  "success": true,
  "transaction_signature": "5Ky7xJ...",
  "reading_id": "read_123e4567",
  "kwh_amount": "25.5"
}
```

### 3. Get Your Statistics

```bash
curl -X GET http://localhost:8080/api/meters/my-stats \
  -H "Authorization: Bearer $JWT_TOKEN"
```

**Response:**
```json
{
  "total_readings": 15,
  "unminted_kwh": "10.5",
  "minted_kwh": "350.0",
  "total_kwh": "360.5"
}
```

## Energy Renewable Certificates (ERC)

### 1. Issue Certificate (REC Authority Only)

```bash
curl -X POST http://localhost:8080/api/erc/issue \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_address": "5yW8R9jk...",
    "kwh_amount": "100.0",
    "expiry_date": "2026-11-10T00:00:00Z",
    "issuer_name": "Green Energy Certifier Inc",
    "metadata": {
      "source": "solar",
      "location": "California"
    }
  }'
```

**Response:**
```json
{
  "id": "cert_123e4567",
  "certificate_id": "ERC-2025-001234",
  "user_id": "123e4567-e89b-12d3-a456-426614174000",
  "wallet_address": "5yW8R9jk...",
  "kwh_amount": "100.0",
  "issue_date": "2025-11-10T10:50:00Z",
  "expiry_date": "2026-11-10T00:00:00Z",
  "issuer_wallet": "rec_authority_wallet",
  "issuer_name": "Green Energy Certifier Inc",
  "status": "active",
  "blockchain_tx_signature": "tx_5Ky7xJ...",
  "metadata": {
    "source": "solar",
    "location": "California"
  }
}
```

### 2. Get Your Certificates

```bash
curl -X GET "http://localhost:8080/api/erc/my-certificates?limit=10&offset=0" \
  -H "Authorization: Bearer $JWT_TOKEN"
```

### 3. Retire a Certificate

```bash
curl -X POST http://localhost:8080/api/erc/ERC-2025-001234/retire \
  -H "Authorization: Bearer $JWT_TOKEN"
```

## Blockchain Operations

### 1. Submit a Transaction

```bash
curl -X POST http://localhost:8080/api/blockchain/submit \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "program_id": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    "instruction_data": "base64_encoded_data",
    "accounts": [
      {
        "pubkey": "5yW8R9jk...",
        "is_signer": true,
        "is_writable": true
      }
    ]
  }'
```

**Response:**
```json
{
  "signature": "5Ky7xJ...",
  "status": "pending",
  "submitted_at": "2025-11-10T10:55:00Z",
  "estimated_confirmation_time": 10
}
```

### 2. Get Transaction Status

```bash
curl -X GET http://localhost:8080/api/blockchain/transactions/5Ky7xJ... \
  -H "Authorization: Bearer $JWT_TOKEN"
```

**Response:**
```json
{
  "signature": "5Ky7xJ...",
  "status": "confirmed",
  "confirmed_at": "2025-11-10T10:55:15Z",
  "slot": 12345678,
  "block_height": 12345678
}
```

### 3. Get Network Status

```bash
curl -X GET http://localhost:8080/api/blockchain/network-status \
  -H "Authorization: Bearer $JWT_TOKEN"
```

**Response:**
```json
{
  "cluster": "devnet",
  "health": "healthy",
  "current_slot": 12345678,
  "block_height": 12345678,
  "transaction_count": 150234,
  "epoch": 420
}
```

## Blockchain Testing

### 1. Run Test Transaction

```bash
curl -X POST http://localhost:8080/api/blockchain/test/transaction \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "transaction_type": "simple_transfer",
    "use_batch": false
  }'
```

**Response:**
```json
{
  "success": true,
  "signature": "test_5Ky7xJ...",
  "batch_id": null,
  "results": {
    "transactions_created": 1,
    "transactions_confirmed": 1,
    "total_sol_transferred": 0.001,
    "network_metrics": {
      "slot": 12345678,
      "avg_confirmation_time_ms": 2000
    }
  },
  "error": null,
  "execution_time_ms": 2500
}
```

### 2. Get Test Statistics

```bash
curl -X GET http://localhost:8080/api/blockchain/test/statistics \
  -H "Authorization: Bearer $JWT_TOKEN"
```

## Governance Operations

### 1. Get Governance Status

```bash
curl -X GET http://localhost:8080/api/governance/status \
  -H "Authorization: Bearer $JWT_TOKEN"
```

**Response:**
```json
{
  "is_paused": false,
  "admin_wallet": "admin_wallet_address",
  "pause_timestamp": null,
  "last_action": "system_started",
  "last_action_timestamp": "2025-11-10T00:00:00Z"
}
```

### 2. Emergency Pause (Admin Only)

```bash
curl -X POST http://localhost:8080/api/governance/emergency-pause \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "reason": "Security audit in progress"
  }'
```

## Oracle Operations

### 1. Submit Price (Oracle Only)

```bash
curl -X POST http://localhost:8080/api/oracle/submit-price \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "KWH/USD",
    "price": "0.145",
    "confidence": "0.99"
  }'
```

### 2. Get Current Prices

```bash
curl -X GET http://localhost:8080/api/oracle/prices \
  -H "Authorization: Bearer $JWT_TOKEN"
```

**Response:**
```json
{
  "prices": [
    {
      "symbol": "KWH/USD",
      "price": "0.145",
      "timestamp": "2025-11-10T11:00:00Z",
      "confidence": "0.99"
    }
  ]
}
```

## Error Handling

### Common Error Responses

**401 Unauthorized:**
```json
{
  "error": "Unauthorized",
  "message": "Invalid or missing JWT token"
}
```

**403 Forbidden:**
```json
{
  "error": "Forbidden",
  "message": "Insufficient permissions for this operation"
}
```

**400 Bad Request:**
```json
{
  "error": "ValidationError",
  "message": "Invalid request parameters",
  "details": {
    "amount": "must be greater than 0"
  }
}
```

**500 Internal Server Error:**
```json
{
  "error": "InternalServerError",
  "message": "An unexpected error occurred"
}
```

## Rate Limiting

- **Default**: 100 requests per minute per IP
- **Authenticated**: 1000 requests per minute per user
- **Headers included in response:**
  - `X-RateLimit-Limit`: Maximum requests allowed
  - `X-RateLimit-Remaining`: Requests remaining
  - `X-RateLimit-Reset`: Time when limit resets (Unix timestamp)

## WebSocket Connections

### Connect to Trading Updates

```javascript
const ws = new WebSocket('ws://localhost:8080/api/ws/trading?token=YOUR_JWT_TOKEN&channels=orders,matches');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};

// Example messages:
// - OrderBookUpdate
// - OrderUpdate
// - MatchNotification
// - EpochTransition
```

## Best Practices

1. **Always validate responses** - Check status codes and error messages
2. **Store JWT securely** - Never expose tokens in client-side code
3. **Handle rate limits** - Implement exponential backoff
4. **Use HTTPS in production** - Never send credentials over HTTP
5. **Implement timeout handling** - Set reasonable timeout values
6. **Log requests for debugging** - Keep audit trails for troubleshooting

## Postman Collection

Import the Postman collection from `docs/openapi/postman/GridTokenX_API.postman_collection.json` for easy testing.
