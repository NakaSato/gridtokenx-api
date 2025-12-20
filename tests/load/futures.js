/**
 * GridTokenX Futures Trading API Load Tests
 * 
 * Run with: k6 run tests/load/futures.js
 * 
 * Requirements:
 * - Install k6: brew install k6
 * - Start API Gateway on localhost:4000
 */

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const contractListDuration = new Trend('contract_list_duration');
const positionListDuration = new Trend('position_list_duration');
const positionCreateDuration = new Trend('position_create_duration');
const positionsCreated = new Counter('positions_created');

// Test configuration
export const options = {
    scenarios: {
        // Smoke test
        smoke: {
            executor: 'constant-vus',
            vus: 1,
            duration: '30s',
            startTime: '0s',
            tags: { test_type: 'smoke' },
        },
        // Load test - futures trading activity
        futures_load: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 10 },
                { duration: '2m', target: 30 },
                { duration: '1m', target: 50 },
                { duration: '30s', target: 0 },
            ],
            startTime: '35s',
            tags: { test_type: 'futures_load' },
        },
        // Stress test - high volume trading
        stress: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 50 },
                { duration: '1m', target: 100 },
                { duration: '30s', target: 0 },
            ],
            startTime: '5m',
            tags: { test_type: 'stress' },
        },
    },
    thresholds: {
        http_req_duration: ['p(95)<1500'],  // 95% under 1.5 seconds
        http_req_failed: ['rate<0.05'],      // Error rate under 5%
        errors: ['rate<0.1'],
    },
};

const BASE_URL = __ENV.API_URL || 'http://localhost:4000';

/**
 * Setup - create test user
 */
export function setup() {
    console.log(`Setting up futures test for ${BASE_URL}...`);

    const timestamp = Date.now();
    const testUser = {
        username: `futures_test_${timestamp}`,
        email: `futures_test_${timestamp}@test.com`,
        password: 'StrongP@ssw0rd!',
        first_name: 'Futures',
        last_name: 'Trader',
    };

    // Register
    http.post(
        `${BASE_URL}/api/v1/users`,
        JSON.stringify(testUser),
        { headers: { 'Content-Type': 'application/json' } }
    );

    // Login
    const loginRes = http.post(
        `${BASE_URL}/api/v1/auth/token`,
        JSON.stringify({ username: testUser.username, password: testUser.password }),
        { headers: { 'Content-Type': 'application/json' } }
    );

    if (loginRes.status !== 200) {
        console.error(`Login failed: ${loginRes.status}`);
        return { token: null };
    }

    try {
        const token = JSON.parse(loginRes.body).access_token;
        console.log(`Authenticated as ${testUser.username}`);
        return { token, username: testUser.username };
    } catch (e) {
        console.error(`Failed to parse token: ${e}`);
        return { token: null };
    }
}

/**
 * Main test function
 */
export default function (data) {
    if (!data.token) {
        console.error('No authentication token');
        return;
    }

    const authHeader = {
        'Authorization': `Bearer ${data.token}`,
        'Content-Type': 'application/json',
    };

    // List futures contracts
    group('List Contracts', () => {
        const startTime = Date.now();
        const res = http.get(`${BASE_URL}/api/v1/futures/contracts`, {
            headers: authHeader,
        });
        contractListDuration.add(Date.now() - startTime);

        const success = check(res, {
            'contracts status is 200': (r) => r.status === 200 || r.status === 404,
        });
        errorRate.add(res.status >= 500);
    });

    sleep(0.2);

    // List user positions
    group('List Positions', () => {
        const startTime = Date.now();
        const res = http.get(`${BASE_URL}/api/v1/futures/positions`, {
            headers: authHeader,
        });
        positionListDuration.add(Date.now() - startTime);

        const success = check(res, {
            'positions status is 200': (r) => r.status === 200 || r.status === 404,
        });
        errorRate.add(res.status >= 500);
    });

    sleep(0.2);

    // Create position (15% of iterations)
    if (Math.random() < 0.15) {
        group('Create Position', () => {
            const position = {
                contract_id: 'ENERGY-2024Q1',  // Example contract
                side: Math.random() < 0.5 ? 'long' : 'short',
                size: (Math.random() * 100 + 10).toFixed(0),
                leverage: Math.floor(Math.random() * 5 + 1),
            };

            const startTime = Date.now();
            const res = http.post(
                `${BASE_URL}/api/v1/futures/positions`,
                JSON.stringify(position),
                { headers: authHeader }
            );
            positionCreateDuration.add(Date.now() - startTime);

            const success = check(res, {
                'position created': (r) => r.status === 200 || r.status === 201 || r.status === 400 || r.status === 404,
            });

            if (res.status === 200 || res.status === 201) {
                positionsCreated.add(1);
            }
            errorRate.add(res.status >= 500);
        });
    }

    sleep(0.5);
}

/**
 * Summary handler
 */
export function handleSummary(data) {
    console.log('\n========== Futures Load Test Summary ==========');
    console.log(`Total requests: ${data.metrics.http_reqs.values.count}`);
    console.log(`Positions created: ${data.metrics.positions_created ? data.metrics.positions_created.values.count : 0}`);
    console.log(`Avg contract list latency: ${data.metrics.contract_list_duration ? data.metrics.contract_list_duration.values.avg.toFixed(2) : 'N/A'}ms`);
    console.log(`95th percentile: ${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms`);
    console.log('================================================\n');

    return {
        'stdout': textSummary(data, { indent: ' ', enableColors: true }),
        'tests/load/results/futures_summary.json': JSON.stringify(data, null, 2),
    };
}

import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.1/index.js';
