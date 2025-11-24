#!/bin/bash
# Performance load test for transaction tracking system
# This script stress tests the transaction tracking endpoints

set -e

# Source common test utilities if available
if [ -f "../testing.env" ]; then
    source "../testing.env"
fi

# Configuration
API_URL="${API_URL:-http://localhost:8080}"
ADMIN_EMAIL="${ADMIN_EMAIL:-admin@example.com}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-adminpass123}"
USER_EMAIL="${USER_EMAIL:-test@example.com}"
USER_PASSWORD="${USER_PASSWORD:-password123}"

# Test parameters
CONCURRENT_USERS=${CONCURRENT_USERS:-10}
REQUESTS_PER_USER=${REQUESTS_PER_USER:-50}
WARMUP_REQUESTS=${WARMUP_REQUESTS:-5}
TEST_DURATION=${TEST_DURATION:-60}  # seconds

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Results tracking
declare -A RESULTS
RESULTS[total_requests]=0
RESULTS[successful_requests]=0
RESULTS[failed_requests]=0
RESULTS[min_response_time]=999999
RESULTS[max_response_time]=0
RESULTS[total_response_time]=0

# Function to get authentication tokens
get_auth_tokens() {
    log_info "Getting authentication tokens..."

    # Get admin token
    response=$(curl -s -X POST "${API_URL}/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"${ADMIN_EMAIL}\",\"password\":\"${ADMIN_PASSWORD}\"}")

    admin_token=$(echo "$response" | jq -r '.token')

    if [ "$admin_token" = "null" ] || [ -z "$admin_token" ]; then
        log_error "Failed to get admin token"
        exit 1
    fi

    # Get user token
    response=$(curl -s -X POST "${API_URL}/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"${USER_EMAIL}\",\"password\":\"${USER_PASSWORD}\"}")

    user_token=$(echo "$response" | jq -r '.token')

    if [ "$user_token" = "null" ] || [ -z "$user_token" ]; then
        log_error "Failed to get user token"
        exit 1
    fi

    export ADMIN_TOKEN="$admin_token"
    export USER_TOKEN="$user_token"

    log_info "Authentication tokens obtained successfully"
}

