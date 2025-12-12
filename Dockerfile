FROM rust:1.91-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    perl \
    make \
    cmake \
    g++ \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./

COPY shared ./shared

COPY admin ./admin

COPY gateway ./gateway

RUN cargo build --workspace --release

FROM debian:trixie-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/cirith-admin /app/cirith-admin

COPY --from=builder /app/target/release/cirith-gateway /app/cirith-gateway

RUN mkdir -p /app/data

COPY config.yml /app/config.yml

EXPOSE 3000 6191