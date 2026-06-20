# Contributing

Thanks for your interest in wanting to help improve this project.

Kindly follow the guide below to get started.

## Setup

Install the root developer tooling:

```bash
bun install
```

This installs Husky hooks and the repository formatting/commit tooling. The root is intentionally
not a Rust workspace; service checks run inside each service under `microservices/`.

## Local Checks

Run repository formatting checks:

```bash
bun run format:check
```

Run Rust quality checks for all Rust services:

```bash
bun run services:check
```

Run service tests:

```bash
bun run services:test
```

Services with database-backed controller tests require their database to be reachable. When the DB
is not reachable locally, the pre-push script runs non-DB tests and tells you which DB-backed tests
were skipped. CI always provisions PostgreSQL for the auth service and runs the full test suite.

## Commit Messages

Use Conventional Commits:

```text
feat: add service health check
fix: correct auth migration setup
docs: update integration guide
```

## Pull Requests

Keep changes scoped. If a change affects a service contract, update that service README and tests in
the same pull request.

**Ensure to strictly follow the PR template/guide to ensure your pull requests are not delayed or
turned down**.
