FROM rust:1 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/auth-service /app/auth-service

EXPOSE 8083

ENV SERVER_PORT=8083
ENV RUST_LOG=auth_service=info,tower_http=info

CMD ["/app/auth-service"]
