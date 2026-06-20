# Rusty Auth k6 Traffic Simulation

This directory contains the k6 traffic simulation for Rusty Auth.

## Files

```text
main.js       k6 entrypoint
.env.sample  local k6 environment template
```

Create your local k6 environment file when you need repeatable overrides:

```bash
cp k6-traffic-simulation/.env.sample k6-traffic-simulation/.env
```

Edit `k6-traffic-simulation/.env` with the values for your local run.

## Environment Source

k6 does not automatically load Rusty Auth's root `.env`, `.env.development`, `.env.staging`, or
`.env.production` files.

Those files belong to the application runtime. Consider checking the root [README.md](../README.md)
to learn more about them.

The k6 script reads only k6 process environment variables through `__ENV`. Source the dedicated k6
env file before running:

```bash
set -a
. ./k6-traffic-simulation/.env
set +a
k6 run k6-traffic-simulation/main.js
```

## Run

Start Rusty Auth and PostgreSQL first, then run:

```bash
set -a
. ./k6-traffic-simulation/.env
set +a
k6 run k6-traffic-simulation/main.js
```

## Current Flow

Each virtual user registers or reuses a unique test account, logs in, captures auth cookies and
tokens, then rotates through protected session, role, and permission routes. The script preserves
fresh `access_token`, `refresh_token`, and `session_id` values returned by protected responses, which
matches Rusty Auth's session-token rotation behavior.

## Data Safety

The script creates users in the target Rusty Auth database. Point `AUTH_BASE_URL` at a local,
staging, or disposable environment unless you intentionally want load-test accounts in that
database. Use `K6_RUN_ID` when you want predictable account prefixes for later cleanup.
