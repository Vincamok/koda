FROM rust:1.88-slim AS builder
ENV CARGO_BUILD_JOBS=1
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
COPY services/worker/src ./services/worker/src

RUN for crate in apps/cli services/orchestrator services/git-manager services/gateway services/mcp-gateway; do \
    mkdir -p $crate/src && echo 'fn main(){}' > $crate/src/main.rs; done && \
    cargo build --release --package koda-worker && \
    cp target/release/koda-worker /koda-worker && \
    rm -rf target /usr/local/rustup /usr/local/cargo

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /koda-worker /usr/local/bin/koda-worker
COPY services/worker/config /config
CMD ["koda-worker"]
