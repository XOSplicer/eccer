FROM rust:1.70.0-bookworm as builder
WORKDIR /usr/src/eccer
RUN apt-get update \
    && apt-get install -y protobuf-compiler libssl-dev libssl3 libcurl4 \
    && rm -rf /var/lib/apt/lists/*
RUN cargo init .
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo check
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y libssl3 libcurl4 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/eccer /usr/local/bin/eccer
RUN eccer --version
ENTRYPOINT [ "eccer" ]