#!/bin/bash

# GridTokenX API Gateway - Quick Start and Test Script
# Starts the server and runs integration tests

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

# Check prerequisites
print_header "Checking Prerequisites"

# Check if PostgreSQL is running
if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo -e "${RED}✗ PostgreSQL is not running${NC}"
    echo "Start it with: brew services start postgresql"
    exit 1
fi
echo -e "${GREEN}✓ PostgreSQL is running${NC}"

# Check if Redis is running
if ! redis-cli ping > /dev/null 2>&1; then
    echo -e "${RED}✗ Redis is not running${NC}"
    echo "Start it with: brew services start redis"
    exit 1
fi
echo -e "${GREEN}✓ Redis is running${NC}"

# Check if database exists
if ! psql -h localhost -U gridtokenx_user -d gridtokenx -c "SELECT 1" > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠ Database may not exist or credentials incorrect${NC}"
    echo "Check your DATABASE_URL in .env"
fi

# Check if .env exists
if [ ! -f .env ]; then
    echo -e "${RED}✗ .env file not found${NC}"
    echo "Copy .env.example to .env and configure it"
    exit 1
fi
echo -e "${GREEN}✓ .env file exists${NC}"

# Build the project
print_header "Building Project"
echo "Building in release mode..."
if cargo build --release 2>&1 | grep -i "error"; then
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Build successful${NC}"

# Start the server in background
print_header "Starting Server"
echo "Starting GridTokenX API Gateway..."

# Kill any existing instance
pkill -f "target/release/gridtokenx" || true
sleep 2

# Start server
./target/release/gridtokenx-apigateway > api-gateway.log 2>&1 &
SERVER_PID=$!

echo -e "${YELLOW}Server PID: $SERVER_PID${NC}"
echo "Waiting for server to start..."

# Wait for server to be ready
for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Server is ready${NC}"
        break
    fi
    
    if [ $i -eq 30 ]; then
        echo -e "${RED}✗ Server failed to start${NC}"
        echo "Check logs: tail -f api-gateway.log"
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi
    
    sleep 1
    echo -n "."
done

echo ""

# Run tests
print_header "Running Integration Tests"

# Test 1: Public endpoints
echo -e "\n${BLUE}Test 1: Public Market Endpoints${NC}"
./scripts/test-market-clearing.sh

echo ""
sleep 2

# Test 2: Admin endpoints (if admin user exists)
echo -e "\n${BLUE}Test 2: Admin Endpoints (Optional)${NC}"
echo "Run this manually if you have admin credentials:"
echo "  ./scripts/test-market-clearing-authenticated.sh"

echo ""
sleep 2

# Test 3: Complete flow
echo -e "\n${BLUE}Test 3: Complete Order Flow${NC}"
./scripts/test-complete-flow.sh

# Summary
print_header "Testing Complete"
echo -e "${GREEN}✓ Integration tests completed${NC}"
echo -e "\n${YELLOW}Server is still running (PID: $SERVER_PID)${NC}"
echo "View logs: tail -f api-gateway.log"
echo "Stop server: kill $SERVER_PID"
echo ""
echo "To run manual tests:"
echo "  curl http://localhost:8080/api/market/epoch/status | jq '.'"
echo "  curl http://localhost:8080/api/market/orderbook | jq '.'"
echo ""
