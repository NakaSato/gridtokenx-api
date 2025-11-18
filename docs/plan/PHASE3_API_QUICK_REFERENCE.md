# Phase 3: API Quick Reference
# For Frontend Developers

**Version**: 1.0  
**Date**: November 13, 2025

---

## 🚀 Quick Start

### Base Configuration

```typescript
const API_CONFIG = {
  baseURL: 'http://localhost:8080/api',
  websocketURL: 'ws://localhost:8080/api/market/ws',
  timeout: 30000,
};
```

### Authentication Header

```typescript
headers: {
  'Content-Type': 'application/json',
  'Authorization': `Bearer ${token}` // For protected endpoints
}
```

---

## 🔐 Authentication Endpoints

### 1. Register with Wallet

**Endpoint**: `POST /auth/wallet/register`  
**Auth Required**: No

```typescript
// Request
{
  "username": "alice",
  "email": "alice@example.com",
  "password": "SecurePass123!",
  "role": "prosumer",
  "first_name": "Alice",
  "last_name": "Smith",
  "create_wallet": true,      // Optional, default: false
  "airdrop_amount": 2.0       // Optional, testnet only
}

// Response (200 OK)
{
  "access_token": "eyJhbGci...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "username": "alice",
    "email": "alice@example.com",
    "role": "prosumer",
    "blockchain_registered": true
  },
  "wallet_info": {
    "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
    "balance_lamports": 2000000000,
    "balance_sol": 2.0,
    "private_key": "base58...",  // DEV ONLY - never show in production
    "airdrop_signature": "sig...",
    "created_new": true
  }
}

// Errors
400: "User already exists" | "Invalid email" | "Password too weak"
500: "Internal server error"
```

---

### 2. Standard Registration

**Endpoint**: `POST /auth/register`  
**Auth Required**: No

```typescript
// Request
{
  "username": "bob",
  "email": "bob@example.com",
  "password": "SecurePass123!",
  "role": "consumer",
  "first_name": "Bob",
  "last_name": "Jones"
}

// Response (200 OK)
{
  "message": "Registration successful. Please check your email to verify your account.",
  "user": {
    "id": "uuid-here",
    "email": "bob@example.com",
    "username": "bob",
    "created_at": "2025-11-13T10:00:00Z"
  },
  "verification_required": true
}

// Note: User must verify email before logging in
```

---

### 3. Login with Wallet Info

**Endpoint**: `POST /auth/wallet/login`  
**Auth Required**: No

```typescript
// Request
{
  "username": "alice",
  "password": "SecurePass123!"
}

// Response (200 OK)
{
  "access_token": "eyJhbGci...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "username": "alice",
    "email": "alice@example.com",
    "role": "prosumer",
    "blockchain_registered": true
  },
  "wallet_info": {
    "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
    "balance_lamports": 1500000000,
    "balance_sol": 1.5
  }
}

// Errors
401: "Invalid username or password"
403: "Account is deactivated"
```

---

### 4. Standard Login

**Endpoint**: `POST /auth/login`  
**Auth Required**: No

```typescript
// Request
{
  "username": "bob",
  "password": "SecurePass123!"
}

// Response (200 OK)
{
  "access_token": "eyJhbGci...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "username": "bob",
    "email": "bob@example.com",
    "role": "consumer"
  }
}
```

---

### 5. Verify Email

**Endpoint**: `GET /auth/verify-email?token={verification_token}`  
**Auth Required**: No

```typescript
// Response (200 OK)
{
  "message": "Email verified successfully. You can now login.",
  "email": "bob@example.com"
}

// Errors
400: "Invalid or expired verification token"
```

---

### 6. Resend Verification Email

**Endpoint**: `POST /auth/resend-verification`  
**Auth Required**: No

```typescript
// Request
{
  "email": "bob@example.com"
}

// Response (200 OK)
{
  "message": "Verification email sent successfully"
}

// Note: Rate limited to prevent abuse
```

---

### 7. Get Profile

**Endpoint**: `GET /auth/profile`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "user": {
    "id": "uuid",
    "username": "alice",
    "email": "alice@example.com",
    "role": "prosumer",
    "first_name": "Alice",
    "last_name": "Smith",
    "wallet_address": "7xKXtg2CW...",
    "created_at": "2025-11-01T00:00:00Z",
    "updated_at": "2025-11-13T10:00:00Z"
  }
}
```

---

### 8. Update Profile

**Endpoint**: `POST /auth/profile/update`  
**Auth Required**: Yes

```typescript
// Request
{
  "first_name": "Alicia",
  "last_name": "Smith-Jones",
  "email": "newemail@example.com"
}

