# GridTokenX API Load Testing

This directory contains load testing scripts for the GridTokenX API Gateway using [k6](https://k6.io/).

## Prerequisites

Install k6:

```bash
# macOS
brew install k6

# Linux (Debian/Ubuntu)
sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# Docker
docker pull grafana/k6
```

## Test Scripts

| Script | Description | Endpoints Tested |
|--------|-------------|------------------|
| `auth.js` | Authentication load tests | `/api/v1/users`, `/api/v1/auth/token`, `/api/v1/users/me` |
| `trading.js` | Trading API load tests | `/api/v1/trading/book`, `/api/v1/trading/trades`, `/api/v1/trading/orders` |

## Running Tests

### Start the API Gateway

```bash
cd /path/to/gridtokenx-apigateway
cargo run
```

### Run Auth Load Tests

```bash
# Smoke test only (quick validation)
k6 run --tag test_type=smoke tests/load/auth.js

# Full test (smoke + load + stress)
k6 run tests/load/auth.js

# With custom API URL
k6 run -e API_URL=http://localhost:4000 tests/load/auth.js
```

### Run Trading Load Tests

```bash
# With test user credentials
k6 run -e TEST_USERNAME=myuser -e TEST_PASSWORD=mypass tests/load/trading.js

# Default (creates new test users)
k6 run tests/load/trading.js
```

## Test Scenarios

### Auth Tests (`auth.js`)

1. **Smoke Test** (30s) - 1 VU, validates basic functionality
2. **Load Test** (4m) - Ramps up to 10 VUs, sustains, ramps down
3. **Stress Test** (3m) - Pushes to 100 VUs to find breaking point

### Trading Tests (`trading.js`)

1. **Trading Load** (2m) - Constant 10 requests/second
   - Order book reads
   - Trade history reads
   - Random order creation (30% of iterations)

## Thresholds

Tests will fail if:
- 95th percentile response time > 500ms (auth) / 1000ms (trading)
- Error rate > 1% (auth) / 5% (trading)

## Results

Results are saved to `tests/load/results/`:
- `auth_summary.json` - Auth test results
- `trading_summary.json` - Trading test results

## Interpreting Results

| Metric | Good | Acceptable | Poor |
|--------|------|------------|------|
| p95 latency | < 200ms | 200-500ms | > 500ms |
| Error rate | < 0.1% | 0.1-1% | > 1% |
| Requests/sec | > 100 | 50-100 | < 50 |

## Example Output

```
========== Load Test Summary ==========
Total requests: 1,234
Failed requests: 2
Avg response time: 45.23ms
95th percentile: 125.67ms
========================================
```

## Grafana Dashboard (Optional)

For real-time visualization, run k6 with InfluxDB output:

```bash
k6 run --out influxdb=http://localhost:8086/k6 tests/load/auth.js
```

Then import the k6 dashboard in Grafana (ID: 2587).
