# Build stage
FROM rust:1.91-slim AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/cache/debconf/* \
    && rm -rf /var/lib/dpkg/*-old

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY diesel.toml ./

# Build the application
RUN cargo build --release

# Runtime stage - use trixie to match rust:1.91-slim glibc version
FROM debian:trixie-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates \
    wget \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/cache/debconf/* \
    && rm -rf /var/lib/dpkg/*-old \
    && rm -rf /tmp/* \
    && rm -rf /var/tmp/*

# Copy the binary from builder
COPY --from=builder /app/target/release/poker_tracker /app/poker_tracker
COPY --from=builder /app/migrations /app/migrations

# Set environment variables
ENV RUST_LOG=info

# Expose port
EXPOSE 8080

# Run the application
CMD ["/app/poker_tracker"]
