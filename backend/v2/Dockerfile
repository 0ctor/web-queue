FROM rust:1-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY .cargo ./.cargo

ENV SQLX_OFFLINE=true

RUN cargo build --release

FROM debian:bookworm-slim AS production

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 appuser

WORKDIR /app

COPY --from=builder /app/target/release/web-queue-api /app/web-queue-api

USER appuser

ENV BACKEND_PORT=4545 \
    RUST_LOG=info \
    TZ=America/Sao_Paulo \
    LOKI_JOB=web-queue

EXPOSE 4545

HEALTHCHECK --interval=30s --timeout=5s --start-period=40s --retries=5 \
  CMD curl -fsS http://127.0.0.1:4545/health/live >/dev/null || exit 1

CMD ["/app/web-queue-api"]
