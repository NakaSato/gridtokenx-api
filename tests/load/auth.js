/**
 * GridTokenX API Gateway Load Tests
 * 
 * Run with: k6 run tests/load/auth.js
 * 
 * Requirements: 
 * - Install k6: brew install k6
 * - Start API Gateway on localhost:4000
 */

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const loginDuration = new Trend('login_duration');
const registerDuration = new Trend('register_duration');
const profileDuration = new Trend('profile_duration');

// Test configuration
export const options = {
    scenarios: {
        // Smoke test - light load
        smoke: {
            executor: 'constant-vus',
            vus: 1,
            duration: '30s',
            startTime: '0s',
            tags: { test_type: 'smoke' },
        },
        // Load test - normal expected load
        load: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '1m', target: 10 },   // Ramp up to 10 users
                { duration: '2m', target: 10 },   // Stay at 10 users
                { duration: '1m', target: 0 },    // Ramp down
            ],
            startTime: '35s',
            tags: { test_type: 'load' },
        },
        // Stress test - beyond normal capacity
        stress: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 20 },  // Ramp up
                { duration: '1m', target: 50 },   // Push limits
                { duration: '30s', target: 100 }, // Breaking point
                { duration: '1m', target: 0 },    // Recovery
            ],
            startTime: '5m',
            tags: { test_type: 'stress' },
        },
    },
    thresholds: {
        http_req_duration: ['p(95)<500'],   // 95% of requests under 500ms
        http_req_failed: ['rate<0.01'],      // Error rate under 1%
        errors: ['rate<0.05'],               // Custom error rate under 5%
    },
};

const BASE_URL = __ENV.API_URL || 'http://localhost:4000';

/**
 * Generate unique test credentials
 */
function generateTestUser() {
    const timestamp = Date.now();
    const vuId = __VU;
    return {
        username: `loadtest_${vuId}_${timestamp}`,
        email: `loadtest_${vuId}_${timestamp}@test.com`,
        password: 'StrongP@ssw0rd!',
        first_name: 'Load',
        last_name: 'Test',
    };
}

/**
 * Main test function
 */
export default function () {
    const user = generateTestUser();
    let authToken = null;

    group('Health Check', () => {
        const res = http.get(`${BASE_URL}/api/v1/status`);
        check(res, {
            'status is 200': (r) => r.status === 200,
            'status is healthy': (r) => {
                try {
                    const body = JSON.parse(r.body);
                    return body.status === 'healthy';
                } catch {
                    return false;
                }
            },
        });
        errorRate.add(res.status !== 200);
    });

    group('User Registration', () => {
        const startTime = Date.now();
        const res = http.post(
            `${BASE_URL}/api/v1/users`,
            JSON.stringify(user),
            { headers: { 'Content-Type': 'application/json' } }
        );
        registerDuration.add(Date.now() - startTime);

        const success = check(res, {
            'registration status is 200': (r) => r.status === 200,
            'registration returns message': (r) => {
                try {
                    const body = JSON.parse(r.body);
                    return body.message !== undefined;
                } catch {
                    return false;
                }
            },
        });
        errorRate.add(!success);
    });

    sleep(0.5);

    group('User Login', () => {
        const startTime = Date.now();
        const res = http.post(
            `${BASE_URL}/api/v1/auth/token`,
            JSON.stringify({
                username: user.username,
                password: user.password,
            }),
            { headers: { 'Content-Type': 'application/json' } }
        );
        loginDuration.add(Date.now() - startTime);

        const success = check(res, {
            'login status is 200': (r) => r.status === 200,
            'login returns token': (r) => {
                try {
                    const body = JSON.parse(r.body);
                    authToken = body.access_token;
                    return body.access_token !== undefined && body.access_token !== '';
                } catch {
                    return false;
                }
            },
        });
        errorRate.add(!success);
    });

    sleep(0.5);

    if (authToken) {
        group('Get Profile', () => {
            const startTime = Date.now();
            const res = http.get(`${BASE_URL}/api/v1/users/me`, {
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${authToken}`,
                },
            });
            profileDuration.add(Date.now() - startTime);

            const success = check(res, {
                'profile status is 200': (r) => r.status === 200,
                'profile returns username': (r) => {
                    try {
                        const body = JSON.parse(r.body);
                        return body.username !== undefined;
                    } catch {
                        return false;
                    }
                },
            });
            errorRate.add(!success);
        });
    }

    sleep(1);
}

/**
 * Summary handler - runs at the end of the test
 */
export function handleSummary(data) {
    console.log('\n========== Load Test Summary ==========');
    console.log(`Total requests: ${data.metrics.http_reqs.values.count}`);
    console.log(`Failed requests: ${data.metrics.http_req_failed.values.passes}`);
    console.log(`Avg response time: ${data.metrics.http_req_duration.values.avg.toFixed(2)}ms`);
    console.log(`95th percentile: ${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms`);
    console.log('========================================\n');

    return {
        'stdout': textSummary(data, { indent: ' ', enableColors: true }),
        'tests/load/results/auth_summary.json': JSON.stringify(data, null, 2),
    };
}

import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.1/index.js';