// Response (200 OK)
{
  "message": "Profile updated successfully",
  "user": { /* updated user object */ }
}
```

---

## 💼 Wallet Endpoints

### 1. Link Wallet Address

**Endpoint**: `POST /user/wallet`  
**Auth Required**: Yes

```typescript
// Request
{
  "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}

// Response (200 OK)
{
  "message": "Wallet linked successfully",
  "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}

// Errors
400: "Invalid wallet address format"
409: "Wallet already linked to another account"
```

---

### 2. Remove Wallet Address

**Endpoint**: `DELETE /user/wallet`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "message": "Wallet unlinked successfully"
}
```

---

## 📊 Meter Reading Endpoints

### 1. Submit Meter Reading

**Endpoint**: `POST /meters/submit-reading`  
**Auth Required**: Yes

```typescript
// Request
{
  "meter_id": "METER-001",
  "reading_kwh": 150.5,
  "timestamp": "2025-11-13T10:00:00Z"
}

// Response (200 OK)
{
  "reading_id": "uuid-123",
  "meter_id": "METER-001",
  "reading_kwh": 150.5,
  "timestamp": "2025-11-13T10:00:00Z",
  "minted": false,
  "created_at": "2025-11-13T10:01:00Z"
}

// Errors
400: "Reading must be greater than previous reading"
400: "Invalid timestamp"
404: "Meter not found"
```

---

### 2. Get My Readings

**Endpoint**: `GET /meters/my-readings`  
**Auth Required**: Yes

```typescript
// Query parameters (optional)
?limit=10&offset=0&minted=false

// Response (200 OK)
{
  "readings": [
    {
      "reading_id": "uuid-123",
      "meter_id": "METER-001",
      "reading_kwh": 150.5,
      "timestamp": "2025-11-13T10:00:00Z",
      "minted": false,
      "created_at": "2025-11-13T10:01:00Z"
    }
  ],
  "total_count": 42,
  "total_energy": 5234.5
}
```

---

### 3. Get User Stats

**Endpoint**: `GET /meters/stats`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "total_readings": 42,
  "total_energy_kwh": 5234.5,
  "minted_readings": 35,
  "unminted_readings": 7,
  "total_tokens_minted": 5000.0,
  "average_reading": 124.6
}
```

---

## 🪙 Token Endpoints

### 1. Mint Tokens from Reading

**Endpoint**: `POST /tokens/mint-from-reading`  
**Auth Required**: Yes

```typescript
// Request
{
  "reading_id": "uuid-123",
  "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}

// Response (200 OK)
{
  "transaction_signature": "abc123def456...",
  "tokens_minted": 150.5,
  "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "status": "confirmed"
}

// Errors
400: "Reading already minted"
400: "Invalid wallet address"
404: "Reading not found"
500: "Blockchain transaction failed"
```

---

### 2. Get Token Balance

**Endpoint**: `GET /tokens/balance/{wallet_address}`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "balance_lamports": 1500000000,
  "balance_sol": 1.5,
  "token_balance": 150.5,
  "last_updated": "2025-11-13T10:30:00Z"
}

// Errors
400: "Invalid wallet address"
```

---

### 3. Get Token Info

**Endpoint**: `GET /tokens/info`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "token_name": "Energy Token",
  "token_symbol": "ENRG",
  "decimals": 9,
  "total_supply": 1000000.0,
  "mint_authority": "AuthorityPubkey..."
}
```

---

## 📈 Trading Endpoints

### 1. Create Order

**Endpoint**: `POST /trading/orders`  
**Auth Required**: Yes

```typescript
// Request
{
  "order_type": "sell",
  "price": 0.18,
  "amount": 100,
  "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}

// Response (200 OK)
{
  "order_id": "order-uuid",
  "order_type": "sell",
  "price": 0.18,
  "amount": 100,
  "filled_amount": 0,
  "remaining_amount": 100,
  "status": "active",
  "created_at": "2025-11-13T11:00:00Z",
  "blockchain_signature": "def456ghi..."
}

// Errors
400: "Insufficient balance"
400: "Price must be positive"
400: "Amount below minimum order size"
```

---

### 2. Get User Orders

**Endpoint**: `GET /trading/orders`  
**Auth Required**: Yes

```typescript
// Query parameters (optional)
?status=active&limit=20&offset=0

// Response (200 OK)
{
  "orders": [
    {
      "id": "order-uuid",
      "order_type": "sell",
      "price": 0.18,
      "amount": 100,
      "filled_amount": 50,
      "remaining_amount": 50,
      "status": "partial",
      "created_at": "2025-11-13T11:00:00Z",
      "updated_at": "2025-11-13T11:15:00Z"
    }
  ],
  "total_count": 15
}

// Status values: 'active', 'partial', 'filled', 'cancelled'
```

---

### 3. Get Order Book

**Endpoint**: `GET /trading/order-book`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "buy_orders": [
    {
      "price": 0.17,
      "amount": 250,
      "total": 42.5,
      "user_count": 5
    }
  ],
  "sell_orders": [
    {
      "price": 0.18,
      "amount": 300,
      "total": 54.0,
      "user_count": 7
    }
  ],
  "spread": 0.01,
  "last_updated": "2025-11-13T11:20:00Z"
}
```

---

### 4. Get Trading History

**Endpoint**: `GET /trading/history`  
**Auth Required**: Yes

```typescript
// Query parameters (optional)
?limit=20&offset=0

// Response (200 OK)
{
  "transactions": [
    {
      "id": "tx-uuid",
      "order_id": "order-uuid",
      "type": "sell",
      "amount": 50,
      "price": 0.18,
      "total": 9.0,
      "counterparty": "BuyerWalletAddress...",
      "signature": "ghi789jkl...",
      "status": "confirmed",
      "created_at": "2025-11-13T11:15:00Z"
    }
  ],
  "total_trades": 10,
  "total_volume": 500,
  "total_revenue": 90.0
}
```

---

### 5. Get Market Data

**Endpoint**: `GET /trading/market`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "current_price": 0.175,
  "24h_high": 0.20,
  "24h_low": 0.15,
  "24h_volume": 1500,
  "24h_change": 0.025,
  "total_orders": 45,
  "active_traders": 12
}
```

---

### 6. Get Trading Stats

**Endpoint**: `GET /trading/stats`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "user_stats": {
    "total_orders": 15,
    "active_orders": 3,
    "completed_orders": 12,
    "total_volume": 500,
    "total_value": 90.0
  },
  "market_stats": {
    "total_volume_24h": 1500,
    "total_trades_24h": 120,
    "average_price_24h": 0.175
  }
}
```

---

### 7. Cancel Order

**Endpoint**: `DELETE /trading/orders/{order_id}`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "message": "Order cancelled successfully",
  "order_id": "order-uuid",
  "cancelled_at": "2025-11-13T11:30:00Z"
}

