---
title: API Reference Documentation
category: reference
subcategory: api
last_updated: 2025-11-08
status: active
related_docs:
  - ../../architecture/backend/API_GATEWAY_ARCHITECTURE.md
  - ../../guides/development/API_DEVELOPMENT.md
tags: [api, rest, endpoints, llm-context]
---

# GridTokenX API Reference

## Overview

The GridTokenX API Gateway provides RESTful endpoints for interacting with the blockchain-based energy trading platform. Built with Rust and Axum, the API handles authentication, energy data submission, trading operations, and administrative functions.

**Base URL**: `http://localhost:8080/api/v1` (development)

**API Version**: 1.0.0

## Table of Contents

1. [Authentication](#authentication)
2. [Users API](#users-api)
3. [Energy API](#energy-api)
4. [Trading API](#trading-api)
5. [Admin API](#admin-api)
6. [WebSocket API](#websocket-api)
7. [Error Handling](#error-handling)
8. [Rate Limiting](#rate-limiting)

---

## Authentication

### Authentication Flow

GridTokenX uses Solana wallet-based authentication with JWT tokens.

#### 1. Request Challenge

```http
POST /api/v1/auth/challenge
Content-Type: application/json

{
  "wallet_address": "5v2w3X8yZ9AbCdEfGhIjKlMnOpQrStUv"
}
```

**Response**:
```json
{
  "challenge": "Sign this message to authenticate: 1234567890",
  "expires_at": "2025-11-08T15:00:00Z"
}
```

#### 2. Submit Signed Challenge

```http
POST /api/v1/auth/verify
Content-Type: application/json

{
  "wallet_address": "5v2w3X8yZ9AbCdEfGhIjKlMnOpQrStUv",
  "signature": "base58_encoded_signature",
  "challenge": "Sign this message to authenticate: 1234567890"
}
```

**Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400,
  "user": {
    "id": "usr_123",
    "wallet_address": "5v2w3X8yZ9AbCdEfGhIjKlMnOpQrStUv",
    "role": "user",
    "created_at": "2025-11-01T10:00:00Z"
  }
}
```

#### 3. Use JWT Token

Include in Authorization header:
```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Token Refresh

```http
POST /api/v1/auth/refresh
Authorization: Bearer {current_token}
```

**Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400
}
```

### Logout

```http
POST /api/v1/auth/logout
Authorization: Bearer {token}
```

**Response**: `204 No Content`

---

## Users API

### Register User

```http
POST /api/v1/users/register
Content-Type: application/json

{
  "wallet_address": "5v2w3X8yZ9AbCdEfGhIjKlMnOpQrStUv",
  "email": "user@example.com",
  "role": "prosumer"
}
```

**Response**: `201 Created`
```json
{
  "id": "usr_123",
  "wallet_address": "5v2w3X8yZ9AbCdEfGhIjKlMnOpQrStUv",
  "email": "user@example.com",
  "role": "prosumer",
  "verified": false,
  "created_at": "2025-11-08T14:00:00Z"
}
```

### Get User Profile

```http
GET /api/v1/users/me
Authorization: Bearer {token}
```

**Response**: `200 OK`
```json
{
  "id": "usr_123",
  "wallet_address": "5v2w3X8yZ9AbCdEfGhIjKlMnOpQrStUv",
  "email": "user@example.com",
  "role": "prosumer",
  "meter_id": "METER_001",
  "token_balance": 150000000000,
  "energy_stats": {
    "total_generated": 120.5,
    "total_consumed": 95.3,
    "net_balance": 25.2
  },
  "created_at": "2025-11-01T10:00:00Z",
  "updated_at": "2025-11-08T14:00:00Z"
}
```

### Update Profile

```http
PATCH /api/v1/users/me
Authorization: Bearer {token}
Content-Type: application/json

{
  "email": "newemail@example.com",
  "notification_preferences": {
    "email_alerts": true,
    "trade_notifications": true
  }
}
```

**Response**: `200 OK` (updated user object)

### Assign Meter

```http
POST /api/v1/users/me/meter
Authorization: Bearer {token}
Content-Type: application/json

{
  "meter_id": "METER_001",
  "location": "Building A, Room 201"
}
```

**Response**: `201 Created`
```json
{
  "meter_id": "METER_001",
  "user_id": "usr_123",
  "location": "Building A, Room 201",
  "status": "active",
  "assigned_at": "2025-11-08T14:30:00Z"
}
```

---

## Energy API

### Submit Energy Reading

```http
POST /api/v1/energy/readings
Authorization: Bearer {token}
Content-Type: application/json

{
  "meter_id": "METER_001",
  "timestamp": "2025-11-08T14:30:00Z",
  "kwh_generated": 2.5,
  "kwh_consumed": 1.8
}
```

**Response**: `201 Created`
```json
{
  "reading_id": "rdg_789",
  "meter_id": "METER_001",
  "timestamp": "2025-11-08T14:30:00Z",
  "kwh_generated": 2.5,
  "kwh_consumed": 1.8,
  "net_energy": 0.7,
  "tokens_minted": 700000000,
  "processed": true
}
```

### Get Energy History

```http
GET /api/v1/energy/history?start_date=2025-11-01&end_date=2025-11-08&interval=daily
Authorization: Bearer {token}
```

**Query Parameters**:
- `start_date` (required): ISO 8601 date
- `end_date` (required): ISO 8601 date
- `interval` (optional): `hourly`, `daily`, `weekly` (default: `daily`)
- `meter_id` (optional): Filter by specific meter

**Response**: `200 OK`
```json
{
  "meter_id": "METER_001",
  "interval": "daily",
  "data": [
    {
      "date": "2025-11-01",
      "kwh_generated": 15.2,
      "kwh_consumed": 12.8,
      "net_energy": 2.4,
      "tokens_earned": 2400000000
    },
    {
      "date": "2025-11-02",
      "kwh_generated": 18.5,
      "kwh_consumed": 14.1,
      "net_energy": 4.4,
      "tokens_earned": 4400000000
    }
  ],
  "summary": {
    "total_generated": 150.5,
    "total_consumed": 120.3,
    "total_net": 30.2,
    "total_tokens": 30200000000
  }
}
```

### Get Real-Time Generation

```http
GET /api/v1/energy/realtime
Authorization: Bearer {token}
```

**Response**: `200 OK`
```json
{
  "meter_id": "METER_001",
  "current_generation": 3.2,
  "current_consumption": 1.5,
  "net_flow": 1.7,
  "timestamp": "2025-11-08T14:45:00Z",
  "forecast_next_hour": 3.8
}
```

---

## Trading API

### Create Sell Order

```http
POST /api/v1/trading/orders/sell
Authorization: Bearer {token}
Content-Type: application/json

{
  "amount": 5000000000,
  "price_per_token": 0.15,
  "min_trade_amount": 1000000000,
  "expires_at": "2025-11-08T16:00:00Z"
}
```

**Response**: `201 Created`
```json
{
  "order_id": "ord_456",
  "type": "sell",
  "amount": 5000000000,
  "amount_kwh": 5.0,
  "price_per_token": 0.15,
  "total_value": 750.00,
  "status": "pending",
  "created_at": "2025-11-08T14:50:00Z",
  "expires_at": "2025-11-08T16:00:00Z"
}
```

### Create Buy Order

```http
POST /api/v1/trading/orders/buy
Authorization: Bearer {token}
Content-Type: application/json

{
  "amount": 3000000000,
  "max_price_per_token": 0.18,
  "min_trade_amount": 500000000
}
```

**Response**: `201 Created`
```json
{
  "order_id": "ord_457",
  "type": "buy",
  "amount": 3000000000,
  "amount_kwh": 3.0,
  "max_price": 0.18,
  "max_total_value": 540.00,
  "status": "pending",
  "created_at": "2025-11-08T14:52:00Z"
}
```

### Get Order Book

```http
GET /api/v1/trading/orderbook
Authorization: Bearer {token}
```

**Response**: `200 OK`
```json
{
  "timestamp": "2025-11-08T14:55:00Z",
  "epoch": 12345,
  "epoch_ends_at": "2025-11-08T15:00:00Z",
  "sell_orders": [
    {
      "order_id": "ord_456",
      "amount": 5000000000,
      "amount_kwh": 5.0,
      "price": 0.15,
      "user_id": "usr_123"
    }
  ],
  "buy_orders": [
    {
      "order_id": "ord_457",
      "amount": 3000000000,
      "amount_kwh": 3.0,
      "max_price": 0.18,
      "user_id": "usr_456"
    }
  ],
  "market_stats": {
    "total_sell_volume": 25000000000,
    "total_buy_volume": 18000000000,
    "average_price": 0.16
  }
}
```

### Get My Orders

```http
GET /api/v1/trading/orders/me?status=pending&limit=20
Authorization: Bearer {token}
```

**Query Parameters**:
- `status` (optional): `pending`, `filled`, `cancelled`, `expired`
- `limit` (optional): Max 100, default 20
- `offset` (optional): Pagination offset

**Response**: `200 OK`
```json
{
  "orders": [
    {
      "order_id": "ord_456",
      "type": "sell",
      "amount": 5000000000,
      "price": 0.15,
      "status": "pending",
      "filled_amount": 0,
      "created_at": "2025-11-08T14:50:00Z"
    }
  ],
  "pagination": {
    "total": 45,
    "limit": 20,
    "offset": 0,
    "has_more": true
  }
}
```

### Cancel Order

```http
DELETE /api/v1/trading/orders/{order_id}
Authorization: Bearer {token}
```

**Response**: `200 OK`
```json
{
  "order_id": "ord_456",
  "status": "cancelled",
  "cancelled_at": "2025-11-08T14:58:00Z"
}
```

### Get Trade History

```http
GET /api/v1/trading/history?start_date=2025-11-01&limit=50
Authorization: Bearer {token}
```

**Response**: `200 OK`
```json
{
  "trades": [
    {
      "trade_id": "trd_789",
      "order_id": "ord_450",
      "type": "sell",
      "amount": 2000000000,
      "price": 0.16,
      "total_value": 320.00,
      "buyer_id": "usr_789",
      "seller_id": "usr_123",
      "executed_at": "2025-11-08T14:00:00Z",
      "epoch": 12344
    }
  ],
  "summary": {
    "total_trades": 23,
    "total_volume": 50000000000,
    "total_value": 8000.00,
    "average_price": 0.16
  }
}
```

---

## Admin API

**Note**: Requires `admin` or `rec_authority` role.

### Get All Users

```http
GET /api/v1/admin/users?role=prosumer&limit=50
Authorization: Bearer {admin_token}
```

**Response**: `200 OK`
```json
{
  "users": [
    {
      "id": "usr_123",
      "wallet_address": "5v2w3X8yZ9...",
      "email": "user@example.com",
      "role": "prosumer",
      "meter_id": "METER_001",
      "status": "active",
      "created_at": "2025-11-01T10:00:00Z"
    }
  ],
  "pagination": {
    "total": 150,
    "limit": 50,
    "offset": 0
  }
}
```

### Verify Meter

```http
POST /api/v1/admin/meters/{meter_id}/verify
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "verified": true,
  "notes": "Meter installation confirmed"
}
```

**Response**: `200 OK`

### System Statistics

```http
GET /api/v1/admin/stats
Authorization: Bearer {admin_token}
```

**Response**: `200 OK`
```json
{
  "users": {
    "total": 150,
    "prosumers": 80,
    "consumers": 70
  },
  "energy": {
    "total_generated_kwh": 5000.5,
    "total_consumed_kwh": 4200.3,
    "total_traded_kwh": 800.2
  },
  "trading": {
    "total_orders": 450,
    "active_orders": 35,
    "completed_trades": 200,
    "total_volume": 800200000000
  },
  "blockchain": {
    "total_transactions": 1250,
    "tokens_minted": 5000500000000,
    "latest_block": 12345678
  }
}
```

---

## WebSocket API

### Connect to WebSocket

```
ws://localhost:8080/api/v1/ws?token={jwt_token}
```

### Subscribe to Order Book Updates

**Send**:
```json
{
  "type": "subscribe",
  "channel": "orderbook"
}
```

**Receive** (every 15 seconds):
```json
{
  "type": "orderbook_update",
  "timestamp": "2025-11-08T15:00:00Z",
  "sell_orders": [...],
  "buy_orders": [...]
}
```

### Subscribe to Trade Executions

**Send**:
```json
{
  "type": "subscribe",
  "channel": "trades"
}
```

**Receive** (on trade):
```json
{
  "type": "trade_executed",
  "trade_id": "trd_790",
  "amount": 2000000000,
  "price": 0.16,
  "executed_at": "2025-11-08T15:00:05Z"
}
```

---

## Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "Amount must be positive",
    "details": {
      "field": "amount",
      "value": -100
    },
    "request_id": "req_abc123"
  }
}
```

### HTTP Status Codes

| Code | Description |
|------|-------------|
| 200  | Success |
| 201  | Created |
| 204  | No Content |
| 400  | Bad Request |
| 401  | Unauthorized |
| 403  | Forbidden |
| 404  | Not Found |
| 409  | Conflict |
| 429  | Too Many Requests |
| 500  | Internal Server Error |
| 503  | Service Unavailable |

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `AUTH_REQUIRED` | 401 | Missing or invalid JWT token |
| `INVALID_SIGNATURE` | 401 | Wallet signature verification failed |
| `INSUFFICIENT_BALANCE` | 400 | Not enough tokens for operation |
| `ORDER_NOT_FOUND` | 404 | Order ID does not exist |
| `METER_ALREADY_ASSIGNED` | 409 | Meter is already assigned to another user |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `BLOCKCHAIN_ERROR` | 500 | Solana RPC error |

---

## Rate Limiting

### Limits

| Endpoint Category | Requests/Minute | Burst |
|-------------------|-----------------|-------|
| Authentication | 10 | 20 |
| Read Operations | 120 | 180 |
| Write Operations | 60 | 90 |
| Admin Operations | 30 | 50 |

### Rate Limit Headers

```http
X-RateLimit-Limit: 120
X-RateLimit-Remaining: 115
X-RateLimit-Reset: 1699451280
```

### Exceeded Rate Limit

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests. Please try again in 45 seconds.",
    "retry_after": 45
  }
}
```

---

## Pagination

### Query Parameters

- `limit`: Max items per page (default: 20, max: 100)
- `offset`: Number of items to skip

### Response Format

```json
{
  "data": [...],
  "pagination": {
    "total": 150,
    "limit": 20,
    "offset": 40,
    "has_more": true
  }
}
```

---

## Testing

### Postman Collection

Import the Postman collection:
```
/docs/technical/reference/api/postman_collection.json
```

### cURL Examples

See full examples in:
```
/docs/technical/guides/development/API_USAGE_EXAMPLES.md
```

---

## Changelog

### v1.0.0 (2025-11-08)
- Initial API release
- Authentication with Solana wallets
- Energy data submission
- P2P trading endpoints
- Admin management endpoints

---

## Support

- **Issues**: GitHub Issues
- **API Status**: https://status.gridtokenx.com (planned)
- **Email**: api-support@gridtokenx.com

---

**Document Version**: 1.0  
**API Version**: 1.0.0  
**Last Updated**: 2025-11-08
