# Web Service Sample

[![build & test](https://github.com/sheroz/axum-web/actions/workflows/ci.yml/badge.svg)](https://github.com/sheroz/axum-web/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/github/license/sheroz/axum-web)](https://github.com/sheroz/axum-web/tree/main/LICENSE)

The project covers:

- web service based on `axum`
  - routing
  - error handling
  - graceful shutdown
- connecting to `redis`
- connecting to `postgres`
  - connection pooling
  - migrations
- parsing `.env` based configuration
- `tracing` based logs
- `docker-compose` configuration
  - `redis` service
  - `postgres` service
  - `pgAdmin` service
    - disabled login dialog (using `PGADMIN_CONFIG_SERVER_MODE` param)

## Run

```text
docker-compose up -d
cargo run
```

### pgAdmin4

link to the web app: [http://127.0.0.1:3000/](http://127.0.0.1:3000/)

link to the pgAdmin4: [http://127.0.0.1:5050/](http://127.0.0.1:5050/)

## Logging

Setting the `RUST_LOG` - logging level on the launch

```text
RUST_LOG=info,hyper=debug,axum_web=trace cargo run
```

## Project Stage

**Development**: this project is under development, you should not expect stability.
