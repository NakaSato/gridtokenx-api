# Energy Trading Implementation Plan

## Test Results Summary

### ✅ What Worked
- API Gateway is running
- User registration works (but roles need fixing)
- Email verification system works

### ❌ What Needs Implementation

#### 1. User Roles
**Issue:** Invalid role error for 'producer' and 'consumer'
**Fix:** Update user registration to accept these roles

#### 2. Missing API Endpoints (All return 404)
- `GET /api/offers` - List energy offers
- `POST /api/offers` - Create energy offer  
- `GET /api/offers/:id` - Get offer details
- `PATCH /api/offers/:id` - Update/cancel offer
- `GET /api/orders` - List energy orders
- `POST /api/orders` - Create energy order
- `GET /api/transactions` - List transactions
- `GET /api/transactions/:id` - Get transaction details
- `GET /api/market/stats` - Market statistics
- `GET /api/market/price-history` - Price history

#### 3. Missing Database Tables
- `offers` - Energy sell offers
- `orders` - Energy buy orders
- `transactions` - Matched trades
- `market_stats` - Aggregated market data

## Implementation Steps

### Phase 1: Database Schema (PRIORITY)
Create tables for trading system

### Phase 2: API Endpoints - Rust
Implement in `api-gateway/src/routes/`

### Phase 3: Business Logic
- Order matching engine
- Price calculation
- Transaction settlement

### Phase 4: Real-time Updates
- WebSocket notifications
- Kafka events

## Database Schema

```sql
-- Energy offers (sell)
CREATE TABLE offers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_by UUID NOT NULL REFERENCES users(id),
    energy_amount DECIMAL(10,2) NOT NULL CHECK (energy_amount > 0),
    price_per_kwh DECIMAL(10,4) NOT NULL CHECK (price_per_kwh > 0),
    offer_type VARCHAR(10) NOT NULL DEFAULT 'sell',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    available_from TIMESTAMPTZ NOT NULL,
    available_until TIMESTAMPTZ NOT NULL,
    location VARCHAR(255),
    energy_source VARCHAR(50), -- solar, wind, hydro, etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_dates CHECK (available_until > available_from)
);

-- Energy orders (buy)
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_by UUID NOT NULL REFERENCES users(id),
    energy_amount DECIMAL(10,2) NOT NULL CHECK (energy_amount > 0),
    max_price_per_kwh DECIMAL(10,4) NOT NULL CHECK (max_price_per_kwh > 0),
    order_type VARCHAR(10) NOT NULL DEFAULT 'buy',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    required_from TIMESTAMPTZ NOT NULL,
    required_until TIMESTAMPTZ NOT NULL,
    preferred_source VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_dates CHECK (required_until > required_from)
);

-- Matched transactions
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    offer_id UUID NOT NULL REFERENCES offers(id),
    order_id UUID NOT NULL REFERENCES orders(id),
    seller_id UUID NOT NULL REFERENCES users(id),
    buyer_id UUID NOT NULL REFERENCES users(id),
    energy_amount DECIMAL(10,2) NOT NULL,
    price_per_kwh DECIMAL(10,4) NOT NULL,
    total_price DECIMAL(10,2) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    blockchain_tx_hash VARCHAR(255),
    settled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id)
);

-- Indexes for performance
CREATE INDEX idx_offers_status ON offers(status);
CREATE INDEX idx_offers_created_by ON offers(created_by);
CREATE INDEX idx_offers_energy_source ON offers(energy_source);
CREATE INDEX idx_offers_dates ON offers(available_from, available_until);

CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_by ON orders(created_by);
CREATE INDEX idx_orders_dates ON orders(required_from, required_until);

CREATE INDEX idx_transactions_offer ON transactions(offer_id);
CREATE INDEX idx_transactions_order ON transactions(order_id);
CREATE INDEX idx_transactions_seller ON transactions(seller_id);
CREATE INDEX idx_transactions_buyer ON transactions(buyer_id);
CREATE INDEX idx_transactions_status ON transactions(status);
```

## API Endpoints Specification

### Offers

```rust
// POST /api/offers
// Create energy offer (producers only)
struct CreateOfferRequest {
    energy_amount: f64,
    price_per_kwh: f64,
    available_from: DateTime,
    available_until: DateTime,
    location: Option<String>,
    energy_source: Option<String>,
}

// GET /api/offers
// List offers with filtering
struct ListOffersQuery {
    status: Option<String>,
    energy_source: Option<String>,
    min_price: Option<f64>,
    max_price: Option<f64>,
    limit: Option<i32>,
    offset: Option<i32>,
}

// PATCH /api/offers/:id
// Update offer (owner only)
struct UpdateOfferRequest {
    status: Option<String>, // active, cancelled, completed
    energy_amount: Option<f64>,
    price_per_kwh: Option<f64>,
}
```

### Orders

```rust
// POST /api/orders
// Create energy order (consumers only)
struct CreateOrderRequest {
    energy_amount: f64,
    max_price_per_kwh: f64,
    required_from: DateTime,
    required_until: DateTime,
    preferred_source: Option<String>,
}

// GET /api/orders
// List user's orders
struct ListOrdersQuery {
    status: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
}
```

### Transactions

```rust
// GET /api/transactions
// List user's transactions
struct ListTransactionsQuery {
    status: Option<String>,
    order_id: Option<Uuid>,
    limit: Option<i32>,
    offset: Option<i32>,
}

// GET /api/transactions/:id
// Get transaction details
```

### Market Data

```rust
// GET /api/market/stats
// Get market statistics
struct MarketStats {
    average_price: f64,
    total_volume: f64,
    active_offers: i32,
    active_orders: i32,
}

// GET /api/market/price-history
// Get historical prices
struct PriceHistoryQuery {
    period: String, // 1h, 24h, 7d, 30d
    energy_source: Option<String>,
}
```

## Next Steps

1. ✅ Create database migration
2. ✅ Update user roles enum
3. ✅ Implement offer endpoints
4. ✅ Implement order endpoints
5. ✅ Implement transaction endpoints
6. ✅ Implement market endpoints
7. ✅ Create matching engine
8. ✅ Add WebSocket notifications

## Testing Strategy

- [x] Integration tests created
- [ ] Database schema created
- [ ] API endpoints implemented
- [ ] Matching engine implemented
- [ ] All tests passing
