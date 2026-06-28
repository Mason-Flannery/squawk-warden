FROM rust:1.96-slim AS builder

WORKDIR /usr/src/app

COPY server ./server
COPY shared ./shared
COPY embedded ./embedded
COPY Cargo.toml Cargo.lock ./


WORKDIR /usr/src/app/server
RUN cargo build --release --package server

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /usr/src/app/target/release/server /app/server
EXPOSE 3000
CMD ["./server"]