import { check, group, sleep } from "k6";
import http from "k6/http";
import exec from "k6/execution";

export const options = {
  scenarios: {
    auth_flow: {
      executor: "ramping-vus",
      stages: [
        { duration: __ENV.K6_WARMUP_DURATION || "30s", target: Number(__ENV.K6_WARMUP_VUS || 5) },
        { duration: __ENV.K6_STEADY_DURATION || "2m", target: Number(__ENV.K6_TARGET_VUS || 25) },
        { duration: __ENV.K6_COOLDOWN_DURATION || "30s", target: 0 },
      ],
    },
  },
  thresholds: {
    http_req_failed: [`rate<${__ENV.K6_MAX_FAILURE_RATE || 0.05}`],
    http_req_duration: [`p(95)<${__ENV.K6_P95_MS || 750}`],
  },
};

const baseUrl = (__ENV.AUTH_BASE_URL || "http://127.0.0.1:8000/api/v1/auth").replace(/\/$/, "");
const password = __ENV.AUTH_TEST_PASSWORD || "password123";
const runId = __ENV.K6_RUN_ID || `${Date.now()}`;

let authState;

function jsonHeaders(extra = {}) {
  return {
    headers: {
      "Content-Type": "application/json",
      ...extra,
    },
  };
}

function protectedHeaders(auth) {
  return {
    headers: {
      Authorization: `Bearer ${auth.accessToken}`,
      user_id: String(auth.userId),
      session_token: auth.refreshToken,
      session_id: auth.sessionId,
      Cookie: `auth_cookie=${auth.authCookie}`,
    },
  };
}

function authCookieFrom(response) {
  const setCookie = response.headers["Set-Cookie"] || response.headers["set-cookie"] || "";
  const match = setCookie.match(/auth_cookie=([^;]+)/);
  return match ? match[1] : "";
}

function saveRotatedTokens(response, auth) {
  let body;
  try {
    body = response.json();
  } catch (_) {
    return auth;
  }

  const payload = body.response || {};
  return {
    ...auth,
    accessToken: payload.access_token || auth.accessToken,
    refreshToken: payload.refresh_token || auth.refreshToken,
    sessionId: payload.session_id || auth.sessionId,
  };
}

function authenticateVu() {
  if (authState) {
    return authState;
  }

  const vu = exec.vu.idInTest;
  const unique = `${runId}_${vu}`;
  const email = `k6_${unique}@example.com`;
  const phoneSeed = `${runId}${String(vu).padStart(4, "0")}`.replace(/\D/g, "");
  const phoneNumber = phoneSeed.slice(-10).padStart(10, "1");

  const registerPayload = {
    first_name: "K6",
    last_name: `User${vu}`,
    email,
    password,
    country: "Load Test",
    country_code: "LT",
    phone_number: phoneNumber,
  };

  const registerResponse = http.post(
    `${baseUrl}/register`,
    JSON.stringify(registerPayload),
    jsonHeaders(),
  );

  check(registerResponse, {
    "register or duplicate handled": (res) => [201, 403].includes(res.status),
  });

  const loginResponse = http.post(
    `${baseUrl}/login`,
    JSON.stringify({ email, password }),
    jsonHeaders(),
  );

  check(loginResponse, {
    "login succeeded": (res) => res.status === 200,
    "login returned body": (res) => Boolean(res.body && res.body.length),
  });

  const body = loginResponse.json();
  const payload = body.response || {};
  const user = payload.user_profile || {};

  authState = {
    email,
    userId: user.id,
    accessToken: payload.access_token,
    refreshToken: payload.refresh_token,
    sessionId: payload.session_id,
    authCookie: authCookieFrom(loginResponse),
  };

  return authState;
}

export default function () {
  const auth = authenticateVu();

  group("protected auth API", () => {
    const requests = [
      ["list roles", "GET", `${baseUrl}/roles`],
      ["list permissions", "GET", `${baseUrl}/permissions`],
      ["list own sessions", "GET", `${baseUrl}/sessions/user/${auth.userId}`],
      ["get current session", "GET", `${baseUrl}/sessions/${auth.sessionId}`],
    ];

    const request = requests[exec.scenario.iterationInTest % requests.length];
    const response = http.request(request[1], request[2], null, protectedHeaders(auth));

    check(response, {
      [`${request[0]} status is 200`]: (res) => res.status === 200,
      [`${request[0]} has response_message`]: (res) => {
        try {
          return typeof res.json("response_message") === "string";
        } catch (_) {
          return false;
        }
      },
    });

    authState = saveRotatedTokens(response, auth);
  });

  sleep(Number(__ENV.K6_SLEEP_SECONDS || 1));
}
