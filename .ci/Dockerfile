# syntax=docker/dockerfile:1
FROM rust:1.88-slim@sha256:38bc5a86d998772d4aec2348656ed21438d20fcdce2795b56ca434cf21430d89 AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    libclang-dev \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

ARG TARGET=x86_64-unknown-linux-musl

RUN rustup target add $TARGET \
    && rustup component add clippy \
    && cargo install tokei --version 14.0.0 --features all --root /usr/local

WORKDIR /app

COPY Cargo.toml Cargo.lock* build.rs ./
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --target $TARGET --locked \
    && rm -rf src

ARG VERSION=0.0.0-0
ENV VERSION=$VERSION \
    RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes"

COPY src/ src/
COPY template.json ./

RUN tokei ./src -o json > /app/codebase.json

RUN cargo clippy --target $TARGET --all-targets --locked -- -D warnings \
    && cargo build --release --target $TARGET --locked \
    && strip /app/target/$TARGET/release/abyss-snaps \
    && cp /app/target/$TARGET/release/abyss-snaps /app/abyss-snaps

FROM scratch AS exporter
COPY --from=builder /app/abyss-snaps /
COPY --from=builder /app/codebase.json /
