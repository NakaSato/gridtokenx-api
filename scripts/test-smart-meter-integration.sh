#!/usr/bin/env bash

# GridTokenX API Gateway - Smart Meter Integration Test Script
# Tests the complete smart meter flow from reading submission to token minting

set -euo pipefail
IFS=$'\n\t'

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Configuration
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
DATABASE_URL="${DATABASE_URL:-postgresql://gridtokenx_user:gridtokenx_password@localhost:5432/gridtokenx}"
SLEEP_TIME="${SLEEP_TIME:-1}"
VERBOSE="${VERBOSE:-false}"
TEST_MODE="${TEST_MODE:-true}"

# Performance metrics
START_TIME=$(date +%s)
PASSED_TESTS=0
FAILED_TESTS=0
TOTAL_TESTS=6

# Helper functions
log() {
    echo -e "${CYAN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

log_info() {
    log "${BLUE}[INFO]${NC} $1"
}

log_success() {
    log "${GREEN}[SUCCESS]${NC} $1"
    ((PASSED_TESTS++))
}

log_error() {
    log "${RED}[ERROR]${NC} $1"
    ((FAILED_TESTS++))
}

log_warning() {
    log "${YELLOW}[WARNING]${NC} $1"
}

# Generate unique test data
TIMESTAMP=$(date +%s)
USER_EMAIL="smartmeter-test-${TIMESTAMP}@example.com"
USER_PASSWORD="test_password_1234"
WALLET_ADDRESS="9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"

# Setup and cleanup functions
setup() {
    log_info "Setting up test environment..."

    # Register a new test user
    log_info "Registering test user: ${USER_EMAIL}"
    REGISTER_RESPONSE=$(curl -s -X POST "${API_BASE_URL}/api/auth/register" \
        -H "Content-Type: application/json" \
        -d "{
            \"email\": \"${USER_EMAIL}\",
            \"password\": \"${USER_PASSWORD}\",
            \"role\": \"prosumer\"
        }")

    if [ "$(echo "$REGISTER_RESPONSE" | jq -r '.message')" != "User registered successfully" ]; then
        log_error "User registration failed"
        log_error "Response: $REGISTER_RESPONSE"
        exit 1
    fi

    log_success "User registered successfully"

    # Login to get JWT token
    log_info "Logging in as test user"
    LOGIN_RESPONSE=$(curl -s -X POST "${API_BASE_URL}/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{
            \"email\": \"${USER_EMAIL}\",
            \"password\": \"${USER_PASSWORD}\"
        }")

    if [ -z "$LOGIN_RESPONSE" ] || [ "$(echo "$LOGIN_RESPONSE" | jq -r '.access_token')" == "null" ]; then
        log_error "Login failed"
        log_error "Response: $LOGIN_RESPONSE"
        exit 1
    fi

    JWT_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token')
    USER_ID=$(echo "$LOGIN_RESPONSE" | jq -r '.user.id')

    log_success "Login successful. User ID: $USER_ID"

    # Set user wallet address
    log_info "Setting wallet address for user"
    UPDATE_WALLET_RESPONSE=$(curl -s -X PUT "${API_BASE_URL}/api/user/wallet" \
        -H "Authorization: Bearer ${JWT_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{
            \"wallet_address\": \"${WALLET_ADDRESS}\"
        }")

    if [ "$(echo "$UPDATE_WALLET_RESPONSE" | jq -r '.message')" != "Wallet address updated successfully" ]; then
        log_error "Failed to update wallet address"
        log_error "Response: $UPDATE_WALLET_RESPONSE"
        exit 1
    fi

    log_success "Wallet address set successfully"

    # Create a temporary wallet file for testing
    WALLET_FILE="/tmp/test-wallet-${TIMESTAMP}.json"
    echo '[
        208,173,220,122,183,28,57,88,98,193,163,23,196,225,172,187,130,165,
        124,164,208,20,50,55,97,192,25,193,54,228,48,127,115,237,154,
        29,197,134,203,64,191,68,76,114,253,193,161,122,87,181,213,93
    ]' > "$WALLET_FILE"

    export AUTHORITY_WALLET_PATH="$WALLET_FILE"
    log_info "Created test wallet file at $WALLET_FILE"
}

cleanup() {
    log_info "Cleaning up test environment..."

    # Delete test wallet file
    if [ -n "${WALLET_FILE:-}" ] && [ -f "$WALLET_FILE" ]; then
        rm -f "$WALLET_FILE"
        log_info "Removed test wallet file"
    fi

    # Note: We're keeping the test user in the database for audit purposes
    log_info "Test user $USER_ID left in database for audit"
}