// Errors
404: "Order not found"
400: "Cannot cancel order that is already filled"
403: "Not authorized to cancel this order"
```

---

## 📜 ERC (Energy Certificate) Endpoints

### 1. Get My Certificates

**Endpoint**: `GET /erc/my-certificates`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "certificates": [
    {
      "certificate_id": "erc-uuid",
      "energy_kwh": 150.5,
      "issue_date": "2025-11-13T10:00:00Z",
      "status": "active",
      "metadata": {
        "meter_id": "METER-001",
        "location": "Bangkok, Thailand"
      }
    }
  ],
  "total_count": 10,
  "total_energy": 1500.0
}
```

---

### 2. Get Certificate Stats

**Endpoint**: `GET /erc/my-stats`  
**Auth Required**: Yes

```typescript
// Response (200 OK)
{
  "total_certificates": 10,
  "active_certificates": 8,
  "retired_certificates": 2,
  "total_energy_certified": 1500.0,
  "total_energy_retired": 300.0
}
```

---

## 🔌 WebSocket API

### Connection

**URL**: `ws://localhost:8080/api/market/ws?token={access_token}`

```typescript
const ws = new WebSocket(`ws://localhost:8080/api/market/ws?token=${token}`);

ws.onopen = () => {
  console.log('Connected');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  handleMessage(message);
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected');
  // Implement reconnection logic
};
```

---

### Message Types

#### 1. Order Book Update

```typescript
{
  "type": "order_book_update",
  "payload": {
    "buy_orders": [
      { "price": 0.17, "amount": 250, "total": 42.5 }
    ],
    "sell_orders": [
      { "price": 0.18, "amount": 300, "total": 54.0 }
    ],
    "timestamp": "2025-11-13T11:20:00Z"
  }
}

