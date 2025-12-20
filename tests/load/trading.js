/**
 * GridTokenX Trading API Load Tests
 * 
 * Run with: k6 run tests/load/trading.js
 * 
 * Requirements:
 * - Install k6: brew install k6
 * - Start API Gateway on localhost:4000
 * - Have test accounts pre-created or adjust the script
 */

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const ordersCreated = new Counter('orders_created');
const orderCreateDuration = new Trend('order_create_duration');
const orderBookDuration = new Trend('order_book_duration');
const tradeHistoryDuration = new Trend('trade_history_duration');

// Test configuration
export const options = {
    scenarios: {
        // Constant rate of trading activity
        trading_load: {
            executor: 'constant-arrival-rate',
            rate: 10,              // 10 requests per second
            timeUnit: '1s',
            duration: '2m',
            preAllocatedVUs: 20,
            maxVUs: 50,
            tags: { test_type: 'trading_load' },
        },
        // Stress test - high volume trading
        stress: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 30 },
                { duration: '1m', target: 50 },
                { duration: '30s', target: 75 },
                { duration: '30s', target: 100 },
                { duration: '1m', target: 0 },
            ],
            startTime: '2m30s',
            tags: { test_type: 'stress' },
        },
    },
    thresholds: {
        http_req_duration: ['p(95)<1000'],  // 95% under 1 second
        http_req_failed: ['rate<0.05'],      // Error rate under 5%
        errors: ['rate<0.1'],                // Custom errors under 10%
    },
};

const BASE_URL = __ENV.API_URL || 'http://localhost:4000';

// Test user credentials (create these before running)
const TEST_USER = {
    username: __ENV.TEST_USERNAME || 'loadtest_trader',
    password: __ENV.TEST_PASSWORD || 'StrongP@ssw0rd!',
};

let cachedToken = null;

/**
 * Setup function - runs once before all VUs
 */
export function setup() {
    console.log(`Setting up test user for ${BASE_URL}...`);
    // Login to get token
    const loginRes = http.post(
        `${BASE_URL}/api/v1/auth/token`,
        JSON.stringify(TEST_USER),
        { headers: { 'Content-Type': 'application/json' } }
    );

    if (loginRes.status !== 200) {
        console.warn(`Initial login failed (${loginRes.status}). Creating new user...`);

        // Create test user
        const timestamp = Date.now();
        const newUser = {
            username: `trader_${timestamp}`,
            email: `trader_${timestamp}@test.com`,
            password: 'StrongP@ssw0rd!',
            first_name: 'Load',
            last_name: 'Trader',
        };

        const regRes = http.post(
            `${BASE_URL}/api/v1/users`,
            JSON.stringify(newUser),
            { headers: { 'Content-Type': 'application/json' } }
        );

        if (regRes.status !== 200) {
            console.error(`User registration failed: ${regRes.status} - ${regRes.body}`);
            return { token: null, username: null };
        }

        // Login with new user
        const newLoginRes = http.post(
            `${BASE_URL}/api/v1/auth/token`,
            JSON.stringify({ username: newUser.username, password: newUser.password }),
            { headers: { 'Content-Type': 'application/json' } }
        );

        if (newLoginRes.status !== 200) {
            console.error(`Login failed for new user: ${newLoginRes.status} - ${newLoginRes.body}`);
            return { token: null, username: null };
        }

        try {
            const body = JSON.parse(newLoginRes.body);
            console.log(`Successfully registered and logged in as ${newUser.username}`);
            return { token: body.access_token, username: newUser.username };
        } catch (e) {
            console.error(`Failed to parse login response: ${e}`);
            return { token: null, username: null };
        }
    }

    try {
        const body = JSON.parse(loginRes.body);
        console.log(`Log in successful for ${TEST_USER.username}`);
        return { token: body.access_token, username: TEST_USER.username };
    } catch (e) {
        console.error(`Failed to parse login response: ${e}`);
        return { token: null, username: null };
    }
}

/**
 * Main test function
 */
export default function (data) {
    const authHeader = data.token
        ? { 'Authorization': `Bearer ${data.token}`, 'Content-Type': 'application/json' }
        : { 'Content-Type': 'application/json' };

    group('Order Book Read', () => {
        const startTime = Date.now();
        const res = http.get(`${BASE_URL}/api/v1/trading/orderbook`, {
            headers: authHeader,
        });
        orderBookDuration.add(Date.now() - startTime);

        const success = check(res, {
            'order book status is 200': (r) => r.status === 200,
        });
        if (!success) {
            console.error(`Order Book Read failed with status ${res.status}: ${res.body}`);
        }
        errorRate.add(!success);
    });

    sleep(0.2);

    group('Trade History', () => {
        const startTime = Date.now();
        const res = http.get(`${BASE_URL}/api/v1/trading/trades`, {
            headers: authHeader,
        });
        tradeHistoryDuration.add(Date.now() - startTime);

        const success = check(res, {
            'trade history status is 200': (r) => r.status === 200 || r.status === 404,
        });
        if (!success) {
            console.error(`Trade History failed with status ${res.status}: ${res.body}`);
        }
        errorRate.add(res.status >= 500);
    });

    sleep(0.2);

    // Only create orders sometimes to avoid overwhelming the system
    if (Math.random() < 0.3 && data.token) {
        group('Create Order', () => {
            const side = Math.random() < 0.5 ? 'buy' : 'sell';
            const order = {
                side: side,
                order_type: 'limit',
                energy_amount: (Math.random() * 10 + 1).toFixed(2),
                price_per_kwh: (Math.random() * 5 + 1).toFixed(2),
            };

            const startTime = Date.now();
            const res = http.post(
                `${BASE_URL}/api/v1/trading/orders`,
                JSON.stringify(order),
                { headers: authHeader }
            );
            orderCreateDuration.add(Date.now() - startTime);

            const success = check(res, {
                'order created': (r) => r.status === 200 || r.status === 201,
            });
            if (!success) {
                console.error(`Create Order failed with status ${res.status}: ${res.body}`);
            }

            if (success) {
                ordersCreated.add(1);
            }
            errorRate.add(!success);
        });
    }

    sleep(0.5);
}

/**
 * Summary handler
 */
export function handleSummary(data) {
    console.log('\n========== Trading Load Test Summary ==========');
    console.log(`Total requests: ${data.metrics.http_reqs.values.count}`);
    console.log(`Orders created: ${data.metrics.orders_created ? data.metrics.orders_created.values.count : 0}`);
    console.log(`Avg order book latency: ${data.metrics.order_book_duration ? data.metrics.order_book_duration.values.avg.toFixed(2) : 'N/A'}ms`);
    console.log(`95th percentile: ${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms`);
    console.log('================================================\n');

    return {
        'stdout': textSummary(data, { indent: ' ', enableColors: true }),
        'tests/load/results/trading_summary.json': JSON.stringify(data, null, 2),
    };
}

import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.1/index.js';
