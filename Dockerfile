# syntax=docker/dockerfile:1

# Build stage
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a dummy project to cache dependencies
RUN cargo new --bin spoons-api
WORKDIR /app/spoons-api

# Copy dependency files first for caching
COPY Cargo.toml Cargo.lock* ./

# Build dependencies only (this layer will be cached)
RUN cargo build --release && rm -rf src target/release/spoons-api*

# Copy actual source code
COPY src ./src

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false appuser

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/spoons-api/target/release/spoons-api /app/spoons-api

# Copy default config
COPY config.yaml /app/config.yaml

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 4000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/spoons-api", "start", "--help"]

# Run the binary
ENTRYPOINT ["/app/spoons-api"]
CMD ["start", "--config", "/app/config.yaml"]
