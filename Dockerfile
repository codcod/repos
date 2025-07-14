# Build stage
FROM rust:1.88-alpine3.22 as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:12.8-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false repos

# Copy the binary from builder stage
COPY --from=builder /app/target/release/repos /usr/local/bin/repos

# Set the user
USER repos

# Set the working directory
WORKDIR /workspace

# Set the entrypoint
ENTRYPOINT ["repos"]
CMD ["--help"]
