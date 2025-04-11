# ---- Builder Stage ----
FROM rust:1.70-slim AS builder
# Using slim-bullseye as base to minimize image size while providing build essentials

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create dummy src/main.rs for dependency caching
RUN mkdir src && echo "fn main(){}" > src/main.rs

# Build dependencies only first to leverage Docker cache
RUN cargo build --release --target-dir /app/target_dep

# Build application
COPY src ./src

# Remove dummy binary hash and build the actual application
RUN rm -f /app/target_dep/release/deps/switchboard* 
RUN cargo build --release --target-dir /app/target_app

# ---- Final Stage ----
FROM debian:12-slim AS final

# Install only the necessary runtime dependencies
# ca-certificates is required for TLS verification with rustls
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target_app/release/switchboard /app/switchboard

# Expose the default port
EXPOSE 8080

# Set the entrypoint
ENTRYPOINT ["/app/switchboard"]

# Environment variables are expected to be provided at runtime via:
# docker run -e PORT=8080 -e ANTHROPIC_API_KEY=your-key -e LOG_FORMAT=pretty -e LOG_LEVEL=info