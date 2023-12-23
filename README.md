# Getting started with Web Services in Rust

(using `axum`, `sqlx`, `postgres`, and `redis`)

[![build & test](https://github.com/sheroz/axum-web/actions/workflows/ci.yml/badge.svg)](https://github.com/sheroz/axum-web/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/github/license/sheroz/axum-web)](https://github.com/sheroz/axum-web/tree/main/LICENSE)

The project covers:

- REST API based on `axum`
  - routing
  - CORS settings
  - error handling
  - graceful shutdown (!!! does not work since upgrading to axum 0.7 and hyper 1.0 !!!)
- Using `PostgreSQL`database with `sqlx` driver
  - migrations
  - async connection pooling
  - async CRUD operations
- Using `Redis`
  - async operations
- Authentication based on `JWT` 
- `.env` based configuration parsing
- `tracing` based logs
- `docker-compose` configuration
  - `redis` service
  - `postgres` service

## Run

```text
docker-compose up -d
cargo run
```

REST API test samples: [/tests/endpoints.http](/tests/endpoints.http)

## Logging

Setting the `RUST_LOG` - logging level on the launch

```text
RUST_LOG=info,hyper=debug,axum_web=trace cargo run
```

## Project Stage

**Development**: this project is under development, you should not expect stability yet.
