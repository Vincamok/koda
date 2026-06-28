FROM rust:1.88-slim AS builder
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
RUN for crate in apps/api apps/cli services/orchestrator services/worker services/git-manager services/gateway services/mcp-gateway; do \
    mkdir -p $crate/src && echo 'fn main(){}' > $crate/src/main.rs; done
RUN cargo build --release --package koda-worker
COPY apps/api/src ./apps/api/src
COPY services/worker/src ./services/worker/src
RUN touch services/worker/src/main.rs && cargo build --release --package koda-worker

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/koda-worker /usr/local/bin/
COPY services/worker/config /config
CMD ["koda-worker"]
