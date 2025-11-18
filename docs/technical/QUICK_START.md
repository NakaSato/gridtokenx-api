# Market Clearing Engine - Quick Start Guide

Get the Market Clearing Engine up and running in minutes.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Running the Service](#running-the-service)
5. [Verification](#verification)
6. [Common Issues](#common-issues)

---

## Prerequisites

### Required Software

- **Rust** 1.70 or higher ([Install Rust](https://rustup.rs/))
- **PostgreSQL** 14 or higher ([Install PostgreSQL](https://www.postgresql.org/download/))
- **Redis** 7.0 or higher ([Install Redis](https://redis.io/download))
- **Solana CLI** tools ([Install Solana](https://docs.solana.com/cli/install-solana-cli-tools))
- **Git** ([Install Git](https://git-scm.com/downloads))

### Optional

- **Docker** and **Docker Compose** ([Install Docker](https://docs.docker.com/get-docker/))

### System Requirements

- **CPU:** 2+ cores recommended
- **RAM:** 4GB minimum, 8GB recommended
- **Disk:** 10GB free space
- **OS:** Linux, macOS, or Windows (WSL2)

---

## Installation

### Option 1: Manual Installation

#### 1. Clone the Repository

```bash
git clone https://github.com/NakaSato/gridtokenx-platform.git
cd gridtokenx-platform
```

#### 2. Install Rust Dependencies

```bash
cd api-gateway
cargo build --release
```

This will download and compile all dependencies. First build takes 5-10 minutes.

#### 3. Setup PostgreSQL

```bash
# Create database
createdb gridtokenx

# Or using psql
psql -U postgres
CREATE DATABASE gridtokenx;
\q
```

#### 4. Run Database Migrations

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run
```

#### 5. Setup Redis

```bash
# Start Redis server
redis-server

# Verify it's running
redis-cli ping
# Expected: PONG
```

#### 6. Setup Solana

```bash
# Configure for devnet
solana config set --url devnet

# Create keypair (or use existing)
solana-keygen new -o ~/.config/solana/id.json

# Fund the account (devnet only)
solana airdrop 2

# Verify balance
solana balance
```

---

### Option 2: Docker Installation

#### 1. Clone the Repository

```bash
git clone https://github.com/NakaSato/gridtokenx-platform.git
cd gridtokenx-platform
```

#### 2. Start All Services

```bash
docker-compose up -d
```

This starts:
- API Gateway (port 8080)
- PostgreSQL (port 5432)
- Redis (port 6379)
- Solana Validator (local testnet)

#### 3. Verify Services

```bash
docker-compose ps
```

All services should show status "Up".

#### 4. Run Migrations

```bash
docker-compose exec api-gateway sqlx migrate run
```

---

## Configuration

### Environment Variables

Create `.env` file in `api-gateway/`:

```bash
# Copy example configuration
cp .env.example .env

# Edit with your settings
nano .env
```

**Minimal Configuration:**

```bash
# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/gridtokenx

# Redis
REDIS_URL=redis://localhost:6379

# Blockchain
SOLANA_RPC_URL=https://api.devnet.solana.com
BLOCKCHAIN_KEYPAIR_PATH=/home/user/.config/solana/id.json

# Server
PORT=8080
RUST_LOG=info
```

**Production Configuration:**

```bash
# Database (use connection pooling)
DATABASE_URL=postgresql://user:password@db.example.com:5432/gridtokenx?sslmode=require
DATABASE_MAX_CONNECTIONS=20

# Redis (use cluster)
REDIS_URL=redis://redis.example.com:6379
REDIS_PASSWORD=your-redis-password

# Blockchain (mainnet)
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
BLOCKCHAIN_KEYPAIR_PATH=/secure/path/to/keypair.json
SOLANA_COMMITMENT=finalized

# Market Clearing
MATCHING_INTERVAL_MS=1000
ORDER_EXPIRATION_HOURS=24

# Settlement
SETTLEMENT_FEE_RATE=0.001
SETTLEMENT_RETRY_ATTEMPTS=3
SETTLEMENT_RETRY_DELAY_SECS=5
SETTLEMENT_CONFIRMATION_TIMEOUT_SECS=30

# Server
PORT=8080
RUST_LOG=info,api_gateway=debug
WORKERS=4

# Security
JWT_SECRET=your-secure-secret-key-here
CORS_ALLOWED_ORIGINS=https://app.gridtokenx.com,https://admin.gridtokenx.com

# Rate Limiting
RATE_LIMIT_PER_MINUTE=100
```

### Configuration File

Alternatively, use `config.toml`:

```toml
[server]
port = 8080
workers = 4

[database]
url = "postgresql://localhost/gridtokenx"
max_connections = 20

[redis]
url = "redis://localhost:6379"

[blockchain]
rpc_url = "https://api.devnet.solana.com"
keypair_path = "/path/to/keypair.json"

[market]
matching_interval_ms = 1000
order_expiration_hours = 24

[settlement]
fee_rate = 0.001
retry_attempts = 3
retry_delay_secs = 5
```

---

## Running the Service

### Development Mode

```bash
cd api-gateway

# Run with hot reloading (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Or run normally
cargo run
```

**Expected Output:**

```
2025-11-14T10:30:00.123Z  INFO api_gateway: Starting GridTokenX API Gateway
2025-11-14T10:30:00.234Z  INFO api_gateway: Connected to PostgreSQL
2025-11-14T10:30:00.345Z  INFO api_gateway: Connected to Redis
2025-11-14T10:30:00.456Z  INFO api_gateway: Blockchain service initialized
2025-11-14T10:30:00.567Z  INFO api_gateway::services::market_clearing: Market clearing engine started
2025-11-14T10:30:00.678Z  INFO api_gateway::services::market_clearing: Matching loop running (interval: 1000ms)
2025-11-14T10:30:00.789Z  INFO api_gateway: Server listening on 0.0.0.0:8080
```

### Production Mode

```bash
# Build optimized binary
cargo build --release

# Run the binary
./target/release/api-gateway

# Or with systemd
sudo systemctl start gridtokenx-api
```

### Docker Mode

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f api-gateway

# Stop services
docker-compose down
```

### Background Process

```bash
# Using nohup
nohup ./target/release/api-gateway > api.log 2>&1 &

# Using screen
screen -S gridtokenx
./target/release/api-gateway
# Press Ctrl+A, then D to detach

# Using systemd (see systemd section below)
```

---

## Verification

### 1. Check Service Health

```bash
curl http://localhost:8080/api/health
```

**Expected Response:**

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2025-11-14T10:30:00Z",
  "services": {
    "database": "connected",
    "redis": "connected",
    "blockchain": "connected"
  }
}
```

### 2. Check Order Book

```bash
# You'll need an auth token first
# For testing, create a user and login

curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "TestPassword123!",
    "name": "Test User"
  }'

# Login to get token
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "TestPassword123!"
  }'

# Use the token
export TOKEN="your-jwt-token-here"

# Check order book depth
curl http://localhost:8080/api/market/depth \
  -H "Authorization: Bearer $TOKEN"
```

**Expected Response:**

```json
{
  "buy_depth": [],
  "sell_depth": [],
  "timestamp": "2025-11-14T10:30:00Z"
}
```

### 3. Create Test Order

```bash
curl -X POST http://localhost:8080/api/trading/orders \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "order_type": "Limit",
    "side": "Buy",
    "energy_amount": 100.0,
    "price": 0.15
  }'
```

**Expected Response:**

```json
{
  "order_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "Pending",
  "created_at": "2025-11-14T10:30:00Z",
  "expires_at": "2025-11-15T10:30:00Z"
}
```

### 4. Verify Logs

```bash
# Check matching engine is running
grep "Matching cycle" logs/api.log

# Should see entries like:
# 2025-11-14T10:30:01Z INFO Matching cycle completed: 0 trades
```

### 5. Check Redis

```bash
redis-cli

# Check order book keys
KEYS order_book:*

# Should see:
# 1) "order_book:snapshot"
# 2) "order_book:buy_orders"
# 3) "order_book:sell_orders"
```

### 6. Check Database

```bash
psql gridtokenx

# Check tables
\dt

# Should see:
# orders, trades, settlements, users, etc.

# Check recent orders
SELECT id, side, energy_amount, price, status FROM orders ORDER BY created_at DESC LIMIT 5;
```

---

## Common Issues

### Issue: Database Connection Failed

**Error:** `error connecting to database: connection refused`

**Solution:**

```bash
# Check PostgreSQL is running
pg_isready

# If not running, start it
# macOS with Homebrew:
brew services start postgresql

# Linux with systemd:
sudo systemctl start postgresql

# Verify DATABASE_URL is correct in .env
echo $DATABASE_URL
```

---

### Issue: Redis Connection Failed

**Error:** `error connecting to Redis: Connection refused`

**Solution:**

```bash
# Check Redis is running
redis-cli ping

# If not running, start it
# macOS with Homebrew:
brew services start redis

# Linux with systemd:
sudo systemctl start redis

# Or manually:
redis-server
```

---

### Issue: Blockchain Connection Failed

**Error:** `Failed to initialize blockchain service`

**Solution:**

```bash
# Check Solana RPC endpoint
curl https://api.devnet.solana.com -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1, "method":"getHealth"}'

# Expected: {"jsonrpc":"2.0","result":"ok","id":1}

# Check keypair file exists and is valid
ls -l ~/.config/solana/id.json
solana-keygen verify <pubkey> ~/.config/solana/id.json

# Verify BLOCKCHAIN_KEYPAIR_PATH in .env points to correct file
```

---

### Issue: Port Already in Use

**Error:** `Address already in use (os error 48)`

**Solution:**

```bash
# Find process using port 8080
lsof -i :8080

# Kill the process
kill <PID>

# Or use a different port in .env
PORT=8081
```

---

### Issue: Migrations Failed

**Error:** `error running migrations`

**Solution:**

```bash
# Check migration files exist
ls migrations/

# Reset database (WARNING: deletes all data)
sqlx database reset

# Or manually run migrations
sqlx migrate run --database-url $DATABASE_URL

# If sqlx-cli not installed:
cargo install sqlx-cli --no-default-features --features postgres
```

---

### Issue: Rust Compilation Errors

**Error:** Various compilation errors

**Solution:**

```bash
# Update Rust to latest stable
rustup update stable

# Clean build cache
cargo clean

# Rebuild
cargo build --release

# Check Rust version
rustc --version
# Should be 1.70 or higher
```

---

## Next Steps

Now that the service is running:

1. **Read the full documentation:** [`MARKET_CLEARING_ENGINE.md`](./MARKET_CLEARING_ENGINE.md)
2. **Explore the API:** [`API_REFERENCE.md`](./API_REFERENCE.md)
3. **Run tests:** `cargo test --lib`
4. **Setup monitoring:** See [Operational Runbook](./MARKET_CLEARING_ENGINE.md#operational-runbook)
5. **Deploy to production:** See [Deployment Guide](./DEPLOYMENT_GUIDE.md)

---

## Getting Help

- **Documentation:** `/docs/technical/`
- **GitHub Issues:** https://github.com/NakaSato/gridtokenx-platform/issues
- **Team Contact:** dev@gridtokenx.com

---

**Last Updated:** November 14, 2025  
**Version:** 1.0.0
