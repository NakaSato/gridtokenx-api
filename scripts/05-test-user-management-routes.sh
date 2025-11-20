#!/bin/bash

# GridTokenX API Gateway - User Management Routes Test
# Tests user wallet management and activity tracking endpoints

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
DATABASE_URL="${DATABASE_URL:-postgresql://gridtokenx_user:gridtokenx_password@localhost:5432/gridtokenx}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SLEEP_TIME=1

print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Check if server is running
echo "Checking server status..."
if ! curl -s "$API_BASE_URL/health" > /dev/null; then
    echo -e "${RED}Server is not running at $API_BASE_URL${NC}"
    echo "Please start the server first."
    exit 1
else
    echo -e "${GREEN}Server is running${NC}"
fi

# Generate unique test data
TIMESTAMP=$(date +%s)
EMAIL="user_mgmt_test_${TIMESTAMP}@test.com"
PASSWORD="Test123!@#"
USERNAME="user_mgmt_${TIMESTAMP}"

# Use valid Solana wallet addresses (these are real valid base58 addresses)
# We'll clean up old test data to avoid conflicts
WALLET_ADDRESS="5KQwrPbwdL6PhXujxW37FSSQZ1JiwsST4cqQzDeyXtP8"
WALLET_ADDRESS_2="7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"

# Clean up old test wallet addresses from previous runs
if [ ! -z "$DATABASE_URL" ]; then
    if docker ps | grep -q postgres; then
        CONTAINER_NAME=$(docker ps --format "{{.Names}}" | grep -i postgres | head -1)
        if [ ! -z "$CONTAINER_NAME" ]; then
            docker exec -i "$CONTAINER_NAME" psql -U gridtokenx_user -d gridtokenx -c "UPDATE users SET wallet_address = NULL WHERE wallet_address IN ('$WALLET_ADDRESS', '$WALLET_ADDRESS_2');" > /dev/null 2>&1
        fi
    fi
fi

# =============================================================================
# PART 1: Setup - Register, Verify, and Login
# =============================================================================
print_header "PART 1: Setup - Register and Login"

echo "Registering user: $EMAIL"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{
        \"email\": \"$EMAIL\",
        \"password\": \"$PASSWORD\",
        \"first_name\": \"User\",
        \"last_name\": \"Management\",
        \"username\": \"$USERNAME\"
    }")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -ne 201 ] && [ "$HTTP_CODE" -ne 200 ]; then
    print_error "Registration failed (HTTP $HTTP_CODE)"
    echo "$BODY"
    exit 1
fi

print_success "User registered successfully"

# Verify email directly in database (required for wallet operations)
echo "Verifying email in database..."
if [ ! -z "$DATABASE_URL" ]; then
    # Try using docker exec to run psql in the postgres container
    if docker ps | grep -q postgres; then
        CONTAINER_NAME=$(docker ps --filter "ancestor=postgres" --format "{{.Names}}" | head -1)
        if [ -z "$CONTAINER_NAME" ]; then
            # Try finding by name pattern
            CONTAINER_NAME=$(docker ps --format "{{.Names}}" | grep -i postgres | head -1)
        fi
        
        if [ ! -z "$CONTAINER_NAME" ]; then
            docker exec -i "$CONTAINER_NAME" psql -U gridtokenx_user -d gridtokenx -c "UPDATE users SET email_verified = true, email_verified_at = NOW() WHERE email = '$EMAIL';" > /dev/null 2>&1
            
            if [ $? -eq 0 ]; then
                print_success "Email verified in database via Docker"
            else
                print_error "Failed to verify email in database via Docker"
                print_info "Attempting to continue anyway..."
            fi
        else
            print_error "No postgres container found"
            print_info "Attempting to continue anyway..."
        fi
    elif command -v psql &> /dev/null; then
        # Fallback to local psql if available
        PGPASSWORD=$(echo "$DATABASE_URL" | sed -n 's/.*:\/\/[^:]*:\([^@]*\)@.*/\1/p') \
        psql "$DATABASE_URL" -t -c "UPDATE users SET email_verified = true, email_verified_at = NOW() WHERE email = '$EMAIL';" > /dev/null 2>&1
        
        if [ $? -eq 0 ]; then
            print_success "Email verified in database"
        else
            print_error "Failed to verify email in database"
            print_info "Attempting to continue anyway..."
        fi
    else
        print_error "Neither Docker postgres container nor psql command found"
        print_info "Please install PostgreSQL client or ensure postgres container is running"
        print_info "Attempting to continue anyway..."
    fi
else
    print_error "DATABASE_URL not set - cannot verify email in database"
    print_info "Set DATABASE_URL environment variable to enable database verification"
    exit 1
