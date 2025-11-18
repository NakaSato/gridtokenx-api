# GridTokenX API Gateway - Quick Start Guide

**Date**: November 18, 2025  
**Purpose**: Get the system running and test the Market Clearing Engine  

---

## Step 1: Start Database Services (2 minutes)

```bash
# Start PostgreSQL and Redis using Docker
docker-compose -f docker-compose.dev.yml up -d

# Wait for services to be ready
sleep 10

# Verify services are running
docker ps | grep gridtokenx
```

**Expected Output:**
```
gridtokenx-postgres  Up 10 seconds (healthy)
gridtokenx-redis     Up 10 seconds (healthy)
```

---

## Step 2: Run Database Migrations (1 minute)

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run

# Expected: Applied 10 migrations
```

---

## Step 3: Build the Project (3-5 minutes)

```bash
# Build in release mode for better performance
cargo build --release

# Or for development (faster build, slower runtime)
cargo build
```

---

## Step 4: Start the API Gateway (1 minute)

```bash
# Option A: Run in development mode
cargo run

# Option B: Run release binary
./target/release/gridtokenx-apigateway

# Option C: Run in background
./target/release/gridtokenx-apigateway > api-gateway.log 2>&1 &
```

**Wait for startup message:**
```
🚀 Server running on http://0.0.0.0:8080
✅ Epoch Scheduler started
✅ Market Clearing Engine started
```

---

## Step 5: Run Integration Tests (5-10 minutes)

### Quick Health Check
```bash
# Test server is responding
curl http://localhost:8080/health | jq '.'
```

### Run Automated Test Suite
```bash
# Run all integration tests
./scripts/run-integration-tests.sh
```

**Or run tests individually:**

```bash
# Test 1: Public endpoints (no auth)
./scripts/test-market-clearing.sh

# Test 2: Admin endpoints (requires admin user)
./scripts/test-market-clearing-authenticated.sh

# Test 3: Complete order flow
./scripts/test-complete-flow.sh
```

---

## Step 6: Create Admin User (First Time Only)

```bash
# Register admin user via API
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@gridtokenx.com",
    "password": "Admin123!@#",
    "full_name": "System Admin",
    "role": "admin",
    "solana_wallet_address": "Admin123Wallet456"
  }'
```

---

## Monitoring & Debugging

### Check Logs
```bash
# Follow server logs
tail -f api-gateway.log

# Search for errors
grep -i error api-gateway.log

# Check epoch transitions
grep -i "epoch" api-gateway.log | tail -20
```

### Monitor Epochs
```bash
# Watch epoch status (updates every 5 seconds)
watch -n 5 'curl -s http://localhost:8080/api/market/epoch/status | jq "."'
```

### Check Order Book
```bash
# View current order book
curl http://localhost:8080/api/market/orderbook | jq '.'
```

### Database Queries
```bash
# Connect to database
docker exec -it gridtokenx-postgres psql -U gridtokenx_user -d gridtokenx

# Check recent epochs
SELECT epoch_number, status, start_time, end_time 
FROM market_epochs 
ORDER BY created_at DESC 
LIMIT 5;

# Check active orders
SELECT id, order_type, energy_amount, price_per_kwh, status 
FROM trading_orders 
WHERE status = 'open' 
LIMIT 10;

# Exit psql
\q
```

---

## Troubleshooting

### Issue: "Database connection failed"
```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# Check logs
docker logs gridtokenx-postgres

# Restart if needed
docker-compose -f docker-compose.dev.yml restart postgres
```

### Issue: "Redis connection failed"
```bash
# Check if Redis is running
docker ps | grep redis

# Test connection
docker exec -it gridtokenx-redis redis-cli ping

# Restart if needed
docker-compose -f docker-compose.dev.yml restart redis
```

### Issue: "Compilation errors"
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Issue: "Port 8080 already in use"
```bash
# Find process using port 8080
lsof -i :8080

# Kill the process
kill -9 <PID>
```

---

## Stopping Services

```bash
# Stop the API Gateway
# If running in foreground: Ctrl+C
# If running in background: kill <PID>

# Stop Docker services
docker-compose -f docker-compose.dev.yml down

# Stop and remove volumes (clean slate)
docker-compose -f docker-compose.dev.yml down -v
```

---

## Next Steps After Testing

1. **If tests pass**: Proceed to Performance Testing
   - Load testing with 1000+ orders
   - Concurrent user simulation
   - Benchmark results documentation

2. **If tests fail**: Debug and fix
   - Check logs for specific errors
   - Verify database schema matches code
   - Check environment configuration
   - Review integration testing guide

3. **Frontend Development**: Begin React dashboard
   - Setup Vite + React + TypeScript
   - Implement Solana wallet integration
   - Build trading interface
   - Connect WebSocket for real-time updates

---

## Quick Reference Commands

```bash
# Start everything
docker-compose -f docker-compose.dev.yml up -d
cargo run

# Run tests
./scripts/run-integration-tests.sh

# Check status
curl http://localhost:8080/health | jq '.'
curl http://localhost:8080/api/market/epoch/status | jq '.'

# View logs
tail -f api-gateway.log

# Stop everything
docker-compose -f docker-compose.dev.yml down
```

---

## Success Criteria

- [ ] Docker services running (PostgreSQL + Redis)
- [ ] Database migrations applied (10 migrations)
- [ ] Server starts without errors
- [ ] Health check returns 200 OK
- [ ] Public endpoints accessible
- [ ] Admin authentication works
- [ ] Orders can be created
- [ ] Epochs transition automatically
- [ ] Order matching occurs
- [ ] WebSocket updates broadcast

---

**Estimated Time**: 15-20 minutes for complete setup and initial testing

**Support**: See `docs/technical/INTEGRATION_TESTING_GUIDE.md` for detailed procedures
