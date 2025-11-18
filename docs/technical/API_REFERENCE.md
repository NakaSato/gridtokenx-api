# Market Clearing Engine - API Reference

Complete API documentation for the Market Clearing Engine.

## Base URL

```
Production: https://api.gridtokenx.com
Development: http://localhost:8080
```

## Authentication

All endpoints require JWT authentication unless specified otherwise.

```http
Authorization: Bearer <jwt_token>
```

### Getting a Token

```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "expires_at": "2025-11-15T10:30:00Z"
}
```

---

## Market Data Endpoints

### Get Order Book Depth

Retrieve current order book depth showing price levels and volumes.

```http
GET /api/market/depth
Authorization: Bearer <token>
```

**Query Parameters:** None

**Response:** `200 OK`

```json
{
  "buy_depth": [
    {
      "price": "0.20",
      "volume": "100.0"
    },
    {
      "price": "0.19",
      "volume": "150.0"
    },
    {
      "price": "0.18",
      "volume": "200.0"
    }
  ],
  "sell_depth": [
    {
      "price": "0.18",
      "volume": "75.0"
    },
    {
      "price": "0.19",
      "volume": "125.0"
    },
    {
      "price": "0.21",
      "volume": "200.0"
    }
  ],
  "timestamp": "2025-11-14T10:30:00Z"
}
```

**Fields:**
- `buy_depth`: Array of buy price levels (highest first)
- `sell_depth`: Array of sell price levels (lowest first)
- `price`: Price per kWh in USD (string for precision)
- `volume`: Total energy amount at price level in kWh

**Errors:**
- `401 Unauthorized`: Invalid or missing authentication token

---

### Get Market Statistics

Retrieve comprehensive market statistics.

```http
GET /api/market/stats
Authorization: Bearer <token>
```

**Query Parameters:**
- `timeframe` (optional): `1h`, `24h`, `7d`, `30d` (default: `24h`)

**Response:** `200 OK`

```json
{
  "total_active_offers": 45,
  "total_pending_orders": 23,
  "total_volume_24h": "15250.5",
  "total_trades_24h": 1245,
  "average_price": "0.185",
  "price_change_24h": "+2.5",
  "last_trade_price": "0.19",
  "last_trade_time": "2025-11-14T10:28:45Z",
  "best_bid": "0.18",
  "best_ask": "0.19",
  "spread": "0.01",
  "spread_percentage": "5.26",
  "timestamp": "2025-11-14T10:30:00Z"
}
```

**Fields:**
- `total_active_offers`: Count of active offers (includes orders)
- `total_pending_orders`: Count of unfilled orders
- `total_volume_24h`: Total trading volume in kWh
- `total_trades_24h`: Number of executed trades
- `average_price`: Volume-weighted average price
- `price_change_24h`: Price change percentage (string with +/- sign)
- `last_trade_price`: Most recent execution price
- `best_bid`: Highest buy price currently available
- `best_ask`: Lowest sell price currently available
- `spread`: Difference between best ask and best bid
- `spread_percentage`: Spread as percentage of mid-price

---

### Get Clearing Price

Retrieve the current market clearing price.

```http
GET /api/market/clearing-price
Authorization: Bearer <token>
```

**Response:** `200 OK`

```json
{
  "clearing_price": "0.185",
  "volume_at_price": "450.0",
  "timestamp": "2025-11-14T10:30:00Z"
}
```

**Fields:**
- `clearing_price`: Current equilibrium price (null if no recent trades)
- `volume_at_price`: Volume traded at this price in last hour
- `timestamp`: Time of calculation

**Errors:**
- `404 Not Found`: No clearing price available (no recent trades)

---

### Get Depth Chart Data

Retrieve cumulative order book depth for charting.

```http
GET /api/market/depth-chart
Authorization: Bearer <token>
```

**Query Parameters:**
- `levels` (optional): Number of price levels (default: `20`, max: `100`)

**Response:** `200 OK`

```json
{
  "bids": [
    {
      "price": "0.20",
      "cumulative_volume": "100.0"
    },
    {
      "price": "0.19",
      "cumulative_volume": "250.0"
    },
    {
      "price": "0.18",
      "cumulative_volume": "450.0"
    }
  ],
  "asks": [
    {
      "price": "0.18",
      "cumulative_volume": "75.0"
    },
    {
      "price": "0.19",
      "cumulative_volume": "200.0"
    },
    {
      "price": "0.21",
      "cumulative_volume": "400.0"
    }
  ],
  "mid_price": "0.19",
  "timestamp": "2025-11-14T10:30:00Z"
}
```

