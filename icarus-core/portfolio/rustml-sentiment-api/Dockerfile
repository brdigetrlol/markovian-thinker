# Multi-stage Dockerfile for RustML Sentiment API

# Stage 1: Build
FROM rust:1.75-slim as builder

WORKDIR /usr/src/app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src
COPY examples ./examples
COPY tests ./tests

# Build the actual application
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 rustml

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/rustml-api /app/rustml-api

# Change ownership
RUN chown -R rustml:rustml /app

# Switch to non-root user
USER rustml

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV BIND_ADDRESS=0.0.0.0:3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/bin/sh", "-c", "curl -f http://localhost:3000/health || exit 1"]

# Run the application
CMD ["/app/rustml-api"]
