FROM rust:1.79-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev libssh2-1-dev zlib1g-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY apps/api/Cargo.toml ./apps/api/
COPY services/orchestrator/Cargo.toml ./services/orchestrator/
COPY services/worker/Cargo.toml ./services/worker/
COPY services/git-manager/Cargo.toml ./services/git-manager/
COPY services/gateway/Cargo.toml ./services/gateway/
COPY services/mcp-gateway/Cargo.toml ./services/mcp-gateway/
RUN for crate in apps/api services/orchestrator services/worker services/git-manager services/gateway services/mcp-gateway; do \
    mkdir -p $crate/src && echo 'fn main(){}' > $crate/src/main.rs; done
RUN cargo build --release --package koda-git-manager
COPY apps/api/src ./apps/api/src
COPY services/git-manager/src ./services/git-manager/src
RUN touch services/git-manager/src/main.rs && cargo build --release --package koda-git-manager

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssh2-1 git && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/koda-git-manager /usr/local/bin/
COPY services/git-manager/config /config
CMD ["koda-git-manager"]