**Usage Example:**

This data can be plotted as a depth chart showing cumulative buy/sell volumes at different price points.

---

### Get Recent Trades

Retrieve recent market trades (public).

```http
GET /api/market/trades/recent
Authorization: Bearer <token>
```

**Query Parameters:**
- `limit` (optional): Number of trades (default: `50`, max: `200`)

**Response:** `200 OK`

```json
{
  "trades": [
    {
      "id": "trade-uuid-1",
      "quantity": "50.0",
      "price": "0.19",
      "timestamp": "2025-11-14T10:28:45Z",
      "side": "Buy"
    },
    {
      "id": "trade-uuid-2",
      "quantity": "75.5",
      "price": "0.185",
      "timestamp": "2025-11-14T10:27:30Z",
      "side": "Sell"
    }
  ],
  "total": 1245
}
```

**Fields:**
- `id`: Trade identifier
- `quantity`: Energy amount traded in kWh
- `price`: Execution price per kWh
- `timestamp`: Trade execution time
- `side`: Initiating side (`Buy` or `Sell`)
- `total`: Total trades in selected timeframe

---

### Get My Trade History

Retrieve authenticated user's trade history.

```http
GET /api/market/trades/my-history
Authorization: Bearer <token>
```

**Query Parameters:**
- `limit` (optional): Number of trades (default: `50`, max: `200`)
- `offset` (optional): Pagination offset (default: `0`)
- `from_date` (optional): ISO 8601 date filter
- `to_date` (optional): ISO 8601 date filter

**Response:** `200 OK`

```json
{
  "trades": [
    {
      "id": "trade-uuid",
      "order_id": "order-uuid",
      "quantity": "50.0",
      "price": "0.19",
      "total_value": "9.50",
      "fee": "0.0095",
      "side": "Buy",
      "executed_at": "2025-11-14T10:25:00Z",
      "settlement_status": "Confirmed",
      "settlement_tx": "3xK8...9mPq",
      "counterparty_id": "user-uuid"
    }
  ],
  "total": 127,
  "limit": 50,
  "offset": 0
}
```

**Fields:**
- `settlement_status`: `Pending`, `Processing`, `Confirmed`, `Failed`, `Cancelled`
- `settlement_tx`: Solana transaction signature (if confirmed)
- `counterparty_id`: Other party in the trade
- `fee`: Trading fee charged

---

## Trading Endpoints

### Create Order

Place a new buy or sell order.

```http
POST /api/trading/orders
Authorization: Bearer <token>
Content-Type: application/json

{
  "order_type": "Limit",
  "side": "Buy",
  "energy_amount": 100.0,
  "price": 0.15
}
```

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `order_type` | string | Yes | `"Limit"` (market orders not yet supported) |
| `side` | string | Yes | `"Buy"` or `"Sell"` |
| `energy_amount` | number | Yes | Energy amount in kWh (> 0) |
| `price` | number | Yes | Price per kWh in USD (> 0) |
| `expires_in_hours` | number | No | Expiration time (default: 24, max: 168) |

**Response:** `201 Created`

```json
{
  "order_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "Pending",
  "created_at": "2025-11-14T10:30:00Z",
  "expires_at": "2025-11-15T10:30:00Z"
}
```

**Validation Errors:** `400 Bad Request`

```json
{
  "error": "Validation failed",
  "details": [
    "energy_amount must be greater than 0",
    "price must be greater than 0"
  ]
}
```

**Other Errors:**
- `401 Unauthorized`: Invalid authentication
- `403 Forbidden`: User not authorized to trade
- `429 Too Many Requests`: Rate limit exceeded

---

### Get My Orders

Retrieve authenticated user's orders.

```http
GET /api/trading/orders
Authorization: Bearer <token>
```

**Query Parameters:**
- `status` (optional): Filter by status (`Pending`, `PartiallyFilled`, `Filled`, `Cancelled`)
- `side` (optional): Filter by side (`Buy`, `Sell`)
- `limit` (optional): Number of orders (default: `50`, max: `200`)
- `offset` (optional): Pagination offset

**Response:** `200 OK`

