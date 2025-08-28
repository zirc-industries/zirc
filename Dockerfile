# Multi-stage Dockerfile for Zirc Language
# Build stage
FROM rust:1.75-slim as builder

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build the project
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 zirc

# Set working directory
WORKDIR /app

# Copy the built binaries from builder stage
COPY --from=builder /app/target/release/zirc-cli /usr/local/bin/
COPY --from=builder /app/target/release/zirc-repl /usr/local/bin/

# Copy examples
COPY examples/ ./examples/

# Change ownership
RUN chown -R zirc:zirc /app

# Switch to non-root user
USER zirc

# Default command is the REPL
CMD ["zirc-repl"]
