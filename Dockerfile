FROM rust:1.68.2-buster as builder
WORKDIR /usr/src/eccer
RUN apt-get update && apt-get install -y protobuf-compiler libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo init .
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo check
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/eccer /usr/local/bin/eccer
ENTRYPOINT [ "eccer" ]