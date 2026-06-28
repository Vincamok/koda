FROM rust:1.79-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Fetch dependencies first (layer cache)
COPY Cargo.toml Cargo.lock ./
COPY apps/api/Cargo.toml apps/api/
COPY apps/cli/Cargo.toml apps/cli/
COPY services/orchestrator/Cargo.toml services/orchestrator/
COPY services/worker/Cargo.toml services/worker/
COPY services/git-manager/Cargo.toml services/git-manager/
COPY services/gateway/Cargo.toml services/gateway/
COPY services/mcp-gateway/Cargo.toml services/mcp-gateway/

# Stub src so cargo can resolve workspace without full source
RUN mkdir -p apps/api/src apps/cli/src services/orchestrator/src services/worker/src \
    services/git-manager/src services/gateway/src services/mcp-gateway/src && \
    echo "fn main(){}" | tee \
      apps/api/src/main.rs apps/cli/src/main.rs \
      services/orchestrator/src/main.rs \
      services/worker/src/main.rs services/git-manager/src/main.rs \
      services/gateway/src/main.rs services/mcp-gateway/src/main.rs && \
    touch apps/api/src/lib.rs

RUN cargo build --release --package koda-api 2>/dev/null || true

# Real source
COPY apps/api/src apps/api/src
COPY infra/migrations infra/migrations

RUN cargo build --release --package koda-api

# ── Runtime ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash koda
USER koda

COPY --from=builder /build/target/release/koda-api /usr/local/bin/koda-api
COPY --from=builder /build/apps/api/config /app/config

WORKDIR /app

HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:8080/api/v1/admin/health || exit 1

EXPOSE 8080

CMD ["koda-api"]
