FROM rust:1.79-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY services/mcp-gateway ./
RUN cargo build --release && \
    cp target/release/mcp-gateway /mcp-gateway && \
    rm -rf target

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /mcp-gateway /usr/local/bin/mcp-gateway
COPY services/mcp-gateway/config /config
CMD ["mcp-gateway"]
