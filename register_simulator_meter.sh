#!/bin/bash

# Configuration
SIMULATOR_URL="http://localhost:8000"
API_GATEWAY_URL="http://localhost:8080"
USERNAME="sim_user_$(date +%s)"
EMAIL="${USERNAME}@example.com"
PASSWORD="StrongPassw0rd!2025"

echo "=== Registering Simulator Meter in API Gateway ==="

# 1. Get Meter ID from Simulator
echo "Fetching meters from Simulator..."
METERS_JSON=$(curl -s "${SIMULATOR_URL}/api/meters/")
METER_ID=$(echo $METERS_JSON | grep -o '"meter_id":"[^"]*"' | head -1 | cut -d'"' -f4)

if [ -z "$METER_ID" ]; then
    echo "Error: No meters found in Simulator."
    exit 1
fi

echo "Found Meter ID: $METER_ID"

# 2. Get Public Key for the Meter
echo "Fetching Public Key for Meter $METER_ID..."
STATUS_JSON=$(curl -s "${SIMULATOR_URL}/api/meters/${METER_ID}/status")
PUBLIC_KEY=$(echo $STATUS_JSON | grep -o '"public_key":"[^"]*"' | cut -d'"' -f4)

if [ -z "$PUBLIC_KEY" ]; then
    echo "Error: Could not find public key for meter."
    exit 1
fi

echo "Found Public Key: ${PUBLIC_KEY:0:20}..."

# 3. Register User in API Gateway
echo "Registering User: $USERNAME..."
REGISTER_RES=$(curl -s -X POST "${API_GATEWAY_URL}/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"email\": \"$EMAIL\", \"password\": \"$PASSWORD\", \"first_name\": \"Sim\", \"last_name\": \"User\"}")

# Extract verification token from logs (hacky but works for dev)
echo "Waiting for verification token in logs..."
sleep 2
TOKEN=$(grep "Verification token for $EMAIL" apigateway.log | tail -n 1 | awk '{print $NF}')

if [ -z "$TOKEN" ]; then
    echo "Error: Could not find verification token."
    # Try to proceed anyway if auto-verified (unlikely)
else
    echo "Verifying email with token: $TOKEN"
    curl -s -X POST "${API_GATEWAY_URL}/api/auth/verify-email?token=$TOKEN"
fi

# 4. Login to get Access Token
echo "Logging in..."
LOGIN_RES=$(curl -s -X POST "${API_GATEWAY_URL}/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"password\": \"$PASSWORD\"}")

ACCESS_TOKEN=$(echo $LOGIN_RES | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$ACCESS_TOKEN" ]; then
    echo "Error: Login failed."
    echo "Response: $LOGIN_RES"
    exit 1
fi

echo "Logged in. Token: ${ACCESS_TOKEN:0:10}..."

# 5. Register Meter in API Gateway
echo "Registering Meter $METER_ID..."
REGISTER_METER_RES=$(curl -s -X POST "${API_GATEWAY_URL}/api/user/meters" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"meter_serial\": \"$METER_ID\",
    \"meter_type\": \"Solar_Prosumer\",
    \"installation_date\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
    \"location_coordinates\": \"0.0,0.0\",
    \"public_key\": \"$PUBLIC_KEY\"
  }")

echo "Meter Registration Response: $REGISTER_METER_RES"

if [[ $REGISTER_METER_RES == *"success"* ]] || [[ $REGISTER_METER_RES == *"meter_id"* ]]; then
    echo "=== SUCCESS: Meter Registered! ==="
    echo "Now waiting for Simulator to send readings..."
else
    echo "=== FAILURE: Meter Registration Failed ==="
fi