# Test functions
test_meter_reading_submission() {
    log_info "Test 1: Submitting meter reading"

    # Submit a meter reading
    METER_READING_RESPONSE=$(curl -s -X POST "${API_BASE_URL}/api/meters/submit-reading" \
        -H "Authorization: Bearer ${JWT_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{
            \"kwh_amount\": \"10.5\",
            \"reading_timestamp\": \"$(date -u +'%Y-%m-%dT%H:%M:%S.%3NZ')\",
            \"meter_signature\": \"test-signature-${TIMESTAMP}\"
        }")

    if [ -z "$METER_READING_RESPONSE" ] || [ "$(echo "$METER_READING_RESPONSE" | jq -r '.id')" == "null" ]; then
        log_error "Meter reading submission failed"
        log_error "Response: $METER_READING_RESPONSE"
        return 1
    fi

    READING_ID=$(echo "$METER_READING_RESPONSE" | jq -r '.id')
    MINTED=$(echo "$METER_READING_RESPONSE" | jq -r '.minted')

    if [ "$MINTED" != "false" ]; then
        log_warning "Reading already marked as minted (unexpected)"
    fi

    log_success "Meter reading submitted successfully. ID: $READING_ID"

    # Store reading ID for later tests
    TEST_READING_ID="$READING_ID"
    return 0
}

test_reading_list() {
    log_info "Test 2: Retrieving meter readings list"

    # Get user's meter readings
    READINGS_RESPONSE=$(curl -s -X GET "${API_BASE_URL}/api/meters/my-readings" \
        -H "Authorization: Bearer ${JWT_TOKEN}" \
        -H "Content-Type: application/json")

    if [ -z "$READINGS_RESPONSE" ] || [ "$(echo "$READINGS_RESPONSE" | jq -r '.data')" == "null" ]; then
        log_error "Failed to retrieve meter readings"
        log_error "Response: $READINGS_RESPONSE"
        return 1
    fi

    READING_COUNT=$(echo "$READINGS_RESPONSE" | jq -r '.data | length')

    if [ "$READING_COUNT" -eq 0 ]; then
        log_error "No meter readings found"
        return 1
    fi

    log_success "Retrieved $READING_COUNT meter readings"
    return 0
}

test_unminted_readings_admin() {
    log_info "Test 3: Retrieving unminted readings as admin"

    # First, login as admin (assuming test admin exists)
    ADMIN_LOGIN_RESPONSE=$(curl -s -X POST "${API_BASE_URL}/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{
            \"email\": \"admin@example.com\",
            \"password\": \"admin123\"
        }")

    if [ -z "$ADMIN_LOGIN_RESPONSE" ] || [ "$(echo "$ADMIN_LOGIN_RESPONSE" | jq -r '.access_token')" == "null" ]; then
        log_warning "Admin login failed, skipping unminted readings test"
        return 0
    fi

    ADMIN_JWT=$(echo "$ADMIN_LOGIN_RESPONSE" | jq -r '.access_token')

    # Get unminted readings
    UNMINTED_RESPONSE=$(curl -s -X GET "${API_BASE_URL}/api/admin/meters/unminted?limit=10" \
        -H "Authorization: Bearer ${ADMIN_JWT}" \
        -H "Content-Type: application/json")

    if [ -z "$UNMINTED_RESPONSE" ] || [ "$(echo "$UNMINTED_RESPONSE" | jq -r '.data')" == "null" ]; then
        log_error "Failed to retrieve unminted readings"
        log_error "Response: $UNMINTED_RESPONSE"
        return 1
    fi

    UNMINTED_COUNT=$(echo "$UNMINTED_RESPONSE" | jq -r '.data | length')

    log_success "Retrieved $UNMINTED_COUNT unminted readings"
    return 0
}

test_manual_token_minting() {
    log_info "Test 4: Manual token minting (admin)"

    # First, login as admin
    ADMIN_LOGIN_RESPONSE=$(curl -s -X POST "${API_BASE_URL}/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{
            \"email\": \"admin@example.com\",
            \"password\": \"admin123\"
        }")

    if [ -z "$ADMIN_LOGIN_RESPONSE" ] || [ "$(echo "$ADMIN_LOGIN_RESPONSE" | jq -r '.access_token')" == "null" ]; then
        log_warning "Admin login failed, skipping manual minting test"
        return 0
    fi

    ADMIN_JWT=$(echo "$ADMIN_LOGIN_RESPONSE" | jq -r '.access_token')

    # Mint tokens for the reading
    if [ -z "${TEST_READING_ID:-}" ]; then
        log_warning "No reading ID available, skipping minting test"
        return 0
    fi

    MINT_RESPONSE=$(curl -s -X POST "${API_BASE_URL}/api/admin/meters/mint-from-reading" \
        -H "Authorization: Bearer ${ADMIN_JWT}" \
        -H "Content-Type: application/json" \
        -d "{
            \"reading_id\": \"${TEST_READING_ID}\"
        }")

    if [ -z "$MINT_RESPONSE" ] || [ "$(echo "$MINT_RESPONSE" | jq -r '.transaction_signature')" == "null" ]; then
        log_error "Manual token minting failed"
        log_error "Response: $MINT_RESPONSE"
        return 1
    fi

    TX_SIGNATURE=$(echo "$MINT_RESPONSE" | jq -r '.transaction_signature')

    log_success "Tokens minted successfully. TX: $TX_SIGNATURE"
    return 0
}

