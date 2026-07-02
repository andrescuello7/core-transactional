# syntax=docker/dockerfile:1

FROM rust:1.81-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release --bin core-transactional
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
RUN mkdir -p /app/docs/data
COPY --from=builder /app/target/release/core-transactional /usr/local/bin/core-transactional
RUN useradd --create-home --uid 10001 appuser \
    && chown -R appuser:appuser /app
USER appuser

EXPOSE 8080
ENV RUST_LOG=info
CMD ["core-transactional"]