```json
{
  "orders": [
    {
      "id": "order-uuid",
      "side": "Buy",
      "energy_amount": "100.0",
      "filled_amount": "45.0",
      "remaining_amount": "55.0",
      "price": "0.15",
      "status": "PartiallyFilled",
      "created_at": "2025-11-14T10:00:00Z",
      "expires_at": "2025-11-15T10:00:00Z",
      "trades": [
        {
          "trade_id": "trade-uuid",
          "quantity": "45.0",
          "executed_at": "2025-11-14T10:15:00Z"
        }
      ]
    }
  ],
  "total": 15,
  "limit": 50,
  "offset": 0
}
```

---

### Get Order Details

Retrieve details of a specific order.

```http
GET /api/trading/orders/{order_id}
Authorization: Bearer <token>
```

**Path Parameters:**
- `order_id`: UUID of the order

**Response:** `200 OK`

```json
{
  "id": "order-uuid",
  "user_id": "user-uuid",
  "side": "Buy",
  "energy_amount": "100.0",
  "filled_amount": "100.0",
  "price": "0.15",
  "status": "Filled",
  "created_at": "2025-11-14T09:00:00Z",
  "updated_at": "2025-11-14T09:30:00Z",
  "expires_at": "2025-11-15T09:00:00Z",
  "trades": [
    {
      "trade_id": "trade-uuid-1",
      "quantity": "50.0",
      "price": "0.15",
      "executed_at": "2025-11-14T09:15:00Z"
    },
    {
      "trade_id": "trade-uuid-2",
      "quantity": "50.0",
      "price": "0.15",
      "executed_at": "2025-11-14T09:30:00Z"
    }
  ]
}
```

**Errors:**
- `404 Not Found`: Order does not exist
- `403 Forbidden`: User does not own this order

---

### Cancel Order

Cancel an active order.

```http
DELETE /api/trading/orders/{order_id}
Authorization: Bearer <token>
```

**Path Parameters:**
- `order_id`: UUID of the order

**Response:** `200 OK`

```json
{
  "success": true,
  "order_id": "order-uuid",
  "status": "Cancelled",
  "cancelled_at": "2025-11-14T10:30:00Z",
  "refunded_amount": "55.0"
}
```

**Errors:**
- `404 Not Found`: Order does not exist
- `403 Forbidden`: User does not own this order
- `409 Conflict`: Order already filled or cancelled

---

## Admin Endpoints

**Note:** Requires admin role authentication.

### Get Market Health

Retrieve comprehensive market health metrics.

```http
GET /api/admin/market/health
Authorization: Bearer <admin-token>
```

**Response:** `200 OK`

```json
{
  "status": "Healthy",
  "timestamp": "2025-11-14T10:30:00Z",
  "order_book_health": {
    "total_orders": 68,
    "buy_orders": 35,
    "sell_orders": 33,
    "expired_orders_removed_1h": 5,
    "avg_spread": "0.01",
    "spread_percentage": "5.26",
    "best_bid": "0.18",
    "best_ask": "0.19",
    "depth_imbalance": "5.71"
  },
  "matching_stats": {
    "matches_last_hour": 142,
    "matches_last_minute": 3,
    "avg_match_time_ms": 15,
    "max_match_time_ms": 45,
    "failed_matches": 0,
    "matching_loop_running": true,
    "last_matching_cycle": "2025-11-14T10:29:58Z"
  },
  "settlement_stats": {
    "pending_settlements": 3,
    "processing_settlements": 1,
    "confirmed_settlements_1h": 138,
    "failed_settlements_1h": 1,
    "avg_settlement_time_ms": 2500,
    "max_settlement_time_ms": 4800,
    "success_rate_1h": "99.28"
  },
  "system_health": {
    "redis_connected": true,
    "postgres_connected": true,
    "solana_connected": true,
    "websocket_clients": 47,
    "memory_usage_mb": 256,
    "cpu_usage_percent": 12.5
  }
}
```

**Fields:**
- `status`: Overall health status (`Healthy`, `Degraded`, `Critical`)
- `depth_imbalance`: Percentage difference between buy and sell volumes
- `success_rate_1h`: Settlement success rate in last hour

---

### Get Trading Analytics

Retrieve detailed trading analytics.

```http
GET /api/admin/market/analytics
Authorization: Bearer <admin-token>
```

**Query Parameters:**
- `timeframe` (optional): `1h`, `24h`, `7d`, `30d` (default: `24h`)
- `group_by` (optional): `hour`, `day`, `week` (for time series data)

**Response:** `200 OK`

