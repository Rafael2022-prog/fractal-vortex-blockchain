# Multi-stage build for Fractal-Vortex Chain
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY tests ./tests

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 fractal

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/fractal-vortex-chain /usr/local/bin/

# Create data directory
RUN mkdir -p /data && chown fractal:fractal /data

# Switch to non-root user
USER fractal

# Expose ports
EXPOSE 30333 9933 9944

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD /usr/local/bin/fractal-vortex-chain info || exit 1

# Default command
CMD ["fractal-vortex-chain", "start"]