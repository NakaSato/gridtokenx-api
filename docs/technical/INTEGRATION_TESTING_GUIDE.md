# Market Clearing Engine - Integration Testing Guide

**Date**: November 15, 2025  
**Status**: Ready for Testing  
**Prerequisites**: Compilation issues fixed ✅

---

## 🎯 Testing Overview

This guide covers the integration testing strategy for the Market Clearing Engine, including:
- Epoch-based order matching
- Settlement processing
- WebSocket real-time updates
- Performance benchmarks
- Recovery scenarios

## 📋 Prerequisites

### 1. Start Required Services

```bash
# Terminal 1: PostgreSQL Database
docker-compose up postgres

# Terminal 2: Redis Cache
docker-compose up redis

# Terminal 3: API Gateway
cd api-gateway
cargo run --release

# Terminal 4: (Optional) Solana Validator for settlement tests
cd anchor
anchor localnet
```

### 2. Verify Services Are Running

```bash
# Check API Gateway
curl http://localhost:8080/health

# Check PostgreSQL
psql -h localhost -U gridtokenx_user -d gridtokenx -c "SELECT 1;"

# Check Redis
redis-cli ping

# Check Solana (optional)
solana cluster-version
```

### 3. Setup Test Users

```bash
# Option 1: Use pre-created test users (faster)
export BUYER_TOKEN="your-test-buyer-jwt-token"
export SELLER_TOKEN="your-test-seller-jwt-token"
export ADMIN_TOKEN="your-admin-jwt-token"

# Option 2: Create test users on the fly (tests will register automatically)
# No environment variables needed
```

---

## 🧪 Test Suites

### Test Suite 1: Manual API Testing

**Script**: `scripts/test-market-clearing.sh`

**What it tests**:
- ✅ Current epoch retrieval
- ✅ Epoch status checking
- ✅ Market statistics
- ✅ Order book depth
- ✅ Admin epoch management

**Run**:
```bash
# Basic test (no authentication)
./scripts/test-market-clearing.sh

# With admin token for admin endpoints
export ADMIN_TOKEN="your-admin-token"
./scripts/test-market-clearing.sh
```

**Expected Output**:
```
✅ API Gateway is running
✅ Current epoch retrieved
✅ Epoch status retrieved
✅ Market statistics retrieved
✅ Order book retrieved
```

---

### Test Suite 2: Authenticated User Flow

**Script**: `scripts/test-market-clearing-authenticated.sh`

**What it tests**:
- ✅ User registration
- ✅ User authentication
- ✅ Current epoch retrieval (authenticated)
- ✅ Order creation (buy and sell)
- ✅ Market statistics
- ✅ Order book with active orders

**Run**:
```bash
./scripts/test-market-clearing-authenticated.sh
```

**Expected Output**:
```
[1/8] ✅ API Gateway is healthy
[2/8] ✅ Test users registered
[3/8] ✅ Users logged in
[4/8] ✅ Current epoch retrieved
[5/8] ✅ Orders created (buy and sell)
[6/8] ✅ Market statistics retrieved
[7/8] ✅ Order book retrieved
[8/8] ✅ Test completed successfully
```

---

### Test Suite 3: Comprehensive Integration Tests

**File**: `tests/integration/epoch-order-matching.test.ts`

**What it tests**:
- ✅ Basic order matching flow
- ✅ Partial fill scenarios
- ✅ Market orders
- ✅ Epoch transitions
- ✅ Settlement creation
- ✅ Manual epoch clearing
- ✅ WebSocket notifications
- ✅ Order book depth
- ✅ Data consistency

**Run**:
```bash
# Run all integration tests
npm run test:integration

# Run only epoch-order-matching tests
npx vitest run tests/integration/epoch-order-matching.test.ts

# Run with verbose output
npx vitest run tests/integration/epoch-order-matching.test.ts --reporter=verbose

# Run in watch mode for development
npx vitest tests/integration/epoch-order-matching.test.ts
```

