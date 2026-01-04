# =============================================================================
# API Gateway Dockerfile - Optimized Multi-Stage Build
# Uses cargo-chef for dependency caching to speed up rebuilds
# =============================================================================

# Stage 1: Planner - Generate recipe for dependencies
FROM lukemathwalker/cargo-chef:latest-rust-1 AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - Build dependencies (cached) then app
FROM lukemathwalker/cargo-chef:latest-rust-1 AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Enable SQLx offline mode (uses .sqlx cache)
ENV SQLX_OFFLINE=true

# Copy recipe and build dependencies first (cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy source code and build application
COPY . .
RUN cargo build --release --bin api-gateway

# Stage 3: Runtime - Minimal production image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false apigateway

# Copy binary from builder
COPY --from=builder /app/target/release/api-gateway /usr/local/bin/

# Run as non-root user
USER apigateway

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:4000/health || exit 1

EXPOSE 4000

CMD ["api-gateway"]
