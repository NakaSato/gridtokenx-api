#!/bin/bash

# Configuration
API_URL="http://localhost:8080"

# Register a new user to ensure valid credentials and fresh state
TIMESTAMP=$(date +%s)
USERNAME="updater_$TIMESTAMP"
EMAIL="updater_$TIMESTAMP@example.com"
PASSWORD="password123"

echo "1. Registering new user: $USERNAME..."
REGISTER_RES=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"email\": \"$EMAIL\", \"password\": \"$PASSWORD\", \"role\": \"prosumer\", \"first_name\": \"Update\", \"last_name\": \"Test\"}")

# Extract Token if returned, or Login
echo "2. Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"password\": \"$PASSWORD\"}")

TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "Login failed. Response: $LOGIN_RESPONSE"
    exit 1
fi
echo "Token obtained."

# 3. Create an Order
echo "3. Creating Sell Order..."
CREATE_RES=$(curl -s -X POST "$API_URL/api/trading/orders" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"order_type": "sell", "energy_amount": 10.0, "price_per_kwh": 0.5}')

ORDER_ID=$(echo $CREATE_RES | grep -o '"id":"[^"]*' | cut -d'"' -f4)
echo "Created Order ID: $ORDER_ID"

if [ -z "$ORDER_ID" ]; then
    echo "Order creation failed: $CREATE_RES"
    exit 1
fi

# 4. Update the Order
echo "4. Updating Order (Amount: 15.0, Price: 0.6)..."
UPDATE_RES=$(curl -s -X PUT "$API_URL/api/trading/orders/$ORDER_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"energy_amount": 15.0, "price_per_kwh": 0.6}')

echo "Update Response: $UPDATE_RES"

# 5. Verify Update
echo "5. Verifying Update via GET..."
GET_RES=$(curl -s -X GET "$API_URL/api/trading/orders" \
  -H "Authorization: Bearer $TOKEN")

# Simple check via grep
echo $GET_RES | grep "15" && echo "✅ Energy amount updated to 15" || echo "❌ Energy amount check failed"
echo $GET_RES | grep "0.6" && echo "✅ Price updated to 0.6" || echo "❌ Price check failed"
