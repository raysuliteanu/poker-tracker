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

# Runtime stage - use distroless for minimal image size and better security
FROM gcr.io/distroless/cc-debian13

WORKDIR /app

# Copy PostgreSQL client libraries and dependencies from builder
# Note: libssl and libcrypto are already in distroless base, not copied
COPY --from=builder /usr/lib/x86_64-linux-gnu/libpq.so.5* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libgssapi_krb5.so.2* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libldap.so.2* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libzstd.so.1* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libkrb5.so.3* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libk5crypto.so.3* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libcom_err.so.2* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libkrb5support.so.0* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/liblber.so.2* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libsasl2.so.2* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libkeyutils.so.1* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libresolv.so.2* /usr/lib/x86_64-linux-gnu/

# Copy the binary from builder
COPY --from=builder /app/target/release/poker_tracker /app/poker_tracker
COPY --from=builder /app/migrations /app/migrations

# Set environment variables
ENV RUST_LOG=info

# Expose port
EXPOSE 8080

# Run the application
CMD ["/app/poker_tracker"]
