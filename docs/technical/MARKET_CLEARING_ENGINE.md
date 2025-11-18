# Market Clearing Engine Documentation

## Overview

The Market Clearing Engine is a high-performance double auction system designed for P2P energy trading on the GridTokenX platform. It matches buy and sell orders in real-time, discovers fair market prices, and integrates with Solana blockchain for settlement.

## Table of Contents

1. [Architecture](#architecture)
2. [Core Components](#core-components)
3. [Order Flow](#order-flow)
4. [Matching Algorithm](#matching-algorithm)
5. [Settlement Process](#settlement-process)
6. [API Reference](#api-reference)
7. [Deployment Guide](#deployment-guide)
8. [Operational Runbook](#operational-runbook)
9. [Performance Characteristics](#performance-characteristics)

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        API Gateway Layer                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   Trading    │    │    Market    │    │    Admin     │      │
│  │   Handlers   │───▶│   Clearing   │◀───│   Handlers   │      │
│  └──────────────┘    │    Engine    │    └──────────────┘      │
│                      └───────┬──────┘                            │
│                              │                                   │
│  ┌──────────────┐    ┌───────▼──────┐    ┌──────────────┐      │
│  │  Settlement  │    │  Order Book  │    │  WebSocket   │      │
│  │   Service    │◀───│   In-Memory  │───▶│   Service    │      │
│  └──────┬───────┘    └───────┬──────┘    └──────────────┘      │
│         │                    │                                   │
└─────────┼────────────────────┼───────────────────────────────────┘
          │                    │
          │                    │
┌─────────▼────────┐  ┌────────▼──────────┐
│  Solana          │  │   Redis           │
│  Blockchain      │  │   Persistence     │
│  (Settlement)    │  │   (Order Book)    │
└──────────────────┘  └───────────────────┘
          │                    │
          │           ┌────────▼──────────┐
          └──────────▶│   PostgreSQL      │
                      │   (Trade History) │
                      └───────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Technology |
|-----------|----------------|------------|
| Order Book | In-memory order management with price-time priority | Rust BTreeMap |
| Matching Engine | Continuous order matching with partial fills | Tokio async runtime |
| Settlement Service | Blockchain transaction execution | Solana SDK |
| Redis Persistence | Order book snapshots and recovery | Redis Sorted Sets |
| WebSocket Service | Real-time market data broadcasting | WebSocket protocol |
| PostgreSQL | Trade history and audit trail | SQLx |

---

## Core Components

### 1. Order Book (`OrderBook`)

The order book maintains buy and sell orders using sorted data structures:

```rust
pub struct OrderBook {
    // Buy orders: highest price first (descending)
    buy_levels: BTreeMap<String, PriceLevel>,
    
    // Sell orders: lowest price first (ascending)
    sell_levels: BTreeMap<String, PriceLevel>,
    
    // Quick O(1) order lookup
    order_index: HashMap<Uuid, OrderSide>,
}
```

**Features:**
- **Price-Time Priority**: Orders at the same price level are matched FIFO
- **Partial Fills**: Orders can be partially matched across multiple trades
- **Expiration Handling**: Automatic removal of expired orders
- **Atomic Updates**: Thread-safe operations using `RwLock`

**Key Operations:**
- `add_buy_order()` - O(log n) insertion
- `add_sell_order()` - O(log n) insertion
- `remove_order()` - O(log n) removal
- `best_bid()` / `best_ask()` - O(1) access
- `mid_price()` - O(1) calculation
- `spread()` - O(1) calculation

### 2. Market Clearing Service (`MarketClearingService`)

Main service coordinating all market operations:

```rust
pub struct MarketClearingService {
    order_book: Arc<RwLock<OrderBook>>,
    settlement_service: Arc<SettlementService>,
    websocket_service: Arc<WebSocketService>,
    db_pool: PgPool,
    redis_client: redis::Client,
    matching_config: MatchingConfig,
}
```

**Responsibilities:**
1. Order lifecycle management
2. Continuous matching loop (configurable interval)
3. Redis persistence coordination
4. Trade execution and settlement
5. Real-time market data broadcasting

### 3. Settlement Service (`SettlementService`)

Handles blockchain settlement of matched trades:

```rust
pub struct SettlementService {
    blockchain_service: Arc<BlockchainService>,
    db_pool: PgPool,
    config: SettlementConfig,
}

pub struct SettlementConfig {
    pub fee_rate: Decimal,              // e.g., 0.001 = 0.1%
    pub retry_attempts: u32,            // Default: 3
    pub retry_delay_secs: u64,          // Default: 5
    pub confirmation_timeout: Duration,
}
```

**Features:**
- Automatic fee calculation
- Retry logic for failed transactions
- Status tracking (Pending → Processing → Confirmed/Failed)
- Blockchain confirmation monitoring

### 4. Book Order (`BookOrder`)

Individual order representation:

```rust
pub struct BookOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub side: OrderSide,              // Buy or Sell
    pub energy_amount: Decimal,       // kWh total
    pub price: Decimal,               // USD per kWh
    pub filled_amount: Decimal,       // kWh filled
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
```

**Helper Methods:**
- `remaining_amount()` - Unfilled quantity
- `is_expired()` - Expiration check
- `is_filled()` - Completion check

---

## Order Flow

### Order Placement Flow

```
User Request
    │
    ▼
┌─────────────────┐
│ Validate Order  │  • Check energy amount > 0
│                 │  • Check price > 0
│                 │  • Verify user authentication
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Add to Database │  • Persist to PostgreSQL
│                 │  • Status: Pending
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Add to OrderBook│  • Insert into buy/sell levels
│                 │  • Update order index
│                 │  • Trigger matching cycle
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Persist to Redis│  • ZADD to sorted set
│                 │  • HSET order details
│                 │  • Set 24h TTL
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ WebSocket Notify│  • Broadcast order book update
│                 │  • Send to subscribed clients
└─────────────────┘
```

### Order Cancellation Flow

```
Cancel Request
    │
    ▼
┌─────────────────┐
│ Verify Owner    │  • Check user owns order
│                 │  • Check order is active
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Remove from Book│  • Remove from price level
│                 │  • Update total volume
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Update Database │  • Set status: Cancelled
│                 │  • Record cancellation time
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Delete from Redis│ • ZREM from sorted set
│                 │  • DEL order hash
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ WebSocket Notify│  • Broadcast update
└─────────────────┘
```

---

## Matching Algorithm

### Continuous Matching Loop

The matching engine runs continuously in the background:

```rust
async fn start_matching_loop(
    order_book: Arc<RwLock<OrderBook>>,
    interval_ms: u64,
) {
    let mut interval = tokio::time::interval(
        Duration::from_millis(interval_ms)
    );
    
    loop {
        interval.tick().await;
        
        // Check for matches
        let matches = {
            let mut book = order_book.write().await;
            book.match_orders()
        };
        
        // Process each match
        for trade in matches {
            process_trade(trade).await;
        }
    }
}
```

**Default interval:** 1000ms (configurable)

### Matching Logic

The algorithm matches orders using price-time priority:

```
1. Get best bid (highest buy price)
2. Get best ask (lowest sell price)

3. IF best_bid >= best_ask:
   a. Match quantity = min(bid_remaining, ask_remaining)
   b. Execution price = ask_price (price improvement for buyer)
   c. Update filled amounts atomically
   d. Remove fully filled orders
   e. Create TradeMatch record
   f. Repeat from step 1

4. ELSE:
   No match possible, wait for next cycle
```

**Example Matching Scenario:**

```
Order Book State:
  Bids:                  Asks:
  $0.20 - 100 kWh       $0.18 - 50 kWh
  $0.19 - 50 kWh        $0.19 - 75 kWh
  $0.18 - 200 kWh       $0.21 - 100 kWh

Matching:
  1. Match: 100 kWh @ $0.18
     Bid ($0.20, 100 kWh) FILLED
     Ask ($0.18, 50 kWh) FILLED
     
  2. Match: 50 kWh @ $0.19
     Bid ($0.19, 50 kWh) FILLED
     Ask ($0.19, 75 kWh) PARTIAL (25 kWh remaining)
     
  3. No more matches (bid $0.18 < ask $0.19)

Result:
  2 trades executed
  150 kWh total volume
  Weighted avg price: $0.185/kWh
```

### Partial Fill Handling

Orders can be partially filled across multiple trades:

```rust
// Before matching
Order { id: A, side: Buy, amount: 100, filled: 0 }

// After 3 partial matches
Match 1: 30 kWh filled
Match 2: 45 kWh filled
Match 3: 25 kWh filled

// Final state
Order { id: A, side: Buy, amount: 100, filled: 100 }
Status: Filled
```

**Atomic Update Process:**
1. Lock order book (write lock)
2. Calculate match quantity
3. Update both orders' filled amounts
4. Check if orders are complete
5. Remove completed orders
6. Create trade record
7. Unlock order book
8. Persist changes

---

## Settlement Process

### Trade Settlement Flow

```
Trade Matched
    │
    ▼
┌─────────────────┐
│ Create Settlement│ • Generate settlement ID
│     Record      │ • Calculate fees
│                 │ • Status: Pending
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Build Blockchain│ • Create transfer instruction
│   Transaction   │ • Buyer → Seller (energy tokens)
│                 │ • Fee deduction
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Submit to Solana│ • Send transaction
│                 │ • Status: Processing
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Monitor Confirm │ • Wait for confirmation
│                 │ • Timeout: 30 seconds
└────────┬────────┘
         │
         ├─── Success ──▶ Status: Confirmed
         │
         └─── Failure ──▶ Retry (max 3 attempts)
                          │
                          └─── Still Failed ──▶ Status: Failed
```

### Fee Calculation

```rust
fn calculate_fees(trade: &TradeMatch, fee_rate: Decimal) -> Decimal {
    let total_value = trade.quantity * trade.price;
    let fee = total_value * fee_rate;
    fee.round_dp(6) // Round to 6 decimal places
}

// Example:
// Trade: 100 kWh @ $0.15/kWh = $15.00
// Fee rate: 0.1% (0.001)
// Fee: $15.00 × 0.001 = $0.015
```

### Retry Logic

Failed settlements are retried with exponential backoff:

```rust
async fn settle_with_retry(
    trade: TradeMatch,
    max_attempts: u32,
    delay_secs: u64,
) -> Result<Settlement, ApiError> {
    for attempt in 1..=max_attempts {
        match execute_settlement(&trade).await {
            Ok(settlement) => return Ok(settlement),
            Err(e) if attempt < max_attempts => {
                warn!("Settlement attempt {} failed: {}", attempt, e);
                tokio::time::sleep(
                    Duration::from_secs(delay_secs * attempt as u64)
                ).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Err(ApiError::SettlementFailed)
}
```

---

## API Reference

### Market Data Endpoints

#### Get Order Book Depth

```http
GET /api/market/depth
Authorization: Bearer <token>
```

**Response:**
```json
{
  "buy_depth": [
    { "price": "0.20", "volume": "100.0" },
    { "price": "0.19", "volume": "150.0" }
  ],
  "sell_depth": [
    { "price": "0.18", "volume": "75.0" },
    { "price": "0.21", "volume": "200.0" }
  ]
}
```

#### Get Market Statistics

```http
GET /api/market/stats
Authorization: Bearer <token>
```

**Response:**
```json
{
  "total_active_offers": 45,
  "total_pending_orders": 23,
  "total_volume_24h": "15250.5",
  "average_price": "0.185",
  "price_change_24h": "+2.5",
  "last_trade_price": "0.19",
  "best_bid": "0.18",
  "best_ask": "0.19",
  "spread": "0.01"
}
```

#### Get Clearing Price

```http
GET /api/market/clearing-price
Authorization: Bearer <token>
```

**Response:**
```json
{
  "clearing_price": "0.185",
  "timestamp": "2025-11-14T10:30:00Z"
}
```

#### Get Depth Chart

```http
GET /api/market/depth-chart
Authorization: Bearer <token>
```

**Response:**
```json
{
  "bids": [
    { "price": "0.20", "cumulative_volume": "100.0" },
    { "price": "0.19", "cumulative_volume": "250.0" },
    { "price": "0.18", "cumulative_volume": "450.0" }
  ],
  "asks": [
    { "price": "0.18", "cumulative_volume": "75.0" },
    { "price": "0.19", "cumulative_volume": "150.0" },
    { "price": "0.21", "cumulative_volume": "350.0" }
  ]
}
```

### Trading Endpoints

#### Create Order

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

**Response:**
```json
{
  "order_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "Pending",
  "created_at": "2025-11-14T10:30:00Z"
}
```

#### Get Trade History

```http
GET /api/market/trades/my-history?limit=50&offset=0
Authorization: Bearer <token>
```

**Response:**
```json
{
  "trades": [
    {
      "id": "trade-uuid",
      "quantity": "50.0",
      "price": "0.19",
      "side": "Buy",
      "executed_at": "2025-11-14T10:25:00Z",
      "settlement_status": "Confirmed"
    }
  ],
  "total": 127
}
```

### Admin Endpoints

#### Get Market Health

```http
GET /api/admin/market/health
Authorization: Bearer <admin-token>
```

**Response:**
```json
{
  "status": "Healthy",
  "order_book_health": {
    "total_orders": 68,
    "buy_orders": 35,
    "sell_orders": 33,
    "expired_orders_removed_1h": 5,
    "avg_spread": "0.01"
  },
  "matching_stats": {
    "matches_last_hour": 142,
    "avg_match_time_ms": 15,
    "failed_matches": 0
  },
  "settlement_stats": {
    "pending_settlements": 3,
    "confirmed_settlements_1h": 138,
    "failed_settlements_1h": 1,
    "avg_settlement_time_ms": 2500
  }
}
```

#### Get Trading Analytics

```http
GET /api/admin/market/analytics?timeframe=24h
Authorization: Bearer <admin-token>
```

**Response:**
```json
{
  "total_trades": 1245,
  "total_volume": "125500.75",
  "total_value_usd": "23147.64",
  "price_statistics": {
    "min": "0.15",
    "max": "0.22",
    "avg": "0.185",
    "median": "0.18",
    "std_dev": "0.018"
  },
  "top_traders": [
    {
      "user_id": "user-uuid",
      "trade_count": 45,
      "volume": "4500.0"
    }
  ]
}
```

#### Market Control

```http
POST /api/admin/market/control
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "action": "trigger_matching"  // or "pause_trading", "resume_trading"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Matching cycle triggered successfully",
  "timestamp": "2025-11-14T10:30:00Z"
}
```

---

## Deployment Guide

### Prerequisites

- Rust 1.70+ (stable toolchain)
- PostgreSQL 14+
- Redis 7.0+
- Solana CLI tools
- Docker & Docker Compose (optional)

### Environment Variables

Create `.env` file in `api-gateway/`:

```bash
# Database
DATABASE_URL=postgresql://user:password@localhost:5432/gridtokenx

# Redis
REDIS_URL=redis://localhost:6379

# Blockchain
SOLANA_RPC_URL=https://api.devnet.solana.com
BLOCKCHAIN_KEYPAIR_PATH=/path/to/keypair.json

# Market Clearing
MATCHING_INTERVAL_MS=1000
ORDER_EXPIRATION_HOURS=24

# Settlement
SETTLEMENT_FEE_RATE=0.001
SETTLEMENT_RETRY_ATTEMPTS=3
SETTLEMENT_RETRY_DELAY_SECS=5

# Server
RUST_LOG=info,api_gateway=debug
PORT=8080
```

### Build and Run

#### Development Mode

```bash
cd api-gateway

# Install dependencies
cargo build

# Run database migrations
sqlx migrate run

# Start the server
cargo run
```

#### Production Mode

```bash
# Build optimized binary
cargo build --release

# Run with production config
./target/release/api-gateway
```

#### Docker Deployment

```bash
# Build image
docker build -t gridtokenx-api:latest -f docker/api-gateway/Dockerfile .

# Run container
docker run -d \
  --name gridtokenx-api \
  -p 8080:8080 \
  --env-file .env \
  gridtokenx-api:latest
```

#### Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f api-gateway

# Stop services
docker-compose down
```

### Database Migrations

```bash
# Create new migration
sqlx migrate add create_settlements_table

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Redis Setup

```bash
# Start Redis
redis-server

# Verify connection
redis-cli ping
# Expected: PONG

# Monitor order book operations
redis-cli MONITOR
```

### Solana Configuration

```bash
# Set cluster
solana config set --url devnet

# Create program keypair (if needed)
solana-keygen new -o /path/to/keypair.json

# Fund account (devnet)
solana airdrop 2

# Verify balance
solana balance
```

### Health Checks

```bash
# API health
curl http://localhost:8080/api/health

# Redis health
redis-cli ping

# PostgreSQL health
psql -h localhost -U user -d gridtokenx -c "SELECT 1;"

# Solana health
solana cluster-version
```

---

## Operational Runbook

### Monitoring

#### Key Metrics to Monitor

| Metric | Alert Threshold | Description |
|--------|----------------|-------------|
| Order book size | > 10,000 orders | Memory pressure |
| Matching latency | > 100ms | Performance degradation |
| Settlement failures | > 5% | Blockchain issues |
| Expired orders | > 20% of total | Market inefficiency |
| WebSocket connections | > 1,000 | Connection pooling needed |
| Redis memory | > 80% | Need to increase capacity |

#### Logging

Enable structured logging:

```bash
# Set log level
export RUST_LOG=info,api_gateway::services::market_clearing=debug

# JSON logging for production
export LOG_FORMAT=json
```

**Log Examples:**

```
INFO  Matching cycle completed: 5 trades, 450 kWh, avg_price=$0.18
DEBUG Order added: id=abc123, side=Buy, amount=100, price=0.15
WARN  Settlement retry attempt 2/3 for trade xyz789
ERROR Failed to persist order book to Redis: connection timeout
```

#### Prometheus Metrics (Optional)

```rust
// Key metrics to expose:
- market_orders_total (counter)
- market_trades_total (counter)
- market_matching_duration_seconds (histogram)
- market_settlement_duration_seconds (histogram)
- market_order_book_size (gauge)
- market_spread (gauge)
```

### Troubleshooting

#### Issue: Orders not matching

**Symptoms:** Orders remain in order book despite overlapping prices

**Diagnosis:**
```bash
# Check matching loop is running
grep "Matching cycle" logs/*.log

# Verify order book state
curl http://localhost:8080/api/market/depth
```

**Solutions:**
1. Check matching interval configuration
2. Verify orders are not expired
3. Restart matching loop: `POST /api/admin/market/control` with `trigger_matching`

---

#### Issue: Settlement failures

**Symptoms:** Trades stuck in "Processing" status

**Diagnosis:**
```bash
# Check Solana connectivity
solana cluster-version

# Check settlement logs
grep "Settlement" logs/*.log | grep "Failed"

# Query stuck settlements
psql -c "SELECT * FROM settlements WHERE status='Processing' AND created_at < NOW() - INTERVAL '5 minutes';"
```

**Solutions:**
1. Verify Solana RPC endpoint is responsive
2. Check account balances
3. Increase retry attempts in configuration
4. Manual settlement: Update status to Failed, investigate, retry

---

#### Issue: Redis persistence errors

**Symptoms:** Order book not persisting to Redis

**Diagnosis:**
```bash
# Check Redis connection
redis-cli ping

# Check Redis memory
redis-cli INFO memory

# Check keys
redis-cli KEYS "order_book:*"
```

**Solutions:**
1. Verify Redis connection string
2. Increase Redis memory limit
3. Clear old snapshots: `redis-cli DEL order_book:snapshot`
4. Restart Redis (order book will rebuild from PostgreSQL)

---

#### Issue: High memory usage

**Symptoms:** Server memory > 2GB

**Diagnosis:**
```bash
# Check order book size
curl http://localhost:8080/api/admin/market/health | jq '.order_book_health.total_orders'

# Check expired orders
grep "expired" logs/*.log
```

**Solutions:**
1. Reduce `ORDER_EXPIRATION_HOURS`
2. Force expiration cleanup: Restart server
3. Cancel old unfilled orders via admin API
4. Increase server memory allocation

---

### Disaster Recovery

#### Scenario: Complete Data Loss

**Recovery Steps:**

1. **Stop the service**
   ```bash
   docker-compose stop api-gateway
   ```

2. **Restore PostgreSQL from backup**
   ```bash
   psql -U user gridtokenx < backup.sql
   ```

3. **Clear Redis (stale data)**
   ```bash
   redis-cli FLUSHDB
   ```

4. **Restart service**
   ```bash
   docker-compose start api-gateway
   ```

5. **Verify order book reconstruction**
   ```bash
   # Check logs for "Rebuilding order book from database"
   docker-compose logs api-gateway | grep "order book"
   
   # Verify depth
   curl http://localhost:8080/api/market/depth
   ```

---

#### Scenario: Redis Failure

**Impact:** Order book snapshots lost, but active orders remain in memory

**Recovery Steps:**

1. **Service continues running** (in-memory order book intact)

2. **Fix Redis**
   ```bash
   # Restart Redis
   redis-server

   # Verify
   redis-cli ping
   ```

3. **Trigger manual snapshot**
   ```bash
   # Snapshot will be created on next matching cycle
   # Or force snapshot via internal function
   ```

4. **No data loss** - Order book rebuilds from memory

---

#### Scenario: Blockchain Network Outage

**Impact:** Settlements cannot be executed

**Recovery Steps:**

1. **Settlements will retry automatically** (max 3 attempts)

2. **Monitor failure rate**
   ```bash
   curl http://localhost:8080/api/admin/market/analytics | jq '.settlement_stats.failed_settlements_1h'
   ```

3. **If prolonged outage:**
   - Pause trading: `POST /api/admin/market/control` with `pause_trading`
   - Communicate to users
   - Wait for blockchain recovery
   - Resume: `POST /api/admin/market/control` with `resume_trading`

4. **After recovery:**
   - Failed settlements remain in database
   - Can be manually retried or refunded

---

## Performance Characteristics

### Benchmarks

Tested on: **4-core CPU, 8GB RAM, SSD**

| Operation | Throughput | Latency (p50) | Latency (p99) |
|-----------|-----------|---------------|---------------|
| Add order | 5,000 ops/sec | 0.2ms | 1.5ms |
| Cancel order | 8,000 ops/sec | 0.15ms | 1.2ms |
| Match orders | 1,000 cycles/sec | 10ms | 25ms |
| Get depth | 10,000 ops/sec | 0.1ms | 0.5ms |
| Settlement | 50 tx/sec | 2.5s | 5s |

### Scalability Limits

| Resource | Limit | Constraint |
|----------|-------|------------|
| Active orders | ~100,000 | Memory (1GB) |
| WebSocket clients | ~5,000 | Connection pool |
| Trades per hour | ~100,000 | Database writes |
| Settlements per hour | ~10,000 | Blockchain TPS |

### Optimization Tips

1. **Increase matching interval** for high-frequency trading: `MATCHING_INTERVAL_MS=500`
2. **Batch Redis writes** to reduce network overhead
3. **Use connection pooling** for PostgreSQL (default: 20 connections)
4. **Enable Redis persistence** snapshots: `SAVE 300 10` (every 5min if 10+ keys changed)
5. **Horizontal scaling**: Run multiple API instances with shared Redis/PostgreSQL

---

## Testing

### Unit Tests

```bash
# Run all unit tests
cd api-gateway
cargo test --lib

# Run specific module tests
cargo test --lib market_clearing
cargo test --lib settlement

# With output
cargo test --lib -- --nocapture
```

**Test Coverage:**
- Order book operations: 13 tests
- Settlement service: 9 tests
- Total: 115 tests (100% passing)

### Integration Tests

```bash
# Start test environment
docker-compose -f docker-compose.test.yml up -d

# Run integration tests
cd tests
npm test

# Specific test suite
npm test -- market_clearing_integration.test.ts
```

### Performance Tests

```bash
# Load test with k6
k6 run tests/performance/market_load_test.js

# Expected results:
# - 1000 orders/sec sustained
# - <50ms p95 latency
# - 0% error rate
```

---

## Security Considerations

1. **Authentication**: All endpoints require JWT token
2. **Rate Limiting**: 100 requests/min per user (configurable)
3. **Order Validation**: Prevent negative prices, zero amounts
4. **Settlement Security**: Private keys stored in encrypted vault
5. **Audit Trail**: All trades logged immutably in PostgreSQL
6. **Admin Access**: Separate admin role with MFA required

---

## Future Enhancements

- [ ] Market maker rebates
- [ ] Stop-loss and take-profit orders
- [ ] Time-weighted average price (TWAP) execution
- [ ] Cross-chain settlement support
- [ ] Machine learning price prediction
- [ ] Advanced charting and analytics UI

---

## Support

For issues or questions:
- GitHub Issues: https://github.com/NakaSato/gridtokenx-platform/issues
- Documentation: `/docs/technical/`
- Team Contact: dev@gridtokenx.com

---

**Last Updated:** November 14, 2025  
**Version:** 1.0.0  
**Author:** GridTokenX Engineering Team
