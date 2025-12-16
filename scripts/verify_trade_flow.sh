#!/bin/bash
set -e

API_URL="http://localhost:4000/api/v1"

# Utility function to extract token
get_token() {
    echo $1 | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4
}

# Utility to extract wallet address
get_wallet() {
    echo $1 | grep -o '"wallet_address":"[^"]*"' | cut -d'"' -f4
}

# 1. Register Seller
echo "----------------------------------------------------------------"
echo "Registering Seller..."
SELLER_ID="seller_$(date +%s)"
SELLER_EMAIL="${SELLER_ID}@test.com"
SELLER_PASS="password123"

SELLER_RESP=$(curl -s -X POST $API_URL/users \
    -H "Content-Type: application/json" \
    -d "{\"username\": \"$SELLER_ID\", \"email\": \"$SELLER_EMAIL\", \"password\": \"$SELLER_PASS\", \"first_name\": \"Seller\", \"last_name\": \"Test\"}")

# Get Token
if echo "$SELLER_RESP" | grep -q "access_token"; then
    SELLER_TOKEN=$(get_token "$SELLER_RESP")
else
    echo "Logging in Seller..."
    LOGIN_RESP=$(curl -s -X POST $API_URL/auth/login \
        -H "Content-Type: application/json" \
        -d "{\"username\": \"$SELLER_ID\", \"password\": \"$SELLER_PASS\"}")
    SELLER_TOKEN=$(get_token "$LOGIN_RESP")
fi
echo "Seller Token: ${SELLER_TOKEN:0:10}..."
sleep 5

# Get Wallet and Fund
SELLER_PROFILE=$(curl -s -X GET $API_URL/users/me \
    -H "Authorization: Bearer $SELLER_TOKEN")
echo "DEBUG Profile Response: $SELLER_PROFILE"
SELLER_WALLET=$(get_wallet "$SELLER_PROFILE")
echo "Seller Wallet: $SELLER_WALLET"

if [ -n "$SELLER_WALLET" ]; then
    echo "Funding Seller Wallet via Faucet (SOL first)..."
    curl -s -X POST $API_URL/dev/faucet \
         -H "Content-Type: application/json" \
         -d '{
               "wallet_address": "'"$SELLER_WALLET"'",
               "amount_sol": 100.0,
               "mint_tokens_kwh": 0.0
             }'
    echo ""
    
    echo "Waiting for SOL confirmation..."
    sleep 3
    
    echo "Minting tokens for Seller..."
    curl -X POST $API_URL/dev/faucet \
         -H "Content-Type: application/json" \
         -d '{
               "wallet_address": "'"$SELLER_WALLET"'",
               "amount_sol": 0.0,
               "mint_tokens_kwh": 1000.0
             }'
    echo ""
else
    echo "Error: Seller wallet not found!"
    exit 1
fi

sleep 2

# 2. Register Buyer
echo "----------------------------------------------------------------"
echo "Registering Buyer..."
BUYER_ID="buyer_$(date +%s)"
BUYER_EMAIL="${BUYER_ID}@test.com"
BUYER_PASS="password123"

BUYER_RESP=$(curl -s -X POST $API_URL/users \
    -H "Content-Type: application/json" \
    -d "{\"username\": \"$BUYER_ID\", \"email\": \"$BUYER_EMAIL\", \"password\": \"$BUYER_PASS\", \"first_name\": \"Buyer\", \"last_name\": \"Test\"}")

if echo "$BUYER_RESP" | grep -q "access_token"; then
    BUYER_TOKEN=$(get_token "$BUYER_RESP")
else
    echo "Logging in Buyer..."
    LOGIN_RESP=$(curl -s -X POST $API_URL/auth/login \
        -H "Content-Type: application/json" \
        -d "{\"username\": \"$BUYER_ID\", \"password\": \"$BUYER_PASS\"}")
    BUYER_TOKEN=$(get_token "$LOGIN_RESP")
fi
echo "Buyer Token: ${BUYER_TOKEN:0:10}..."

BUYER_PROFILE=$(curl -s -X GET $API_URL/users/me \
    -H "Authorization: Bearer $BUYER_TOKEN")
BUYER_WALLET=$(get_wallet "$BUYER_PROFILE")
echo "Buyer Wallet: $BUYER_WALLET"

if [ -n "$BUYER_WALLET" ]; then
    echo "Funding Buyer Wallet via Faucet..."
    curl -X POST $API_URL/dev/faucet \
         -H "Content-Type: application/json" \
         -d '{
               "wallet_address": "'"$BUYER_WALLET"'",
               "amount_sol": 100.0,
               "mint_tokens_kwh": 0.0
             }'
    echo ""
else
    echo "Error: Buyer wallet not found!"
    exit 1
fi

sleep 2

# 3. Create Sell Order
echo "----------------------------------------------------------------"
echo "Creating Sell Order (50 kWh @ $0.15)..."
SELL_RESP=$(curl -s -X POST $API_URL/trading/orders \
    -H "Authorization: Bearer $SELLER_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"side": "Sell", "energy_amount": "50.0", "price_per_kwh": "0.15", "order_type": "Limit"}')
echo "Sell Order Response: $SELL_RESP"

sleep 1

# 4. Create Buy Order
echo "----------------------------------------------------------------"
echo "Creating Buy Order (50 kWh @ $0.15)..."
BUY_RESP=$(curl -s -X POST $API_URL/trading/orders \
    -H "Authorization: Bearer $BUYER_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"side": "Buy", "energy_amount": "50.0", "price_per_kwh": "0.15", "order_type": "Limit"}')
echo "Buy Order Response: $BUY_RESP"

sleep 1

# 5. Trigger Matching
echo "----------------------------------------------------------------"
echo "Triggering Matching Engine..."
MATCH_RESP=$(curl -s -X POST $API_URL/trading/admin/match-orders \
    -H "X-API-Key: engineering-department-api-key-2025" \
    -H "Content-Type: application/json")
echo "Match Response: $MATCH_RESP"

# 6. Check Order Book
echo "----------------------------------------------------------------"
echo "Checking User Orders..."
SELLER_ORDERS=$(curl -s -X GET $API_URL/trading/orders \
    -H "Authorization: Bearer $SELLER_TOKEN")
echo "Seller Orders: $SELLER_ORDERS"

BUYER_ORDERS=$(curl -s -X GET $API_URL/trading/orders \
    -H "Authorization: Bearer $BUYER_TOKEN")
echo "Buyer Orders: $BUYER_ORDERS"

echo "----------------------------------------------------------------"
echo "Verification Flow Complete!"
