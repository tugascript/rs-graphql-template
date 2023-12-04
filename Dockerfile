# Build Stage
FROM rust:1.74.0 AS builder
WORKDIR /app

# Copy over the manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# Copy over the source code
COPY ./src ./src
COPY ./entities ./entities
COPY ./migrations ./migrations

# Build for release
RUN cargo build --release --bin app

# Final Stage
FROM debian:bookworm-slim AS runtime

WORKDIR /usr/src/bin

# Install SSL Certificates
RUN apt-get update -y \
    && apt-get install -y \
    ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Appease dotenvy
RUN touch .env

# Copy the binary and extra files from the builder stage
COPY --from=builder /app/target/release/app app

# Set the binary as the entrypoint of the container
ENTRYPOINT ["./app"]
