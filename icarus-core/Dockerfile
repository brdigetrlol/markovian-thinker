# Multi-stage Dockerfile for Icarus MCP HTTP Server

# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/icarus-mcp-http.rs

# Build dependencies (cached layer)
RUN cargo build --release --bin icarus-mcp-http

# Remove dummy files
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual binary
RUN touch src/lib.rs && \
    cargo build --release --bin icarus-mcp-http

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 icarus

# Set working directory
WORKDIR /home/icarus

# Copy binary from builder
COPY --from=builder /app/target/release/icarus-mcp-http /usr/local/bin/

# Copy configuration (if exists)
COPY icarus-config.toml ./icarus-config.toml 2>/dev/null || true

# Set ownership
RUN chown -R icarus:icarus /home/icarus

# Switch to non-root user
USER icarus

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the server
CMD ["icarus-mcp-http", "3000"]
