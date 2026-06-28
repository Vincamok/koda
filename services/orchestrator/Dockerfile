FROM rust:1.88-slim AS builder
ENV CARGO_BUILD_JOBS=1
ENV SQLX_OFFLINE=true
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY apps/api/Cargo.toml ./apps/api/
COPY apps/cli/Cargo.toml ./apps/cli/
COPY services/orchestrator/Cargo.toml ./services/orchestrator/
COPY services/worker/Cargo.toml ./services/worker/
COPY services/git-manager/Cargo.toml ./services/git-manager/
COPY services/gateway/Cargo.toml ./services/gateway/
COPY services/mcp-gateway/Cargo.toml ./services/mcp-gateway/

COPY apps/api/src ./apps/api/src
COPY services/orchestrator/src ./services/orchestrator/src
COPY .sqlx ./.sqlx

RUN for crate in apps/cli services/worker services/git-manager services/gateway services/mcp-gateway; do \
    mkdir -p $crate/src && echo 'fn main(){}' > $crate/src/main.rs; done && \
    cargo build --release --package koda-orchestrator && \
    cp target/release/koda-orchestrator /koda-orchestrator && \
    rm -rf target /usr/local/rustup /usr/local/cargo

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /koda-orchestrator /usr/local/bin/koda-orchestrator
COPY services/orchestrator/config /config
ENV CONFIG_PATH=/config
CMD ["koda-orchestrator"]
