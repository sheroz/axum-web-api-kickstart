# Web samples using `axum`

[![build & test](https://github.com/sheroz/axum-web/actions/workflows/ci.yml/badge.svg)](https://github.com/sheroz/axum-web/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/github/license/sheroz/axum-web)](https://github.com/sheroz/axum-web/tree/main/LICENSE)

Components:

- web service
- redis / docker
- postgres / docker

## Run

```text
docker-compose up -d
cargo run
```

### pgAdmin4

link to the web app: [http://127.0.0.1:3000/](http://127.0.0.1:3000/)

link to the pgAdmin4: [http://127.0.0.1:9191/](http://127.0.0.1:9191/)

## Logging

Setting the `RUST_LOG` - logging level on the launch

```text
RUST_LOG=info,hyper=debug,axum_web=trace cargo run
```

## Project Stage

**Development**: this project is under development, you should not expect stability.