**Test Scenarios Covered**:

#### 1. Basic Order Matching
```typescript
// Scenario: Overlapping bid and ask
Seller: 100 kWh @ $0.15/kWh
Buyer:  100 kWh @ $0.20/kWh
Expected: Match at clearing price between $0.15-$0.20
```

#### 2. Partial Fills
```typescript
// Scenario: Large order matched by multiple small orders
Buyer:  300 kWh @ $0.25/kWh
Seller: 100 kWh @ $0.20/kWh (partial match 1)
Seller: 100 kWh @ $0.22/kWh (partial match 2)
Expected: Buyer order partially filled (200/300 kWh)
```

#### 3. Market Orders
```typescript
// Scenario: Market order matches at best available price
Market Buy: 60 kWh (no price limit)
Sell Limit: 50 kWh @ $0.18/kWh (best ask)
Expected: Match 50 kWh at $0.18/kWh
```

#### 4. Non-overlapping Orders
```typescript
// Scenario: Orders don't match due to price gap
Seller: 75 kWh @ $0.30/kWh (ask too high)
Buyer:  75 kWh @ $0.10/kWh (bid too low)
Expected: Both orders remain open
```

---

## 📊 Performance Testing

### Load Test 1: Multiple Concurrent Orders

**Goal**: Test system under load with many simultaneous orders

```bash
# Run performance test
npx vitest run tests/integration/epoch-order-matching.test.ts -t "concurrent"
```

**Metrics to Track**:
- Order placement latency: < 200ms
- Matching engine cycle time: < 1s
- Database query time: < 100ms
- Total throughput: 100+ orders/minute

### Load Test 2: High-Frequency Matching

**Goal**: Stress test matching engine with rapid order flow

**Setup**:
```typescript
// Create 100 orders in quick succession
for (let i = 0; i < 100; i++) {
  await createOrder({
    side: i % 2 === 0 ? 'buy' : 'sell',
    energy_amount: '10',
    price: `${0.15 + (i * 0.001)}`,
  });
}
```

**Expected Results**:
- All orders processed within 5 seconds
- No database deadlocks
- No duplicate matches
- Consistent order book state

---

## 🔄 Epoch Transition Testing

### Test Scenario: Complete Epoch Lifecycle

**Duration**: ~16 minutes (one full epoch + transitions)

**Steps**:
1. **Observe current epoch** (pending or active)
2. **Create orders** in active epoch
3. **Wait for epoch transition** (15 minutes from start)
4. **Verify automatic clearing** happens
5. **Check matches created** in database
6. **Verify settlements** initiated

**Manual Test**:
```bash
# Terminal 1: Watch epoch status
watch -n 10 'curl -s http://localhost:8080/api/market/epoch/current | jq ".status, .time_remaining_seconds"'

# Terminal 2: Monitor matches
watch -n 5 'curl -s -H "Authorization: Bearer $BUYER_TOKEN" \
  http://localhost:8080/api/trading/matches | jq ".matches | length"'

# Terminal 3: Create test orders
./scripts/test-market-clearing-authenticated.sh
```

### Test Scenario: Manual Epoch Clearing

**Goal**: Test admin ability to trigger clearing outside normal schedule

```bash
# Get current epoch ID
EPOCH_ID=$(curl -s http://localhost:8080/api/market/epoch/current | jq -r '.id')

# Trigger manual clearing
curl -X POST -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  http://localhost:8080/api/admin/epochs/$EPOCH_ID/trigger \
  -d '{"reason": "Manual test clearing"}'

# Verify clearing occurred
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  http://localhost:8080/api/admin/epochs/$EPOCH_ID/stats | jq
```

---

## 🔌 WebSocket Testing

### Test Scenario: Real-time Match Notifications

**File**: `tests/integration/epoch-order-matching.test.ts`

