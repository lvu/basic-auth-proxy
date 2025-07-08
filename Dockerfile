FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin basic-auth-proxy

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