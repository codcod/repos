# Build stage
FROM rust:1.90-alpine3.22 AS builder

WORKDIR /app

# Install build dependencies for Alpine
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    git \
    gcc

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the application with optimizations (already musl in Alpine)
RUN cargo build --release && \
    strip target/release/repos

# Runtime stage - use scratch for minimal size
FROM scratch

# Copy CA certificates for HTTPS
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the statically linked binary
COPY --from=builder /app/target/release/repos /repos

# Set the entrypoint
ENTRYPOINT ["/repos"]
CMD ["--help"]
