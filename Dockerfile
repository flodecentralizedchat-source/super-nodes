# Multi-stage Docker build for SuperNode
# Optimized for minimal image size and security

# ============================================================
# Stage 1: Build
# ============================================================
FROM rust:1.88-slim-bookworm as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Copy all source and manifests
COPY . .

# Build in release mode
RUN cargo build --release

# ============================================================
# Stage 2: Runtime
# ============================================================
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && adduser --disabled-password --gecos '' supernode

# Copy binary from builder
COPY --from=builder /app/target/release/supernode /usr/local/bin/supernode

# Set permissions
RUN chmod +x /usr/local/bin/supernode \
    && chown supernode:supernode /usr/local/bin/supernode

# Switch to non-root user
USER supernode

# Expose ports
# 9000 - Main server (QUIC/WebSocket)
# 3000 - HTTP API
# 9090 - Prometheus metrics
EXPOSE 9000 3000 9090

# Health check (uses API_PORT env var, defaults to 3000)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${API_PORT:-3000}/health || exit 1

# Run the application
CMD ["supernode"]
