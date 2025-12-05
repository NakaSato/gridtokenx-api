#!/bin/bash

# Configuration
API_GATEWAY_URL="http://localhost:8080"
SIMULATOR_URL="http://localhost:8000"
USERNAME="sim_user_1764717129"
PASSWORD="StrongPassw0rd!2025"
TOKEN="pQJ6d3pNnc4c9a1kRz1kvQbmfRJ7J2TTXmLDHFbRxLq"
METER_ID="aa9fc746-e14b-4c9d-888b-c948c87f4516"

echo "=== Manual Registration ==="

# 1. Get Public Key
echo "Fetching Public Key..."
STATUS_JSON=$(curl -s "${SIMULATOR_URL}/api/meters/${METER_ID}/status")
PUBLIC_KEY=$(echo $STATUS_JSON | grep -o '"public_key":"[^"]*"' | cut -d'"' -f4)
echo "Public Key: $PUBLIC_KEY"

# 2. Verify Email
echo "Verifying Email..."
curl -s -X GET "${API_GATEWAY_URL}/api/auth/verify-email?token=$TOKEN"
echo ""

# 3. Login
echo "Logging in..."
LOGIN_RES=$(curl -s -X POST "${API_GATEWAY_URL}/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"password\": \"$PASSWORD\"}")

ACCESS_TOKEN=$(echo $LOGIN_RES | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
echo "Access Token: ${ACCESS_TOKEN:0:10}..."

# 4. Register Meter
echo "Registering Meter..."
curl -v -X POST "${API_GATEWAY_URL}/api/user/meters" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"meter_serial\": \"$METER_ID\",
    \"meter_type\": \"Solar_Prosumer\",
    \"installation_date\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
    \"location_coordinates\": \"0.0,0.0\",
    \"meter_public_key\": \"$PUBLIC_KEY\"
  }"
