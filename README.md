# Getting started with Web Service in Rust

[![build & test](https://github.com/sheroz/axum-web/actions/workflows/ci.yml/badge.svg)](https://github.com/sheroz/axum-web/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/github/license/sheroz/axum-web)](https://github.com/sheroz/axum-web/tree/main/LICENSE)

The project covers:

- Web service based on `axum`
  - routing
  - CORS settings
  - error handling
  - graceful shutdown (!!! does not work after upgrading to axum 0.7 and hyper 1.0 !!!)
- Using `PostgreSQL`database with `sqlx` driver
  - migrations
  - async connection pooling
  - async CRUD operations
- Using `Redis`
  - async operations
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

link to the web app: [http://127.0.0.1:3000/](http://127.0.0.1:3000/)

## Logging

Setting the `RUST_LOG` - logging level on the launch

```text
RUST_LOG=info,hyper=debug,axum_web=trace cargo run
```

## Project Stage

**Development**: this project is under development, you should not expect stability.
