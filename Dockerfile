FROM rust:1.97.0-slim AS builder
WORKDIR /app

ARG PROJECT_NAME
ARG PROJECT_VERSION

RUN cargo install cargo-set --locked

COPY . .

RUN cargo set package.name "abyss-${PROJECT_NAME}" && \
    cargo set package.version "${PROJECT_VERSION}" && \
    cargo build --release

FROM scratch
ARG PROJECT_NAME

COPY --from=builder /app/target/release/abyss-${PROJECT_NAME} /