**Test**:
```typescript
it('should receive order match notifications via WebSocket', async () => {
  const ws = new WebSocket(`ws://localhost:8080/ws?token=${buyerToken}`);
  
  ws.on('message', (data) => {
    const message = JSON.parse(data.toString());
    if (message.type === 'order_matched') {
      console.log('Match notification received:', message);
      // Verify notification contains match details
      expect(message.data.match_id).toBeDefined();
      expect(message.data.clearing_price).toBeDefined();
    }
  });
  
  // Create matching orders to trigger notification
  await createMatchingOrders();
});
```

**Manual Test**:
```bash
# Use wscat to connect to WebSocket
npm install -g wscat
wscat -c "ws://localhost:8080/ws?token=$BUYER_TOKEN"

# In another terminal, create orders that will match
curl -X POST -H "Authorization: Bearer $SELLER_TOKEN" \
  -H "Content-Type: application/json" \
  http://localhost:8080/api/trading/orders \
  -d '{"energy_amount": "-50", "price_per_kwh": "0.15", "order_type": "Limit"}'
  
# You should see match notification in wscat terminal
```

---

## 🛡️ Recovery & Edge Cases

### Test 1: Server Restart Recovery

**Goal**: Verify system recovers gracefully from unexpected shutdown

**Steps**:
1. Create several orders in current epoch
2. Kill API Gateway process (`Ctrl+C`)
3. Restart API Gateway
4. Verify:
   - Orders still exist
   - Epoch state is correct
   - Matching resumes automatically
   - No duplicate matches created

### Test 2: Database Connection Loss

**Goal**: Test resilience to temporary database outage

**Steps**:
1. Stop PostgreSQL container
2. Attempt to create orders (should fail gracefully)
3. Restart PostgreSQL
4. Verify system reconnects automatically
5. Create orders (should succeed)

### Test 3: Epoch Boundary Edge Cases

**Goal**: Test orders created exactly at epoch transition

**Scenario**:
```typescript
// Create order in last second of epoch
const currentEpoch = await getCurrentEpoch();
const timeRemaining = currentEpoch.time_remaining_seconds;

if (timeRemaining < 5) {
  // Create order
  const order = await createOrder({ ... });
  
  // Verify it's assigned to correct epoch
  expect(order.epoch_id).toBe(currentEpoch.id);
  
  // Wait for transition
  await sleep(timeRemaining * 1000 + 1000);
  
  // Verify order wasn't lost
  const orderStatus = await getOrderStatus(order.id);
  expect(orderStatus).toBeDefined();
}
```

---

## 📈 Success Criteria

### ✅ Core Functionality
- [ ] Orders can be created successfully
- [ ] Orders are assigned to correct epochs
- [ ] Matching occurs automatically every ~1 second
- [ ] Partial fills work correctly
- [ ] Epoch transitions happen on schedule (every 15 minutes)
- [ ] Settlements are created for matches

### ✅ Performance
- [ ] Order placement: < 200ms
- [ ] Order matching: < 1s per cycle
- [ ] API response times: < 100ms (p95)
- [ ] Can handle 100+ orders per epoch
- [ ] Can handle 10+ concurrent users

### ✅ Data Integrity
- [ ] No duplicate matches
- [ ] Order book sums correctly
- [ ] Match prices are within bid-ask spread
- [ ] Settlement amounts match trade amounts
- [ ] Database constraints enforced

### ✅ Reliability
- [ ] System recovers from restarts
- [ ] Handles database reconnection
- [ ] Handles Redis failure gracefully
- [ ] Error messages are clear
- [ ] Logs are comprehensive

### ✅ Real-time Features
- [ ] WebSocket connections stable
- [ ] Match notifications sent promptly
- [ ] Epoch transition events broadcast
- [ ] Order book updates pushed

---

## 🐛 Troubleshooting

### Issue: Orders not matching

**Check**:
```bash
# Verify matching engine is running
curl http://localhost:8080/api/market/stats | jq '.last_match_time'

# Check order book
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/market/depth | jq