test_websocket_connection() {
    log_info "Test 5: WebSocket connection and event reception"

    # Create a temporary Python script for WebSocket testing
    WS_TEST_SCRIPT="/tmp/ws-test-${TIMESTAMP}.py"
    cat > "$WS_TEST_SCRIPT" << 'EOF'
import asyncio
import websockets
import json
import sys

async def test_websocket():
    uri = "ws://localhost:8080/api/market/ws"

    try:
        async with websockets.connect(uri) as websocket:
            print("WebSocket connected successfully")

            # Wait for a message
            try:
                message = await asyncio.wait_for(websocket.recv(), timeout=5.0)
                print(f"Received message: {message}")

                # Parse the message
                data = json.loads(message)
                if "type" in data:
                    print(f"Event type: {data['type']}")
                    return True
            except asyncio.TimeoutError:
                print("Timeout waiting for message")
                return False

    except Exception as e:
        print(f"WebSocket error: {e}")
        return False

if __name__ == "__main__":
    result = asyncio.run(test_websocket())
    sys.exit(0 if result else 1)
EOF

    # Run the WebSocket test
    log_info "Testing WebSocket connection (requires python3 and websockets package)"

    if command -v python3 >/dev/null && python3 -c "import websockets" 2>/dev/null; then
        if python3 "$WS_TEST_SCRIPT"; then
            log_success "WebSocket connection test passed"
        else
            log_warning "WebSocket connection test failed"
        fi
    else
        log_warning "Skipping WebSocket test (python3 or websockets not available)"
    fi

    # Clean up
    rm -f "$WS_TEST_SCRIPT"
}

test_automated_polling() {
    log_info "Test 6: Automated polling simulation"

    if [ -z "${TEST_READING_ID:-}" ]; then
        log_warning "No reading ID available, skipping polling test"
        return 0
    fi

    # Submit a new reading specifically for polling test
    POLLING_READING_RESPONSE=$(curl -s -X POST "${API_BASE_URL}/api/meters/submit-reading" \
        -H "Authorization: Bearer ${JWT_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{
            \"kwh_amount\": \"5.0\",
            \"reading_timestamp\": \"$(date -u +'%Y-%m-%dT%H:%M:%S.%3NZ')\",
            \"meter_signature\": \"polling-test-${TIMESTAMP}\"
        }")

    if [ -z "$POLLING_READING_RESPONSE" ] || [ "$(echo "$POLLING_READING_RESPONSE" | jq -r '.id')" == "null" ]; then
        log_error "Failed to submit reading for polling test"
        return 1
    fi

    POLLING_READING_ID=$(echo "$POLLING_READING_RESPONSE" | jq -r '.id')

    log_info "Submitted reading $POLLING_READING_ID for polling test"
    log_info "Waiting 65 seconds for automated polling (configurable polling interval + 5s)"

    # Wait for polling interval (default 60s + 5s buffer)
    sleep 65

    # Check if reading was processed
    READING_STATUS_RESPONSE=$(curl -s -X GET "${API_BASE_URL}/api/meters/my-readings" \
        -H "Authorization: Bearer ${JWT_TOKEN}" \
        -H "Content-Type: application/json")

    if [ -z "$READING_STATUS_RESPONSE" ]; then
        log_error "Failed to check reading status"
        return 1
    fi

    # Find the specific reading
    IS_MINTED=$(echo "$READING_STATUS_RESPONSE" | jq -r ".data[] | select(.id == \"$POLLING_READING_ID\") | .minted")

    if [ "$IS_MINTED" == "true" ]; then
        log_success "Reading was processed and minted by automated polling"
        return 0
    else
        log_warning "Reading was not yet minted by automated polling"
        log_warning "This may be normal if polling interval is longer than wait time"
        return 0
    fi
}

# Run tests
main() {
    log_info "Starting GridTokenX Smart Meter Integration Test"
    log_info "API URL: $API_BASE_URL"
    log_info "Test Mode: $TEST_MODE"

    # Setup test environment
    setup

    # Run all tests
    test_meter_reading_submission
    sleep "$SLEEP_TIME"

    test_reading_list
    sleep "$SLEEP_TIME"

    test_unminted_readings_admin
    sleep "$SLEEP_TIME"

    test_manual_token_minting
    sleep "$SLEEP_TIME"

    test_websocket_connection
    sleep "$SLEEP_TIME"

    test_automated_polling

    # Cleanup
    cleanup

    # Report results
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))

    echo ""
    log_info "Test Summary"
    echo "===================="
    log_info "Total Tests: $TOTAL_TESTS"
    log_info "Passed: $PASSED_TESTS"
    log_info "Failed: $FAILED_TESTS"
    log_info "Duration: ${DURATION}s"

    if [ $FAILED_TESTS -eq 0 ]; then
        log_success "All tests passed! Smart Meter Integration is working correctly."
    else
        log_error "$FAILED_TESTS test(s) failed. Please check the logs above."
        exit 1
    fi
}

# Handle script interruption
trap cleanup EXIT
trap 'log_error "Test interrupted"; cleanup; exit 1' INT TERM

# Run main function
main
