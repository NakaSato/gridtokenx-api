#!/bin/bash

# Base URL
API_URL="http://localhost:8080"
SIM_URL="http://localhost:8000"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "Starting Verification Flow..."

# Generate unique user
TIMESTAMP=$(date +%s)
USERNAME="alice_sig_$TIMESTAMP"
EMAIL="alice_sig_$TIMESTAMP@example.com"

# 1. Register User
echo "1. Registering User ($USERNAME)..."
REGISTER_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "'"$USERNAME"'",
    "password": "SecurePass123!",
    "email": "'"$EMAIL"'",
    "first_name": "Alice",
    "last_name": "Test"
  }')
echo "Registration Response: $REGISTER_RESPONSE"

# 1.5 Verify Email
echo "1.5 Verifying Email..."
sleep 2 # Wait for log to be written
VERIFICATION_TOKEN=$(grep "Verification token generated for $EMAIL" apigateway.log | tail -n 1 | awk -F': ' '{print $NF}')
echo "Found Verification Token: $VERIFICATION_TOKEN"

if [ -z "$VERIFICATION_TOKEN" ]; then
  echo -e "${RED}Failed to find verification token in logs.${NC}"
  exit 1
fi

VERIFY_RES=$(curl -s -X GET "$API_URL/api/auth/verify-email?token=$VERIFICATION_TOKEN")
echo "Verification Response: $VERIFY_RES"

# 2. Login to get token
echo "2. Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "'"$USERNAME"'",
    "password": "SecurePass123!"
  }')
TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
  echo -e "${RED}Failed to login. Response: $LOGIN_RESPONSE${NC}"
  exit 1
fi
echo -e "${GREEN}Logged in. Token: ${TOKEN:0:10}...${NC}"

# 3. Add Meter to Simulator and Get Public Key
echo "3. Adding Meter to Simulator..."
ADD_METER_RES=$(curl -s -X POST "$SIM_URL/api/meters/add" \
  -H "Content-Type: application/json" \
  -d '{
    "meter_type": "Solar_Prosumer",
    "location": "Test Location",
    "solar_capacity": 5.0,
    "battery_capacity": 10.0,
    "trading_preference": "Moderate"
  }')

METER_ID=$(echo $ADD_METER_RES | python3 -c "import sys, json; print(json.load(sys.stdin)['meter']['meter_id'])")
echo "Added Meter ID: $METER_ID"

echo "Getting Public Key..."
STATUS_RES=$(curl -s "$SIM_URL/api/meters/$METER_ID/status")
echo "Status Response: $STATUS_RES"
PUBLIC_KEY=$(echo $STATUS_RES | python3 -c "import sys, json; print(json.load(sys.stdin)['public_key'])")
echo "Public Key: $PUBLIC_KEY"

# 4. Register Meter in API Gateway
echo "4. Registering Meter in API Gateway..."
JSON_BODY=$(cat <<EOF
{
  "meter_serial": "$METER_ID",
  "meter_public_key": "$PUBLIC_KEY",
  "meter_type": "solar",
  "location_address": "Test Location"
}
EOF
)

REGISTER_METER_RES=$(curl -s -X POST "$API_URL/api/user/meters" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$JSON_BODY")
echo "Meter Registration Response: $REGISTER_METER_RES"

if [[ "$REGISTER_METER_RES" == *"error"* ]]; then
  echo -e "${RED}Meter registration failed.${NC}"
  # Don't exit, maybe it failed because it's already registered?
fi

# 0. Restart Simulator (Skipped to preserve running instance)
# echo "0. Restarting Simulator..."
# ./restart_simulator.sh
# sleep 5

# 5. Wait for readings and verify signature
echo "5. Waiting for Simulator to send readings..."
sleep 15

echo "Checking logs for signature verification..."
grep "Verifying signature" apigateway.log | tail -n 5
grep "Signature verification successful" apigateway.log | tail -n 5

if grep -q "Signature verification successful" apigateway.log; then
  echo -e "${GREEN}SUCCESS: Signature verification confirmed!${NC}"
else
  echo -e "${RED}FAILURE: Signature verification not found in logs.${NC}"
  echo "Recent logs:"
  tail -n 20 apigateway.log
fi
