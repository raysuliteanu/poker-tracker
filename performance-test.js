
import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';
import { Trend } from 'k6/metrics';

// Custom Trends for better reporting
const registerLatency = new Trend('register_latency');
const loginLatency = new Trend('login_latency');
const createSessionLatency = new Trend('create_session_latency');
const getSessionsLatency = new Trend('get_sessions_latency');

// --- Test Configuration ---
// VUs can be overridden via K6_VUS environment variable (default: 100)
const TARGET_VUS = __ENV.K6_VUS ? parseInt(__ENV.K6_VUS) : 100;

export const options = {
  stages: [
    { duration: '30s', target: TARGET_VUS }, // Ramp-up to target VUs
    { duration: '60s', target: TARGET_VUS }, // Stay at target VUs
    { duration: '10s', target: 0 },  // Ramp-down to 0 users
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'], // http errors should be less than 1%
    http_req_duration: ['p(95)<500'], // 95% of requests should be below 500ms
    register_latency: ['p(95)<800'],
    login_latency: ['p(95)<500'],
    create_session_latency: ['p(95)<500'],
    get_sessions_latency: ['p(95)<500'],
  },
};

const BASE_URL = 'http://127.0.0.1:8080/api';

// --- Helper Functions ---

// Generates a random string for unique usernames and emails
function randomString(length) {
  const charset = 'abcdefghijklmnopqrstuvwxyz';
  let res = '';
  for (let i = 0; i < length; i++) {
    res += charset.charAt(Math.floor(Math.random() * charset.length));
  }
  return res;
}

// Formats the current date as YYYY-MM-DD
function getTodayDate() {
    const date = new Date();
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
}


// --- Test Lifecycle ---

// setup() is called once before the test starts.
// We create 100 unique users, one for each VU to eliminate contention.
export function setup() {
  const users = [];
  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  console.log(`Creating ${TARGET_VUS} unique users for load test...`);

  for (let i = 0; i < TARGET_VUS; i++) {
    const username = `testuser_${randomString(8)}`;
    const email = `${username}@test.com`;
    const password = 'password123';

    const registerPayload = JSON.stringify({
      username,
      email,
      password,
    });

    const res = http.post(`${BASE_URL}/auth/register`, registerPayload, params);

    if (!check(res, { 'setup: user registered successfully': (r) => r.status === 201 })) {
      console.error(`Failed to register user ${i}: ${res.status} ${res.body}`);
    }

    users.push({ email, password });
  }

  console.log(`Successfully created ${users.length} users`);
  return { users };
}


// The main function that virtual users will execute repeatedly.
export default function (data) {
  // Each VU uses its own unique user (VU IDs are 1-indexed)
  const userIndex = (__VU - 1) % data.users.length;
  const user = data.users[userIndex];

  // 1. Login to get JWT token
  const loginPayload = JSON.stringify({
    email: user.email,
    password: user.password,
  });

  const authParams = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const loginRes = http.post(`${BASE_URL}/auth/login`, loginPayload, authParams);
  check(loginRes, { 'main: user logged in successfully': (r) => r.status === 200 });
  loginLatency.add(loginRes.timings.duration);

  const authToken = loginRes.json('token');
  if (!authToken) {
    console.error('Login failed, no auth token received.');
    return;
  }

  const sessionParams = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${authToken}`,
    },
  };

  // 2. Create a new poker session
  const createSessionPayload = JSON.stringify({
    session_date: getTodayDate(),
    duration_minutes: 60,
    buy_in_amount: 100.50,
    cash_out_amount: 150.75,
    notes: 'Performance test session',
  });

  const createRes = http.post(`${BASE_URL}/sessions`, createSessionPayload, sessionParams);
  check(createRes, { 'main: session created successfully': (r) => r.status === 201 });
  createSessionLatency.add(createRes.timings.duration);

  sleep(1); // Think time between actions

  // 3. Get all poker sessions
  const getRes = http.get(`${BASE_URL}/sessions`, sessionParams);
  check(getRes, { 'main: got sessions successfully': (r) => r.status === 200 });
  getSessionsLatency.add(getRes.timings.duration);

  sleep(2); // Wait before the next iteration
}