```json
{
  "timeframe": "24h",
  "total_trades": 1245,
  "total_volume": "125500.75",
  "total_value_usd": "23147.64",
  "unique_traders": 89,
  "price_statistics": {
    "min": "0.15",
    "max": "0.22",
    "avg": "0.185",
    "median": "0.18",
    "std_dev": "0.018",
    "vwap": "0.184"
  },
  "volume_statistics": {
    "min_trade": "10.0",
    "max_trade": "500.0",
    "avg_trade": "100.8",
    "median_trade": "85.0"
  },
  "top_traders": [
    {
      "user_id": "user-uuid",
      "trade_count": 45,
      "volume": "4500.0",
      "value_usd": "832.50"
    }
  ],
  "hourly_breakdown": [
    {
      "hour": "2025-11-14T10:00:00Z",
      "trades": 52,
      "volume": "5234.5",
      "avg_price": "0.185"
    }
  ]
}
```

**Fields:**
- `vwap`: Volume-weighted average price
- `hourly_breakdown`: Time series data (if `group_by` specified)

---

### Market Control

Control market operations (pause, resume, trigger matching).

```http
POST /api/admin/market/control
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "action": "trigger_matching"
}
```

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action` | string | Yes | `"pause_trading"`, `"resume_trading"`, `"trigger_matching"`, `"clear_expired"` |
| `reason` | string | No | Reason for action (logged) |

**Actions:**
- `pause_trading`: Stop accepting new orders and pause matching
- `resume_trading`: Resume normal operations
- `trigger_matching`: Force immediate matching cycle
- `clear_expired`: Manually remove all expired orders

**Response:** `200 OK`

```json
{
  "success": true,
  "action": "trigger_matching",
  "message": "Matching cycle triggered successfully",
  "result": {
    "matches_found": 5,
    "volume_matched": "450.0",
    "execution_time_ms": 12
  },
  "timestamp": "2025-11-14T10:30:00Z"
}
```

**Errors:**
- `403 Forbidden`: Not an admin user
- `400 Bad Request`: Invalid action

---

## WebSocket API

Real-time market data streaming.

### Connection

```javascript
const ws = new WebSocket('wss://api.gridtokenx.com/ws');

// Authenticate
ws.send(JSON.stringify({
  type: 'auth',
  token: 'your-jwt-token'
}));
```

### Subscribe to Channels

```javascript
// Subscribe to order book updates
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'orderbook'
}));

// Subscribe to trade executions
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'trades'
}));

// Subscribe to market statistics
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'stats'
}));
```

### Message Formats

**Order Book Update:**
```json
{
  "type": "orderbook",
  "data": {
    "buy_depth": [...],
    "sell_depth": [...],
    "timestamp": "2025-11-14T10:30:00Z"
  }
}
```

**Trade Execution:**
```json
{
  "type": "trade",
  "data": {
    "id": "trade-uuid",
    "quantity": "50.0",
    "price": "0.19",
    "side": "Buy",
    "timestamp": "2025-11-14T10:30:00Z"
  }
}
```

**Market Statistics:**
```json
{
  "type": "stats",
  "data": {
    "best_bid": "0.18",
    "best_ask": "0.19",
    "last_price": "0.19",
    "spread": "0.01",
    "volume_24h": "15250.5",
    "timestamp": "2025-11-14T10:30:00Z"
  }
}
```

---

## Rate Limits

| Endpoint Type | Rate Limit | Window |
|--------------|------------|--------|
| Market Data | 100 req/min | Per user |
| Trading | 30 req/min | Per user |
| Admin | 200 req/min | Per admin |
| WebSocket | 10 msg/sec | Per connection |

**Rate Limit Headers:**

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1699963200
```

**Rate Limit Error:** `429 Too Many Requests`

```json
{
  "error": "Rate limit exceeded",
  "retry_after": 45
}
```

---

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| 400 | Bad Request | Invalid request parameters |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource does not exist |
| 409 | Conflict | Resource state conflict |
| 422 | Unprocessable Entity | Validation failed |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |
| 503 | Service Unavailable | Service temporarily unavailable |

**Error Response Format:**

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "details": ["Additional detail 1", "Additional detail 2"],
  "timestamp": "2025-11-14T10:30:00Z",
  "request_id": "req-uuid"
}
```

---

## Postman Collection

Import our Postman collection for easy API testing:

```
/api-gateway/postman/market-clearing-api.postman_collection.json
```

---

**Last Updated:** November 14, 2025  
**API Version:** 1.0.0
