# Build stage
FROM rust:1.88-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false app

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/basic-auth-proxy /app/basic-auth-proxy

# Change ownership to non-root user
RUN chown app:app /app/basic-auth-proxy

USER app

EXPOSE 8080

ENTRYPOINT ["/app/basic-auth-proxy"]