FROM rust:1.75

RUN mkdir ./app

COPY ./src ./app/src
COPY ./tests ./app/tests
COPY ./.env ./app/.env
COPY ./.env_test ./app/.env_test
COPY ./Cargo.toml ./app/Cargo.toml

EXPOSE 3000
