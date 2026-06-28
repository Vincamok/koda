FROM rust:1.88-slim AS builder
ENV CARGO_BUILD_JOBS=1
ENV SQLX_OFFLINE=true
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY apps/api/Cargo.toml apps/api/
COPY apps/cli/Cargo.toml apps/cli/
COPY services/orchestrator/Cargo.toml services/orchestrator/
COPY services/worker/Cargo.toml services/worker/
COPY services/git-manager/Cargo.toml services/git-manager/
COPY services/gateway/Cargo.toml services/gateway/
COPY services/mcp-gateway/Cargo.toml services/mcp-gateway/

COPY apps/api/src apps/api/src
COPY infra/migrations infra/migrations
COPY .sqlx .sqlx

RUN for crate in apps/cli services/orchestrator services/worker services/git-manager services/gateway services/mcp-gateway; do \
    mkdir -p $crate/src && echo 'fn main(){}' > $crate/src/main.rs; done && \
    cargo build --release --package koda-api && \
    cp target/release/koda-api /koda-api && \
    rm -rf target /usr/local/rustup /usr/local/cargo

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -ms /bin/bash koda
USER koda
COPY --from=builder /koda-api /usr/local/bin/koda-api
COPY --from=builder /build/apps/api/config /app/config
WORKDIR /app
HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:8080/api/v1/admin/health || exit 1
EXPOSE 8080
CMD ["koda-api"]
