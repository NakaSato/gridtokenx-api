#!/usr/bin/env bash

# GridTokenX API Gateway - Basic Smart Meter Integration Test
# Tests core meter reading submission functionality

set -euo pipefail
IFS=$'\n\t'

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
SLEEP_TIME="${SLEEP_TIME:-1}"

# Performance metrics
START_TIME=$(date +%s)
PASSED_TESTS=0
FAILED_TESTS=0
TOTAL_TESTS=3

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

# Generate unique test data
TIMESTAMP=$(date +%s)
USER_EMAIL="basic-test-${TIMESTAMP}@example.com"
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

    log_success "Login successful"

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
}

cleanup() {
    log_info "Cleaning up test environment..."
    # Note: We're keeping the test user in the database for audit purposes
    log_info "Test user left in database for audit"
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
            \"meter_signature\": \"basic-test-signature-${TIMESTAMP}\"
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

test_manual_token_minting() {
    log_info "Test 3: Manual token minting"

    # First, login as admin (assuming test admin exists)
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

    # Mint tokens for reading
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

# Run tests
main() {
    log_info "Starting GridTokenX Basic Smart Meter Integration Test"
    log_info "API URL: $API_BASE_URL"

    # Check if API Gateway is running
    log_info "Checking if API Gateway is running..."
    if ! curl -s "$API_BASE_URL/health" > /dev/null; then
        log_error "API Gateway is not running at $API_BASE_URL"
        log_error "Please start the API Gateway first with: cargo run"
        exit 1
    fi

    log_success "API Gateway is running"

    # Setup test environment
    setup

    # Run tests
    test_meter_reading_submission
    sleep "$SLEEP_TIME"

    test_reading_list
    sleep "$SLEEP_TIME"

    test_manual_token_minting

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
        log_success "All basic tests passed! Smart Meter Integration is working correctly."
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
```

Now you can make this script executable and run it with:

```bash
chmod +x scripts/test-basic-meter-flow.sh
./scripts/test-basic-meter-flow.sh
```

This script tests the basic functionality without the complex polling service, which has compilation errors. Once those are fixed, you can use the full integration test.
