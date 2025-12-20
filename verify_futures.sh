#!/bin/bash

# 1. Get Token
echo "1. Authenticating..."
TOKEN=$(curl -s -X POST http://localhost:4000/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username": "seller1", "password": "Password123!"}' | sed -E 's/.*"access_token":"([^"]+)".*/\1/')

if [ -z "$TOKEN" ] || [ "${TOKEN:0:1}" == "{" ]; then
    echo "❌ Failed to get token"
    exit 1
fi
echo "✅ Token captured"

# 2. Get Products
echo "2. Getting Products..."
PRODUCTS=$(curl -s -X GET http://localhost:4000/api/v1/futures/products \
  -H "Authorization: Bearer $TOKEN")
echo "Response: $PRODUCTS"

PRODUCT_ID=$(echo $PRODUCTS | grep -o '"id":"[^"]*"' | head -n 1 | cut -d'"' -f4)

if [ -z "$PRODUCT_ID" ]; then
    echo "❌ No product found"
    exit 1
fi
echo "✅ Found Product ID: $PRODUCT_ID"

# 3. Create Order
echo "3. Creating Order..."
ORDER_RESP=$(curl -s -X POST http://localhost:4000/api/v1/futures/orders \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"product_id\": \"$PRODUCT_ID\", \"side\": \"long\", \"order_type\": \"market\", \"quantity\": 1.0, \"price\": 50000.0, \"leverage\": 5}")
echo "Response: $ORDER_RESP"

if [[ $ORDER_RESP == *"order_id"* ]]; then
    echo "✅ Order Created"
else
    echo "❌ Order Creation Failed"
    exit 1
fi

# 4. Get Positions
echo "4. Getting Positions..."
POSITIONS=$(curl -s -X GET http://localhost:4000/api/v1/futures/positions \
  -H "Authorization: Bearer $TOKEN")
echo "Response: $POSITIONS"

if [[ $POSITIONS == *"side"* ]]; then
    echo "✅ Verification Complete: Futures Trading is ACTIVE"
else
    echo "⚠️ No positions found (auto-fill might be disabled?)"
fi
