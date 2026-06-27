FROM rust:1.79-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
# Only this crate has its own workspace — build standalone
COPY services/mcp-gateway ./
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mcp-gateway /usr/local/bin/
COPY services/mcp-gateway/config /config
CMD ["mcp-gateway"]
