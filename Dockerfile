FROM rust:latest as builder

RUN USER=root cargo new --bin bablfsh
WORKDIR /bablfsh

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./config ./config

RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest
COPY --from=builder /bablfsh/target/x86_64-unknown-linux-musl/release/bablfsh /usr/local/bin/bablfsh
COPY ./entrypoint.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]