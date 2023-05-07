# Use cargo chef to cache dependencies
FROM lukemathwalker/cargo-chef:latest-rust-1.65.0 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release --bin srv-rs

# Runtime stage
FROM debian:bullseye-slim AS runtime

WORKDIR /app

# Install OpenSSL and ca-certificates
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/srv-rs srv-rs
COPY .env.prod .env

ENTRYPOINT ["./srv-rs"]