#!/bin/bash

# GridTokenX API Gateway - Market Clearing Engine Integration Test
# This script tests the Market Clearing Engine endpoints without authentication

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
SLEEP_TIME=2

# Helper function to print section headers
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

# Helper function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ $2${NC}"
    else
        echo -e "${RED}✗ $2${NC}"
    fi
}

# Check if server is running
print_header "1. Health Check"
echo "Testing server availability at $API_BASE_URL..."
if curl -s "$API_BASE_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Server is running${NC}"
else
    echo -e "${RED}✗ Server is not running at $API_BASE_URL${NC}"
    echo "Please start the server first: cargo run"
    exit 1
fi

# Test 1: Get current epoch (public endpoint)
print_header "2. Test Current Epoch (Public)"
echo "GET /api/market/epoch"
RESPONSE=$(curl -s -w "\n%{http_code}" "$API_BASE_URL/api/market/epoch")
HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ]; then
    echo -e "${GREEN}✓ Successfully retrieved current epoch${NC}"
    echo "$BODY" | jq '.'
else
    echo -e "${RED}✗ Failed to retrieve current epoch (HTTP $HTTP_CODE)${NC}"
    echo "$BODY"
fi

sleep $SLEEP_TIME

# Test 2: Get epoch status
print_header "3. Test Epoch Status (Public)"
echo "GET /api/market/epoch/status"
RESPONSE=$(curl -s -w "\n%{http_code}" "$API_BASE_URL/api/market/epoch/status")
HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ]; then
    echo -e "${GREEN}✓ Successfully retrieved epoch status${NC}"
    echo "$BODY" | jq '.'
    
    # Extract epoch information
    EPOCH_NUMBER=$(echo "$BODY" | jq -r '.epoch_number')
    EPOCH_STATUS=$(echo "$BODY" | jq -r '.status')
    
    echo -e "\n${YELLOW}Current Epoch: $EPOCH_NUMBER${NC}"
    echo -e "${YELLOW}Status: $EPOCH_STATUS${NC}"
else
    echo -e "${RED}✗ Failed to retrieve epoch status (HTTP $HTTP_CODE)${NC}"
    echo "$BODY"
fi

sleep $SLEEP_TIME

# Test 3: Get order book
print_header "4. Test Order Book (Public)"
echo "GET /api/market/orderbook"
RESPONSE=$(curl -s -w "\n%{http_code}" "$API_BASE_URL/api/market/orderbook")
HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ]; then
    echo -e "${GREEN}✓ Successfully retrieved order book${NC}"
    echo "$BODY" | jq '.'
    
    # Count orders
    BUY_ORDERS=$(echo "$BODY" | jq '.buy_orders | length')
    SELL_ORDERS=$(echo "$BODY" | jq '.sell_orders | length')
    
    echo -e "\n${YELLOW}Buy Orders: $BUY_ORDERS${NC}"
    echo -e "${YELLOW}Sell Orders: $SELL_ORDERS${NC}"
else
    echo -e "${RED}✗ Failed to retrieve order book (HTTP $HTTP_CODE)${NC}"
    echo "$BODY"
fi

sleep $SLEEP_TIME

# Test 4: Get market statistics
print_header "5. Test Market Statistics (Public)"
echo "GET /api/market/stats"
RESPONSE=$(curl -s -w "\n%{http_code}" "$API_BASE_URL/api/market/stats")
HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ]; then
    echo -e "${GREEN}✓ Successfully retrieved market statistics${NC}"
    echo "$BODY" | jq '.'
else
    echo -e "${RED}✗ Failed to retrieve market statistics (HTTP $HTTP_CODE)${NC}"
    echo "$BODY"
fi

sleep $SLEEP_TIME

# Summary
print_header "Test Summary"
echo -e "${GREEN}✓ Public Market Data Endpoints Tested${NC}"
echo -e "${YELLOW}Note: Admin endpoints require authentication${NC}"
echo -e "${YELLOW}Run test-market-clearing-authenticated.sh for full testing${NC}"

echo -e "\n${BLUE}Testing complete!${NC}\n"
