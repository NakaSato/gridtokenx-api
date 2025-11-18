# Market Clearing Engine Design Document

**Last Updated**: November 9, 2025  
**Status**: Architecture Complete ✅  
**Implementation**: In Progress ⏳  

---

## 📋 OVERVIEW

The Market Clearing Engine is the core component of GridTokenX's energy trading platform. It implements a double-sided auction mechanism where buyers and sellers submit energy orders that are matched during discrete time intervals (epochs).

### Key Features
- **15-minute epoch intervals** for continuous trading
- **Price-time priority matching** algorithm
- **Partial fill support** for large orders
- **Automated settlement** processing
- **Real-time order book** management
- **Recovery mechanisms** for server restarts

---

## 🏗️ ARCHITECTURE

### Core Components

#### 1. Epoch Scheduler Service (`epoch_scheduler.rs`)
```
EpochScheduler
├── Current epoch tracking
├── Automatic epoch transitions (15-min intervals)
├── State machine (pending → active → clearing → settled)
├── Recovery from restarts
└── Manual epoch triggering for testing
```

**Key Functions:**
- `start()` - Initialize scheduler with tokio interval timer
- `get_current_epoch()` - Retrieve active epoch
- `trigger_epoch_clearing()` - Manual epoch processing
- `recover_from_restart()` - Restore state on startup

#### 2. Order Matcher Service (`order_matcher.rs`)
```
OrderMatcher
├── Buy order book (price desc, time asc)
├── Sell order book (price asc, time asc)
├── Matching algorithm (price-time priority)
├── Partial fill logic
└── Clearing price calculation
```

**Algorithm:**
1. Sort buy orders by highest price, then earliest time
2. Sort sell orders by lowest price, then earliest time
3. Match while best buy price ≥ best sell price
4. Use sell price as clearing price (market mechanism)
5. Handle partial fills for order size mismatches

#### 3. Market Clearing Service (`market_clearing_service.rs`)
```
MarketClearingService
├── Epoch management (create, update, query)
├── Order book aggregation
├── Order matching orchestration
├── Settlement creation
├── Trading history
└── Market statistics
```

**Key Functions:**
- `run_order_matching()` - Execute matching for an epoch
- `create_settlement()` - Generate settlement records
- `get_order_book()` - Retrieve current orders
- `cancel_order()` - Order cancellation logic

---

## 📊 DATA MODELS

### Market Epoch
```rust
pub struct MarketEpoch {
    pub id: Uuid,
    pub epoch_number: i64,           // YYYYMMDDHHMM format
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: String,               // pending/active/cleared/expired
    pub clearing_price: Option<BigDecimal>,
    pub total_volume: BigDecimal,
    pub total_orders: i64,
    pub matched_orders: i64,
}
```

### Order Match
```rust
pub struct OrderMatch {
    pub id: Uuid,
    pub epoch_id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub matched_amount: BigDecimal,
    pub match_price: BigDecimal,
    pub match_time: DateTime<Utc>,
    pub status: String,              // pending/settled/failed
}
```

### Settlement
```rust
pub struct Settlement {
    pub id: Uuid,
    pub epoch_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub energy_amount: BigDecimal,
    pub price_per_kwh: BigDecimal,
    pub total_amount: BigDecimal,
    pub fee_amount: BigDecimal,         // 1% transaction fee
    pub net_amount: BigDecimal,
    pub status: String,              // pending/processed/failed
}
```

---

## 🔄 ORDER MATCHING ALGORITHM

### Price-Time Priority Rules
1. **Price Priority**: Higher buy prices and lower sell prices get priority
2. **Time Priority**: Earlier orders get priority at same price
3. **Market Clearing Price**: Use sell order price (favoring sellers)
4. **Partial Fills**: Large orders can be partially matched