# Verify orders exist
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/trading/orders | jq '.orders[] | .id, .status'
```

**Common Causes**:
- Orders in different epochs
- Non-overlapping prices
- Orders already filled
- Matching engine not running

### Issue: Epoch not transitioning

**Check**:
```bash
# Check current epoch
curl http://localhost:8080/api/market/epoch/current | jq '.status, .end_time'

# Check epoch scheduler logs
docker logs p2p-api-gateway | grep -i epoch | tail -20
```

**Common Causes**:
- Epoch scheduler not started
- System time incorrect
- Database connection issues

### Issue: WebSocket disconnecting

**Check**:
```bash
# Check WebSocket endpoint
curl -i -N -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" \
  -H "Sec-WebSocket-Key: test" \
  http://localhost:8080/ws?token=$TOKEN
```

**Common Causes**:
- Invalid JWT token
- Connection timeout (increase client timeout)
- Proxy interfering with WebSocket
- API Gateway restarted

---

## 📊 Test Results Template

```markdown
# Market Clearing Engine - Integration Test Results

**Date**: YYYY-MM-DD
**Tester**: [Name]
**Environment**: Local Development / Staging / Production
**API Gateway Version**: X.Y.Z

## Test Summary

| Test Suite | Tests Run | Passed | Failed | Skipped |
|------------|-----------|--------|--------|---------|
| Manual API Tests | 7 | 7 | 0 | 0 |
| Authenticated Flow | 8 | 8 | 0 | 0 |
| Integration Tests | 25 | 23 | 2 | 0 |
| **Total** | **40** | **38** | **2** | **0** |

## Performance Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Order Placement Latency (p95) | < 200ms | 145ms | ✅ Pass |
| Matching Cycle Time | < 1s | 850ms | ✅ Pass |
| API Response Time (p95) | < 100ms | 78ms | ✅ Pass |
| Orders Per Epoch | 100+ | 127 | ✅ Pass |
| Concurrent Users | 10+ | 15 | ✅ Pass |

## Failed Tests

### Test: WebSocket reconnection after server restart
- **Status**: ❌ Failed
- **Error**: Connection closed without notification
- **Root Cause**: WebSocket cleanup on server shutdown needs improvement
- **Action**: Create issue #XXX

### Test: Settlement creation under high load
- **Status**: ❌ Failed
- **Error**: Settlement service timeout after 5s
- **Root Cause**: Database pool exhausted
- **Action**: Increase connection pool size

## Observations

- Epoch transitions occurred on schedule (15-minute intervals)
- Order matching consistently completed within 1 second
- No data consistency issues observed
- WebSocket performance excellent under normal load
- Database queries well-optimized

## Recommendations

1. Increase database connection pool from 20 to 30 connections
2. Add reconnection logic to WebSocket client
3. Implement circuit breaker for settlement service
4. Add monitoring for epoch transition delays
5. Consider caching frequently accessed epoch data

## Sign-off

- [ ] All critical tests passed
- [ ] Performance meets requirements
- [ ] Known issues documented
- [ ] Ready for next phase: [Performance Testing / Staging / Production]

**Approved by**: _______________  
**Date**: _______________
```

---

## 🚀 Next Steps After Integration Testing

Once integration tests pass:

1. **Performance Testing** (2 days)
   - Load testing with 1000+ orders
   - Stress testing with 100+ concurrent users
   - Latency profiling
   - Database query optimization

2. **Staging Deployment** (1 day)
   - Deploy to staging environment
   - Run full test suite
   - Monitor for 24 hours
   - Document any issues

3. **Production Deployment** (1 day)
   - Blue-green deployment
   - Gradual rollout (10% → 50% → 100%)
   - Real-time monitoring
   - Rollback plan ready

4. **Post-Deployment** (Ongoing)
   - Monitor epoch transitions
   - Track matching performance
   - User feedback collection
   - Bug fixes and optimizations

---

**Document Version**: 1.0  
**Last Updated**: November 15, 2025  
**Maintained By**: GridTokenX Development Team
