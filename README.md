# Getting started with Web Services in Rust

Kick-start template for building REST API Web service in Rust using `axum`, `JWT`, `SQLx`, `PostgreSQL`, and `Redis`

[![build & test](https://github.com/sheroz/axum-web/actions/workflows/ci.yml/badge.svg)](https://github.com/sheroz/axum-web/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/github/license/sheroz/axum-web)](https://github.com/sheroz/axum-web/tree/main/LICENSE)

The project covers:

- REST API based on `axum`
  - routing
  - api versioning
  - CORS settings
  - error handling
  - graceful shutdown
- `JWT` based authentication & authorization
  - access tokens
  - refresh tokens
  - setting the tokens expiry time (based on configuration)
  - refresh tokens rotation technique
  - revoking the issued tokens by using `Redis` (based on configuration)
    - revoke all tokens issued until the current time
    - revoke tokens belonging to a user issued until the current time
    - cleanup of revoked tokens
- `PostgreSQL`database with `sqlx` driver
  - database migrations
  - async connection pooling
  - async CRUD operations
- `Redis` in-memory storage
  - async operations
- `.env` based configuration parsing
- `tracing` based logs
- `docker-compose` configuration
  - `Redis` service
  - `PostgreSQL` service
- API tests
- GitHub CI configuration
  - `docker` based end-to-end tests

## Run

```text
docker-compose up -d
cargo run
```

run in test configuration

```text
ENV_TEST=1 cargo run
```

REST API parameters: [tests/endpoints.http](/tests/endpoints.http)

## Logging

Setting the `RUST_LOG` - logging level on the launch

```text
RUST_LOG=info,hyper=debug,axum_web=trace cargo run
```

## Project Stage

**Development**: this project is under development, you should not expect stability yet.