### Matching Process
```rust
while let Some(buy_order) = buy_orders.first_mut() {
    if let Some(sell_order) = sell_orders.first_mut() {
        if buy_order.price_per_kwh >= sell_order.price_per_kwh {
            let match_amount = buy_order.energy_amount.min(sell_order.energy_amount);
            let match_price = sell_order.price_per_kwh; // Clearing price
            
            // Create match, update orders, continue
        } else {
            break; // No more matches possible
        }
    }
}
```

### Edge Cases Handled
- **Single-sided markets**: No matches when only buyers or sellers
- **Zero volume orders**: Orders with 0 kWh are ignored
- **Same price different times**: Time priority determines matching
- **Large order mismatches**: Partial fills with remainder orders

---

## 💾 DATABASE SCHEMA

### Market Epochs Table
```sql
CREATE TABLE market_epochs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    epoch_number BIGINT UNIQUE NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    status VARCHAR(20) NOT NULL,
    clearing_price NUMERIC(20, 8),
    total_volume NUMERIC(20, 8) DEFAULT 0,
    total_orders BIGINT DEFAULT 0,
    matched_orders BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_market_epochs_time ON market_epochs(start_time, end_time);
CREATE INDEX idx_market_epochs_status ON market_epochs(status);
CREATE INDEX idx_market_epochs_number ON market_epochs(epoch_number);
```

### Order Matches Table
```sql
CREATE TABLE order_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    epoch_id UUID NOT NULL REFERENCES market_epochs(id),
    buy_order_id UUID NOT NULL REFERENCES trading_orders(id),
    sell_order_id UUID NOT NULL REFERENCES trading_orders(id),
    matched_amount NUMERIC(20, 8) NOT NULL,
    match_price NUMERIC(20, 8) NOT NULL,
    match_time TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    settlement_id UUID REFERENCES settlements(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_order_matches_epoch ON order_matches(epoch_id);
CREATE INDEX idx_order_matches_orders ON order_matches(buy_order_id, sell_order_id);
CREATE INDEX idx_order_matches_status ON order_matches(status);
```

### Settlements Table
```sql
CREATE TABLE settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    epoch_id UUID NOT NULL REFERENCES market_epochs(id),
    buyer_id UUID NOT NULL REFERENCES users(id),
    seller_id UUID NOT NULL REFERENCES users(id),
    energy_amount NUMERIC(20, 8) NOT NULL,
    price_per_kwh NUMERIC(20, 8) NOT NULL,
    total_amount NUMERIC(20, 8) NOT NULL,
    fee_amount NUMERIC(20, 8) NOT NULL,
    net_amount NUMERIC(20, 8) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    transaction_hash VARCHAR(66),     -- Blockchain transaction reference
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_settlements_epoch ON settlements(epoch_id);
CREATE INDEX idx_settlements_users ON settlements(buyer_id, seller_id);
CREATE INDEX idx_settlements_status ON settlements(status);
```

---

## ⚡ PERFORMANCE OPTIMIZATIONS

### Database Indexes
- **Time-based indexes** for epoch queries
- **Status indexes** for filtering active orders
- **Composite indexes** for common query patterns

### Memory Management
- **Order book caching** during matching
- **Batch database operations** for settlements
- **Connection pooling** for high concurrency

### Algorithm Efficiency
- **O(n log n)** sorting for order books
- **O(m)** matching where m = min(buyers, sellers)
- **Early termination** when no matches possible

---

## 🔄 RECOVERY MECHANISMS

### Server Restart Recovery
1. **Epoch State Recovery**: Find incomplete epochs and resume
2. **Order Recovery**: Reconstruct order books from pending orders
3. **Settlement Recovery**: Process any orphaned settlements
4. **Timer Restart**: Resume 15-minute interval schedule