# Function to perform a single request and record metrics
perform_request() {
    local endpoint=$1
    local method=${2:-GET}
    local token=$3
    local request_id=$4

    local start_time=$(date +%s%N)

    response=$(curl -s -w "%{http_code}" -X "${method}" "${API_URL}${endpoint}" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer ${token}" \
        2>/dev/null)

    local http_code="${response: -3}"
    local response_body="${response%???}"

    local end_time=$(date +%s%N)
    local response_time=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds

    # Record results
    RESULTS[total_requests]=$((${RESULTS[total_requests]} + 1))
    RESULTS[total_response_time]=$((${RESULTS[total_response_time]} + response_time))

    if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
        RESULTS[successful_requests]=$((${RESULTS[successful_requests]} + 1))
    else
        RESULTS[failed_requests]=$((${RESULTS[failed_requests]} + 1))
        log_warn "Request ${request_id} failed with HTTP ${http_code}: ${response_body:0:100}..."
    fi

    # Update min/max response times
    if [ $response_time -lt ${RESULTS[min_response_time]} ]; then
        RESULTS[min_response_time]=$response_time
    fi

    if [ $response_time -gt ${RESULTS[max_response_time]} ]; then
        RESULTS[max_response_time]=$response_time
    fi

    return 0
}

# Function to warm up the server
warmup_server() {
    log_info "Warming up server with ${WARMUP_REQUESTS} requests..."

    for ((i=1; i<=WARMUP_REQUESTS; i++)); do
        perform_request "/api/v1/transactions/history" "GET" "$ADMIN_TOKEN" "warmup_${i}"
    done

    # Reset results after warmup
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    log_success "Server warmed up"
}

# Function to run concurrent load test
run_concurrent_test() {
    local endpoint=$1
    local token=$2
    local label=$3

    log_info "Running concurrent load test for ${label}..."

    # Create a temporary file to store PIDs
    local pid_file=$(mktemp)

    # Start concurrent users
    for ((i=1; i<=CONCURRENT_USERS; i++)); do
        (
            for ((j=1; j<=REQUESTS_PER_USER; j++)); do
                perform_request "${endpoint}" "GET" "$token" "${label}_user${i}_req${j}"
                # Small delay to avoid overwhelming the server
                sleep 0.01
            done
        ) &

        # Store PID
        echo $! >> "$pid_file"
    done

    # Wait for all background processes to complete
    local pids=$(cat "$pid_file")
    wait $pids

    # Clean up
    rm -f "$pid_file"

    log_success "Concurrent load test for ${label} completed"
}

# Function to run sustained load test
run_sustained_test() {
    local endpoint=$1
    local token=$2
    local label=$3

    log_info "Running sustained load test for ${label} (${TEST_DURATION}s)..."

    # Create a temporary file to store PIDs
    local pid_file=$(mktemp)

    # Start concurrent users
    for ((i=1; i<=CONCURRENT_USERS; i++)); do
        (
            local end_time=$(($(date +%s) + TEST_DURATION))
            local req_num=1

            while [ $(date +%s) -lt $end_time ]; do
                perform_request "${endpoint}" "GET" "$token" "${label}_user${i}_req${req_num}"
                req_num=$((req_num + 1))
                sleep 0.1
            done
        ) &

        # Store PID
        echo $! >> "$pid_file"
    done

    # Wait for all background processes to complete
    local pids=$(cat "$pid_file")
    wait $pids

    # Clean up
    rm -f "$pid_file"

    log_success "Sustained load test for ${label} completed"
}

# Function to print test results
print_results() {
    local test_name=$1

    echo ""
    echo "========================================="
    echo "Test Results: ${test_name}"
    echo "========================================="

    local total_requests=${RESULTS[total_requests]}
    local successful_requests=${RESULTS[successful_requests]}
    local failed_requests=${RESULTS[failed_requests]}

    echo "Total Requests: $total_requests"
    echo "Successful Requests: $successful_requests"
    echo "Failed Requests: $failed_requests"
    echo "Success Rate: $(echo "scale=2; $successful_requests * 100 / $total_requests" | bc -l)%"

    if [ $total_requests -gt 0 ]; then
        local avg_response_time=$((${RESULTS[total_response_time]} / $total_requests))
        echo "Average Response Time: ${avg_response_time}ms"
        echo "Min Response Time: ${RESULTS[min_response_time]}ms"
        echo "Max Response Time: ${RESULTS[max_response_time]}ms"
    fi

    echo "========================================="
    echo ""
}

# Function to test transaction history performance
test_transaction_history() {
    log_info "Testing transaction history performance..."

    # Reset results
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_concurrent_test "/api/v1/transactions/history" "$ADMIN_TOKEN" "transaction_history"
    print_results "Transaction History - Concurrent Load"

    # Reset results for sustained test
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_sustained_test "/api/v1/transactions/history" "$ADMIN_TOKEN" "transaction_history"
    print_results "Transaction History - Sustained Load"
}

# Function to test transaction statistics performance
test_transaction_stats() {
    log_info "Testing transaction statistics performance..."

    # Reset results
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_concurrent_test "/api/v1/transactions/stats" "$ADMIN_TOKEN" "transaction_stats"
    print_results "Transaction Statistics - Concurrent Load"

    # Reset results for sustained test
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_sustained_test "/api/v1/transactions/stats" "$ADMIN_TOKEN" "transaction_stats"
    print_results "Transaction Statistics - Sustained Load"
}

# Function to test user transactions performance
test_user_transactions() {
    log_info "Testing user transactions performance..."

    # Reset results
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_concurrent_test "/api/v1/transactions/user" "$USER_TOKEN" "user_transactions"
    print_results "User Transactions - Concurrent Load"

    # Reset results for sustained test
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_sustained_test "/api/v1/transactions/user" "$USER_TOKEN" "user_transactions"
    print_results "User Transactions - Sustained Load"
}

# Function to test filtered queries performance
test_filtered_queries() {
    log_info "Testing filtered queries performance..."

    # Reset results
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_concurrent_test "/api/v1/transactions/history?operation_type=trading_order&status=confirmed&limit=50" "$ADMIN_TOKEN" "filtered_queries"
    print_results "Filtered Queries - Concurrent Load"

    # Reset results for sustained test
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_sustained_test "/api/v1/transactions/history?operation_type=trading_order&status=confirmed&limit=50" "$ADMIN_TOKEN" "filtered_queries"
    print_results "Filtered Queries - Sustained Load"
}

# Function to test pagination performance
test_pagination() {
    log_info "Testing pagination performance..."

    # Reset results
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_concurrent_test "/api/v1/transactions/history?limit=100&offset=100" "$ADMIN_TOKEN" "pagination"
    print_results "Pagination - Concurrent Load"

    # Reset results for sustained test
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_sustained_test "/api/v1/transactions/history?limit=100&offset=100" "$ADMIN_TOKEN" "pagination"
    print_results "Pagination - Sustained Load"
}

# Function to test transaction status performance
test_transaction_status() {
    log_info "Testing transaction status performance..."

    # First, get a transaction ID from user transactions
    response=$(curl -s -X GET "${API_URL}/api/v1/transactions/user?limit=1" \
        -H "Authorization: Bearer ${USER_TOKEN}")

    if ! echo "$response" | jq . > /dev/null 2>&1; then
        log_warn "Failed to get user transactions for status test"
        return
    fi

    transaction_id=$(echo "$response" | jq -r '.[0].operation_id // empty')

    if [ -z "$transaction_id" ] || [ "$transaction_id" = "null" ]; then
        log_warn "No transactions found for status test, skipping"
        return
    fi

    # Reset results
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_concurrent_test "/api/v1/transactions/${transaction_id}/status" "$USER_TOKEN" "transaction_status"
    print_results "Transaction Status - Concurrent Load"

    # Reset results for sustained test
    RESULTS[total_requests]=0
    RESULTS[successful_requests]=0
    RESULTS[failed_requests]=0
    RESULTS[min_response_time]=999999
    RESULTS[max_response_time]=0
    RESULTS[total_response_time]=0

    run_sustained_test "/api/v1/transactions/${transaction_id}/status" "$USER_TOKEN" "transaction_status"
    print_results "Transaction Status - Sustained Load"
}

# Function to generate performance report
generate_report() {
    log_info "Generating performance report..."

    local report_file="transaction-performance-report-$(date +%Y%m%d-%H%M%S).txt"

    cat > "$report_file" << EOF
GridTokenX Transaction Tracking Performance Report
Generated on: $(date)
Test Configuration:
- API URL: $API_URL
- Concurrent Users: $CONCURRENT_USERS
- Requests Per User: $REQUESTS_PER_USER
- Test Duration: $TEST_DURATION seconds
- Warmup Requests: $WARMUP_REQUESTS

Performance Tests:
1. Transaction History Performance
2. Transaction Statistics Performance
3. User Transactions Performance
4. Filtered Queries Performance
5. Pagination Performance
6. Transaction Status Performance

Results:
(See detailed output above for each test)

Recommendations:
- Monitor response times during peak usage
- Consider implementing response caching for frequently accessed data
- Optimize database queries for filtered operations
- Implement rate limiting to prevent system overload
EOF

    log_success "Performance report generated: $report_file"
}

# Main execution
main() {
    log_info "Starting Transaction Tracking Performance Tests"
    log_info "Configuration:"
    log_info "- API URL: $API_URL"
    log_info "- Concurrent Users: $CONCURRENT_USERS"
    log_info "- Requests Per User: $REQUESTS_PER_USER"
    log_info "- Test Duration: $TEST_DURATION seconds"
    log_info "- Warmup Requests: $WARMUP_REQUESTS"

    # Check if required tools are available
    if ! command -v jq &> /dev/null; then
        log_error "jq is required but not installed. Please install jq and try again."
        exit 1
    fi

    if ! command -v bc &> /dev/null; then
        log_error "bc is required but not installed. Please install bc and try again."
        exit 1
    fi

    # Check if API is running
    if ! curl -s "${API_URL}/health" > /dev/null; then
        log_error "API server is not running at ${API_URL}. Please start the server and try again."
        exit 1
    fi

    # Get authentication tokens
    get_auth_tokens

    # Warm up the server
    warmup_server

    # Run performance tests
    test_transaction_history
    test_transaction_stats
    test_user_transactions
    test_filtered_queries
    test_pagination
    test_transaction_status

    # Generate performance report
    generate_report

    log_success "All performance tests completed successfully!"
}

# Run the main function
main
