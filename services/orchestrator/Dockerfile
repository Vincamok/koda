FROM rust:1.88-slim AS builder

# Limiter le parallélisme pour éviter l'OOM sur les petits runners
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
# Stub source files for dependency caching
RUN for crate in apps/api apps/cli services/orchestrator services/worker services/git-manager services/gateway services/mcp-gateway; do \
    mkdir -p $crate/src && echo 'fn main(){}' > $crate/src/main.rs; done
RUN cargo build --release --package koda-orchestrator
# Build with real sources
COPY apps/api/src ./apps/api/src
COPY services/orchestrator/src ./services/orchestrator/src
RUN touch services/orchestrator/src/main.rs && cargo build --release --package koda-orchestrator

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/koda-orchestrator /usr/local/bin/
COPY services/orchestrator/config /config
ENV CONFIG_PATH=/config
CMD ["koda-orchestrator"]
