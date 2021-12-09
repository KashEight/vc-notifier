FROM rust:1.55.0 AS builder

RUN cargo new --bin app
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src/ /app/src/

RUN cargo build --release

FROM gcr.io/distroless/cc

COPY --from=builder /app/target/release/vc-notifier /
CMD ["./vc-notifier"]
