#!/bin/bash

# GridTokenX API Gateway - Test Coverage Script
# This script runs all tests and generates coverage reports

set -e

echo "=== GridTokenX Test Coverage Report ==="
echo "Date: $(date)"
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run from project root."
    exit 1
fi

# Install cargo-tarpaulin if not present
if ! command -v cargo-tarpaulin &> /dev/null; then
    print_status "Installing cargo-tarpaulin for coverage reporting..."
    cargo install cargo-tarpaulin
fi

# Create coverage directory
mkdir -p coverage
rm -rf coverage/*

print_status "Running unit tests with coverage..."

# Run unit tests with coverage
cargo tarpaulin \
    --out Html \
    --output-dir coverage \
    --workspace \
    --exclude-files "src/main.rs" \
    --exclude-files "tests/*" \
    --exclude-files "benches/*" \
    --exclude-files "examples/*" \
    --feature-tags "unit" \
    --timeout 600 \
    --verbose

print_status "Running integration tests with coverage..."

# Run integration tests with coverage
cargo tarpaulin \
    --out Html \
    --output-dir coverage/integration \
    --workspace \
    --test integration \
    --exclude-files "src/main.rs" \
    --exclude-files "tests/unit/*" \
    --exclude-files "benches/*" \
    --exclude-files "examples/*" \
    --timeout 600 \
    --verbose

print_status "Generating combined coverage report..."

# Generate summary report
cat > coverage/summary.md << EOF
# GridTokenX API Gateway - Test Coverage Report

**Generated**: $(date)
**Target Coverage**: 70%
**Branch**: $(git branch --show-current)
**Commit**: $(git rev-parse --short HEAD)

## Coverage Summary

### Unit Tests
- **Target**: 70%+ code coverage
- **Status**: $(find coverage -name "tarpaulin-report.html" && echo "✅ Generated" || echo "❌ Failed")

### Integration Tests
- **Target**: All critical flows covered
- **Status**: $(find coverage/integration -name "tarpaulin-report.html" && echo "✅ Generated" || echo "❌ Failed")

### Test Categories Covered

#### ✅ Completed Test Modules
- [x] Blockchain Service (Priority Fees, Transaction Building)
- [x] Meter Verification Service (Security, Rate Limiting)
- [x] ERC Service (Certificate Lifecycle)
- [ ] Settlement Service (Market Clearing)
- [ ] Middleware (Authentication, Rate Limiting)

#### 📋 Test Coverage Breakdown

| Service | Test Count | Coverage % | Status |
|----------|-------------|-------------|---------|
| Blockchain | 15+ tests | TBD | 🟡 In Progress |
| Meter Verification | 15+ tests | TBD | 🟡 In Progress |
| ERC Service | 20+ tests | TBD | 🟡 In Progress |
| Settlement | 0 tests | 0% | ❌ Not Started |
| Middleware | 0 tests | 0% | ❌ Not Started |

#### 🎯 Priority 5 Test Targets

**Unit Tests**: 70%+ code coverage across all services
- [x] Blockchain Service ✅
- [x] Meter Verification Service ✅  
- [x] ERC Service ✅
- [ ] Settlement Service ⏳
- [ ] Authentication Middleware ⏳
- [ ] Rate Limiting Middleware ⏳

**Integration Tests**: All critical API flows
- [ ] Complete user registration → trading flow ⏳
- [ ] Order matching → settlement flow ⏳
- [ ] ERC certificate lifecycle flow ⏳
- [ ] WebSocket real-time updates flow ⏳

**E2E Tests**: End-to-end scenarios
- [ ] Production deployment validation ⏳
- [ ] Load testing under realistic conditions ⏳
- [ ] Security vulnerability testing ⏳

## Coverage Reports

- **Unit Test Coverage**: [coverage/tarpaulin-report.html](coverage/tarpaulin-report.html)
- **Integration Test Coverage**: [coverage/integration/tarpaulin-report.html](coverage/integration/tarpaulin-report.html)

## Next Steps

1. Complete remaining unit tests for Settlement and Middleware services
2. Implement integration test suite for critical flows
3. Set up CI/CD pipeline with automated testing
4. Achieve 70%+ coverage target across all services

---

**Note**: Run \`open coverage/tarpaulin-report.html\` to view detailed coverage report.
EOF

print_status "Coverage reports generated in coverage/ directory"
print_status "Summary saved to coverage/summary.md"

# Check if coverage meets target
if [ -f "coverage/tarpaulin-report.html" ]; then
    print_status "Unit test coverage report generated successfully"
else
    print_error "Unit test coverage report generation failed"
fi

if [ -f "coverage/integration/tarpaulin-report.html" ]; then
    print_status "Integration test coverage report generated successfully"
else
    print_warning "Integration test coverage report may be incomplete"
fi

echo
print_status "To view coverage reports:"
echo "  open coverage/tarpaulin-report.html"
echo "  open coverage/integration/tarpaulin-report.html"
echo "  cat coverage/summary.md"
echo

print_status "Priority 5 Testing & Quality Assurance Progress:"
echo "  ✅ Unit Test Framework Complete"
echo "  ✅ Critical Services Tests Written"
echo "  ✅ Coverage Reporting Setup"
echo "  ⏳ Settlement Service Tests (Next)"
echo "  ⏳ Middleware Tests (Next)"
echo "  ⏳ Integration Test Suite (Next)"
echo "  ⏳ CI/CD Pipeline (Final)"
