FROM rust:1.86-slim as builder

ARG DATABASE_URL

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev pkg-config openssl \
    && cargo install cargo-watch \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/authgate

# Copy manifests and source
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

ENV SQLX_OFFLINE=true

# Prepare sqlx macros for offline builds
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/authgate/target \
    cargo install sqlx-cli --no-default-features --features postgres && \
    cargo sqlx prepare -- --lib && \
    cargo build --release && \
    cp target/release/authgate .

# Runtime container
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/authgate/authgate /app/
COPY --from=builder /usr/src/authgate/.sqlx /app/.sqlx

EXPOSE 4181

CMD ["/app/authgate"]
