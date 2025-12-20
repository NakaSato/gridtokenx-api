#!/bin/bash

# Configuration
API_URL="http://localhost:4000"
AUTH_FILE=".auth_token"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "============================================"
echo "Verifying Analytics API"
echo "============================================"

# Helper function to extract JSON field using python
get_json_field() {
    echo "$1" | python3 -c "import sys, json; print(json.load(sys.stdin)$2)" 2>/dev/null
}

# 1. Authenticate (Reuse token if valid, else login)
# For simplicity, we'll just login as seller1 again to get a fresh token
echo -e "\n1. Authenticating as seller1..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/auth/token" \
  -H "Content-Type: application/json" \
  -d '{"username":"seller1", "password":"Password123!"}')

# Try to get access_token first, then token
TOKEN=$(echo $LOGIN_RESPONSE | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('access_token') or data.get('token'))" 2>/dev/null)

if [ -z "$TOKEN" ] || [ "$TOKEN" == "None" ]; then
    echo -e "${RED}Authentication failed.${NC}"
    echo "Response: $LOGIN_RESPONSE"
    exit 1
fi
echo -e "${GREEN}Authenticated.${NC}"

# 2. Test Market Analytics
echo -e "\n2. Testing GET /api/v1/analytics/market?timeframe=24h..."
MARKET_RESPONSE=$(curl -s -X GET "$API_URL/api/v1/analytics/market?timeframe=24h" \
  -H "Authorization: Bearer $TOKEN")

# Check if response contains "market_overview"
if echo "$MARKET_RESPONSE" | grep -q "market_overview"; then
    echo -e "${GREEN}Market analytics retrieved.${NC}"
    # Print partial output
    echo "$MARKET_RESPONSE" | head -c 200
    echo "..."
else
    echo -e "${RED}Failed to get market analytics.${NC}"
    echo "Response: $MARKET_RESPONSE"
fi

# 3. Test User Analytics
echo -e "\n3. Testing GET /api/v1/analytics/my-stats?timeframe=24h..."
USER_STATS_RESPONSE=$(curl -s -X GET "$API_URL/api/v1/analytics/my-stats?timeframe=24h" \
  -H "Authorization: Bearer $TOKEN")

if echo "$USER_STATS_RESPONSE" | grep -q "username"; then
    echo -e "${GREEN}User stats retrieved.${NC}"
    echo "$USER_STATS_RESPONSE" | head -c 200
    echo "..."
else
    echo -e "${RED}Failed to get user stats.${NC}"
    echo "Response: $USER_STATS_RESPONSE"
fi

echo -e "\n============================================"
echo "Verification Complete"
echo "============================================"