### Recovery Process
```rust
pub async fn recover_from_restart(&self) -> Result<()> {
    // 1. Find incomplete epochs
    let incomplete_epochs = self.get_incomplete_epochs().await?;
    
    // 2. Resume processing for stuck epochs
    for epoch in incomplete_epochs {
        if epoch.status == "active" {
            self.run_order_matching(epoch.id).await?;
        }
    }
    
    // 3. Restart epoch timer
    self.start_epoch_timer().await?;
    
    Ok(())
}
```

---

## 🔧 API ENDPOINTS

### Market Data Endpoints
```
GET /api/v1/market/current-epoch     # Get active epoch
GET /api/v1/market/order-book      # Get current order book
GET /api/v1/market/history         # Get trading history
GET /api/v1/market/statistics      # Get market statistics
```

### Admin Endpoints
```
POST /api/v1/admin/trigger-epoch  # Manual epoch clearing
GET /api/v1/admin/epochs          # List all epochs
POST /api/v1/admin/recover         # Trigger recovery process
```

---

## 📈 MONITORING & METRICS

### Key Performance Indicators
- **Order matching latency** (< 1 second for 1000 orders)
- **Epoch processing time** (< 5 seconds per epoch)
- **Settlement success rate** (> 99%)
- **Order book depth** (real-time monitoring)
- **Price discovery efficiency** (spread analysis)

### Logging Strategy
```rust
// Structured logging for market operations
info!("Epoch {} started with {} orders", epoch_id, order_count);
info!("Order matched: {} kWh at {} price", amount, price);
warn!("Settlement failed for match {}: {}", match_id, error);
error!("Epoch processing failed: {}", error);
```

---

## 🧪 TESTING STRATEGY

### Unit Tests
- **Epoch scheduler** - Timer accuracy and state transitions
- **Order matching** - Algorithm correctness and edge cases
- **Settlement logic** - Fee calculation and amount validation
- **Recovery mechanisms** - Restart scenarios

### Integration Tests
- **End-to-end workflows** - Order to settlement pipeline
- **Performance tests** - Load testing with 10K+ orders
- **Concurrent operations** - Multi-user trading scenarios
- **Failure scenarios** - Database disconnection, timeout handling

### Test Scenarios
```rust
#[tokio::test]
async fn test_perfect_match_scenario() {
    // Equal buy/sell volumes at overlapping prices
    // Verify all orders fully matched
}

#[tokio::test] 
async fn test_partial_fill_scenario() {
    // Excess supply or demand
    // Verify partial fills and remaining orders
}

#[tokio::test]
async fn test_no_match_scenario() {
    // Non-overlapping price ranges
    // Verify no matches and unchanged order book
}
```

---

## 🔮 FUTURE ENHANCEMENTS

### Phase 2 Features
- **Continuous double auction** with intra-epoch matching
- **Advanced order types** (limit, market, iceberg)
- **Price improvement** mechanisms
- **Cross-epoch** order persistence

### Performance Scaling
- **Horizontal scaling** with multiple matching engines
- **Redis-based** order book caching
- **Event streaming** for real-time updates
- **Database sharding** for high volume

### Integration Opportunities
- **Grid operator APIs** for real-time demand data
- **Weather forecasting** for supply prediction
- **ML-based** price recommendations
- **Smart contract** integration for settlements

---

## 📚 REFERENCE IMPLEMENTATION

### Current Status
✅ **Epoch Scheduler**: Complete with tokio timer implementation  
✅ **Order Matcher**: Complete with price-time priority algorithm  
✅ **Market Clearing Service**: Complete with settlement logic  
✅ **Database Schema**: Migrations created and tested  
✅ **Unit Tests**: Comprehensive test coverage implemented  

### Next Steps
⏳ **Integration Testing**: End-to-end workflow validation  
⏳ **Performance Testing**: Load testing with high volumes  
⏳ **WebSocket Integration**: Real-time order book updates  
⏳ **Settlement Service**: Blockchain transaction processing  

---

**Document Version**: 1.0  
**Last Review**: November 9, 2025  
**Next Review**: After integration testing completion
