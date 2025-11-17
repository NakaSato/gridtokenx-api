# GridTokenX API Documentation

**Version:** 1.0.0  
**Base URL:** `http://localhost:8080` (Development)  
**Last Updated:** November 13, 2025

## Table of Contents

1. [Overview](#overview)
2. [Authentication](#authentication)
3. [Error Handling](#error-handling)
4. [Pagination](#pagination)
5. [API Endpoints](#api-endpoints)
   - [Authentication](#authentication-endpoints)
   - [User Management](#user-management)
   - [Meter Readings](#meter-readings)
   - [Energy Trading](#energy-trading)
   - [Trading Orders](#trading-orders)
   - [ERC Certificates](#erc-certificates)
   - [Health Check](#health-check)
6. [WebSocket API](#websocket-api)
7. [Rate Limiting](#rate-limiting)
8. [Examples](#examples)

---

## Overview

The GridTokenX API is a RESTful API for managing decentralized energy trading on the Solana blockchain. It enables:

- User registration and authentication
- Smart meter reading submission
- Energy token minting
- Peer-to-peer energy trading
- Renewable Energy Certificate (ERC) management
- Real-time WebSocket updates

### API Features

- **JWT Authentication**: Secure token-based authentication
- **Role-Based Access Control**: Producer, Consumer, Prosumer roles
- **Pagination**: Consistent pagination across all list endpoints
- **Validation**: Comprehensive input validation
- **Error Codes**: Structured error responses
- **Rate Limiting**: Protection against abuse
- **Real-time Updates**: WebSocket support for live data

---

## Authentication

### Authentication Flow

1. **Register** a new account (`POST /api/auth/register`)
2. **Verify** email (check email for verification link)
3. **Login** to receive JWT token (`POST /api/auth/login`)
4. Include token in **Authorization** header for protected endpoints

### JWT Token

Include the JWT token in the Authorization header:

```
Authorization: Bearer <your_jwt_token>
```

### Token Expiration

- Access tokens expire after 24 hours
- Refresh tokens expire after 7 days
- Use refresh endpoint to get new access token

---

## Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "AUTH_1001",
    "code_number": 1001,
    "message": "Invalid email or password",
    "details": null,
    "field": "password"
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-11-13T10:30:00Z"
}
```

### Error Code Categories

| Code Range | Category | Description |
|------------|----------|-------------|
| 1xxx | Authentication | Login, registration, token errors |
| 2xxx | Authorization | Permission and role errors |
| 3xxx | Validation | Input validation errors |
| 4xxx | Resources | Not found, conflict errors |
| 5xxx | Business Logic | Insufficient balance, etc. |
| 6xxx | Blockchain | Solana transaction errors |
| 7xxx | Database | Database operation errors |
| 8xxx | External Services | Third-party service errors |
| 9xxx | Rate Limiting | Too many requests |

### Common Error Codes

| Code | Message | Description |
|------|---------|-------------|
| AUTH_1001 | Invalid credentials | Wrong username/password |
| AUTH_1002 | Token expired | JWT token has expired |
| AUTH_1003 | Email not verified | Email verification required |
| VALIDATION_3001 | Invalid input | Input validation failed |
| RESOURCE_4001 | Not found | Resource doesn't exist |
| PERMISSION_2001 | Forbidden | Insufficient permissions |

---

## Pagination

All list endpoints support pagination with consistent parameters.

### Query Parameters

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `page` | integer | 1 | 1+ | Page number |
| `page_size` | integer | 20 | 1-100 | Items per page |
| `sort_by` | string | varies | - | Field to sort by |
| `sort_order` | string | desc | asc/desc | Sort direction |

### Response Format

```json
{
  "data": [...],
  "pagination": {
    "current_page": 1,
    "total_pages": 10,
    "total_items": 195,
    "items_per_page": 20,
    "has_next": true,
    "has_previous": false
  }
}
```

### Example Request

```bash
GET /api/users?page=2&page_size=50&sort_by=created_at&sort_order=desc
```

---

## API Endpoints

### Authentication Endpoints

#### Register User

**POST** `/api/auth/register`

Register a new user account.

**Request Body:**
```json
{
  "username": "john_producer",
  "email": "john@example.com",
  "password": "SecurePass123!",
  "first_name": "John",
  "last_name": "Doe",
  "role": "producer"
}
```

**Validation Rules:**
- `username`: 3-50 characters, alphanumeric + underscore/hyphen
- `email`: Valid email format
- `password`: Min 8 characters, must contain letter and number
- `role`: One of: consumer, producer, prosumer

**Response:** `200 OK`
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "john_producer",
    "email": "john@example.com",
    "role": "producer",
    "email_verified": false,
    "created_at": "2025-11-13T10:00:00Z"
  },
  "message": "Registration successful. Please verify your email."
}
```

**Errors:**
- `400`: Validation error (weak password, invalid email)
- `409`: Username or email already exists

---

#### Login

**POST** `/api/auth/login`

Authenticate and receive JWT token.

**Request Body:**
```json
{
  "username": "john_producer",
  "password": "SecurePass123!"
}
```

**Response:** `200 OK`
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "john_producer",
    "role": "producer",
    "email_verified": true
  }
}
```

**Errors:**
- `401`: Invalid credentials
- `403`: Email not verified

---

#### Get Current User

**GET** `/api/auth/me`

Get authenticated user's profile.

**Headers:**
```
Authorization: Bearer <token>
```

**Response:** `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_producer",
  "email": "john@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "role": "producer",
  "wallet_address": "GvPhiX9W1v3fj8WbN5D2TzzPwf1Kp1TfMg1e8KW1Pump",
  "email_verified": true,
  "created_at": "2025-11-13T10:00:00Z"
}
```

---

#### Refresh Token

**POST** `/api/auth/refresh`

Get new access token using refresh token.

**Request Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response:** `200 OK`
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

---

### User Management

#### List Users

**GET** `/api/users`

List all users (Admin only).

**Query Parameters:**
- `page`: Page number (default: 1)
- `page_size`: Items per page (default: 20, max: 100)
- `sort_by`: created_at, username, email, role
- `sort_order`: asc, desc
- `search`: Search username/email/name
- `role`: Filter by role

**Example:**
```bash
GET /api/users?page=1&page_size=20&sort_by=created_at&role=producer&search=john
```

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "john_producer",
      "email": "john@example.com",
      "first_name": "John",
      "last_name": "Doe",
      "role": "producer",
      "email_verified": true,
      "created_at": "2025-11-13T10:00:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 5,
    "total_items": 95,
    "items_per_page": 20,
    "has_next": true,
    "has_previous": false
  }
}
```

---

### Meter Readings

#### Submit Reading

**POST** `/api/meters/readings`

Submit a smart meter reading (Producer/Prosumer only).

**Request Body:**
```json
{
  "kwh_amount": 150.5,
  "reading_timestamp": "2025-11-13T09:00:00Z"
}
```

**Validation:**
- `kwh_amount`: 0 to 1,000,000 kWh
- `reading_timestamp`: Cannot be in the future

**Response:** `200 OK`
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440000",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "wallet_address": "GvPhiX9W1v3fj8WbN5D2TzzPwf1Kp1TfMg1e8KW1Pump",
  "kwh_amount": 150.5,
  "reading_timestamp": "2025-11-13T09:00:00Z",
  "submitted_at": "2025-11-13T10:30:00Z",
  "minted": false,
  "mint_tx_signature": null
}
```

**Errors:**
- `400`: Invalid amount (negative or too high)
- `403`: Not a producer/prosumer

---

#### Get My Readings

**GET** `/api/meters/my-readings`

Get authenticated user's meter readings.

**Query Parameters:**
- `page`: Page number
- `page_size`: Items per page
- `sort_by`: submitted_at, reading_timestamp, kwh_amount
- `sort_order`: asc, desc
- `minted`: Filter by minted status (true/false)

**Example:**
```bash
GET /api/meters/my-readings?page=1&page_size=20&minted=false&sort_by=kwh_amount&sort_order=desc
```

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440000",
      "kwh_amount": 150.5,
      "reading_timestamp": "2025-11-13T09:00:00Z",
      "submitted_at": "2025-11-13T10:30:00Z",
      "minted": false,
      "mint_tx_signature": null
    }
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 3,
    "total_items": 45,
    "items_per_page": 20,
    "has_next": true,
    "has_previous": false
  }
}
```

---

#### Get User Statistics

**GET** `/api/meters/my-stats`

Get user's meter reading statistics.

**Response:** `200 OK`
```json
{
  "total_readings": 45,
  "total_kwh": 6750.5,
  "minted_readings": 30,
  "minted_kwh": 4500.0,
  "unminted_readings": 15,
  "unminted_kwh": 2250.5
}
```

---

### Energy Trading

#### Create Offer

**POST** `/api/offers`

Create energy offer (Producer only).

**Request Body:**
```json
{
  "energy_amount": 100.0,
  "price_per_kwh": 0.15,
  "energy_source": "solar",
  "available_from": "2025-11-13T00:00:00Z",
  "available_until": "2025-11-20T00:00:00Z"
}
```

**Validation:**
- `energy_amount`: > 0, reasonable limit
- `price_per_kwh`: > 0, < 1000
- `energy_source`: solar, wind, hydro, mixed, biomass, geothermal
- `available_until`: Cannot be in the past

**Response:** `200 OK`
```json
{
  "id": "770e8400-e29b-41d4-a716-446655440000",
  "seller_id": "550e8400-e29b-41d4-a716-446655440000",
  "seller_username": "john_producer",
  "energy_amount": 100.0,
  "price_per_kwh": 0.15,
  "energy_source": "solar",
  "status": "active",
  "available_from": "2025-11-13T00:00:00Z",
  "available_until": "2025-11-20T00:00:00Z",
  "created_at": "2025-11-13T10:30:00Z"
}
```

**Errors:**
- `400`: Invalid energy source or price
- `403`: Not a producer

---

#### List Offers

**GET** `/api/offers`

List active energy offers.

**Query Parameters:**
- `page`: Page number
- `page_size`: Items per page
- `sort_by`: price_per_kwh, energy_amount, created_at, available_until
- `sort_order`: asc, desc
- `energy_source`: Filter by source
- `min_price`: Minimum price filter
- `max_price`: Maximum price filter
- `min_amount`: Minimum amount filter

**Example:**
```bash
GET /api/offers?energy_source=solar&max_price=0.20&sort_by=price_per_kwh&sort_order=asc
```

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "seller_id": "550e8400-e29b-41d4-a716-446655440000",
      "seller_username": "john_producer",
      "energy_amount": 100.0,
      "price_per_kwh": 0.15,
      "energy_source": "solar",
      "status": "active",
      "available_from": "2025-11-13T00:00:00Z",
      "available_until": "2025-11-20T00:00:00Z",
      "created_at": "2025-11-13T10:30:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 5,
    "total_items": 92,
    "items_per_page": 20,
    "has_next": true,
    "has_previous": false
  }
}
```

---

#### Create Order

**POST** `/api/orders`

Create energy purchase order (Consumer only).

**Request Body:**
```json
{
  "energy_amount": 50.0,
  "max_price_per_kwh": 0.20,
  "preferred_source": "solar"
}
```

**Validation:**
- `energy_amount`: > 0
- `max_price_per_kwh`: > 0, < 1000
- `preferred_source`: Optional, must be valid energy source

**Response:** `200 OK`
```json
{
  "id": "880e8400-e29b-41d4-a716-446655440000",
  "buyer_id": "650e8400-e29b-41d4-a716-446655440000",
  "energy_amount": 50.0,
  "max_price_per_kwh": 0.20,
  "preferred_source": "solar",
  "status": "pending",
  "created_at": "2025-11-13T10:35:00Z"
}
```

**Errors:**
- `400`: Invalid amount or price
- `403`: Not a consumer

---

#### List Orders

**GET** `/api/orders`

List user's energy orders.

**Query Parameters:**
- `page`: Page number
- `page_size`: Items per page
- `sort_by`: created_at, energy_amount, max_price_per_kwh
- `sort_order`: asc, desc
- `status`: Filter by status (pending, partial, filled, cancelled)

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "880e8400-e29b-41d4-a716-446655440000",
      "buyer_id": "650e8400-e29b-41d4-a716-446655440000",
      "energy_amount": 50.0,
      "max_price_per_kwh": 0.20,
      "preferred_source": "solar",
      "status": "pending",
      "created_at": "2025-11-13T10:35:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 2,
    "total_items": 25,
    "items_per_page": 20,
    "has_next": true,
    "has_previous": false
  }
}
```

---

#### List Transactions

**GET** `/api/transactions`

List user's trading transactions.

**Query Parameters:**
- `page`: Page number
- `page_size`: Items per page
- `sort_by`: created_at, settled_at, energy_amount, total_price
- `sort_order`: asc, desc
- `status`: Filter by status (pending, processing, completed, failed)

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "990e8400-e29b-41d4-a716-446655440000",
      "offer_id": "770e8400-e29b-41d4-a716-446655440000",
      "order_id": "880e8400-e29b-41d4-a716-446655440000",
      "seller_id": "550e8400-e29b-41d4-a716-446655440000",
      "buyer_id": "650e8400-e29b-41d4-a716-446655440000",
      "energy_amount": 50.0,
      "price_per_kwh": 0.15,
      "total_price": 7.50,
      "status": "completed",
      "blockchain_tx_hash": "5yZ...",
      "created_at": "2025-11-13T10:40:00Z",
      "settled_at": "2025-11-13T10:45:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 3,
    "total_items": 48,
    "items_per_page": 20,
    "has_next": true,
    "has_previous": false
  }
}
```

---

### Trading Orders

#### Get My Orders

**GET** `/api/trading/my-orders`

Get user's trading orders.

**Query Parameters:**
- `page`: Page number
- `page_size`: Items per page
- `sort_by`: created_at, price_per_kwh, energy_amount, filled_at
- `sort_order`: asc, desc
- `status`: Filter by status
- `side`: Filter by side (buy/sell)

**Response:** Similar to List Orders

---

### ERC Certificates

#### Get My Certificates

**GET** `/api/erc/my-certificates`

Get user's Renewable Energy Certificates.

**Query Parameters:**
- `page`: Page number
- `page_size`: Items per page
- `sort_by`: issue_date, expiry_date, kwh_amount, status
- `sort_order`: asc, desc
- `status`: Filter by status (active, retired, expired)

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "aa0e8400-e29b-41d4-a716-446655440000",
      "certificate_id": "ERC-2025-001234",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "wallet_address": "GvPhiX9W1v3fj8WbN5D2TzzPwf1Kp1TfMg1e8KW1Pump",
      "kwh_amount": 1000.0,
      "issue_date": "2025-11-01T00:00:00Z",
      "expiry_date": "2026-11-01T00:00:00Z",
      "issuer_wallet": "BvPhiX9W1v3fj8WbN5D2TzzPwf1Kp1TfMg1e8KW2Cert",
      "issuer_name": "GridTokenX Authority",
      "status": "active",
      "blockchain_tx_signature": "3xY...",
      "metadata": {
        "energy_source": "solar",
        "location": "California"
      }
    }
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 1,
    "total_items": 5,
    "items_per_page": 20,
    "has_next": false,
    "has_previous": false
  }
}
```

---

### Health Check

#### Basic Health

**GET** `/health`

Basic health check.

**Response:** `200 OK`
```json
{
  "status": "healthy",
  "timestamp": "2025-11-13T10:00:00Z",
  "version": "1.0.0"
}
```

---

#### Readiness Check

**GET** `/health/ready`

Detailed readiness check with dependencies.

**Response:** `200 OK`
```json
{
  "status": "healthy",
  "timestamp": "2025-11-13T10:00:00Z",
  "uptime": 3600,
  "version": "1.0.0",
  "dependencies": {
    "database": {
      "status": "healthy",
      "response_time_ms": 5,
      "message": "Connected"
    },
    "redis": {
      "status": "healthy",
      "response_time_ms": 2,
      "message": "Connected"
    },
    "blockchain": {
      "status": "healthy",
      "response_time_ms": 150,
      "message": "Connected to Solana RPC"
    }
  }
}
```

---

#### Liveness Check

**GET** `/health/live`

Simple liveness check.

**Response:** `200 OK`
```json
{
  "status": "alive",
  "timestamp": "2025-11-13T10:00:00Z"
}
```

---

## WebSocket API

### Connection

Connect to WebSocket endpoint for real-time updates.

**Endpoint:** `ws://localhost:8080/ws`

**Authentication:** Include JWT token in connection query
```
ws://localhost:8080/ws?token=<your_jwt_token>
```

### Events

#### New Offer Created
```json
{
  "type": "offer_created",
  "data": {
    "offer_id": "770e8400-e29b-41d4-a716-446655440000",
    "energy_amount": 100.0,
    "price_per_kwh": 0.15,
    "energy_source": "solar",
    "seller_username": "john_producer"
  },
  "timestamp": "2025-11-13T10:30:00Z"
}
```

#### New Order Created
```json
{
  "type": "order_created",
  "data": {
    "order_id": "880e8400-e29b-41d4-a716-446655440000",
    "energy_amount": 50.0,
    "max_price_per_kwh": 0.20,
    "preferred_source": "solar"
  },
  "timestamp": "2025-11-13T10:35:00Z"
}
```

#### Trade Match
```json
{
  "type": "trade_matched",
  "data": {
    "transaction_id": "990e8400-e29b-41d4-a716-446655440000",
    "offer_id": "770e8400-e29b-41d4-a716-446655440000",
    "order_id": "880e8400-e29b-41d4-a716-446655440000",
    "energy_amount": 50.0,
    "price_per_kwh": 0.15
  },
  "timestamp": "2025-11-13T10:40:00Z"
}
```

---

## Rate Limiting

### Limits

- **Authentication endpoints**: 10 requests per minute
- **General API**: 100 requests per minute
- **WebSocket connections**: 5 per user

### Headers

Rate limit information in response headers:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1699876800
```

### Error Response

**429 Too Many Requests**
```json
{
  "error": {
    "code": "RATE_LIMIT_9001",
    "message": "Rate limit exceeded. Please try again later.",
    "retry_after": 60
  }
}
```

---

## Examples

### Complete User Flow

#### 1. Register User
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "solar_farm_01",
    "email": "solar@example.com",
    "password": "SecurePass123!",
    "first_name": "Solar",
    "last_name": "Farm",
    "role": "producer"
  }'
```

#### 2. Login
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "solar_farm_01",
    "password": "SecurePass123!"
  }'
```

#### 3. Submit Meter Reading
```bash
curl -X POST http://localhost:8080/api/meters/readings \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "kwh_amount": 150.5,
    "reading_timestamp": "2025-11-13T09:00:00Z"
  }'
```

#### 4. Create Energy Offer
```bash
curl -X POST http://localhost:8080/api/offers \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "energy_amount": 100.0,
    "price_per_kwh": 0.15,
    "energy_source": "solar",
    "available_until": "2025-11-20T00:00:00Z"
  }'
```

#### 5. List Active Offers
```bash
curl http://localhost:8080/api/offers?page=1&energy_source=solar&sort_by=price_per_kwh \
  -H "Authorization: Bearer <token>"
```

#### 6. Create Purchase Order (as Consumer)
```bash
curl -X POST http://localhost:8080/api/orders \
  -H "Authorization: Bearer <consumer_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "energy_amount": 50.0,
    "max_price_per_kwh": 0.20,
    "preferred_source": "solar"
  }'
```

#### 7. View Transactions
```bash
curl http://localhost:8080/api/transactions?page=1&status=completed \
  -H "Authorization: Bearer <token>"
```

---

## Support

For API support and questions:
- **Email**: support@gridtokenx.com
- **Documentation**: https://docs.gridtokenx.com
- **GitHub**: https://github.com/gridtokenx/platform

---

**Last Updated:** November 13, 2025  
**API Version:** 1.0.0