// Action: Update order book display
```

---

#### 2. Order Matched

```typescript
{
  "type": "order_matched",
  "payload": {
    "order_id": "your-order-uuid",
    "matched_amount": 50,
    "match_price": 0.18,
    "buyer": "BuyerWallet...",
    "seller": "SellerWallet...",
    "timestamp": "2025-11-13T11:15:00Z"
  }
}

// Action: Show notification "Your order matched!"
```

---

#### 3. Order Filled

```typescript
{
  "type": "order_filled",
  "payload": {
    "order_id": "your-order-uuid",
    "status": "filled",
    "transaction_signature": "ghi789jkl...",
    "tokens_sold": 100,
    "payment_received": 18.0,
    "timestamp": "2025-11-13T11:20:00Z"
  }
}

// Action: Show success notification, refresh orders, update balance
```

---

#### 4. Epoch Transition

```typescript
{
  "type": "epoch_transition",
  "payload": {
    "previous_epoch": 1234,
    "current_epoch": 1235,
    "clearing_price": 0.175,
    "total_volume": 500,
    "total_matches": 15,
    "timestamp": "2025-11-13T11:00:00Z"
  }
}

// Action: Show epoch notification, refresh market data
```

---

## ⚠️ Error Handling

### Standard Error Response

```typescript
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": { /* additional context */ }
  }
}
```

### Common Status Codes

```
200 OK - Success
201 Created - Resource created
400 Bad Request - Invalid input
401 Unauthorized - Missing/invalid token
403 Forbidden - Insufficient permissions
404 Not Found - Resource doesn't exist
409 Conflict - Resource already exists
429 Too Many Requests - Rate limit exceeded
500 Internal Server Error - Server error
503 Service Unavailable - Service down
```

---

## 🔐 Authentication Flow

### 1. Initial Login/Registration

```typescript
// 1. Register or Login
const response = await fetch('/api/auth/wallet/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ username, password })
});

const { access_token, user, wallet_info } = await response.json();

// 2. Store token
localStorage.setItem('token', access_token);
localStorage.setItem('user', JSON.stringify(user));
localStorage.setItem('wallet', JSON.stringify(wallet_info));
```

---

### 2. Authenticated Requests

```typescript
// Include token in Authorization header
const response = await fetch('/api/trading/orders', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${localStorage.getItem('token')}`
  },
  body: JSON.stringify(orderData)
});
```

---

### 3. Token Refresh (Future)

```typescript
// When 401 error received
if (response.status === 401) {
  // Clear token
  localStorage.removeItem('token');
  // Redirect to login
  window.location.href = '/login';
}
```

---

## 💡 Best Practices

### 1. API Calls

```typescript
// Create a centralized API service
class ApiService {
  private baseURL = 'http://localhost:8080/api';
  
  private async request(endpoint, options) {
    const token = localStorage.getItem('token');
    const response = await fetch(`${this.baseURL}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(token && { 'Authorization': `Bearer ${token}` }),
        ...options.headers
      }
    });
    
    if (!response.ok) {
      throw new Error(await response.text());
    }
    
    return response.json();
  }
  
  async login(username, password) {
    return this.request('/auth/wallet/login', {
      method: 'POST',
      body: JSON.stringify({ username, password })
    });
  }
  
  async createOrder(orderData) {
    return this.request('/trading/orders', {
      method: 'POST',
      body: JSON.stringify(orderData)
    });
  }
}
```

---

### 2. Error Handling

```typescript
try {
  const data = await api.createOrder(orderData);
  showSuccessNotification('Order created!');
} catch (error) {
  if (error.message.includes('Insufficient balance')) {
    showErrorNotification('Not enough balance to create order');
  } else {
    showErrorNotification('Failed to create order');
  }
  console.error(error);
}
```

---

### 3. Loading States

```typescript
const [isLoading, setIsLoading] = useState(false);

const handleSubmit = async () => {
  setIsLoading(true);
  try {
    await api.createOrder(orderData);
  } finally {
    setIsLoading(false);
  }
};
```

---

## 📚 Additional Resources

- **OpenAPI Docs**: http://localhost:8080/api/docs
- **Health Check**: http://localhost:8080/health
- **Postman Collection**: `../../api-gateway/postman/`
- **Test Script**: `./scripts/test-wallet-auth.sh`

---

**Document Version**: 1.0  
**Last Updated**: November 13, 2025

---

*Quick reference for all Phase 3 API endpoints. Backend is ready for integration.*
