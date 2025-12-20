/**
 * GridTokenX Smart Meter API Load Tests
 * 
 * Run with: k6 run tests/load/meters.js
 * 
 * Requirements:
 * - Install k6: brew install k6
 * - Start API Gateway on localhost:4000
 * - Have authenticated user with registered meter
 */

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const meterListDuration = new Trend('meter_list_duration');
const meterStatusDuration = new Trend('meter_status_duration');
const readingSubmitDuration = new Trend('reading_submit_duration');
const readingsSubmitted = new Counter('readings_submitted');

// Test configuration
export const options = {
    scenarios: {
        // Smoke test - validate functionality
        smoke: {
            executor: 'constant-vus',
            vus: 1,
            duration: '30s',
            startTime: '0s',
            tags: { test_type: 'smoke' },
        },
        // Load test - simulate meter polling from multiple meters
        meter_polling: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 20 },  // Simulate 20 meters
                { duration: '2m', target: 50 },   // Scale to 50 meters
                { duration: '1m', target: 100 },  // Peak load: 100 meters
                { duration: '30s', target: 0 },   // Ramp down
            ],
            startTime: '35s',
            tags: { test_type: 'meter_polling' },
        },
    },
    thresholds: {
        http_req_duration: ['p(95)<1000'],  // 95% under 1 second
        http_req_failed: ['rate<0.05'],      // Error rate under 5%
        errors: ['rate<0.1'],                // Custom errors under 10%
    },
};

const BASE_URL = __ENV.API_URL || 'http://localhost:4000';

let cachedAuth = null;

/**
 * Setup - create test user and meter
 */
export function setup() {
    console.log(`Setting up test environment for ${BASE_URL}...`);

    const timestamp = Date.now();
    const testUser = {
        username: `meter_test_${timestamp}`,
        email: `meter_test_${timestamp}@test.com`,
        password: 'StrongP@ssw0rd!',
        first_name: 'Meter',
        last_name: 'Test',
    };

    // Register user
    const regRes = http.post(
        `${BASE_URL}/api/v1/users`,
        JSON.stringify(testUser),
        { headers: { 'Content-Type': 'application/json' } }
    );

    if (regRes.status !== 200) {
        console.warn(`Registration failed: ${regRes.status} - Trying login...`);
    }

    // Login
    const loginRes = http.post(
        `${BASE_URL}/api/v1/auth/token`,
        JSON.stringify({ username: testUser.username, password: testUser.password }),
        { headers: { 'Content-Type': 'application/json' } }
    );

    if (loginRes.status !== 200) {
        console.error(`Login failed: ${loginRes.status} - ${loginRes.body}`);
        return { token: null, meterId: null };
    }

    let token;
    try {
        token = JSON.parse(loginRes.body).access_token;
    } catch (e) {
        console.error(`Failed to parse token: ${e}`);
        return { token: null, meterId: null };
    }

    console.log(`Successfully authenticated as ${testUser.username}`);

    // Register a test meter
    const meterData = {
        serial_number: `TEST_METER_${timestamp}`,
        location: 'Load Test Location',
    };

    const meterRes = http.post(
        `${BASE_URL}/api/v1/meters`,
        JSON.stringify(meterData),
        { headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` } }
    );

    let meterId = null;
    if (meterRes.status === 200 || meterRes.status === 201) {
        try {
            meterId = JSON.parse(meterRes.body).serial_number || meterData.serial_number;
            console.log(`Registered meter: ${meterId}`);
        } catch (e) {
            meterId = meterData.serial_number;
        }
    } else {
        console.warn(`Meter registration: ${meterRes.status} - Using default meter ID`);
        meterId = meterData.serial_number;
    }

    return { token, meterId, username: testUser.username };
}

/**
 * Main test function
 */
export default function (data) {
    if (!data.token) {
        console.error('No authentication token available');
        return;
    }

    const authHeader = {
        'Authorization': `Bearer ${data.token}`,
        'Content-Type': 'application/json',
    };

    // List meters
    group('List Meters', () => {
        const startTime = Date.now();
        const res = http.get(`${BASE_URL}/api/v1/meters`, {
            headers: authHeader,
        });
        meterListDuration.add(Date.now() - startTime);

        const success = check(res, {
            'list meters status is 200': (r) => r.status === 200,
        });
        errorRate.add(!success);
    });

    sleep(0.2);

    // Get meter status
    if (data.meterId) {
        group('Meter Status', () => {
            const startTime = Date.now();
            const res = http.get(`${BASE_URL}/api/v1/meters/${data.meterId}/status`, {
                headers: authHeader,
            });
            meterStatusDuration.add(Date.now() - startTime);

            const success = check(res, {
                'meter status is 200 or 404': (r) => r.status === 200 || r.status === 404,
            });
            errorRate.add(res.status >= 500);
        });
    }

    sleep(0.2);

    // Submit meter reading (20% of iterations to simulate realistic polling)
    if (Math.random() < 0.2 && data.meterId) {
        group('Submit Reading', () => {
            const reading = {
                energy_produced: parseFloat((Math.random() * 10 + 0.5).toFixed(2)),
                energy_consumed: parseFloat((Math.random() * 5).toFixed(2)),
                timestamp: new Date().toISOString(),
            };

            const startTime = Date.now();
            const res = http.post(
                `${BASE_URL}/api/v1/meters/${data.meterId}/readings`,
                JSON.stringify(reading),
                { headers: authHeader }
            );
            readingSubmitDuration.add(Date.now() - startTime);

            const success = check(res, {
                'reading submitted': (r) => r.status === 200 || r.status === 201,
            });

            if (success) {
                readingsSubmitted.add(1);
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
    console.log('\n========== Meter Load Test Summary ==========');
    console.log(`Total requests: ${data.metrics.http_reqs.values.count}`);
    console.log(`Readings submitted: ${data.metrics.readings_submitted ? data.metrics.readings_submitted.values.count : 0}`);
    console.log(`Avg meter list latency: ${data.metrics.meter_list_duration ? data.metrics.meter_list_duration.values.avg.toFixed(2) : 'N/A'}ms`);
    console.log(`95th percentile: ${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms`);
    console.log('==============================================\n');

    return {
        'stdout': textSummary(data, { indent: ' ', enableColors: true }),
        'tests/load/results/meters_summary.json': JSON.stringify(data, null, 2),
    };
}

import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.1/index.js';
