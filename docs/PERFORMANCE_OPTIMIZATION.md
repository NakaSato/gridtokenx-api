# Performance Optimization Guide

**Date**: November 13, 2025  
**Version**: 1.0  
**Status**: Implemented

## Overview

This guide covers the performance optimization features implemented for the GridTokenX API Gateway, including database indexing and Redis caching strategies.

## Table of Contents

1. [Database Indexes](#database-indexes)
2. [Redis Caching](#redis-caching)
3. [Performance Monitoring](#performance-monitoring)
4. [Best Practices](#best-practices)
5. [Troubleshooting](#troubleshooting)

---

## Database Indexes

### Overview

We've added **50+ strategic indexes** across all major tables to improve query performance. These indexes target the most common query patterns:

- User authentication and profile lookups
- Meter reading queries
- Trading order matching
- Transaction history
- ERC certificate management

### Index Categories

#### 1. User Table Indexes

```sql
-- Authentication and lookup
idx_users_wallet_address    -- Wallet-based login
idx_users_email            -- Email-based login
idx_users_username         -- Username-based login
idx_users_role             -- Role filtering
idx_users_email_verified   -- Unverified users
idx_users_role_created     -- User listing with role filter
```

**Use Cases**:
- User login (wallet, email, username)
- User search and filtering
- Role-based queries
- Email verification flow

#### 2. Meter Readings Indexes

```sql
-- Primary queries
idx_meter_readings_user_id        -- User's readings
idx_meter_readings_wallet         -- Wallet readings
idx_meter_readings_minted         -- Unminted readings
idx_meter_readings_user_minted    -- User + minted filter
idx_meter_readings_timestamp      -- Time-based queries
idx_meter_readings_tx_signature   -- Blockchain lookup
```

**Use Cases**:
- User reading history
- Unminted readings for token minting
- Time-based analytics
- Blockchain transaction tracking

#### 3. Trading Orders Indexes

```sql
-- Order management
idx_trading_orders_user_id        -- User's orders
idx_trading_orders_status         -- Active/filled orders
idx_trading_orders_user_status    -- User + status filter
idx_trading_orders_side           -- Buy/sell filtering
idx_trading_orders_status_side    -- Order book queries
idx_trading_orders_price          -- Price-based matching
idx_trading_orders_filled_at      -- Transaction history
```

**Use Cases**:
- User order history
- Active order listing
- Order book construction
- Price-based matching
- Transaction history

#### 4. Energy Offers Indexes

```sql
-- Marketplace queries
idx_energy_offers_seller_id       -- Seller's offers
idx_energy_offers_status          -- Active offers
idx_energy_offers_source          -- Energy source filter
idx_energy_offers_price           -- Price sorting
idx_energy_offers_marketplace     -- Combined marketplace query
idx_energy_offers_availability    -- Time window queries
idx_energy_offers_amount          -- Amount filtering
```

**Use Cases**:
- Marketplace offer listing
- Energy source filtering
- Price-based sorting
- Availability window queries
- Seller offer management

#### 5. Energy Orders Indexes

```sql
-- Order management
idx_energy_orders_buyer_id        -- Buyer's orders
idx_energy_orders_status          -- Order status queries
idx_energy_orders_buyer_status    -- Buyer + status filter
idx_energy_orders_source          -- Preferred source filter
idx_energy_orders_price           -- Price-based matching
```

**Use Cases**:
- Buyer order history
- Order matching
- Status filtering
- Energy source preferences

#### 6. Energy Transactions Indexes

```sql
-- Transaction queries
idx_energy_transactions_seller       -- Seller transactions
idx_energy_transactions_buyer        -- Buyer transactions
idx_energy_transactions_status       -- Status filtering
idx_energy_transactions_offer        -- Offer-based lookup
idx_energy_transactions_order        -- Order-based lookup
idx_energy_transactions_settled      -- Settlement queries
idx_energy_transactions_blockchain   -- Blockchain lookup
idx_energy_transactions_participants -- Combined participant query
```

**Use Cases**:
- User transaction history (buyer/seller)
- Settlement tracking
- Blockchain verification
- Offer/order transaction lookup

#### 7. ERC Certificates Indexes

```sql
-- Certificate management
idx_erc_certificates_user_id      -- User's certificates
idx_erc_certificates_wallet       -- Wallet lookup
idx_erc_certificates_cert_id      -- Certificate ID lookup
idx_erc_certificates_status       -- Status filtering
idx_erc_certificates_user_status  -- User + status filter
idx_erc_certificates_expiry       -- Expiry date queries
idx_erc_certificates_blockchain   -- Blockchain lookup
```

**Use Cases**:
- User certificate portfolio
- Certificate verification
- Expiry management
- Status filtering

### Applying Indexes

#### Automatic Application

```bash
# Run the index application script
cd api-gateway
./scripts/apply-performance-indexes.sh
```

The script will:
1. Check database connection
2. Show current index statistics
3. Apply all indexes (~50 indexes)
4. Show updated statistics
5. Display index sizes

#### Manual Application

```bash
# Connect to database
psql -h localhost -p 5432 -U gridtokenx -d gridtokenx

# Apply migration
\i migrations/009_add_performance_indexes.sql

# Verify indexes
SELECT * FROM pg_indexes WHERE schemaname = 'public';
```

### Index Maintenance

#### Monitor Index Usage

```sql
-- Find unused indexes (idx_scan = 0)
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
WHERE idx_scan = 0
ORDER BY pg_relation_size(indexrelid) DESC;
```

#### Rebuild Indexes

```sql
-- Rebuild specific index (with minimal locking)
REINDEX INDEX CONCURRENTLY idx_users_wallet_address;

-- Rebuild all indexes on a table
REINDEX TABLE CONCURRENTLY users;
```

#### Analyze Tables

```sql
-- Update statistics after bulk operations
ANALYZE users;
ANALYZE meter_readings;
ANALYZE trading_orders;
```

### Expected Performance Improvements

| Query Type | Before | After | Improvement |
|------------|--------|-------|-------------|
| User login | 50ms | 5ms | 10x faster |
| Reading list | 200ms | 20ms | 10x faster |
| Order matching | 500ms | 50ms | 10x faster |
| Transaction history | 300ms | 30ms | 10x faster |
| Marketplace listing | 400ms | 40ms | 10x faster |

---

## Redis Caching

### Overview

The `CacheService` provides Redis-based caching with automatic TTL management, cache-aside patterns, and comprehensive statistics tracking.

### Cache Service Features

```rust
use api_gateway::services::CacheService;

// Initialize cache service
let cache = CacheService::new("redis://localhost:6379", 300)?;
```

### Cache Key Prefixes

```rust
use api_gateway::services::cache_service::keys;

// Predefined key prefixes
keys::USER_PROFILE      // "user:profile"
keys::USER_WALLET       // "user:wallet"
keys::METER_STATS       // "meter:stats"
keys::ORDER_BOOK        // "order:book"
keys::ACTIVE_OFFERS     // "offers:active"
keys::MARKET_DATA       // "market:data"
keys::HEALTH_CHECK      // "health:check"
keys::RATE_LIMIT        // "rate_limit"
```

### Basic Operations

#### Set and Get

```rust
// Set with default TTL (5 minutes)
cache.set("user:123", &user_data).await?;

// Set with custom TTL
cache.set_with_ttl("session:abc", &session, Duration::from_secs(3600)).await?;

// Get value
let user: Option<User> = cache.get("user:123").await?;
```

#### Cache-Aside Pattern

```rust
// Get from cache or compute if missing
let user_stats = cache.get_or_compute(
    &format!("meter:stats:{}", user_id),
    || async {
        // Compute stats from database
        meter_service.get_user_stats(user_id).await
    }
).await?;

// With custom TTL
let market_data = cache.get_or_compute_with_ttl(
    "market:data:solar",
    Duration::from_secs(60),
    || async {
        trading_service.get_market_data("solar").await
    }
).await?;
```

#### Delete Operations

```rust
// Delete single key
cache.delete("user:123").await?;

// Delete pattern (e.g., all user caches)
let deleted_count = cache.delete_pattern("user:*").await?;
```

#### Counter Operations

```rust
// Increment counter
let count = cache.increment("api:requests:total").await?;

// Increment with TTL (rate limiting)
let count = cache.increment_with_ttl(
    "rate_limit:user:123",
    Duration::from_secs(60)
).await?;
```

### Caching Strategies

#### 1. User Profile Caching

```rust
// Cache user profile for 5 minutes
let user_key = CacheService::build_key(keys::USER_PROFILE, &user_id);
cache.set_with_ttl(&user_key, &user, Duration::from_secs(300)).await?;

// Invalidate on update
cache.delete(&user_key).await?;
```

#### 2. Meter Statistics Caching

```rust
// Cache expensive aggregate queries
let stats_key = CacheService::build_key(keys::METER_STATS, &user_id);
let stats = cache.get_or_compute_with_ttl(
    &stats_key,
    Duration::from_secs(600), // 10 minutes
    || meter_service.calculate_stats(user_id)
).await?;
```

#### 3. Active Offers Caching

```rust
// Cache marketplace listings
let offers_key = CacheService::build_key_parts(&[
    keys::ACTIVE_OFFERS,
    "solar",
    "page1"
]);

cache.set_with_ttl(&offers_key, &offers, Duration::from_secs(60)).await?;

// Invalidate when new offer created
cache.delete_pattern("offers:active:*").await?;
```

#### 4. Order Book Caching

```rust
// Cache order book snapshot
let order_book_key = CacheService::build_key(keys::ORDER_BOOK, "solar");
cache.set_with_ttl(&order_book_key, &order_book, Duration::from_secs(10)).await?;
```

### Cache Invalidation Patterns

#### 1. Time-Based (TTL)

Most common approach - let cache expire naturally:

```rust
// Short TTL for frequently changing data
cache.set_with_ttl(&key, &data, Duration::from_secs(60)).await?;

// Long TTL for stable data
cache.set_with_ttl(&key, &data, Duration::from_secs(3600)).await?;
```

#### 2. Event-Based

Invalidate on specific events:

```rust
// On user update
cache.delete(&format!("user:profile:{}", user_id)).await?;

// On order creation
cache.delete_pattern("order:book:*").await?;
cache.delete_pattern("offers:active:*").await?;
```

#### 3. Write-Through

Update cache immediately on write:

```rust
// Update database
db.update_user(&user).await?;

// Update cache
let user_key = format!("user:profile:{}", user.id);
cache.set(&user_key, &user).await?;
```

### Cache Statistics

```rust
// Get cache statistics
let stats = cache.get_stats().await?;
println!("Cache hits: {}", stats.hits);
println!("Cache misses: {}", stats.misses);
println!("Hit rate: {:.2}%", stats.hit_rate);
```

### Cache TTL Recommendations

| Data Type | TTL | Rationale |
|-----------|-----|-----------|
| User profiles | 5 minutes | Moderate update frequency |
| Meter statistics | 10 minutes | Expensive aggregations |
| Active offers | 1 minute | Frequent changes |
| Order book | 10 seconds | Real-time data |
| Market data | 1 minute | Near real-time |
| Health checks | 30 seconds | Quick refresh |
| Static data | 1 hour | Rarely changes |

---

## Performance Monitoring

### Database Performance

#### Slow Query Identification

```sql
-- Enable pg_stat_statements
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- Find slowest queries
SELECT 
    query,
    calls,
    total_exec_time,
    mean_exec_time,
    max_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 20;
```

#### Table Statistics

```sql
-- Check table scan statistics
SELECT 
    schemaname,
    tablename,
    seq_scan,
    seq_tup_read,
    idx_scan,
    idx_tup_fetch,
    n_tup_ins,
    n_tup_upd,
    n_tup_del
FROM pg_stat_user_tables
ORDER BY seq_scan DESC;
```

#### Index Effectiveness

```sql
-- Check index usage
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch,
    pg_size_pretty(pg_relation_size(indexrelid)) as size
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan DESC;
```

### Cache Performance

```rust
// Monitor cache statistics
let stats = cache.get_stats().await?;

// Log metrics
tracing::info!(
    "Cache stats - Hits: {}, Misses: {}, Hit Rate: {:.2}%",
    stats.hits,
    stats.misses,
    stats.hit_rate
);
```

### Application Metrics

The API Gateway exposes Prometheus metrics at `/metrics`:

```bash
# Check metrics endpoint
curl http://localhost:8080/metrics

# Key metrics to monitor:
# - http_request_duration_seconds (API response times)
# - database_operations_total (DB operation count)
# - cache_operations_total (Cache hit/miss rates)
# - trading_operations_total (Trading activity)
```

---

## Best Practices

### Database Optimization

1. **Use Indexes Wisely**
   - Add indexes for frequently queried columns
   - Avoid over-indexing (slows writes)
   - Use partial indexes for filtered queries
   - Use composite indexes for multi-column queries

2. **Query Optimization**
   - Use EXPLAIN ANALYZE to understand query plans
   - Avoid SELECT * - fetch only needed columns
   - Use pagination for large result sets
   - Batch operations when possible

3. **Connection Pooling**
   - Configure appropriate pool size
   - Monitor pool utilization
   - Set connection timeouts

4. **Regular Maintenance**
   - Run VACUUM ANALYZE weekly
   - Rebuild indexes monthly
   - Update statistics after bulk operations

### Caching Best Practices

1. **Cache What's Expensive**
   - Complex aggregations
   - Frequently accessed data
   - Expensive API calls
   - Database joins

2. **Don't Cache Everything**
   - Real-time data (use short TTL instead)
   - User-specific transient data
   - Data that changes frequently
   - Large objects (consider size limits)

3. **Cache Invalidation**
   - Use TTL for automatic expiration
   - Invalidate on writes
   - Use cache patterns (e.g., "user:*")
   - Monitor stale data issues

4. **Error Handling**
   - Don't fail on cache errors
   - Fall back to database on cache miss
   - Log cache errors for monitoring
   - Use cache gracefully (optional speedup)

### Performance Testing

```bash
# Load testing with Apache Bench
ab -n 1000 -c 10 http://localhost:8080/api/offers

# Stress testing with wrk
wrk -t4 -c100 -d30s http://localhost:8080/api/users

# Monitor during tests
watch -n 1 'curl -s http://localhost:8080/metrics | grep http_request'
```

---

## Troubleshooting

### High Database Load

**Symptoms**:
- Slow query response times
- High CPU usage
- Connection pool exhaustion

**Solutions**:
1. Check for missing indexes
2. Optimize slow queries
3. Increase connection pool size
4. Add caching layer
5. Consider read replicas

### Cache Issues

**Symptoms**:
- Low cache hit rate
- Stale data
- High cache memory usage

**Solutions**:
1. Adjust TTL values
2. Implement proper invalidation
3. Monitor cache size
4. Use cache compression
5. Implement cache warming

### Index Bloat

**Symptoms**:
- Large index sizes
- Slow index scans
- Wasted disk space

**Solutions**:
```sql
-- Rebuild bloated indexes
REINDEX INDEX CONCURRENTLY idx_name;

-- Monitor index bloat
SELECT 
    schemaname,
    tablename,
    indexname,
    pg_size_pretty(pg_relation_size(indexrelid)) as size
FROM pg_stat_user_indexes
ORDER BY pg_relation_size(indexrelid) DESC;
```

---

## Performance Checklist

- [ ] Database indexes applied
- [ ] Query plans optimized (EXPLAIN ANALYZE)
- [ ] Cache service configured
- [ ] Cache keys defined
- [ ] Cache TTLs appropriate
- [ ] Monitoring metrics enabled
- [ ] Slow query logging enabled
- [ ] Connection pool configured
- [ ] Load testing performed
- [ ] Performance benchmarks established

---

## Additional Resources

- [PostgreSQL Performance Tips](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [Redis Best Practices](https://redis.io/topics/optimization)
- [Database Indexing Guide](https://use-the-index-luke.com/)
- [Caching Strategies](https://aws.amazon.com/caching/best-practices/)

---

**Last Updated**: November 13, 2025  
**Maintained By**: GridTokenX Development Team