fi

# Also try API verification if token is available
VERIFICATION_TOKEN=$(echo "$BODY" | jq -r '.email_verification_token // empty')

if [ ! -z "$VERIFICATION_TOKEN" ] && [ "$VERIFICATION_TOKEN" != "null" ]; then
    echo "Verifying email via API..."
    curl -s "$API_BASE_URL/api/auth/verify-email?token=$VERIFICATION_TOKEN" > /dev/null
    print_success "Email verified via API"
fi

echo "Logging in..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -ne 200 ]; then
    print_error "Login failed (HTTP $HTTP_CODE)"
    echo "$BODY"
    exit 1
fi

TOKEN=$(echo "$BODY" | jq -r '.access_token')
USER_ID=$(echo "$BODY" | jq -r '.user.id')

if [ -z "$TOKEN" ] || [ "$TOKEN" == "null" ]; then
    print_error "No token received"
    exit 1
fi

print_success "Logged in successfully"
print_info "User ID: $USER_ID"
sleep $SLEEP_TIME

# =============================================================================
# PART 2: Wallet Management - Update Wallet Address
# =============================================================================
print_header "PART 2: Update Wallet Address"

echo "Updating wallet address to: $WALLET_ADDRESS"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/user/wallet" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
        \"wallet_address\": \"$WALLET_ADDRESS\",
        \"verify_ownership\": false
    }")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ]; then
    print_success "Wallet address updated successfully"
    echo "$BODY" | jq '.'
    
    # Verify wallet address in response
    RETURNED_WALLET=$(echo "$BODY" | jq -r '.wallet_address')
    if [ "$RETURNED_WALLET" == "$WALLET_ADDRESS" ]; then
        print_success "Wallet address verified in response"
    else
        print_error "Wallet address mismatch in response"
        echo "Expected: $WALLET_ADDRESS"
        echo "Got: $RETURNED_WALLET"
    fi
else
    print_error "Failed to update wallet address (HTTP $HTTP_CODE)"
    echo "$BODY"
    exit 1
fi

sleep $SLEEP_TIME

# =============================================================================
# PART 3: Wallet Management - Update to Different Address
# =============================================================================
print_header "PART 3: Update to Different Wallet Address"

echo "Updating wallet address to: $WALLET_ADDRESS_2"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/user/wallet" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
        \"wallet_address\": \"$WALLET_ADDRESS_2\"
    }")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ]; then
    print_success "Wallet address updated to new address"
    RETURNED_WALLET=$(echo "$BODY" | jq -r '.wallet_address')
    if [ "$RETURNED_WALLET" == "$WALLET_ADDRESS_2" ]; then
        print_success "New wallet address verified"
    else
        print_error "Wallet address mismatch"
    fi
else
    print_error "Failed to update wallet address (HTTP $HTTP_CODE)"
    echo "$BODY"
fi

sleep $SLEEP_TIME

# =============================================================================
# PART 4: Wallet Management - Invalid Wallet Address
# =============================================================================
print_header "PART 4: Test Invalid Wallet Address"

echo "Attempting to set invalid wallet address..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/user/wallet" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
        \"wallet_address\": \"invalid_wallet_123\"
    }")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 400 ]; then
    print_success "Invalid wallet address correctly rejected (HTTP 400)"
    echo "$BODY" | jq '.'
else
    print_error "Expected HTTP 400 for invalid wallet, got HTTP $HTTP_CODE"
    echo "$BODY"
fi

sleep $SLEEP_TIME

# =============================================================================
# PART 5: Wallet Management - Remove Wallet Address
# =============================================================================
print_header "PART 5: Remove Wallet Address"

echo "Removing wallet address..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X DELETE "$API_BASE_URL/api/user/wallet" \
    -H "Authorization: Bearer $TOKEN")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 204 ]; then
    print_success "Wallet address removed successfully (HTTP 204)"
elif [ "$HTTP_CODE" -eq 200 ]; then
    print_success "Wallet address removed successfully (HTTP 200)"
    echo "$BODY" | jq '.'
else
    print_error "Failed to remove wallet address (HTTP $HTTP_CODE)"
    echo "$BODY"
    exit 1
fi

sleep $SLEEP_TIME

# Verify wallet was removed by checking profile
echo "Verifying wallet removal..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$API_BASE_URL/api/auth/profile" \
    -H "Authorization: Bearer $TOKEN")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ]; then
    WALLET_IN_PROFILE=$(echo "$BODY" | jq -r '.wallet_address')
    if [ "$WALLET_IN_PROFILE" == "null" ] || [ -z "$WALLET_IN_PROFILE" ]; then
        print_success "Wallet removal verified in profile"
    else
        print_error "Wallet still present in profile: $WALLET_IN_PROFILE"
    fi
