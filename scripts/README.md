# GridTokenX API Gateway - Test Scripts

## Script Flow:

1. Health Check
2. User Registration (Buyer & Seller)
3. User Authentication
4. Wallet Connection Testing
5. Meter Verification Testing
6. Order Creation
7. Market Data Verification
8. Order Matching Monitoring
9. Settlement Processing Monitoring
10. Token & ERC Testing
11. Blockchain Connectivity Testing
12. Comprehensive Summary


This directory contains sequentially numbered test scripts for comprehensive testing of the GridTokenX API Gateway.

## 📋 Test Sequence

The scripts are numbered to run in a logical order, from basic connectivity tests to comprehensive load testing:

### Phase 1: Foundation Tests (01-03)
- **01-test-blockchain-connection.sh** - Verify Solana blockchain connectivity
- **02-test-complete-flow.sh** - Test complete user registration and order flow
- **03-test-meter-verification-flow.sh** - Test meter verification and ownership

### Phase 2: Core Functionality (04-07)
- **04-test-token-minting-e2e.sh** - End-to-end token minting from meter readings
- **05-test-market-clearing.sh** - Test order matching and market clearing
- **06-test-settlement-flow.sh** - Test settlement and blockchain transfers
- **07-test-erc-lifecycle.sh** - Test ERC certificate issuance and lifecycle

### Phase 3: Quality Assurance (08-10)
- **08-run-integration-tests.sh** - Run comprehensive integration test suite
- **09-test-coverage.sh** - Generate code coverage reports
- **10-load-test-api.sh** - Performance and load testing

## 🚀 Quick Start

### Using Makefile (Recommended)

```bash
# Run individual tests
make test-01-blockchain
make test-02-flow
make test-03-meter

# Run grouped test suites
make test-integration    # Tests 01-03
make test-e2e           # Tests 04-07
make test-all-scripts   # Tests 01-09

# Run full test suite
make test-full          # Unit tests + all scripts

# Quick smoke test
make test-quick         # Tests 01-02

# Priority 5 testing
make test-priority5     # All test scripts
```

### Running Scripts Directly

```bash
# Make scripts executable (if not already)
chmod +x scripts/*.sh

# Run individual scripts
./scripts/01-test-blockchain-connection.sh
./scripts/02-test-complete-flow.sh

# Run all tests in sequence
for script in scripts/[0-9]*.sh; do
    echo "Running $script..."
    bash "$script" || exit 1
done
```

## 🔧 Prerequisites

### Required Services
- PostgreSQL database (port 5432)
- Redis cache (port 6379)
- Solana validator (port 8899) or devnet connection
- API Gateway running (port 8080)

### Required Tools
- `curl` - HTTP client
- `jq` - JSON processor
- `cargo` - Rust build tool (for coverage)
- `wrk` or `ab` - Load testing tools

### Environment Variables
```bash
# Optional: Override default API URL
export API_BASE_URL=http://localhost:8080

# Optional: Database connection
export DATABASE_URL=postgresql://user:pass@localhost:5432/gridtokenx

# Optional: Redis connection
export REDIS_URL=redis://localhost:6379
```

## 📊 Test Coverage

### Unit Tests
```bash
make test-unit
# or
cargo test --lib
```

### Integration Tests
```bash
make test-integration
# Runs: 01-blockchain, 02-flow, 03-meter
```

### E2E Tests
```bash
make test-e2e
# Runs: 04-minting, 05-clearing, 06-settlement, 07-erc
```

### Coverage Report
```bash
make coverage
# or
./scripts/09-test-coverage.sh
```

### Load Testing
```bash
make load-test
# or
./scripts/10-load-test-api.sh
```

## 🎯 Test Scenarios

### 01. Blockchain Connection
- Verifies Solana RPC endpoint connectivity
- Checks authority wallet balance
- Tests basic transaction capabilities

### 02. Complete Flow
- User registration (buyer + seller)
- Email verification
- Authentication (JWT)
- Wallet connection
- Order creation (buy/sell)
- Order book validation
- Market epoch status

### 03. Meter Verification
- Meter registration
- Ownership verification
- Serial number validation
- Meter key authentication
- Reading submission authorization

### 04. Token Minting E2E
- Submit meter reading
- Verify energy calculation
- Mint tokens on Solana
- Update user balance
- Blockchain transaction validation

### 05. Market Clearing
- Create multiple orders
- Trigger epoch transition
- Order matching algorithm
- Price discovery
- Order book updates

### 06. Settlement Flow
- Match buyer and seller orders
- Calculate settlement amounts
- Execute blockchain transfers
- Update balances
- Verify transaction signatures

### 07. ERC Lifecycle
- Issue energy certificates
- Certificate validation
- Transfer certificates
- Retire certificates
- Blockchain state verification

### 08. Integration Tests
- Comprehensive API testing
- Cross-module integration
- Error handling validation
- Edge case testing

### 09. Coverage
- Unit test coverage
- Integration test coverage
- Line coverage metrics
- Branch coverage analysis
- HTML report generation

### 10. Load Testing
- Concurrent user simulation
- Request throughput measurement
- Response time analysis
- Error rate monitoring
- Resource utilization

## 📈 Success Metrics

### Performance Targets
- P95 latency: < 200ms
- Throughput: > 100 req/s
- Error rate: < 1%
- Test coverage: > 70%

### Test Pass Criteria
- All unit tests passing
- All integration tests passing
- No critical errors in logs
- Blockchain transactions confirmed
- Database consistency maintained

## 🐛 Troubleshooting

### Common Issues

**1. "Server not running"**
```bash
# Start the API Gateway
make dev
# or
cargo run
```

**2. "Database connection failed"**
```bash
# Reset database
make db-reset
# or
cargo sqlx database drop -y
cargo sqlx database create
cargo sqlx migrate run
```

**3. "Blockchain connection failed"**
```bash
# Start local Solana validator
solana-test-validator
# or use devnet
export SOLANA_RPC_URL=https://api.devnet.solana.com
```

**4. "Redis connection failed"**
```bash
# Start Redis
redis-server
# or via Docker
docker run -d -p 6379:6379 redis:latest
```

**5. "jq: command not found"**
```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq
```

## 📝 Test Reports

Test results and reports are generated in:
- `target/debug/coverage/` - Coverage HTML reports
- `target/criterion/` - Benchmark reports
- `logs/` - Application logs
- Console output - Immediate test results

## 🔄 Continuous Integration

These scripts are designed to run in CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Run test suite
  run: |
    make test-integration
    make test-e2e
    make coverage
```

```yaml
# Example GitLab CI
test:
  script:
    - make test-full
    - make coverage
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: target/coverage/cobertura.xml
```

## 🆘 Support

For issues or questions:
1. Check application logs: `tail -f logs/*.log`
2. Review test output for specific errors
3. Verify all prerequisites are installed
4. Ensure all services are running: `make status`
5. Check environment configuration: `make env`

## 📚 Additional Resources

- [Implementation Plan](../docs/IMPLEMENTATION_PLAN_NEXT_STEPS.md)
- [API Documentation](http://localhost:8080/swagger-ui/)
- [Database Migrations](../migrations/)
- [Project README](../README.md)
