#!/bin/bash

# Configuration
API_URL="http://localhost:8080"
# Using the corporate user (PEA/MEA) token we used in previous steps (assuming we can get it or login again)
# Or use the peer user. 
# Let's login as peer user to get a fresh token.

# Register a new user to ensure valid credentials
TIMESTAMP=$(date +%s)
USERNAME="verify_user_$TIMESTAMP"
EMAIL="verify_$TIMESTAMP@example.com"
PASSWORD="password123"

echo "Registering new user: $USERNAME..."
curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"email\": \"$EMAIL\", \"password\": \"$PASSWORD\", \"role\": \"prosumer\", \"first_name\": \"Test\", \"last_name\": \"User\"}"

echo "Logging in as $USERNAME..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"password\": \"$PASSWORD\"}")

TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "Login failed"
    echo "Response: $LOGIN_RESPONSE"
    exit 1
fi

echo "Login successful. Token: $TOKEN"

echo "----------------------------------------"
echo "1. Listing Active Orders (GET /api/trading/orders?status=active)"
curl -s -X GET "$API_URL/api/trading/orders?status=active" \
  -H "Authorization: Bearer $TOKEN" | json_pp

echo "----------------------------------------"
echo "2. Listing Order History (GET /api/trading/orders)"
curl -s -X GET "$API_URL/api/trading/orders" \
  -H "Authorization: Bearer $TOKEN" | json_pp

echo "----------------------------------------"
echo "3. Testing Cancel Order (DELETE /api/trading/orders/ID - EXPECTING 404/405)"
# Arbitrary ID just to check route existence
curl -s -X DELETE "$API_URL/api/trading/orders/0fb1c192-552a-49ff-a84e-3db31ea40440" \
  -H "Authorization: Bearer $TOKEN" -v
