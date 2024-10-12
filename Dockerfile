FROM rust:1.181

COPY ./src ./src
COPY ./tests ./tests
COPY ./.env ./.env
COPY ./.env_test ./.env_test
COPY ./.env_test_docker ./.env_test_docker
COPY ./Cargo.toml ./Cargo.toml

EXPOSE 3000
