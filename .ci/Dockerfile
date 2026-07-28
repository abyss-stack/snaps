# syntax=docker/dockerfile:1
FROM rust:1.88-slim AS builder

RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl && \
    rustup component add clippy

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    rm -rf src

COPY src/ src/
COPY template.json ./

ARG VERSION
ENV VERSION=$VERSION

ENV RUSTFLAGS="-C target-feature=+crt-static"

RUN cargo clippy --target x86_64-unknown-linux-musl --all-targets -- -D warnings

RUN cargo build --release --target x86_64-unknown-linux-musl && \
    cp /app/target/x86_64-unknown-linux-musl/release/abyss-snaps /app/abyss-snaps

FROM scratch AS exporter
COPY --from=builder /app/abyss-snaps /
