# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.92
ARG APP_NAME=auth

FROM rust:${RUST_VERSION}-bookworm AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG APP_NAME=auth
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked --bin ${APP_NAME}

FROM debian:bookworm-slim AS runtime
ARG APP_NAME=auth

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /usr/sbin/nologin appuser
WORKDIR /app

COPY --from=builder /app/target/release/${APP_NAME} /usr/local/bin/${APP_NAME}
COPY config ./config

ENV APP__ENV=production
ENV APP__SERVER__HOST=0.0.0.0
ENV APP__SERVER__PORT=8000

EXPOSE 8000

USER appuser
CMD ["auth"]