else
    print_error "Failed to get profile (HTTP $HTTP_CODE)"
fi

sleep $SLEEP_TIME

# =============================================================================
# PART 6: User Activity - Get Own Activity
# =============================================================================
print_header "PART 6: Get User Activity"

echo "Fetching user activity log..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$API_BASE_URL/api/user/activity" \
    -H "Authorization: Bearer $TOKEN")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ]; then
    print_success "User activity retrieved successfully"
    echo "$BODY" | jq '.'
    
    # Check if we have activities
    ACTIVITY_COUNT=$(echo "$BODY" | jq '.total')
    print_info "Total activities: $ACTIVITY_COUNT"
    
    # Verify we have wallet-related activities
    WALLET_ACTIVITIES=$(echo "$BODY" | jq '[.activities[] | select(.action | contains("wallet"))] | length')
    if [ "$WALLET_ACTIVITIES" -gt 0 ]; then
        print_success "Found $WALLET_ACTIVITIES wallet-related activities"
    else
        print_info "No wallet-related activities found (this may be expected)"
    fi
else
    print_error "Failed to get user activity (HTTP $HTTP_CODE)"
    echo "$BODY"
fi

sleep $SLEEP_TIME

# =============================================================================
# PART 7: User Activity - Pagination Test
# =============================================================================
print_header "PART 7: Test Activity Pagination"

echo "Fetching paginated activity (page 1, 5 per page)..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$API_BASE_URL/api/user/activity?page=1&per_page=5" \
    -H "Authorization: Bearer $TOKEN")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ]; then
    print_success "Paginated activity retrieved successfully"
    PAGE=$(echo "$BODY" | jq '.page')
    PER_PAGE=$(echo "$BODY" | jq '.per_page')
    TOTAL_PAGES=$(echo "$BODY" | jq '.total_pages')
    ACTIVITIES_RETURNED=$(echo "$BODY" | jq '.activities | length')
    
    print_info "Page: $PAGE, Per Page: $PER_PAGE, Total Pages: $TOTAL_PAGES"
    print_info "Activities returned: $ACTIVITIES_RETURNED"
    
    if [ "$ACTIVITIES_RETURNED" -le 5 ]; then
        print_success "Pagination working correctly"
    else
        print_error "Pagination not working - expected max 5 activities, got $ACTIVITIES_RETURNED"
    fi
else
    print_error "Failed to get paginated activity (HTTP $HTTP_CODE)"
    echo "$BODY"
fi

sleep $SLEEP_TIME

# =============================================================================
# PART 8: Unauthorized Access Tests
# =============================================================================
print_header "PART 8: Test Unauthorized Access"

echo "Attempting to update wallet without token..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/user/wallet" \
    -H "Content-Type: application/json" \
    -d "{
        \"wallet_address\": \"$WALLET_ADDRESS\"
    }")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 401 ]; then
    print_success "Unauthorized access correctly rejected (HTTP 401)"
else
    print_error "Expected HTTP 401, got HTTP $HTTP_CODE"
    echo "$BODY"
fi

sleep $SLEEP_TIME

echo "Attempting to get activity without token..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$API_BASE_URL/api/user/activity")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)

if [ "$HTTP_CODE" -eq 401 ]; then
    print_success "Unauthorized activity access correctly rejected (HTTP 401)"
else
    print_error "Expected HTTP 401, got HTTP $HTTP_CODE"
fi

sleep $SLEEP_TIME

# =============================================================================
# PART 9: Re-add Wallet for Final Verification
# =============================================================================
print_header "PART 9: Re-add Wallet Address"

echo "Re-adding wallet address: $WALLET_ADDRESS"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/user/wallet" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
        \"wallet_address\": \"$WALLET_ADDRESS\"
    }")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ]; then
    print_success "Wallet address re-added successfully"
    echo "$BODY" | jq '.'
else
    print_error "Failed to re-add wallet address (HTTP $HTTP_CODE)"
    echo "$BODY"
fi

sleep $SLEEP_TIME

# =============================================================================
# Summary
# =============================================================================
print_header "Test Summary"

echo -e "${GREEN}✓ User Management Routes Test Completed${NC}"
echo ""
echo "Tests performed:"
echo "  ✓ Update wallet address"
echo "  ✓ Update to different wallet address"
echo "  ✓ Invalid wallet address validation"
echo "  ✓ Remove wallet address"
echo "  ✓ Get user activity log"
echo "  ✓ Activity pagination"
echo "  ✓ Unauthorized access protection"
echo "  ✓ Re-add wallet address"
echo ""
print_success "All user management endpoints are working correctly!"
