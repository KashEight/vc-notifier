FROM rust:1.55.0 AS builder

WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc

COPY --from=builder /app/target/release/vc-notifier /
CMD ["./vc-notifier"]
